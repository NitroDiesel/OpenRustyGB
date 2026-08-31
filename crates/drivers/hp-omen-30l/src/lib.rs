#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8, SpeedRange,
};
use openrustygb_driver_api::{
    ExactWriteError, HidDeviceMatch, HidEndpointInfo, OutputReport, OutputWriter, write_exact,
};

pub const OUTPUT_REPORT_LEN: usize = 58;
pub const ZONE_COUNT: usize = 7;
pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x103C,
    product_id: 0x84FD,
    interface_number: None,
    usage_page: None,
    usage: None,
};

const ZONE_NAMES: [&str; ZONE_COUNT] = [
    "Omen Logo",
    "Light Bar",
    "Front Fan",
    "CPU Cooler",
    "Front Bottom Fan",
    "Front Middle Fan",
    "Front Top Fan",
];
const LED_NAMES: [&str; ZONE_COUNT] = [
    "Logo LED",
    "Bar LED",
    "Fan LED",
    "CPU LED",
    "Bottom Fan LED",
    "Middle Fan LED",
    "Top Fan LED",
];

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OmenMode {
    Static = 0x01,
    Direct = 0x04,
    Off = 0x05,
    Breathing = 0x06,
    ColorCycle = 0x07,
    Blinking = 0x08,
    Wave = 0x09,
    Radial = 0x0A,
}

impl OmenMode {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Static => "Static",
            Self::Direct => "Direct",
            Self::Off => "Off",
            Self::Breathing => "Breathing",
            Self::ColorCycle => "Color Cycle",
            Self::Blinking => "Blinking",
            Self::Wave => "Wave",
            Self::Radial => "Radial",
        }
    }

    const fn is_effect(self) -> bool {
        matches!(
            self,
            Self::Breathing | Self::ColorCycle | Self::Blinking | Self::Wave | Self::Radial
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidSettings {
    Brightness(u8),
    Speed(u8),
    ColorCount { mode: OmenMode, actual: usize },
}

impl fmt::Display for InvalidSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Brightness(value) => write!(f, "brightness must be 0..=100, got {value}"),
            Self::Speed(value) => write!(f, "effect speed must be 1..=3, got {value}"),
            Self::ColorCount { mode, actual } => match mode {
                OmenMode::Direct | OmenMode::Static => {
                    write!(f, "{} requires exactly 7 colors, got {actual}", mode.name())
                }
                OmenMode::Off => write!(f, "Off requires no colors, got {actual}"),
                OmenMode::Wave => write!(f, "Wave requires exactly 6 colors, got {actual}"),
                _ => write!(
                    f,
                    "{} requires between 1 and 6 colors, got {actual}",
                    mode.name()
                ),
            },
        }
    }
}

impl std::error::Error for InvalidSettings {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeTransaction {
    reports: Vec<OutputReport<OUTPUT_REPORT_LEN>>,
}

impl ModeTransaction {
    /// Builds all seven native zone updates for one displayed mode.
    ///
    /// # Errors
    /// Rejects settings outside the ranges exposed by the upstream controller.
    pub fn new(
        mode: OmenMode,
        speed: u8,
        brightness: u8,
        colors: &[Rgb8],
    ) -> Result<Self, InvalidSettings> {
        if mode != OmenMode::Off && brightness > 100 {
            return Err(InvalidSettings::Brightness(brightness));
        }
        if mode.is_effect() && !(1..=3).contains(&speed) {
            return Err(InvalidSettings::Speed(speed));
        }
        let valid_colors = match mode {
            OmenMode::Direct | OmenMode::Static => colors.len() == ZONE_COUNT,
            OmenMode::Off => colors.is_empty(),
            OmenMode::Wave => colors.len() == 6,
            _ => (1..=6).contains(&colors.len()),
        };
        if !valid_colors {
            return Err(InvalidSettings::ColorCount {
                mode,
                actual: colors.len(),
            });
        }

        let mut reports = Vec::with_capacity(if mode.is_effect() {
            ZONE_COUNT * colors.len()
        } else {
            ZONE_COUNT
        });
        for zone in 0..ZONE_COUNT {
            if mode.is_effect() {
                for (color_index, color) in colors.iter().copied().enumerate() {
                    reports.push(zone_report(
                        zone,
                        mode,
                        speed,
                        brightness,
                        color_index + 1,
                        colors.len(),
                        Some(color),
                    ));
                }
            } else {
                let color = match mode {
                    OmenMode::Direct | OmenMode::Static => Some(colors[zone]),
                    _ => None,
                };
                reports.push(zone_report(
                    zone,
                    mode,
                    0,
                    brightness,
                    usize::from(color.is_some()),
                    usize::from(color.is_some()),
                    color,
                ));
            }
        }
        Ok(Self { reports })
    }

