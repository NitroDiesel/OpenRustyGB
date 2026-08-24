#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const REPORT_LEN: usize = 65;
pub const LED_COUNT: usize = 4;
const WHITE: Rgb8 = Rgb8::new(0xFF, 0xFF, 0xFF);
pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x306F,
    product_id: 0x1234,
    interface_number: None,
    usage_page: None,
    usage: None,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum WushiMode {
    Direct = 1,
    Breathing = 3,
    RainbowWave = 4,
    SpectrumCycle = 6,
    RaceCycle = 8,
    Stacking = 10,
}

impl WushiMode {
    const fn has_speed(self) -> bool {
        !matches!(self, Self::Direct)
    }

    const fn has_brightness(self) -> bool {
        matches!(self, Self::Direct | Self::Stacking)
    }

    const fn has_direction(self) -> bool {
        matches!(self, Self::RainbowWave | Self::RaceCycle | Self::Stacking)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidSettings {
    Speed(u8),
    Brightness(u8),
}

impl fmt::Display for InvalidSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Speed(value) => write!(f, "Wushi L50 speed must be in 1..=4, got {value}"),
            Self::Brightness(value) => {
                write!(f, "Wushi L50 brightness must be in 1..=2, got {value}")
            }
        }
    }
}

impl std::error::Error for InvalidSettings {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction(OutputReport<REPORT_LEN>);

impl ModeTransaction {
    /// Builds one platform-correct native hardware-mode feature report.
    ///
    /// # Errors
    ///
    /// Returns an error for a mode-dependent speed or brightness outside the
    /// native range.
    pub fn new(
        mode: WushiMode,
        colors: [Rgb8; LED_COUNT],
        brightness: u8,
        speed: u8,
        direction: Direction,
    ) -> Result<Self, InvalidSettings> {
        if mode.has_speed() && !(1..=4).contains(&speed) {
            return Err(InvalidSettings::Speed(speed));
        }
        if mode.has_brightness() && !(1..=2).contains(&brightness) {
            return Err(InvalidSettings::Brightness(brightness));
        }

        let mut report = [0; REPORT_LEN];
        let offset = platform_offset(&mut report);
        report[offset] = 0x16;
        report[offset + 1] = mode as u8;
        report[offset + 2] = if mode.has_speed() { speed } else { 1 };
        report[offset + 3] = if mode.has_brightness() { brightness } else { 1 };

        let effective_colors = match mode {
            WushiMode::Direct => colors,
            WushiMode::Breathing => [colors[0], WHITE, WHITE, WHITE],
            WushiMode::RainbowWave
            | WushiMode::SpectrumCycle
            | WushiMode::RaceCycle
            | WushiMode::Stacking => [WHITE; LED_COUNT],
        };
        for (index, color) in effective_colors.iter().enumerate() {
            let start = offset + 4 + index * 3;
            report[start..start + 3].copy_from_slice(&[color.r, color.g, color.b]);
        }
        if mode.has_direction() {
            match direction {
                Direction::Left => report[offset + 0x11] = 1,
                Direction::Right => report[offset + 0x12] = 1,
            }
        }
        Ok(Self(OutputReport::from_array(report)))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<REPORT_LEN> {
        &self.0
    }

    /// Sends the native feature report.
    ///
    /// # Errors
    ///
    /// Returns an error from the feature-report transport.
    pub fn apply<W: FeatureWriter<REPORT_LEN>>(&self, writer: &mut W) -> Result<(), W::Error> {
        send_feature(writer, &self.0)
    }
}

#[cfg(target_os = "windows")]
fn platform_offset(report: &mut [u8; REPORT_LEN]) -> usize {
    report[0] = 0xCC;
    1
}

#[cfg(not(target_os = "windows"))]
fn platform_offset(_report: &mut [u8; REPORT_LEN]) -> usize {
    0
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

fn mode_description(
    name: &str,
    value: u32,
    color_mode: ModeColorMode,
    speed: Option<SpeedRange>,
    brightness: Option<BrightnessRange>,
) -> ModeDescription {
    ModeDescription {
        name: name.into(),
        value,
        color_mode,
        speed,
        brightness,
    }
}

#[must_use]
pub fn description(device_name: &str) -> ControllerDescription {
    let speed = || {
        Some(SpeedRange {
            min: 1,
            max: 4,
            current: 3,
        })
    };
    let brightness = || {
        Some(BrightnessRange {
            min: 1,
            max: 2,
            current: 2,
        })
    };
    ControllerDescription {
        name: device_name.into(),
        vendor: "Wushi".into(),
        description: "Wushi L50 device".into(),
        device_type: DeviceType::Accessory,
        modes: vec![
            mode_description("Direct", 1, ModeColorMode::PerLed, None, brightness()),
            mode_description("Breathing", 3, ModeColorMode::PerLed, speed(), None),
            mode_description("Rainbow Wave", 4, ModeColorMode::None, speed(), None),
            mode_description("Spectrum Cycle", 6, ModeColorMode::None, speed(), None),
            mode_description("Race Cycle", 8, ModeColorMode::None, speed(), None),
            mode_description("Stacking", 10, ModeColorMode::None, speed(), brightness()),
        ],
        zone_names: vec!["Dock".into()],
        led_names: (1..=LED_COUNT)
            .map(|number| format!("Dock Zone {number}"))
            .collect(),
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
            Arc::from(&b"wushi-test"[..]),
            0x306F,
            0x1234,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn matcher_preserves_product_only_detection() {
        assert!(matches(&endpoint(0, 1, 2)));
        assert!(matches(&endpoint(9, 0xFF00, 7)));
        let mut other = endpoint(0, 1, 2);
        other.product_id = 0x1235;
        assert!(!matches(&other));
    }

    #[test]
    fn direct_packet_preserves_platform_framing_and_four_colors() {
        let transaction = ModeTransaction::new(
            WushiMode::Direct,
            [
                Rgb8::new(1, 2, 3),
                Rgb8::new(4, 5, 6),
                Rgb8::new(7, 8, 9),
                Rgb8::new(10, 11, 12),
            ],
            2,
            99,
            Direction::Right,
        )
        .unwrap();
        let bytes = transaction.report().as_bytes();
        #[cfg(target_os = "windows")]
        let offset = {
            assert_eq!(bytes[0], 0xCC);
            1
        };
        #[cfg(not(target_os = "windows"))]
        let offset = 0;
        assert_eq!(&bytes[offset..offset + 4], &[0x16, 1, 1, 2]);
        assert_eq!(
            &bytes[offset + 4..offset + 16],
            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
        );
        assert_eq!(bytes[offset + 0x11], 0);
        assert_eq!(bytes[offset + 0x12], 0);
    }

    #[test]
    fn effects_preserve_defaults_direction_and_bounds() {
        let breathing = ModeTransaction::new(
            WushiMode::Breathing,
            [Rgb8::new(1, 2, 3); LED_COUNT],
            99,
            3,
            Direction::Left,
        )
        .unwrap();
        let offset = usize::from(cfg!(target_os = "windows"));
        assert_eq!(
            &breathing.report().as_bytes()[offset..offset + 11],
            &[0x16, 3, 3, 1, 1, 2, 3, 0xFF, 0xFF, 0xFF, 0xFF]
        );

        let stacking = ModeTransaction::new(
            WushiMode::Stacking,
            [Rgb8::BLACK; LED_COUNT],
            2,
            4,
            Direction::Right,
        )
        .unwrap();
        assert_eq!(stacking.report().as_bytes()[offset + 0x12], 1);
        assert!(
            ModeTransaction::new(
                WushiMode::RaceCycle,
                [Rgb8::BLACK; LED_COUNT],
                1,
                0,
                Direction::Left,
            )
            .is_err()
        );
    }

    #[test]
    fn model_preserves_six_modes_and_four_zones() {
        let device = description("JSAUX RGB Docking Station");
        assert_eq!(device.modes.len(), 6);
        assert_eq!(device.led_names.len(), LED_COUNT);
        assert_eq!(device.led_names[3], "Dock Zone 4");
    }
}
