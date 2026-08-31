#![forbid(unsafe_code)]

use std::fmt;
use std::thread;
use std::time::Duration;

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    FeatureReader, FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const FEATURE_REPORT_LEN: usize = 65;
const TRANSACTION_REPORT_COUNT: usize = 10;
const REPORT_DELAY: Duration = Duration::from_millis(1);
const SETTLE_DELAY: Duration = Duration::from_millis(33);

pub const MATCH_PRO: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x05AC,
    product_id: 0x024F,
    interface_number: Some(3),
    usage_page: Some(0xFF13),
    usage: Some(0x0001),
};

pub const MATCH_NORMAL: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x05AC,
    product_id: 0x024F,
    interface_number: Some(2),
    usage_page: Some(0xFF13),
    usage: Some(0x0001),
};

pub const PRO_MATRIX: [[Option<u8>; 22]; 6] = [
    [
        Some(0),
        None,
        Some(1),
        Some(2),
        Some(3),
        Some(4),
        None,
        Some(5),
        Some(6),
        Some(7),
        Some(8),
        None,
        Some(9),
        Some(10),
        Some(11),
        Some(12),
        None,
        Some(13),
        None,
        None,
        None,
        None,
    ],
    [
        Some(14),
        Some(15),
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
        None,
        None,
        None,
        Some(28),
        Some(29),
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
        None,
        None,
        Some(47),
        Some(48),
        Some(49),
        Some(50),
        Some(51),
    ],
    [
        Some(52),
        None,
        None,
        Some(53),
        Some(54),
        Some(55),
        Some(56),
        Some(57),
        Some(58),
        Some(59),
        Some(60),
        Some(61),
        Some(62),
        None,
        Some(63),
        Some(64),
        None,
        Some(65),
        Some(66),
        Some(67),
        Some(68),
        None,
    ],
    [
        Some(69),
        None,
        None,
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
        None,
        Some(81),
        None,
        Some(82),
        Some(83),
        Some(84),
        Some(85),
    ],
    [
        Some(86),
        Some(87),
        Some(88),
        None,
        None,
        None,
        None,
        Some(89),
        None,
        None,
        None,
        None,
        Some(90),
        Some(91),
        Some(92),
        Some(93),
        Some(94),
        Some(95),
        None,
        Some(96),
        Some(97),
        None,
    ],
];

pub const NORMAL_MATRIX: [[Option<u8>; 22]; 6] = [
    [
        Some(0),
        None,
        Some(1),
        Some(2),
        Some(3),
        Some(4),
        None,
        Some(5),
        Some(6),
        Some(7),
        Some(8),
        None,
        Some(9),
        Some(10),
        Some(11),
        Some(12),
        None,
        Some(13),
        Some(14),
        Some(15),
        Some(16),
        Some(17),
    ],
    [
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
        None,
        None,
        None,
        Some(32),
        Some(33),
        Some(34),
        Some(35),
        Some(36),
    ],
    [
        Some(37),
        None,
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
        None,
        None,
        Some(51),
        Some(52),
        Some(53),
        Some(54),
        Some(55),
    ],
    [
        Some(56),
        None,
        None,
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
        None,
        Some(67),
        Some(68),
        None,
        Some(69),
        Some(70),
        Some(71),
        Some(72),
        None,
    ],
    [
        Some(73),
        None,
        None,
        Some(74),
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
        None,
        None,
        Some(85),
        None,
        Some(86),
        Some(87),
        Some(88),
        Some(89),
    ],
    [
        Some(90),
        Some(91),
        Some(92),
        None,
        None,
        None,
        None,
        Some(93),
        None,
        None,
        None,
        None,
        Some(94),
        Some(95),
        Some(96),
        Some(97),
        Some(98),
        Some(99),
        None,
        Some(100),
        Some(101),
        None,
    ],
];

