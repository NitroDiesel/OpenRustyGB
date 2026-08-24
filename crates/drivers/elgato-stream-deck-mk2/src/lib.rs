#![forbid(unsafe_code)]

use std::fmt;

use jpeg_encoder::{ColorType, Encoder, EncodingError};
use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    ExactWriteError, FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, OutputWriter,
    send_feature,
};

pub const FEATURE_REPORT_LEN: usize = 32;
pub const OUTPUT_REPORT_LEN: usize = 1024;
pub const BUTTON_COUNT: usize = 15;
pub const MATRIX_MAP: [[u8; 5]; 3] = [[0, 1, 2, 3, 4], [5, 6, 7, 8, 9], [10, 11, 12, 13, 14]];
pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x0FD9,
    product_id: 0x0080,
    interface_number: Some(0),
    usage_page: None,
    usage: None,
};

const IMAGE_WIDTH: u16 = 72;
const IMAGE_HEIGHT: u16 = 72;
const IMAGE_CHANNELS: usize = 3;
const JPEG_QUALITY: u8 = 95;
const OUTPUT_HEADER_LEN: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidColorCount(pub usize);

impl fmt::Display for InvalidColorCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Elgato Stream Deck MK.2 requires exactly {BUTTON_COUNT} colors, got {}",
            self.0
        )
    }
}

impl std::error::Error for InvalidColorCount {}

#[derive(Debug)]
pub enum FrameBuildError {
    InvalidColorCount(InvalidColorCount),
    Jpeg(EncodingError),
    JpegTooLarge(usize),
}

impl fmt::Display for FrameBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidColorCount(error) => error.fmt(f),
            Self::Jpeg(error) => write!(f, "could not encode Stream Deck button JPEG: {error}"),
            Self::JpegTooLarge(length) => write!(
                f,
                "Stream Deck button JPEG is {length} bytes; maximum is {}",
                OUTPUT_REPORT_LEN - OUTPUT_HEADER_LEN
            ),
        }
    }
}

impl std::error::Error for FrameBuildError {}

impl From<InvalidColorCount> for FrameBuildError {
    fn from(value: InvalidColorCount) -> Self {
        Self::InvalidColorCount(value)
    }
}

impl From<EncodingError> for FrameBuildError {
    fn from(value: EncodingError) -> Self {
        Self::Jpeg(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FullFrameTransaction(Vec<OutputReport<OUTPUT_REPORT_LEN>>);

impl FullFrameTransaction {
    /// Encodes a complete 15-button frame into native HID reports.
    ///
    /// # Errors
    ///
    /// Returns an error unless all 15 colors are supplied, JPEG encoding
    /// succeeds, and each encoded image fits in one native report.
    pub fn new(colors: &[Rgb8]) -> Result<Self, FrameBuildError> {
        if colors.len() != BUTTON_COUNT {
            return Err(InvalidColorCount(colors.len()).into());
        }
        let reports = (0_u8..)
            .zip(colors)
            .map(|(index, color)| button_report(index, *color))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self(reports))
    }

    #[must_use]
    pub fn reports(&self) -> &[OutputReport<OUTPUT_REPORT_LEN>] {
        &self.0
    }

    /// Writes all 15 button reports in matrix order and rejects short writes.
    ///
    /// # Errors
    ///
    /// Returns the first transport or short-write error.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        for report in &self.0 {
            let actual = writer
                .write_output(report)
                .map_err(ExactWriteError::Transport)?;
            if actual != OUTPUT_REPORT_LEN && actual != OUTPUT_REPORT_LEN - 1 {
                return Err(ExactWriteError::ShortWrite {
                    expected: OUTPUT_REPORT_LEN - 1,
                    actual,
                });
            }
        }
        Ok(())
    }
}

