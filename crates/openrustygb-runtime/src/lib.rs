#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, channel, sync_channel};
use std::thread::{self, JoinHandle};

use openrustygb_domain::{ControllerRef, Rgb8};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct OperationId(u64);

impl OperationId {
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandOutcome {
    Applied {
        operation: OperationId,
    },
    Superseded {
        operation: OperationId,
        by: OperationId,
    },
    Failed {
        operation: OperationId,
        error: String,
    },
}

#[derive(Debug)]
pub struct CommandTicket {
    operation: OperationId,
    completion: Receiver<CommandOutcome>,
}

impl CommandTicket {
    #[must_use]
    pub const fn operation(&self) -> OperationId {
        self.operation
    }

    /// Waits for the controller actor to finish or supersede this operation.
    ///
    /// # Errors
    ///
    /// Returns [`WaitError`] if the actor exits without reporting an outcome.
    pub fn wait(self) -> Result<CommandOutcome, WaitError> {
        self.completion.recv().map_err(|_| WaitError)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaitError;

impl fmt::Display for WaitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("controller actor closed before reporting command completion")
    }
}

impl std::error::Error for WaitError {}

#[derive(Debug)]
pub struct StartError(std::io::Error);

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not start controller actor: {}", self.0)
    }
}

impl std::error::Error for StartError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubmitError {
    StaleController,
    QueueFull,
    ControllerClosed,
}

impl fmt::Display for SubmitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaleController => f.write_str("controller reference is stale"),
            Self::QueueFull => f.write_str("controller command queue is full"),
            Self::ControllerClosed => f.write_str("controller is closed"),
        }
    }
}

impl std::error::Error for SubmitError {}

pub trait ControllerBackend: Send + 'static {
    type Barrier: Send + 'static;
    type Error: std::error::Error + Send + Sync + 'static;

    /// Applies the latest pending coalescible whole-device color.
    ///
    /// # Errors
    ///
    /// Returns the backend error when the device operation fails.
    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error>;

    /// Applies a non-coalescible command after all preceding colors complete.
    ///
    /// # Errors
    ///
    /// Returns the backend error when the barrier operation fails.
    fn apply_barrier(&mut self, barrier: Self::Barrier) -> Result<(), Self::Error>;

    /// Stops the backend before the actor releases its transport.
    ///
    /// # Errors
    ///
    /// Returns the backend error when teardown cannot complete cleanly.
    fn shutdown(&mut self) -> Result<(), Self::Error>;
}

#[derive(Debug)]
enum Operation<B> {
    WholeColor(Rgb8),
    Barrier(B),
}

#[derive(Debug)]
struct Envelope<B> {
    id: OperationId,
    operation: Operation<B>,
    completion: std::sync::mpsc::Sender<CommandOutcome>,
}

#[derive(Debug)]
enum Message<B> {
    Command(Envelope<B>),
    Shutdown(std::sync::mpsc::Sender<()>),
}

#[derive(Debug)]
pub struct ControllerActor<B: ControllerBackend> {
    target: ControllerRef,
    next_operation: AtomicU64,
    ingress: SyncSender<Message<B::Barrier>>,
    worker: Option<JoinHandle<()>>,
}

impl<B: ControllerBackend> ControllerActor<B> {
    /// Starts a bounded, single-writer controller actor.
    ///
    /// # Errors
    ///
    /// Returns [`StartError`] if the operating system cannot create the worker thread.
    pub fn start(
        target: ControllerRef,
        backend: B,
        queue_capacity: usize,
    ) -> Result<Self, StartError> {
        let (ingress, receiver) = sync_channel(queue_capacity.max(1));
        let worker = thread::Builder::new()
            .name(format!("controller-actor-{:?}", target.id))
            .spawn(move || run_actor(backend, &receiver))
            .map_err(StartError)?;
        Ok(Self {
            target,
            next_operation: AtomicU64::new(1),
            ingress,
            worker: Some(worker),
        })
    }

