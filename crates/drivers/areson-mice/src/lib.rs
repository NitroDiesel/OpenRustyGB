#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const FEATURE_REPORT_LEN: usize = 17;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AresonModel {
    pub name: &'static str,
    pub matcher: HidDeviceMatch,
}

const fn model(name: &'static str, product_id: u16) -> AresonModel {
    AresonModel {
        name,
        matcher: HidDeviceMatch {
            vendor_id: 0x25A7,
            product_id,
            interface_number: Some(1),
            usage_page: Some(0xFF02),
            usage: Some(2),
        },
    }
}

pub const MODELS: [AresonModel; 6] = [
    model("ZET GAMING Edge Air Pro (Wireless)", 0xFA3F),
    model("ZET GAMING Edge Air Pro", 0xFA40),
    model("ZET GAMING Edge Air Elit (Wireless)", 0xFA48),
    model("ZET GAMING Edge Air Elit", 0xFA49),
    model("Redragon M914 NIX (Wireless)", 0xFA7C),
    model("Redragon M914 NIX", 0xFA7B),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum AresonMode {
    RainbowWave = 0x00,
    Breathing = 0x01,
    Static = 0x02,
    SpectrumCycle = 0x03,
    Off = 0x04,
    SingleColorWave = 0x05,
    ColorfulBreathing = 0x07,
}

impl AresonMode {
    const fn has_color(self) -> bool {
        matches!(self, Self::Static | Self::Breathing | Self::SingleColorWave)
    }

    const fn has_speed(self) -> bool {
        !matches!(self, Self::Static | Self::Off)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidSettings {
    Brightness(u8),
    Speed(u8),
}

impl fmt::Display for InvalidSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Brightness(value) => {
                write!(f, "Areson brightness must be in 1..=10, got {value}")
            }
            Self::Speed(value) => write!(f, "Areson speed must be in 1..=10, got {value}"),
        }
    }
}

impl std::error::Error for InvalidSettings {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction(OutputReport<FEATURE_REPORT_LEN>);

impl ModeTransaction {
    /// Serializes one complete native hardware-mode feature report.
    ///
    /// # Errors
    ///
    /// Returns an error for brightness or mode-dependent speed values outside
    /// the native one-based lookup tables.
    pub fn new(
        mode: AresonMode,
        color: Rgb8,
        brightness: u8,
        speed: u8,
    ) -> Result<Self, InvalidSettings> {
        if mode != AresonMode::Off && !(1..=10).contains(&brightness) {
            return Err(InvalidSettings::Brightness(brightness));
        }
        if mode.has_speed() && !(1..=10).contains(&speed) {
            return Err(InvalidSettings::Speed(speed));
        }

        let color = if mode.has_color() { color } else { Rgb8::BLACK };
        let mut report = [0; FEATURE_REPORT_LEN];
        report[0] = 0x08;
        report[1] = 0x07;
        report[4] = 0xA0;
        report[5] = 0x07;
        report[6] = mode as u8;
        report[7..10].copy_from_slice(&[color.r, color.g, color.b]);
        report[10] = speed_value(mode, speed);
        if mode != AresonMode::Off {
            report[11] = BRIGHTNESS_VALUES[usize::from(brightness - 1)];
        }
        report[12] = report[6..12]
            .iter()
            .fold(0x55_u8, |checksum, value| checksum.wrapping_sub(*value));
        report[16] = 0x4A;
        Ok(Self(OutputReport::from_array(report)))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<FEATURE_REPORT_LEN> {
        &self.0
    }

    /// Sends the native feature report.
    ///
    /// # Errors
    ///
    /// Returns an error from the feature-report transport.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        send_feature(writer, &self.0)
    }
}

const BRIGHTNESS_VALUES: [u8; 10] = [0x19, 0x32, 0x4B, 0x64, 0x7D, 0x96, 0xAF, 0xC8, 0xE1, 0xFF];
const HIGH_SPEED_VALUES: [u8; 10] = [0xFF, 0xE6, 0xD2, 0xBE, 0xAA, 0x96, 0x82, 0x6E, 0x46, 0x28];
const LOW_SPEED_VALUES: [u8; 10] = [0x2D, 0x28, 0x23, 0x1E, 0x19, 0x13, 0x0F, 0x0A, 0x05, 0x03];

fn speed_value(mode: AresonMode, speed: u8) -> u8 {
    let index = usize::from(speed.saturating_sub(1));
    match mode {
        AresonMode::RainbowWave => LOW_SPEED_VALUES[index],
        AresonMode::Breathing
        | AresonMode::SpectrumCycle
        | AresonMode::SingleColorWave
        | AresonMode::ColorfulBreathing => HIGH_SPEED_VALUES[index],
        AresonMode::Static | AresonMode::Off => 0,
    }
}

#[must_use]
pub fn match_model(endpoint: &HidEndpointInfo) -> Option<&'static AresonModel> {
    MODELS.iter().find(|model| model.matcher.matches(endpoint))
}

