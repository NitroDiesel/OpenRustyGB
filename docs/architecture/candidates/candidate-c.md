# Candidate C — event-sourced semantic kernel and isolated effect executors

**Scope.** This counter-design replaces upstream commit
8121ee29f46d58f90a56348eb5bf7a64f52f923b with Rust and egui/eframe. Unlike
an actor-owned-controller design, one append-only journal plus pure reducer is
the global source of semantic truth. Per-device executors own only an open
transport and an ephemeral queue, never a controller's desired or applied
state.

## Usage (caller's view, written first)

Callers have a small capability-bound port: read a projection at a journal
sequence, subscribe to deltas, or execute a semantic intent. They never select
a driver, mutate a controller, write HID, edit a profile JSON document, or
construct an SDK packet.

### Eframe controller page

~~~rust
use openrustygb_domain::{AppIntent, ColorTarget, ControllerOperation, Rgb8};
use openrustygb_ui::ProjectionBridge;

fn set_wheel(bridge: &ProjectionBridge, id: ControllerId, color: Rgb8) {
    bridge.submit(AppIntent::controller(
        id,
        ControllerOperation::set_colors(ColorTarget::whole_device(), vec![color]),
    ));
}

fn update(&mut self, ctx: &egui::Context) {
    self.view.apply_all(self.bridge.drain_deltas());
    render_controller_page(ctx, &self.view, &self.bridge);
}
~~~

The page derives controls from ControllerSnapshot capabilities. The exact
Pulsefire Haste 2 page has one scroll-wheel direct-color control and cannot
render a DPI, save, firmware, polling, profile, or generic-report action.

### CLI profile application

~~~rust
use openrustygb_core::{RunMode, Runtime};
use openrustygb_domain::{AppIntent, ProfileRef};

async fn activate(data_dir: DataDir) -> anyhow::Result<()> {
    let runtime = Runtime::connect_or_start(RunMode::Cli, data_dir).await?;
    let report = runtime.cli_port()
        .execute(AppIntent::apply_profile(ProfileRef::named("Night")?))
        .await?;
    println!("{}", report.human_summary());
    runtime.shutdown().await?;
    Ok(())
}
~~~

The port expands profile targets, preserves each target's barrier semantics,
and waits for causal hardware outcomes where upstream behavior is synchronous.
It reports partial per-device results instead of inventing a global hardware
transaction.

### Exact Haste 2 lighting

~~~rust
async fn set_haste_wheel(port: &ControlPort) -> Result<IntentResult, AppError> {
    let haste = port.view().await?.one(
        ControllerSelector::usb(0x03F0, 0x0B97)
            .on_hid_interface(2)
            .vendor_usage(0xFF90, 0xFF00),
    )?;
    port.execute(AppIntent::controller(
        haste.id,
        ControllerOperation::set_colors(ColorTarget::whole_device(), vec![Rgb8::RED]),
    )).await
}
~~~

This explicit intent is journalled before any executor sees it. Discovery,
open, profile loading, reconnect, and replay never create an Haste output
effect automatically.

## Shape

### Semantic data and append-only facts

~~~rust
// crates/domain
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ControllerId(pub uuid::Uuid);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct JournalSeq(pub u64);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeviceEffectOrder(pub u64);

#[repr(i32)]
pub enum DeviceType {
    // Generated from the pinned upstream numeric table; every discriminant is
    // literal and tested, with Unknown kept last.
    Motherboard = 0,
    // ...
    Unknown = UPSTREAM_UNKNOWN_DEVICE_TYPE,
}

pub struct ControllerSnapshot {
    pub id: ControllerId,
    pub origin: ControllerOrigin,
    pub identity: PublicIdentity,
    pub metadata: ControllerMetadata,
    pub device_type: DeviceType,
    pub flags: ControllerFlags,
    pub topology: LightingTopology,
    pub desired: LightingState,
    pub applied: AppliedLightingState,
    pub capabilities: ControllerCapabilities,
    pub availability: Availability,
    pub at: JournalSeq,
}

pub struct LightingTopology {
    /// Stable upstream flat LED ordering; no self-referential zone pointers.
    pub leds: Vec<Led>,
    pub zones: Vec<Zone>,
    pub segments: Vec<Segment>,
    pub matrix: Option<MatrixMap>,
}

