#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const FEATURE_REPORT_LEN: usize = 64;
pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x22D4,
    product_id: 0x1503,
    interface_number: Some(1),
    usage_page: Some(0xFF01),
    usage: Some(0x0002),
};
const PROTOCOL_SETTING_MAX: u8 = 64;
const DEFAULT_PALETTE: [u8; 21] = [
    0xFF, 0x00, 0x00, 0xFF, 0xA5, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x7F, 0xFF, 0x00,
    0x00, 0xFF, 0x8B, 0x00, 0xFF,
];

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GloriousMode {
    Off = 0x00,
    Custom = 0x01,
    Flashing = 0x02,
    Breathing = 0x04,
    SpectrumCycle = 0x06,
    RainbowWave = 0x10,
    Chase = 0x11,
    Wave = 0x14,
    SpectrumBreathing = 0x15,
}

impl GloriousMode {
    const fn uses_color(self) -> bool {
        matches!(self, Self::Custom | Self::Breathing)
    }

    const fn uses_brightness(self) -> bool {
        !matches!(self, Self::Off)
    }

    const fn uses_speed(self) -> bool {
        !matches!(self, Self::Off | Self::Custom)
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
                write!(
                    f,
                    "Glorious Model I brightness must be in 0..=100, got {value}"
                )
            }
            Self::Speed(value) => {
                write!(f, "Glorious Model I speed must be in 1..=100, got {value}")
            }
        }
    }
}

