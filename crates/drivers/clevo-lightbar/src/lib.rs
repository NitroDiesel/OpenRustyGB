#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const FEATURE_REPORT_LEN: usize = 8;
pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x048D,
    product_id: 0x7001,
    interface_number: None,
    usage_page: Some(0xFF03),
    usage: Some(0x0002),
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ClevoMode {
    Off = 0,
    Direct = 1,
    Breathing = 2,
    Wave = 3,
    Bounce = 4,
    Marquee = 5,
    Scan = 6,
}

impl ClevoMode {
    const fn uses_color(self) -> bool {
        matches!(
            self,
            Self::Direct | Self::Breathing | Self::Bounce | Self::Scan
        )
    }

    const fn uses_speed(self) -> bool {
        !matches!(self, Self::Off | Self::Direct)
    }
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
                write!(f, "CLEVO brightness must be in 0..=100, got {value}")
            }
            Self::Speed(value) => write!(f, "CLEVO speed must be in 1..=10, got {value}"),
        }
    }
}

impl std::error::Error for InvalidSettings {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColorTransaction(OutputReport<FEATURE_REPORT_LEN>);

impl ColorTransaction {
    #[must_use]
    pub const fn new(color: Rgb8) -> Self {
        Self(OutputReport::from_array([
            0x14, 0, 1, color.r, color.g, color.b, 0, 0,
        ]))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<FEATURE_REPORT_LEN> {
        &self.0
    }

    /// Sends one native color feature report.
    ///
    /// # Errors
    ///
    /// Returns the feature-report transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        send_feature(writer, &self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrightnessTransaction(OutputReport<FEATURE_REPORT_LEN>);

impl BrightnessTransaction {
    /// Builds the native mono-mode brightness report.
    ///
    /// # Errors
    ///
    /// Returns an error above the native `0..=100` range.
    pub fn new(brightness: u8) -> Result<Self, InvalidSettings> {
        if brightness > 100 {
            return Err(InvalidSettings::Brightness(brightness));
        }
        Ok(Self(OutputReport::from_array([
            0x08, 0x22, 1, 1, brightness, 1, 0, 0,
        ])))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<FEATURE_REPORT_LEN> {
        &self.0
    }

    /// Sends one native brightness feature report.
    ///
    /// # Errors
    ///
    /// Returns the feature-report transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        send_feature(writer, &self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction {
    reports: Vec<OutputReport<FEATURE_REPORT_LEN>>,
}

impl ModeTransaction {
    /// Builds the native mode transaction, including color or four-report off
    /// sequences where required.
    ///
    /// # Errors
    ///
    /// Returns an error for mode-dependent settings outside the native ranges.
    pub fn new(
        mode: ClevoMode,
        color: Rgb8,
        brightness: u8,
        speed: u8,
    ) -> Result<Self, InvalidSettings> {
        if mode != ClevoMode::Off && brightness > 100 {
            return Err(InvalidSettings::Brightness(brightness));
        }
        if mode.uses_speed() && !(1..=10).contains(&speed) {
            return Err(InvalidSettings::Speed(speed));
        }

        if mode == ClevoMode::Off {
            return Ok(Self {
                reports: vec![
                    OutputReport::from_array([0x12, 0, 3, 0, 0, 0, 0, 0]),
                    OutputReport::from_array([0x08, 5, 0, 0, 0, 0, 0, 0]),
                    OutputReport::from_array([0x08, 1, 0, 0, 0, 0, 0, 0]),
                    OutputReport::from_array([0x1A, 0, 0, 0, 0, 0, 0, 1]),
                ],
            });
        }

        let mut reports = Vec::with_capacity(2);
        if mode.uses_color() {
            reports.push(ColorTransaction::new(color).0);
        }
        let protocol_speed = if mode == ClevoMode::Direct {
            11
        } else {
            11 - speed
        };
        reports.push(OutputReport::from_array([
            0x08,
            0x22,
            mode as u8,
            protocol_speed,
            brightness,
            1,
            0,
            0,
        ]));
        Ok(Self { reports })
    }

    #[must_use]
    pub fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>] {
        &self.reports
    }

    /// Sends the complete native transaction in order.
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

#[must_use]
pub fn firmware_version(release_number: u16) -> String {
    format!("{}.{:02}", release_number >> 8, release_number & 0xFF)
}

fn mode_description(
    name: &str,
    mode: ClevoMode,
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
        max: 100,
        current: 100,
    });
    let speed = Some(SpeedRange {
        min: 1,
        max: 10,
        current: 5,
    });
    ControllerDescription {
        name: "CLEVO Lightbar".into(),
        vendor: "CLEVO Computers".into(),
        description: "CLEVO Laptop Lightbar".into(),
        device_type: DeviceType::LedStrip,
        modes: vec![
            mode_description(
                "Direct",
                ClevoMode::Direct,
                ModeColorMode::PerLed,
                None,
                brightness,
            ),
            mode_description(
                "Breathing",
                ClevoMode::Breathing,
                ModeColorMode::PerLed,
                speed,
                brightness,
            ),
            mode_description(
                "Wave",
                ClevoMode::Wave,
                ModeColorMode::None,
                speed,
                brightness,
            ),
            mode_description(
                "Bounce",
                ClevoMode::Bounce,
                ModeColorMode::PerLed,
                speed,
                brightness,
            ),
            mode_description(
                "Marquee",
                ClevoMode::Marquee,
                ModeColorMode::None,
                speed,
                brightness,
            ),
            mode_description(
                "Scan",
                ClevoMode::Scan,
                ModeColorMode::PerLed,
                speed,
                brightness,
            ),
            mode_description("Off", ClevoMode::Off, ModeColorMode::None, None, None),
        ],
        zone_names: vec!["Lightbar".into()],
        led_names: vec!["Lightbar".into()],
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::EFFECTS)
            .union(ControllerCapabilities::BRIGHTNESS),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn endpoint(interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"clevo-test"[..]),
            0x048D,
            0x7001,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
        .with_release_number(0x0307)
    }

