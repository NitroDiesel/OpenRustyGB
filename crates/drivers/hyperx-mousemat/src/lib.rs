#![forbid(unsafe_code)]

use std::fmt;
use std::time::Duration;

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const FEATURE_REPORT_LEN: usize = 65;
pub const PROTOCOL_COLOR_SLOTS: usize = 32;
pub const KEEPALIVE_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HyperXMousematModel {
    pub name: &'static str,
    pub matcher: HidDeviceMatch,
    pub first_zone_leds: usize,
    pub second_zone_leds: usize,
}

impl HyperXMousematModel {
    #[must_use]
    pub const fn led_count(self) -> usize {
        self.first_zone_leds + self.second_zone_leds
    }
}

const FURY_ULTRA: HyperXMousematModel = HyperXMousematModel {
    name: "HyperX Fury Ultra",
    matcher: HidDeviceMatch {
        vendor_id: 0x0951,
        product_id: 0x1705,
        interface_number: Some(0),
        usage_page: None,
        usage: None,
    },
    first_zone_leds: 15,
    second_zone_leds: 5,
};

const PULSEFIRE_MAT: HyperXMousematModel = HyperXMousematModel {
    name: "HyperX Pulsefire Mat",
    matcher: HidDeviceMatch {
        vendor_id: 0x03F0,
        product_id: 0x0F8D,
        interface_number: Some(1),
        usage_page: Some(0xFF90),
        usage: Some(0xFF00),
    },
    first_zone_leds: 15,
    second_zone_leds: 5,
};

#[cfg(target_os = "windows")]
const FURY_A_XL_MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x0951,
    product_id: 0x1741,
    interface_number: Some(1),
    usage_page: Some(0xFF90),
    usage: Some(0xFF00),
};

#[cfg(not(target_os = "windows"))]
const FURY_A_XL_MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x0951,
    product_id: 0x1741,
    interface_number: Some(0),
    usage_page: Some(0x000C),
    usage: Some(0x0001),
};

const FURY_A_XL: HyperXMousematModel = HyperXMousematModel {
    name: "HyperX Pulsefire Mat RGB Mouse Pad XL",
    matcher: FURY_A_XL_MATCH,
    first_zone_leds: 2,
    second_zone_leds: 0,
};

pub const MODELS: [HyperXMousematModel; 3] = [FURY_ULTRA, PULSEFIRE_MAT, FURY_A_XL];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidColorCount {
    pub expected: usize,
    pub actual: usize,
}

impl fmt::Display for InvalidColorCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HyperX mousemat requires exactly {} colors, got {}",
            self.expected, self.actual
        )
    }
}

impl std::error::Error for InvalidColorCount {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectColorTransaction([OutputReport<FEATURE_REPORT_LEN>; 3]);

impl DirectColorTransaction {
    /// Builds the profile-selection report and both 16-color protocol reports.
    ///
    /// The native implementation always reads 32 colors despite exposing only two
    /// or 20 LEDs. This serializer preserves the protocol shape and safely fills
    /// the unused slots with black.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidColorCount`] unless one color is supplied for every
    /// model LED.
    pub fn new(model: HyperXMousematModel, colors: &[Rgb8]) -> Result<Self, InvalidColorCount> {
        if colors.len() != model.led_count() {
            return Err(InvalidColorCount {
                expected: model.led_count(),
                actual: colors.len(),
            });
        }

        let mut select_profile = [0; FEATURE_REPORT_LEN];
        select_profile[1] = 0x04;
        select_profile[2] = 0xF2;
        select_profile[9] = 0x02;

        let first_colors = color_report(colors, 0);
        let second_colors = color_report(colors, 16);
        Ok(Self([
            OutputReport::from_array(select_profile),
            OutputReport::from_array(first_colors),
            OutputReport::from_array(second_colors),
        ]))
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<FEATURE_REPORT_LEN>; 3] {
        &self.0
    }

    /// Sends the full native three-report direct-color transaction.
    ///
    /// # Errors
    ///
    /// Returns the first HID feature-report error.
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

fn color_report(colors: &[Rgb8], start: usize) -> [u8; FEATURE_REPORT_LEN] {
    let mut report = [0; FEATURE_REPORT_LEN];
    for slot in 0..16 {
        let color = colors.get(start + slot).copied().unwrap_or(Rgb8::BLACK);
        let offset = slot * 4 + 1;
        report[offset..offset + 4].copy_from_slice(&[0x81, color.r, color.g, color.b]);
    }
    report
}

#[must_use]
pub fn match_model(endpoint: &HidEndpointInfo) -> Option<HyperXMousematModel> {
    MODELS
        .iter()
        .copied()
        .find(|model| model.matcher.matches(endpoint))
}

#[must_use]
pub fn description(model: HyperXMousematModel) -> ControllerDescription {
    let mut zone_names = Vec::with_capacity(2);
    let mut led_names = Vec::with_capacity(model.led_count());
    add_zone(
        "Underglow",
        model.first_zone_leds,
        &mut zone_names,
        &mut led_names,
    );
    add_zone(
        "LED Strip",
        model.second_zone_leds,
        &mut zone_names,
        &mut led_names,
    );
    ControllerDescription {
        name: model.name.into(),
        vendor: "HyperX".into(),
        description: "HyperX Mousemat Device".into(),
        device_type: DeviceType::MouseMat,
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 0xFFFF,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: None,
        }],
        zone_names,
        led_names,
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR),
    }
}