impl std::error::Error for InvalidSettings {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction(OutputReport<FEATURE_REPORT_LEN>);

impl ModeTransaction {
    /// Builds the complete native mode report.
    ///
    /// Public settings retain the upstream `0..=100` ranges. The device
    /// protocol safely saturates both fields at `64`.
    ///
    /// # Errors
    /// Returns an error for settings outside the ranges exposed upstream.
    pub fn new(
        mode: GloriousMode,
        color: Rgb8,
        brightness: u8,
        speed: u8,
    ) -> Result<Self, InvalidSettings> {
        if mode.uses_brightness() && brightness > 100 {
            return Err(InvalidSettings::Brightness(brightness));
        }
        if mode.uses_speed() && !(1..=100).contains(&speed) {
            return Err(InvalidSettings::Speed(speed));
        }

        let mut bytes = [0; FEATURE_REPORT_LEN];
        bytes[0] = 0xA1;
        bytes[1] = 0x0C;
        bytes[5] = 0x01;
        bytes[16] = mode as u8;
        if mode.uses_color() {
            bytes[17..20].copy_from_slice(&[color.r, color.g, color.b]);
        } else {
            bytes[17..38].copy_from_slice(&DEFAULT_PALETTE);
        }
        bytes[56] = if mode.uses_brightness() {
            brightness.min(PROTOCOL_SETTING_MAX)
        } else {
            0
        };
        bytes[58] = if mode.uses_speed() {
            speed.min(PROTOCOL_SETTING_MAX)
        } else {
            0
        };
        Ok(Self(OutputReport::from_array(bytes)))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<FEATURE_REPORT_LEN> {
        &self.0
    }

    /// Sends the complete native feature report.
    ///
    /// # Errors
    /// Returns the HID feature-report transport error.
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
pub fn firmware_version(release_number: u16) -> String {
    release_number.to_string()
}

fn mode_description(
    name: &str,
    mode: GloriousMode,
    color_mode: ModeColorMode,
    speed: Option<SpeedRange>,
    brightness: Option<BrightnessRange>,
) -> ModeDescription {
    ModeDescription {
        name: name.into(),
        value: mode as u32,
        color_mode,
        speed,
        brightness,
    }
}

#[must_use]
pub fn description(vendor: &str) -> ControllerDescription {
    let brightness = Some(BrightnessRange {
        min: 0,
        max: 100,
        current: 50,
    });
    let speed = Some(SpeedRange {
        min: 1,
        max: 100,
        current: 50,
    });
    ControllerDescription {
        name: "Glorious Model I".into(),
        vendor: vendor.into(),
        description: "Glorious Device".into(),
        device_type: DeviceType::Mouse,
        modes: vec![
            mode_description(
                "Custom",
                GloriousMode::Custom,
                ModeColorMode::PerLed,
                None,
                brightness,
            ),
            mode_description(
                "Flashing",
                GloriousMode::Flashing,
                ModeColorMode::None,
                speed,
                brightness,
            ),
            mode_description(
                "Chase",
                GloriousMode::Chase,
                ModeColorMode::None,
                speed,
                brightness,
            ),
            mode_description(
                "Wave",
                GloriousMode::Wave,
                ModeColorMode::None,
                speed,
                brightness,
            ),
            mode_description(
                "Spectrum Cycle",
                GloriousMode::SpectrumCycle,
                ModeColorMode::None,
                speed,
                brightness,
            ),
            mode_description(
                "Breathing",
                GloriousMode::Breathing,
                ModeColorMode::PerLed,
                speed,
                brightness,
            ),
            mode_description(
                "Spectrum Breathing",
                GloriousMode::SpectrumBreathing,
                ModeColorMode::None,
                speed,
                brightness,
            ),
            mode_description(
                "Rainbow Wave",
                GloriousMode::RainbowWave,
                ModeColorMode::None,
                speed,
                brightness,
            ),
            mode_description("Off", GloriousMode::Off, ModeColorMode::None, None, None),
        ],
        zone_names: vec!["Mouse".into()],
        led_names: vec!["LED".into()],
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
            Arc::from(&b"glorious-test"[..]),
            0x22D4,
            0x1503,
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
        assert!(matches(&endpoint(1, 0xFF01, 2)));
        assert!(!matches(&endpoint(0, 0xFF01, 2)));
        assert!(!matches(&endpoint(1, 0xFF00, 2)));
        assert!(!matches(&endpoint(1, 0xFF01, 1)));
    }

    #[test]
    fn custom_and_breathing_use_the_selected_color() {
        for mode in [GloriousMode::Custom, GloriousMode::Breathing] {
            let transaction = ModeTransaction::new(mode, Rgb8::new(1, 2, 3), 50, 50).unwrap();
            let bytes = transaction.report().as_bytes();
            assert_eq!(&bytes[..6], &[0xA1, 0x0C, 0, 0, 0, 1]);
            assert_eq!(bytes[16], mode as u8);
            assert_eq!(&bytes[17..20], &[1, 2, 3]);
        }
    }

    #[test]
    fn hardware_effects_use_the_native_palette_and_saturated_settings() {
        let transaction =
            ModeTransaction::new(GloriousMode::RainbowWave, Rgb8::BLACK, 100, 100).unwrap();
        let bytes = transaction.report().as_bytes();
        assert_eq!(&bytes[17..38], &DEFAULT_PALETTE);
        assert_eq!(bytes[56], 64);
        assert_eq!(bytes[58], 64);
        assert_eq!(bytes[60], 0);
    }

    #[test]
    fn settings_off_packet_firmware_and_model_are_preserved() {
        assert!(ModeTransaction::new(GloriousMode::Wave, Rgb8::BLACK, 101, 50).is_err());
        assert!(ModeTransaction::new(GloriousMode::Wave, Rgb8::BLACK, 50, 0).is_err());
        let off = ModeTransaction::new(GloriousMode::Off, Rgb8::BLACK, 255, 0).unwrap();
        assert_eq!(off.report().as_bytes()[16], 0);
        assert_eq!(off.report().as_bytes()[56], 0);
        assert_eq!(off.report().as_bytes()[58], 0);
        assert_eq!(firmware_version(513), "513");
        let device = description("Glorious");
        assert_eq!(device.modes.len(), 9);
        assert_eq!(device.zone_names, ["Mouse"]);
        assert_eq!(device.led_names, ["LED"]);
    }
}
