#![forbid(unsafe_code)]

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    ExactHidMatch, ExactWriteError, HidEndpointInfo, OutputReport, OutputWriter, PrefixTooLong,
    write_exact,
};

pub const MATCH: ExactHidMatch = ExactHidMatch {
    vendor_id: 0x3537,
    product_id: 0x100F,
    interface_number: 2,
    usage_page: 0xFF7A,
    usage: 0x0001,
};

pub const OUTPUT_REPORT_LEN: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticColorTransaction {
    report: OutputReport<OUTPUT_REPORT_LEN>,
}

impl StaticColorTransaction {
    /// Creates the `GameSir` static-color packet with its low-byte checksum.
    ///
    /// # Errors
    ///
    /// Returns [`PrefixTooLong`] if the fixed command cannot fit its 64-byte
    /// output report. This would indicate a programming error.
    pub fn new(color: Rgb8) -> Result<Self, PrefixTooLong> {
        let mut command = [
            0x05, 0x08, 0x0A, 0x01, 0x03, color.r, color.g, color.b, 0x00, 0x00,
        ];
        command[9] = command[..9]
            .iter()
            .fold(0_u8, |checksum, byte| checksum.wrapping_add(*byte));
        Ok(Self {
            report: OutputReport::zero_padded(&command)?,
        })
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<OUTPUT_REPORT_LEN> {
        &self.report
    }

    /// Writes the complete packet once.
    ///
    /// # Errors
    ///
    /// Returns [`ExactWriteError`] for a transport error or short write.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        write_exact(writer, &self.report)
    }
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description() -> ControllerDescription {
    ControllerDescription {
        name: "GameSir Nova 2 Lite".into(),
        vendor: "GameSir".into(),
        description: "GameSir RGB Device".into(),
        device_type: DeviceType::Gamepad,
        modes: vec![ModeDescription {
            name: "Static".into(),
            value: 0xFFFF,
            color_mode: ModeColorMode::PerLed,
        }],
        zone_names: vec!["Controller".into()],
        led_names: vec!["Main LED".into()],
        capabilities: ControllerCapabilities::PER_LED_COLOR,
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;
    use std::sync::Arc;

    use super::*;

    #[derive(Debug)]
    struct FakeError;

    impl fmt::Display for FakeError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("fake transport error")
        }
    }

    impl std::error::Error for FakeError {}

    #[derive(Debug)]
    struct RecordingWriter {
        actual: usize,
        reports: Vec<[u8; OUTPUT_REPORT_LEN]>,
    }

    impl OutputWriter<OUTPUT_REPORT_LEN> for RecordingWriter {
        type Error = FakeError;

        fn write_output(
            &mut self,
            report: &OutputReport<OUTPUT_REPORT_LEN>,
        ) -> Result<usize, Self::Error> {
            self.reports.push(*report.as_bytes());
            Ok(self.actual)
        }
    }

    fn endpoint(interface_number: i32, usage_page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"gamesir-test"[..]),
            0x3537,
            0x100F,
            interface_number,
            usage_page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn matcher_rejects_every_non_exact_interface_field() {
        assert!(matches(&endpoint(2, 0xFF7A, 0x0001)));
        assert!(!matches(&endpoint(1, 0xFF7A, 0x0001)));
        assert!(!matches(&endpoint(2, 0xFF00, 0x0001)));
        assert!(!matches(&endpoint(2, 0xFF7A, 0x0002)));
    }

    #[test]
    fn static_color_matches_the_native_packet_and_checksum() {
        let transaction = StaticColorTransaction::new(Rgb8::new(0x12, 0x34, 0x56)).unwrap();
        let bytes = transaction.report().as_bytes();
        assert_eq!(
            &bytes[..10],
            &[0x05, 0x08, 0x0A, 0x01, 0x03, 0x12, 0x34, 0x56, 0x00, 0xB7]
        );
        assert!(bytes[10..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn checksum_wraps_to_the_low_byte() {
        let transaction = StaticColorTransaction::new(Rgb8::new(0xFF, 0xFF, 0xFF)).unwrap();
        assert_eq!(transaction.report().as_bytes()[9], 0x18);
    }

    #[test]
    fn short_write_is_an_error() {
        let transaction = StaticColorTransaction::new(Rgb8::new(1, 2, 3)).unwrap();
        let mut writer = RecordingWriter {
            actual: 63,
            reports: Vec::new(),
        };
        assert!(matches!(
            transaction.apply(&mut writer),
            Err(ExactWriteError::ShortWrite {
                expected: 64,
                actual: 63
            })
        ));
    }

    #[test]
    fn description_preserves_the_original_controller_shape() {
        let device = description();
        assert_eq!(device.name, "GameSir Nova 2 Lite");
        assert_eq!(device.description, "GameSir RGB Device");
        assert_eq!(device.device_type, DeviceType::Gamepad);
        assert_eq!(device.modes.len(), 1);
        assert_eq!(device.modes[0].name, "Static");
        assert_eq!(device.modes[0].value, 0xFFFF);
        assert_eq!(device.zone_names, ["Controller"]);
        assert_eq!(device.led_names, ["Main LED"]);
    }
}
