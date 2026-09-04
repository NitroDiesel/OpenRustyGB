#![forbid(unsafe_code)]

use std::fmt;
use std::thread;
use std::time::Duration;

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    ExactWriteError, FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, OutputWriter,
    write_exact,
};

pub const OUTPUT_REPORT_LEN: usize = 64;
pub const CONFIG_REPORT_LEN: usize = 117;
pub const CUSTOM_REPORT_LEN: usize = 361;
pub const LED_COUNT: usize = 104;
const PHYSICAL_LED_COUNT: usize = 120;
const SHORT_DELAY: Duration = Duration::from_millis(5);
const MODE_SETTLE_DELAY: Duration = Duration::from_millis(10);

#[derive(Clone, Copy, Debug)]
pub struct AocKeyboardModel {
    pub name: &'static str,
    pub matcher: HidDeviceMatch,
}

pub const MODELS: [AocKeyboardModel; 2] = [
    AocKeyboardModel {
        name: "AOC GK500",
        matcher: HidDeviceMatch {
            vendor_id: 0x3938,
            product_id: 0x1178,
            interface_number: None,
            usage_page: Some(0xFF19),
            usage: Some(0xFF19),
        },
    },
    AocKeyboardModel {
        name: "AOC GK500",
        matcher: HidDeviceMatch {
            vendor_id: 0x3938,
            product_id: 0x1229,
            interface_number: None,
            usage_page: Some(0xFF19),
            usage: Some(0xFF19),
        },
    },
];

pub const MATRIX_MAP: [[Option<u8>; 21]; 6] = [
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
        Some(9),
        Some(10),
        Some(11),
        Some(12),
        Some(13),
        Some(14),
        Some(15),
        None,
        None,
        None,
        None,
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
        Some(30),
        Some(31),
        Some(32),
        Some(33),
        Some(34),
        Some(35),
        Some(36),
    ],
    [
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
        Some(50),
        Some(51),
        Some(52),
        Some(53),
        Some(54),
        Some(55),
        Some(56),
        Some(57),
    ],
    [
        Some(58),
        Some(59),
        Some(60),
        Some(61),
        Some(62),
        Some(63),
        Some(64),
        Some(65),
        Some(66),
        Some(67),
        Some(68),
        Some(69),
        None,
        Some(70),
        None,
        None,
        None,
        Some(71),
        Some(72),
        Some(73),
        None,
    ],
    [
        Some(74),
        None,
        Some(75),
        Some(76),
        Some(77),
        Some(78),
        Some(79),
        Some(80),
        Some(81),
        Some(82),
        Some(83),
        Some(84),
        Some(85),
        None,
        None,
        Some(86),
        None,
        Some(87),
        Some(88),
        Some(89),
        Some(90),
    ],
    [
        Some(91),
        Some(92),
        Some(93),
        None,
        None,
        None,
        Some(94),
        None,
        None,
        None,
        Some(95),
        Some(96),
        Some(97),
        Some(98),
        Some(99),
        Some(100),
        Some(101),
        Some(102),
        None,
        Some(103),
        None,
    ],
];

const PHYSICAL_LED_IDS: [u8; LED_COUNT] = [
    90, 92, 77, 63, 79, 94, 81, 96, 82, 83, 98, 40, 55, 85, 100, 104, 75, 76, 91, 62, 48, 64, 50,
    65, 66, 67, 97, 68, 84, 70, 59, 74, 89, 58, 73, 88, 103, 60, 61, 47, 78, 33, 49, 35, 80, 51,
    52, 53, 69, 99, 25, 44, 29, 14, 43, 28, 13, 102, 45, 46, 32, 93, 18, 34, 20, 95, 36, 37, 38,
    54, 10, 57, 72, 87, 30, 31, 17, 2, 3, 19, 5, 6, 21, 22, 23, 39, 11, 42, 27, 12, 101, 15, 0, 1,
    4, 7, 8, 24, 9, 26, 41, 56, 71, 86,
];

