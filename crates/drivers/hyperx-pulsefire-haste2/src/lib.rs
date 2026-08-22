#![forbid(unsafe_code)]

use openrustygb_domain::{ControllerCapabilities, ControllerDescription, DeviceType, Rgb8};
use openrustygb_driver_api::{
    ExactHidMatch, ExactWriteError, HidEndpointInfo, OutputReport, OutputWriter, PrefixTooLong,
};

pub const MATCH: ExactHidMatch = ExactHidMatch {
    vendor_id: 0x03F0,
    product_id: 0x0B97,
    interface_number: 2,
    usage_page: 0xFF90,
    usage: 0xFF00,
};

pub const OUTPUT_REPORT_LEN: usize = 65;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WheelColorTransaction {
    reports: [OutputReport<OUTPUT_REPORT_LEN>; 2],
}

impl WheelColorTransaction {
    /// Creates the only output transaction approved for this device.
    ///
    /// # Errors
    ///
    /// Returns [`PrefixTooLong`] if an approved command prefix cannot fit the
    /// fixed 65-byte report. This would indicate a programming error.
    pub fn new(color: Rgb8) -> Result<Self, PrefixTooLong> {
        Ok(Self {
            reports: [
                OutputReport::zero_padded(&[0x44, 0x01, 0x01])?,
                OutputReport::zero_padded(&[0x44, 0x02, 0x00, 0x00, color.r, color.g, color.b])?,
            ],
        })
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<OUTPUT_REPORT_LEN>; 2] {
        &self.reports
    }

    /// Writes the primer and RGB reports serially as one logical transaction.
    ///
    /// # Errors
    ///
    /// Returns [`ExactWriteError`] on a transport error or short write and
    /// stops before sending any later report.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        for report in &self.reports {
            let actual = writer
                .write_output(report)
                .map_err(ExactWriteError::Transport)?;
            // The API buffer is always 65 bytes. Windows HID reports the
            // descriptor's 64-byte payload after consuming the report-ID byte.
            if actual != OUTPUT_REPORT_LEN && actual != OUTPUT_REPORT_LEN - 1 {
                return Err(ExactWriteError::ShortWrite {
                    expected: OUTPUT_REPORT_LEN - 1,
                    actual,
                });
            }
        }
        Ok(())
    }
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description() -> ControllerDescription {
    ControllerDescription {
        name: "HyperX Pulsefire Haste 2".into(),
        vendor: "HyperX".into(),
        description: "HyperX Pulsefire Haste 2 Mouse".into(),
        device_type: DeviceType::Mouse,
        modes: Vec::new(),
        zone_names: vec!["Scroll Wheel".into()],
        led_names: vec!["Scroll".into()],
        capabilities: ControllerCapabilities::DIRECT_COLOR,
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;
    use std::sync::Arc;

    use super::*;

    #[derive(Debug, Eq, PartialEq)]
    struct FakeError;

    impl fmt::Display for FakeError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("fake error")
        }
    }

    impl std::error::Error for FakeError {}

    #[derive(Debug, Default)]
    struct RecordingWriter {
        reports: Vec<[u8; OUTPUT_REPORT_LEN]>,
        next_len: Option<usize>,
    }

    impl OutputWriter<OUTPUT_REPORT_LEN> for RecordingWriter {
        type Error = FakeError;

        fn write_output(
            &mut self,
            report: &OutputReport<OUTPUT_REPORT_LEN>,
        ) -> Result<usize, Self::Error> {
            self.reports.push(*report.as_bytes());
            Ok(self.next_len.take().unwrap_or(OUTPUT_REPORT_LEN))
        }
    }

    fn endpoint(interface_number: i32, usage_page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"test-path"[..]),
            0x03F0,
            0x0B97,
            interface_number,
            usage_page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn matcher_fails_closed_on_every_interface_field() {
        assert!(matches(&endpoint(2, 0xFF90, 0xFF00)));
        assert!(!matches(&endpoint(0, 0xFF90, 0xFF00)));
        assert!(!matches(&endpoint(1, 0xFF90, 0xFF00)));
        assert!(!matches(&endpoint(2, 0xFF00, 0xFF00)));
        assert!(!matches(&endpoint(2, 0xFF90, 0x0001)));
    }

    #[test]
    fn wheel_color_is_exactly_two_zero_padded_65_byte_output_reports() {
        let transaction = WheelColorTransaction::new(Rgb8::new(0x12, 0x34, 0x56)).unwrap();
        let mut writer = RecordingWriter::default();
        transaction.apply(&mut writer).unwrap();

        assert_eq!(writer.reports.len(), 2);
        assert_eq!(&writer.reports[0][..3], &[0x44, 0x01, 0x01]);
        assert!(writer.reports[0][3..].iter().all(|byte| *byte == 0));
        assert_eq!(
            &writer.reports[1][..7],
            &[0x44, 0x02, 0x00, 0x00, 0x12, 0x34, 0x56]
        );
        assert!(writer.reports[1][7..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn short_write_stops_the_transaction() {
        let transaction = WheelColorTransaction::new(Rgb8::new(1, 2, 3)).unwrap();
        let mut writer = RecordingWriter {
            next_len: Some(63),
            ..RecordingWriter::default()
        };

        assert_eq!(
            transaction.apply(&mut writer),
            Err(ExactWriteError::ShortWrite {
                expected: 64,
                actual: 63,
            })
        );
        assert_eq!(writer.reports.len(), 1);
    }

    #[test]
    fn accepts_windows_report_id_excluded_completion_length() {
        let transaction = WheelColorTransaction::new(Rgb8::new(1, 2, 3)).unwrap();
        let mut writer = RecordingWriter {
            next_len: Some(64),
            ..RecordingWriter::default()
        };

        transaction.apply(&mut writer).unwrap();
        assert_eq!(writer.reports.len(), 2);
    }

    #[test]
    fn approved_reports_contain_no_dpi_or_legacy_haste_signatures() {
        let transaction = WheelColorTransaction::new(Rgb8::new(0x32, 0x81, 0xF2)).unwrap();
        for report in transaction.reports() {
            let bytes = report.as_bytes();
            assert!(!bytes.starts_with(&[0x32, 0x01, 0x01]));
            assert!(!bytes.starts_with(&[0x00, 0x04, 0xF2]));
            assert!(!bytes.starts_with(&[0x00, 0x81]));
        }
    }

    #[test]
    fn public_description_cannot_advertise_non_lighting_controls() {
        let device = description();
        assert_eq!(device.zone_names, ["Scroll Wheel"]);
        assert_eq!(device.led_names, ["Scroll"]);
        assert!(
            device
                .capabilities
                .contains(ControllerCapabilities::DIRECT_COLOR)
        );
    }
}