fn button_report(
    button_index: u8,
    color: Rgb8,
) -> Result<OutputReport<OUTPUT_REPORT_LEN>, FrameBuildError> {
    let pixel_count = usize::from(IMAGE_WIDTH) * usize::from(IMAGE_HEIGHT);
    let mut pixels = Vec::with_capacity(pixel_count * IMAGE_CHANNELS);
    for _ in 0..pixel_count {
        pixels.extend_from_slice(&[color.r, color.g, color.b]);
    }

    let mut jpeg = Vec::new();
    Encoder::new(&mut jpeg, JPEG_QUALITY).encode(
        &pixels,
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        ColorType::Rgb,
    )?;
    if jpeg.len() > OUTPUT_REPORT_LEN - OUTPUT_HEADER_LEN {
        return Err(FrameBuildError::JpegTooLarge(jpeg.len()));
    }

    let mut report = [0; OUTPUT_REPORT_LEN];
    let jpeg_length = u16::try_from(jpeg.len()).expect("JPEG report payload is below 1017 bytes");
    let [jpeg_length_low, jpeg_length_high] = jpeg_length.to_le_bytes();
    report[..OUTPUT_HEADER_LEN].copy_from_slice(&[
        0x02,
        0x07,
        button_index,
        0x01,
        jpeg_length_low,
        jpeg_length_high,
        0,
        0,
    ]);
    report[OUTPUT_HEADER_LEN..OUTPUT_HEADER_LEN + jpeg.len()].copy_from_slice(&jpeg);
    Ok(OutputReport::from_array(report))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrightnessTransaction(OutputReport<FEATURE_REPORT_LEN>);

impl BrightnessTransaction {
    #[must_use]
    pub fn new(brightness: u8) -> Self {
        let mut report = [0; FEATURE_REPORT_LEN];
        report[..3].copy_from_slice(&[0x03, 0x08, brightness]);
        Self(OutputReport::from_array(report))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<FEATURE_REPORT_LEN> {
        &self.0
    }

    /// Sends the native brightness feature report.
    ///
    /// # Errors
    ///
    /// Returns a transport error from the feature-report writer.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        send_feature(writer, &self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResetTransaction(OutputReport<FEATURE_REPORT_LEN>);

impl Default for ResetTransaction {
    fn default() -> Self {
        let mut report = [0; FEATURE_REPORT_LEN];
        report[..2].copy_from_slice(&[0x03, 0x02]);
        Self(OutputReport::from_array(report))
    }
}

impl ResetTransaction {
    #[must_use]
    pub const fn report(&self) -> &OutputReport<FEATURE_REPORT_LEN> {
        &self.0
    }

    /// Sends the native reset feature report.
    ///
    /// # Errors
    ///
    /// Returns a transport error from the feature-report writer.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        send_feature(writer, &self.0)
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
        vendor: "Elgato".into(),
        description: "Stream Deck MK.2 Controller".into(),
        device_type: DeviceType::Accessory,
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 0,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: None,
        }],
        zone_names: vec!["Button Matrix".into()],
        led_names: (1..=BUTTON_COUNT)
            .map(|number| format!("Button {number}"))
            .collect(),
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR),
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::sync::Arc;

    use super::*;

    fn endpoint(
        vendor: u16,
        product: u16,
        interface: i32,
        page: u16,
        usage: u16,
    ) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"stream-deck-test"[..]),
            vendor,
            product,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[derive(Debug, Default)]
    struct RecordingWriter {
        reports: Vec<[u8; OUTPUT_REPORT_LEN]>,
        next_len: Option<usize>,
    }

    impl OutputWriter<OUTPUT_REPORT_LEN> for RecordingWriter {
        type Error = Infallible;

        fn write_output(
            &mut self,
            report: &OutputReport<OUTPUT_REPORT_LEN>,
        ) -> Result<usize, Self::Error> {
            let mut bytes = [0; OUTPUT_REPORT_LEN];
            bytes.copy_from_slice(report.as_bytes());
            self.reports.push(bytes);
            Ok(self.next_len.take().unwrap_or(OUTPUT_REPORT_LEN))
        }
    }

    #[test]
    fn matcher_preserves_product_and_interface_only_detection() {
        assert!(matches(&endpoint(0x0FD9, 0x0080, 0, 0x0001, 2)));
        assert!(matches(&endpoint(0x0FD9, 0x0080, 0, 0xFF00, 9)));
        assert!(!matches(&endpoint(0x0FD9, 0x0080, 1, 0x0001, 2)));
        assert!(!matches(&endpoint(0x0FD9, 0x0081, 0, 0x0001, 2)));
    }

    #[test]
    fn full_frame_encodes_fifteen_zero_padded_jpeg_reports_in_order() {
        let colors = [Rgb8::new(0x12, 0x34, 0x56); BUTTON_COUNT];
        let transaction = FullFrameTransaction::new(&colors).unwrap();
        assert_eq!(transaction.reports().len(), BUTTON_COUNT);
        for (index, report) in transaction.reports().iter().enumerate() {
            let bytes = report.as_bytes();
            let jpeg_length = usize::from(bytes[4]) | (usize::from(bytes[5]) << 8);
            assert_eq!(
                &bytes[..4],
                &[
                    0x02,
                    0x07,
                    u8::try_from(index).expect("button count is below 256"),
                    0x01,
                ]
            );
            assert_eq!(&bytes[8..10], &[0xFF, 0xD8]);
            assert_eq!(&bytes[8 + jpeg_length - 2..8 + jpeg_length], &[0xFF, 0xD9]);
            assert!(bytes[8 + jpeg_length..].iter().all(|byte| *byte == 0));
        }

        let mut writer = RecordingWriter::default();
        transaction.apply(&mut writer).unwrap();
        assert_eq!(writer.reports.len(), BUTTON_COUNT);
        assert_eq!(writer.reports[14][2], 14);
    }

    #[test]
    fn windows_payload_length_is_valid_but_shorter_writes_stop_the_frame() {
        let transaction = FullFrameTransaction::new(&[Rgb8::BLACK; BUTTON_COUNT]).unwrap();
        let mut windows_writer = RecordingWriter {
            next_len: Some(OUTPUT_REPORT_LEN - 1),
            ..RecordingWriter::default()
        };
        transaction.apply(&mut windows_writer).unwrap();
        assert_eq!(windows_writer.reports.len(), BUTTON_COUNT);

        let mut short_writer = RecordingWriter {
            next_len: Some(OUTPUT_REPORT_LEN - 2),
            ..RecordingWriter::default()
        };
        assert_eq!(
            transaction.apply(&mut short_writer),
            Err(ExactWriteError::ShortWrite {
                expected: OUTPUT_REPORT_LEN - 1,
                actual: OUTPUT_REPORT_LEN - 2,
            })
        );
        assert_eq!(short_writer.reports.len(), 1);
    }

    #[test]
    fn count_features_matrix_and_model_are_preserved() {
        assert!(FullFrameTransaction::new(&[Rgb8::BLACK; BUTTON_COUNT - 1]).is_err());
        assert_eq!(
            BrightnessTransaction::new(77).report().as_bytes()[..3],
            [3, 8, 77]
        );
        assert_eq!(ResetTransaction::default().report().as_bytes()[..2], [3, 2]);
        assert_eq!(MATRIX_MAP[2][4], 14);
        let device = description("Elgato Stream Deck MK.2");
        assert_eq!(device.led_names.len(), BUTTON_COUNT);
        assert_eq!(device.led_names[0], "Button 1");
        assert_eq!(device.led_names[14], "Button 15");
    }
}
