#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription,
    Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    ExactWriteError, HidDeviceMatch, HidEndpointInfo, OutputReport, OutputWriter, write_exact,
};

pub const OUTPUT_REPORT_LEN: usize = 63;
pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x0483,
    product_id: 0x5750,
    interface_number: Some(0),
    usage_page: Some(0xFFA0),
    usage: Some(1),
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum EkMode {
    Static = 0,
    Breathing = 1,
    Fading = 2,
    Marquee = 3,
    CoveringMarquee = 4,
    Pulse = 5,
    SpectrumWave = 6,
    Alternating = 7,
    Candle = 8,
}

impl EkMode {
    const fn index(self) -> usize {
        self as usize
    }
}

const MODE_DATA: [[u8; 16]; 9] = [
    [
        0x10, 0x12, 0x29, 0xAA, 1, 0x10, 0xA2, 0x60, 0, 0x10, 0x20, 1, 1, 0, 0xFF, 0x64,
    ],
    [
        0x10, 0x12, 0x29, 0xAA, 1, 0x10, 0xA2, 0x60, 0, 0x10, 0x20, 1, 2, 0, 0xFF, 0x64,
    ],
    [
        0x10, 0x12, 0x29, 0xAA, 1, 0x10, 0xA2, 0x60, 0, 0x10, 0x20, 1, 3, 0xFF, 0xFF, 0x64,
    ],
    [
        0x10, 0x12, 0x29, 0xAA, 1, 0x10, 0xA2, 0x60, 0, 0x10, 0x20, 1, 4, 0, 0xFF, 0x64,
    ],
    [
        0x10, 0x12, 0x29, 0xAA, 1, 0x10, 0xA2, 0x60, 0, 0x10, 0x20, 1, 5, 0, 0xFF, 0x64,
    ],
    [
        0x10, 0x12, 0x29, 0xAA, 1, 0x10, 0xA2, 0x60, 0, 0x10, 0x20, 1, 6, 0, 0xFF, 0x64,
    ],
    [
        0x10, 0x12, 0x29, 0xAA, 1, 0x10, 0xA2, 0x60, 0, 0x10, 0x20, 1, 7, 0, 0xFF, 0x64,
    ],
    [
        0x10, 0x12, 0x29, 0xAA, 1, 0x10, 0xA2, 0x60, 0, 0x10, 0x20, 1, 8, 0, 0xFF, 0x64,
    ],
    [
        0x10, 0x12, 0x29, 0xAA, 1, 0x10, 0xA2, 0x60, 0, 0x10, 0x20, 1, 9, 0, 0xFF, 0x64,
    ],
];
const SPEED_DATA: [u8; 9] = [0, 0x0C, 0x19, 0x25, 0x32, 0x3E, 0x4B, 0x57, 0x64];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidSpeed(pub u8);

impl fmt::Display for InvalidSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EK Loop Connect speed must be in 0..=8, got {}", self.0)
    }
}

