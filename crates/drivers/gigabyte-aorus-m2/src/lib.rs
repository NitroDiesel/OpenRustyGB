#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const FEATURE_REPORT_LEN: usize = 8;
pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x1044,
    product_id: 0x7A40,
    interface_number: Some(3),
    usage_page: Some(0xFF01),
    usage: Some(0x0001),
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AorusMode {
    Direct = 0x00,
    Static = 0x01,
    Breathing = 0x02,
    SpectrumCycle = 0x03,
    Flashing = 0x04,
    DoubleFlash = 0x05,
    Off = 0xFF,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidSettings {
    pub brightness: u8,
    pub speed: u8,
}

impl fmt::Display for InvalidSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Aorus M2 brightness must be 0..100 and speed must be 0..22, got brightness {} and speed {}",
            self.brightness, self.speed
        )
    }
}

impl std::error::Error for InvalidSettings {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectColorTransaction(OutputReport<FEATURE_REPORT_LEN>);

impl DirectColorTransaction {
    #[must_use]
    pub const fn new(color: Rgb8) -> Self {
        Self(OutputReport::from_array([
            0xCD, 0, color.r, color.g, color.b, 0, 0, 0,
        ]))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<FEATURE_REPORT_LEN> {
        &self.0
    }

    /// Sends one direct-color feature report.
    ///
    /// # Errors
    ///
    /// Returns the HID transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        send_feature(writer, &self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction(OutputReport<FEATURE_REPORT_LEN>);

impl ModeTransaction {
    /// Builds the native hardware-mode feature report.
    ///
    /// Direct is encoded as Static here because the native controller uses the
    /// hardware-mode packet only to set Direct-mode brightness. Off is Static
    /// with black and zero brightness.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidSettings`] outside the native UI ranges.
    pub fn new(
        mode: AorusMode,
        color: Rgb8,
        brightness: u8,
        speed: u8,
    ) -> Result<Self, InvalidSettings> {
        if brightness > 100 || speed > 22 {
            return Err(InvalidSettings { brightness, speed });
        }
        let (encoded_mode, encoded_color, encoded_brightness, encoded_speed) = match mode {
            AorusMode::Direct => (AorusMode::Static as u8, color, brightness, 0),
            AorusMode::Off => (AorusMode::Static as u8, Rgb8::BLACK, 0, 0),
            _ => (mode as u8, color, brightness, speed),
        };
        Ok(Self(OutputReport::from_array([
            0xCC,
            encoded_mode,
            encoded_brightness,
            encoded_color.r,
            encoded_color.g,
            encoded_color.b,
            encoded_speed,
            0,
        ])))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<FEATURE_REPORT_LEN> {
        &self.0
    }

    /// Sends one hardware-mode feature report.
    ///
    /// # Errors
    ///
    /// Returns the HID transport error.
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
    let brightness = Some(BrightnessRange {
        min: 0,
        max: 100,
        current: 100,
    });
    let speed = Some(SpeedRange {
        min: 22,
        max: 0,
        current: 11,
    });
    let definitions = [
        (
            "Direct",
            AorusMode::Direct,
            ModeColorMode::PerLed,
            None,
            brightness,
        ),
        (
            "Static",
            AorusMode::Static,
            ModeColorMode::PerLed,
            None,
            brightness,
        ),
        (
            "Breathing",
            AorusMode::Breathing,
            ModeColorMode::PerLed,
            speed,
            brightness,
        ),
        (
            "Spectrum Cycle",
            AorusMode::SpectrumCycle,
            ModeColorMode::None,
            speed,
            brightness,
        ),
        (
            "Flashing",
            AorusMode::Flashing,
            ModeColorMode::PerLed,
            speed,
            brightness,
        ),
        (
            "Double Flash",
            AorusMode::DoubleFlash,
            ModeColorMode::PerLed,
            speed,
            brightness,
        ),
        ("Off", AorusMode::Off, ModeColorMode::None, None, None),
    ];
    ControllerDescription {
        name: device_name.into(),
        vendor: "Gigabyte".into(),
        description: "Gigabyte Mouse Device".into(),
        device_type: DeviceType::Mouse,
        modes: definitions
            .into_iter()
            .map(
                |(name, mode, color_mode, speed, brightness)| ModeDescription {
                    name: name.into(),
                    value: if mode == AorusMode::Off {
                        AorusMode::Static as u32
                    } else {
                        mode as u32
                    },
                    color_mode,
                    speed,
                    brightness,
                },
            )
            .collect(),
        zone_names: vec!["Mouse".into()],
        led_names: vec!["LED 1".into()],
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::EFFECTS)
            .union(ControllerCapabilities::BRIGHTNESS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn endpoint(interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"aorus-test"[..]),
            0x1044,
            0x7A40,
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
        assert!(matches(&endpoint(3, 0xFF01, 1)));
        assert!(!matches(&endpoint(2, 0xFF01, 1)));
        assert!(!matches(&endpoint(3, 0xFF00, 1)));
    }

    #[test]
    fn direct_and_hardware_packets_are_byte_exact() {
        assert_eq!(
            DirectColorTransaction::new(Rgb8::new(1, 2, 3))
                .report()
                .as_bytes(),
            &[0xCD, 0, 1, 2, 3, 0, 0, 0]
        );
        let mode = ModeTransaction::new(AorusMode::Breathing, Rgb8::new(4, 5, 6), 100, 22).unwrap();
        assert_eq!(mode.report().as_bytes(), &[0xCC, 2, 100, 4, 5, 6, 22, 0]);
    }

    #[test]
    fn direct_brightness_workaround_and_off_are_preserved() {
        let direct = ModeTransaction::new(AorusMode::Direct, Rgb8::new(1, 2, 3), 50, 22).unwrap();
        assert_eq!(direct.report().as_bytes(), &[0xCC, 1, 50, 1, 2, 3, 0, 0]);
        let off = ModeTransaction::new(AorusMode::Off, Rgb8::new(9, 9, 9), 100, 22).unwrap();
        assert_eq!(off.report().as_bytes(), &[0xCC, 1, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn settings_and_model_shape_are_exact() {
        assert!(ModeTransaction::new(AorusMode::Static, Rgb8::BLACK, 101, 0).is_err());
        assert!(ModeTransaction::new(AorusMode::Static, Rgb8::BLACK, 100, 23).is_err());
        let device = description("Gigabyte Aorus M2");
        assert_eq!(device.modes.len(), 7);
        assert_eq!(device.zone_names, ["Mouse"]);
        assert_eq!(device.modes[2].speed.unwrap().current, 11);
        assert_eq!(device.modes[6].value, 1);
    }
}
