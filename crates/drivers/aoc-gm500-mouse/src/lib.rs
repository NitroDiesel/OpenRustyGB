#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const FEATURE_REPORT_LEN: usize = 60;
pub const LED_COUNT: usize = 2;
pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x3938,
    product_id: 0x1179,
    interface_number: Some(1),
    usage_page: Some(0xFF19),
    usage: Some(0xFF19),
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AocMouseMode {
    Static = 0x00,
    SpectrumCycle = 0x80,
    Breathing = 0x01,
    BreathingRandom = 0x81,
    Flashing = 0x02,
    FlashingRandom = 0x82,
    Wave = 0x03,
    RainbowWave = 0x83,
    Dpi = 0x04,
}

impl AocMouseMode {
    const fn uses_speed(self) -> bool {
        !matches!(self, Self::Static | Self::Dpi)
    }

    const fn uses_direction(self) -> bool {
        matches!(self, Self::Wave | Self::RainbowWave)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Clockwise = 0,
    CounterClockwise = 1,
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
                write!(f, "AOC GM500 brightness must be in 0..=3, got {value}")
            }
            Self::Speed(value) => write!(f, "AOC GM500 speed must be in 1..=3, got {value}"),
        }
    }
}

impl std::error::Error for InvalidSettings {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction(OutputReport<FEATURE_REPORT_LEN>);

impl ModeTransaction {
    /// Builds the native Direct report. The upstream implementation always
    /// applies high brightness, medium speed, and clockwise direction here.
    #[must_use]
    pub fn direct(colors: [Rgb8; LED_COUNT]) -> Self {
        Self::build(AocMouseMode::Static, 3, 2, Direction::Clockwise, colors)
    }

    /// Builds one complete native hardware-mode feature report.
    ///
    /// # Errors
    /// Returns an error for settings outside the ranges exposed upstream.
    pub fn new(
        mode: AocMouseMode,
        colors: [Rgb8; LED_COUNT],
        brightness: u8,
        speed: u8,
        direction: Direction,
    ) -> Result<Self, InvalidSettings> {
        if brightness > 3 {
            return Err(InvalidSettings::Brightness(brightness));
        }
        if mode.uses_speed() && !(1..=3).contains(&speed) {
            return Err(InvalidSettings::Speed(speed));
        }
        let speed = if mode == AocMouseMode::Dpi { 0 } else { speed };
        let direction = if mode.uses_direction() {
            direction
        } else {
            Direction::Clockwise
        };
        Ok(Self::build(mode, brightness, speed, direction, colors))
    }

    fn build(
        mode: AocMouseMode,
        brightness: u8,
        speed: u8,
        direction: Direction,
        colors: [Rgb8; LED_COUNT],
    ) -> Self {
        let mut bytes = [0; FEATURE_REPORT_LEN];
        bytes[0..10].copy_from_slice(&[
            0x20,
            0x03,
            0x01,
            mode as u8,
            speed,
            brightness,
            direction as u8,
            0x01,
            0x02,
            0xFF,
        ]);
        for (index, value) in [
            (0x0D, 0x01),
            (0x0E, 0x03),
            (0x0F, 0xFF),
            (0x10, 0x7F),
            (0x13, 0x01),
            (0x14, 0x04),
            (0x17, 0xFF),
            (0x19, 0x01),
            (0x1A, 0x05),
            (0x1C, 0xFF),
            (0x1F, 0x01),
            (0x20, 0x06),
            (0x21, 0xFF),
            (0x23, 0xFF),
            (0x25, 0x01),
            (0x26, 0x07),
            (0x27, 0xFF),
            (0x28, 0xFF),
            (0x2C, 0x0A),
            (0x2D, 0x0A),
            (0x30, 0x14),
            (0x31, 0x08),
            (0x3A, 0x32),
            (0x3B, 0x32),
        ] {
            bytes[index] = value;
        }
        bytes[0x33..0x36].copy_from_slice(&[colors[0].r, colors[0].g, colors[0].b]);
        bytes[0x36..0x39].copy_from_slice(&[colors[1].r, colors[1].g, colors[1].b]);
        Self(OutputReport::from_array(bytes))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<FEATURE_REPORT_LEN> {
        &self.0
    }

    /// Sends the complete native feature report.
    ///
    /// # Errors
    /// Returns the HID feature-report transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        send_feature(writer, &self.0)
    }
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

fn mode_description(
    name: &str,
    mode: AocMouseMode,
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
        max: 3,
        current: 3,
    });
    let speed = Some(SpeedRange {
        min: 3,
        max: 1,
        current: 2,
    });
    ControllerDescription {
        name: "AOC GM500".into(),
        vendor: "AOC".into(),
        description: "AOC Mouse Device".into(),
        device_type: DeviceType::Mouse,
        modes: vec![
            mode_description(
                "Direct",
                AocMouseMode::Static,
                ModeColorMode::PerLed,
                None,
                brightness,
            ),
            mode_description(
                "Spectrum Cycle",
                AocMouseMode::SpectrumCycle,
                ModeColorMode::None,
                speed,
                brightness,
            ),
            mode_description(
                "Breathing",
                AocMouseMode::Breathing,
                ModeColorMode::PerLed,
                speed,
                brightness,
            ),
            mode_description(
                "Flashing",
                AocMouseMode::Flashing,
                ModeColorMode::PerLed,
                speed,
                brightness,
            ),
            mode_description(
                "Wave",
                AocMouseMode::Wave,
                ModeColorMode::PerLed,
                speed,
                brightness,
            ),
            mode_description(
                "Rainbow Wave",
                AocMouseMode::RainbowWave,
                ModeColorMode::None,
                speed,
                brightness,
            ),
            mode_description(
                "DPI",
                AocMouseMode::Dpi,
                ModeColorMode::None,
                None,
                brightness,
            ),
        ],
        zone_names: vec!["Logo".into(), "Scroll Wheel".into()],
        led_names: vec!["Logo".into(), "Scroll Wheel".into()],
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::EFFECTS)
            .union(ControllerCapabilities::BRIGHTNESS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn endpoint(interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"aoc-gm500-test"[..]),
            0x3938,
            0x1179,
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
        assert!(matches(&endpoint(1, 0xFF19, 0xFF19)));
        assert!(!matches(&endpoint(0, 0xFF19, 0xFF19)));
        assert!(!matches(&endpoint(1, 0xFF18, 0xFF19)));
        assert!(!matches(&endpoint(1, 0xFF19, 0xFF18)));
    }

