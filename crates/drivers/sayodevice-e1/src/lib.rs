#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription,
    Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    ExactWriteError, HidDeviceMatch, HidEndpointInfo, OutputReport, OutputWriter,
    PendingInputReader,
};

pub const REPORT_LEN: usize = 64;
pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x8089,
    product_id: 0x0007,
    interface_number: None,
    usage_page: Some(0xFF11),
    usage: Some(2),
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SayoMode {
    Direct = 0xFF,
    Breathing = 0x02,
    Wave = 0x04,
    Switch = 0x06,
    Blink = 0x08,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidSpeed(pub u8);

impl fmt::Display for InvalidSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SayoDevice speed must be in 0..=3, got {}", self.0)
    }
}

impl std::error::Error for InvalidSpeed {}

#[derive(Debug, Eq, PartialEq)]
pub enum ApplyError<E> {
    Output(ExactWriteError<E>),
    Drain(E),
}

impl<E: fmt::Display> fmt::Display for ApplyError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Output(error) => write!(f, "could not write SayoDevice report: {error}"),
            Self::Drain(error) => write!(f, "could not drain SayoDevice input: {error}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for ApplyError<E> {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction(OutputReport<REPORT_LEN>);

impl ModeTransaction {
    /// Builds the native lighting command and its little-endian word checksum.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidSpeed`] for an animated-mode speed outside `0..=3`.
    pub fn new(mode: SayoMode, speed: u8, color: Rgb8, random: bool) -> Result<Self, InvalidSpeed> {
        if mode != SayoMode::Direct && speed > 3 {
            return Err(InvalidSpeed(speed));
        }
        let hardware_mode = if mode == SayoMode::Direct {
            0
        } else {
            mode as u8
        };
        let hardware_speed = if mode == SayoMode::Direct {
            3
        } else {
            3 - speed
        };
        let color_mode = u8::from(random && mode != SayoMode::Direct);
        let mode_byte = (hardware_speed << 6) | (color_mode << 4) | (hardware_mode & 0x0F);
        let payload = [
            0x1C, 0x00, 0x11, 0x00, 0x01, 0x00, 0x00, 0x00, 0x15, 0x00, 0x28, 0x00, 0x26, 0x00,
            0x4C, 0x00, 0x26, 0x00, 0x00, 0x00, mode_byte, 0x00, 0x80, 0x80, color.r, color.g,
            color.b,
        ];
        Ok(Self(packet(&payload)))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<REPORT_LEN> {
        &self.0
    }

    /// Writes the command and non-blockingly drains all pending input reports.
    ///
    /// # Errors
    ///
    /// Returns an output, short-write, or pending-input transport error.
    pub fn apply<W, E>(&self, io: &mut W) -> Result<(), ApplyError<E>>
    where
        W: OutputWriter<REPORT_LEN, Error = E> + PendingInputReader<REPORT_LEN, Error = E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        apply_and_drain(io, &self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveTransaction(OutputReport<REPORT_LEN>);

impl Default for SaveTransaction {
    fn default() -> Self {
        Self(packet(&[0x06, 0x00, 0x0D, 0x00, 0x96, 0x72]))
    }
}

impl SaveTransaction {
    #[must_use]
    pub const fn report(&self) -> &OutputReport<REPORT_LEN> {
        &self.0
    }

    /// Writes the save command and non-blockingly drains pending input.
    ///
    /// # Errors
    ///
    /// Returns an output, short-write, or pending-input transport error.
    pub fn apply<W, E>(&self, io: &mut W) -> Result<(), ApplyError<E>>
    where
        W: OutputWriter<REPORT_LEN, Error = E> + PendingInputReader<REPORT_LEN, Error = E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        apply_and_drain(io, &self.0)
    }
}

fn packet(command: &[u8]) -> OutputReport<REPORT_LEN> {
    debug_assert!(command.len() <= REPORT_LEN - 4);
    let mut checksum = 0x1221_u16;
    for word in command.chunks(2) {
        checksum = checksum.wrapping_add(u16::from(word[0]));
        if let Some(high) = word.get(1) {
            checksum = checksum.wrapping_add(u16::from(*high) << 8);
        }
    }
    let [checksum_low, checksum_high] = checksum.to_le_bytes();
    let mut report = [0; REPORT_LEN];
    report[..4].copy_from_slice(&[0x21, 0x12, checksum_low, checksum_high]);
    report[4..4 + command.len()].copy_from_slice(command);
    OutputReport::from_array(report)
}

fn apply_and_drain<W, E>(io: &mut W, report: &OutputReport<REPORT_LEN>) -> Result<(), ApplyError<E>>
where
    W: OutputWriter<REPORT_LEN, Error = E> + PendingInputReader<REPORT_LEN, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    let actual = io
        .write_output(report)
        .map_err(|error| ApplyError::Output(ExactWriteError::Transport(error)))?;
    if actual != REPORT_LEN && actual != REPORT_LEN - 1 {
        return Err(ApplyError::Output(ExactWriteError::ShortWrite {
            expected: REPORT_LEN - 1,
            actual,
        }));
    }
    let mut pending = [0; REPORT_LEN];
    while io.read_pending(&mut pending).map_err(ApplyError::Drain)? > 0 {}
    Ok(())
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description(device_name: &str) -> ControllerDescription {
    let speed = || {
        Some(SpeedRange {
            min: 0,
            max: 3,
            current: 1,
        })
    };
    ControllerDescription {
        name: device_name.into(),
        vendor: "SayoDevice".into(),
        description: "SayoDevice E1 Knob".into(),
        device_type: DeviceType::Keyboard,
        modes: vec![
            ModeDescription {
                name: "Direct".into(),
                value: 0xFF,
                color_mode: ModeColorMode::PerLed,
                speed: None,
                brightness: None,
            },
            ModeDescription {
                name: "Breathing".into(),
                value: 2,
                color_mode: ModeColorMode::PerLed,
                speed: speed(),
                brightness: None,
            },
            ModeDescription {
                name: "Wave".into(),
                value: 4,
                color_mode: ModeColorMode::None,
                speed: speed(),
                brightness: None,
            },
            ModeDescription {
                name: "Switch".into(),
                value: 6,
                color_mode: ModeColorMode::None,
                speed: speed(),
                brightness: None,
            },
            ModeDescription {
                name: "Blink".into(),
                value: 8,
                color_mode: ModeColorMode::PerLed,
                speed: speed(),
                brightness: None,
            },
        ],
        zone_names: vec!["Underglow".into()],
        led_names: vec!["LED 1".into()],
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::EFFECTS),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::convert::Infallible;
    use std::sync::Arc;

    use super::*;

    fn endpoint(interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"sayo-test"[..]),
            0x8089,
            0x0007,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[derive(Debug, Default)]
    struct RecordingIo {
        reports: Vec<[u8; REPORT_LEN]>,
        pending_reads: VecDeque<usize>,
    }

    impl OutputWriter<REPORT_LEN> for RecordingIo {
        type Error = Infallible;

        fn write_output(
            &mut self,
            report: &OutputReport<REPORT_LEN>,
        ) -> Result<usize, Self::Error> {
            self.reports.push(*report.as_bytes());
            Ok(REPORT_LEN - 1)
        }
    }

    impl PendingInputReader<REPORT_LEN> for RecordingIo {
        type Error = Infallible;

        fn read_pending(&mut self, _report: &mut [u8; REPORT_LEN]) -> Result<usize, Self::Error> {
            Ok(self.pending_reads.pop_front().unwrap_or(0))
        }
    }

    #[test]
    fn matcher_preserves_product_and_usage_only_detection() {
        assert!(matches(&endpoint(0, 0xFF11, 2)));
        assert!(matches(&endpoint(7, 0xFF11, 2)));
        assert!(!matches(&endpoint(0, 0xFF11, 1)));
        assert!(!matches(&endpoint(0, 0xFF12, 2)));
    }

    #[test]
    fn mode_and_save_packets_preserve_native_layout_and_checksum() {
        let direct =
            ModeTransaction::new(SayoMode::Direct, 99, Rgb8::new(0x11, 0x22, 0x33), true).unwrap();
        let bytes = direct.report().as_bytes();
        assert_eq!(&bytes[..6], &[0x21, 0x12, 0xA8, 0xB6, 0x1C, 0x00]);
        assert_eq!(bytes[24], 0xC0);
        assert_eq!(&bytes[28..31], &[0x11, 0x22, 0x33]);
        assert!(bytes[31..].iter().all(|byte| *byte == 0));

        let save = SaveTransaction::default();
        assert_eq!(
            &save.report().as_bytes()[..10],
            &[0x21, 0x12, 0xCA, 0x84, 0x06, 0x00, 0x0D, 0x00, 0x96, 0x72]
        );
    }

    #[test]
    fn animated_speed_is_inverted_and_write_drains_pending_input() {
        let breathing = ModeTransaction::new(SayoMode::Breathing, 1, Rgb8::BLACK, true).unwrap();
        assert_eq!(breathing.report().as_bytes()[24], 0x92);
        assert!(ModeTransaction::new(SayoMode::Wave, 4, Rgb8::BLACK, false).is_err());

        let mut io = RecordingIo {
            pending_reads: VecDeque::from([64, 64, 0]),
            ..RecordingIo::default()
        };
        breathing.apply(&mut io).unwrap();
        assert_eq!(io.reports.len(), 1);
        assert!(io.pending_reads.is_empty());
    }

    #[test]
    fn model_preserves_five_modes_and_one_led() {
        let device = description("SayoDevice E1");
        assert_eq!(device.modes.len(), 5);
        assert_eq!(device.zone_names, ["Underglow"]);
        assert_eq!(device.led_names, ["LED 1"]);
    }
}
