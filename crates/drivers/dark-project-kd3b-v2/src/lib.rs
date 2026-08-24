#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    ExactWriteError, HidDeviceMatch, HidEndpointInfo, OutputReport, OutputWriter, write_exact,
};

pub const OUTPUT_REPORT_LEN: usize = 256;
pub const LED_COUNT: usize = 87;
pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x195D,
    product_id: 0x2061,
    interface_number: Some(2),
    usage_page: Some(0xFFC2),
    usage: Some(0x0004),
};

const PACKET_MAP: [usize; LED_COUNT] = [
    5, 11, 17, 23, 29, 35, 41, 47, 53, 59, 65, 71, 77, 83, 89, 95, 0, 6, 12, 18, 24, 30, 36, 42,
    48, 54, 60, 66, 72, 78, 84, 90, 96, 1, 7, 13, 19, 25, 31, 37, 43, 49, 55, 61, 67, 73, 79, 85,
    91, 97, 2, 8, 14, 20, 26, 32, 38, 44, 50, 56, 62, 68, 80, 3, 15, 21, 27, 33, 39, 45, 51, 57,
    63, 69, 81, 93, 4, 10, 16, 34, 52, 58, 64, 76, 88, 94, 100,
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
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidColorCount(pub usize);

impl fmt::Display for InvalidColorCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Dark Project KD3B V2 requires exactly {LED_COUNT} colors, got {}",
            self.0
        )
    }
}

impl std::error::Error for InvalidColorCount {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerLedColorTransaction([OutputReport<OUTPUT_REPORT_LEN>; 2]);

impl PerLedColorTransaction {
    /// Serializes the native red/green and blue/auxiliary reports.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidColorCount`] unless all 87 key colors are supplied.
    pub fn new(colors: &[Rgb8]) -> Result<Self, InvalidColorCount> {
        if colors.len() != LED_COUNT {
            return Err(InvalidColorCount(colors.len()));
        }
        let mut red_green = [0; OUTPUT_REPORT_LEN];
        let mut blue_aux = [0; OUTPUT_REPORT_LEN];
        red_green[..5].copy_from_slice(&[0x08, 0x07, 0, 0, 0]);
        blue_aux[..5].copy_from_slice(&[0x08, 0x07, 0, 1, 0]);
        for (index, color) in colors.iter().enumerate() {
            let offset = PACKET_MAP[index];
            red_green[5 + offset] = color.r;
            red_green[107 + offset] = color.g;
            blue_aux[5 + offset] = color.b;
        }
        Ok(Self([
            OutputReport::from_array(red_green),
            OutputReport::from_array(blue_aux),
        ]))
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<OUTPUT_REPORT_LEN>; 2] {
        &self.0
    }

    /// Sends both reports in native order and rejects a short write.
    ///
    /// # Errors
    ///
    /// Returns the first transport or short-write error.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        for report in &self.0 {
            write_exact(writer, report)?;
        }
        Ok(())
    }
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description(device_name: &str) -> ControllerDescription {
    ControllerDescription {
        name: device_name.into(),
        vendor: "Dark Project".into(),
        description: "Dark Project Keyboard Device".into(),
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
    use std::sync::Arc;

    fn endpoint(interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"dark-project-test"[..]),
            0x195D,
            0x2061,
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
        assert!(matches(&endpoint(2, 0xFFC2, 4)));
        assert!(!matches(&endpoint(1, 0xFFC2, 4)));
        assert!(!matches(&endpoint(2, 0xFFC2, 3)));
    }

    #[test]
    fn reports_preserve_headers_mapping_and_channel_split() {
        let mut colors = [Rgb8::BLACK; LED_COUNT];
        colors[0] = Rgb8::new(1, 2, 3);
        colors[16] = Rgb8::new(4, 5, 6);
        colors[86] = Rgb8::new(7, 8, 9);
        let tx = PerLedColorTransaction::new(&colors).unwrap();
        assert_eq!(&tx.reports()[0].as_bytes()[..5], &[8, 7, 0, 0, 0]);
        assert_eq!(&tx.reports()[1].as_bytes()[..5], &[8, 7, 0, 1, 0]);
        assert_eq!(tx.reports()[0].as_bytes()[10], 1);
        assert_eq!(tx.reports()[0].as_bytes()[112], 2);
        assert_eq!(tx.reports()[1].as_bytes()[10], 3);
        assert_eq!(tx.reports()[0].as_bytes()[5], 4);
        assert_eq!(tx.reports()[0].as_bytes()[105], 7);
        assert_eq!(tx.reports()[0].as_bytes()[207], 8);
        assert_eq!(tx.reports()[1].as_bytes()[105], 9);
    }

    #[test]
    fn exact_count_matrix_and_model_are_preserved() {
        assert!(PerLedColorTransaction::new(&[Rgb8::BLACK; LED_COUNT - 1]).is_err());
        assert_eq!(MATRIX_MAP.len(), 6);
        assert_eq!(MATRIX_MAP[0][0], Some(0));
        assert_eq!(MATRIX_MAP[5][17], Some(86));
        let device = description("Dark Project KD3B V2");
        assert_eq!(device.led_names.len(), LED_COUNT);
        assert_eq!(device.led_names[0], "Escape");
        assert_eq!(device.led_names[86], "Right Arrow");
    }
}
