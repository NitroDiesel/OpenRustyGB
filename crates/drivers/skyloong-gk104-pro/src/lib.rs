#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    ExactWriteError, HidDeviceMatch, HidEndpointInfo, OutputReport, OutputWriter, write_exact,
};

pub const OUTPUT_REPORT_LEN: usize = 65;
pub const LED_COUNT: usize = 106;
pub const MAX_BRIGHTNESS: u8 = 127;
const PROTOCOL_LED_COUNT: usize = 132;
const LED_BYTES_PER_CHUNK: usize = 56;
const TOTAL_LED_BYTES: usize = PROTOCOL_LED_COUNT * 4;
const CHUNK_COUNT: usize = TOTAL_LED_BYTES.div_ceil(LED_BYTES_PER_CHUNK);
const DIRECT_REPORT_COUNT: usize = CHUNK_COUNT + 1;

const COMMAND_PING: u8 = 0x0C;
const COMMAND_MODE: u8 = 0x0B;
const COMMAND_LE_DEFINE: u8 = 0x1A;
const MODE_OFFLINE: u8 = 0x04;
const MODE_ONLINE: u8 = 0x05;
const LE_DEFINE_SET: u8 = 0x01;
const LE_DEFINE_SAVE: u8 = 0x02;

pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x1EA7,
    product_id: 0x0907,
    interface_number: Some(1),
    usage_page: None,
    usage: None,
};

// The native SwapKey helper inserts the two split-space edits at `key_idx - 1`.
// Preserve its resulting non-column-sorted tail because LED order is observable.
const KEY_VALUES: [usize; LED_COUNT] = [
    0, 2, 3, 4, 5, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
    32, 33, 34, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56,
    58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 79, 84, 85, 86,
    88, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 102, 104, 106, 107, 108, 109, 110, 111, 114, 112,
    118, 116, 120, 121, 122, 124, 125, 126, 127, 128, 130,
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
    "Pause/Break",
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
    "Num Lock",
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
    "Backslash",
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
    "Enter",
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
    "Left Space",
    "Left Alt",
    "Right Space",
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
        Some(94),
        None,
        Some(93),
        None,
        Some(96),
        None,
        Some(95),
        None,
        Some(97),
        Some(98),
        Some(99),
        Some(100),
        Some(101),
        Some(102),
        Some(103),
        None,
        Some(104),
        Some(105),
        None,
    ],
];

fn crc16_ccitt_false(bytes: &[u8]) -> u16 {
    let mut crc = 0xFFFF_u16;
    for byte in bytes {
        crc ^= u16::from(*byte) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}

fn finish_report(mut bytes: [u8; OUTPUT_REPORT_LEN]) -> OutputReport<OUTPUT_REPORT_LEN> {
    bytes[7] = 0;
    bytes[8] = 0;
    let crc = crc16_ccitt_false(&bytes);
    let [crc_low, crc_high] = crc.to_le_bytes();
    bytes[7] = crc_low;
    bytes[8] = crc_high;
    OutputReport::from_array(bytes)
}

fn command(command: u8, subcommand: u8) -> OutputReport<OUTPUT_REPORT_LEN> {
    let mut bytes = [0; OUTPUT_REPORT_LEN];
    bytes[1] = command;
    bytes[2] = subcommand;
    finish_report(bytes)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Initialization([OutputReport<OUTPUT_REPORT_LEN>; 3]);

impl Initialization {
    #[must_use]
    pub fn new() -> Self {
        Self([
            command(COMMAND_PING, 0),
            command(COMMAND_MODE, MODE_ONLINE),
            command(COMMAND_PING, 0),
        ])
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<OUTPUT_REPORT_LEN>; 3] {
        &self.0
    }

    /// Sends the native ping/online/ping initialization sequence.
    ///
    /// # Errors
    /// Returns the first transport or short-write error.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        apply_reports(writer, &self.0)
    }
}

impl Default for Initialization {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Shutdown(OutputReport<OUTPUT_REPORT_LEN>);

impl Shutdown {
    #[must_use]
    pub fn new() -> Self {
        Self(command(COMMAND_MODE, MODE_OFFLINE))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<OUTPUT_REPORT_LEN> {
        &self.0
    }

    /// Returns the keyboard to native offline mode.
    ///
    /// # Errors
    /// Returns a transport or short-write error.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        write_exact(writer, &self.0)
    }
}

impl Default for Shutdown {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidSettings {
    ColorCount(usize),
    Brightness(u8),
}

impl fmt::Display for InvalidSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ColorCount(actual) => write!(
                f,
                "Skyloong GK104 Pro requires exactly {LED_COUNT} colors, got {actual}"
            ),
            Self::Brightness(actual) => write!(
                f,
                "Skyloong GK104 Pro brightness must be between 0 and {MAX_BRIGHTNESS}, got {actual}"
            ),
        }
    }
}