pub struct LedRange { pub first: LedIndex, pub len: NonZeroU16 }
impl LedRange {
    pub fn checked(first: LedIndex, len: u16, count: usize) -> Result<Self, TopologyError> {
        todo!("validate value-owned LED range")
    }
}

pub enum ControllerOperation {
    SetMode { mode: ModeSelection },
    SetColors { target: ColorTarget, colors: Vec<Rgb8> },
    SetZoneMode { zone: ZoneId, mode: ModeSelection },
    SetLedColor { led: LedIndex, color: Rgb8 },
    Save,
    // Generated compatibility variants retain all remaining upstream behavior.
}

pub enum AppIntent {
    Controller { id: ControllerId, operation: ControllerOperation },
    ApplyProfile { profile: ProfileRef },
    SaveProfile { draft: ProfileDraft },
    Settings { change: SettingChange },
    Detection { command: DetectionCommand },
    Lifecycle { command: LifecycleCommand },
}

pub struct EventEnvelope {
    pub seq: JournalSeq,
    pub causation: CausationId,
    pub correlation: CorrelationId,
    pub kind: KernelEvent,
}

pub enum KernelEvent {
    IntentAccepted { grant: RecordedGrant, intent: ValidatedIntent },
    ProviderObserved { provider: ProviderId, observation: ProviderObservation },
    CompatibilityDocumentObserved { document: ObservedDocument },
    DesiredStateChanged { id: ControllerId, state: LightingState, reason: UpdateReason },
    EffectRequested { effect: PlannedEffect },
    EffectAttempted { key: EffectKey },
    EffectSuperseded { key: EffectKey, by: EffectKey },
    EffectSucceeded { key: EffectKey, receipt: EffectReceipt },
    EffectFailed { key: EffectKey, error: EffectError },
    EffectUncertainAfterCrash { key: EffectKey },
    LifecycleChanged { change: LifecycleFact },
    Checkpoint { projection_hash: [u8; 32], covers_through: JournalSeq },
}

pub struct PlannedEffect {
    pub key: EffectKey,
    pub controller: ControllerId,
    pub device_order: DeviceEffectOrder,
    pub delivery: DeliveryClass,
    pub operation: DriverOperation,
    pub retry: RecoveryPolicy,
}

pub enum DeliveryClass { Coalescible, Barrier }
~~~

The journal contains only validated domain facts. Its generated compatibility
table is the one source for stable device-type values, controller/mode flags,
update reasons, and JSON description fields. Wire bytes, JSON syntax trees,
OS handles, and plugin plumbing never appear here. Desired state changes at
intent acceptance; applied state changes only on a causal EffectSucceeded.
Failures, supersession, and uncertainty remain visible facts rather than
optimistic state overwritten by a UI.

### One kernel, one reducer

~~~rust
// crates/kernel
pub struct ControlPort { grant: CallerGrant, kernel: KernelHandle }

impl ControlPort {
    pub async fn view(&self) -> Result<AppView, AppError> {
        todo!("immutable projection plus JournalSeq")
    }
    pub fn subscribe(&self) -> ProjectionSubscription {
        todo!("bounded deltas with explicit resync")
    }
    pub async fn execute(&self, intent: AppIntent) -> Result<IntentResult, AppError> {
        todo!("journal one semantic request and await its defined completion")
    }
}

pub struct JournalWriter {
    journal: Box<dyn AppendOnlyJournal>,
    state: KernelState,
    publisher: ProjectionPublisher,
    effects: EffectRouterHandle,
}

impl JournalWriter {
    async fn submit(&mut self, grant: CallerGrant, input: AppIntent)
        -> Result<IntentResult, AppError>
    {
        // Validate and authorize once, plan a contiguous append batch, append,
        // reduce, publish, then notify effects. Never wait on I/O here.
        todo!("sole semantic mutation route")
    }

    async fn record_provider(&mut self, observation: ProviderObservation)
        -> Result<(), AppError>
    {
        todo!("normalize local, network, and plugin observations")
    }

    async fn record_effect(&mut self, fact: KernelEvent) -> Result<(), AppError> {
        // Accept only outstanding effects for the current controller epoch and
        // valid device order; audit duplicate/stale results without mutation.
        todo!("causal reconciliation")
    }
}

