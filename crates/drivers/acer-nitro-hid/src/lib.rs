#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const FEATURE_REPORT_LEN: usize = 11;
pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x0CF2,
    product_id: 0x5130,
    interface_number: None,
    usage_page: Some(0xFF5A),
    usage: Some(0x0001),
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcerDevice {
    Keyboard,
    ChassisLed,
}

impl AcerDevice {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Keyboard => "Acer Nitro HID Keyboard",
            Self::ChassisLed => "Acer Nitro HID LED Device",
        }
    }

    #[must_use]
    pub const fn led_count(self) -> usize {
        match self {
            Self::Keyboard => 4,
            Self::ChassisLed => 1,
        }
    }

    const fn device_id(self) -> u8 {
        match self {
            Self::Keyboard => 0x21,
            Self::ChassisLed => 0x65,
        }
    }

    fn supports(self, mode: AcerMode) -> bool {
        self == Self::Keyboard
            || matches!(
                mode,
                AcerMode::Direct | AcerMode::Static | AcerMode::Breathing | AcerMode::Neon
            )
    }

    const fn max_speed(self) -> u8 {
        match self {
            Self::Keyboard => 9,
            Self::ChassisLed => 5,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcerMode {
    Direct = 1,
    Static = 2,
    Breathing = 4,
    Neon = 5,
    Wave = 7,
    Shifting = 8,
    Zoom = 9,
    Meteor = 10,
    Twinkling = 11,
}

impl AcerMode {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Direct => "Direct",
            Self::Static => "Static",
            Self::Breathing => "Breathing",
            Self::Neon => "Neon",
            Self::Wave => "Wave",
            Self::Shifting => "Shifting",
            Self::Zoom => "Zoom",
            Self::Meteor => "Meteor",
            Self::Twinkling => "Twinkling",
        }
    }

    const fn uses_color(self) -> bool {
        matches!(self, Self::Direct | Self::Static | Self::Breathing)
    }

    const fn uses_speed(self) -> bool {
        !matches!(self, Self::Direct | Self::Static)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    const fn protocol(self) -> u8 {
        match self {
            Self::Left => 1,
            Self::Right => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidSettings {
    UnsupportedMode {
        device: AcerDevice,
        mode: AcerMode,
    },
    Brightness(u8),
    Speed {
        device: AcerDevice,
        speed: u8,
    },
    ColorCount {
        mode: AcerMode,
        expected: usize,
        actual: usize,
    },
}

impl fmt::Display for InvalidSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedMode { device, mode } => {
                write!(f, "{} does not support {}", device.name(), mode.name())
            }
            Self::Brightness(value) => write!(f, "brightness must be 0..=100, got {value}"),
            Self::Speed { device, speed } => write!(
                f,
                "{} effect speed must be 1..={}, got {speed}",
                device.name(),
                device.max_speed()
            ),
            Self::ColorCount {
                mode,
                expected,
                actual,
            } => write!(
                f,
                "{} requires exactly {expected} colors, got {actual}",
                mode.name()
            ),
        }
    }
}

impl std::error::Error for InvalidSettings {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction(Vec<OutputReport<FEATURE_REPORT_LEN>>);

impl ModeTransaction {
    /// Builds the complete native per-zone or all-zone update.
    ///
    /// # Errors
    /// Rejects modes, ranges, and color counts not exposed by the selected profile.
    pub fn new(
        device: AcerDevice,
        mode: AcerMode,
        brightness: u8,
        speed: u8,
        direction: Direction,
        colors: &[Rgb8],
    ) -> Result<Self, InvalidSettings> {
        if !device.supports(mode) {
            return Err(InvalidSettings::UnsupportedMode { device, mode });
        }
        if brightness > 100 {
            return Err(InvalidSettings::Brightness(brightness));
        }
        if mode.uses_speed() && !(1..=device.max_speed()).contains(&speed) {
            return Err(InvalidSettings::Speed { device, speed });
        }
        let expected = if mode == AcerMode::Direct {
            device.led_count()
        } else {
            usize::from(mode.uses_color())
        };
        if colors.len() != expected {
            return Err(InvalidSettings::ColorCount {
                mode,
                expected,
                actual: colors.len(),
            });
        }

        if mode == AcerMode::Direct {
            return Ok(Self(
                colors
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(index, color)| {
                        OutputReport::from_array([
                            0xA4,
                            device.device_id(),
                            AcerMode::Static as u8,
                            brightness,
                            0,
                            1,
                            color.r,
                            color.g,
                            color.b,
                            1 << index,
                            0,
                        ])
                    })
                    .collect(),
            ));
        }

        let color = colors.first().copied().unwrap_or(Rgb8::BLACK);
        Ok(Self(vec![OutputReport::from_array([
            0xA4,
            device.device_id(),
            mode as u8,
            brightness,
            if mode.uses_speed() { speed } else { 0 },
            if mode == AcerMode::Wave {
                direction.protocol()
            } else {
                1
            },
            color.r,
            color.g,
            color.b,
            0x0F,
            0,
        ])]))
    }

    #[must_use]
    pub fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>] {
        &self.0
    }

    /// Sends every feature report in native order.
    ///
    /// # Errors
    /// Returns the first feature transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        for report in &self.0 {
            send_feature(writer, report)?;
        }
        Ok(())
    }
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description(device: AcerDevice) -> ControllerDescription {
    let brightness = Some(BrightnessRange {
        min: 0,
        max: 100,
        current: 100,
    });
    let speed = Some(SpeedRange {
        min: 1,
        max: u32::from(device.max_speed()),
        current: 4,
    });
    let modes = [
        AcerMode::Direct,
        AcerMode::Static,
        AcerMode::Breathing,
        AcerMode::Neon,
        AcerMode::Wave,
        AcerMode::Shifting,
        AcerMode::Zoom,
        AcerMode::Meteor,
        AcerMode::Twinkling,
    ];
    ControllerDescription {
        name: device.name().into(),
        vendor: "Acer".into(),
        description: match device {
            AcerDevice::Keyboard => "Acer Nitro HID Keyboard Device".into(),
            AcerDevice::ChassisLed => "Acer Nitro HID LED Device".into(),
        },
        device_type: match device {
            AcerDevice::Keyboard => DeviceType::Laptop,
            AcerDevice::ChassisLed => DeviceType::Light,
        },
        modes: modes
            .into_iter()
            .filter(|mode| device.supports(*mode))
            .map(|mode| ModeDescription {
                name: mode.name().into(),
                value: mode as u32,
                color_mode: match mode {
                    AcerMode::Direct => ModeColorMode::PerLed,
                    AcerMode::Static | AcerMode::Breathing => ModeColorMode::ModeSpecific,
                    AcerMode::Neon
                    | AcerMode::Wave
                    | AcerMode::Shifting
                    | AcerMode::Zoom
                    | AcerMode::Meteor
                    | AcerMode::Twinkling => ModeColorMode::None,
                },
                speed: mode.uses_speed().then_some(speed).flatten(),
                brightness,
            })
            .collect(),
        zone_names: vec![match device {
            AcerDevice::Keyboard => "Keyboard Backlight Zone".into(),
            AcerDevice::ChassisLed => "LED".into(),
        }],
        led_names: match device {
            AcerDevice::Keyboard => (1..=4)
                .map(|index| format!("Keyboard Backlight Zone {index}"))
                .collect(),
            AcerDevice::ChassisLed => vec!["LED".into()],
        },
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::EFFECTS)
            .union(ControllerCapabilities::BRIGHTNESS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::sync::Arc;

    #[derive(Debug, Default)]
    struct Writer(Vec<[u8; FEATURE_REPORT_LEN]>);
    impl FeatureWriter<FEATURE_REPORT_LEN> for Writer {
        type Error = io::Error;
        fn send_feature_report(
            &mut self,
            report: &OutputReport<FEATURE_REPORT_LEN>,
        ) -> Result<(), Self::Error> {
            self.0.push(*report.as_bytes());
            Ok(())
        }
    }

    fn endpoint(page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"acer-test"[..]),
            0x0CF2,
            0x5130,
            9,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn endpoint_match_is_usage_exact_and_interface_flexible() {
        assert!(matches(&endpoint(0xFF5A, 1)));
        assert!(!matches(&endpoint(0xFF5A, 2)));
        assert!(!matches(&endpoint(0xFF59, 1)));
    }

    #[test]
    fn keyboard_direct_uses_four_static_zone_packets() {
        let colors = [
            Rgb8::new(1, 2, 3),
            Rgb8::new(4, 5, 6),
            Rgb8::new(7, 8, 9),
            Rgb8::new(10, 11, 12),
        ];
        let tx = ModeTransaction::new(
            AcerDevice::Keyboard,
            AcerMode::Direct,
            80,
            0,
            Direction::Left,
            &colors,
        )
        .unwrap();
        assert_eq!(tx.reports().len(), 4);
        assert_eq!(
            tx.reports()[0].as_bytes(),
            &[0xA4, 0x21, 2, 80, 0, 1, 1, 2, 3, 1, 0]
        );
        assert_eq!(
            tx.reports()[3].as_bytes(),
            &[0xA4, 0x21, 2, 80, 0, 1, 10, 11, 12, 8, 0]
        );
    }

    #[test]
    fn hardware_modes_use_all_zones_and_profile_device_ids() {
        let wave = ModeTransaction::new(
            AcerDevice::Keyboard,
            AcerMode::Wave,
            100,
            9,
            Direction::Right,
            &[],
        )
        .unwrap();
        assert_eq!(
            wave.reports()[0].as_bytes(),
            &[0xA4, 0x21, 7, 100, 9, 2, 0, 0, 0, 0x0F, 0]
        );
        let breathing = ModeTransaction::new(
            AcerDevice::ChassisLed,
            AcerMode::Breathing,
            50,
            5,
            Direction::Left,
            &[Rgb8::new(1, 2, 3)],
        )
        .unwrap();
        assert_eq!(
            breathing.reports()[0].as_bytes(),
            &[0xA4, 0x65, 4, 50, 5, 1, 1, 2, 3, 0x0F, 0]
        );
    }

    #[test]
    fn profile_ranges_modes_descriptions_and_transport_are_checked() {
        assert!(
            ModeTransaction::new(
                AcerDevice::ChassisLed,
                AcerMode::Wave,
                100,
                5,
                Direction::Left,
                &[]
            )
            .is_err()
        );
        assert!(
            ModeTransaction::new(
                AcerDevice::ChassisLed,
                AcerMode::Neon,
                100,
                6,
                Direction::Left,
                &[]
            )
            .is_err()
        );
        assert_eq!(description(AcerDevice::Keyboard).modes.len(), 9);
        assert_eq!(description(AcerDevice::ChassisLed).modes.len(), 4);
        let keyboard = description(AcerDevice::Keyboard);
        assert_eq!(keyboard.description, "Acer Nitro HID Keyboard Device");
        assert_eq!(keyboard.modes[0].color_mode, ModeColorMode::PerLed);
        assert_eq!(keyboard.modes[1].color_mode, ModeColorMode::ModeSpecific);
        assert_eq!(keyboard.modes[2].color_mode, ModeColorMode::ModeSpecific);
        assert_eq!(keyboard.modes[3].color_mode, ModeColorMode::None);
        assert_eq!(
            description(AcerDevice::ChassisLed).description,
            "Acer Nitro HID LED Device"
        );
        let tx = ModeTransaction::new(
            AcerDevice::ChassisLed,
            AcerMode::Direct,
            100,
            0,
            Direction::Left,
            &[Rgb8::BLACK],
        )
        .unwrap();
        let mut writer = Writer::default();
        tx.apply(&mut writer).unwrap();
        assert_eq!(writer.0, [*tx.reports()[0].as_bytes()]);
    }
}
