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

pub const OUTPUT_REPORT_LEN: usize = 65;
const FRAME_LEN: usize = 155 * 3;
const REPORT_DELAY: Duration = Duration::from_millis(2);
const DATA_REPORT_COUNT: usize = 8;
const TRANSACTION_REPORT_COUNT: usize = DATA_REPORT_COUNT + 2;

const fn matcher(product_id: u16) -> HidDeviceMatch {
    HidDeviceMatch {
        vendor_id: 0x04D9,
        product_id,
        interface_number: Some(1),
        usage_page: None,
        usage: None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DuckyModel {
    Shine7One2Rgb,
    One2RgbTkl,
}

impl DuckyModel {
    #[must_use]
    pub const fn matcher(self) -> HidDeviceMatch {
        match self {
            Self::Shine7One2Rgb => matcher(0x0348),
            Self::One2RgbTkl => matcher(0x0356),
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Shine7One2Rgb => "Ducky Shine 7/Ducky One 2 RGB",
            Self::One2RgbTkl => "Ducky One 2 RGB TKL",
        }
    }

    #[must_use]
    pub const fn led_count(self) -> usize {
        match self {
            Self::Shine7One2Rgb => 132,
            Self::One2RgbTkl => 108,
        }
    }

    #[must_use]
    pub const fn matrix_width(self) -> usize {
        match self {
            Self::Shine7One2Rgb => 23,
            Self::One2RgbTkl => 19,
        }
    }
}

pub const MODELS: [DuckyModel; 2] = [DuckyModel::Shine7One2Rgb, DuckyModel::One2RgbTkl];

pub const FULL_MATRIX: [[Option<u8>; 23]; 6] = [
    [
        Some(0),
        None,
        Some(12),
        Some(18),
        Some(24),
        Some(30),
        None,
        Some(42),
        Some(48),
        Some(54),
        Some(60),
        None,
        Some(66),
        Some(72),
        Some(78),
        Some(84),
        Some(90),
        Some(96),
        Some(102),
        Some(108),
        Some(114),
        Some(120),
        Some(126),
    ],
    [
        Some(1),
        Some(7),
        Some(13),
        Some(19),
        Some(25),
        Some(31),
        Some(37),
        Some(43),
        Some(49),
        Some(55),
        Some(61),
        None,
        Some(67),
        Some(73),
        Some(85),
        None,
        Some(91),
        Some(97),
        Some(103),
        Some(109),
        Some(115),
        Some(121),
        Some(127),
    ],
    [
        Some(2),
        None,
        Some(8),
        Some(14),
        Some(20),
        Some(26),
        None,
        Some(32),
        Some(38),
        Some(44),
        Some(50),
        Some(56),
        Some(62),
        Some(68),
        Some(74),
        Some(86),
        Some(92),
        Some(98),
        Some(104),
        Some(110),
        Some(116),
        Some(122),
        Some(128),
    ],
    [
        Some(3),
        None,
        Some(9),
        Some(15),
        Some(21),
        Some(27),
        None,
        Some(33),
        Some(39),
        Some(45),
        Some(51),
        Some(57),
        Some(63),
        Some(69),
        Some(75),
        Some(87),
        None,
        None,
        None,
        Some(111),
        Some(117),
        Some(123),
        None,
    ],
    [
        Some(4),
        Some(10),
        Some(16),
        Some(22),
        Some(28),
        Some(34),
        None,
        Some(40),
        None,
        Some(46),
        Some(52),
        Some(58),
        Some(64),
        Some(70),
        Some(82),
        None,
        None,
        Some(100),
        None,
        Some(112),
        Some(118),
        Some(124),
        Some(131),
    ],
    [
        Some(5),
        Some(11),
        Some(17),
        None,
        None,
        None,
        None,
        Some(41),
        None,
        None,
        None,
        None,
        Some(65),
        Some(77),
        Some(83),
        Some(89),
        Some(95),
        Some(101),
        Some(107),
        Some(113),
        None,
        Some(125),
        None,
    ],
];

pub const TKL_MATRIX: [[Option<u8>; 19]; 6] = [
    [
        Some(0),
        None,
        Some(12),
        Some(18),
        Some(24),
        Some(30),
        None,
        Some(42),
        Some(48),
        Some(54),
        Some(60),
        None,
        Some(66),
        Some(72),
        Some(78),
        Some(84),
        Some(90),
        Some(96),
        Some(102),
    ],
    [
        Some(1),
        Some(7),
        Some(13),
        Some(19),
        Some(25),
        Some(31),
        Some(37),
        Some(43),
        Some(49),
        Some(55),
        Some(61),
        None,
        Some(67),
        Some(73),
        Some(85),
        None,
        Some(91),
        Some(97),
        Some(103),
    ],
    [
        Some(2),
        None,
        Some(8),
        Some(14),
        Some(20),
        Some(26),
        None,
        Some(32),
        Some(38),
        Some(44),
        Some(50),
        Some(56),
        Some(62),
        Some(68),
        Some(74),
        Some(86),
        Some(92),
        Some(98),
        Some(104),
    ],
    [
        Some(3),
        None,
        Some(9),
        Some(15),
        Some(21),
        Some(27),
        None,
        Some(33),
        Some(39),
        Some(45),
        Some(51),
        Some(57),
        Some(63),
        Some(69),
        Some(75),
        Some(87),
        None,
        None,
        None,
    ],
    [
        Some(4),
        Some(10),
        Some(16),
        Some(22),
        Some(28),
        Some(34),
        None,
        Some(40),
        None,
        Some(46),
        Some(52),
        Some(58),
        Some(64),
        Some(70),
        Some(82),
        None,
        None,
        Some(100),
        None,
    ],
    [
        Some(5),
        Some(11),
        Some(17),
        None,
        None,
        None,
        None,
        Some(41),
        None,
        None,
        None,
        None,
        Some(65),
        Some(77),
        Some(83),
        Some(89),
        Some(95),
        Some(101),
        Some(107),
    ],
];

const LED_NAMES: [&str; 132] = [
    "Key: Escape",
    "Key: `",
    "Key: Tab",
    "Key: Caps Lock",
    "Key: Left Shift",
    "Key: Left Control",
    "",
    "Key: 1",
    "Key: Q",
    "Key: A",
    "Key: \\ (ISO)",
    "Key: Left Windows",
    "Key: F1",
    "Key: 2",
    "Key: W",
    "Key: S",
    "Key: Z",
    "Key: Left Alt",
    "Key: F2",
    "Key: 3",
    "Key: E",
    "Key: D",
    "Key: X",
    "",
    "Key: F3",
    "Key: 4",
    "Key: R",
    "Key: F",
    "Key: C",
    "",
    "Key: F4",
    "Key: 5",
    "Key: T",
    "Key: G",
    "Key: V",
    "",
    "",
    "Key: 6",
    "Key: Y",
    "Key: H",
    "Key: B",
    "Key: Space",
    "Key: F5",
    "Key: 7",
    "Key: U",
    "Key: J",
    "Key: N",
    "",
    "Key: F6",
    "Key: 8",
    "Key: I",
    "Key: K",
    "Key: M",
    "",
    "Key: F7",
    "Key: 9",
    "Key: O",
    "Key: L",
    "Key: ,",
    "",
    "Key: F8",
    "Key: 0",
    "Key: P",
    "Key: ;",
    "Key: .",
    "Key: Right Alt",
    "Key: F9",
    "Key: -",
    "Key: [",
    "Key: '",
    "Key: /",
    "",
    "Key: F10",
    "Key: =",
    "Key: ]",
    "Key: #",
    "",
    "Key: Right Windows",
    "Key: F11",
    "",
    "",
    "",
    "Key: Right Shift",
    "Key: Right Fn",
    "Key: F12",
    "Key: Backspace",
    "Key: \\ (ANSI)",
    "Key: Enter",
    "",
    "Key: Right Control",
    "Key: Print Screen",
    "Key: Insert",
    "Key: Delete",
    "",
    "",
    "Key: Left Arrow",
    "Key: Scroll Lock",
    "Key: Home",
    "Key: End",
    "",
    "Key: Up Arrow",
    "Key: Down Arrow",
    "Key: Pause/Break",
    "Key: Page Up",
    "Key: Page Down",
    "",
    "",
    "Key: Right Arrow",
    "Key: Calculator",
    "Key: Num Lock",
    "Key: Number Pad 7",
    "Key: Number Pad 4",
    "Key: Number Pad 1",
    "Key: Number Pad 0",
    "Key: Media Mute",
    "Key: Number Pad /",
    "Key: Number Pad 8",
    "Key: Number Pad 5",
    "Key: Number Pad 2",
    "",
    "Key: Media Volume -",
    "Key: Number Pad *",
    "Key: Number Pad 9",
    "Key: Number Pad 6",
    "Key: Number Pad 3",
    "Key: Number Pad .",
    "Key: Media Volume +",
    "Key: Number Pad -",
    "Key: Number Pad +",
    "",
    "",
    "Key: Number Pad Enter",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidColorCount {
    pub model: DuckyModel,
    pub actual: usize,
}

impl fmt::Display for InvalidColorCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} requires exactly {} colors, got {}",
            self.model.name(),
            self.model.led_count(),
            self.actual
        )
    }
}

