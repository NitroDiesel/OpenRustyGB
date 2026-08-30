#![forbid(unsafe_code)]

use std::fmt;
use std::thread;
use std::time::Duration;

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const FEATURE_REPORT_LEN: usize = 521;
pub const LED_COUNT: usize = 87;
pub const SPEED_MIN: u8 = 0;
pub const SPEED_MAX: u8 = 4;
pub const EFFECT_BRIGHTNESS_MAX: u8 = 0x7F;
pub const CUSTOM_BRIGHTNESS_MAX: u8 = 0xFF;

pub const MATCH_TKL: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x1A2C,
    product_id: 0x1511,
    interface_number: Some(3),
    usage_page: Some(0xFF00),
    usage: Some(0x0002),
};

pub const MATCH_TKL_V2: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x1A2C,
    product_id: 0x2511,
    interface_number: Some(3),
    usage_page: Some(0xFF00),
    usage: Some(0x0002),
};

pub const MATCHES: [HidDeviceMatch; 2] = [MATCH_TKL, MATCH_TKL_V2];

const LED_SEQUENCE_POSITIONS: [usize; LED_COUNT] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 70, 51, 52, 53,
    58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 50, 75, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86,
    88, 89, 94, 95, 96, 98, 100, 101, 102, 103, 104, 105, 106,
];

pub const MATRIX_MAP: [[Option<u8>; 18]; 6] = [
    [
        Some(0),
        None,
        Some(1),
        Some(2),
        Some(3),
        Some(4),
        Some(5),
        Some(6),
        Some(7),
        Some(8),
        None,
        Some(9),
        Some(10),
        Some(11),
        Some(12),
        Some(13),
        Some(14),
        Some(15),
    ],
    [
        Some(16),
        Some(17),
        Some(18),
        Some(19),
        Some(20),
        Some(21),
        Some(22),
        Some(23),
        Some(24),
        Some(25),
        Some(26),
        Some(27),
        Some(28),
        Some(29),
        None,
        Some(30),
        Some(31),
        Some(32),
    ],
    [
        Some(33),
        None,
        Some(34),
        Some(35),
        Some(36),
        Some(37),
        Some(38),
        Some(39),
        Some(40),
        Some(41),
        Some(42),
        Some(43),
        Some(44),
        Some(45),
        Some(46),
        Some(47),
        Some(48),
        Some(49),
    ],
    [
        Some(50),
        None,
        Some(51),
        Some(52),
        Some(53),
        Some(54),
        Some(55),
        Some(56),
        Some(57),
        Some(58),
        Some(59),
        Some(60),
        Some(61),
        None,
        Some(62),
        None,
        None,
        None,
    ],
    [
        Some(63),
        None,
        Some(64),
        Some(65),
        Some(66),
        Some(67),
        Some(68),
        Some(69),
        Some(70),
        Some(71),
        Some(72),
        Some(73),
        None,
        Some(74),
        None,
        None,
        Some(75),
        None,
    ],
    [
        Some(76),
        Some(77),
        Some(78),
        None,
        None,
        None,
        Some(79),
        None,
        None,
        None,
        Some(80),
        Some(81),
        None,
        Some(82),
        Some(83),
        Some(84),
        Some(85),
        Some(86),
    ],
];

