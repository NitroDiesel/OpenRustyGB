#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    ExactWriteError, FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, OutputWriter,
    send_feature, write_exact,
};

pub const FEATURE_REPORT_LEN: usize = 9;
pub const OUTPUT_REPORT_LEN: usize = 65;
pub const KEYBOARD_LED_COUNT: usize = 4;
pub const FRONT_BAR_LED_COUNT: usize = 22;
pub const EFFECT_COLOR_COUNT: usize = 7;
pub const MAX_BRIGHTNESS: u8 = 50;
pub const MAX_SPEED: u8 = 10;
pub const DEFAULT_SPEED: u32 = 5;

const fn matcher(product_id: u16, usage_page: u16) -> HidDeviceMatch {
    HidDeviceMatch {
        vendor_id: 0x048D,
        product_id,
        interface_number: None,
        usage_page: Some(usage_page),
        usage: Some(0x0001),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IonicoModel {
    Keyboard,
    FrontBar,
}

impl IonicoModel {
    #[must_use]
    pub const fn matcher(self) -> HidDeviceMatch {
        match self {
            Self::Keyboard => matcher(0xCE00, 0xFF12),
            Self::FrontBar => matcher(0x6005, 0xFF03),
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Keyboard => "Ionico Keyboard",
            Self::FrontBar => "Ionico Light Bar",
        }
    }

    #[must_use]
    pub const fn led_count(self) -> usize {
        match self {
            Self::Keyboard => KEYBOARD_LED_COUNT,
            Self::FrontBar => FRONT_BAR_LED_COUNT,
        }
    }
}

pub const MODELS: [IonicoModel; 2] = [IonicoModel::Keyboard, IonicoModel::FrontBar];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IonicoMode {
    Direct,
    Breathing,
    Wave,
    Raindrops,
    Flashing,
    Off,
}

impl IonicoMode {
    const fn value(self, model: IonicoModel) -> Option<u8> {
        match (self, model) {
            (Self::Direct, _) => Some(1),
            (Self::Breathing, _) => Some(2),
            (Self::Wave, IonicoModel::Keyboard) => Some(3),
            (Self::Wave, IonicoModel::FrontBar) => Some(32),
            (Self::Raindrops, IonicoModel::FrontBar) => Some(10),
            (Self::Flashing, IonicoModel::Keyboard) => Some(18),
            (Self::Off, _) => Some(0),
            _ => None,
        }
    }

    const fn uses_effect_colors(self) -> bool {
        matches!(
            self,
            Self::Breathing | Self::Wave | Self::Raindrops | Self::Flashing
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidSettings {
    UnsupportedMode {
        model: IonicoModel,
        mode: IonicoMode,
    },
    ColorCount {
        expected: usize,
        actual: usize,
    },
    Brightness(u8),
    Speed(u8),
}

impl fmt::Display for InvalidSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedMode { model, mode } => {
                write!(f, "mode {mode:?} is not supported by {model:?}")
            }
            Self::ColorCount { expected, actual } => {
                write!(
                    f,
                    "Ionico transaction requires {expected} colors, got {actual}"
                )
            }
            Self::Brightness(value) => write!(
                f,
                "Ionico brightness must be between 0 and {MAX_BRIGHTNESS}, got {value}"
            ),
            Self::Speed(value) => {
                write!(
                    f,
                    "Ionico speed must be between 0 and {MAX_SPEED}, got {value}"
                )
            }
        }
    }
}

impl std::error::Error for InvalidSettings {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IonicoReport {
    Feature(OutputReport<FEATURE_REPORT_LEN>),
    Output(OutputReport<OUTPUT_REPORT_LEN>),
}

#[derive(Debug)]
pub enum ApplyError<E> {
    Feature(E),
    Output(ExactWriteError<E>),
}

impl<E: fmt::Display> fmt::Display for ApplyError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Feature(error) => write!(f, "feature transport failed: {error}"),
            Self::Output(error) => write!(f, "{error}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for ApplyError<E> {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction(Vec<IonicoReport>);

impl ModeTransaction {
    /// Builds the complete mode-and-color transaction used by the native controller.
    ///
    /// # Errors
    /// Returns an error for model-specific modes, invalid ranges, or incomplete colors.
    pub fn new(
        model: IonicoModel,
        mode: IonicoMode,
        colors: &[Rgb8],
        brightness: u8,
        speed: u8,
    ) -> Result<Self, InvalidSettings> {
        let Some(mode_value) = mode.value(model) else {
            return Err(InvalidSettings::UnsupportedMode { model, mode });
        };
        if mode != IonicoMode::Off && brightness > MAX_BRIGHTNESS {
            return Err(InvalidSettings::Brightness(brightness));
        }
        if mode.uses_effect_colors() && speed > MAX_SPEED {
            return Err(InvalidSettings::Speed(speed));
        }

        let expected_colors = if mode == IonicoMode::Direct {
            model.led_count()
        } else if mode.uses_effect_colors() {
            EFFECT_COLOR_COUNT
        } else {
            0
        };
        if colors.len() != expected_colors {
            return Err(InvalidSettings::ColorCount {
                expected: expected_colors,
                actual: colors.len(),
            });
        }

        if mode == IonicoMode::Off {
            return Ok(Self(vec![IonicoReport::Feature(feature_report([
                0, 0x09, 0x02, 0, 0, 0, 0, 0, 0,
            ]))]));
        }

        let device_mode = if mode == IonicoMode::Direct && model == IonicoModel::FrontBar {
            0x33
        } else {
            mode_value
        };
        let mode_speed = if mode.uses_effect_colors() { speed } else { 0 };
        let mut reports = vec![IonicoReport::Feature(mode_report(
            device_mode,
            brightness,
            mode_speed,
        ))];

        if mode == IonicoMode::Direct && model == IonicoModel::FrontBar {
            reports.extend(front_bar_direct_reports(colors));
        } else {
            reports.extend(indexed_color_reports(colors));
        }
        Ok(Self(reports))
    }

    #[must_use]
    pub fn reports(&self) -> &[IonicoReport] {
        &self.0
    }

    /// Sends feature and output reports in their native order.
    ///
    /// # Errors
    /// Returns the first feature transport, output transport, or short-write error.
    pub fn apply<W, E>(&self, writer: &mut W) -> Result<(), ApplyError<E>>
    where
        W: FeatureWriter<FEATURE_REPORT_LEN, Error = E>
            + OutputWriter<OUTPUT_REPORT_LEN, Error = E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        apply_reports(writer, &self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveTransaction(OutputReport<FEATURE_REPORT_LEN>);

impl SaveTransaction {
    #[must_use]
    pub const fn new() -> Self {
        Self(OutputReport::from_array([0, 0x1A, 0, 1, 4, 0, 0, 0, 1]))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<FEATURE_REPORT_LEN> {
        &self.0
    }

    /// Persists the current device state in BIOS.
    ///
    /// # Errors
    /// Returns the feature transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        send_feature(writer, &self.0)
    }
}

impl Default for SaveTransaction {
    fn default() -> Self {
        Self::new()
    }
}

const fn feature_report(bytes: [u8; FEATURE_REPORT_LEN]) -> OutputReport<FEATURE_REPORT_LEN> {
    OutputReport::from_array(bytes)
}

const fn mode_report(mode: u8, brightness: u8, speed: u8) -> OutputReport<FEATURE_REPORT_LEN> {
    feature_report([0, 0x08, 0x02, mode, speed, brightness, 0x08, 0, 0])
}

fn indexed_color_reports(colors: &[Rgb8]) -> Vec<IonicoReport> {
    colors
        .iter()
        .enumerate()
        .map(|(index, color)| {
            IonicoReport::Feature(feature_report([
                0,
                0x14,
                0,
                index.to_le_bytes()[0].wrapping_add(1),
                color.r,
                color.g,
                color.b,
                0,
                0,
            ]))
        })
        .collect()
}

fn front_bar_direct_reports(colors: &[Rgb8]) -> [IonicoReport; 3] {
    let start = IonicoReport::Feature(feature_report([0, 0x12, 0, 0, 0, 0, 0, 0, 0]));
    let mut bytes = [0; OUTPUT_REPORT_LEN];
    for (index, color) in colors.iter().enumerate() {
        let offset = 1 + index * 3;
        bytes[offset] = color.r;
        if offset + 1 < OUTPUT_REPORT_LEN {
            bytes[offset + 1] = color.b;
        }
        if offset + 2 < OUTPUT_REPORT_LEN {
            bytes[offset + 2] = color.g;
        }
    }
    let output = IonicoReport::Output(OutputReport::from_array(bytes));
    let finish = IonicoReport::Feature(feature_report([0, 0x12, 0, 1, 0, 0, 0, 0, 0]));
    [start, output, finish]
}

fn apply_reports<W, E>(writer: &mut W, reports: &[IonicoReport]) -> Result<(), ApplyError<E>>
where
    W: FeatureWriter<FEATURE_REPORT_LEN, Error = E> + OutputWriter<OUTPUT_REPORT_LEN, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    for report in reports {
        match report {
            IonicoReport::Feature(report) => {
                send_feature(writer, report).map_err(ApplyError::Feature)?;
            }
            IonicoReport::Output(report) => {
                write_exact(writer, report).map_err(ApplyError::Output)?;
            }
        }
    }
    Ok(())
}

#[must_use]
pub fn match_model(endpoint: &HidEndpointInfo) -> Option<IonicoModel> {
    MODELS
        .iter()
        .copied()
        .find(|model| model.matcher().matches(endpoint))
}

#[must_use]
pub fn description(model: IonicoModel) -> ControllerDescription {
    let speed = Some(SpeedRange {
        min: 0,
        max: u32::from(MAX_SPEED),
        current: DEFAULT_SPEED,
    });
    let brightness = Some(BrightnessRange {
        min: 0,
        max: MAX_BRIGHTNESS,
        current: MAX_BRIGHTNESS,
    });
    let wave_value = match model {
        IonicoModel::Keyboard => 3,
        IonicoModel::FrontBar => 32,
    };
    let mut mode_descriptions = vec![
        mode_description("Direct", 1, ModeColorMode::PerLed, None, brightness),
        mode_description("Breathing", 2, ModeColorMode::PerLed, speed, brightness),
        mode_description("Wave", wave_value, ModeColorMode::PerLed, speed, brightness),
    ];
    match model {
        IonicoModel::Keyboard => mode_descriptions.push(mode_description(
            "Flashing",
            18,
            ModeColorMode::PerLed,
            speed,
            brightness,
        )),
        IonicoModel::FrontBar => mode_descriptions.push(mode_description(
            "Raindrops",
            10,
            ModeColorMode::PerLed,
            speed,
            brightness,
        )),
    }
    mode_descriptions.push(mode_description("Off", 0, ModeColorMode::None, None, None));

    let (device_type, zone, prefix) = match model {
        IonicoModel::Keyboard => (DeviceType::Keyboard, "Keyboard", "Keyboard Zone"),
        IonicoModel::FrontBar => (DeviceType::LedStrip, "Front Bar", "Bar Led"),
    };
    ControllerDescription {
        name: model.name().into(),
        vendor: "Pcspecialist".into(),
        description: model.name().into(),
        device_type,
        modes: mode_descriptions,
        zone_names: vec![zone.into()],
        led_names: (1..=model.led_count())
            .map(|index| format!("{prefix} {index}"))
            .collect(),
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::EFFECTS)
            .union(ControllerCapabilities::BRIGHTNESS),
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::sync::Arc;

    #[derive(Debug, Default)]
    struct FakeTransport {
        features: Vec<Vec<u8>>,
        outputs: Vec<Vec<u8>>,
        short_output: bool,
    }

    impl FeatureWriter<FEATURE_REPORT_LEN> for FakeTransport {
        type Error = io::Error;

        fn send_feature_report(
            &mut self,
            report: &OutputReport<FEATURE_REPORT_LEN>,
        ) -> Result<(), Self::Error> {
            self.features.push(report.as_bytes().to_vec());
            Ok(())
        }
    }

    impl OutputWriter<OUTPUT_REPORT_LEN> for FakeTransport {
        type Error = io::Error;

        fn write_output(
            &mut self,
            report: &OutputReport<OUTPUT_REPORT_LEN>,
        ) -> Result<usize, Self::Error> {
            self.outputs.push(report.as_bytes().to_vec());
            Ok(if self.short_output {
                OUTPUT_REPORT_LEN - 1
            } else {
                OUTPUT_REPORT_LEN
            })
        }
    }

    fn endpoint(product: u16, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"ionico-test"[..]),
            0x048D,
            product,
            4,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn matchers_preserve_page_and_usage_without_interface_constraint() {
        assert_eq!(
            match_model(&endpoint(0xCE00, 0xFF12, 1)),
            Some(IonicoModel::Keyboard)
        );
        assert_eq!(
            match_model(&endpoint(0x6005, 0xFF03, 1)),
            Some(IonicoModel::FrontBar)
        );
        assert!(match_model(&endpoint(0xCE00, 0xFF03, 1)).is_none());
        assert!(match_model(&endpoint(0xCE00, 0xFF12, 2)).is_none());
    }

    #[test]
    fn keyboard_direct_and_effect_reports_are_preserved() {
        let direct = ModeTransaction::new(
            IonicoModel::Keyboard,
            IonicoMode::Direct,
            &[Rgb8::new(1, 2, 3); KEYBOARD_LED_COUNT],
            50,
            9,
        )
        .unwrap();
        assert_eq!(direct.reports().len(), 5);
        assert_eq!(
            direct.reports()[0],
            IonicoReport::Feature(mode_report(1, 50, 0))
        );
        assert_eq!(
            direct.reports()[1],
            IonicoReport::Feature(feature_report([0, 0x14, 0, 1, 1, 2, 3, 0, 0]))
        );

        let effect = ModeTransaction::new(
            IonicoModel::Keyboard,
            IonicoMode::Flashing,
            &[Rgb8::new(4, 5, 6); EFFECT_COLOR_COUNT],
            40,
            7,
        )
        .unwrap();
        assert_eq!(effect.reports().len(), 8);
        assert_eq!(
            effect.reports()[0],
            IonicoReport::Feature(mode_report(18, 40, 7))
        );
    }

    #[test]
    fn front_bar_direct_preserves_rbg_wire_order_and_safe_final_led() {
        let mut colors = [Rgb8::BLACK; FRONT_BAR_LED_COUNT];
        colors[0] = Rgb8::new(1, 2, 3);
        colors[20] = Rgb8::new(4, 5, 6);
        colors[21] = Rgb8::new(7, 8, 9);
        let transaction =
            ModeTransaction::new(IonicoModel::FrontBar, IonicoMode::Direct, &colors, 50, 0)
                .unwrap();
        assert_eq!(transaction.reports().len(), 4);
        assert_eq!(
            transaction.reports()[0],
            IonicoReport::Feature(mode_report(0x33, 50, 0))
        );
        let IonicoReport::Output(output) = &transaction.reports()[2] else {
            panic!("third report must be the front-bar output")
        };
        assert_eq!(&output.as_bytes()[1..4], &[1, 3, 2]);
        assert_eq!(&output.as_bytes()[61..64], &[4, 6, 5]);
        assert_eq!(output.as_bytes()[64], 7);
    }

    #[test]
    fn mode_validation_off_save_and_short_output_are_preserved() {
        assert!(matches!(
            ModeTransaction::new(IonicoModel::Keyboard, IonicoMode::Raindrops, &[], 0, 0),
            Err(InvalidSettings::UnsupportedMode { .. })
        ));
        assert!(matches!(
            ModeTransaction::new(IonicoModel::Keyboard, IonicoMode::Direct, &[], 50, 0),
            Err(InvalidSettings::ColorCount {
                expected: 4,
                actual: 0
            })
        ));
        let off =
            ModeTransaction::new(IonicoModel::FrontBar, IonicoMode::Off, &[], 255, 255).unwrap();
        assert_eq!(
            off.reports()[0],
            IonicoReport::Feature(feature_report([0, 9, 2, 0, 0, 0, 0, 0, 0]))
        );
        assert_eq!(
            SaveTransaction::new().report().as_bytes(),
            &[0, 0x1A, 0, 1, 4, 0, 0, 0, 1]
        );

        let direct = ModeTransaction::new(
            IonicoModel::FrontBar,
            IonicoMode::Direct,
            &[Rgb8::BLACK; FRONT_BAR_LED_COUNT],
            50,
            0,
        )
        .unwrap();
        let mut transport = FakeTransport {
            short_output: true,
            ..FakeTransport::default()
        };
        assert!(direct.apply(&mut transport).is_err());
        assert_eq!(transport.features.len(), 2);
        assert_eq!(transport.outputs.len(), 1);
    }

    #[test]
    fn model_descriptions_preserve_modes_zones_and_led_counts() {
        let keyboard = description(IonicoModel::Keyboard);
        assert_eq!(
            keyboard
                .modes
                .iter()
                .map(|mode| mode.name.as_str())
                .collect::<Vec<_>>(),
            ["Direct", "Breathing", "Wave", "Flashing", "Off"]
        );
        assert_eq!(keyboard.led_names.len(), KEYBOARD_LED_COUNT);
        let bar = description(IonicoModel::FrontBar);
        assert_eq!(
            bar.modes
                .iter()
                .map(|mode| mode.name.as_str())
                .collect::<Vec<_>>(),
            ["Direct", "Breathing", "Wave", "Raindrops", "Off"]
        );
        assert_eq!(bar.led_names.len(), FRONT_BAR_LED_COUNT);
    }
}
