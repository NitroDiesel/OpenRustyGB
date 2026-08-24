#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    ExactWriteError, HidDeviceMatch, HidEndpointInfo, OutputReport, OutputWriter, write_exact,
};

pub const LED_COUNT: usize = 49;
pub const MATRIX_SIZE: usize = 7;
pub const OUTPUT_REPORT_LEN: usize = 64;
pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x1A86,
    product_id: 0xE316,
    interface_number: Some(0),
    usage_page: Some(0xFF00),
    usage: Some(1),
};

pub const MATRIX_MAP: [usize; LED_COUNT] = [
    6, 5, 4, 3, 2, 1, 0, 7, 8, 9, 10, 11, 12, 13, 20, 19, 18, 17, 16, 15, 14, 21, 22, 23, 24, 25,
    26, 27, 34, 33, 32, 31, 30, 29, 28, 35, 36, 37, 38, 39, 40, 41, 48, 47, 46, 45, 44, 43, 42,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidColorCount(pub usize);

impl fmt::Display for InvalidColorCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Skydimo frame requires {LED_COUNT} colors, got {}",
            self.0
        )
    }
}

impl std::error::Error for InvalidColorCount {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrameTransaction {
    reports: [OutputReport<OUTPUT_REPORT_LEN>; 4],
}

impl FrameTransaction {
    /// Builds three GRB data reports and the native frame-end report.
    ///
    /// # Errors
    ///
    /// Returns an error unless exactly 49 matrix colors are supplied.
    pub fn new(colors: &[Rgb8]) -> Result<Self, InvalidColorCount> {
        if colors.len() != LED_COUNT {
            return Err(InvalidColorCount(colors.len()));
        }
        let mut frame = [0; LED_COUNT * 3];
        for (index, color) in colors.iter().enumerate() {
            frame[index * 3..index * 3 + 3].copy_from_slice(&[color.g, color.r, color.b]);
        }
        let mut reports = [
            data_report(0, &frame[..60]),
            data_report(20, &frame[60..120]),
            data_report(40, &frame[120..]),
            OutputReport::from_array([0; OUTPUT_REPORT_LEN]),
        ];
        let mut end = [0; OUTPUT_REPORT_LEN];
        end[0] = 1;
        end[1] = 0xFF;
        end[2] = 0xFF;
        end[3] = 49;
        end[60] = 0x1E;
        reports[3] = OutputReport::from_array(end);
        Ok(Self { reports })
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<OUTPUT_REPORT_LEN>; 4] {
        &self.reports
    }

    /// Sends the three chunks and frame-end report in native order.
    ///
    /// # Errors
    ///
    /// Stops on a transport failure or short write.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        for report in &self.reports {
            write_exact(writer, report)?;
        }
        Ok(())
    }
}

fn data_report(start_led: u8, chunk: &[u8]) -> OutputReport<OUTPUT_REPORT_LEN> {
    let mut report = [0; OUTPUT_REPORT_LEN];
    report[0] = 1;
    report[1] = start_led;
    report[3..3 + chunk.len()].copy_from_slice(chunk);
    report[63] = crc8(&report[..63]);
    OutputReport::from_array(report)
}

#[must_use]
pub fn crc8(data: &[u8]) -> u8 {
    let mut crc = 0_u8;
    for byte in data {
        crc ^= *byte;
        for _ in 0..8 {
            crc = if crc & 0x80 != 0 {
                (crc << 1) ^ 0x07
            } else {
                crc << 1
            };
        }
    }
    crc
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description(endpoint: &HidEndpointInfo) -> ControllerDescription {
    let manufacturer = endpoint.manufacturer.as_deref().unwrap_or("Skydimo");
    let product = endpoint.product.as_deref().unwrap_or("HID Device");
    ControllerDescription {
        name: format!("{manufacturer} {product}"),
        vendor: "Skydimo".into(),
        description: "Skydimo HID Device".into(),
        device_type: DeviceType::HeadsetStand,
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 0,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: None,
        }],
        zone_names: vec!["Matrix".into()],
        led_names: (1..=LED_COUNT)
            .map(|index| format!("LED {index}"))
            .collect(),
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
            Arc::from(&b"skydimo-test"[..]),
            0x1A86,
            0xE316,
            interface,
            page,
            usage,
            Some(Arc::from("Sky")),
            Some(Arc::from("SK0902")),
            None,
        )
    }

    #[test]
    fn matcher_is_exact() {
        assert!(matches(&endpoint(0, 0xFF00, 1)));
        assert!(!matches(&endpoint(1, 0xFF00, 1)));
        assert!(!matches(&endpoint(0, 0xFF01, 1)));
    }

    #[test]
    fn frame_chunks_are_grb_crc_protected_and_terminated() {
        let colors: Vec<_> = (0_u8..49)
            .map(|index| Rgb8::new(index, index + 1, index + 2))
            .collect();
        let frame = FrameTransaction::new(&colors).unwrap();
        assert_eq!(&frame.reports()[0].as_bytes()[..6], &[1, 0, 0, 1, 0, 2]);
        assert_eq!(
            frame.reports()[0].as_bytes()[63],
            crc8(&frame.reports()[0].as_bytes()[..63])
        );
        assert_eq!(frame.reports()[1].as_bytes()[1], 20);
        assert_eq!(frame.reports()[2].as_bytes()[1], 40);
        assert_eq!(&frame.reports()[3].as_bytes()[..4], &[1, 0xFF, 0xFF, 49]);
        assert_eq!(frame.reports()[3].as_bytes()[60], 0x1E);
        assert_eq!(frame.reports()[3].as_bytes()[63], 0);
    }

    #[test]
    fn matrix_serpentine_map_and_model_shape_are_preserved() {
        assert_eq!(
            &MATRIX_MAP[..14],
            &[6, 5, 4, 3, 2, 1, 0, 7, 8, 9, 10, 11, 12, 13]
        );
        assert!(FrameTransaction::new(&[Rgb8::BLACK; 48]).is_err());
        let device = description(&endpoint(0, 0xFF00, 1));
        assert_eq!(device.name, "Sky SK0902");
        assert_eq!(device.led_names.len(), 49);
        assert_eq!(device.zone_names, ["Matrix"]);
    }
}
