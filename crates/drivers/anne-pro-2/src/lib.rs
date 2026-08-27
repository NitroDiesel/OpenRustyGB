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

pub const OUTPUT_REPORT_LEN: usize = 64;
pub const LED_COUNT: usize = 61;
const PHYSICAL_LED_COUNT: usize = 70;
const FRAME_LEN: usize = PHYSICAL_LED_COUNT * 3;
const PAYLOAD_LEN: usize = 51;
const REPORT_COUNT: usize = FRAME_LEN.div_ceil(PAYLOAD_LEN);
const REPORT_DELAY: Duration = Duration::from_millis(50);

const fn matcher(vendor_id: u16, product_id: u16) -> HidDeviceMatch {
    HidDeviceMatch {
        vendor_id,
        product_id,
        interface_number: Some(1),
        usage_page: None,
        usage: None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AnnePro2Model {
    pub matcher: HidDeviceMatch,
}

pub const MODELS: [AnnePro2Model; 5] = [
    AnnePro2Model {
        matcher: matcher(0x04D9, 0x8008),
    },
    AnnePro2Model {
        matcher: matcher(0x04D9, 0x8009),
    },
    AnnePro2Model {
        matcher: matcher(0x04D9, 0xA292),
    },
    AnnePro2Model {
        matcher: matcher(0x04D9, 0xA293),
    },
    AnnePro2Model {
        matcher: matcher(0x3311, 0xA297),
    },
];

const LED_NAMES: [&str; LED_COUNT] = [
    "Escape",
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
    "Left Control",
    "Left Windows",
    "Left Alt",
    "Space",
    "Right Alt",
    "Right Function",
    "Menu",
    "Right Control",
];

pub const MATRIX_MAP: [[Option<u8>; 14]; 5] = [
    [
        Some(0),
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
    ],
    [
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
        Some(38),
        Some(39),
        Some(40),
        None,
    ],
    [
        Some(41),
        None,
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
        None,
    ],
    [
        Some(53),
        None,
        Some(54),
        Some(55),
        None,
        None,
        Some(56),
        None,
        None,
        Some(57),
        Some(58),
        Some(59),
        Some(60),
        None,
    ],
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidColorCount(pub usize);

impl fmt::Display for InvalidColorCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Anne Pro 2 requires exactly {LED_COUNT} colors, got {}",
            self.0
        )
    }
}

impl std::error::Error for InvalidColorCount {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectColorTransaction([OutputReport<OUTPUT_REPORT_LEN>; REPORT_COUNT]);

impl DirectColorTransaction {
    /// Maps 61 logical keys into the keyboard's 70 physical RGB positions.
    ///
    /// # Errors
    /// Returns an error unless every logical key color is supplied.
    pub fn new(colors: &[Rgb8]) -> Result<Self, InvalidColorCount> {
        if colors.len() != LED_COUNT {
            return Err(InvalidColorCount(colors.len()));
        }

        let mut frame = [0; FRAME_LEN];
        let mut physical_index = 0;
        for (logical_index, color) in colors.iter().enumerate() {
            let offset = physical_index * 3;
            frame[offset..offset + 3].copy_from_slice(&[color.r, color.g, color.b]);
            physical_index += match logical_index {
                40 | 41 | 52 | 53 | 60 => 2,
                55 | 56 => 3,
                _ => 1,
            };
        }
        debug_assert_eq!(physical_index, PHYSICAL_LED_COUNT);

        let mut reports = [[0; OUTPUT_REPORT_LEN]; REPORT_COUNT];
        for (packet_index, chunk) in frame.chunks(PAYLOAD_LEN).enumerate() {
            let report = &mut reports[packet_index];
            report[..4].copy_from_slice(&[0, 123, 16, 65]);
            report[4] = 0x50 | packet_index.to_le_bytes()[0];
            report[5] = (chunk.len() + 4).to_le_bytes()[0];
            report[6..9].copy_from_slice(&[0, 0, 125]);
            report[9..13].copy_from_slice(&[32, 3, 255, 2]);
            report[13..13 + chunk.len()].copy_from_slice(chunk);
        }
        Ok(Self(reports.map(OutputReport::from_array)))
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<OUTPUT_REPORT_LEN>; REPORT_COUNT] {
        &self.0
    }