impl std::error::Error for InvalidSpeed {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction(OutputReport<OUTPUT_REPORT_LEN>);

impl ModeTransaction {
    /// Builds the native mode report with the selected color and speed.
    ///
    /// # Errors
    ///
    /// Returns an error when an animated mode uses a speed above eight.
    pub fn new(mode: EkMode, color: Rgb8, speed: u8) -> Result<Self, InvalidSpeed> {
        if mode != EkMode::Static && speed > 8 {
            return Err(InvalidSpeed(speed));
        }
        let mut report = [0; OUTPUT_REPORT_LEN];
        report[..16].copy_from_slice(&MODE_DATA[mode.index()]);
        report[10] = 0x10;
        report[14] = if mode == EkMode::Static {
            0
        } else {
            SPEED_DATA[usize::from(speed)]
        };
        report[16..19].copy_from_slice(&[color.r, color.g, color.b]);
        report[47] = 0xFF;
        Ok(Self(OutputReport::from_array(report)))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<OUTPUT_REPORT_LEN> {
        &self.0
    }

    /// Sends one complete hardware-mode output report.
    ///
    /// # Errors
    ///
    /// Returns a transport failure or short write.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        write_exact(writer, &self.0)
    }
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description(endpoint: &HidEndpointInfo) -> ControllerDescription {
    let name = match (&endpoint.manufacturer, &endpoint.product) {
        (Some(manufacturer), Some(product)) => format!("{manufacturer} {product}"),
        _ => "EK Loop Connect".into(),
    };
    let speed = Some(SpeedRange {
        min: 0,
        max: 8,
        current: 4,
    });
    let modes = [
        ("Static", EkMode::Static, ModeColorMode::PerLed, None),
        ("Breathing", EkMode::Breathing, ModeColorMode::PerLed, speed),
        ("Fading", EkMode::Fading, ModeColorMode::None, speed),
        ("Marquee", EkMode::Marquee, ModeColorMode::PerLed, speed),
        (
            "Covering Marquee",
            EkMode::CoveringMarquee,
            ModeColorMode::None,
            speed,
        ),
        ("Pulse", EkMode::Pulse, ModeColorMode::PerLed, speed),
        (
            "Spectrum_Wave",
            EkMode::SpectrumWave,
            ModeColorMode::None,
            speed,
        ),
        (
            "Alternating",
            EkMode::Alternating,
            ModeColorMode::PerLed,
            speed,
        ),
        ("Candle", EkMode::Candle, ModeColorMode::PerLed, speed),
    ];
    ControllerDescription {
        description: name.clone(),
        name,
        vendor: "EK".into(),
        device_type: DeviceType::LedStrip,
        modes: modes
            .into_iter()
            .map(|(name, mode, color_mode, speed)| ModeDescription {
                name: name.into(),
                value: mode as u32,
                color_mode,
                speed,
                brightness: None,
            })
            .collect(),
        zone_names: vec!["Loop Connect".into()],
        led_names: vec!["EK LED".into()],
        capabilities: ControllerCapabilities::PER_LED_COLOR.union(ControllerCapabilities::EFFECTS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn endpoint(interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"ek-test"[..]),
            0x0483,
            0x5750,
            interface,
            page,
            usage,
            Some(Arc::from("EKWB")),
            Some(Arc::from("Loop Connect")),
            None,
        )
    }

    #[test]
    fn matcher_is_exact() {
        assert!(matches(&endpoint(0, 0xFFA0, 1)));
        assert!(!matches(&endpoint(1, 0xFFA0, 1)));
        assert!(!matches(&endpoint(0, 0xFF00, 1)));
    }

    #[test]
    fn static_and_animated_packets_preserve_templates() {
        let static_report = ModeTransaction::new(EkMode::Static, Rgb8::new(1, 2, 3), 255).unwrap();
        assert_eq!(
            &static_report.report().as_bytes()[..19],
            &[
                0x10, 0x12, 0x29, 0xAA, 1, 0x10, 0xA2, 0x60, 0, 0x10, 0x10, 1, 1, 0, 0, 0x64, 1, 2,
                3
            ]
        );
        assert_eq!(static_report.report().as_bytes()[47], 0xFF);

        let candle = ModeTransaction::new(EkMode::Candle, Rgb8::new(4, 5, 6), 8).unwrap();
        assert_eq!(candle.report().as_bytes()[12], 9);
        assert_eq!(candle.report().as_bytes()[14], 0x64);
        assert_eq!(&candle.report().as_bytes()[16..19], &[4, 5, 6]);
    }

    #[test]
    fn speed_bounds_and_model_shape_are_preserved() {
        assert!(ModeTransaction::new(EkMode::Breathing, Rgb8::BLACK, 9).is_err());
        let device = description(&endpoint(0, 0xFFA0, 1));
        assert_eq!(device.name, "EKWB Loop Connect");
        assert_eq!(device.modes.len(), 9);
        assert_eq!(device.zone_names, ["Loop Connect"]);
        assert_eq!(device.led_names, ["EK LED"]);
    }
}
