#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const FEATURE_REPORT_LEN: usize = 32;
pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x3938,
    product_id: 0x1162,
    interface_number: Some(1),
    usage_page: Some(0xFF19),
    usage: Some(0xFF19),
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AocMode {
    Static = 0x00,
    SpectrumCycle = 0x80,
    Breathing = 0x01,
    BreathingRandom = 0x81,
    Flashing = 0x02,
    FlashingRandom = 0x82,
    Wave = 0x03,
    RainbowWave = 0x83,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Clockwise = 0,
    CounterClockwise = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidSettings {
    pub brightness: u8,
    pub speed: u8,
}

impl fmt::Display for InvalidSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AOC brightness must be 0..3 and speed must be 1..3, got brightness {} and speed {}",
            self.brightness, self.speed
        )
    }
}

impl std::error::Error for InvalidSettings {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction(OutputReport<FEATURE_REPORT_LEN>);

impl ModeTransaction {
    /// Builds the complete native AOC mode state report.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidSettings`] for values outside the native UI ranges.
    pub fn new(
        mode: AocMode,
        color: Rgb8,
        brightness: u8,
        speed: u8,
        direction: Direction,
    ) -> Result<Self, InvalidSettings> {
        if brightness > 3 || !(1..=3).contains(&speed) {
            return Err(InvalidSettings { brightness, speed });
        }
        let mut bytes = [0; FEATURE_REPORT_LEN];
        bytes[0..9].copy_from_slice(&[
            0x20,
            brightness,
            speed,
            direction as u8,
            0x01,
            mode as u8,
            color.r,
            color.g,
            color.b,
        ]);
        for index in [9, 12, 15, 16, 19, 23, 25, 26, 27, 29] {
            bytes[index] = 0xFF;
        }
        bytes[13] = 0x3F;
        bytes[30] = 0x32;
        bytes[31] = 0x32;
        Ok(Self(OutputReport::from_array(bytes)))
    }

    #[must_use]
    pub const fn direct(color: Rgb8) -> Self {
        let mut bytes = [0; FEATURE_REPORT_LEN];
        bytes[0] = 0x20;
        bytes[1] = 3;
        bytes[2] = 2;
        bytes[4] = 1;
        bytes[6] = color.r;
        bytes[7] = color.g;
        bytes[8] = color.b;
        bytes[9] = 0xFF;
        bytes[12] = 0xFF;
        bytes[13] = 0x3F;
        bytes[15] = 0xFF;
        bytes[16] = 0xFF;
        bytes[19] = 0xFF;
        bytes[23] = 0xFF;
        bytes[25] = 0xFF;
        bytes[26] = 0xFF;
        bytes[27] = 0xFF;
        bytes[29] = 0xFF;
        bytes[30] = 0x32;
        bytes[31] = 0x32;
        Self(OutputReport::from_array(bytes))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<FEATURE_REPORT_LEN> {
        &self.0
    }

    /// Sends one complete 32-byte AOC feature report.
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

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description(device_name: &str) -> ControllerDescription {
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
    let modes = [
        ("Direct", AocMode::Static, ModeColorMode::PerLed, None),
        (
            "Spectrum Cycle",
            AocMode::SpectrumCycle,
            ModeColorMode::None,
            speed,
        ),
        (
            "Breathing",
            AocMode::Breathing,
            ModeColorMode::PerLed,
            speed,
        ),
        ("Flashing", AocMode::Flashing, ModeColorMode::PerLed, speed),
        ("Wave", AocMode::Wave, ModeColorMode::PerLed, speed),
        (
            "Rainbow Wave",
            AocMode::RainbowWave,
            ModeColorMode::None,
            speed,
        ),
    ];
    ControllerDescription {
        name: device_name.into(),
        vendor: "AOC".into(),
        description: "AOC Mousemat Device".into(),
        device_type: DeviceType::MouseMat,
        modes: modes
            .into_iter()
            .map(|(name, mode, color_mode, speed)| ModeDescription {
                name: name.into(),
                value: mode as u32,
                color_mode,
                speed,
                brightness,
            })
            .collect(),
        zone_names: vec!["Mousemat".into()],
        led_names: vec!["Mousemat".into()],
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
            Arc::from(&b"aoc-test"[..]),
            0x3938,
            0x1162,
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
        assert!(!matches(&endpoint(1, 0xFF19, 0xFF18)));
    }

    #[test]
    fn direct_and_effect_reports_preserve_native_bytes() {
        let direct = ModeTransaction::direct(Rgb8::new(1, 2, 3));
        assert_eq!(
            &direct.report().as_bytes()[..9],
            &[0x20, 3, 2, 0, 1, 0, 1, 2, 3]
        );
        let wave = ModeTransaction::new(
            AocMode::RainbowWave,
            Rgb8::new(4, 5, 6),
            2,
            1,
            Direction::CounterClockwise,
        )
        .unwrap();
        assert_eq!(
            &wave.report().as_bytes()[..9],
            &[0x20, 2, 1, 1, 1, 0x83, 4, 5, 6]
        );
        assert_eq!(&wave.report().as_bytes()[29..], &[0xFF, 0x32, 0x32]);
    }

    #[test]
    fn settings_and_model_shape_are_exact() {
        assert!(
            ModeTransaction::new(AocMode::Wave, Rgb8::BLACK, 4, 2, Direction::Clockwise).is_err()
        );
        assert!(
            ModeTransaction::new(AocMode::Wave, Rgb8::BLACK, 3, 0, Direction::Clockwise).is_err()
        );
        let device = description("AOC AGON AMM700");
        assert_eq!(device.modes.len(), 6);
        assert_eq!(device.zone_names, ["Mousemat"]);
        assert_eq!(device.modes[5].value, 0x83);
    }
}