    #[test]
    fn matcher_preserves_product_and_usage_without_interface_constraint() {
        assert!(matches(&endpoint(0, 0xFF03, 2)));
        assert!(matches(&endpoint(4, 0xFF03, 2)));
        assert!(!matches(&endpoint(0, 0xFF00, 2)));
        assert!(!matches(&endpoint(0, 0xFF03, 1)));
    }

    #[test]
    fn direct_and_wave_packets_preserve_color_and_speed_rules() {
        let direct = ModeTransaction::new(ClevoMode::Direct, Rgb8::new(1, 2, 3), 100, 0).unwrap();
        assert_eq!(direct.reports().len(), 2);
        assert_eq!(direct.reports()[0].as_bytes(), &[0x14, 0, 1, 1, 2, 3, 0, 0]);
        assert_eq!(
            direct.reports()[1].as_bytes(),
            &[8, 0x22, 1, 11, 100, 1, 0, 0]
        );

        let wave = ModeTransaction::new(ClevoMode::Wave, Rgb8::new(9, 9, 9), 50, 10).unwrap();
        assert_eq!(wave.reports().len(), 1);
        assert_eq!(wave.reports()[0].as_bytes(), &[8, 0x22, 3, 1, 50, 1, 0, 0]);
    }

    #[test]
    fn off_sequence_and_brightness_report_are_exact() {
        let off = ModeTransaction::new(ClevoMode::Off, Rgb8::BLACK, 255, 0).unwrap();
        assert_eq!(off.reports().len(), 4);
        assert_eq!(off.reports()[0].as_bytes(), &[0x12, 0, 3, 0, 0, 0, 0, 0]);
        assert_eq!(off.reports()[3].as_bytes(), &[0x1A, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(
            BrightnessTransaction::new(42).unwrap().report().as_bytes(),
            &[8, 0x22, 1, 1, 42, 1, 0, 0]
        );
    }

    #[test]
    fn bounds_firmware_and_controller_shape_are_preserved() {
        assert!(ModeTransaction::new(ClevoMode::Direct, Rgb8::BLACK, 101, 0).is_err());
        assert!(ModeTransaction::new(ClevoMode::Breathing, Rgb8::BLACK, 100, 0).is_err());
        assert_eq!(firmware_version(0x0307), "3.07");
        let device = description();
        assert_eq!(device.modes.len(), 7);
        assert_eq!(device.zone_names, ["Lightbar"]);
        assert_eq!(device.led_names, ["Lightbar"]);
    }
}