    #[must_use]
    pub fn reports(&self) -> &[OutputReport<OUTPUT_REPORT_LEN>] {
        &self.reports
    }

    /// Sends each zone update in native order and rejects short HID writes.
    ///
    /// # Errors
    /// Returns the first transport failure or short write.
    pub fn apply<W: OutputWriter<OUTPUT_REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        for report in &self.reports {
            write_exact(writer, report)?;
        }
        Ok(())
    }
}

fn zone_report(
    zone: usize,
    mode: OmenMode,
    speed: u8,
    brightness: u8,
    color_number: usize,
    color_count: usize,
    color: Option<Rgb8>,
) -> OutputReport<OUTPUT_REPORT_LEN> {
    let mut report = [0; OUTPUT_REPORT_LEN];
    report[0x02] = 0x12;
    report[0x03] = mode as u8;
    report[0x36] = u8::try_from(zone + 1).expect("seven zones fit in u8");
    report[0x37] = 0x01;
    if mode == OmenMode::Off {
        return OutputReport::from_array(report);
    }

    report[0x30] = brightness;
    report[0x04] = u8::try_from(color_count).expect("six colors fit in u8");
    report[0x05] = u8::try_from(color_number).expect("six colors fit in u8");
    let color = color.expect("non-Off reports have a color");
    if mode == OmenMode::Direct {
        report[0x31] = 0x04;
        let offset = 0x08 + zone * 4;
        report[offset] = 100;
        report[offset + 1..offset + 4].copy_from_slice(&[color.r, color.g, color.b]);
    } else {
        report[0x31] = if mode == OmenMode::Static { 0x02 } else { 0x0A };
        let offset = 0x08 + zone * 3;
        report[offset..offset + 3].copy_from_slice(&[color.r, color.g, color.b]);
        if mode.is_effect() {
            report[0x39] = speed;
        }
    }
    OutputReport::from_array(report)
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description() -> ControllerDescription {
    let brightness = Some(BrightnessRange {
        min: 0,
        max: 100,
        current: 100,
    });
    let speed = Some(SpeedRange {
        min: 1,
        max: 3,
        current: 2,
    });
    ControllerDescription {
        name: "HP Omen 30L".into(),
        vendor: "HP".into(),
        description: "HP Omen 30L Device".into(),
        device_type: DeviceType::Motherboard,
        modes: [
            (OmenMode::Direct, ModeColorMode::PerLed, None, brightness),
            (OmenMode::Static, ModeColorMode::PerLed, None, brightness),
            (OmenMode::Off, ModeColorMode::None, None, None),
            (
                OmenMode::Breathing,
                ModeColorMode::PerLed,
                speed,
                brightness,
            ),
            (
                OmenMode::ColorCycle,
                ModeColorMode::PerLed,
                speed,
                brightness,
            ),
            (OmenMode::Blinking, ModeColorMode::PerLed, speed, brightness),
            (OmenMode::Wave, ModeColorMode::PerLed, speed, brightness),
            (OmenMode::Radial, ModeColorMode::PerLed, speed, brightness),
        ]
        .into_iter()
        .map(|(mode, color_mode, speed, brightness)| ModeDescription {
            name: mode.name().into(),
            value: mode as u32,
            color_mode,
            speed,
            brightness,
        })
        .collect(),
        zone_names: ZONE_NAMES.iter().map(|name| (*name).into()).collect(),
        led_names: LED_NAMES.iter().map(|name| (*name).into()).collect(),
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

    #[derive(Debug)]
    struct RecordingWriter {
        reports: Vec<[u8; OUTPUT_REPORT_LEN]>,
        accepted: usize,
    }

    impl OutputWriter<OUTPUT_REPORT_LEN> for RecordingWriter {
        type Error = io::Error;

        fn write_output(
            &mut self,
            report: &OutputReport<OUTPUT_REPORT_LEN>,
        ) -> Result<usize, Self::Error> {
            self.reports.push(*report.as_bytes());
            Ok(self.accepted)
        }
    }

    fn endpoint(vendor_id: u16, product_id: u16, interface: i32) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"omen-test"[..]),
            vendor_id,
            product_id,
            interface,
            0xFF00,
            1,
            None,
            None,
            None,
        )
    }

    #[test]
    fn product_match_keeps_native_unconstrained_interface_and_usage() {
        assert!(matches(&endpoint(0x103C, 0x84FD, 9)));
        assert!(!matches(&endpoint(0x103C, 0x84FC, 9)));
        assert!(!matches(&endpoint(0x103D, 0x84FD, 9)));
    }

    #[test]
    fn direct_and_static_reports_preserve_zone_offsets() {
        let colors = [Rgb8::new(1, 2, 3); ZONE_COUNT];
        let direct = ModeTransaction::new(OmenMode::Direct, 0, 75, &colors).unwrap();
        assert_eq!(direct.reports().len(), 7);
        assert_eq!(&direct.reports()[0].as_bytes()[..6], &[0, 0, 0x12, 4, 1, 1]);
        assert_eq!(&direct.reports()[0].as_bytes()[8..12], &[100, 1, 2, 3]);
        assert_eq!(&direct.reports()[6].as_bytes()[32..36], &[100, 1, 2, 3]);
        assert_eq!(direct.reports()[6].as_bytes()[0x36], 7);
        assert_eq!(direct.reports()[6].as_bytes()[0x30], 75);

        let static_tx = ModeTransaction::new(OmenMode::Static, 0, 60, &colors).unwrap();
        assert_eq!(&static_tx.reports()[6].as_bytes()[26..29], &[1, 2, 3]);
        assert_eq!(static_tx.reports()[6].as_bytes()[0x31], 2);
    }

    #[test]
    fn effect_reports_preserve_zone_color_order_and_speed() {
        let colors = [Rgb8::new(1, 2, 3), Rgb8::new(4, 5, 6)];
        let tx = ModeTransaction::new(OmenMode::Breathing, 3, 80, &colors).unwrap();
        assert_eq!(tx.reports().len(), 14);
        assert_eq!(&tx.reports()[0].as_bytes()[4..6], &[2, 1]);
        assert_eq!(&tx.reports()[1].as_bytes()[4..6], &[2, 2]);
        assert_eq!(&tx.reports()[1].as_bytes()[8..11], &[4, 5, 6]);
        assert_eq!(tx.reports()[1].as_bytes()[0x39], 3);
        assert_eq!(tx.reports()[2].as_bytes()[0x36], 2);
    }

    #[test]
    fn off_and_setting_ranges_are_exact() {
        let off = ModeTransaction::new(OmenMode::Off, 0, 0, &[]).unwrap();
        assert_eq!(off.reports().len(), 7);
        assert_eq!(&off.reports()[0].as_bytes()[..6], &[0, 0, 0x12, 5, 0, 0]);
        assert_eq!(off.reports()[0].as_bytes()[0x36..], [1, 1, 0, 0]);
        assert!(ModeTransaction::new(OmenMode::Direct, 0, 101, &[Rgb8::BLACK; 7]).is_err());
        assert!(ModeTransaction::new(OmenMode::Wave, 2, 100, &[Rgb8::BLACK; 5]).is_err());
        assert!(ModeTransaction::new(OmenMode::Radial, 0, 100, &[Rgb8::BLACK]).is_err());
    }

    #[test]
    fn transport_short_write_and_description_are_checked() {
        let tx = ModeTransaction::new(OmenMode::Off, 0, 0, &[]).unwrap();
        let mut writer = RecordingWriter {
            reports: Vec::new(),
            accepted: OUTPUT_REPORT_LEN - 1,
        };
        assert!(matches!(
            tx.apply(&mut writer),
            Err(ExactWriteError::ShortWrite { .. })
        ));
        let description = description();
        assert_eq!(description.modes.len(), 8);
        assert_eq!(description.zone_names, ZONE_NAMES);
        assert_eq!(description.led_names, LED_NAMES);
    }
}
