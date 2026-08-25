#![forbid(unsafe_code)]

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    ExactWriteError, HidDeviceMatch, HidEndpointInfo, OutputReport, OutputWriter, write_exact,
};
use std::fmt;

pub const LED_COUNT: usize = 6;
pub const OUTPUT_REPORT_LEN: usize = 9;
pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x04D8,
    product_id: 0xF372,
    interface_number: None,
    usage_page: None,
    usage: None,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Pattern {
    TrafficLights = 1,
    Pattern2 = 2,
    Pattern3 = 3,
    Pattern4 = 4,
    Police = 5,
    Pattern6 = 6,
    Pattern7 = 7,
    Pattern8 = 8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LuxaforPacket(OutputReport<OUTPUT_REPORT_LEN>);

impl LuxaforPacket {
    #[must_use]
    pub const fn direct(led: u8, color: Rgb8) -> Self {
        Self(OutputReport::from_array([
            0, 1, led, color.r, color.g, color.b, 100, 0, 0,
        ]))
    }
    #[must_use]
    pub const fn fade(led: u8, color: Rgb8) -> Self {
        Self(OutputReport::from_array([
            0, 2, led, color.r, color.g, color.b, 100, 0, 0,
        ]))
    }
    #[must_use]
    pub const fn strobe(led: u8, color: Rgb8) -> Self {
        Self(OutputReport::from_array([
            0, 3, led, color.r, color.g, color.b, 100, 0, 255,
        ]))
    }
    #[must_use]
    pub const fn wave(wave_type: u8, color: Rgb8) -> Self {
        Self(OutputReport::from_array([
            0, 4, wave_type, color.r, color.g, color.b, 0, 255, 100,
        ]))
    }
    #[must_use]
    pub const fn pattern(pattern: Pattern) -> Self {
        Self(OutputReport::from_array([
            0,
            6,
            pattern as u8,
            255,
            0,
            0,
            0,
            0,
            0,
        ]))
    }
    #[must_use]
    pub const fn report(&self) -> &OutputReport<OUTPUT_REPORT_LEN> {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidColorCount(pub usize);
impl fmt::Display for InvalidColorCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Luxafor Flag requires {LED_COUNT} colors, got {}",
            self.0
        )
    }
}
impl std::error::Error for InvalidColorCount {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectTransaction([LuxaforPacket; LED_COUNT]);
impl DirectTransaction {
    /// Builds one native Direct packet for each physical LED index `1..=6`.
    ///
    /// # Errors
    /// Returns an error unless exactly six colors are supplied.
    pub fn new(colors: &[Rgb8]) -> Result<Self, InvalidColorCount> {
        let colors: [Rgb8; LED_COUNT] = colors
            .try_into()
            .map_err(|_| InvalidColorCount(colors.len()))?;
        Ok(Self([
            LuxaforPacket::direct(1, colors[0]),
            LuxaforPacket::direct(2, colors[1]),
            LuxaforPacket::direct(3, colors[2]),
            LuxaforPacket::direct(4, colors[3]),
            LuxaforPacket::direct(5, colors[4]),
            LuxaforPacket::direct(6, colors[5]),
        ]))
    }
    #[must_use]
    pub const fn packets(&self) -> &[LuxaforPacket; LED_COUNT] {
        &self.0
    }
    /// Sends all six LED packets in physical index order.
    ///
    /// # Errors
    /// Stops on a transport failure or short write.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        for packet in &self.0 {
            write_exact(writer, packet.report())?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatternTransaction(LuxaforPacket);
impl PatternTransaction {
    #[must_use]
    pub const fn new(pattern: Pattern) -> Self {
        Self(LuxaforPacket::pattern(pattern))
    }
    #[must_use]
    pub const fn packet(&self) -> &LuxaforPacket {
        &self.0
    }
    /// Sends one native pattern packet.
    ///
    /// # Errors
    /// Returns a transport failure or short write.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        write_exact(writer, self.0.report())
    }
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description() -> ControllerDescription {
    let mut modes = vec![ModeDescription {
        name: "Direct".into(),
        value: 1,
        color_mode: ModeColorMode::PerLed,
        speed: None,
        brightness: None,
    }];
    for (name, pattern) in [
        ("Traffic Lights", Pattern::TrafficLights),
        ("Pattern 2", Pattern::Pattern2),
        ("Pattern 3", Pattern::Pattern3),
        ("Pattern 4", Pattern::Pattern4),
        ("Police", Pattern::Police),
        ("Pattern 6", Pattern::Pattern6),
        ("Pattern 7", Pattern::Pattern7),
        ("Pattern 8", Pattern::Pattern8),
    ] {
        modes.push(ModeDescription {
            name: name.into(),
            value: 6 + ((pattern as u32) << 8),
            color_mode: ModeColorMode::None,
            speed: None,
            brightness: None,
        });
    }
    ControllerDescription {
        name: "Luxafor Flag".into(),
        vendor: "Luxafor".into(),
        description: "Luxafor Device".into(),
        device_type: DeviceType::Accessory,
        modes,
        zone_names: vec!["Flag".into(), "Rear".into()],
        led_names: vec![
            "Flag LED".into(),
            "Flag LED".into(),
            "Flag LED".into(),
            "Rear LED".into(),
            "Rear LED".into(),
            "Rear LED".into(),
        ],
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::EFFECTS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    fn endpoint(interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"luxafor-test"[..]),
            0x04D8,
            0xF372,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }
    #[test]
    fn product_match_preserves_unconstrained_interface_and_usage() {
        assert!(matches(&endpoint(0, 0, 0)));
        assert!(matches(&endpoint(4, 0xFF00, 7)));
    }
    #[test]
    fn every_native_packet_format_is_byte_exact() {
        let color = Rgb8::new(1, 2, 3);
        assert_eq!(
            LuxaforPacket::direct(2, color).report().as_bytes(),
            &[0, 1, 2, 1, 2, 3, 100, 0, 0]
        );
        assert_eq!(
            LuxaforPacket::fade(2, color).report().as_bytes(),
            &[0, 2, 2, 1, 2, 3, 100, 0, 0]
        );
        assert_eq!(
            LuxaforPacket::strobe(2, color).report().as_bytes(),
            &[0, 3, 2, 1, 2, 3, 100, 0, 255]
        );
        assert_eq!(
            LuxaforPacket::wave(5, color).report().as_bytes(),
            &[0, 4, 5, 1, 2, 3, 0, 255, 100]
        );
        assert_eq!(
            LuxaforPacket::pattern(Pattern::Police).report().as_bytes(),
            &[0, 6, 5, 255, 0, 0, 0, 0, 0]
        );
    }
    #[test]
    fn direct_order_patterns_and_model_shape_are_preserved() {
        let colors = [Rgb8::BLACK; LED_COUNT];
        let direct = DirectTransaction::new(&colors).unwrap();
        for (index, packet) in direct.packets().iter().enumerate() {
            assert_eq!(usize::from(packet.report().as_bytes()[2]), index + 1);
        }
        assert!(DirectTransaction::new(&colors[..5]).is_err());
        let device = description();
        assert_eq!(device.modes.len(), 9);
        assert_eq!(device.modes[5].value, 0x0506);
        assert_eq!(device.zone_names, ["Flag", "Rear"]);
        assert_eq!(device.led_names.len(), 6);
    }
}
