#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    ExactHidMatch, FeatureWriter, HidEndpointInfo, OutputReport, send_feature,
};

pub const MATCH: ExactHidMatch = ExactHidMatch {
    vendor_id: 0x4E53,
    product_id: 0x5406,
    interface_number: 1,
    usage_page: 0xFF01,
    usage: 0x0001,
};

pub const FEATURE_REPORT_LEN: usize = 8;
pub const BRIGHTNESS_MIN: u8 = 0x0A;
pub const BRIGHTNESS_MAX: u8 = 0x64;
pub const SPEED_MIN: u8 = 0x01;
pub const SPEED_MAX: u8 = 0x0A;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum N5312Mode {
    Breathing = 0x00,
    SingleBreath = 0x01,
    Direct = 0x02,
    Off = 0x03,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidModeSettings {
    Brightness(u8),
    Speed(u8),
}

impl fmt::Display for InvalidModeSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Brightness(value) => write!(f, "brightness {value} is outside 10..=100"),
            Self::Speed(value) => write!(f, "speed {value} is invalid for this mode"),
        }
    }
}

impl std::error::Error for InvalidModeSettings {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Initialization {
    report: OutputReport<FEATURE_REPORT_LEN>,
}

impl Default for Initialization {
    fn default() -> Self {
        Self::new()
    }
}

impl Initialization {
    /// Builds the feature report sent when the native driver opens the mouse.
    ///
    #[must_use]
    pub const fn new() -> Self {
        Self {
            report: OutputReport::from_array([0x07, 0xA0, 0, 0, 0, 0, 0, 0]),
        }
    }

    /// Sends the initialization feature report.
    ///
    /// # Errors
    ///
    /// Returns the HID transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        send_feature(writer, &self.report)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction {
    reports: [OutputReport<FEATURE_REPORT_LEN>; 2],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColorTransaction {
    report: OutputReport<FEATURE_REPORT_LEN>,
}

impl ColorTransaction {
    #[must_use]
    pub const fn new(color: Rgb8) -> Self {
        Self {
            report: OutputReport::from_array([0x07, 0x0B, 0x01, color.r, color.g, color.b, 0, 0]),
        }
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<FEATURE_REPORT_LEN> {
        &self.report
    }

    /// Sends one color feature report.
    ///
    /// # Errors
    ///
    /// Returns the HID transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        send_feature(writer, &self.report)
    }
}

impl ModeTransaction {
    /// Builds the native color report followed by its mode report.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidModeSettings`] when a UI-exposed range is violated.
    pub fn new(
        mode: N5312Mode,
        color: Rgb8,
        brightness: u8,
        speed: u8,
    ) -> Result<Self, InvalidModeSettings> {
        let (color, brightness, speed) = match mode {
            N5312Mode::Direct => {
                validate_brightness(brightness)?;
                if speed != 0 {
                    return Err(InvalidModeSettings::Speed(speed));
                }
                (color, brightness, 0)
            }
            N5312Mode::Breathing | N5312Mode::SingleBreath => {
                validate_brightness(brightness)?;
                if !(SPEED_MIN..=SPEED_MAX).contains(&speed) {
                    return Err(InvalidModeSettings::Speed(speed));
                }
                (color, brightness, speed)
            }
            N5312Mode::Off => (Rgb8::BLACK, 0, 0),
        };

        let color_report = ColorTransaction::new(color).report;
        let mode_report =
            OutputReport::from_array([0x07, 0x0A, mode as u8, 0x01, 0x01, 0x01, speed, brightness]);
        Ok(Self {
            reports: [color_report, mode_report],
        })
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>; 2] {
        &self.reports
    }