impl std::error::Error for InvalidColorCount {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Initialization(OutputReport<OUTPUT_REPORT_LEN>);

impl Initialization {
    #[must_use]
    pub const fn new() -> Self {
        let mut report = [0; OUTPUT_REPORT_LEN];
        report[1] = 0x41;
        report[2] = 0x01;
        Self(OutputReport::from_array(report))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<OUTPUT_REPORT_LEN> {
        &self.0
    }

    /// Sends the native direct-mode initialization with its 2 ms pacing.
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
pub struct DirectColorTransaction([OutputReport<OUTPUT_REPORT_LEN>; TRANSACTION_REPORT_COUNT]);

impl DirectColorTransaction {
    /// Builds the native initialize-color, eight data, and terminate report sequence.
    ///
    /// # Errors
    /// Returns an error unless the selected model's complete color surface is supplied.
    pub fn new(model: DuckyModel, colors: &[Rgb8]) -> Result<Self, InvalidColorCount> {
        if colors.len() != model.led_count() {
            return Err(InvalidColorCount {
                model,
                actual: colors.len(),
            });
        }

        let mut frame = [0; FRAME_LEN];
        for (index, color) in colors.iter().enumerate() {
            frame[index * 3..index * 3 + 3].copy_from_slice(&[color.r, color.g, color.b]);
        }

        let mut reports = [[0; OUTPUT_REPORT_LEN]; TRANSACTION_REPORT_COUNT];
        reports[0][1] = 0x56;
        reports[0][2] = 0x81;
        reports[0][5] = 0x01;
        reports[0][9] = 0x08;
        reports[0][13..17].fill(0xAA);

        let mut offset = 0;
        for packet_id in 0..DATA_REPORT_COUNT {
            let report = &mut reports[packet_id + 1];
            report[1] = 0x56;
            report[2] = 0x83;
            report[3] = packet_id.to_le_bytes()[0];
            let (payload_start, payload_len) = if packet_id == 0 {
                report[5] = 0x01;
                report[9] = 0x80;
                report[10] = 0x01;
                report[12] = 0xC1;
                report[17..21].fill(0xFF);
                (25, 40)
            } else {
                (5, 60)
            };
            report[payload_start..payload_start + payload_len]
                .copy_from_slice(&frame[offset..offset + payload_len]);
            offset += payload_len;
        }
        debug_assert_eq!(offset, 460);

        let terminate = &mut reports[TRANSACTION_REPORT_COUNT - 1];
        terminate[1] = 0x51;
        terminate[2] = 0x28;
        terminate[5] = 0xFF;

        Ok(Self(reports.map(OutputReport::from_array)))
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<OUTPUT_REPORT_LEN>; TRANSACTION_REPORT_COUNT] {
        &self.0
    }

    /// Sends the complete ten-report transaction with 2 ms pacing after every write.
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
pub fn match_model(endpoint: &HidEndpointInfo) -> Option<DuckyModel> {
    MODELS
        .iter()
        .copied()
        .find(|model| model.matcher().matches(endpoint))
}

#[must_use]
pub fn description(model: DuckyModel) -> ControllerDescription {
    ControllerDescription {
        name: model.name().into(),
        vendor: "Ducky".into(),
        description: "Ducky Keyboard Device".into(),
        device_type: DeviceType::Keyboard,
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 0xFFFF,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: None,
        }],
        zone_names: vec!["Keyboard".into()],
        led_names: LED_NAMES[..model.led_count()]
            .iter()
            .map(|name| (*name).into())
            .collect(),
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

    fn endpoint(product_id: u16, interface: i32) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"ducky-test"[..]),
            0x04D9,
            product_id,
            interface,
            0xFF00,
            1,
            None,
            None,
            None,
        )
    }

