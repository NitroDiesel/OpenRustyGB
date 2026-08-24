#![forbid(unsafe_code)]

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const FEATURE_REPORT_LEN: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RedragonModel {
    pub name: &'static str,
    pub matcher: HidDeviceMatch,
}

const fn model(name: &'static str, product_id: u16) -> RedragonModel {
    RedragonModel {
        name,
        matcher: HidDeviceMatch {
            vendor_id: 0x04D9,
            product_id,
            interface_number: Some(2),
            usage_page: Some(0xFFA0),
            usage: None,
        },
    }
}

pub const MODELS: [RedragonModel; 11] = [
    model("Redragon M711 Cobra", 0xFC30),
    model("Redragon M715 Dagger", 0xFC39),
    model("Redragon M716 Inquisitor", 0xFC3A),
    model("Redragon M908 Impact", 0xFC4D),
    model("Redragon M602 Griffin", 0xFC38),
    model("Redragon M808 Storm", 0xFC5F),
    model("Redragon M801 Sniper", 0xFC58),
    model("Redragon M810 Taipan", 0xFA7E),
    model("Redragon M987 Reaping", 0xFC69),
    model("Redragon M921 Azzinoth", 0xFC40),
    model("Redragon M711-FPS-1 Cobra FPS", 0xFC62),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum RedragonMode {
    Wave = 0x00,
    RandomBreathing = 0x01,
    Static = 0x02,
    Breathing = 0x04,
    Rainbow = 0x08,
    Flashing = 0x10,
}

const APPLY_REPORT: OutputReport<FEATURE_REPORT_LEN> =
    OutputReport::from_array([0x02, 0xF1, 0x02, 0x04, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

const fn write_report(address: u16, data: &[u8]) -> OutputReport<FEATURE_REPORT_LEN> {
    assert!(
        data.len() <= 8,
        "Redragon write payload exceeds report capacity"
    );
    let mut report = [0; FEATURE_REPORT_LEN];
    let address = address.to_le_bytes();
    report[0] = 0x02;
    report[1] = 0xF3;
    report[2] = address[0];
    report[3] = address[1];
    report[4] = match data.len() {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
        _ => unreachable!(),
    };
    let mut index = 0;
    while index < data.len() {
        report[8 + index] = data[index];
        index += 1;
    }
    OutputReport::from_array(report)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Initialization {
    reports: [OutputReport<FEATURE_REPORT_LEN>; 2],
}

impl Default for Initialization {
    fn default() -> Self {
        Self::new()
    }
}

impl Initialization {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            reports: [write_report(0x002C, &[0]), APPLY_REPORT],
        }
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>; 2] {
        &self.reports
    }

    /// Selects profile zero and applies it, matching the native open lifecycle.
    ///
    /// # Errors
    ///
    /// Stops on the first feature-report transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        apply_reports(writer, &self.reports)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColorTransaction {
    reports: [OutputReport<FEATURE_REPORT_LEN>; 2],
}

impl ColorTransaction {
    #[must_use]
    pub const fn new(color: Rgb8) -> Self {
        Self {
            reports: [
                write_report(0x0449, &[color.r, color.g, color.b]),
                APPLY_REPORT,
            ],
        }
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>; 2] {
        &self.reports
    }

    /// Writes the current color and applies it.
    ///
    /// # Errors
    ///
    /// Stops on the first feature-report transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        apply_reports(writer, &self.reports)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction {
    reports: [OutputReport<FEATURE_REPORT_LEN>; 2],
}

impl ModeTransaction {
    #[must_use]
    pub const fn new(mode: RedragonMode, color: Rgb8) -> Self {
        Self {
            reports: [
                write_report(0x0449, &[color.r, color.g, color.b, 0x01, 0x00, mode as u8]),
                APPLY_REPORT,
            ],
        }
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>; 2] {
        &self.reports
    }

    /// Writes one native hardware effect and applies it.
    ///
    /// # Errors
    ///
    /// Stops on the first feature-report transport error.
    pub fn apply<W: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), W::Error> {
        apply_reports(writer, &self.reports)
    }
}

fn apply_reports<W: FeatureWriter<FEATURE_REPORT_LEN>>(
    writer: &mut W,
    reports: &[OutputReport<FEATURE_REPORT_LEN>],
) -> Result<(), W::Error> {
    for report in reports {
        send_feature(writer, report)?;
    }
    Ok(())
}

#[must_use]
pub fn match_model(endpoint: &HidEndpointInfo) -> Option<&'static RedragonModel> {
    MODELS.iter().find(|model| model.matcher.matches(endpoint))
}