impl std::error::Error for InvalidSettings {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectColorTransaction([OutputReport<OUTPUT_REPORT_LEN>; DIRECT_REPORT_COUNT]);

impl DirectColorTransaction {
    /// Serializes all logical keys into the native 132-slot LE definition and save sequence.
    ///
    /// # Errors
    /// Returns an error unless all key colors and a valid brightness are supplied.
    pub fn new(colors: &[Rgb8], brightness: u8) -> Result<Self, InvalidSettings> {
        if colors.len() != LED_COUNT {
            return Err(InvalidSettings::ColorCount(colors.len()));
        }
        if brightness > MAX_BRIGHTNESS {
            return Err(InvalidSettings::Brightness(brightness));
        }

        let mut led_data = [0; TOTAL_LED_BYTES];
        for (color, slot) in colors.iter().zip(KEY_VALUES) {
            let offset = slot * 4;
            led_data[offset..offset + 4].copy_from_slice(&[color.r, color.g, color.b, brightness]);
        }

        let mut reports = std::array::from_fn(|_| OutputReport::from_array([0; OUTPUT_REPORT_LEN]));
        for (chunk_index, chunk) in led_data.chunks(LED_BYTES_PER_CHUNK).enumerate() {
            let address = chunk_index * LED_BYTES_PER_CHUNK;
            let mut bytes = [0; OUTPUT_REPORT_LEN];
            bytes[1] = COMMAND_LE_DEFINE;
            bytes[2] = LE_DEFINE_SET;
            bytes[3..6].copy_from_slice(&address.to_le_bytes()[..3]);
            bytes[6] = chunk.len().to_le_bytes()[0];
            bytes[9..9 + chunk.len()].copy_from_slice(chunk);
            reports[chunk_index] = finish_report(bytes);
        }
        reports[CHUNK_COUNT] = command(COMMAND_LE_DEFINE, LE_DEFINE_SAVE);
        Ok(Self(reports))
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<OUTPUT_REPORT_LEN>; DIRECT_REPORT_COUNT] {
        &self.0
    }

    /// Sends all chunks followed by the native persistent-save command.
    ///
    /// # Errors
    /// Returns the first transport or short-write error.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        apply_reports(writer, &self.0)
    }
}

