#![forbid(unsafe_code)]

use std::fmt;
use std::thread;
use std::time::Duration;

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    ExactWriteError, HidDeviceMatch, HidEndpointInfo, OutputReport, OutputWriter, write_exact,
};

pub const OUTPUT_REPORT_LEN: usize = 265;
pub const LED_COUNT: usize = 104;
const PHYSICAL_LED_COUNT: usize = 126;
const BYTES_PER_LED: usize = 4;
const FRAME_LEN: usize = PHYSICAL_LED_COUNT * BYTES_PER_LED;
const PAYLOAD_LEN: usize = 256;
const REPORT_COUNT: usize = FRAME_LEN.div_ceil(PAYLOAD_LEN);
const FIXED_BRIGHTNESS: u8 = 0x64;
const REPORT_DELAY: Duration = Duration::from_millis(2);

pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x125F,
    product_id: 0x9418,
    interface_number: Some(2),
    usage_page: Some(0xFF01),
    usage: Some(0x0001),
};

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
    11, 22, 30, 25, 27, 7, 51, 57, 62, 86, 87, 83, 85, 79, 72, 0, 14, 15, 23, 31, 39, 38, 46, 47,
    55, 63, 71, 70, 54, 81, 102, 118, 110, 92, 100, 108, 109, 9, 8, 16, 24, 32, 33, 41, 40, 48, 56,
    64, 65, 49, 82, 94, 119, 111, 88, 96, 104, 112, 17, 10, 18, 26, 34, 35, 43, 42, 50, 58, 66, 67,
    84, 89, 97, 105, 121, 12, 20, 28, 36, 37, 45, 44, 52, 60, 69, 122, 115, 90, 98, 106, 114, 6,
    124, 75, 91, 77, 125, 61, 4, 117, 93, 101, 99, 107,
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
pub struct InvalidColorCount(pub usize);

impl fmt::Display for InvalidColorCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "XPG Summoner requires exactly {LED_COUNT} colors, got {}",
            self.0
        )
    }
}

impl std::error::Error for InvalidColorCount {}