    /// Sends all five reports with the native 50 ms pacing after each write.
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
pub fn match_model(endpoint: &HidEndpointInfo) -> Option<AnnePro2Model> {
    MODELS
        .iter()
        .copied()
        .find(|model| model.matcher.matches(endpoint))
}

#[must_use]
pub fn description() -> ControllerDescription {
    ControllerDescription {
        name: "Anne Pro 2".into(),
        vendor: "Obinslab".into(),
        description: "Obinslab Anne Pro 2 Device".into(),
        device_type: DeviceType::Keyboard,
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 0,
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

    fn endpoint(vendor: u16, product: u16, interface: i32) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"anne-pro-2-test"[..]),
            vendor,
            product,
            interface,
            0xFF00,
            1,
            None,
            None,
            None,
        )
    }

    fn frame(transaction: &DirectColorTransaction) -> Vec<u8> {
        let mut result = Vec::with_capacity(FRAME_LEN);
        for (index, report) in transaction.reports().iter().enumerate() {
            let count = if index + 1 == REPORT_COUNT {
                FRAME_LEN % PAYLOAD_LEN
            } else {
                PAYLOAD_LEN
            };
            result.extend_from_slice(&report.as_bytes()[13..13 + count]);
        }
        result
    }

    #[test]
    fn all_native_identities_require_interface_one_without_usage_constraints() {
        for model in MODELS {
            assert_eq!(
                match_model(&endpoint(
                    model.matcher.vendor_id,
                    model.matcher.product_id,
                    1
                )),
                Some(model)
            );
            assert!(
                match_model(&endpoint(
                    model.matcher.vendor_id,
                    model.matcher.product_id,
                    0
                ))
                .is_none()
            );
        }
        assert!(match_model(&endpoint(0x04D9, 0xFFFF, 1)).is_none());
    }

    #[test]
    fn reports_preserve_headers_fragmentation_and_zero_tail() {
        let transaction = DirectColorTransaction::new(&[Rgb8::BLACK; LED_COUNT]).unwrap();
        assert_eq!(transaction.reports().len(), 5);
        for (index, report) in transaction.reports().iter().enumerate() {
            assert_eq!(&report.as_bytes()[..4], &[0, 123, 16, 65]);
            assert_eq!(report.as_bytes()[4], 0x50 | index.to_le_bytes()[0]);
            assert_eq!(&report.as_bytes()[6..13], &[0, 0, 125, 32, 3, 255, 2]);
        }
        assert_eq!(transaction.reports()[0].as_bytes()[5], 55);
        assert_eq!(transaction.reports()[4].as_bytes()[5], 10);
        assert_eq!(&transaction.reports()[4].as_bytes()[19..], &[0; 45]);
    }

    #[test]
    fn logical_keys_preserve_native_physical_gaps() {
        let mut colors = [Rgb8::BLACK; LED_COUNT];
        for (index, color) in colors.iter_mut().enumerate() {
            let value = index.to_le_bytes()[0].wrapping_add(1);
            *color = Rgb8::new(value, value.wrapping_add(1), value.wrapping_add(2));
        }
        let transaction = DirectColorTransaction::new(&colors).unwrap();
        let frame = frame(&transaction);
        let mut physical = 0;
        for (logical, color) in colors.iter().enumerate() {
            assert_eq!(
                &frame[physical * 3..physical * 3 + 3],
                &[color.r, color.g, color.b]
            );
            let advance = match logical {
                40 | 41 | 52 | 53 | 60 => 2,
                55 | 56 => 3,
                _ => 1,
            };
            for gap in 1..advance {
                assert_eq!(
                    &frame[(physical + gap) * 3..(physical + gap) * 3 + 3],
                    &[0, 0, 0]
                );
            }
            physical += advance;
        }
        assert_eq!(physical, PHYSICAL_LED_COUNT);
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
    fn matrix_and_metadata_are_preserved() {
        assert_eq!(MATRIX_MAP[0][0], Some(0));
        assert_eq!(MATRIX_MAP[4][12], Some(60));
        assert_eq!(MATRIX_MAP.iter().flatten().flatten().count(), LED_COUNT);
        let device = description();
        assert_eq!(device.name, "Anne Pro 2");
        assert_eq!(device.vendor, "Obinslab");
        assert_eq!(device.led_names.len(), LED_COUNT);
        assert_eq!(device.led_names[60], "Right Control");
    }
}
