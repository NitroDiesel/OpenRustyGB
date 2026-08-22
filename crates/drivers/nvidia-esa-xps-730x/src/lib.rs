#![forbid(unsafe_code)]

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    ExactWriteError, HidDeviceMatch, HidEndpointInfo, OutputReport, OutputWriter, write_exact,
};

pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x0955,
    product_id: 0x000A,
    interface_number: None,
    usage_page: Some(0xFFDE),
    usage: Some(0x0002),
};
pub const OUTPUT_REPORT_LEN: usize = 4;
pub const ZONE_COUNT: usize = 5;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NvidiaEsaZone {
    FrontDriveBays = 0,
    FrontUsb = 1,
    Rear = 2,
    Internal = 3,
    FrontAudio = 4,
}

const ZONES: [NvidiaEsaZone; ZONE_COUNT] = [
    NvidiaEsaZone::FrontDriveBays,
    NvidiaEsaZone::FrontUsb,
    NvidiaEsaZone::Rear,
    NvidiaEsaZone::Internal,
    NvidiaEsaZone::FrontAudio,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZoneColorTransaction(OutputReport<OUTPUT_REPORT_LEN>);

impl ZoneColorTransaction {
    #[must_use]
    pub const fn new(zone: NvidiaEsaZone, color: Rgb8) -> Self {
        Self(OutputReport::from_array([
            0x42 + zone as u8,
            encode_channel(color.r),
            encode_channel(color.g),
            encode_channel(color.b),
        ]))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<OUTPUT_REPORT_LEN> {
        &self.0
    }

    /// Sends one zone report.
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllZonesTransaction {
    reports: [OutputReport<OUTPUT_REPORT_LEN>; ZONE_COUNT],
}

impl AllZonesTransaction {
    #[must_use]
    pub fn new(colors: [Rgb8; ZONE_COUNT]) -> Self {
        Self {
            reports: std::array::from_fn(|index| {
                ZoneColorTransaction::new(ZONES[index], colors[index]).0
            }),
        }
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<OUTPUT_REPORT_LEN>; ZONE_COUNT] {
        &self.reports
    }

    /// Sends the five zones from command `42` through `46`.
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

const fn encode_channel(value: u8) -> u8 {
    (u8::MAX - value) / 17
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description() -> ControllerDescription {
    ControllerDescription {
        name: "Nvidia ESA - Dell XPS 730x".into(),
        vendor: "NVIDIA".into(),
        description: "Nvidia ESA USB Device".into(),
        device_type: DeviceType::Case,
        modes: vec![ModeDescription {
            name: "Static".into(),
            value: 0,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: None,
        }],
        zone_names: vec![
            "Front Drive Bays".into(),
            "Front USB".into(),
            "Rear".into(),
            "Internal".into(),
            "Front Audio".into(),
        ],
        led_names: vec!["LED".into(); ZONE_COUNT],
        capabilities: ControllerCapabilities::PER_LED_COLOR.union(ControllerCapabilities::EFFECTS),
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
            Arc::from(&b"esa-test"[..]),
            0x0955,
            0x000A,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn matcher_accepts_any_interface_but_exact_usage() {
        assert!(matches(&endpoint(0, 0xFFDE, 2)));
        assert!(matches(&endpoint(7, 0xFFDE, 2)));
        assert!(!matches(&endpoint(0, 0xFFDD, 2)));
        assert!(!matches(&endpoint(0, 0xFFDE, 1)));
    }

    #[test]
    fn inverse_channel_encoding_matches_native_boundaries() {
        assert_eq!(encode_channel(0), 15);
        assert_eq!(encode_channel(16), 14);
        assert_eq!(encode_channel(17), 14);
        assert_eq!(encode_channel(255), 0);
    }

    #[test]
    fn zone_report_preserves_command_and_inverse_rgb() {
        assert_eq!(
            ZoneColorTransaction::new(NvidiaEsaZone::Rear, Rgb8::new(0, 255, 17))
                .report()
                .as_bytes(),
            &[0x44, 15, 0, 14]
        );
    }

    #[test]
    fn all_zones_preserve_native_command_order() {
        let transaction = AllZonesTransaction::new([Rgb8::BLACK; ZONE_COUNT]);
        let commands: Vec<_> = transaction
            .reports()
            .iter()
            .map(|report| report.as_bytes()[0])
            .collect();
        assert_eq!(commands, [0x42, 0x43, 0x44, 0x45, 0x46]);
    }

    #[test]
    fn all_zones_stop_on_short_write() {
        let mut writer = RecordingWriter {
            actual: 3,
            reports: Vec::new(),
        };
        assert!(
            AllZonesTransaction::new([Rgb8::BLACK; ZONE_COUNT])
                .apply(&mut writer)
                .is_err()
        );
        assert_eq!(writer.reports.len(), 1);
    }

    #[test]
    fn description_preserves_five_zone_case_shape() {
        let device = description();
        assert_eq!(device.device_type, DeviceType::Case);
        assert_eq!(device.zone_names.len(), ZONE_COUNT);
        assert_eq!(device.led_names, ["LED", "LED", "LED", "LED", "LED"]);
        assert_eq!(device.modes[0].name, "Static");
    }
}