fn add_zone(name: &str, count: usize, zone_names: &mut Vec<String>, led_names: &mut Vec<String>) {
    if count == 0 {
        return;
    }
    zone_names.push(name.into());
    for index in 0..count {
        if count == 1 {
            led_names.push(name.into());
        } else {
            led_names.push(format!("{name} LED {}", index + 1));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::sync::Arc;

    use super::*;

    #[derive(Debug, Default)]
    struct FakeWriter(Vec<[u8; FEATURE_REPORT_LEN]>);

    impl FeatureWriter<FEATURE_REPORT_LEN> for FakeWriter {
        type Error = Infallible;

        fn send_feature_report(
            &mut self,
            report: &OutputReport<FEATURE_REPORT_LEN>,
        ) -> Result<(), Self::Error> {
            self.0.push(*report.as_bytes());
            Ok(())
        }
    }

    fn endpoint(model: HyperXMousematModel) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"hyperx-mat-test"[..]),
            model.matcher.vendor_id,
            model.matcher.product_id,
            model.matcher.interface_number.unwrap(),
            model.matcher.usage_page.unwrap_or(0x0001),
            model.matcher.usage.unwrap_or(0x0002),
            None,
            None,
            None,
        )
    }

    #[test]
    fn all_native_models_match_their_platform_endpoint() {
        for model in MODELS {
            assert_eq!(match_model(&endpoint(model)), Some(model));
        }
        let mut wrong = endpoint(PULSEFIRE_MAT);
        wrong.interface_number = 0;
        assert!(match_model(&wrong).is_none());
    }

    #[test]
    fn transaction_is_byte_exact_and_zero_pads_unused_slots() {
        let mut colors = vec![Rgb8::BLACK; PULSEFIRE_MAT.led_count()];
        colors[0] = Rgb8::new(1, 2, 3);
        colors[15] = Rgb8::new(4, 5, 6);
        colors[16] = Rgb8::new(7, 8, 9);
        colors[19] = Rgb8::new(10, 11, 12);
        let transaction = DirectColorTransaction::new(PULSEFIRE_MAT, &colors).unwrap();
        assert_eq!(
            &transaction.reports()[0].as_bytes()[..10],
            &[0, 4, 0xF2, 0, 0, 0, 0, 0, 0, 2]
        );
        assert_eq!(&transaction.reports()[1].as_bytes()[1..5], &[0x81, 1, 2, 3]);
        assert_eq!(
            &transaction.reports()[1].as_bytes()[61..65],
            &[0x81, 4, 5, 6]
        );
        assert_eq!(&transaction.reports()[2].as_bytes()[1..5], &[0x81, 7, 8, 9]);
        assert_eq!(
            &transaction.reports()[2].as_bytes()[13..17],
            &[0x81, 10, 11, 12]
        );
        assert_eq!(
            &transaction.reports()[2].as_bytes()[17..21],
            &[0x81, 0, 0, 0]
        );

        let mut writer = FakeWriter::default();
        transaction.apply(&mut writer).unwrap();
        assert_eq!(writer.0.len(), 3);
    }

    #[test]
    fn two_led_model_never_reads_the_missing_thirty_colors() {
        let transaction =
            DirectColorTransaction::new(FURY_A_XL, &[Rgb8::new(1, 2, 3), Rgb8::new(4, 5, 6)])
                .unwrap();
        assert_eq!(
            &transaction.reports()[1].as_bytes()[1..9],
            &[0x81, 1, 2, 3, 0x81, 4, 5, 6]
        );
        assert_eq!(
            &transaction.reports()[1].as_bytes()[9..13],
            &[0x81, 0, 0, 0]
        );
        assert!(
            transaction.reports()[2].as_bytes()[1..]
                .chunks_exact(4)
                .all(|slot| slot == [0x81, 0, 0, 0])
        );
    }

    #[test]
    fn color_count_and_model_shape_are_exact() {
        assert!(DirectColorTransaction::new(PULSEFIRE_MAT, &[Rgb8::BLACK; 19]).is_err());
        let standard = description(PULSEFIRE_MAT);
        assert_eq!(standard.zone_names, ["Underglow", "LED Strip"]);
        assert_eq!(standard.led_names.len(), 20);
        let xl = description(FURY_A_XL);
        assert_eq!(xl.zone_names, ["Underglow"]);
        assert_eq!(xl.led_names, ["Underglow LED 1", "Underglow LED 2"]);
        assert_eq!(KEEPALIVE_INTERVAL, Duration::from_millis(50));
    }
}