    /// Submits a coalescible whole-device color command.
    ///
    /// # Errors
    ///
    /// Returns [`SubmitError`] for a stale target, full queue, or retired actor.
    pub fn submit_whole_color(
        &self,
        target: ControllerRef,
        color: Rgb8,
    ) -> Result<CommandTicket, SubmitError> {
        self.submit(target, Operation::WholeColor(color))
    }

    /// Submits a non-coalescible barrier command.
    ///
    /// # Errors
    ///
    /// Returns [`SubmitError`] for a stale target, full queue, or retired actor.
    pub fn submit_barrier(
        &self,
        target: ControllerRef,
        barrier: B::Barrier,
    ) -> Result<CommandTicket, SubmitError> {
        self.submit(target, Operation::Barrier(barrier))
    }

    fn submit(
        &self,
        target: ControllerRef,
        operation: Operation<B::Barrier>,
    ) -> Result<CommandTicket, SubmitError> {
        if target != self.target {
            return Err(SubmitError::StaleController);
        }
        let id = OperationId(self.next_operation.fetch_add(1, Ordering::Relaxed));
        let (completion, receiver) = channel();
        let envelope = Envelope {
            id,
            operation,
            completion,
        };
        self.ingress
            .try_send(Message::Command(envelope))
            .map_err(|error| match error {
                TrySendError::Full(_) => SubmitError::QueueFull,
                TrySendError::Disconnected(_) => SubmitError::ControllerClosed,
            })?;
        Ok(CommandTicket {
            operation: id,
            completion: receiver,
        })
    }

    /// Closes admission, drains accepted commands, stops the backend, and joins its worker.
    ///
    /// # Errors
    ///
    /// Returns [`WaitError`] if the worker exits before acknowledging shutdown.
    pub fn shutdown(mut self) -> Result<(), WaitError> {
        let (acknowledge, receiver) = channel();
        self.ingress
            .send(Message::Shutdown(acknowledge))
            .map_err(|_| WaitError)?;
        receiver.recv().map_err(|_| WaitError)?;
        let Some(worker) = self.worker.take() else {
            return Err(WaitError);
        };
        if worker.join().is_err() {
            return Err(WaitError);
        }
        Ok(())
    }
}

fn run_actor<B: ControllerBackend>(mut backend: B, receiver: &Receiver<Message<B::Barrier>>) {
    while let Ok(first) = receiver.recv() {
        let mut batch = VecDeque::from([first]);
        while let Ok(message) = receiver.try_recv() {
            batch.push_back(message);
        }
        if process_batch(&mut backend, batch) {
            return;
        }
    }
    let _ = backend.shutdown();
}

fn process_batch<B: ControllerBackend>(
    backend: &mut B,
    mut batch: VecDeque<Message<B::Barrier>>,
) -> bool {
    while let Some(message) = batch.pop_front() {
        match message {
            Message::Command(mut envelope) => {
                if matches!(envelope.operation, Operation::WholeColor(_)) {
                    while matches!(
                        batch.front(),
                        Some(Message::Command(Envelope {
                            operation: Operation::WholeColor(_),
                            ..
                        }))
                    ) {
                        let Message::Command(next) =
                            batch.pop_front().expect("front was a command")
                        else {
                            unreachable!("front pattern checked above")
                        };
                        let _ = envelope.completion.send(CommandOutcome::Superseded {
                            operation: envelope.id,
                            by: next.id,
                        });
                        envelope = next;
                    }
                }
                apply(backend, envelope);
            }
            Message::Shutdown(acknowledge) => {
                let _ = backend.shutdown();
                let _ = acknowledge.send(());
                return true;
            }
        }
    }
    false
}