pub struct KernelState {
    // Rebuilt by replay. Executors never own or modify this state.
    controllers: BTreeMap<ControllerId, ControllerRecord>,
    providers: BTreeMap<ProviderId, ProviderLease>,
    profiles: ProfileState,
    settings: SettingsState,
    plugins: PluginState,
    lifecycle: LifecycleState,
    at: JournalSeq,
}

pub fn plan(state: &KernelState, grant: &CallerGrant, intent: AppIntent)
    -> Result<AppendPlan, AppError>
{
    // Validate topology/capabilities/profile rules, allocate device order,
    // decide barrier/coalescing policy, and produce facts—not a raw write.
    todo!("pure planner")
}

pub fn reduce(previous: &KernelState, fact: &EventEnvelope) -> KernelState {
    // Total, deterministic, I/O-free function used for live reduction/replay.
    todo!("global semantic source of truth")
}
~~~

JournalWriter is the only short global critical section. It atomically appends
an accepted intent with its derived DesiredStateChanged and EffectRequested
facts, then reduces and publishes the projection. It never opens a device,
encodes SDK, parses a live file, invokes a callback, or waits for an effect.
This gives all consumers a linearization point while keeping expensive work
outside the global path.

### Isolated effect executors, consistency, and head-of-line behavior

~~~rust
// crates/effects
pub trait EffectEndpoint: Send {
    fn controller(&self) -> ControllerId;
    fn epoch(&self) -> ControllerEpoch;
    async fn apply(&mut self, operation: DriverOperation) -> Result<EffectReceipt, EffectError>;
    async fn stop_accepting(&mut self) -> Result<(), EffectError>;
    fn into_retained_transport(self: Box<Self>) -> RetainedTransport;
}

pub struct DeviceEffectExecutor {
    // Open endpoint, next order cursor, coalescing buffer, cancellation only.
    // It has no ControllerState, profiles, callbacks, or semantic view.
    endpoint: Box<dyn EffectEndpoint>,
    inbox: DeviceEffectStream,
    cancellation: CancellationToken,
}

impl DeviceEffectExecutor {
    async fn run(mut self, results: EffectResultPort) {
        // Receive matching controller epoch in DeviceEffectOrder. Coalesce only
        // contiguous whole-device color/mode effects; append Superseded for
        // every dropped key; flush before barriers; append Attempted before
        // output; append Success/Failure after output; retain transport on stop.
        todo!("side effects only")
    }
}
~~~

EffectRouter tails committed EffectRequested facts and sends them to a leased
endpoint supplied by the local detector, NetworkProvider, or PluginProvider.
Those owners retain physical/proxy/virtual resources; the kernel owns the
semantic aggregate. They all append the same ProviderObserved/progress/removal
facts, so local detection and local-client/network behavior project identically
without flattening their ownership.

Ordering is exact:

1. The planner allocates a global JournalSeq and a controller-epoch-local
   DeviceEffectOrder in one append batch. No later same-device effect can
   overtake it.
2. Desired state and its ticket become visible at that sequence. This is the
   consistent read point for UI, CLI, profiles, SDK, callbacks, and plugins.
3. Each executor serializes only its own device order. Whole-device color/mode
   updates may supersede a pending same-device update after the latest barrier;
   it journals EffectSuperseded. It never coalesces through a barrier.
4. Zone, LED, zone-mode, and save effects are barriers: the executor completes
   preceding effects, then waits for its physical result. Their callers wait
   for EffectSucceeded or EffectFailed. Whole-device updates return Queued and
   later resolve Applied, Superseded, Failed, or Uncertain.
5. Outcomes re-enter JournalWriter, which causally verifies, appends, reduces,
   and broadcasts the applied state/update reason. No executor changes a view.

A slow HID device head-of-line blocks later effects only for that ControllerId,
as required by hardware ordering; other devices execute concurrently. The
global head-of-line cost is bounded journal admission only. It validates and
appends small batches, then releases the sequencer before I/O. Bounded queues
and backpressure prevent an SDK/UI burst from consuming unbounded memory.
This is worse than per-controller semantic actors at extremely high mutation
rates, but better when a replayable total order, cross-surface consistency,
and crash diagnosis are worth the global sequencing cost.

Physical exactly-once output is impossible across a crash. The executor appends
EffectAttempted before output; an unpaired requested/attempted fact becomes
EffectUncertainAfterCrash on recovery. It is never blindly replayed. A proven
idempotent operation may offer user-initiated retry; Save and unknown or
persistent operations require a new explicit intent. Haste 2 permits no
background retry. This reconciles effects honestly instead of claiming a
hardware result that may not have occurred.