    #[test]
    fn direct_report_preserves_fixed_settings_and_both_leds() {
        let report = ModeTransaction::direct([Rgb8::new(1, 2, 3), Rgb8::new(4, 5, 6)]);
        assert_eq!(
            &report.report().as_bytes()[..10],
            &[0x20, 0x03, 0x01, 0, 2, 3, 0, 1, 2, 0xFF]
        );
        assert_eq!(&report.report().as_bytes()[0x33..0x39], &[1, 2, 3, 4, 5, 6]);
        assert_eq!(&report.report().as_bytes()[0x3A..], &[0x32, 0x32]);
    }

    #[test]
    fn random_modes_and_settings_match_native_protocol() {
        let report = ModeTransaction::new(
            AocMouseMode::FlashingRandom,
            [Rgb8::BLACK; LED_COUNT],
            2,
            1,
            Direction::CounterClockwise,
        )
        .unwrap();
        assert_eq!(&report.report().as_bytes()[3..7], &[0x82, 1, 2, 0]);
        let wave = ModeTransaction::new(
            AocMouseMode::Wave,
            [Rgb8::BLACK; LED_COUNT],
            2,
            1,
            Direction::CounterClockwise,
        )
        .unwrap();
        assert_eq!(&wave.report().as_bytes()[3..7], &[3, 1, 2, 1]);
        assert!(
            ModeTransaction::new(
                AocMouseMode::Wave,
                [Rgb8::BLACK; LED_COUNT],
                4,
                2,
                Direction::Clockwise,
            )
            .is_err()
        );
        assert!(
            ModeTransaction::new(
                AocMouseMode::Wave,
                [Rgb8::BLACK; LED_COUNT],
                3,
                0,
                Direction::Clockwise,
            )
            .is_err()
        );
        assert!(
            ModeTransaction::new(
                AocMouseMode::Dpi,
                [Rgb8::BLACK; LED_COUNT],
                3,
                3,
                Direction::CounterClockwise,
            )
            .is_ok()
        );
        let dpi = ModeTransaction::new(
            AocMouseMode::Dpi,
            [Rgb8::BLACK; LED_COUNT],
            3,
            3,
            Direction::CounterClockwise,
        )
        .unwrap();
        assert_eq!(&dpi.report().as_bytes()[3..7], &[4, 0, 3, 0]);
    }

    #[test]
    fn public_model_shape_is_preserved() {
        let device = description();
        assert_eq!(device.modes.len(), 7);
        assert_eq!(device.zone_names, ["Logo", "Scroll Wheel"]);
        assert_eq!(device.led_names, ["Logo", "Scroll Wheel"]);
        assert_eq!(device.modes[6].value, 4);
    }
}
