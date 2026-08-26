#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    ExactWriteError, FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, OutputWriter,
    send_feature, write_exact,
};

pub const FEATURE_REPORT_LEN: usize = 9;
pub const OUTPUT_REPORT_LEN: usize = 65;
pub const KEY_LED_COUNT: usize = 98;
pub const UNDERGLOW_LED_COUNT: usize = 63;
const KEY_PACKET_COUNT: usize = 6;
const UNDERGLOW_PACKET_COUNT: usize = 3;
const KEY_PROTOCOL_COLOR_COUNT: usize = 128;

pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x3402,
    product_id: 0x0300,
    interface_number: None,
    usage_page: Some(0xFF11),
    usage: Some(0x00F0),
};

const KEY_VALUES: [usize; KEY_LED_COUNT] = [
    78, 79, 98, 100, 77, 0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53,
    54, 55, 56, 57, 58, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 84, 85, 86, 87, 88,
    89, 90, 91, 92, 93, 94, 95, 97, 99, 105, 106, 109, 110, 107, 112, 113, 111, 115, 116, 117, 118,
    119, 120, 121,
];

const KEY_NAMES: [&str; KEY_LED_COUNT] = [
    "Media Previous",
    "Media Play/Pause",
    "Media Next",
    "Media Mute",
    "Media Stop",
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
    "Pound",
    "Enter",
    "Left Shift",
    "ISO Backslash",
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
    "Space",
    "Space",
    "Left Alt",
    "Space",
    "Space",
    "Space",
    "Right Alt",
    "Right Function",
    "Menu",
    "Right Control",
    "Left Arrow",
    "Down Arrow",
    "Right Arrow",
];