### Crate and module map

| Crate/module | Owns | Depends on |
| --- | --- | --- |
| openrustygb-domain | Values, topology/ranges, intents, snapshots, stable enums | Rust core only |
| openrustygb-compat | Generated upstream tables, fixture metadata, port-ledger schema | domain |
| openrustygb-journal | Segmented append-only records, batches, checkpoints, replay cursor | domain |
| openrustygb-kernel | Planner, reducer, KernelState, grants, projection publication | domain, journal, compat |
| openrustygb-effects | Effect router, provider leases, per-device executors, result port | domain, kernel |
| openrustygb-hardware | Rust-native HID/USB/I2C/serial inventory and transports | domain, OS Rust bindings |
| drivers/family-* | Rust detector/adapter implementations and private serializers | domain, hardware, effects |
| openrustygb-storage | JSON import/materialization, migrations, profiles/settings schemas | domain, kernel, compat |
| openrustygb-sdk | Private ORGB codec/session/callbacks and NetworkProvider | domain, kernel |
| openrustygb-plugin | Wasm host, PluginProvider, declarative UI/profile data | domain, kernel, effects |
| openrustygb-platform | Windows/Linux/macOS lifecycle, service, and autostart adapters | domain, kernel |
| openrustygb-ui | eframe projection pages, tray, accessibility, localization | domain, kernel, plugin |
| openrustygb-app and xtask | One executable, packaging, inventory/parity evidence | product crates/manifests |

The dependency graph points inward. No crate builds, links, or loads C++,
Qt, qmake, CMake C++ code, or a C++ FFI shim.

### Inventory, safe detection, and Haste 2

A checked inventory record exists for all 197 upstream controller-family
directories and 224 detector source files, naming Rust owner, match rule,
transport, upstream provenance, fixtures, and parity evidence. The exact Haste
2 adapter is an additive record, not a silent reuse of the legacy-Haste path.
The audit fails on missing, duplicate, orphaned, or untested-complete entries.

Detector enumeration is side-effect free. Attach may inspect identity through a
read-only capability but has no output/feature-report surface. It appends a
ProviderObserved attachment fact, never EffectRequested. The planner applies
manual configuration/profile matching to semantic state. An ordinary
controller whose policy allows startup restore can receive a separately
journalled post-attach effect before ready publication, preserving upstream
behavior without making detection itself write. Haste 2 is NoAutomaticWrites.

~~~rust
// crates/drivers/hyperx-haste2
const HASTE_2: ExactHidMatch = ExactHidMatch {
    vid: 0x03F0, pid: 0x0B97, interface_number: 2,
    usage_page: 0xFF90, usage: 0xFF00,
};

struct Haste2Endpoint {
    // Private output-only fixed-length endpoint; no generic HID, feature
    // report, DPI, firmware, reset, save, profile, polling, lift-off, or read.
    output: OutputReport65,
}

impl Haste2Endpoint {
    fn description(identity: PublicIdentity) -> ControllerSnapshot {
        todo!("exactly one scroll-wheel LED and direct color capability")
    }
    async fn direct_color(&mut self, color: Rgb8) -> Result<EffectReceipt, EffectError> {
        let primer = OutputReport65::zero_padded([0x44, 0x01, 0x01])?;
        let rgb = OutputReport65::zero_padded([
            0x44, 0x02, 0x00, 0x00, color.r, color.g, color.b,
        ])?;
        self.output.write_exact(primer).await?;
        self.output.write_exact(rgb).await?;
        Ok(EffectReceipt::applied())
    }
}
~~~

The driver accepts only interface 2; it never opens MI_00 mouse input or MI_01
keyboard/consumer/system controls for lighting. OutputReport65 zero-pads to
exactly 65 bytes and rejects a short write. Direct color sends only
44 01 01 then 44 02 00 00 RR GG BB. There is no discovery write, keepalive,
auto-profile effect, recovery retry, or generic escape hatch. Live parity
requires exact-device lighting, pointer, buttons, wheel, DPI, polling,
reconnect, and teardown evidence.

### Legacy JSON, profiles, SDK, and plugins