    fn transmitted_frame(transaction: &DirectColorTransaction) -> Vec<u8> {
        let mut result = Vec::with_capacity(460);
        result.extend_from_slice(&transaction.reports()[1].as_bytes()[25..]);
        for report in &transaction.reports()[2..9] {
            result.extend_from_slice(&report.as_bytes()[5..]);
        }
        result
    }

    #[test]
    fn both_models_require_interface_one_without_usage_constraints() {
        for model in MODELS {
            assert_eq!(
                match_model(&endpoint(model.matcher().product_id, 1)),
                Some(model)
            );
            assert!(match_model(&endpoint(model.matcher().product_id, 0)).is_none());
        }
        assert!(match_model(&endpoint(0xFFFF, 1)).is_none());
    }

    #[test]
    fn initialization_and_transaction_headers_are_exact() {
        assert_eq!(
            &Initialization::new().report().as_bytes()[..3],
            &[0, 0x41, 1]
        );
        let transaction =
            DirectColorTransaction::new(DuckyModel::Shine7One2Rgb, &[Rgb8::BLACK; 132]).unwrap();
        assert_eq!(transaction.reports().len(), 10);
        assert_eq!(&transaction.reports()[0].as_bytes()[..3], &[0, 0x56, 0x81]);
        assert_eq!(&transaction.reports()[0].as_bytes()[13..17], &[0xAA; 4]);
        assert_eq!(
            &transaction.reports()[1].as_bytes()[..4],
            &[0, 0x56, 0x83, 0]
        );
        assert_eq!(
            &transaction.reports()[8].as_bytes()[..4],
            &[0, 0x56, 0x83, 7]
        );
        assert_eq!(
            &transaction.reports()[9].as_bytes()[..6],
            &[0, 0x51, 0x28, 0, 0, 0xFF]
        );
    }

