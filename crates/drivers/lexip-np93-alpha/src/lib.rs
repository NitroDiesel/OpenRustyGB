#![forbid(unsafe_code)]

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    ExactHidMatch, ExactWriteError, HidEndpointInfo, OutputReport, OutputWriter, PrefixTooLong,
    write_exact,
};

pub const MATCH: ExactHidMatch = ExactHidMatch {
    vendor_id: 0x04D8,
    product_id: 0xFD0A,
    interface_number: 0,
    usage_page: 0x0001,
    usage: 0x0002,
};

pub const OUTPUT_REPORT_LEN: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectColorTransaction {
    report: OutputReport<OUTPUT_REPORT_LEN>,
}

impl DirectColorTransaction {
    /// Creates the Lexip direct-color packet used by the pinned native driver.
    ///
    /// # Errors
    ///
    /// Returns [`PrefixTooLong`] if the fixed command cannot fit its output
    /// report. This would indicate a programming error.
    pub fn new(color: Rgb8) -> Result<Self, PrefixTooLong> {
        Ok(Self {
            report: OutputReport::zero_padded(&[
                0x00, 0x24, 0x01, color.r, color.g, color.b, 0x00, 0x64, 0x80,
            ])?,
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
        name: "Np93 ALPHA - Gaming Mouse".into(),
        vendor: "Lexip".into(),
        description: "Np93 ALPHA - Gaming Mouse".into(),
        device_type: DeviceType::Mouse,
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 0x00,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: None,
        }],
        zone_names: vec!["Mouse".into()],
        led_names: vec!["LED 1".into()],
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR),
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
            Arc::from(&b"lexip-test"[..]),
            0x04D8,
            0xFD0A,
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
        assert!(matches(&endpoint(0, 0x0001, 0x0002)));
        assert!(!matches(&endpoint(1, 0x0001, 0x0002)));
        assert!(!matches(&endpoint(0, 0x0002, 0x0002)));
        assert!(!matches(&endpoint(0, 0x0001, 0x0001)));
    }

    #[test]
    fn direct_color_matches_the_native_packet() {
        let transaction = DirectColorTransaction::new(Rgb8::new(0x12, 0x34, 0x56)).unwrap();
        let bytes = transaction.report().as_bytes();
        assert_eq!(
            &bytes[..9],
            &[0x00, 0x24, 0x01, 0x12, 0x34, 0x56, 0x00, 0x64, 0x80]
        );
        assert!(bytes[9..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn short_write_is_an_error() {
        let transaction = DirectColorTransaction::new(Rgb8::new(1, 2, 3)).unwrap();
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
        assert_eq!(device.name, "Np93 ALPHA - Gaming Mouse");
        assert_eq!(device.vendor, "Lexip");
        assert_eq!(device.device_type, DeviceType::Mouse);
        assert_eq!(device.modes.len(), 1);
        assert_eq!(device.modes[0].name, "Direct");
        assert_eq!(device.modes[0].value, 0);
        assert_eq!(device.zone_names, ["Mouse"]);
        assert_eq!(device.led_names, ["LED 1"]);
    }
}
