#![forbid(unsafe_code)]

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x06A3,
    product_id: 0x0DC5,
    interface_number: None,
    usage_page: None,
    usage: None,
};
pub const OPEN_REPORT_LEN: usize = 9;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnableTransaction(OutputReport<2>);

impl Default for EnableTransaction {
    fn default() -> Self {
        Self::new()
    }
}

impl EnableTransaction {
    #[must_use]
    pub const fn new() -> Self {
        Self(OutputReport::from_array([0xA1, 0x00]))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<2> {
        &self.0
    }

    /// Sends the device-enable feature report.
    ///
    /// # Errors
    ///
    /// Returns the HID transport error.
    pub fn apply<W: FeatureWriter<2>>(&self, writer: &mut W) -> Result<(), W::Error> {
        send_feature(writer, &self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntensityTransaction(OutputReport<3>);

impl IntensityTransaction {
    #[must_use]
    pub const fn new(intensity: u8) -> Self {
        Self(OutputReport::from_array([
            0xA6,
            0x00,
            if intensity > 100 { 100 } else { intensity },
        ]))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<3> {
        &self.0
    }

    /// Sends the clamped intensity feature report.
    ///
    /// # Errors
    ///
    /// Returns the HID transport error.
    pub fn apply<W: FeatureWriter<3>>(&self, writer: &mut W) -> Result<(), W::Error> {
        send_feature(writer, &self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectColorTransaction(OutputReport<9>);

impl DirectColorTransaction {
    #[must_use]
    pub const fn new(color: Rgb8) -> Self {
        Self(OutputReport::from_array([
            0xA2, 0x00, color.r, color.g, color.b, 0, 0, 0, 0,
        ]))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<9> {
        &self.0
    }

    /// Sends one direct-color feature report.
    ///
    /// # Errors
    ///
    /// Returns the HID transport error.
    pub fn apply<W: FeatureWriter<9>>(&self, writer: &mut W) -> Result<(), W::Error> {
        send_feature(writer, &self.0)
    }
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description() -> ControllerDescription {
    ControllerDescription {
        name: "MadCatz Cyborg Gaming Light".into(),
        vendor: "MadCatz".into(),
        description: "MadCatz Cyborg Gaming Light".into(),
        device_type: DeviceType::Accessory,
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 0,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: Some(BrightnessRange {
                min: 0,
                max: 100,
                current: 100,
            }),
        }],
        zone_names: vec!["Cyborg".into()],
        led_names: vec!["LED".into()],
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::BRIGHTNESS),
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::sync::Arc;

    use super::*;

    #[derive(Debug, Default)]
    struct MixedWriter(Vec<Vec<u8>>);

    impl<const N: usize> FeatureWriter<N> for MixedWriter {
        type Error = Infallible;

        fn send_feature_report(&mut self, report: &OutputReport<N>) -> Result<(), Self::Error> {
            self.0.push(report.as_bytes().to_vec());
            Ok(())
        }
    }

    fn endpoint(interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"cyborg-test"[..]),
            0x06A3,
            0x0DC5,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn matcher_accepts_any_interface_and_usage_for_product() {
        assert!(matches(&endpoint(0, 0, 0)));
        assert!(matches(&endpoint(5, 0xFF00, 7)));
    }

    #[test]
    fn mixed_length_reports_are_byte_exact_and_orderable() {
        let mut writer = MixedWriter::default();
        EnableTransaction::new().apply(&mut writer).unwrap();
        IntensityTransaction::new(75).apply(&mut writer).unwrap();
        DirectColorTransaction::new(Rgb8::new(1, 2, 3))
            .apply(&mut writer)
            .unwrap();
        assert_eq!(writer.0[0], [0xA1, 0]);
        assert_eq!(writer.0[1], [0xA6, 0, 75]);
        assert_eq!(writer.0[2], [0xA2, 0, 1, 2, 3, 0, 0, 0, 0]);
    }

    #[test]
    fn intensity_clamps_at_native_maximum() {
        assert_eq!(
            IntensityTransaction::new(255).report().as_bytes(),
            &[0xA6, 0, 100]
        );
        assert_eq!(
            IntensityTransaction::new(0).report().as_bytes(),
            &[0xA6, 0, 0]
        );
    }

    #[test]
    fn description_preserves_brightness_and_single_led_shape() {
        let device = description();
        assert_eq!(device.device_type, DeviceType::Accessory);
        assert_eq!(device.modes[0].brightness.unwrap().current, 100);
        assert_eq!(device.zone_names, ["Cyborg"]);
        assert_eq!(device.led_names, ["LED"]);
    }
}