const LED_NAMES: [&str; LED_COUNT] = [
    "Key: Escape",
    "Key: F1",
    "Key: F2",
    "Key: F3",
    "Key: F4",
    "Key: F5",
    "Key: F6",
    "Key: F7",
    "Key: F8",
    "Key: F9",
    "Key: F10",
    "Key: F11",
    "Key: F12",
    "Key: Print Screen",
    "Key: Scroll Lock",
    "Key: Pause/Break",
    "Key: `",
    "Key: 1",
    "Key: 2",
    "Key: 3",
    "Key: 4",
    "Key: 5",
    "Key: 6",
    "Key: 7",
    "Key: 8",
    "Key: 9",
    "Key: 0",
    "Key: -",
    "Key: =",
    "Key: Backspace",
    "Key: Insert",
    "Key: Home",
    "Key: Page Up",
    "Key: Tab",
    "Key: Q",
    "Key: W",
    "Key: E",
    "Key: R",
    "Key: T",
    "Key: Y",
    "Key: U",
    "Key: I",
    "Key: O",
    "Key: P",
    "Key: [",
    "Key: ]",
    "Key: \\ (ANSI)",
    "Key: Delete",
    "Key: End",
    "Key: Page Down",
    "Key: Caps Lock",
    "Key: A",
    "Key: S",
    "Key: D",
    "Key: F",
    "Key: G",
    "Key: H",
    "Key: J",
    "Key: K",
    "Key: L",
    "Key: ;",
    "Key: '",
    "Key: Enter",
    "Key: Left Shift",
    "Key: Z",
    "Key: X",
    "Key: C",
    "Key: V",
    "Key: B",
    "Key: N",
    "Key: M",
    "Key: ,",
    "Key: .",
    "Key: /",
    "Key: Right Shift",
    "Key: Up Arrow",
    "Key: Left Control",
    "Key: Left Windows",
    "Key: Left Alt",
    "Key: Space",
    "Key: Right Alt",
    "Key: Right Fn",
    "Key: Menu",
    "Key: Right Control",
    "Key: Left Arrow",
    "Key: Down Arrow",
    "Key: Right Arrow",
];

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyroxMode {
    Wave = 0x00,
    Const = 0x01,
    Breathe = 0x02,
    Heartrate = 0x03,
    Point = 0x04,
    Winnower = 0x05,
    Stars = 0x06,
    Spectrum = 0x07,
    Plumflower = 0x08,
    Shoot = 0x09,
    AmbilightRotate = 0x0A,
    Ripple = 0x0B,
    Custom = 0x0C,
}

impl KeyroxMode {
    const fn uses_speed(self) -> bool {
        !matches!(self, Self::Const | Self::Custom)
    }

    const fn color_requirement(self) -> ColorRequirement {
        match self {
            Self::Const => ColorRequirement::Fixed,
            Self::Breathe
            | Self::Heartrate
            | Self::Point
            | Self::Stars
            | Self::Plumflower
            | Self::Shoot
            | Self::Ripple => ColorRequirement::RandomOrFixed,
            _ => ColorRequirement::None,
        }
    }

