#![forbid(unsafe_code)]

use std::fmt;
use std::thread;
use std::time::Duration;

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription,
    Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const FEATURE_REPORT_LEN: usize = 264;
pub const LED_COUNT: usize = 104;
pub const SPEED_MIN: u8 = 5;
pub const SPEED_MAX: u8 = 16;
const PROFILE: u8 = 1;

pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x264A,
    product_id: 0x3006,
    interface_number: Some(1),
    usage_page: Some(0xFF01),
    usage: None,
};

const KEY_OFFSETS: [usize; LED_COUNT] = [
    0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x15, 0x16, 0x17, 0x18,
    0x19, 0x1A, 0x1B, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, 0x26, 0x27, 0x28, 0x29, 0x2A,
    0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x34, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B,
    0x3C, 0x3E, 0x3F, 0x40, 0x41, 0x42, 0x43, 0x44, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4E,
    0x4F, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E,
    0x5F, 0x60, 0x61, 0x62, 0x63, 0x64, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6C, 0x6D, 0x6F, 0x70, 0x72,
    0x73, 0x75, 0x76, 0x77, 0x78, 0x7C, 0x80, 0x81,
];

pub const MATRIX_MAP: [[Option<u8>; 23]; 6] = [
    [
        Some(0),
        None,
        Some(8),
        Some(15),
        Some(22),
        Some(29),
        None,
        Some(37),
        Some(44),
        Some(51),
        Some(58),
        None,
        Some(65),
        Some(73),
        Some(81),
        Some(88),
        Some(94),
        Some(100),
        Some(102),
        None,
        None,
        None,
        None,
    ],
    [
        Some(1),
        Some(9),
        Some(16),
        Some(23),
        Some(30),
        Some(38),
        Some(45),
        Some(52),
        Some(59),
        Some(66),
        Some(74),
        None,
        Some(82),
        Some(89),
        Some(103),
        None,
        Some(7),
        Some(21),
        Some(36),
        Some(50),
        Some(64),
        Some(80),
        Some(93),
    ],
    [
        Some(2),
        None,
        Some(10),
        Some(17),
        Some(24),
        Some(31),
        None,
        Some(39),
        Some(46),
        Some(53),
        Some(60),
        Some(67),
        Some(75),
        Some(83),
        Some(90),
        Some(95),
        Some(14),
        Some(28),
        Some(43),
        Some(57),
        Some(72),
        Some(87),
        Some(86),
    ],
    [
        Some(3),
        None,
        Some(11),
        Some(18),
        Some(25),
        Some(32),
        None,
        Some(40),
        Some(47),
        Some(54),
        Some(61),
        Some(68),
        Some(76),
        Some(84),
        Some(96),
        None,
        None,
        None,
        None,
        Some(35),
        Some(99),
        Some(63),
        None,
    ],
    [
        Some(4),
        None,
        Some(26),
        Some(33),
        Some(41),
        Some(48),
        None,
        Some(55),
        None,
        Some(62),
        Some(69),
        Some(77),
        Some(85),
        Some(91),
        Some(101),
        None,
        None,
        Some(27),
        None,
        Some(42),
        Some(49),
        Some(71),
        Some(98),
    ],
    [
        Some(5),
        Some(12),
        Some(19),
        None,
        None,
        None,
        None,
        Some(34),
        None,
        None,
        None,
        None,
        Some(70),
        Some(78),
        Some(92),
        Some(97),
        Some(6),
        Some(13),
        Some(20),
        Some(56),
        None,
        Some(79),
        None,
    ],
];

