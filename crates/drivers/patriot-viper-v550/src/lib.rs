#![forbid(unsafe_code)]

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    ExactHidMatch, FeatureWriter, HidEndpointInfo, OutputReport, send_feature,
};

pub const MATCH: ExactHidMatch = ExactHidMatch {
    vendor_id: 0x0C45,
    product_id: 0x7E18,
    interface_number: 2,
    usage_page: 0xFF18,
    usage: 0x0001,
};
pub const FEATURE_REPORT_LEN: usize = 64;
pub const LED_COUNT: usize = 7;
const LED_INDICES: [u8; LED_COUNT] = [0, 1, 2, 3, 4, 5, 6];

const INIT_REPORT: [u8; FEATURE_REPORT_LEN] = [
    0x01, 0x00, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0x50, 0xDE, 0x8D, 0x77, 0x09, 0xDF, 0x8D, 0x77, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x30, 0x58, 0x7C, 0x77, 0x78, 0x81, 0x43, 0x00, 0x30, 0x58, 0x7C, 0x77,
    0x8C, 0x5D, 0x9B, 0x77, 0x00, 0x00, 0x3D, 0x00, 0x98, 0xF5, 0x19, 0x08, 0x00, 0x00, 0x00, 0xEE,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Initialization(OutputReport<FEATURE_REPORT_LEN>);

impl Default for Initialization {
    fn default() -> Self {
        Self::new()
    }
}

impl Initialization {
    #[must_use]
    pub const fn new() -> Self {
        Self(OutputReport::from_array(INIT_REPORT))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<FEATURE_REPORT_LEN> {
        &self.0
    }

    /// Sends the feature report used when the native controller opens.
    ///
    /// # Errors
    ///
    /// Returns the HID transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        send_feature(writer, &self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerLedColorTransaction {
    reports: [OutputReport<FEATURE_REPORT_LEN>; LED_COUNT],
}

impl PerLedColorTransaction {
    #[must_use]
    pub fn new(colors: [Rgb8; LED_COUNT]) -> Self {
        let reports = std::array::from_fn(|index| {
            let mut bytes = [0_u8; FEATURE_REPORT_LEN];
            bytes[0] = 0x01;
            bytes[1] = 0x13;
            bytes[2] = LED_INDICES[index];
            bytes[3] = 0xFF;
            bytes[4] = colors[index].r;
            bytes[5] = colors[index].g;
            bytes[6] = colors[index].b;
            let xor = bytes[..63].iter().fold(0_u8, |value, byte| value ^ byte);
            bytes[63] = if xor % 2 == 0 {
                xor.wrapping_add(1)
            } else {
                xor.wrapping_sub(1)
            };
            OutputReport::from_array(bytes)
        });
        Self { reports }
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>; LED_COUNT] {
        &self.reports
    }

    /// Sends all seven LED reports in native order.
    ///
    /// # Errors
    ///
    /// Stops and returns the first HID transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        for report in &self.reports {
            send_feature(writer, report)?;
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
        name: "Patriot Viper V550".into(),
        vendor: "Patriot".into(),
        description: "Patriot Viper Mouse".into(),
        device_type: DeviceType::Mouse,
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 1,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: None,
        }],
        zone_names: vec!["Left".into(), "Right".into(), "Mousewheel".into()],
        led_names: (1..=LED_COUNT)
            .map(|index| format!("LED {index}"))
            .collect(),
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR),
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::sync::Arc;

    use super::*;

    #[derive(Debug, Default)]
    struct RecordingWriter(Vec<[u8; FEATURE_REPORT_LEN]>);

    impl FeatureWriter<FEATURE_REPORT_LEN> for RecordingWriter {
        type Error = Infallible;

        fn send_feature_report(
            &mut self,
            report: &OutputReport<FEATURE_REPORT_LEN>,
        ) -> Result<(), Self::Error> {
            self.0.push(*report.as_bytes());
            Ok(())
        }
    }

    fn endpoint(interface_number: i32, usage_page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"v550-test"[..]),
            0x0C45,
            0x7E18,
            interface_number,
            usage_page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn matcher_is_exact() {
        assert!(matches(&endpoint(2, 0xFF18, 1)));
        assert!(!matches(&endpoint(1, 0xFF18, 1)));
        assert!(!matches(&endpoint(2, 0xFF00, 1)));
        assert!(!matches(&endpoint(2, 0xFF18, 2)));
    }

    #[test]
    fn initialization_is_byte_exact() {
        assert_eq!(Initialization::new().report().as_bytes(), &INIT_REPORT);
    }

    #[test]
    fn per_led_reports_preserve_order_color_and_checksum_rule() {
        let colors = std::array::from_fn(|index| Rgb8::new(LED_INDICES[index], 0x20, 0x40));
        let transaction = PerLedColorTransaction::new(colors);
        for (index, report) in transaction.reports().iter().enumerate() {
            let bytes = report.as_bytes();
            assert_eq!(
                &bytes[..7],
                &[
                    1,
                    0x13,
                    LED_INDICES[index],
                    0xFF,
                    LED_INDICES[index],
                    0x20,
                    0x40,
                ]
            );
            let xor = bytes[..63].iter().fold(0_u8, |value, byte| value ^ byte);
            assert_eq!(
                bytes[63],
                if xor % 2 == 0 {
                    xor.wrapping_add(1)
                } else {
                    xor.wrapping_sub(1)
                }
            );
        }
    }

    #[test]
    fn apply_sends_seven_reports() {
        let mut writer = RecordingWriter::default();
        PerLedColorTransaction::new([Rgb8::BLACK; LED_COUNT])
            .apply(&mut writer)
            .unwrap();
        assert_eq!(writer.0.len(), LED_COUNT);
    }

    #[test]
    fn description_preserves_three_zones_and_seven_leds() {
        let device = description();
        assert_eq!(device.zone_names, ["Left", "Right", "Mousewheel"]);
        assert_eq!(device.led_names.len(), 7);
        assert_eq!(device.modes[0].value, 1);
    }
}
