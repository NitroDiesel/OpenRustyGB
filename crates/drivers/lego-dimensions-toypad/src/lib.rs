#![forbid(unsafe_code)]

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription,
    Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    ExactWriteError, HidDeviceMatch, HidEndpointInfo, OutputReport, OutputWriter, write_exact,
};

pub const OUTPUT_REPORT_LEN: usize = 32;
pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x0E6F,
    product_id: 0x0241,
    interface_number: None,
    usage_page: None,
    usage: None,
};

const REPORT_ID: u8 = 0x55;
const DIRECT_COMMAND: u8 = 0xC0;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToypadMode {
    Flash = 0xC3,
    Fade = 0xC2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Activation(OutputReport<OUTPUT_REPORT_LEN>);

impl Activation {
    #[must_use]
    pub const fn new() -> Self {
        let mut bytes = [0; OUTPUT_REPORT_LEN];
        bytes[1] = REPORT_ID;
        bytes[2] = 0x0F;
        bytes[3] = 0xB0;
        bytes[4] = 0x01;
        bytes[5] = 0x28;
        bytes[6] = 0x63;
        bytes[7] = 0x29;
        bytes[8] = 0x20;
        bytes[9] = 0x4C;
        bytes[10] = 0x45;
        bytes[11] = 0x47;
        bytes[12] = 0x4F;
        bytes[13] = 0x20;
        bytes[14] = 0x32;
        bytes[15] = 0x30;
        bytes[16] = 0x31;
        bytes[17] = 0x34;
        bytes[18] = 0xF7;
        Self(OutputReport::from_array(bytes))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<OUTPUT_REPORT_LEN> {
        &self.0
    }

    /// Sends the native Toy Pad activation report.
    ///
    /// # Errors
    ///
    /// Returns a transport or short-write error.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        write_exact(writer, &self.0)
    }
}

impl Default for Activation {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectColorTransaction([OutputReport<OUTPUT_REPORT_LEN>; 3]);

impl DirectColorTransaction {
    #[must_use]
    pub fn new(colors: [Rgb8; 3]) -> Self {
        Self([
            direct_report(1, colors[0]),
            direct_report(2, colors[1]),
            direct_report(3, colors[2]),
        ])
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<OUTPUT_REPORT_LEN>; 3] {
        &self.0
    }

    /// Sends Center, Left, and Right direct reports in native order.
    ///
    /// # Errors
    ///
    /// Returns the first transport or short-write error.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        for report in &self.0 {
            write_exact(writer, report)?;
        }
        Ok(())
    }
}