    const fn direction_requirement(self) -> DirectionRequirement {
        match self {
            Self::Wave => DirectionRequirement::All,
            Self::Winnower | Self::AmbilightRotate => DirectionRequirement::UpDown,
            _ => DirectionRequirement::None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ColorRequirement {
    None,
    Fixed,
    RandomOrFixed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DirectionRequirement {
    None,
    All,
    UpDown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModeColor {
    None,
    Random,
    Fixed(Rgb8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidSettings {
    ColorCount(usize),
    Brightness(u8),
    Speed(u8),
    Color(KeyroxMode, ModeColor),
    Direction(KeyroxMode, Direction),
    CustomHardwareMode,
}

impl fmt::Display for InvalidSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ColorCount(actual) => {
                write!(
                    f,
                    "Keyrox TKL requires exactly {LED_COUNT} colors, got {actual}"
                )
            }
            Self::Brightness(value) => write!(
                f,
                "Keyrox hardware-effect brightness must be in 0..={EFFECT_BRIGHTNESS_MAX}, got {value}"
            ),
            Self::Speed(value) => write!(
                f,
                "Keyrox effect speed must be in {SPEED_MIN}..={SPEED_MAX}, got {value}"
            ),
            Self::Color(mode, color) => {
                write!(
                    f,
                    "color setting {color:?} is invalid for Keyrox mode {mode:?}"
                )
            }
            Self::Direction(mode, direction) => write!(
                f,
                "direction {direction:?} is invalid for Keyrox mode {mode:?}"
            ),
            Self::CustomHardwareMode => write!(
                f,
                "Custom mode requires a per-key color transaction instead of hardware-effect settings"
            ),
        }
    }
}

impl std::error::Error for InvalidSettings {}

fn mode_report(mode: KeyroxMode) -> OutputReport<FEATURE_REPORT_LEN> {
    let mut bytes = [0; FEATURE_REPORT_LEN];
    bytes[5] = 0x01;
    bytes[7] = 0x04;
    bytes[9] = mode as u8;
    OutputReport::from_array(bytes)
}

fn direction_value(mode: KeyroxMode, direction: Direction) -> Result<u8, InvalidSettings> {
    match (mode.direction_requirement(), direction) {
        (DirectionRequirement::All, Direction::Left) => Ok(0x10),
        (DirectionRequirement::None, _) | (DirectionRequirement::All, Direction::Right) => Ok(0),
        (DirectionRequirement::All, Direction::Up) => Ok(0x20),
        (DirectionRequirement::All, Direction::Down) => Ok(0x30),
        (DirectionRequirement::UpDown, Direction::Up) => Ok(0xA0),
        (DirectionRequirement::UpDown, Direction::Down) => Ok(0xB0),
        _ => Err(InvalidSettings::Direction(mode, direction)),
    }
}

fn validate_color(mode: KeyroxMode, color: ModeColor) -> Result<(), InvalidSettings> {
    match (mode.color_requirement(), color) {
        (ColorRequirement::None, ModeColor::None)
        | (ColorRequirement::Fixed, ModeColor::Fixed(_))
        | (ColorRequirement::RandomOrFixed, ModeColor::Random | ModeColor::Fixed(_)) => Ok(()),
        _ => Err(InvalidSettings::Color(mode, color)),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustomColorTransaction([OutputReport<FEATURE_REPORT_LEN>; 2]);

impl CustomColorTransaction {
    /// Builds the Custom mode-selection and per-key feature reports.
    ///
    /// # Errors
    /// Returns an error unless exactly 87 logical colors are supplied.
    pub fn new(colors: &[Rgb8], brightness: u8) -> Result<Self, InvalidSettings> {
        if colors.len() != LED_COUNT {
            return Err(InvalidSettings::ColorCount(colors.len()));
        }
        let mut color_bytes = [0; FEATURE_REPORT_LEN];
        color_bytes[5] = 0xB0;
        color_bytes[6] = 0x01;
        color_bytes[7] = 0x07;
        for (color, position) in colors.iter().zip(LED_SEQUENCE_POSITIONS) {
            let offset = 9 + position * 4;
            color_bytes[offset..offset + 4]
                .copy_from_slice(&[color.r, color.g, color.b, brightness]);
        }
        Ok(Self([
            mode_report(KeyroxMode::Custom),
            OutputReport::from_array(color_bytes),
        ]))
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>; 2] {
        &self.0
    }

    /// Sends both native feature reports with the original 10 ms pacing.
    ///
    /// # Errors
    /// Returns the first HID feature-report transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        for report in &self.0 {
            send_feature(writer, report)?;
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HardwareModeTransaction([OutputReport<FEATURE_REPORT_LEN>; 2]);

impl HardwareModeTransaction {
    /// Builds the mode-selection and mode-data feature reports.
    ///
    /// # Errors
    /// Returns an error when a setting is outside the ranges or capabilities
    /// exposed by the upstream mode.
    pub fn new(
        mode: KeyroxMode,
        brightness: u8,
        speed: u8,
        direction: Direction,
        color: ModeColor,
    ) -> Result<Self, InvalidSettings> {
        if mode == KeyroxMode::Custom {
            return Err(InvalidSettings::CustomHardwareMode);
        }
        if brightness > EFFECT_BRIGHTNESS_MAX {
            return Err(InvalidSettings::Brightness(brightness));
        }
        if mode.uses_speed() && !(SPEED_MIN..=SPEED_MAX).contains(&speed) {
            return Err(InvalidSettings::Speed(speed));
        }
        validate_color(mode, color)?;
        let direction_value = direction_value(mode, direction)?;

        let mut bytes = [0; FEATURE_REPORT_LEN];
        bytes[5] = 0x09;
        bytes[7] = 0x05;
        bytes[8] = mode as u8;
        bytes[9] = brightness;
        bytes[10] = 0xFF;

        if mode.uses_speed() {
            bytes[11] = speed;
            bytes[12] = 0xFF;
            if mode == KeyroxMode::Spectrum {
                bytes[11] += 0x80;
            }
        }
        match color {
            ModeColor::None => {}
            ModeColor::Random => bytes[11] += 0x80,
            ModeColor::Fixed(rgb) => {
                bytes[12..15].copy_from_slice(&[rgb.r, rgb.g, rgb.b]);
            }
        }
        bytes[11] += direction_value;

        Ok(Self([mode_report(mode), OutputReport::from_array(bytes)]))
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>; 2] {
        &self.0
    }

    /// Sends both native feature reports with the original 10 ms pacing.
    ///
    /// # Errors
    /// Returns the first HID feature-report transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        for report in &self.0 {
            send_feature(writer, report)?;
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }
}

#[must_use]
pub fn match_model(endpoint: &HidEndpointInfo) -> Option<&'static str> {
    MATCHES
        .iter()
        .position(|matcher| matcher.matches(endpoint))
        .map(|index| match index {
            0 => "Red Square Keyrox TKL",
            _ => "Red Square Keyrox TKL V2",
        })
}

fn mode_description(
    name: &str,
    mode: KeyroxMode,
    color_mode: ModeColorMode,
    speed: Option<SpeedRange>,
    brightness: BrightnessRange,
) -> ModeDescription {
    ModeDescription {
        name: name.into(),
        value: mode as u32,
        color_mode,
        speed,
        brightness: Some(brightness),
    }
}

#[must_use]
pub fn description(name: &str) -> ControllerDescription {
    let speed = Some(SpeedRange {
        min: u32::from(SPEED_MIN),
        max: u32::from(SPEED_MAX),
        current: u32::from(SPEED_MAX),
    });
    let effect_brightness = BrightnessRange {
        min: 0,
        max: EFFECT_BRIGHTNESS_MAX,
        current: EFFECT_BRIGHTNESS_MAX,
    };
    let custom_brightness = BrightnessRange {
        min: 0,
        max: CUSTOM_BRIGHTNESS_MAX,
        current: CUSTOM_BRIGHTNESS_MAX,
    };
    ControllerDescription {
        name: name.into(),
        vendor: "Red Square".into(),
        description: "Red Square Keyrox Device".into(),
        device_type: DeviceType::Keyboard,
        modes: mode_descriptions(speed, effect_brightness, custom_brightness),
        zone_names: vec!["Keyboard".into()],
        led_names: LED_NAMES.iter().map(ToString::to_string).collect(),
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::EFFECTS)
            .union(ControllerCapabilities::BRIGHTNESS),
    }
}

fn mode_descriptions(
    speed: Option<SpeedRange>,
    effect_brightness: BrightnessRange,
    custom_brightness: BrightnessRange,
) -> Vec<ModeDescription> {
    [
        (
            "Custom",
            KeyroxMode::Custom,
            ModeColorMode::PerLed,
            None,
            custom_brightness,
        ),
        (
            "Wave",
            KeyroxMode::Wave,
            ModeColorMode::None,
            speed,
            effect_brightness,
        ),
        (
            "Const",
            KeyroxMode::Const,
            ModeColorMode::PerLed,
            None,
            effect_brightness,
        ),
        (
            "Breathe",
            KeyroxMode::Breathe,
            ModeColorMode::PerLed,
            speed,
            effect_brightness,
        ),
        (
            "Heartrate",
            KeyroxMode::Heartrate,
            ModeColorMode::PerLed,
            speed,
            effect_brightness,
        ),
        (
            "Point",
            KeyroxMode::Point,
            ModeColorMode::PerLed,
            speed,
            effect_brightness,
        ),
        (
            "Winnower",
            KeyroxMode::Winnower,
            ModeColorMode::None,
            speed,
            effect_brightness,
        ),
        (
            "Stars",
            KeyroxMode::Stars,
            ModeColorMode::PerLed,
            speed,
            effect_brightness,
        ),
        (
            "Spectrum",
            KeyroxMode::Spectrum,
            ModeColorMode::None,
            speed,
            effect_brightness,
        ),
        (
            "Plumflower",
            KeyroxMode::Plumflower,
            ModeColorMode::PerLed,
            speed,
            effect_brightness,
        ),
        (
            "Shoot",
            KeyroxMode::Shoot,
            ModeColorMode::PerLed,
            speed,
            effect_brightness,
        ),
        (
            "Ambilight Rotate",
            KeyroxMode::AmbilightRotate,
            ModeColorMode::None,
            speed,
            effect_brightness,
        ),
        (
            "Ripple",
            KeyroxMode::Ripple,
            ModeColorMode::PerLed,
            speed,
            effect_brightness,
        ),
    ]
    .into_iter()
    .map(|(name, mode, color_mode, speed, brightness)| {
        mode_description(name, mode, color_mode, speed, brightness)
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn endpoint(product_id: u16, interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"keyrox-test"[..]),
            0x1A2C,
            product_id,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn both_matchers_are_exact() {
        assert_eq!(
            match_model(&endpoint(0x1511, 3, 0xFF00, 2)),
            Some("Red Square Keyrox TKL")
        );
        assert_eq!(
            match_model(&endpoint(0x2511, 3, 0xFF00, 2)),
            Some("Red Square Keyrox TKL V2")
        );
        assert_eq!(match_model(&endpoint(0x1511, 2, 0xFF00, 2)), None);
        assert_eq!(match_model(&endpoint(0x1511, 3, 0xFF01, 2)), None);
        assert_eq!(match_model(&endpoint(0x1511, 3, 0xFF00, 1)), None);
    }

    #[test]
    fn custom_packet_maps_logical_colors_to_native_slots() {
        let mut colors = [Rgb8::BLACK; LED_COUNT];
        colors[0] = Rgb8::new(1, 2, 3);
        colors[46] = Rgb8::new(4, 5, 6);
        colors[86] = Rgb8::new(7, 8, 9);
        let transaction = CustomColorTransaction::new(&colors, 0xA5).unwrap();
        let mode = transaction.reports()[0].as_bytes();
        assert_eq!(&mode[5..10], &[1, 0, 4, 0, 0x0C]);
        let report = transaction.reports()[1].as_bytes();
        assert_eq!(&report[5..8], &[0xB0, 1, 7]);
        assert_eq!(&report[9..13], &[1, 2, 3, 0xA5]);
        assert_eq!(&report[289..293], &[4, 5, 6, 0xA5]);
        assert_eq!(&report[433..437], &[7, 8, 9, 0xA5]);
        assert!(report[437..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn hardware_modes_preserve_native_setting_encoding() {
        let wave = HardwareModeTransaction::new(
            KeyroxMode::Wave,
            0x7F,
            4,
            Direction::Down,
            ModeColor::None,
        )
        .unwrap();
        assert_eq!(&wave.reports()[0].as_bytes()[5..10], &[1, 0, 4, 0, 0]);
        assert_eq!(
            &wave.reports()[1].as_bytes()[5..13],
            &[9, 0, 5, 0, 0x7F, 0xFF, 0x34, 0xFF]
        );

        let random = HardwareModeTransaction::new(
            KeyroxMode::Ripple,
            10,
            2,
            Direction::Left,
            ModeColor::Random,
        )
        .unwrap();
        assert_eq!(
            &random.reports()[1].as_bytes()[8..13],
            &[0x0B, 10, 0xFF, 0x82, 0xFF]
        );

        let fixed = HardwareModeTransaction::new(
            KeyroxMode::Const,
            12,
            4,
            Direction::Down,
            ModeColor::Fixed(Rgb8::new(0x12, 0x34, 0x56)),
        )
        .unwrap();
        assert_eq!(
            &fixed.reports()[1].as_bytes()[8..15],
            &[1, 12, 0xFF, 0, 0x12, 0x34, 0x56]
        );
    }

    #[test]
    fn invalid_mode_settings_are_rejected() {
        assert!(
            HardwareModeTransaction::new(
                KeyroxMode::Wave,
                128,
                4,
                Direction::Left,
                ModeColor::None
            )
            .is_err()
        );
        assert!(
            HardwareModeTransaction::new(
                KeyroxMode::Wave,
                127,
                5,
                Direction::Left,
                ModeColor::None
            )
            .is_err()
        );
        assert!(
            HardwareModeTransaction::new(
                KeyroxMode::Const,
                127,
                4,
                Direction::Left,
                ModeColor::Random
            )
            .is_err()
        );
        assert!(
            HardwareModeTransaction::new(
                KeyroxMode::Winnower,
                127,
                4,
                Direction::Left,
                ModeColor::None
            )
            .is_err()
        );
        assert!(
            HardwareModeTransaction::new(
                KeyroxMode::Custom,
                127,
                4,
                Direction::Left,
                ModeColor::None
            )
            .is_err()
        );
        assert!(CustomColorTransaction::new(&[Rgb8::BLACK; LED_COUNT - 1], 255).is_err());
    }

    #[test]
    fn layout_and_metadata_are_preserved() {
        assert_eq!(MATRIX_MAP.iter().flatten().flatten().count(), LED_COUNT);
        assert_eq!(MATRIX_MAP[0][0], Some(0));
        assert_eq!(MATRIX_MAP[5][17], Some(86));
        let device = description("Red Square Keyrox TKL");
        assert_eq!(device.modes.len(), 13);
        assert_eq!(device.led_names.len(), LED_COUNT);
        assert_eq!(device.led_names[46], "Key: \\ (ANSI)");
        assert_eq!(device.modes[0].brightness.unwrap().max, 255);
        assert_eq!(device.modes[1].brightness.unwrap().max, 127);
    }
}