const LED_NAMES: [&str; LED_COUNT] = [
    "Escape",
    "F1",
    "F2",
    "F3",
    "F4",
    "F5",
    "F6",
    "F7",
    "F8",
    "F9",
    "F10",
    "F11",
    "F12",
    "Print Screen",
    "Scroll Lock",
    "Pause Break",
    "Back Tick",
    "1",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "9",
    "0",
    "Minus",
    "Equals",
    "Backspace",
    "Insert",
    "Home",
    "Page Up",
    "Numpad Lock",
    "Numpad Divide",
    "Numpad Times",
    "Numpad Minus",
    "Tab",
    "Q",
    "W",
    "E",
    "R",
    "T",
    "Y",
    "U",
    "I",
    "O",
    "P",
    "Left Bracket",
    "Right Bracket",
    "ANSI Backslash",
    "Delete",
    "End",
    "Page Down",
    "Numpad 7",
    "Numpad 8",
    "Numpad 9",
    "Numpad Plus",
    "Caps Lock",
    "A",
    "S",
    "D",
    "F",
    "G",
    "H",
    "J",
    "K",
    "L",
    "Semicolon",
    "Quote",
    "ISO Enter",
    "Numpad 4",
    "Numpad 5",
    "Numpad 6",
    "Left Shift",
    "Z",
    "X",
    "C",
    "V",
    "B",
    "N",
    "M",
    "Comma",
    "Period",
    "Forward Slash",
    "Right Shift",
    "Up Arrow",
    "Numpad 1",
    "Numpad 2",
    "Numpad 3",
    "Numpad Enter",
    "Left Control",
    "Left Windows",
    "Left Alt",
    "Space",
    "Right Alt",
    "Right Function",
    "Menu",
    "Right Control",
    "Left Arrow",
    "Down Arrow",
    "Right Arrow",
    "Numpad 0",
    "Numpad Period",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AocKeyboardMode {
    Static,
    SpectrumCycle,
    Breathing,
    React,
    Ripple,
    Radar,
    Fireworks,
    Flashing,
    Wave,
    RainbowWave,
    ConcentricCircles,
    WWave,
    Direct,
}

impl AocKeyboardMode {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Static => "Static",
            Self::SpectrumCycle => "Spectrum Cycle",
            Self::Breathing => "Breathing",
            Self::React => "React",
            Self::Ripple => "Ripple",
            Self::Radar => "Radar",
            Self::Fireworks => "Fireworks",
            Self::Flashing => "Flashing",
            Self::Wave => "Wave",
            Self::RainbowWave => "Rainbow Wave",
            Self::ConcentricCircles => "Concentric Circles",
            Self::WWave => "W Wave",
            Self::Direct => "Direct",
        }
    }

    const fn protocol(self) -> u8 {
        match self {
            Self::Static | Self::SpectrumCycle => 0x00,
            Self::Breathing => 0x01,
            Self::React => 0x02,
            Self::Ripple => 0x04,
            Self::Radar => 0x05,
            Self::Fireworks => 0x06,
            Self::Flashing => 0x07,
            Self::Wave | Self::RainbowWave => 0x08,
            Self::Direct => 0x09,
            Self::ConcentricCircles => 0x0A,
            Self::WWave => 0x0B,
        }
    }

    const fn uses_speed(self) -> bool {
        !matches!(self, Self::Static | Self::Direct)
    }

    const fn uses_direction(self) -> bool {
        matches!(
            self,
            Self::Radar | Self::Wave | Self::RainbowWave | Self::ConcentricCircles
        )
    }

    const fn supports_random(self) -> bool {
        !matches!(self, Self::Static | Self::Wave | Self::Direct)
    }

    const fn requires_random(self) -> bool {
        matches!(self, Self::SpectrumCycle | Self::RainbowWave)
    }

    const fn default_color_mode(self) -> ModeColorMode {
        match self {
            Self::SpectrumCycle | Self::RainbowWave => ModeColorMode::Random,
            Self::Direct => ModeColorMode::PerLed,
            _ => ModeColorMode::ModeSpecific,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    const fn protocol(self) -> u8 {
        match self {
            Self::Left => 1,
            Self::Right => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidSettings {
    DirectMode,
    Brightness(u8),
    Speed { mode: AocKeyboardMode, speed: u8 },
    Direction { mode: AocKeyboardMode },
    RandomNotSupported { mode: AocKeyboardMode },
    RandomRequired { mode: AocKeyboardMode },
    ColorCount(usize),
}

impl fmt::Display for InvalidSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DirectMode => write!(f, "Direct uses the per-key transaction"),
            Self::Brightness(value) => write!(f, "brightness must be 0..=3, got {value}"),
            Self::Speed { mode, speed } => write!(
                f,
                "{} speed must be {}",
                mode.name(),
                if mode.uses_speed() { "1..=3" } else { "0" }
            )
            .and_then(|()| write!(f, ", got {speed}")),
            Self::Direction { mode } => write!(f, "{} does not expose direction", mode.name()),
            Self::RandomNotSupported { mode } => {
                write!(f, "{} does not expose random color", mode.name())
            }
            Self::RandomRequired { mode } => {
                write!(f, "{} requires random color", mode.name())
            }
            Self::ColorCount(actual) => {
                write!(
                    f,
                    "AOC GK500 requires exactly {LED_COUNT} colors, got {actual}"
                )
            }
        }
    }
}

impl std::error::Error for InvalidSettings {}

#[derive(Debug, Eq, PartialEq)]
pub enum ApplyError<E> {
    Output(ExactWriteError<E>),
    Feature(E),
}

impl<E: fmt::Display> fmt::Display for ApplyError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Output(error) => write!(f, "output report failed: {error}"),
            Self::Feature(error) => write!(f, "feature report failed: {error}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for ApplyError<E> {}

const fn lifecycle_report(command: u8) -> OutputReport<OUTPUT_REPORT_LEN> {
    let mut bytes = [0; OUTPUT_REPORT_LEN];
    bytes[0] = 0x09;
    bytes[1] = command;
    OutputReport::from_array(bytes)
}

fn validate_common(
    mode: AocKeyboardMode,
    brightness: u8,
    speed: u8,
    direction: Direction,
    random: bool,
) -> Result<(), InvalidSettings> {
    if mode == AocKeyboardMode::Direct {
        return Err(InvalidSettings::DirectMode);
    }
    if brightness > 3 {
        return Err(InvalidSettings::Brightness(brightness));
    }
    if (mode.uses_speed() && !(1..=3).contains(&speed)) || (!mode.uses_speed() && speed != 0) {
        return Err(InvalidSettings::Speed { mode, speed });
    }
    if !mode.uses_direction() && direction != Direction::Left {
        return Err(InvalidSettings::Direction { mode });
    }
    if random && !mode.supports_random() {
        return Err(InvalidSettings::RandomNotSupported { mode });
    }
    if mode.requires_random() && !random {
        return Err(InvalidSettings::RandomRequired { mode });
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction {
    start: OutputReport<OUTPUT_REPORT_LEN>,
    config: OutputReport<CONFIG_REPORT_LEN>,
    end: OutputReport<OUTPUT_REPORT_LEN>,
}

impl ModeTransaction {
    /// Builds one native start/config/end hardware-effect transaction.
    ///
    /// # Errors
    /// Rejects settings outside the mode surface exposed by upstream `OpenRGB`.
    pub fn new(
        mode: AocKeyboardMode,
        brightness: u8,
        speed: u8,
        direction: Direction,
        random: bool,
        color: Rgb8,
    ) -> Result<Self, InvalidSettings> {
        validate_common(mode, brightness, speed, direction, random)?;
        let mut bytes = [0; CONFIG_REPORT_LEN];
        bytes[0] = 0x14;
        bytes[1] = 0x01;
        bytes[6] = mode.protocol();
        let offset = 0x07 + 9 * usize::from(mode.protocol());
        bytes[offset] = color.r;
        bytes[offset + 1] = color.g;
        bytes[offset + 2] = color.b;
        bytes[offset + 3] = u8::from(random);
        bytes[offset + 4] = direction.protocol();
        bytes[offset + 5] = speed;
        bytes[offset + 6] = brightness;
        let checksum = bytes[..115]
            .iter()
            .fold(0x4A9Eu16, |sum, byte| sum.wrapping_add(u16::from(*byte)));
        bytes[115..117].copy_from_slice(&checksum.to_le_bytes());
        Ok(Self {
            start: lifecycle_report(0x21),
            config: OutputReport::from_array(bytes),
            end: lifecycle_report(0x22),
        })
    }

    #[must_use]
    pub const fn config_report(&self) -> &OutputReport<CONFIG_REPORT_LEN> {
        &self.config
    }

    /// Sends the native transaction with its 5/5/10 ms pacing.
    ///
    /// # Errors
    /// Returns the first output short-write, output transport, or feature transport error.
    pub fn apply<W>(&self, writer: &mut W) -> Result<(), ApplyError<<W as OutputWriter<64>>::Error>>
    where
        W: OutputWriter<OUTPUT_REPORT_LEN>
            + FeatureWriter<CONFIG_REPORT_LEN, Error = <W as OutputWriter<64>>::Error>,
    {
        write_exact(writer, &self.start).map_err(ApplyError::Output)?;
        thread::sleep(SHORT_DELAY);
        <W as FeatureWriter<CONFIG_REPORT_LEN>>::send_feature_report(writer, &self.config)
            .map_err(ApplyError::Feature)?;
        thread::sleep(SHORT_DELAY);
        write_exact(writer, &self.end).map_err(ApplyError::Output)?;
        thread::sleep(MODE_SETTLE_DELAY);
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectTransaction {
    start: OutputReport<OUTPUT_REPORT_LEN>,
    custom: OutputReport<CUSTOM_REPORT_LEN>,
    end: OutputReport<OUTPUT_REPORT_LEN>,
}

impl DirectTransaction {
    /// Builds the native per-key custom frame. Brightness is validated because
    /// upstream exposes it for Direct, although the custom packet itself has no brightness byte.
    ///
    /// # Errors
    /// Rejects brightness outside 0..=3 or a logical color count other than 104.
    pub fn new(brightness: u8, colors: &[Rgb8]) -> Result<Self, InvalidSettings> {
        if brightness > 3 {
            return Err(InvalidSettings::Brightness(brightness));
        }
        if colors.len() != LED_COUNT {
            return Err(InvalidSettings::ColorCount(colors.len()));
        }
        let mut bytes = [0; CUSTOM_REPORT_LEN];
        bytes[0] = 0x20;
        for (color, physical_id) in colors.iter().zip(PHYSICAL_LED_IDS) {
            let slot = usize::from(physical_id);
            bytes[1 + slot] = color.r;
            bytes[1 + PHYSICAL_LED_COUNT + slot] = color.g;
            bytes[1 + 2 * PHYSICAL_LED_COUNT + slot] = color.b;
        }
        Ok(Self {
            start: lifecycle_report(0x21),
            custom: OutputReport::from_array(bytes),
            end: lifecycle_report(0x22),
        })
    }

    #[must_use]
    pub const fn custom_report(&self) -> &OutputReport<CUSTOM_REPORT_LEN> {
        &self.custom
    }

    /// Sends the native transaction with its 5/5/5 ms pacing.
    ///
    /// # Errors
    /// Returns the first output short-write, output transport, or feature transport error.
    pub fn apply<W>(&self, writer: &mut W) -> Result<(), ApplyError<<W as OutputWriter<64>>::Error>>
    where
        W: OutputWriter<OUTPUT_REPORT_LEN>
            + FeatureWriter<CUSTOM_REPORT_LEN, Error = <W as OutputWriter<64>>::Error>,
    {
        write_exact(writer, &self.start).map_err(ApplyError::Output)?;
        thread::sleep(SHORT_DELAY);
        <W as FeatureWriter<CUSTOM_REPORT_LEN>>::send_feature_report(writer, &self.custom)
            .map_err(ApplyError::Feature)?;
        thread::sleep(SHORT_DELAY);
        write_exact(writer, &self.end).map_err(ApplyError::Output)?;
        thread::sleep(SHORT_DELAY);
        Ok(())
    }
}

#[must_use]
pub fn match_model(endpoint: &HidEndpointInfo) -> Option<AocKeyboardModel> {
    MODELS
        .into_iter()
        .find(|model| model.matcher.matches(endpoint))
}

fn mode_description(mode: AocKeyboardMode) -> ModeDescription {
    let brightness = Some(BrightnessRange {
        min: 0,
        max: 3,
        current: 3,
    });
    let speed = mode.uses_speed().then_some(SpeedRange {
        min: 3,
        max: 1,
        current: 2,
    });
    ModeDescription {
        name: mode.name().into(),
        value: u32::from(mode.protocol()),
        color_mode: mode.default_color_mode(),
        speed,
        brightness,
    }
}

#[must_use]
pub fn description() -> ControllerDescription {
    let modes = [
        AocKeyboardMode::Static,
        AocKeyboardMode::SpectrumCycle,
        AocKeyboardMode::Breathing,
        AocKeyboardMode::React,
        AocKeyboardMode::Ripple,
        AocKeyboardMode::Radar,
        AocKeyboardMode::Fireworks,
        AocKeyboardMode::Flashing,
        AocKeyboardMode::Wave,
        AocKeyboardMode::RainbowWave,
        AocKeyboardMode::ConcentricCircles,
        AocKeyboardMode::WWave,
        AocKeyboardMode::Direct,
    ];
    ControllerDescription {
        name: "AOC GK500".into(),
        vendor: "AOC".into(),
        description: "AOC Keyboard Device".into(),
        device_type: DeviceType::Keyboard,
        modes: modes.into_iter().map(mode_description).collect(),
        zone_names: vec!["Keyboard".into()],
        led_names: LED_NAMES.iter().map(|name| (*name).into()).collect(),
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::EFFECTS)
            .union(ControllerCapabilities::BRIGHTNESS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::sync::Arc;

    #[derive(Debug, Default)]
    struct RecordingIo {
        output: Vec<[u8; OUTPUT_REPORT_LEN]>,
        configs: Vec<[u8; CONFIG_REPORT_LEN]>,
        customs: Vec<[u8; CUSTOM_REPORT_LEN]>,
        short_write: bool,
    }

    impl OutputWriter<OUTPUT_REPORT_LEN> for RecordingIo {
        type Error = io::Error;

        fn write_output(
            &mut self,
            report: &OutputReport<OUTPUT_REPORT_LEN>,
        ) -> Result<usize, Self::Error> {
            self.output.push(*report.as_bytes());
            Ok(if self.short_write {
                OUTPUT_REPORT_LEN - 1
            } else {
                OUTPUT_REPORT_LEN
            })
        }
    }

    impl FeatureWriter<CONFIG_REPORT_LEN> for RecordingIo {
        type Error = io::Error;

        fn send_feature_report(
            &mut self,
            report: &OutputReport<CONFIG_REPORT_LEN>,
        ) -> Result<(), Self::Error> {
            self.configs.push(*report.as_bytes());
            Ok(())
        }
    }

    impl FeatureWriter<CUSTOM_REPORT_LEN> for RecordingIo {
        type Error = io::Error;

        fn send_feature_report(
            &mut self,
            report: &OutputReport<CUSTOM_REPORT_LEN>,
        ) -> Result<(), Self::Error> {
            self.customs.push(*report.as_bytes());
            Ok(())
        }
    }

    fn endpoint(product_id: u16, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"aoc-keyboard-test"[..]),
            0x3938,
            product_id,
            7,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn matcher_accepts_both_native_products_and_requires_usage() {
        assert!(match_model(&endpoint(0x1178, 0xFF19, 0xFF19)).is_some());
        assert!(match_model(&endpoint(0x1229, 0xFF19, 0xFF19)).is_some());
        assert!(match_model(&endpoint(0x1178, 0xFF18, 0xFF19)).is_none());
        assert!(match_model(&endpoint(0x1178, 0xFF19, 1)).is_none());
        assert!(match_model(&endpoint(0x1179, 0xFF19, 0xFF19)).is_none());
    }

    #[test]
    fn mode_packet_preserves_slot_checksum_random_speed_and_direction() {
        let tx = ModeTransaction::new(
            AocKeyboardMode::Radar,
            3,
            2,
            Direction::Left,
            true,
            Rgb8::new(1, 2, 3),
        )
        .unwrap();
        let report = tx.config_report().as_bytes();
        assert_eq!(&report[..7], &[0x14, 1, 0, 0, 0, 0, 5]);
        let offset = 0x07 + 9 * 5;
        assert_eq!(&report[offset..offset + 7], &[1, 2, 3, 1, 1, 2, 3]);
        let checksum = report[..115]
            .iter()
            .fold(0x4A9Eu16, |sum, byte| sum.wrapping_add(u16::from(*byte)));
        assert_eq!(&report[115..117], &checksum.to_le_bytes());

        let right = ModeTransaction::new(
            AocKeyboardMode::Wave,
            3,
            1,
            Direction::Right,
            false,
            Rgb8::new(4, 5, 6),
        )
        .unwrap();
        let wave_offset = 0x07 + 9 * 8;
        assert_eq!(right.config_report().as_bytes()[wave_offset + 4], 0);
    }

    #[test]
    fn mode_validation_preserves_native_surface() {
        assert!(
            ModeTransaction::new(
                AocKeyboardMode::Static,
                3,
                0,
                Direction::Left,
                false,
                Rgb8::BLACK,
            )
            .is_ok()
        );
        assert!(
            ModeTransaction::new(
                AocKeyboardMode::Static,
                3,
                1,
                Direction::Left,
                false,
                Rgb8::BLACK,
            )
            .is_err()
        );
        assert!(
            ModeTransaction::new(
                AocKeyboardMode::SpectrumCycle,
                3,
                2,
                Direction::Left,
                false,
                Rgb8::BLACK,
            )
            .is_err()
        );
        assert!(
            ModeTransaction::new(
                AocKeyboardMode::Wave,
                3,
                2,
                Direction::Left,
                true,
                Rgb8::BLACK,
            )
            .is_err()
        );
        assert!(
            ModeTransaction::new(
                AocKeyboardMode::Breathing,
                4,
                2,
                Direction::Left,
                false,
                Rgb8::BLACK,
            )
            .is_err()
        );
    }

    #[test]
    fn direct_packet_maps_logical_keys_into_native_planes_and_ignores_brightness() {
        let mut colors = [Rgb8::BLACK; LED_COUNT];
        colors[0] = Rgb8::new(1, 2, 3);
        colors[92] = Rgb8::new(4, 5, 6);
        colors[103] = Rgb8::new(7, 8, 9);
        let low = DirectTransaction::new(0, &colors).unwrap();
        let high = DirectTransaction::new(3, &colors).unwrap();
        assert_eq!(low.custom_report(), high.custom_report());
        let report = low.custom_report().as_bytes();
        assert_eq!(report[0], 0x20);
        assert_eq!(report[1 + 90], 1);
        assert_eq!(report[1 + 120 + 90], 2);
        assert_eq!(report[1 + 240 + 90], 3);
        assert_eq!(&report[1..4], &[4, 0, 0]);
        assert_eq!(report[1 + 86], 7);
        assert_eq!(report[1 + 120 + 86], 8);
        assert_eq!(report[1 + 240 + 86], 9);
        assert_eq!(report[120], 0);
        assert!(DirectTransaction::new(4, &colors).is_err());
        assert!(DirectTransaction::new(3, &colors[..LED_COUNT - 1]).is_err());
    }

    #[test]
    fn transactions_send_native_start_feature_end_order_and_reject_short_output() {
        let mode = ModeTransaction::new(
            AocKeyboardMode::Breathing,
            3,
            2,
            Direction::Left,
            false,
            Rgb8::new(1, 2, 3),
        )
        .unwrap();
        let mut io = RecordingIo::default();
        mode.apply(&mut io).unwrap();
        assert_eq!(io.output.len(), 2);
        assert_eq!(&io.output[0][..2], &[0x09, 0x21]);
        assert_eq!(&io.output[1][..2], &[0x09, 0x22]);
        assert_eq!(io.configs.len(), 1);

        let direct = DirectTransaction::new(3, &[Rgb8::BLACK; LED_COUNT]).unwrap();
        direct.apply(&mut io).unwrap();
        assert_eq!(io.customs.len(), 1);

        let mut short = RecordingIo {
            short_write: true,
            ..RecordingIo::default()
        };
        assert!(mode.apply(&mut short).is_err());
        assert_eq!(short.output.len(), 1);
        assert!(short.configs.is_empty());
    }

    #[test]
    fn layout_and_mode_metadata_match_upstream_defaults() {
        assert_eq!(MATRIX_MAP.iter().flatten().flatten().count(), LED_COUNT);
        let mut ids = PHYSICAL_LED_IDS.to_vec();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), LED_COUNT);
        let device = description();
        assert_eq!(device.led_names.len(), LED_COUNT);
        assert_eq!(device.modes.len(), 13);
        assert_eq!(device.modes[0].color_mode, ModeColorMode::ModeSpecific);
        assert_eq!(device.modes[1].color_mode, ModeColorMode::Random);
        assert_eq!(device.modes[12].color_mode, ModeColorMode::PerLed);
        assert_eq!(device.modes[1].speed.unwrap().min, 3);
        assert_eq!(device.modes[1].speed.unwrap().max, 1);
    }
}