fn apply<B: ControllerBackend>(backend: &mut B, envelope: Envelope<B::Barrier>) {
    let result = match envelope.operation {
        Operation::WholeColor(color) => backend.apply_whole_color(color),
        Operation::Barrier(barrier) => backend.apply_barrier(barrier),
    };
    let outcome = match result {
        Ok(()) => CommandOutcome::Applied {
            operation: envelope.id,
        },
        Err(error) => CommandOutcome::Failed {
            operation: envelope.id,
            error: error.to_string(),
        },
    };
    let _ = envelope.completion.send(outcome);
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::num::{NonZeroU32, NonZeroU64};
    use std::sync::{Arc, Mutex};

    use openrustygb_domain::{ControllerId, Incarnation};

    use super::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Barrier {
        Save,
    }

    #[derive(Debug)]
    struct FakeBackend {
        log: Arc<Mutex<Vec<String>>>,
    }

    impl ControllerBackend for FakeBackend {
        type Barrier = Barrier;
        type Error = Infallible;

        fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
            self.log.lock().unwrap().push(format!("color:{}", color.r));
            Ok(())
        }

        fn apply_barrier(&mut self, barrier: Self::Barrier) -> Result<(), Self::Error> {
            self.log.lock().unwrap().push(format!("{barrier:?}"));
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), Self::Error> {
            self.log.lock().unwrap().push("shutdown".into());
            Ok(())
        }
    }

    fn reference(incarnation: u32) -> ControllerRef {
        ControllerRef {
            id: ControllerId::new(NonZeroU64::new(7).unwrap()),
            incarnation: Incarnation::new(NonZeroU32::new(incarnation).unwrap()),
        }
    }

    #[test]
    fn rejects_a_stale_incarnation_before_enqueue() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let actor = ControllerActor::start(reference(2), FakeBackend { log }, 8).unwrap();
        assert!(matches!(
            actor.submit_whole_color(reference(1), Rgb8::new(1, 2, 3)),
            Err(SubmitError::StaleController)
        ));
        actor.shutdown().unwrap();
    }

    #[test]
    fn barriers_preserve_hardware_order_and_shutdown_joins_backend() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let actor = ControllerActor::start(
            reference(1),
            FakeBackend {
                log: Arc::clone(&log),
            },
            8,
        )
        .unwrap();

        let first = actor
            .submit_whole_color(reference(1), Rgb8::new(10, 0, 0))
            .unwrap();
        let save = actor.submit_barrier(reference(1), Barrier::Save).unwrap();
        let last = actor
            .submit_whole_color(reference(1), Rgb8::new(20, 0, 0))
            .unwrap();
        assert!(matches!(
            first.wait().unwrap(),
            CommandOutcome::Applied { .. }
        ));
        assert!(matches!(
            save.wait().unwrap(),
            CommandOutcome::Applied { .. }
        ));
        assert!(matches!(
            last.wait().unwrap(),
            CommandOutcome::Applied { .. }
        ));
        actor.shutdown().unwrap();

        assert_eq!(
            *log.lock().unwrap(),
            ["color:10", "Save", "color:20", "shutdown"]
        );
    }

    #[test]
    fn adjacent_whole_colors_coalesce_and_report_supersession() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut backend = FakeBackend {
            log: Arc::clone(&log),
        };
        let (first_sender, first_receiver) = channel();
        let (last_sender, last_receiver) = channel();
        let batch = VecDeque::from([
            Message::Command(Envelope {
                id: OperationId(1),
                operation: Operation::WholeColor(Rgb8::new(10, 0, 0)),
                completion: first_sender,
            }),
            Message::Command(Envelope {
                id: OperationId(2),
                operation: Operation::WholeColor(Rgb8::new(20, 0, 0)),
                completion: last_sender,
            }),
        ]);

        assert!(!process_batch(&mut backend, batch));
        assert_eq!(
            first_receiver.recv().unwrap(),
            CommandOutcome::Superseded {
                operation: OperationId(1),
                by: OperationId(2),
            }
        );
        assert_eq!(
            last_receiver.recv().unwrap(),
            CommandOutcome::Applied {
                operation: OperationId(2),
            }
        );
        assert_eq!(*log.lock().unwrap(), ["color:20"]);
    }
}