fn mode_description(
    name: &str,
    value: u32,
    color_mode: ModeColorMode,
    speed: Option<SpeedRange>,
    brightness: Option<BrightnessRange>,
) -> ModeDescription {
    ModeDescription {
        name: name.into(),
        value,
        color_mode,
        speed,
        brightness,
    }
}

#[must_use]
pub fn description(device_name: &str) -> ControllerDescription {
    let brightness = Some(BrightnessRange {
        min: 1,
        max: 10,
        current: 10,
    });
    let speed = |current: u32| {
        Some(SpeedRange {
            min: 1,
            max: 10,
            current,
        })
    };
    ControllerDescription {
        name: device_name.into(),
        vendor: "Areson".into(),
        description: "Areson mouse".into(),
        device_type: DeviceType::Mouse,
        modes: vec![
            mode_description("Static", 2, ModeColorMode::PerLed, None, brightness),
            mode_description(
                "Rainbow Wave",
                0,
                ModeColorMode::None,
                speed(10),
                brightness,
            ),
            mode_description("Breathing", 1, ModeColorMode::PerLed, speed(1), brightness),
            mode_description(
                "Spectrum Cycle",
                3,
                ModeColorMode::None,
                speed(10),
                brightness,
            ),
            mode_description(
                "Single Color Wave",
                5,
                ModeColorMode::PerLed,
                speed(10),
                brightness,
            ),
            mode_description(
                "Colorful Breathing",
                7,
                ModeColorMode::None,
                speed(10),
                brightness,
            ),
            mode_description("Off", 4, ModeColorMode::None, None, None),
        ],
        zone_names: vec!["Mouse".into()],
        led_names: vec!["LED 1".into()],
        capabilities: ControllerCapabilities::PER_LED_COLOR
            .union(ControllerCapabilities::EFFECTS)
            .union(ControllerCapabilities::BRIGHTNESS),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn endpoint(product: u16, interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"areson-test"[..]),
            0x25A7,
            product,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn all_six_models_require_the_exact_interface_and_usage() {
        for model in MODELS {
            assert_eq!(
                match_model(&endpoint(model.matcher.product_id, 1, 0xFF02, 2)),
                Some(&model)
            );
        }
        assert!(match_model(&endpoint(0xFA3F, 0, 0xFF02, 2)).is_none());
        assert!(match_model(&endpoint(0xFA3F, 1, 0xFF00, 2)).is_none());
    }

    #[test]
    fn static_rainbow_and_off_reports_preserve_native_bytes() {
        let static_report =
            ModeTransaction::new(AresonMode::Static, Rgb8::new(0x11, 0x22, 0x33), 10, 99).unwrap();
        assert_eq!(
            static_report.report().as_bytes(),
            &[
                8, 7, 0, 0, 0xA0, 7, 2, 0x11, 0x22, 0x33, 0, 0xFF, 0xEE, 0, 0, 0, 0x4A
            ]
        );

        let rainbow =
            ModeTransaction::new(AresonMode::RainbowWave, Rgb8::new(1, 2, 3), 1, 10).unwrap();
        assert_eq!(
            &rainbow.report().as_bytes()[6..13],
            &[0, 0, 0, 0, 3, 0x19, 0x39]
        );

        let off = ModeTransaction::new(AresonMode::Off, Rgb8::new(1, 2, 3), 0, 0).unwrap();
        assert_eq!(&off.report().as_bytes()[6..13], &[4, 0, 0, 0, 0, 0, 0x51]);
    }

    #[test]
    fn settings_tables_and_model_shape_are_preserved() {
        assert!(ModeTransaction::new(AresonMode::Breathing, Rgb8::BLACK, 0, 1).is_err());
        assert!(ModeTransaction::new(AresonMode::Breathing, Rgb8::BLACK, 1, 11).is_err());
        let breathing = ModeTransaction::new(AresonMode::Breathing, Rgb8::BLACK, 1, 1).unwrap();
        assert_eq!(breathing.report().as_bytes()[10], 0xFF);
        let device = description("Redragon M914 NIX");
        assert_eq!(device.modes.len(), 7);
        assert_eq!(device.zone_names, ["Mouse"]);
        assert_eq!(device.led_names, ["LED 1"]);
    }
}
