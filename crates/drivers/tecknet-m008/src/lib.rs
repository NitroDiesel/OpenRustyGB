#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x04D9,
    product_id: 0xFC05,
    interface_number: None,
    usage_page: Some(0xFFA0),
    usage: Some(0x0001),
};
pub const FEATURE_REPORT_LEN: usize = 16;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TecknetMode {
    Direct = 0x00,
    Breathing = 0x01,
    Off = 0xFF,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidSpeed(pub u8);

impl fmt::Display for InvalidSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tecknet breathing speed must be from 0 through 3, got {}",
            self.0
        )
    }
}

impl std::error::Error for InvalidSpeed {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeColorTransaction(OutputReport<FEATURE_REPORT_LEN>);

impl ModeColorTransaction {
    /// Builds the complete state report used by both mode and color updates.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidSpeed`] when a breathing speed cannot index the native table.
    pub fn new(mode: TecknetMode, color: Rgb8, speed: u8) -> Result<Self, InvalidSpeed> {
        let encoded_speed = match mode {
            TecknetMode::Breathing => [0x00, 0x06, 0x03, 0x01]
                .get(usize::from(speed))
                .copied()
                .ok_or(InvalidSpeed(speed))?,
            TecknetMode::Direct | TecknetMode::Off => 0,
        };
        let brightness = if mode == TecknetMode::Off { 0 } else { 3 };
        let mut bytes = [0; FEATURE_REPORT_LEN];
        bytes[..7].copy_from_slice(&[
            0x02,
            0x04,
            0xFF - color.r,
            0xFF - color.g,
            0xFF - color.b,
            brightness,
            encoded_speed,
        ]);
        Ok(Self(OutputReport::from_array(bytes)))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<FEATURE_REPORT_LEN> {
        &self.0
    }

    /// Sends one complete Tecknet state feature report.
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
    let fixed_brightness = Some(BrightnessRange {
        min: 0,
        max: 0,
        current: 0,
    });
    ControllerDescription {
        name: device_name.into(),
        vendor: "Tecknet".into(),
        description: device_name.into(),
        device_type: DeviceType::Mouse,
        modes: vec![
            ModeDescription {
                name: "Direct".into(),
                value: TecknetMode::Direct as u32,
                color_mode: ModeColorMode::PerLed,
                speed: Some(SpeedRange {
                    min: 0,
                    max: 0,
                    current: 0,
                }),
                brightness: fixed_brightness,
            },
            ModeDescription {
                name: "Off".into(),
                value: u32::from(TecknetMode::Off as u8),
                color_mode: ModeColorMode::None,
                speed: Some(SpeedRange {
                    min: 0,
                    max: 0,
                    current: 0,
                }),
                brightness: None,
            },
            ModeDescription {
                name: "Breathing".into(),
                value: TecknetMode::Breathing as u32,
                color_mode: ModeColorMode::PerLed,
                speed: Some(SpeedRange {
                    min: 1,
                    max: 3,
                    current: 2,
                }),
                brightness: fixed_brightness,
            },
        ],
        zone_names: vec!["Logo".into()],
        led_names: vec!["Logo".into()],
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::EFFECTS)
            .union(ControllerCapabilities::BRIGHTNESS),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn endpoint(interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"tecknet-test"[..]),
            0x04D9,
            0xFC05,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn matcher_accepts_any_interface_but_exact_usage() {
        assert!(matches(&endpoint(0, 0xFFA0, 1)));
        assert!(matches(&endpoint(3, 0xFFA0, 1)));
        assert!(!matches(&endpoint(0, 0xFFA1, 1)));
    }

    #[test]
    fn direct_report_inverts_rgb_and_uses_high_brightness() {
        let transaction =
            ModeColorTransaction::new(TecknetMode::Direct, Rgb8::new(0x12, 0x34, 0x56), 0).unwrap();
        assert_eq!(
            &transaction.report().as_bytes()[..7],
            &[2, 4, 0xED, 0xCB, 0xA9, 3, 0]
        );
    }

    #[test]
    fn breathing_speed_table_is_exact() {
        let expected = [0, 6, 3, 1];
        for (speed, encoded) in expected.into_iter().enumerate() {
            let speed = u8::try_from(speed).unwrap();
            let report =
                ModeColorTransaction::new(TecknetMode::Breathing, Rgb8::BLACK, speed).unwrap();
            assert_eq!(report.report().as_bytes()[6], encoded);
        }
        assert!(ModeColorTransaction::new(TecknetMode::Breathing, Rgb8::BLACK, 4).is_err());
    }

    #[test]
    fn off_is_safe_and_forces_zero_brightness_and_speed() {
        let transaction =
            ModeColorTransaction::new(TecknetMode::Off, Rgb8::new(1, 2, 3), 3).unwrap();
        assert_eq!(&transaction.report().as_bytes()[5..7], &[0, 0]);
    }

    #[test]
    fn description_preserves_three_modes_and_logo_shape() {
        let device = description("Tecknet M008");
        assert_eq!(device.modes.len(), 3);
        assert_eq!(device.modes[1].value, 0xFF);
        assert_eq!(device.modes[2].speed.unwrap().current, 2);
        assert_eq!(device.led_names, ["Logo"]);
    }
}
