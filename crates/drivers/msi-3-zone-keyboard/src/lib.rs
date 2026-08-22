#![forbid(unsafe_code)]

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x1770,
    product_id: 0xFF00,
    interface_number: None,
    usage_page: None,
    usage: None,
};
pub const FEATURE_REPORT_LEN: usize = 8;
pub const LED_COUNT: usize = 4;
pub const REPORT_COUNT: usize = 7;

const REPORT_ZONE_IDS: [u8; REPORT_COUNT] = [1, 2, 3, 4, 5, 6, 7];
const REPORT_COLOR_INDEXES: [usize; REPORT_COUNT] = [0, 1, 2, 3, 3, 3, 3];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerLedColorTransaction {
    reports: [OutputReport<FEATURE_REPORT_LEN>; REPORT_COUNT],
}

impl PerLedColorTransaction {
    #[must_use]
    pub fn new(colors: [Rgb8; LED_COUNT]) -> Self {
        Self {
            reports: std::array::from_fn(|index| {
                let color = colors[REPORT_COLOR_INDEXES[index]];
                OutputReport::from_array([
                    0x01,
                    0x02,
                    0x40,
                    REPORT_ZONE_IDS[index],
                    color.r,
                    color.g,
                    color.b,
                    0xEC,
                ])
            }),
        }
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>; REPORT_COUNT] {
        &self.reports
    }

    /// Sends the seven native feature reports in zone-ID order.
    ///
    /// # Errors
    ///
    /// Stops on the first HID transport error.
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
        name: "MSI 3-Zone Keyboard".into(),
        vendor: "MSI".into(),
        description: "MSI 3-Zone Keyboard Device".into(),
        device_type: DeviceType::Laptop,
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 0,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: None,
        }],
        zone_names: vec!["Keyboard".into(), "Aux".into()],
        led_names: vec![
            "Keyboard Left".into(),
            "Keyboard Middle".into(),
            "Keyboard Right".into(),
            "Aux".into(),
        ],
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

    fn endpoint(interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"msi-test"[..]),
            0x1770,
            0xFF00,
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
        assert!(matches(&endpoint(4, 0xFF00, 2)));
        let mut wrong = endpoint(0, 0, 0);
        wrong.product_id = 0xFF01;
        assert!(!matches(&wrong));
    }

    #[test]
    fn reports_preserve_native_bytes_zone_ids_and_aux_reuse() {
        let colors = [
            Rgb8::new(1, 2, 3),
            Rgb8::new(4, 5, 6),
            Rgb8::new(7, 8, 9),
            Rgb8::new(10, 11, 12),
        ];
        let transaction = PerLedColorTransaction::new(colors);
        assert_eq!(
            transaction.reports()[0].as_bytes(),
            &[1, 2, 64, 1, 1, 2, 3, 236]
        );
        assert_eq!(
            transaction.reports()[2].as_bytes(),
            &[1, 2, 64, 3, 7, 8, 9, 236]
        );
        for report in &transaction.reports()[3..] {
            assert_eq!(&report.as_bytes()[4..7], &[10, 11, 12]);
        }
        assert_eq!(transaction.reports()[6].as_bytes()[3], 7);
    }

    #[test]
    fn apply_sends_all_seven_reports_in_order() {
        let mut writer = RecordingWriter::default();
        PerLedColorTransaction::new([Rgb8::BLACK; LED_COUNT])
            .apply(&mut writer)
            .unwrap();
        assert_eq!(writer.0.len(), REPORT_COUNT);
        assert_eq!(
            writer.0.iter().map(|report| report[3]).collect::<Vec<_>>(),
            REPORT_ZONE_IDS
        );
    }

    #[test]
    fn description_preserves_laptop_zones_and_four_leds() {
        let device = description();
        assert_eq!(device.device_type, DeviceType::Laptop);
        assert_eq!(device.modes[0].value, 0);
        assert_eq!(device.zone_names, ["Keyboard", "Aux"]);
        assert_eq!(device.led_names.len(), LED_COUNT);
    }
}