pub const MATRIX_MAP: [[Option<u8>; 17]; 7] = [
    [
        Some(4),
        Some(0),
        Some(1),
        Some(2),
        Some(3),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ],
    [
        Some(5),
        None,
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
        Some(16),
        Some(17),
        Some(18),
        Some(19),
        Some(20),
    ],
    [
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
        Some(37),
    ],
    [
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
    ],
    [
        Some(55),
        Some(56),
        Some(57),
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
        None,
        None,
        None,
    ],
    [
        Some(69),
        Some(70),
        Some(71),
        Some(72),
        Some(73),
        Some(74),
        Some(75),
        Some(76),
        Some(77),
        Some(78),
        Some(79),
        Some(80),
        None,
        Some(81),
        None,
        Some(82),
        None,
    ],
    [
        Some(83),
        Some(84),
        Some(87),
        None,
        Some(85),
        Some(86),
        Some(90),
        Some(88),
        Some(89),
        None,
        Some(91),
        Some(92),
        Some(93),
        Some(94),
        Some(95),
        Some(96),
        Some(97),
    ],
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Zone {
    Keyboard,
    Underglow,
}

impl Zone {
    const fn feature_selector(self) -> u8 {
        match self {
            Self::Keyboard => 0xF0,
            Self::Underglow => 0xF1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidColorCounts {
    pub keyboard: usize,
    pub underglow: usize,
}

impl fmt::Display for InvalidColorCounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HYTE Keeb TKL requires exactly {KEY_LED_COUNT} keyboard and {UNDERGLOW_LED_COUNT} underglow colors, got {} and {}",
            self.keyboard, self.underglow
        )
    }
}

impl std::error::Error for InvalidColorCounts {}

#[derive(Debug)]
pub enum ApplyError<E> {
    Feature(E),
    Output(ExactWriteError<E>),
}

impl<E: fmt::Display> fmt::Display for ApplyError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Feature(error) => write!(f, "feature transport failed: {error}"),
            Self::Output(error) => write!(f, "{error}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for ApplyError<E> {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectColorTransaction {
    keyboard_feature: OutputReport<FEATURE_REPORT_LEN>,
    keyboard_packets: [OutputReport<OUTPUT_REPORT_LEN>; KEY_PACKET_COUNT],
    underglow_feature: OutputReport<FEATURE_REPORT_LEN>,
    underglow_packets: [OutputReport<OUTPUT_REPORT_LEN>; UNDERGLOW_PACKET_COUNT],
}

impl DirectColorTransaction {
    /// Builds the two native direct-color zone transfers.
    ///
    /// # Errors
    /// Returns an error unless both logical zones contain their exact native LED counts.
    pub fn new(keyboard: &[Rgb8], underglow: &[Rgb8]) -> Result<Self, InvalidColorCounts> {
        if keyboard.len() != KEY_LED_COUNT || underglow.len() != UNDERGLOW_LED_COUNT {
            return Err(InvalidColorCounts {
                keyboard: keyboard.len(),
                underglow: underglow.len(),
            });
        }

        let mut protocol_keys = [Rgb8::BLACK; KEY_PROTOCOL_COLOR_COUNT];
        for (color, value) in keyboard.iter().zip(KEY_VALUES) {
            protocol_keys[value] = *color;
        }

        Ok(Self {
            keyboard_feature: feature_report(Zone::Keyboard),
            keyboard_packets: key_packets(&protocol_keys),
            underglow_feature: feature_report(Zone::Underglow),
            underglow_packets: underglow_packets(underglow),
        })
    }

    #[must_use]
    pub const fn keyboard_packets(&self) -> &[OutputReport<OUTPUT_REPORT_LEN>; KEY_PACKET_COUNT] {
        &self.keyboard_packets
    }

    #[must_use]
    pub const fn underglow_packets(
        &self,
    ) -> &[OutputReport<OUTPUT_REPORT_LEN>; UNDERGLOW_PACKET_COUNT] {
        &self.underglow_packets
    }

    /// Sends the keyboard selector and pages, then the underglow selector and pages.
    ///
    /// # Errors
    /// Returns the first feature, output, or short-write failure.
    pub fn apply<E, W>(&self, writer: &mut W) -> Result<(), ApplyError<E>>
    where
        E: std::error::Error + Send + Sync + 'static,
        W: FeatureWriter<FEATURE_REPORT_LEN, Error = E>
            + OutputWriter<OUTPUT_REPORT_LEN, Error = E>,
    {
        send_feature(writer, &self.keyboard_feature).map_err(ApplyError::Feature)?;
        for packet in &self.keyboard_packets {
            write_exact(writer, packet).map_err(ApplyError::Output)?;
        }
        send_feature(writer, &self.underglow_feature).map_err(ApplyError::Feature)?;
        for packet in &self.underglow_packets {
            write_exact(writer, packet).map_err(ApplyError::Output)?;
        }
        Ok(())
    }
}

fn feature_report(zone: Zone) -> OutputReport<FEATURE_REPORT_LEN> {
    let mut bytes = [0; FEATURE_REPORT_LEN];
    bytes[1] = 0x04;
    bytes[2] = zone.feature_selector();
    OutputReport::from_array(bytes)
}

fn key_packets(
    colors: &[Rgb8; KEY_PROTOCOL_COLOR_COUNT],
) -> [OutputReport<OUTPUT_REPORT_LEN>; KEY_PACKET_COUNT] {
    let mut bytes = [[0; OUTPUT_REPORT_LEN]; KEY_PACKET_COUNT];
    for (index, color) in colors.iter().enumerate() {
        write_color(&mut bytes, index, *color);
    }
    bytes.map(OutputReport::from_array)
}

fn underglow_packets(colors: &[Rgb8]) -> [OutputReport<OUTPUT_REPORT_LEN>; UNDERGLOW_PACKET_COUNT] {
    let mut bytes = [[0; OUTPUT_REPORT_LEN]; UNDERGLOW_PACKET_COUNT];
    for (index, color) in colors.iter().enumerate() {
        write_color(&mut bytes, index, *color);
    }
    bytes.map(OutputReport::from_array)
}

fn write_color<const P: usize>(
    packets: &mut [[u8; OUTPUT_REPORT_LEN]; P],
    index: usize,
    color: Rgb8,
) {
    let channel = index * 3;
    for (offset, value) in [color.r, color.g, color.b].into_iter().enumerate() {
        let stream_index = channel + offset;
        packets[stream_index / 64][1 + stream_index % 64] = value;
    }
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description(device_name: &str) -> ControllerDescription {
    let mut led_names: Vec<String> = KEY_NAMES.iter().map(|name| (*name).into()).collect();
    led_names.extend((1..=UNDERGLOW_LED_COUNT).map(|index| format!("Underglow LED {index}")));
    ControllerDescription {
        name: device_name.into(),
        vendor: "HYTE".into(),
        description: "HYTE Keyboard Device".into(),
        device_type: DeviceType::Keyboard,
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 0xFFFF,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: None,
        }],
        zone_names: vec!["Keyboard".into(), "Underglow".into()],
        led_names,
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
    struct RecordingWriter {
        features: Vec<Vec<u8>>,
        outputs: Vec<Vec<u8>>,
    }

    impl FeatureWriter<FEATURE_REPORT_LEN> for RecordingWriter {
        type Error = io::Error;

        fn send_feature_report(
            &mut self,
            report: &OutputReport<FEATURE_REPORT_LEN>,
        ) -> Result<(), Self::Error> {
            self.features.push(report.as_bytes().to_vec());
            Ok(())
        }
    }

    impl OutputWriter<OUTPUT_REPORT_LEN> for RecordingWriter {
        type Error = io::Error;

        fn write_output(
            &mut self,
            report: &OutputReport<OUTPUT_REPORT_LEN>,
        ) -> Result<usize, Self::Error> {
            self.outputs.push(report.as_bytes().to_vec());
            Ok(OUTPUT_REPORT_LEN)
        }
    }

    fn endpoint(page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"hyte-test"[..]),
            0x3402,
            0x0300,
            4,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn matcher_preserves_usage_without_requiring_an_interface() {
        assert!(matches(&endpoint(0xFF11, 0xF0)));
        assert!(!matches(&endpoint(0xFF11, 0xF1)));
        assert!(!matches(&endpoint(0xFF10, 0xF0)));
    }

    #[test]
    fn transaction_preserves_mapping_page_boundaries_and_zero_fills_gaps() {
        let mut keys = [Rgb8::BLACK; KEY_LED_COUNT];
        keys[5] = Rgb8::new(1, 2, 3); // Escape, protocol color zero.
        keys[97] = Rgb8::new(4, 5, 6); // Right arrow, protocol color 121.
        let underglow = [Rgb8::new(7, 8, 9); UNDERGLOW_LED_COUNT];
        let transaction = DirectColorTransaction::new(&keys, &underglow).unwrap();
        assert_eq!(
            &transaction.keyboard_packets()[0].as_bytes()[1..4],
            &[1, 2, 3]
        );
        let right_arrow_channel = 121 * 3;
        let packet = right_arrow_channel / 64;
        let offset = 1 + right_arrow_channel % 64;
        assert_eq!(
            &transaction.keyboard_packets()[packet].as_bytes()[offset..offset + 3],
            &[4, 5, 6]
        );
        assert_eq!(transaction.keyboard_packets()[5].as_bytes()[64], 0);
        assert_eq!(
            &transaction.underglow_packets()[0].as_bytes()[1..4],
            &[7, 8, 9]
        );
        assert_eq!(
            &transaction.underglow_packets()[2].as_bytes()[59..62],
            &[7, 8, 9]
        );
        assert_eq!(
            &transaction.underglow_packets()[2].as_bytes()[62..65],
            &[0, 0, 0]
        );
    }

    #[test]
    fn apply_preserves_native_zone_order_and_report_counts() {
        let transaction = DirectColorTransaction::new(
            &[Rgb8::BLACK; KEY_LED_COUNT],
            &[Rgb8::BLACK; UNDERGLOW_LED_COUNT],
        )
        .unwrap();
        let mut writer = RecordingWriter::default();
        transaction.apply(&mut writer).unwrap();
        assert_eq!(writer.features.len(), 2);
        assert_eq!(&writer.features[0][..3], &[0, 4, 0xF0]);
        assert_eq!(&writer.features[1][..3], &[0, 4, 0xF1]);
        assert_eq!(writer.outputs.len(), 9);
    }

    #[test]
    fn model_counts_layout_and_input_validation_are_preserved() {
        assert!(
            DirectColorTransaction::new(
                &[Rgb8::BLACK; KEY_LED_COUNT - 1],
                &[Rgb8::BLACK; UNDERGLOW_LED_COUNT],
            )
            .is_err()
        );
        assert_eq!(MATRIX_MAP.len(), 7);
        assert_eq!(
            description("HYTE Keeb TKL").zone_names,
            ["Keyboard", "Underglow"]
        );
        assert_eq!(description("HYTE Keeb TKL").led_names.len(), 161);
    }
}
