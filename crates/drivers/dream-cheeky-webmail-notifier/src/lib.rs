#![forbid(unsafe_code)]

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    ExactWriteError, HidDeviceMatch, HidEndpointInfo, OutputReport, OutputWriter, write_exact,
};

pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x1D34,
    product_id: 0x0004,
    interface_number: None,
    usage_page: None,
    usage: None,
};
pub const OUTPUT_REPORT_LEN: usize = 9;

const INIT_REPORTS: [[u8; OUTPUT_REPORT_LEN]; 4] = [
    [0x00, 0x1F, 0x02, 0x00, 0x5F, 0x00, 0x00, 0x1F, 0x03],
    [0x00, 0x00, 0x02, 0x00, 0x5F, 0x00, 0x00, 0x1F, 0x04],
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1F, 0x05],
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01],
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Initialization {
    reports: [OutputReport<OUTPUT_REPORT_LEN>; 4],
}

impl Default for Initialization {
    fn default() -> Self {
        Self::new()
    }
}

impl Initialization {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            reports: [
                OutputReport::from_array(INIT_REPORTS[0]),
                OutputReport::from_array(INIT_REPORTS[1]),
                OutputReport::from_array(INIT_REPORTS[2]),
                OutputReport::from_array(INIT_REPORTS[3]),
            ],
        }
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<OUTPUT_REPORT_LEN>; 4] {
        &self.reports
    }

    /// Sends all four initialization reports in native order.
    ///
    /// # Errors
    ///
    /// Stops on a transport error or short write.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        for report in &self.reports {
            write_exact(writer, report)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectColorTransaction(OutputReport<OUTPUT_REPORT_LEN>);

impl DirectColorTransaction {
    #[must_use]
    pub const fn new(color: Rgb8) -> Self {
        Self(OutputReport::from_array([
            0x00,
            scale_channel(color.r),
            scale_channel(color.g),
            scale_channel(color.b),
            0x00,
            0x00,
            0x00,
            0x1F,
            0x05,
        ]))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<OUTPUT_REPORT_LEN> {
        &self.0
    }

    /// Sends one scaled direct-color report.
    ///
    /// # Errors
    ///
    /// Returns a transport error or short write.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        write_exact(writer, &self.0)
    }
}

const fn scale_channel(value: u8) -> u8 {
    (value >> 2) + if value == u8::MAX { 1 } else { 0 }
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description() -> ControllerDescription {
    ControllerDescription {
        name: "Dream Cheeky Webmail Notifier".into(),
        vendor: "Dream Cheeky".into(),
        description: "Dream Cheeky Device".into(),
        device_type: DeviceType::Accessory,
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 0,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: None,
        }],
        zone_names: vec!["LED".into()],
        led_names: vec!["LED".into()],
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

    fn endpoint(interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"dream-test"[..]),
            0x1D34,
            0x0004,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn matcher_accepts_any_interface_and_usage_for_the_exact_product() {
        assert!(matches(&endpoint(0, 0, 0)));
        assert!(matches(&endpoint(7, 0xFF00, 9)));
        let mut wrong = endpoint(0, 0, 0);
        wrong.product_id = 5;
        assert!(!matches(&wrong));
    }

    #[test]
    fn initialization_is_four_byte_exact_reports() {
        for (report, expected) in Initialization::new().reports().iter().zip(INIT_REPORTS) {
            assert_eq!(report.as_bytes(), &expected);
        }
    }

    #[test]
    fn color_scaling_preserves_native_endpoints_and_packet() {
        assert_eq!(scale_channel(0), 0);
        assert_eq!(scale_channel(254), 63);
        assert_eq!(scale_channel(255), 64);
        assert_eq!(
            DirectColorTransaction::new(Rgb8::new(255, 128, 4))
                .report()
                .as_bytes(),
            &[0, 64, 32, 1, 0, 0, 0, 0x1F, 0x05]
        );
    }

    #[test]
    fn initialization_stops_on_short_write() {
        let mut writer = RecordingWriter {
            actual: 8,
            reports: Vec::new(),
        };
        assert!(matches!(
            Initialization::new().apply(&mut writer),
            Err(ExactWriteError::ShortWrite {
                expected: 9,
                actual: 8
            })
        ));
        assert_eq!(writer.reports.len(), 1);
    }

    #[test]
    fn description_preserves_single_accessory_led() {
        let device = description();
        assert_eq!(device.device_type, DeviceType::Accessory);
        assert_eq!(device.zone_names, ["LED"]);
        assert_eq!(device.led_names, ["LED"]);
        assert_eq!(device.modes[0].name, "Direct");
    }
}