fn apply_reports<W: OutputWriter<OUTPUT_REPORT_LEN>>(
    writer: &mut W,
    reports: &[OutputReport<OUTPUT_REPORT_LEN>],
) -> Result<(), ExactWriteError<W::Error>> {
    for report in reports {
        write_exact(writer, report)?;
    }
    Ok(())
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description(device_name: &str) -> ControllerDescription {
    ControllerDescription {
        name: device_name.into(),
        vendor: "Skyloong".into(),
        description: "Skyloong GK104 Pro Keyboard".into(),
        device_type: DeviceType::Keyboard,
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 0xFFFF,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: Some(BrightnessRange {
                min: 0,
                max: MAX_BRIGHTNESS,
                current: MAX_BRIGHTNESS,
            }),
        }],
        zone_names: vec!["Keyboard".into()],
        led_names: LED_NAMES.iter().map(|name| (*name).into()).collect(),
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::BRIGHTNESS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::sync::Arc;

    #[derive(Debug, Default)]
    struct FakeWriter {
        reports: Vec<Vec<u8>>,
        short_at: Option<usize>,
    }

    impl OutputWriter<OUTPUT_REPORT_LEN> for FakeWriter {
        type Error = io::Error;

        fn write_output(
            &mut self,
            report: &OutputReport<OUTPUT_REPORT_LEN>,
        ) -> Result<usize, Self::Error> {
            self.reports.push(report.as_bytes().to_vec());
            if self.short_at == Some(self.reports.len() - 1) {
                Ok(OUTPUT_REPORT_LEN - 1)
            } else {
                Ok(OUTPUT_REPORT_LEN)
            }
        }
    }

    fn endpoint(interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"skyloong-test"[..]),
            0x1EA7,
            0x0907,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    fn assert_crc(report: &OutputReport<OUTPUT_REPORT_LEN>) {
        let mut bytes = *report.as_bytes();
        let expected = u16::from_le_bytes([bytes[7], bytes[8]]);
        bytes[7] = 0;
        bytes[8] = 0;
        assert_eq!(crc16_ccitt_false(&bytes), expected);
    }

    #[test]
    fn matcher_preserves_interface_without_inventing_usage_constraints() {
        assert!(matches(&endpoint(1, 0xFF00, 1)));
        assert!(matches(&endpoint(1, 0x0001, 2)));
        assert!(!matches(&endpoint(0, 0xFF00, 1)));
    }

    #[test]
    fn lifecycle_preserves_ping_online_ping_and_offline_reports() {
        let init = Initialization::new();
        assert_eq!(init.reports()[0].as_bytes()[1..3], [COMMAND_PING, 0]);
        assert_eq!(
            init.reports()[1].as_bytes()[1..3],
            [COMMAND_MODE, MODE_ONLINE]
        );
        assert_eq!(init.reports()[2].as_bytes()[1..3], [COMMAND_PING, 0]);
        assert_eq!(
            Shutdown::new().report().as_bytes()[1..3],
            [COMMAND_MODE, MODE_OFFLINE]
        );
        for report in init.reports() {
            assert_crc(report);
        }
        assert_crc(Shutdown::new().report());
    }

    #[test]
    fn direct_reports_preserve_slot_mapping_chunking_brightness_and_save() {
        let mut colors = [Rgb8::BLACK; LED_COUNT];
        colors[0] = Rgb8::new(1, 2, 3);
        colors[1] = Rgb8::new(4, 5, 6);
        colors[93] = Rgb8::new(10, 11, 12);
        colors[94] = Rgb8::new(13, 14, 15);
        colors[95] = Rgb8::new(16, 17, 18);
        colors[96] = Rgb8::new(19, 20, 21);
        colors[LED_COUNT - 1] = Rgb8::new(7, 8, 9);
        let transaction = DirectColorTransaction::new(&colors, 127).unwrap();
        assert_eq!(transaction.reports().len(), 11);
        let first = transaction.reports()[0].as_bytes();
        assert_eq!(
            &first[1..7],
            &[COMMAND_LE_DEFINE, LE_DEFINE_SET, 0, 0, 0, 56]
        );
        assert_eq!(&first[9..13], &[1, 2, 3, 127]);
        assert_eq!(&first[17..21], &[4, 5, 6, 127]);
        let split_space_chunk = transaction.reports()[8].as_bytes();
        assert_eq!(&split_space_chunk[9..13], &[13, 14, 15, 127]);
        assert_eq!(&split_space_chunk[17..21], &[10, 11, 12, 127]);
        assert_eq!(&split_space_chunk[25..29], &[19, 20, 21, 127]);
        assert_eq!(&split_space_chunk[33..37], &[16, 17, 18, 127]);
        let last_chunk = transaction.reports()[9].as_bytes();
        assert_eq!(&last_chunk[3..7], &[0xF8, 0x01, 0, 24]);
        assert_eq!(&last_chunk[9 + 16..9 + 20], &[7, 8, 9, 127]);
        let save = transaction.reports()[10].as_bytes();
        assert_eq!(&save[1..3], &[COMMAND_LE_DEFINE, LE_DEFINE_SAVE]);
        for report in transaction.reports() {
            assert_crc(report);
        }
    }

    #[test]
    fn invalid_settings_and_short_writes_are_rejected() {
        assert!(matches!(
            DirectColorTransaction::new(&[Rgb8::BLACK; LED_COUNT - 1], 127),
            Err(InvalidSettings::ColorCount(105))
        ));
        assert!(matches!(
            DirectColorTransaction::new(&[Rgb8::BLACK; LED_COUNT], 128),
            Err(InvalidSettings::Brightness(128))
        ));
        let transaction = DirectColorTransaction::new(&[Rgb8::BLACK; LED_COUNT], 127).unwrap();
        let mut writer = FakeWriter {
            short_at: Some(4),
            ..FakeWriter::default()
        };
        assert!(transaction.apply(&mut writer).is_err());
        assert_eq!(writer.reports.len(), 5);
    }

    #[test]
    fn exact_layout_and_metadata_are_preserved() {
        assert_eq!(MATRIX_MAP.len(), 6);
        assert_eq!(MATRIX_MAP[0][0], Some(0));
        assert_eq!(MATRIX_MAP[5][19], Some(105));
        assert_eq!(KEY_VALUES[70], 79);
        assert_eq!(KEY_VALUES[93..97], [114, 112, 118, 116]);
        let device = description("Skyloong GK104 Pro");
        assert_eq!(device.led_names.len(), LED_COUNT);
        assert_eq!(device.led_names[93], "Left Space");
        assert_eq!(device.led_names[95], "Right Space");
        assert_eq!(device.modes[0].value, 0xFFFF);
        assert_eq!(device.modes[0].brightness.unwrap().max, 127);
    }
}
