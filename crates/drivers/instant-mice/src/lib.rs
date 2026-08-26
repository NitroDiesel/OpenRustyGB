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
pub const COLOR_REPORT_COUNT: usize = 6;

const fn matcher(product_id: u16) -> HidDeviceMatch {
    HidDeviceMatch {
        vendor_id: 0x30FA,
        product_id,
        interface_number: Some(1),
        usage_page: Some(0xFF01),
        usage: Some(0x0001),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InstantMouseModel {
    pub name: &'static str,
    pub matcher: HidDeviceMatch,
    pub ant_effects: bool,
}

pub const MODELS: [InstantMouseModel; 4] = [
    InstantMouseModel {
        name: "Advanced GTA 250 USB Gaming Mouse",
        matcher: matcher(0x1030),
        ant_effects: false,
    },
    InstantMouseModel {
        name: "Anko KM43243952 USB Gaming Mouse",
        matcher: matcher(0x1440),
        ant_effects: false,
    },
    InstantMouseModel {
        name: "Anko KM43277483 USB Gaming Mouse",
        matcher: matcher(0x1540),
        ant_effects: false,
    },
    InstantMouseModel {
        name: "AntEsports GM600 USB Gaming Mouse",
        matcher: matcher(0x1040),
        ant_effects: true,
    },
];

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstantMode {
    Direct = 0x0A,
    Off = 0xFF,
    Fill = 0x03,
    Loop = 0x04,
    SpectrumCycle = 0x06,
    RainbowWave = 0x07,
    Breathing = 0x08,
    AntBreathing = 0x09,
    Enraptured = 0xBB,
    Flicker = 0xB8,
    Ripple = 0xBA,
    StarTreck = 0xB9,
}

impl InstantMode {
    const fn uses_speed(self) -> bool {
        !matches!(self, Self::Direct | Self::Off | Self::StarTreck)
    }

    const fn uses_brightness(self) -> bool {
        matches!(self, Self::Direct | Self::Fill | Self::Loop)
    }

    const fn uses_direction(self) -> bool {
        matches!(
            self,
            Self::RainbowWave | Self::Fill | Self::Loop | Self::Ripple
        )
    }

    const fn uses_color(self) -> bool {
        matches!(self, Self::Direct | Self::Breathing | Self::AntBreathing)
    }

    const fn ant_only(self) -> bool {
        matches!(
            self,
            Self::AntBreathing | Self::Enraptured | Self::Flicker | Self::Ripple | Self::StarTreck
        )
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Right = 0,
    Left = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidSettings {
    UnsupportedMode(InstantMode),
    Speed(u8),
    Brightness(u8),
}

impl fmt::Display for InvalidSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedMode(mode) => {
                write!(f, "mode {mode:?} is not supported by this Instant mouse")
            }
            Self::Speed(value) => write!(f, "Instant mouse speed must be in 0..=5, got {value}"),
            Self::Brightness(value) => {
                write!(f, "Instant mouse brightness must be in 0..=7, got {value}")
            }
        }
    }
}

impl std::error::Error for InvalidSettings {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColorTransaction([OutputReport<FEATURE_REPORT_LEN>; COLOR_REPORT_COUNT]);

impl ColorTransaction {
    #[must_use]
    pub fn new(color: Rgb8) -> Self {
        Self([
            color_report(color, 0),
            color_report(color, 2),
            color_report(color, 4),
            color_report(color, 6),
            color_report(color, 8),
            color_report(color, 10),
        ])
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>; COLOR_REPORT_COUNT] {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction {
    reports: Vec<OutputReport<FEATURE_REPORT_LEN>>,
}

impl ModeTransaction {
    /// Builds the complete upstream transaction for a displayed mode.
    ///
    /// # Errors
    /// Returns an error for model-specific modes or settings outside the
    /// ranges exposed by `OpenRGB`.
    pub fn new(
        model: InstantMouseModel,
        mode: InstantMode,
        color: Rgb8,
        speed: u8,
        brightness: u8,
        direction: Direction,
    ) -> Result<Self, InvalidSettings> {
        if mode.ant_only() != model.ant_effects && mode.ant_only() {
            return Err(InvalidSettings::UnsupportedMode(mode));
        }
        if model.ant_effects && mode == InstantMode::Breathing {
            return Err(InvalidSettings::UnsupportedMode(mode));
        }
        if !model.ant_effects && mode == InstantMode::AntBreathing {
            return Err(InvalidSettings::UnsupportedMode(mode));
        }
        if mode.uses_speed() && speed > 5 {
            return Err(InvalidSettings::Speed(speed));
        }
        if mode.uses_brightness() && brightness > 7 {
            return Err(InvalidSettings::Brightness(brightness));
        }

        if mode == InstantMode::Off {
            let mut reports = vec![mode_report(InstantMode::Direct, 0, 0, Direction::Right)];
            reports.extend(ColorTransaction::new(Rgb8::BLACK).0);
            return Ok(Self { reports });
        }

        let speed = if mode.uses_speed() { speed } else { 0 };
        let brightness = if mode.uses_brightness() {
            brightness
        } else {
            0
        };
        let direction = if mode.uses_direction() {
            direction
        } else {
            Direction::Right
        };
        let mut reports = vec![mode_report(mode, speed, brightness, direction)];
        if mode.uses_color() {
            reports.extend(ColorTransaction::new(color).0);
        }
        Ok(Self { reports })
    }

    #[must_use]
    pub fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>] {
        &self.reports
    }

    /// Sends every native feature report in transaction order.
    ///
    /// # Errors
    /// Stops on the first HID feature-report transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        for report in &self.reports {
            send_feature(writer, report)?;
        }
        Ok(())
    }
}

fn color_report(color: Rgb8, slot: u8) -> OutputReport<FEATURE_REPORT_LEN> {
    let red = 0x0F - color.r / 16;
    let green = 0x0F - color.g / 16;
    let blue = 0x0F - color.b / 16;
    OutputReport::from_array([
        0x07,
        0x14,
        (slot << 4) | green,
        (red << 4) | blue,
        0,
        0,
        0,
        0,
    ])
}

fn mode_report(
    mode: InstantMode,
    speed: u8,
    brightness: u8,
    direction: Direction,
) -> OutputReport<FEATURE_REPORT_LEN> {
    OutputReport::from_array([
        0x07,
        0x13,
        0xFF,
        ((mode as u8) << 4) | (speed & 0x0F),
        ((direction as u8) << 4) | 0x0B,
        0x0F - (brightness & 0x0F),
        (mode as u8) & 0xF0,
        0,
    ])
}

#[must_use]
pub fn match_model(endpoint: &HidEndpointInfo) -> Option<InstantMouseModel> {
    MODELS
        .iter()
        .copied()
        .find(|model| model.matcher.matches(endpoint))
}

fn mode_description(
    name: &str,
    mode: InstantMode,
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
pub fn description(model: InstantMouseModel) -> ControllerDescription {
    let speed = Some(SpeedRange {
        min: 0,
        max: 5,
        current: 2,
    });
    let brightness = Some(BrightnessRange {
        min: 0,
        max: 7,
        current: 7,
    });
    let mut controller_modes = common_modes(model, speed, brightness);
    if model.ant_effects {
        controller_modes.extend(ant_modes(speed));
    }
    controller_modes.push(mode_description(
        "Off",
        InstantMode::Off,
        ModeColorMode::None,
        None,
        None,
    ));
    ControllerDescription {
        name: model.name.into(),
        vendor: model.name.into(),
        description: "Instant USB Gaming Mouse".into(),
        device_type: DeviceType::Mouse,
        modes: controller_modes,
        zone_names: vec!["Mouse".into()],
        led_names: vec!["Mouse".into()],
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::EFFECTS)
            .union(ControllerCapabilities::BRIGHTNESS),
    }
}

fn common_modes(
    model: InstantMouseModel,
    speed: Option<SpeedRange>,
    brightness: Option<BrightnessRange>,
) -> Vec<ModeDescription> {
    let breathing = if model.ant_effects {
        InstantMode::AntBreathing
    } else {
        InstantMode::Breathing
    };
    vec![
        mode_description(
            "Direct",
            InstantMode::Direct,
            ModeColorMode::PerLed,
            None,
            brightness,
        ),
        mode_description(
            "Rainbow wave",
            InstantMode::RainbowWave,
            ModeColorMode::None,
            speed,
            None,
        ),
        mode_description(
            "Spectrum cycle",
            InstantMode::SpectrumCycle,
            ModeColorMode::None,
            speed,
            None,
        ),
        mode_description("Breathing", breathing, ModeColorMode::PerLed, speed, None),
        mode_description(
            "Fill",
            InstantMode::Fill,
            ModeColorMode::None,
            speed,
            brightness,
        ),
        mode_description(
            "Loop",
            InstantMode::Loop,
            ModeColorMode::None,
            speed,
            brightness,
        ),
    ]
}

fn ant_modes(speed: Option<SpeedRange>) -> [ModeDescription; 4] {
    [
        mode_description(
            "Enrpatured",
            InstantMode::Enraptured,
            ModeColorMode::None,
            speed,
            None,
        ),
        mode_description(
            "Flicker",
            InstantMode::Flicker,
            ModeColorMode::None,
            speed,
            None,
        ),
        mode_description(
            "Ripple",
            InstantMode::Ripple,
            ModeColorMode::None,
            speed,
            None,
        ),
        mode_description(
            "Star treck",
            InstantMode::StarTreck,
            ModeColorMode::None,
            None,
            None,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn endpoint(product_id: u16, interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"instant-test"[..]),
            0x30FA,
            product_id,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn all_four_models_match_only_the_exact_lighting_endpoint() {
        for model in MODELS {
            assert_eq!(
                match_model(&endpoint(model.matcher.product_id, 1, 0xFF01, 1)),
                Some(model)
            );
        }
        assert!(match_model(&endpoint(0x1030, 0, 0xFF01, 1)).is_none());
        assert!(match_model(&endpoint(0x1030, 1, 0xFF00, 1)).is_none());
        assert!(match_model(&endpoint(0x1030, 1, 0xFF01, 2)).is_none());
    }

    #[test]
    fn color_quantization_and_all_six_dpi_slots_are_exact() {
        let transaction = ColorTransaction::new(Rgb8::new(0, 128, 255));
        assert_eq!(
            transaction.reports()[0].as_bytes(),
            &[0x07, 0x14, 0x07, 0xF0, 0, 0, 0, 0]
        );
        assert_eq!(
            transaction.reports()[5].as_bytes(),
            &[0x07, 0x14, 0xA7, 0xF0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn direct_breathing_and_off_transactions_preserve_order() {
        let standard = MODELS[0];
        let direct = ModeTransaction::new(
            standard,
            InstantMode::Direct,
            Rgb8::new(1, 2, 3),
            5,
            7,
            Direction::Left,
        )
        .unwrap();
        assert_eq!(direct.reports().len(), 7);
        assert_eq!(
            direct.reports()[0].as_bytes(),
            &[7, 0x13, 0xFF, 0xA0, 0x0B, 8, 0, 0]
        );
        let breathing = ModeTransaction::new(
            standard,
            InstantMode::Breathing,
            Rgb8::BLACK,
            2,
            7,
            Direction::Left,
        )
        .unwrap();
        assert_eq!(breathing.reports().len(), 7);
        assert_eq!(
            &breathing.reports()[0].as_bytes()[3..6],
            &[0x82, 0x0B, 0x0F]
        );
        let off = ModeTransaction::new(
            standard,
            InstantMode::Off,
            Rgb8::new(1, 2, 3),
            5,
            7,
            Direction::Left,
        )
        .unwrap();
        assert_eq!(off.reports().len(), 7);
        assert_eq!(
            off.reports()[0].as_bytes(),
            &[7, 0x13, 0xFF, 0xA0, 0x0B, 0x0F, 0, 0]
        );
        assert_eq!(
            off.reports()[1].as_bytes(),
            &[7, 0x14, 0x0F, 0xFF, 0, 0, 0, 0]
        );
    }

    #[test]
    fn ant_modes_and_model_shapes_are_preserved() {
        let standard = description(MODELS[0]);
        let ant = description(MODELS[3]);
        assert_eq!(standard.modes.len(), 7);
        assert_eq!(ant.modes.len(), 11);
        assert_eq!(ant.modes[3].value, InstantMode::AntBreathing as u32);
        assert!(
            ModeTransaction::new(
                MODELS[0],
                InstantMode::Ripple,
                Rgb8::BLACK,
                2,
                0,
                Direction::Left,
            )
            .is_err()
        );
        assert!(
            ModeTransaction::new(
                MODELS[3],
                InstantMode::Breathing,
                Rgb8::BLACK,
                2,
                0,
                Direction::Right,
            )
            .is_err()
        );
    }
}
