#![forbid(unsafe_code)]

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription,
    Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x27B8,
    product_id: 0x01ED,
    interface_number: None,
    usage_page: Some(0xFF00),
    usage: Some(0x0001),
};
pub const FEATURE_REPORT_LEN: usize = 9;
pub const LED_COUNT: usize = 2;

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlinkMode {
    Off = 0,
    Direct = 1,
    Fade = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlinkLed {
    A = 1,
    B = 2,
}

const LEDS: [BlinkLed; LED_COUNT] = [BlinkLed::A, BlinkLed::B];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedUpdateTransaction(OutputReport<FEATURE_REPORT_LEN>);

impl LedUpdateTransaction {
    #[must_use]
    pub const fn new(led: BlinkLed, color: Rgb8, speed: u32) -> Self {
        Self(OutputReport::from_array([
            0x01,
            0x63,
            color.r,
            color.g,
            color.b,
            ((speed & 0xFF00) >> 8) as u8,
            (speed & 0x00FF) as u8,
            led as u8,
            0x00,
        ]))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<FEATURE_REPORT_LEN> {
        &self.0
    }

    /// Sends one LED update feature report.
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
pub struct ModeTransaction {
    reports: [OutputReport<FEATURE_REPORT_LEN>; LED_COUNT],
}

impl ModeTransaction {
    #[must_use]
    pub fn new(mode: BlinkMode, colors: [Rgb8; LED_COUNT], speed: u32) -> Self {
        let colors = if mode == BlinkMode::Off {
            [Rgb8::BLACK; LED_COUNT]
        } else {
            colors
        };
        let speed = if mode == BlinkMode::Fade { speed } else { 0 };
        Self {
            reports: std::array::from_fn(|index| {
                LedUpdateTransaction::new(LEDS[index], colors[index], speed).0
            }),
        }
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>; LED_COUNT] {
        &self.reports
    }

    /// Sends LED A followed by LED B.
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
pub fn description(device_name: &str) -> ControllerDescription {
    ControllerDescription {
        name: "Blink".into(),
        vendor: "ThingM".into(),
        description: device_name.into(),
        device_type: DeviceType::LedStrip,
        modes: vec![
            ModeDescription {
                name: "Off".into(),
                value: BlinkMode::Off as u32,
                color_mode: ModeColorMode::None,
                speed: None,
                brightness: None,
            },
            ModeDescription {
                name: "Direct".into(),
                value: BlinkMode::Direct as u32,
                color_mode: ModeColorMode::PerLed,
                speed: None,
                brightness: None,
            },
            ModeDescription {
                name: "Fade".into(),
                value: BlinkMode::Fade as u32,
                color_mode: ModeColorMode::PerLed,
                speed: Some(SpeedRange {
                    min: 0xFFFF,
                    max: 0,
                    current: 0,
                }),
                brightness: None,
            },
        ],
        zone_names: vec!["blink(1) mk2".into()],
        led_names: vec!["LED A".into(), "LED B".into()],
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::EFFECTS),
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
            Arc::from(&b"blink-test"[..]),
            0x27B8,
            0x01ED,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn matcher_accepts_any_interface_with_exact_usage() {
        assert!(matches(&endpoint(0, 0xFF00, 1)));
        assert!(matches(&endpoint(4, 0xFF00, 1)));
        assert!(!matches(&endpoint(0, 0xFF01, 1)));
    }

    #[test]
    fn led_packet_preserves_16_bit_speed_and_led_id() {
        assert_eq!(
            LedUpdateTransaction::new(BlinkLed::B, Rgb8::new(1, 2, 3), 0x1234)
                .report()
                .as_bytes(),
            &[1, 0x63, 1, 2, 3, 0x12, 0x34, 2, 0]
        );
    }

    #[test]
    fn speed_above_16_bits_matches_native_low_word_behavior() {
        assert_eq!(
            LedUpdateTransaction::new(BlinkLed::A, Rgb8::BLACK, 0x1_ABCD)
                .report()
                .as_bytes()[5..7],
            [0xAB, 0xCD]
        );
    }

    #[test]
    fn off_forces_both_leds_black_and_direct_zeros_speed() {
        let off = ModeTransaction::new(BlinkMode::Off, [Rgb8::new(1, 2, 3); 2], 0x1234);
        assert_eq!(off.reports()[0].as_bytes(), &[1, 0x63, 0, 0, 0, 0, 0, 1, 0]);
        let direct = ModeTransaction::new(BlinkMode::Direct, [Rgb8::new(1, 2, 3); 2], 0x1234);
        assert_eq!(&direct.reports()[1].as_bytes()[5..8], &[0, 0, 2]);
    }

    #[test]
    fn apply_preserves_a_then_b_order() {
        let mut writer = RecordingWriter::default();
        ModeTransaction::new(BlinkMode::Fade, [Rgb8::BLACK; 2], 1)
            .apply(&mut writer)
            .unwrap();
        assert_eq!(
            writer.0.iter().map(|report| report[7]).collect::<Vec<_>>(),
            [1, 2]
        );
    }

    #[test]
    fn description_preserves_modes_speed_and_two_led_shape() {
        let device = description("ThingM blink(1) mk2");
        assert_eq!(device.device_type, DeviceType::LedStrip);
        assert_eq!(device.modes.len(), 3);
        assert_eq!(device.modes[2].speed.unwrap().min, 0xFFFF);
        assert_eq!(device.led_names, ["LED A", "LED B"]);
    }
}
