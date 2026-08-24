#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const FEATURE_REPORT_LEN: usize = 9;
pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x1044,
    product_id: 0x7A30,
    interface_number: Some(0),
    usage_page: Some(0xFF01),
    usage: Some(0x0001),
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum AorusCaseMode {
    Off = 0x00,
    Custom = 0x01,
    Breathing = 0x02,
    SpectrumCycle = 0x03,
    Flashing = 0x04,
    DoubleFlashing = 0x05,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidSettings {
    Brightness(u8),
    Speed(u8),
}

impl fmt::Display for InvalidSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Brightness(value) => {
                write!(f, "AORUS case brightness must be in 0..=9, got {value}")
            }
            Self::Speed(value) => write!(f, "AORUS case speed must be in 6..=10, got {value}"),
        }
    }
}

impl std::error::Error for InvalidSettings {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction {
    reports: [OutputReport<FEATURE_REPORT_LEN>; 3],
}

impl ModeTransaction {
    /// Builds the native color, mode, and commit feature reports.
    ///
    /// # Errors
    ///
    /// Returns an error when a mode-dependent brightness or speed falls
    /// outside the values exposed by the native controller.
    pub fn new(
        mode: AorusCaseMode,
        color: Rgb8,
        brightness: u8,
        speed: u8,
    ) -> Result<Self, InvalidSettings> {
        if matches!(
            mode,
            AorusCaseMode::Custom | AorusCaseMode::Flashing | AorusCaseMode::DoubleFlashing
        ) && brightness > 9
        {
            return Err(InvalidSettings::Brightness(brightness));
        }
        if matches!(
            mode,
            AorusCaseMode::Breathing
                | AorusCaseMode::SpectrumCycle
                | AorusCaseMode::Flashing
                | AorusCaseMode::DoubleFlashing
        ) && !(6..=10).contains(&speed)
        {
            return Err(InvalidSettings::Speed(speed));
        }

        let (encoded_mode, encoded_color, encoded_brightness, encoded_speed) = match mode {
            AorusCaseMode::Off => (AorusCaseMode::Custom, Rgb8::BLACK, 10, 10),
            AorusCaseMode::Custom => (mode, color, brightness, 9),
            AorusCaseMode::Breathing => (mode, color, 9, speed),
            AorusCaseMode::SpectrumCycle => (mode, Rgb8::new(0xFF, 0, 0), 9, speed),
            AorusCaseMode::Flashing | AorusCaseMode::DoubleFlashing => {
                (mode, color, brightness * 10, speed)
            }
        };

        Ok(Self {
            reports: [
                OutputReport::from_array([
                    0,
                    1,
                    0xC8,
                    encoded_color.r,
                    encoded_color.g,
                    encoded_color.b,
                    8,
                    1,
                    0,
                ]),
                OutputReport::from_array([
                    0,
                    1,
                    0xC9,
                    encoded_mode as u8,
                    encoded_brightness,
                    encoded_speed,
                    1,
                    8,
                    0,
                ]),
                OutputReport::from_array([0, 1, 0xB6, 0, 0, 0, 0, 0, 0]),
            ],
        })
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>; 3] {
        &self.reports
    }