const fn lifecycle_report() -> OutputReport<OUTPUT_REPORT_LEN> {
    let mut report = [0; OUTPUT_REPORT_LEN];
    report[0] = 0x07;
    report[1] = 0xEA;
    OutputReport::from_array(report)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Initialization(OutputReport<OUTPUT_REPORT_LEN>);

impl Initialization {
    #[must_use]
    pub const fn new() -> Self {
        Self(lifecycle_report())
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<OUTPUT_REPORT_LEN> {
        &self.0
    }

    /// Sends the native initialization report and preserves its 2 ms pacing.
    ///
    /// # Errors
    /// Returns a transport or short-write error.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        write_exact(writer, &self.0)?;
        thread::sleep(REPORT_DELAY);
        Ok(())
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
    pub const fn new() -> Self {
        Self(lifecycle_report())
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<OUTPUT_REPORT_LEN> {
        &self.0
    }

    /// Sends the native terminate-color report and preserves its 2 ms pacing.
    ///
    /// # Errors
    /// Returns a transport or short-write error.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        write_exact(writer, &self.0)?;
        thread::sleep(REPORT_DELAY);
        Ok(())
    }
}

impl Default for Shutdown {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectColorTransaction([OutputReport<OUTPUT_REPORT_LEN>; REPORT_COUNT]);

impl DirectColorTransaction {
    /// Maps 104 logical keys into the keyboard's 126 four-byte protocol slots.
    ///
    /// # Errors
    /// Returns an error unless every logical key color is supplied.
    pub fn new(colors: &[Rgb8]) -> Result<Self, InvalidColorCount> {
        if colors.len() != LED_COUNT {
            return Err(InvalidColorCount(colors.len()));
        }

        let mut frame = [0; FRAME_LEN];
        for (color, physical_id) in colors.iter().zip(PHYSICAL_LED_IDS) {
            let offset = usize::from(physical_id) * BYTES_PER_LED;
            frame[offset..offset + BYTES_PER_LED].copy_from_slice(&[
                FIXED_BRIGHTNESS,
                color.r,
                color.g,
                color.b,
            ]);
        }

        let mut reports = [[0; OUTPUT_REPORT_LEN]; REPORT_COUNT];
        for (packet_id, chunk) in frame.chunks(PAYLOAD_LEN).enumerate() {
            let report = &mut reports[packet_id];
            report[..4].copy_from_slice(&[0x07, 0xA3, 0x08, 0]);
            report[4] = packet_id.to_le_bytes()[0];
            report[6..6 + chunk.len()].copy_from_slice(chunk);
        }
        Ok(Self(reports.map(OutputReport::from_array)))
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<OUTPUT_REPORT_LEN>; REPORT_COUNT] {
        &self.0
    }

    /// Sends both color packets with the native 2 ms pacing after each write.
    ///
    /// # Errors
    /// Returns the first transport or short-write error.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        for report in &self.0 {
            write_exact(writer, report)?;
            thread::sleep(REPORT_DELAY);
        }
        Ok(())
    }
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description() -> ControllerDescription {
    ControllerDescription {
        name: "XPG Summoner Gaming Keyboard".into(),
        vendor: "XPG".into(),
        description: "XPG Summoner Keyboard Device".into(),
        device_type: DeviceType::Keyboard,
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 1,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: None,
        }],
        zone_names: vec!["Keyboard".into()],
        led_names: LED_NAMES.iter().map(|name| (*name).into()).collect(),
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::sync::Arc;

    #[derive(Debug, Default)]
    struct ShortWriter {
        calls: usize,
    }

    impl OutputWriter<OUTPUT_REPORT_LEN> for ShortWriter {
        type Error = io::Error;

        fn write_output(
            &mut self,
            _report: &OutputReport<OUTPUT_REPORT_LEN>,
        ) -> Result<usize, Self::Error> {
            self.calls += 1;
            Ok(OUTPUT_REPORT_LEN - 1)
        }
    }

    fn endpoint(interface: i32, usage_page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"xpg-summoner-test"[..]),
            0x125F,
            0x9418,
            interface,
            usage_page,
            usage,
            None,
            None,
            None,
        )
    }

    fn frame(transaction: &DirectColorTransaction) -> Vec<u8> {
        let mut result = Vec::with_capacity(FRAME_LEN);
        result.extend_from_slice(&transaction.reports()[0].as_bytes()[6..262]);
        result.extend_from_slice(&transaction.reports()[1].as_bytes()[6..254]);
        result
    }

    #[test]
    fn matcher_requires_the_native_interface_page_and_usage() {
        assert!(matches(&endpoint(2, 0xFF01, 1)));
        assert!(!matches(&endpoint(1, 0xFF01, 1)));
        assert!(!matches(&endpoint(2, 0xFF00, 1)));
        assert!(!matches(&endpoint(2, 0xFF01, 2)));
    }

    #[test]
    fn lifecycle_reports_are_exact_and_identical() {
        let initialization = Initialization::new();
        let shutdown = Shutdown::new();
        assert_eq!(initialization.report(), shutdown.report());
        assert_eq!(
            &initialization.report().as_bytes()[..4],
            &[0x07, 0xEA, 0, 0]
        );
        assert_eq!(&initialization.report().as_bytes()[4..], &[0; 261]);
    }

    #[test]
    fn direct_reports_preserve_headers_fragmentation_and_zero_tail() {
        let transaction = DirectColorTransaction::new(&[Rgb8::BLACK; LED_COUNT]).unwrap();
        assert_eq!(transaction.reports().len(), 2);
        assert_eq!(
            &transaction.reports()[0].as_bytes()[..6],
            &[7, 0xA3, 8, 0, 0, 0]
        );
        assert_eq!(
            &transaction.reports()[1].as_bytes()[..6],
            &[7, 0xA3, 8, 0, 1, 0]
        );
        assert_eq!(&transaction.reports()[0].as_bytes()[262..], &[0; 3]);
        assert_eq!(&transaction.reports()[1].as_bytes()[254..], &[0; 11]);
    }

    #[test]
    fn logical_colors_use_native_protocol_ids_and_fixed_brightness() {
        let mut colors = [Rgb8::BLACK; LED_COUNT];
        colors[0] = Rgb8::new(1, 2, 3);
        colors[15] = Rgb8::new(4, 5, 6);
        colors[103] = Rgb8::new(7, 8, 9);
        let data = frame(&DirectColorTransaction::new(&colors).unwrap());
        assert_eq!(&data[11 * 4..11 * 4 + 4], &[100, 1, 2, 3]);
        assert_eq!(&data[..4], &[100, 4, 5, 6]);
        assert_eq!(&data[107 * 4..107 * 4 + 4], &[100, 7, 8, 9]);
        assert_eq!(&data[4..8], &[0, 0, 0, 0]);
    }

    #[test]
    fn invalid_counts_and_short_writes_are_rejected() {
        assert!(DirectColorTransaction::new(&[Rgb8::BLACK; LED_COUNT - 1]).is_err());
        let transaction = DirectColorTransaction::new(&[Rgb8::BLACK; LED_COUNT]).unwrap();
        let mut writer = ShortWriter::default();
        assert!(transaction.apply(&mut writer).is_err());
        assert_eq!(writer.calls, 1);
    }

    #[test]
    fn layout_and_metadata_are_preserved() {
        assert_eq!(MATRIX_MAP.iter().flatten().flatten().count(), LED_COUNT);
        assert_eq!(MATRIX_MAP[0][0], Some(0));
        assert_eq!(MATRIX_MAP[5][19], Some(103));
        let device = description();
        assert_eq!(device.name, "XPG Summoner Gaming Keyboard");
        assert_eq!(device.led_names.len(), LED_COUNT);
        assert_eq!(device.led_names[103], "Numpad Period");
    }
}