fn direct_report(zone: u8, color: Rgb8) -> OutputReport<OUTPUT_REPORT_LEN> {
    let mut bytes = [0; OUTPUT_REPORT_LEN];
    bytes[1..9].copy_from_slice(&[
        REPORT_ID,
        0x06,
        DIRECT_COMMAND,
        0x02,
        zone,
        color.r,
        color.g,
        color.b,
    ]);
    bytes[9] = checksum(&bytes, 9);
    OutputReport::from_array(bytes)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction(OutputReport<OUTPUT_REPORT_LEN>);

impl ModeTransaction {
    #[must_use]
    pub fn new(mode: ToypadMode, speed: u8, color: Rgb8) -> Self {
        let mut bytes = [0; OUTPUT_REPORT_LEN];
        bytes[1] = REPORT_ID;
        bytes[3] = mode as u8;
        match mode {
            ToypadMode::Flash => {
                bytes[2] = 0x09;
                bytes[4] = 0x1F;
                bytes[6] = speed;
                bytes[7] = speed;
                bytes[8] = 10;
                bytes[9..12].copy_from_slice(&[color.r, color.g, color.b]);
                bytes[12] = checksum(&bytes, 12);
            }
            ToypadMode::Fade => {
                bytes[2] = 0x08;
                bytes[4] = 0x0F;
                bytes[6] = speed;
                bytes[7] = 10;
                bytes[8..11].copy_from_slice(&[color.r, color.g, color.b]);
                bytes[11] = checksum(&bytes, 11);
            }
        }
        Self(OutputReport::from_array(bytes))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<OUTPUT_REPORT_LEN> {
        &self.0
    }

    /// Sends one native all-zone Flash or Fade report.
    ///
    /// # Errors
    ///
    /// Returns a transport or short-write error.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        write_exact(writer, &self.0)
    }
}

fn checksum(bytes: &[u8; OUTPUT_REPORT_LEN], end: usize) -> u8 {
    bytes[1..end]
        .iter()
        .fold(0, |sum, byte| sum.wrapping_add(*byte))
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description(device_name: &str) -> ControllerDescription {
    let effect_speed = Some(SpeedRange {
        min: 0,
        max: 255,
        current: 127,
    });
    ControllerDescription {
        name: device_name.into(),
        vendor: "Logic3".into(),
        description: "Lego Dimensions Toypad Base".into(),
        device_type: DeviceType::LedStrip,
        modes: vec![
            ModeDescription {
                name: "Direct".into(),
                value: 0,
                color_mode: ModeColorMode::PerLed,
                speed: None,
                brightness: None,
            },
            ModeDescription {
                name: "Flash".into(),
                value: ToypadMode::Flash as u32,
                color_mode: ModeColorMode::PerLed,
                speed: effect_speed,
                brightness: None,
            },
            ModeDescription {
                name: "Fade".into(),
                value: ToypadMode::Fade as u32,
                color_mode: ModeColorMode::PerLed,
                speed: effect_speed,
                brightness: None,
            },
        ],
        zone_names: vec!["Center".into(), "Left".into(), "Right".into()],
        led_names: vec!["LED".into(), "LED".into(), "LED".into()],
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::EFFECTS),
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::sync::Arc;

    use super::*;

    #[derive(Debug, Default)]
    struct Writer(Vec<[u8; OUTPUT_REPORT_LEN]>);

    impl OutputWriter<OUTPUT_REPORT_LEN> for Writer {
        type Error = Infallible;

        fn write_output(
            &mut self,
            report: &OutputReport<OUTPUT_REPORT_LEN>,
        ) -> Result<usize, Self::Error> {
            self.0.push(*report.as_bytes());
            Ok(OUTPUT_REPORT_LEN)
        }
    }

    fn endpoint(interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"toypad-test"[..]),
            0x0E6F,
            0x0241,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn matcher_preserves_product_only_native_detector() {
        assert!(matches(&endpoint(0, 1, 2)));
        assert!(matches(&endpoint(3, 0xFF00, 1)));
        let wrong = HidEndpointInfo::new(
            Arc::from(&b"wrong"[..]),
            0x0E6F,
            0x0242,
            0,
            1,
            2,
            None,
            None,
            None,
        );
        assert!(!matches(&wrong));
    }

    #[test]
    fn activation_is_byte_exact() {
        assert_eq!(
            &Activation::new().report().as_bytes()[1..19],
            &[
                0x55, 0x0F, 0xB0, 1, 0x28, 0x63, 0x29, 0x20, 0x4C, 0x45, 0x47, 0x4F, 0x20, 0x32,
                0x30, 0x31, 0x34, 0xF7
            ]
        );
    }

    #[test]
    fn direct_transaction_preserves_zone_order_and_checksums() {
        let transaction = DirectColorTransaction::new([
            Rgb8::new(1, 2, 3),
            Rgb8::new(4, 5, 6),
            Rgb8::new(7, 8, 9),
        ]);
        for (index, report) in transaction.reports().iter().enumerate() {
            assert_eq!(report.as_bytes()[5], u8::try_from(index + 1).unwrap());
            assert_eq!(report.as_bytes()[9], checksum(report.as_bytes(), 9));
        }
        let mut writer = Writer::default();
        transaction.apply(&mut writer).unwrap();
        assert_eq!(writer.0.len(), 3);
    }

    #[test]
    fn flash_and_fade_reports_are_byte_exact() {
        let flash = ModeTransaction::new(ToypadMode::Flash, 0x22, Rgb8::new(1, 2, 3));
        assert_eq!(
            &flash.report().as_bytes()[1..13],
            &[0x55, 9, 0xC3, 0x1F, 0, 0x22, 0x22, 10, 1, 2, 3, 0x94]
        );
        let fade = ModeTransaction::new(ToypadMode::Fade, 0x22, Rgb8::new(1, 2, 3));
        assert_eq!(
            &fade.report().as_bytes()[1..12],
            &[0x55, 8, 0xC2, 0x0F, 0, 0x22, 10, 1, 2, 3, 0x60]
        );
    }

    #[test]
    fn description_preserves_modes_and_three_single_led_zones() {
        let device = description("Lego Dimensions Toypad Base");
        assert_eq!(device.device_type, DeviceType::LedStrip);
        assert_eq!(device.zone_names, ["Center", "Left", "Right"]);
        assert_eq!(device.led_names, ["LED", "LED", "LED"]);
        assert_eq!(device.modes[1].speed.unwrap().current, 127);
        assert_eq!(device.modes[2].value, 0xC2);
    }
}