    #[test]
    fn logical_rgb_is_linear_and_native_uninitialized_tail_is_zeroed() {
        let mut colors = [Rgb8::BLACK; 132];
        colors[0] = Rgb8::new(1, 2, 3);
        colors[131] = Rgb8::new(4, 5, 6);
        let frame = transmitted_frame(
            &DirectColorTransaction::new(DuckyModel::Shine7One2Rgb, &colors).unwrap(),
        );
        assert_eq!(&frame[..3], &[1, 2, 3]);
        assert_eq!(&frame[393..396], &[4, 5, 6]);
        assert_eq!(&frame[396..], &[0; 64]);

        let tkl = transmitted_frame(
            &DirectColorTransaction::new(DuckyModel::One2RgbTkl, &[Rgb8::BLACK; 108]).unwrap(),
        );
        assert_eq!(&tkl[324..], &[0; 136]);
    }

    #[test]
    fn invalid_counts_and_short_writes_are_rejected() {
        assert!(DirectColorTransaction::new(DuckyModel::One2RgbTkl, &[Rgb8::BLACK; 107]).is_err());
        let transaction =
            DirectColorTransaction::new(DuckyModel::One2RgbTkl, &[Rgb8::BLACK; 108]).unwrap();
        let mut writer = ShortWriter::default();
        assert!(transaction.apply(&mut writer).is_err());
        assert_eq!(writer.calls, 1);
    }

    #[test]
    fn model_shapes_and_metadata_are_preserved() {
        assert_eq!(FULL_MATRIX.len(), 6);
        assert_eq!(FULL_MATRIX[4][22], Some(131));
        assert_eq!(TKL_MATRIX.len(), 6);
        assert_eq!(TKL_MATRIX[5][18], Some(107));
        assert_eq!(description(DuckyModel::Shine7One2Rgb).led_names.len(), 132);
        assert_eq!(description(DuckyModel::One2RgbTkl).led_names.len(), 108);
        assert_eq!(
            description(DuckyModel::Shine7One2Rgb).led_names[131],
            "Key: Number Pad Enter"
        );
    }
}