const PRO_KEY_CODES: [u8; 98] = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x77, 0x13, 0x14,
    0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x67, 0x74, 0x20, 0x21, 0x22,
    0x7A, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x43, 0x76,
    0x32, 0x33, 0x34, 0x7B, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0x41, 0x42,
    0x55, 0x79, 0x44, 0x45, 0x46, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50, 0x51, 0x52, 0x53,
    0x54, 0x65, 0x56, 0x57, 0x58, 0x6A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F, 0x60, 0x62, 0x63, 0x64, 0x66,
    0x68, 0x69,
];

const NORMAL_KEY_CODES: [u8; 102] = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x77, 0x70, 0x73,
    0x75, 0x78, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x67,
    0x74, 0x20, 0x21, 0x22, 0x7A, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F,
    0x30, 0x31, 0x43, 0x76, 0x32, 0x33, 0x34, 0x7B, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E,
    0x3F, 0x40, 0x41, 0x42, 0x55, 0x79, 0x44, 0x45, 0x46, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F,
    0x50, 0x51, 0x52, 0x53, 0x54, 0x65, 0x56, 0x57, 0x58, 0x6A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F, 0x60,
    0x62, 0x63, 0x64, 0x66, 0x68, 0x69,
];

const NORMAL_LED_NAMES: [&str; 102] = [
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
    "Key: Delete",
    "Key: Print Screen",
    "Key: Pause/Break",
    "Key: Home",
    "Key: End",
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
    "Key: +",
    "Key: Backspace",
    "Key: Insert",
    "Key: Num Lock",
    "Key: Number Pad /",
    "Key: Number Pad *",
    "Key: Number Pad -",
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
    "Key: \\",
    "Key: Page Up",
    "Key: Number Pad 7",
    "Key: Number Pad 8",
    "Key: Number Pad 9",
    "Key: Number Pad +",
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
    "Key: Page Down",
    "Key: Number Pad 4",
    "Key: Number Pad 5",
    "Key: Number Pad 6",
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
    "Key: Number Pad 1",
    "Key: Number Pad 2",
    "Key: Number Pad 3",
    "Key: Number Pad Enter",
    "Key: Left Control",
    "Key: Left Windows",
    "Key: Left Alt",
    "Key: Space",
    "Key: Right Alt",
    "Key: Right Fn",
    "Key: Right Control",
    "Key: Left Arrow",
    "Key: Down Arrow",
    "Key: Right Arrow",
    "Key: Number Pad 0",
    "Key: Number Pad .",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValkyrieModel {
    Vk99Pro,
    Vk99,
}

impl ValkyrieModel {
    #[must_use]
    pub const fn matcher(self) -> HidDeviceMatch {
        match self {
            Self::Vk99Pro => MATCH_PRO,
            Self::Vk99 => MATCH_NORMAL,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Vk99Pro => "Valkyrie VK99 Pro",
            Self::Vk99 => "Valkyrie VK99",
        }
    }

    #[must_use]
    pub const fn led_count(self) -> usize {
        match self {
            Self::Vk99Pro => 98,
            Self::Vk99 => 102,
        }
    }

    const fn key_codes(self) -> &'static [u8] {
        match self {
            Self::Vk99Pro => &PRO_KEY_CODES,
            Self::Vk99 => &NORMAL_KEY_CODES,
        }
    }
}