const LED_NAMES: [&str; LED_COUNT] = [
    "Key: Escape",
    "Key: `",
    "Key: Tab",
    "Key: Caps Lock",
    "Key: Left Shift",
    "Key: Left Control",
    "Key: Left Arrow",
    "Key: Insert",
    "Key: F1",
    "Key: 1",
    "Key: Q",
    "Key: A",
    "Key: Left Windows",
    "Key: Down Arrow",
    "Key: Delete",
    "Key: F2",
    "Key: 2",
    "Key: W",
    "Key: S",
    "Key: Left Alt",
    "Key: Right Arrow",
    "Key: Home",
    "Key: F3",
    "Key: 3",
    "Key: E",
    "Key: D",
    "Key: Z",
    "Key: Up Arrow",
    "Key: End",
    "Key: F4",
    "Key: 4",
    "Key: R",
    "Key: F",
    "Key: X",
    "Key: Space",
    "Key: Number Pad 4",
    "Key: Page Up",
    "Key: F5",
    "Key: 5",
    "Key: T",
    "Key: G",
    "Key: C",
    "Key: Number Pad 1",
    "Key: Page Down",
    "Key: F6",
    "Key: 6",
    "Key: Y",
    "Key: H",
    "Key: V",
    "Key: Number Pad 2",
    "Key: Num Lock",
    "Key: F7",
    "Key: 7",
    "Key: U",
    "Key: J",
    "Key: B",
    "Key: Number Pad 0",
    "Key: Number Pad 7",
    "Key: F8",
    "Key: 8",
    "Key: I",
    "Key: K",
    "Key: N",
    "Key: Number Pad 6",
    "Key: Number Pad /",
    "Key: F9",
    "Key: 9",
    "Key: O",
    "Key: L",
    "Key: M",
    "Key: Right Alt",
    "Key: Number Pad 3",
    "Key: Number Pad 8",
    "Key: F10",
    "Key: 0",
    "Key: P",
    "Key: ;",
    "Key: ,",
    "Key: Right Fn",
    "Key: Number Pad .",
    "Key: Number Pad *",
    "Key: F11",
    "Key: -",
    "Key: [",
    "Key: '",
    "Key: .",
    "Key: Number Pad +",
    "Key: Number Pad 9",
    "Key: F12",
    "Key: =",
    "Key: ]",
    "Key: /",
    "Key: Menu",
    "Key: Number Pad -",
    "Key: Print Screen",
    "Key: \\ (ANSI)",
    "Key: Enter",
    "Key: Right Control",
    "Key: Number Pad Enter",
    "Key: Number Pad 5",
    "Key: Scroll Lock",
    "Key: Right Shift",
    "Key: Pause/Break",
    "Key: Backspace",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoseidonMode {
    Static,
    Reactive,
    ArrowFlow,
    Wave,
    Ripple,
}

impl PoseidonMode {
    const fn value(self) -> u8 {
        match self {
            Self::Static => 0,
            Self::Reactive => 1,
            Self::ArrowFlow => 2,
            Self::Wave => 3,
            Self::Ripple => 4,
        }
    }

    const fn uses_speed(self) -> bool {
        matches!(self, Self::Wave)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    const fn value(self) -> u8 {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidSettings {
    ColorCount(usize),
    Speed(u8),
}

impl fmt::Display for InvalidSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ColorCount(actual) => {
                write!(
                    f,
                    "Poseidon Z RGB requires exactly {LED_COUNT} colors, got {actual}"
                )
            }
            Self::Speed(value) => write!(
                f,
                "Poseidon Z RGB wave speed must be between {SPEED_MIN} and {SPEED_MAX}, got {value}"
            ),
        }
    }
}

impl std::error::Error for InvalidSettings {}

fn validate_colors(colors: &[Rgb8]) -> Result<(), InvalidSettings> {
    if colors.len() == LED_COUNT {
        Ok(())
    } else {
        Err(InvalidSettings::ColorCount(colors.len()))
    }
}

fn control_report(
    mode: PoseidonMode,
    direction: Direction,
    speed: u8,
) -> OutputReport<FEATURE_REPORT_LEN> {
    let mut report = [0; FEATURE_REPORT_LEN];
    report[0] = 0x07;
    report[1] = 0x02;
    report[2] = PROFILE;
    report[8] = PROFILE;
    report[10] = direction.value();
    report[12] = mode.value();
    report[13] = 4;
    report[16] = 0x08;
    report[18] = speed;
    report[19] = 0x50;
    OutputReport::from_array(report)
}

fn normalized_settings(
    mode: PoseidonMode,
    direction: Direction,
    speed: u8,
) -> Result<(Direction, u8), InvalidSettings> {
    if mode.uses_speed() {
        if !(SPEED_MIN..=SPEED_MAX).contains(&speed) {
            return Err(InvalidSettings::Speed(speed));
        }
        Ok((direction, speed))
    } else {
        Ok((Direction::Left, 0))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectColorTransaction([OutputReport<FEATURE_REPORT_LEN>; 2]);

impl DirectColorTransaction {
    /// Builds the two native direct-color feature reports.
    ///
    /// # Errors
    /// Returns an error unless all 104 key colors are supplied.
    pub fn new(colors: &[Rgb8]) -> Result<Self, InvalidSettings> {
        validate_colors(colors)?;
        let mut red_green = [0; FEATURE_REPORT_LEN];
        let mut blue = [0; FEATURE_REPORT_LEN];
        red_green[..4].copy_from_slice(&[0x07, 0x0E, PROFILE, 1]);
        blue[..4].copy_from_slice(&[0x07, 0x0E, PROFILE, 2]);
        for (color, offset) in colors.iter().zip(KEY_OFFSETS) {
            red_green[offset] = color.r;
            red_green[offset + 128] = color.g;
            blue[offset] = color.b;
        }
        Ok(Self([
            OutputReport::from_array(red_green),
            OutputReport::from_array(blue),
        ]))
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>; 2] {
        &self.0
    }

    /// Sends red/green, waits 5 ms, then sends blue.
    ///
    /// # Errors
    /// Returns the first feature transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        send_feature(writer, &self.0[0])?;
        thread::sleep(Duration::from_millis(5));
        send_feature(writer, &self.0[1])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction(OutputReport<FEATURE_REPORT_LEN>);

impl ModeTransaction {
    /// Builds an automatically persisted hardware-profile mode command.
    ///
    /// # Errors
    /// Returns an error when Wave uses a speed outside the native range.
    pub fn new(
        mode: PoseidonMode,
        direction: Direction,
        speed: u8,
    ) -> Result<Self, InvalidSettings> {
        let (direction, speed) = normalized_settings(mode, direction, speed)?;
        Ok(Self(control_report(mode, direction, speed)))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<FEATURE_REPORT_LEN> {
        &self.0
    }

    /// Sends the profile command and preserves the native 200 ms settling delay.
    ///
    /// # Errors
    /// Returns the feature transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        send_feature(writer, &self.0)?;
        thread::sleep(Duration::from_millis(200));
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileColorTransaction([OutputReport<FEATURE_REPORT_LEN>; 4]);

impl ProfileColorTransaction {
    /// Builds the native red, green, blue, and final profile-control sequence.
    ///
    /// # Errors
    /// Returns an error for incomplete colors or an invalid Wave speed.
    pub fn new(
        mode: PoseidonMode,
        direction: Direction,
        speed: u8,
        colors: &[Rgb8],
    ) -> Result<Self, InvalidSettings> {
        validate_colors(colors)?;
        let (direction, speed) = normalized_settings(mode, direction, speed)?;
        let mut reports = [[0; FEATURE_REPORT_LEN]; 4];
        for (channel, report) in reports[..3].iter_mut().enumerate() {
            report[..4].copy_from_slice(&[0x07, 0x09, PROFILE, (channel + 1).to_le_bytes()[0]]);
            for (color, offset) in colors.iter().zip(KEY_OFFSETS) {
                report[offset] = match channel {
                    0 => color.r,
                    1 => color.g,
                    _ => color.b,
                };
            }
        }
        reports[3] = *control_report(mode, direction, speed).as_bytes();
        Ok(Self(reports.map(OutputReport::from_array)))
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>; 4] {
        &self.0
    }

    /// Sends each color channel with 10 ms pacing, then reapplies profile control.
    ///
    /// # Errors
    /// Returns the first feature transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        for report in &self.0[..3] {
            send_feature(writer, report)?;
            thread::sleep(Duration::from_millis(10));
        }
        send_feature(writer, &self.0[3])
    }
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description() -> ControllerDescription {
    ControllerDescription {
        name: "Thermaltake Poseidon Z RGB".into(),
        vendor: "Thermaltake".into(),
        description: "Thermaltake Poseidon Z RGB Device".into(),
        device_type: DeviceType::Keyboard,
        modes: vec![
            ModeDescription {
                name: "Direct".into(),
                value: 0,
                color_mode: ModeColorMode::PerLed,
                speed: None,
                brightness: None,
            },
            ModeDescription {
                name: "Static".into(),
                value: 0,
                color_mode: ModeColorMode::PerLed,
                speed: None,
                brightness: None,
            },
            ModeDescription {
                name: "Wave".into(),
                value: 3,
                color_mode: ModeColorMode::None,
                speed: Some(SpeedRange {
                    min: u32::from(SPEED_MIN),
                    max: u32::from(SPEED_MAX),
                    current: u32::from(SPEED_MIN),
                }),
                brightness: None,
            },
            ModeDescription {
                name: "Ripple".into(),
                value: 4,
                color_mode: ModeColorMode::None,
                speed: None,
                brightness: None,
            },
            ModeDescription {
                name: "Reactive".into(),
                value: 1,
                color_mode: ModeColorMode::None,
                speed: None,
                brightness: None,
            },
        ],
        zone_names: vec!["Keyboard".into()],
        led_names: LED_NAMES.iter().map(|name| (*name).into()).collect(),
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::EFFECTS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::sync::Arc;

    #[derive(Debug, Default)]
    struct Writer(Vec<[u8; FEATURE_REPORT_LEN]>);

    impl FeatureWriter<FEATURE_REPORT_LEN> for Writer {
        type Error = io::Error;

        fn send_feature_report(
            &mut self,
            report: &OutputReport<FEATURE_REPORT_LEN>,
        ) -> Result<(), Self::Error> {
            self.0.push(*report.as_bytes());
            Ok(())
        }
    }

    fn endpoint(interface: i32, usage_page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"poseidon-test"[..]),
            0x264A,
            0x3006,
            interface,
            usage_page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn matcher_preserves_interface_and_page_without_usage_constraint() {
        assert!(matches(&endpoint(1, 0xFF01, 0x77)));
        assert!(!matches(&endpoint(0, 0xFF01, 0x77)));
        assert!(!matches(&endpoint(1, 0xFF00, 0x77)));
    }

    #[test]
    fn direct_reports_preserve_split_channels_and_key_offsets() {
        let mut colors = [Rgb8::BLACK; LED_COUNT];
        colors[0] = Rgb8::new(1, 2, 3);
        colors[103] = Rgb8::new(4, 5, 6);
        let transaction = DirectColorTransaction::new(&colors).unwrap();
        assert_eq!(&transaction.reports()[0].as_bytes()[..4], &[7, 14, 1, 1]);
        assert_eq!(&transaction.reports()[1].as_bytes()[..4], &[7, 14, 1, 2]);
        assert_eq!(transaction.reports()[0].as_bytes()[0x08], 1);
        assert_eq!(transaction.reports()[0].as_bytes()[0x88], 2);
        assert_eq!(transaction.reports()[1].as_bytes()[0x08], 3);
        assert_eq!(transaction.reports()[0].as_bytes()[0x81], 4);
        assert_eq!(transaction.reports()[0].as_bytes()[0x101], 5);
        assert_eq!(transaction.reports()[1].as_bytes()[0x81], 6);
    }

    #[test]
    fn profile_reports_preserve_channels_control_and_ranges() {
        let colors = [Rgb8::new(1, 2, 3); LED_COUNT];
        let transaction =
            ProfileColorTransaction::new(PoseidonMode::Wave, Direction::Right, 16, &colors)
                .unwrap();
        assert_eq!(&transaction.reports()[0].as_bytes()[..4], &[7, 9, 1, 1]);
        assert_eq!(&transaction.reports()[1].as_bytes()[..4], &[7, 9, 1, 2]);
        assert_eq!(&transaction.reports()[2].as_bytes()[..4], &[7, 9, 1, 3]);
        let control = transaction.reports()[3].as_bytes();
        assert_eq!(control[10], 1);
        assert_eq!(control[12], 3);
        assert_eq!(control[13], 4);
        assert_eq!(control[18], 16);
        assert!(ModeTransaction::new(PoseidonMode::Wave, Direction::Left, 4).is_err());
        let ripple = ModeTransaction::new(PoseidonMode::Ripple, Direction::Right, 99).unwrap();
        assert_eq!(ripple.report().as_bytes()[10], 0);
        assert_eq!(ripple.report().as_bytes()[18], 0);
        assert!(DirectColorTransaction::new(&colors[..103]).is_err());
    }

    #[test]
    fn apply_preserves_report_order() {
        let transaction = DirectColorTransaction::new(&[Rgb8::BLACK; LED_COUNT]).unwrap();
        let mut writer = Writer::default();
        transaction.apply(&mut writer).unwrap();
        assert_eq!(writer.0.len(), 2);
        assert_eq!(writer.0[0][3], 1);
        assert_eq!(writer.0[1][3], 2);
    }

    #[test]
    fn matrix_and_metadata_are_preserved() {
        assert_eq!(MATRIX_MAP.iter().flatten().flatten().count(), LED_COUNT);
        assert_eq!(MATRIX_MAP[0][0], Some(0));
        assert_eq!(MATRIX_MAP[1][14], Some(103));
        let device = description();
        assert_eq!(device.led_names.len(), LED_COUNT);
        assert_eq!(device.modes.len(), 5);
        assert_eq!(device.led_names[103], "Key: Backspace");
    }
}