    /// Sends the color, mode, and commit reports in native order.
    ///
    /// # Errors
    ///
    /// Stops on the first feature-report transport error.
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

fn mode_description(
    name: &str,
    mode: AorusCaseMode,
    color_mode: ModeColorMode,
    speed: Option<SpeedRange>,
    brightness: Option<BrightnessRange>,
) -> ModeDescription {
    ModeDescription {
        name: name.into(),
        value: mode as u32,
        color_mode,
        speed,
        brightness,
    }
}

#[must_use]
pub fn description() -> ControllerDescription {
    let brightness = Some(BrightnessRange {
        min: 0,
        max: 9,
        current: 9,
    });
    let speed = Some(SpeedRange {
        min: 10,
        max: 6,
        current: 9,
    });
    ControllerDescription {
        name: "Gigabyte AORUS C300 GLASS".into(),
        vendor: "Gigabyte".into(),
        description: "Gigabyte AORUS PC Case Device".into(),
        device_type: DeviceType::Case,
        modes: vec![
            mode_description(
                "Custom",
                AorusCaseMode::Custom,
                ModeColorMode::PerLed,
                None,
                brightness,
            ),
            mode_description("Off", AorusCaseMode::Off, ModeColorMode::None, None, None),
            mode_description(
                "Breathing",
                AorusCaseMode::Breathing,
                ModeColorMode::PerLed,
                speed,
                None,
            ),
            mode_description(
                "Spectrum Cycle",
                AorusCaseMode::SpectrumCycle,
                ModeColorMode::None,
                speed,
                None,
            ),
            mode_description(
                "Flashing",
                AorusCaseMode::Flashing,
                ModeColorMode::PerLed,
                speed,
                brightness,
            ),
            mode_description(
                "Double Flashing",
                AorusCaseMode::DoubleFlashing,
                ModeColorMode::PerLed,
                speed,
                brightness,
            ),
        ],
        zone_names: vec!["Case".into()],
        led_names: vec!["Case".into()],
        capabilities: ControllerCapabilities::PER_LED_COLOR
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

    fn endpoint(interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"aorus-case-test"[..]),
            0x1044,
            0x7A30,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn matcher_is_exact() {
        assert!(matches(&endpoint(0, 0xFF01, 1)));
        assert!(!matches(&endpoint(1, 0xFF01, 1)));
        assert!(!matches(&endpoint(0, 0xFF00, 1)));
        assert!(!matches(&endpoint(0, 0xFF01, 2)));
    }

    #[test]
    fn custom_and_off_packets_preserve_native_normalization() {
        let custom =
            ModeTransaction::new(AorusCaseMode::Custom, Rgb8::new(0x11, 0x22, 0x33), 7, 255)
                .unwrap();
        assert_eq!(
            custom.reports()[0].as_bytes(),
            &[0, 1, 0xC8, 0x11, 0x22, 0x33, 8, 1, 0]
        );
        assert_eq!(
            custom.reports()[1].as_bytes(),
            &[0, 1, 0xC9, 1, 7, 9, 1, 8, 0]
        );

        let off = ModeTransaction::new(AorusCaseMode::Off, Rgb8::new(1, 2, 3), 255, 255).unwrap();
        assert_eq!(off.reports()[0].as_bytes(), &[0, 1, 0xC8, 0, 0, 0, 8, 1, 0]);
        assert_eq!(
            off.reports()[1].as_bytes(),
            &[0, 1, 0xC9, 1, 10, 10, 1, 8, 0]
        );
    }

    #[test]
    fn animated_modes_preserve_overrides_scaling_and_commit() {
        let spectrum =
            ModeTransaction::new(AorusCaseMode::SpectrumCycle, Rgb8::new(1, 2, 3), 255, 6).unwrap();
        assert_eq!(
            spectrum.reports()[0].as_bytes(),
            &[0, 1, 0xC8, 0xFF, 0, 0, 8, 1, 0]
        );
        assert_eq!(
            spectrum.reports()[1].as_bytes(),
            &[0, 1, 0xC9, 3, 9, 6, 1, 8, 0]
        );

        let flashing =
            ModeTransaction::new(AorusCaseMode::DoubleFlashing, Rgb8::new(4, 5, 6), 9, 10).unwrap();
        assert_eq!(
            flashing.reports()[1].as_bytes(),
            &[0, 1, 0xC9, 5, 90, 10, 1, 8, 0]
        );
        assert_eq!(
            flashing.reports()[2].as_bytes(),
            &[0, 1, 0xB6, 0, 0, 0, 0, 0, 0]
        );

        let mut writer = RecordingWriter::default();
        flashing.apply(&mut writer).unwrap();
        assert_eq!(writer.0.len(), 3);
    }

    #[test]
    fn bounds_and_controller_shape_are_preserved() {
        assert!(ModeTransaction::new(AorusCaseMode::Custom, Rgb8::BLACK, 10, 0).is_err());
        assert!(ModeTransaction::new(AorusCaseMode::Breathing, Rgb8::BLACK, 0, 5).is_err());
        let device = description();
        assert_eq!(device.modes.len(), 6);
        assert_eq!(device.zone_names, ["Case"]);
        assert_eq!(device.led_names, ["Case"]);
        assert_eq!(device.modes[2].speed.unwrap().current, 9);
    }
}