pub const MODELS: [ValkyrieModel; 2] = [ValkyrieModel::Vk99Pro, ValkyrieModel::Vk99];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidColorCount {
    pub model: ValkyrieModel,
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
pub struct DirectColorTransaction([OutputReport<FEATURE_REPORT_LEN>; TRANSACTION_REPORT_COUNT]);

impl DirectColorTransaction {
    /// Builds the complete native initialize, color, and terminate sequence.
    ///
    /// # Errors
    /// Returns an error unless every logical key color is supplied.
    pub fn new(model: ValkyrieModel, colors: &[Rgb8]) -> Result<Self, InvalidColorCount> {
        if colors.len() != model.led_count() {
            return Err(InvalidColorCount {
                model,
                actual: colors.len(),
            });
        }

        let mut reports = [[0; FEATURE_REPORT_LEN]; TRANSACTION_REPORT_COUNT];
        reports[0][1] = 0x04;
        reports[0][2] = 0x20;
        reports[0][9] = 0x08;

        let mut frame = vec![0; model.led_count() * 4];
        for ((slot, color), key_code) in
            frame.chunks_exact_mut(4).zip(colors).zip(model.key_codes())
        {
            slot.copy_from_slice(&[*key_code, color.r, color.g, color.b]);
        }
        for (index, chunk) in frame.chunks(64).enumerate() {
            reports[index + 1][1..=chunk.len()].copy_from_slice(chunk);
        }

        reports[9][1] = 0x04;
        reports[9][2] = 0x02;
        Ok(Self(reports.map(OutputReport::from_array)))
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>; TRANSACTION_REPORT_COUNT] {
        &self.0
    }

    /// Sends ten feature reports and performs both native feature reads with
    /// the original pacing.
    ///
    /// # Errors
    /// Returns the first feature transport error.
    pub fn apply<T, E>(&self, transport: &mut T) -> Result<(), E>
    where
        T: FeatureWriter<FEATURE_REPORT_LEN, Error = E>
            + FeatureReader<FEATURE_REPORT_LEN, Error = E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        send_feature(transport, &self.0[0])?;
        thread::sleep(REPORT_DELAY);
        read_feature(transport)?;

        for report in &self.0[1..8] {
            send_feature(transport, report)?;
            thread::sleep(REPORT_DELAY);
        }

        send_feature(transport, &self.0[8])?;
        thread::sleep(REPORT_DELAY);
        send_feature(transport, &self.0[9])?;
        thread::sleep(REPORT_DELAY);
        read_feature(transport)?;
        thread::sleep(SETTLE_DELAY);
        Ok(())
    }
}

fn read_feature<T, E>(transport: &mut T) -> Result<(), E>
where
    T: FeatureReader<FEATURE_REPORT_LEN, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut response = [0; FEATURE_REPORT_LEN];
    transport.get_feature_report(&mut response)?;
    thread::sleep(REPORT_DELAY);
    Ok(())
}

#[must_use]
pub fn match_model(endpoint: &HidEndpointInfo) -> Option<ValkyrieModel> {
    MODELS
        .iter()
        .copied()
        .find(|model| model.matcher().matches(endpoint))
}

fn led_names(model: ValkyrieModel) -> Vec<String> {
    match model {
        ValkyrieModel::Vk99 => NORMAL_LED_NAMES.iter().map(|name| (*name).into()).collect(),
        ValkyrieModel::Vk99Pro => NORMAL_LED_NAMES[..14]
            .iter()
            .chain(&NORMAL_LED_NAMES[18..])
            .map(|name| (*name).into())
            .collect(),
    }
}