    /// Sends the color report and then the mode report.
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

fn validate_brightness(value: u8) -> Result<(), InvalidModeSettings> {
    if (BRIGHTNESS_MIN..=BRIGHTNESS_MAX).contains(&value) {
        Ok(())
    } else {
        Err(InvalidModeSettings::Brightness(value))
    }
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description() -> ControllerDescription {
    let brightness = Some(BrightnessRange {
        min: BRIGHTNESS_MIN,
        max: BRIGHTNESS_MAX,
        current: BRIGHTNESS_MAX,
    });
    let speed = Some(SpeedRange {
        min: u32::from(SPEED_MIN),
        max: u32::from(SPEED_MAX),
        current: u32::from(SPEED_MIN),
    });
    ControllerDescription {
        name: "N5312A USB Optical Mouse".into(),
        vendor: "Unknown".into(),
        description: "N5312A Device".into(),
        device_type: DeviceType::Mouse,
        modes: vec![
            ModeDescription {
                name: "Direct".into(),
                value: N5312Mode::Direct as u32,
                color_mode: ModeColorMode::PerLed,
                speed: None,
                brightness,
            },
            ModeDescription {
                name: "Breathing".into(),
                value: N5312Mode::Breathing as u32,
                color_mode: ModeColorMode::PerLed,
                speed,
                brightness,
            },
            ModeDescription {
                name: "Single Breath".into(),
                value: N5312Mode::SingleBreath as u32,
                color_mode: ModeColorMode::PerLed,
                speed,
                brightness,
            },
            ModeDescription {
                name: "Off".into(),
                value: N5312Mode::Off as u32,
                color_mode: ModeColorMode::None,
                speed: None,
                brightness: None,
            },
        ],
        zone_names: vec!["Mouse".into()],
        led_names: vec!["LED 1".into()],
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::EFFECTS)
            .union(ControllerCapabilities::BRIGHTNESS),
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
            Arc::from(&b"n5312-test"[..]),
            0x4E53,
            0x5406,
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
        assert!(matches(&endpoint(1, 0xFF01, 1)));
        assert!(!matches(&endpoint(0, 0xFF01, 1)));
        assert!(!matches(&endpoint(1, 0xFF00, 1)));
        assert!(!matches(&endpoint(1, 0xFF01, 2)));
    }

    #[test]
    fn initialization_matches_native_feature_report() {
        let mut writer = RecordingWriter::default();
        Initialization::new().apply(&mut writer).unwrap();
        assert_eq!(writer.0, [[0x07, 0xA0, 0, 0, 0, 0, 0, 0]]);
    }

    #[test]
    fn breathing_transaction_matches_native_order_and_bytes() {
        let transaction =
            ModeTransaction::new(N5312Mode::Breathing, Rgb8::new(1, 2, 3), 100, 1).unwrap();
        assert_eq!(
            transaction.reports()[0].as_bytes(),
            &[0x07, 0x0B, 0x01, 1, 2, 3, 0, 0]
        );
        assert_eq!(
            transaction.reports()[1].as_bytes(),
            &[0x07, 0x0A, 0x00, 1, 1, 1, 1, 100]
        );
    }

    #[test]
    fn off_forces_black_and_zero_controls() {
        let transaction =
            ModeTransaction::new(N5312Mode::Off, Rgb8::new(255, 255, 255), 255, 255).unwrap();
        assert_eq!(
            transaction.reports()[0].as_bytes(),
            &[0x07, 0x0B, 0x01, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            transaction.reports()[1].as_bytes(),
            &[0x07, 0x0A, 0x03, 1, 1, 1, 0, 0]
        );
    }

    #[test]
    fn ranges_reject_invalid_settings() {
        assert!(ModeTransaction::new(N5312Mode::Direct, Rgb8::BLACK, 9, 0).is_err());
        assert!(ModeTransaction::new(N5312Mode::Breathing, Rgb8::BLACK, 100, 0).is_err());
        assert!(ModeTransaction::new(N5312Mode::Direct, Rgb8::BLACK, 100, 1).is_err());
    }

    #[test]
    fn description_preserves_modes_and_single_led_shape() {
        let device = description();
        assert_eq!(device.modes.len(), 4);
        assert_eq!(device.modes[0].name, "Direct");
        assert_eq!(device.modes[1].speed.unwrap().current, 1);
        assert_eq!(device.modes[0].brightness.unwrap().current, 100);
        assert_eq!(device.zone_names, ["Mouse"]);
        assert_eq!(device.led_names, ["LED 1"]);
    }
}