fn mode(name: &str, value: RedragonMode, color_mode: ModeColorMode) -> ModeDescription {
    ModeDescription {
        name: name.into(),
        value: value as u32,
        color_mode,
        speed: None,
        brightness: None,
    }
}

#[must_use]
pub fn description(device_name: &str) -> ControllerDescription {
    ControllerDescription {
        name: device_name.into(),
        vendor: "Redragon".into(),
        description: "Redragon Mouse Device".into(),
        device_type: DeviceType::Mouse,
        modes: vec![
            mode("Static", RedragonMode::Static, ModeColorMode::PerLed),
            mode("Wave", RedragonMode::Wave, ModeColorMode::PerLed),
            mode("Breathing", RedragonMode::Breathing, ModeColorMode::PerLed),
            mode("Rainbow", RedragonMode::Rainbow, ModeColorMode::None),
            mode("Flashing", RedragonMode::Flashing, ModeColorMode::PerLed),
        ],
        zone_names: vec!["Mouse".into()],
        led_names: vec!["Mouse".into()],
        capabilities: ControllerCapabilities::PER_LED_COLOR.union(ControllerCapabilities::EFFECTS),
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::sync::Arc;

    use super::*;

    #[derive(Debug, Default)]
    struct RecordingWriter(Vec<[u8; FEATURE_REPORT_LEN]>);

    impl FeatureWriter<FEATURE_REPORT_LEN> for RecordingWriter {
        type Error = Infallible;

        fn send_feature_report(
            &mut self,
            report: &OutputReport<FEATURE_REPORT_LEN>,
        ) -> Result<(), Self::Error> {
            self.0.push(*report.as_bytes());
            Ok(())
        }
    }

    fn endpoint(product: u16, interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"redragon-test"[..]),
            0x04D9,
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
    fn all_eleven_models_require_interface_two_and_vendor_usage_page() {
        for model in MODELS {
            assert_eq!(
                match_model(&endpoint(model.matcher.product_id, 2, 0xFFA0, 7)),
                Some(&model)
            );
        }
        assert!(match_model(&endpoint(0xFC30, 1, 0xFFA0, 7)).is_none());
        assert!(match_model(&endpoint(0xFC30, 2, 0xFF00, 7)).is_none());
    }

    #[test]
    fn initialization_preserves_profile_selection_and_apply() {
        let mut writer = RecordingWriter::default();
        Initialization::new().apply(&mut writer).unwrap();
        assert_eq!(
            writer.0,
            [
                [2, 0xF3, 0x2C, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [2, 0xF1, 2, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            ]
        );
    }

    #[test]
    fn mode_and_color_transactions_preserve_native_packets() {
        let mut writer = RecordingWriter::default();
        ModeTransaction::new(RedragonMode::Breathing, Rgb8::new(0x11, 0x22, 0x33))
            .apply(&mut writer)
            .unwrap();
        assert_eq!(
            writer.0[0],
            [
                2, 0xF3, 0x49, 4, 6, 0, 0, 0, 0x11, 0x22, 0x33, 1, 0, 4, 0, 0
            ]
        );
        assert_eq!(writer.0[1], *APPLY_REPORT.as_bytes());

        let color = ColorTransaction::new(Rgb8::new(1, 2, 3));
        assert_eq!(
            color.reports()[0].as_bytes(),
            &[2, 0xF3, 0x49, 4, 3, 0, 0, 0, 1, 2, 3, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn controller_shape_and_effect_values_are_preserved() {
        let device = description("Redragon M711 Cobra");
        assert_eq!(device.modes.len(), 5);
        assert_eq!(device.zone_names, ["Mouse"]);
        assert_eq!(device.led_names, ["Mouse"]);
        assert_eq!(device.modes[0].value, 2);
        assert_eq!(device.modes[3].color_mode, ModeColorMode::None);
        assert!(
            device
                .capabilities
                .contains(ControllerCapabilities::EFFECTS)
        );
    }
}