#[must_use]
pub fn description(model: ValkyrieModel) -> ControllerDescription {
    ControllerDescription {
        name: model.name().into(),
        vendor: "Valkyrie".into(),
        description: "Valkyrie Keyboard Device".into(),
        device_type: DeviceType::Keyboard,
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 0xFFFF,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: None,
        }],
        zone_names: vec!["Keyboard".into()],
        led_names: led_names(model),
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::sync::Arc;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Event {
        Write([u8; FEATURE_REPORT_LEN]),
        Read(u8),
    }

    #[derive(Debug, Default)]
    struct RecordingTransport(Vec<Event>);

    impl FeatureWriter<FEATURE_REPORT_LEN> for RecordingTransport {
        type Error = io::Error;

        fn send_feature_report(
            &mut self,
            report: &OutputReport<FEATURE_REPORT_LEN>,
        ) -> Result<(), Self::Error> {
            self.0.push(Event::Write(*report.as_bytes()));
            Ok(())
        }
    }

    impl FeatureReader<FEATURE_REPORT_LEN> for RecordingTransport {
        type Error = io::Error;

        fn get_feature_report(
            &mut self,
            report: &mut [u8; FEATURE_REPORT_LEN],
        ) -> Result<usize, Self::Error> {
            self.0.push(Event::Read(report[0]));
            Ok(FEATURE_REPORT_LEN)
        }
    }

    fn endpoint(interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"valkyrie-test"[..]),
            0x05AC,
            0x024F,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn both_interfaces_match_exactly() {
        assert_eq!(
            match_model(&endpoint(3, 0xFF13, 1)),
            Some(ValkyrieModel::Vk99Pro)
        );
        assert_eq!(
            match_model(&endpoint(2, 0xFF13, 1)),
            Some(ValkyrieModel::Vk99)
        );
        assert_eq!(match_model(&endpoint(1, 0xFF13, 1)), None);
        assert_eq!(match_model(&endpoint(3, 0xFF12, 1)), None);
        assert_eq!(match_model(&endpoint(3, 0xFF13, 2)), None);
    }

    #[test]
    fn lifecycle_and_fragmentation_are_exact() {
        let pro = DirectColorTransaction::new(ValkyrieModel::Vk99Pro, &[Rgb8::BLACK; 98]).unwrap();
        assert_eq!(
            &pro.reports()[0].as_bytes()[..10],
            &[0, 4, 0x20, 0, 0, 0, 0, 0, 0, 8]
        );
        assert_eq!(pro.reports()[8].as_bytes(), &[0; FEATURE_REPORT_LEN]);
        assert_eq!(&pro.reports()[9].as_bytes()[..3], &[0, 4, 2]);
        assert!(
            pro.reports()[7].as_bytes()[9..]
                .iter()
                .all(|byte| *byte == 0)
        );

        let normal = DirectColorTransaction::new(ValkyrieModel::Vk99, &[Rgb8::BLACK; 102]).unwrap();
        assert!(
            normal.reports()[7].as_bytes()[25..]
                .iter()
                .all(|byte| *byte == 0)
        );
    }

    #[test]
    fn logical_colors_keep_native_key_codes_and_rgb_order() {
        let mut colors = [Rgb8::BLACK; 102];
        colors[0] = Rgb8::new(1, 2, 3);
        colors[16] = Rgb8::new(4, 5, 6);
        colors[101] = Rgb8::new(7, 8, 9);
        let tx = DirectColorTransaction::new(ValkyrieModel::Vk99, &colors).unwrap();
        assert_eq!(&tx.reports()[1].as_bytes()[1..5], &[1, 1, 2, 3]);
        assert_eq!(&tx.reports()[2].as_bytes()[1..5], &[0x75, 4, 5, 6]);
        assert_eq!(&tx.reports()[7].as_bytes()[21..25], &[0x69, 7, 8, 9]);
    }

    #[test]
    fn apply_preserves_write_read_order() {
        let tx = DirectColorTransaction::new(ValkyrieModel::Vk99Pro, &[Rgb8::BLACK; 98]).unwrap();
        let mut transport = RecordingTransport::default();
        tx.apply(&mut transport).unwrap();
        assert_eq!(transport.0.len(), 12);
        assert!(matches!(transport.0[0], Event::Write(_)));
        assert_eq!(transport.0[1], Event::Read(0));
        assert!(
            transport.0[2..10]
                .iter()
                .all(|event| matches!(event, Event::Write(_)))
        );
        assert_eq!(transport.0[11], Event::Read(0));
    }

    #[test]
    fn invalid_counts_and_layouts_are_checked() {
        assert!(DirectColorTransaction::new(ValkyrieModel::Vk99Pro, &[Rgb8::BLACK; 97]).is_err());
        assert!(DirectColorTransaction::new(ValkyrieModel::Vk99, &[Rgb8::BLACK; 101]).is_err());
        assert_eq!(PRO_MATRIX.iter().flatten().flatten().count(), 98);
        assert_eq!(NORMAL_MATRIX.iter().flatten().flatten().count(), 102);
        assert_eq!(description(ValkyrieModel::Vk99Pro).led_names.len(), 98);
        assert_eq!(description(ValkyrieModel::Vk99).led_names.len(), 102);
        assert_eq!(description(ValkyrieModel::Vk99Pro).led_names[14], "Key: `");
        assert_eq!(
            description(ValkyrieModel::Vk99).led_names[14],
            "Key: Print Screen"
        );
    }
}