The journal is semantic truth; OpenRGB.json, Configuration.json, and
profiles/*.json are compatibility projections and controlled external inputs.
First migration parses, validates, backs up, and appends
CompatibilityDocumentObserved. A storage materializer atomically writes the
compatible JSON with journal sequence/source hash. On next startup an
externally modified hash is imported as a new observed document, never silently
raced. Unknown fields remain namespaced opaque data until a migration owns
them.

~~~rust
pub struct ProfileMatchKey {
    stable_identity: StableIdentity,
    i2c_address: Option<u16>,
    // HID location and I2C bus number intentionally do not exist here.
}
impl ProfileMatchKey {
    pub fn from_identity(identity: &DeviceIdentity) -> Self {
        todo!("retain stable identity/address; omit location/bus")
    }
}
~~~

The active journal is append-only and segmented. Checkpoint facts carry the
reproducible projection hash and covered sequence. After all durable
materializers/consumers acknowledge, old immutable segments can be archived or
removed under explicit retention; they are never rewritten. Lagging consumers
receive ResyncRequired and load a checkpoint view. This bounds replay/log
growth while keeping an audit trail for the configured period.

SdkGateway keeps wire types private: it validates the 16-byte ORGB header,
packet IDs/layouts, and 8 MiB limit before allocation, maps packets to intents,
and maps causal journal outcomes to replies. Its fixture-tested VersionContract
preserves v0 timeout negotiation, v5-and-earlier index addressing, v6 stable
IDs/acks, callbacks, and feature flags. Session index views are reducer
projections; loopback-local privileges are proved by peer socket address, not
packet metadata.

Existing Qt/C++ API v5 binaries cannot enter a pure Rust egui process. The
replacement is a major-versioned Wasm Component host that supplies virtual
controllers, settings schemas, namespaced profile data, tray actions,
localized strings, declarative PluginUiTree, and bounded SDK extensions.
Plugins append through a PluginProvider and cannot acquire raw HID or arbitrary
process/socket access. PluginMigrate imports legacy metadata/data without
executing a binary plugin. A Rust-only LegacyBridge may talk over a documented
socket to a user-started separately installed upstream process; it never loads
or ships Qt/C++ and is an escape hatch, not a claim of native binary support.

### UI, platforms, releases, and deletion

Eframe renders AppView into dynamic controller pages, schema-driven settings,
profiles/manual devices/information views, plugins, runtime language changes,
tray UI, and accessible keyboard/contrast/scalable-text/non-color/reduced-
motion behavior. It has no transport, SDK, or JSON dependency.

One executable composes GUI/tray, CLI, standalone, SDK client/server, headless
server, local-client, and Windows service modes over the same kernel.
Rust-native platform adapters translate Windows service/power/autostart, Linux
udev/systemd/tmpfiles/D-Bus, and macOS IOKit/autostart; suspend/resume appends
lifecycle facts and requests a safe rescan only.

| Platform | Required released artifacts |
| --- | --- |
| Windows | i686/x86_64 portable ZIP and x86_64 MSI |
| Linux | AppImage and DEB for i386/amd64/armhf/arm64; x86_64 RPM; x86_64/arm64 Flatpak |
| macOS | Intel and Apple Silicon application ZIP bundles |

Every matrix cell requires native install/launch, domain/replay/codec/migration/
UI/plugin/platform/driver evidence, and claimed hardware verification; a build
or --version is insufficient. The deletion gate requires every port-ledger
row complete and no current-tree C/C++/headers, Qt project, qmake/CMake C++
build, or C++ FFI dependency. GPL-2.0 notices, upstream history/remote,
provenance, and generated fixtures remain.

## Concurrency and shutdown contract

* JournalWriter is one bounded semantic sequencer with no I/O. Projection
  reads are consistent at JournalSeq; physical effects are eventually
  consistent and show desired versus applied state.
* Effects parallelize by ControllerId/epoch. A device executor owns only
  endpoint/cursor/buffer/cancellation, never callbacks, controller state,
  profiles, or global registry entries.
* Provider facts, effect results, document imports, and lifecycle changes
  re-enter JournalWriter; replay makes stale, duplicate, and wrong-epoch
  outcome handling deterministic.
* Callbacks use copied, bounded subscriptions outside locks. Slow consumers
  resync instead of blocking journal reduction or hardware writes.
* Unplug/shutdown order is fixed: (1) append WithdrawalBegun and close
  admission, (2) cancel/join executor and provider worker while cancellation
  facts are journalled, (3) close subscriptions/drain callback walks, (4)
  append ControllerWithdrawn and publish a snapshot without that handle, (5)
  release retained transport/provider state. No join/send/callback holds a
  journal or registry guard. Repeated stop/removal is idempotent.

## Compatibility and parity gates

| Gate | Required proof |
| --- | --- |
| Semantic model | Golden reducer traces for numeric types/flags, modes, speed/brightness/direction, zones/segments/matrices/flat LEDs, callbacks, update reasons, configuration, and save semantics. |
| Journal | Deterministic replay hash, checkpoint/restart/archive, stale result, uncertain crash, barrier/coalescing, backpressure and head-of-line tests. |
| Drivers | 197-family/224-detector-source inventory audit, no-write detection, deterministic match conflicts, protocol fixtures, and family evidence. |
| Haste 2 | Exact match rejections; only two 65-byte reports; short-write/error/cancel tests; physical non-lighting behavior observations. |
| Persistence | Legacy JSON/profile corpus, unknown-field/hash-conflict/atomic recovery, and profile key tests. |
| SDK | Header/layout/limit/timeout/index/stable-ID/ack/callback/flag/loopback golden tests plus interop. |
| Plugins/modes | Component conformance/migration/bridge limits and UI/platform mode accessibility/lifecycle tests. |
| Release/deletion | Native installer verification on every cell; complete port ledger; C++/Qt prohibition. |

## Red-flag screen

| Red flag | Result |
| --- | --- |
| Shallow module | Clear: the three-method ControlPort hides total order, validation, planning, persistence, projection, effect routing, and reconciliation. |
| Information leakage | Clear: packets, reports, JSON ASTs, OS handles, Qt values, and Wasm internals remain private adapters. |
| Temporal decomposition | Clear: modules own journal/reduction, effects, providers, storage, protocol, plugins, platform, or UI knowledge—not load/validate/save stages. |
| Pass-through method | Clear: JournalWriter validates, plans, allocates causality/order, appends, reduces, publishes, and reconciles; adapters enforce distinct boundary policy. |

## Rationale (one page)

### Problem

The port needs one compatible aggregate view across local, remote, and plugin
owners, with safe writes and crash truth, while preserving controller, SDK,
profile, platform, and release semantics. The tension is that device effects
must serialize locally but profiles, callbacks, and clients benefit from one
replayable global story.

### Usage (caller’s view)

UI, CLI, and Haste callers read a sequence-aware projection, subscribe, or
submit an intent. They see causal outcomes rather than locks, HID handles,
JSON, SDK indices, or a scheduler, so their contract remains small despite the
system's hardware and compatibility complexity.

### Shape

The journal/reducer owns desired/applied state and cross-provider facts; device
executors own only serial side effects. The port is deep because it hides
authorization, total ordering, coalescing, crash reconciliation, and
materialization. This applies boundary-discipline,
separate-before-serializing-shared-state, and encode-lessons-in-structure.

### Synthesis decision

Pending parent arena synthesis. Candidate C is the structurally distinct
event-sourced option: replay and causal diagnosis are first-class, accepting a
global semantic admission cost. It rejects fictional in-process Qt binary
plugin compatibility.

### Tradeoffs accepted

* We accept global semantic admission in exchange for one replayable truth
  across profiles, providers, callbacks, and clients.
* We accept desired/applied divergence in exchange for honest crash outcomes
  instead of impossible exactly-once HID claims.
* We accept checkpoint/archive machinery in exchange for bounded replay and
  controlled log growth.
* We accept declarative Wasm plugin migration in exchange for an all-Rust
  ownership and UI-lifetime boundary.

### Alternatives considered

* Per-controller semantic actors lose when global replay/causality is primary:
  they hide local serialization but require an aggregate merge story.
* One global hardware worker loses because one slow USB device blocks everyone,
  exposing device scheduling to unrelated callers.
* A mutable ResourceManager loses because callers learn locks/callback timing
  and ownership, a large shallow interface.

### Open questions and risks

* Is expected command throughput low enough for one durable semantic sequencer?
* Which operations can prove idempotent enough for user-initiated recovery?
* What retention, storage, and privacy policy should govern archived journals?
* Which plugin migrations and signing resources are available before release?

### Next implementation step

Build a journal/reducer/replay harness with one mock controller and the exact
Haste 2 output fixture before adding broad drivers or UI.
