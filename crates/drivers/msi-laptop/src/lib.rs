#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    FeatureWriter, HidDeviceMatch, HidEndpointInfo, OutputReport, send_feature,
};

pub const FEATURE_REPORT_LEN: usize = 525;
pub const SUPPORTED_MANUFACTURER: &str = "Micro-Star International Co., Ltd.";
pub const SUPPORTED_PRODUCT: &str = "Raider A18 HX A9WJG";

pub const KEYBOARD_MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x1038,
    product_id: 0x1122,
    interface_number: None,
    usage_page: None,
    usage: None,
};

pub const LIGHTBAR_MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x1038,
    product_id: 0x1161,
    interface_number: Some(0),
    usage_page: None,
    usage: None,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemIdentity {
    pub manufacturer: String,
    pub product_name: String,
}

impl SystemIdentity {
    #[must_use]
    pub fn is_supported(&self) -> bool {
        self.manufacturer == SUPPORTED_MANUFACTURER && self.product_name == SUPPORTED_PRODUCT
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MsiLaptopDevice {
    Keyboard,
    Lightbar,
}

impl MsiLaptopDevice {
    #[must_use]
    pub const fn matcher(self) -> HidDeviceMatch {
        match self {
            Self::Keyboard => KEYBOARD_MATCH,
            Self::Lightbar => LIGHTBAR_MATCH,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Keyboard => "MSI Laptop Keyboard",
            Self::Lightbar => "MSI Laptop Lightbar",
        }
    }

    #[must_use]
    pub const fn led_count(self) -> usize {
        match self {
            Self::Keyboard => KEYBOARD_LEDS.len(),
            Self::Lightbar => LIGHTBAR_LEDS.len(),
        }
    }

    const fn packet_id(self) -> u8 {
        match self {
            Self::Keyboard => 0x66,
            Self::Lightbar => 0x06,
        }
    }

    const fn leds(self) -> &'static [LedDefinition] {
        match self {
            Self::Keyboard => &KEYBOARD_LEDS,
            Self::Lightbar => &LIGHTBAR_LEDS,
        }
    }
}

pub const DEVICES: [MsiLaptopDevice; 2] = [MsiLaptopDevice::Keyboard, MsiLaptopDevice::Lightbar];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LedDefinition {
    pub name: &'static str,
    pub id: u8,
}

pub const KEYBOARD_LEDS: [LedDefinition; 102] = [
    LedDefinition {
        name: "Key: A",
        id: 0x04,
    },
    LedDefinition {
        name: "Key: B",
        id: 0x05,
    },
    LedDefinition {
        name: "Key: C",
        id: 0x06,
    },
    LedDefinition {
        name: "Key: D",
        id: 0x07,
    },
    LedDefinition {
        name: "Key: E",
        id: 0x08,
    },
    LedDefinition {
        name: "Key: F",
        id: 0x09,
    },
    LedDefinition {
        name: "Key: G",
        id: 0x0A,
    },
    LedDefinition {
        name: "Key: H",
        id: 0x0B,
    },
    LedDefinition {
        name: "Key: I",
        id: 0x0C,
    },
    LedDefinition {
        name: "Key: J",
        id: 0x0D,
    },
    LedDefinition {
        name: "Key: K",
        id: 0x0E,
    },
    LedDefinition {
        name: "Key: L",
        id: 0x0F,
    },
    LedDefinition {
        name: "Key: M",
        id: 0x10,
    },
    LedDefinition {
        name: "Key: N",
        id: 0x11,
    },
    LedDefinition {
        name: "Key: O",
        id: 0x12,
    },
    LedDefinition {
        name: "Key: P",
        id: 0x13,
    },
    LedDefinition {
        name: "Key: Q",
        id: 0x14,
    },
    LedDefinition {
        name: "Key: R",
        id: 0x15,
    },
    LedDefinition {
        name: "Key: S",
        id: 0x16,
    },
    LedDefinition {
        name: "Key: T",
        id: 0x17,
    },
    LedDefinition {
        name: "Key: U",
        id: 0x18,
    },
    LedDefinition {
        name: "Key: V",
        id: 0x19,
    },
    LedDefinition {
        name: "Key: W",
        id: 0x1A,
    },
    LedDefinition {
        name: "Key: X",
        id: 0x1B,
    },
    LedDefinition {
        name: "Key: Y",
        id: 0x1C,
    },
    LedDefinition {
        name: "Key: Z",
        id: 0x1D,
    },
    LedDefinition {
        name: "Key: 1",
        id: 0x1E,
    },
    LedDefinition {
        name: "Key: 2",
        id: 0x1F,
    },
    LedDefinition {
        name: "Key: 3",
        id: 0x20,
    },
    LedDefinition {
        name: "Key: 4",
        id: 0x21,
    },
    LedDefinition {
        name: "Key: 5",
        id: 0x22,
    },
    LedDefinition {
        name: "Key: 6",
        id: 0x23,
    },
    LedDefinition {
        name: "Key: 7",
        id: 0x24,
    },
    LedDefinition {
        name: "Key: 8",
        id: 0x25,
    },
    LedDefinition {
        name: "Key: 9",
        id: 0x26,
    },
    LedDefinition {
        name: "Key: 0",
        id: 0x27,
    },
    LedDefinition {
        name: "Key: Escape",
        id: 0x29,
    },
    LedDefinition {
        name: "Key: Tab",
        id: 0x2B,
    },
    LedDefinition {
        name: "Key: Space",
        id: 0x2C,
    },
    LedDefinition {
        name: "Key: -",
        id: 0x2D,
    },
    LedDefinition {
        name: "Key: =",
        id: 0x2E,
    },
    LedDefinition {
        name: "Key: [",
        id: 0x2F,
    },
    LedDefinition {
        name: "Key: ]",
        id: 0x30,
    },
    LedDefinition {
        name: "Key: ;",
        id: 0x33,
    },
    LedDefinition {
        name: "Key: '",
        id: 0x34,
    },
    LedDefinition {
        name: "Key: `",
        id: 0x35,
    },
    LedDefinition {
        name: "Key: ,",
        id: 0x36,
    },
    LedDefinition {
        name: "Key: .",
        id: 0x37,
    },
    LedDefinition {
        name: "Key: /",
        id: 0x38,
    },
    LedDefinition {
        name: "Key: Caps Lock",
        id: 0x39,
    },
    LedDefinition {
        name: "Key: F1",
        id: 0x3A,
    },
    LedDefinition {
        name: "Key: F2",
        id: 0x3B,
    },
    LedDefinition {
        name: "Key: F3",
        id: 0x3C,
    },
    LedDefinition {
        name: "Key: F4",
        id: 0x3D,
    },
    LedDefinition {
        name: "Key: F5",
        id: 0x3E,
    },
    LedDefinition {
        name: "Key: F6",
        id: 0x3F,
    },
    LedDefinition {
        name: "Key: F7",
        id: 0x40,
    },
    LedDefinition {
        name: "Key: F8",
        id: 0x41,
    },
    LedDefinition {
        name: "Key: F9",
        id: 0x42,
    },
    LedDefinition {
        name: "Key: F10",
        id: 0x43,
    },
    LedDefinition {
        name: "Key: F11",
        id: 0x44,
    },
    LedDefinition {
        name: "Key: F12",
        id: 0x45,
    },
    LedDefinition {
        name: "Key: Print Screen",
        id: 0x46,
    },
    LedDefinition {
        name: "Key: Scroll Lock",
        id: 0x47,
    },
    LedDefinition {
        name: "Key: Insert",
        id: 0x49,
    },
    LedDefinition {
        name: "Home/Page Up",
        id: 0x4B,
    },
    LedDefinition {
        name: "Key: Delete",
        id: 0x4C,
    },
    LedDefinition {
        name: "Key: Page Down",
        id: 0x4E,
    },
    LedDefinition {
        name: "Key: Right Arrow",
        id: 0x4F,
    },
    LedDefinition {
        name: "Key: Left Arrow",
        id: 0x50,
    },
    LedDefinition {
        name: "Key: Down Arrow",
        id: 0x51,
    },
    LedDefinition {
        name: "Key: Up Arrow",
        id: 0x52,
    },
    LedDefinition {
        name: "Key: Num Lock",
        id: 0x53,
    },
    LedDefinition {
        name: "Key: Number Pad /",
        id: 0x54,
    },
    LedDefinition {
        name: "Key: Number Pad *",
        id: 0x55,
    },
    LedDefinition {
        name: "Key: Number Pad -",
        id: 0x56,
    },
    LedDefinition {
        name: "Key: Number Pad +",
        id: 0x57,
    },
    LedDefinition {
        name: "Key: Number Pad Enter",
        id: 0x58,
    },
    LedDefinition {
        name: "Key: Number Pad 1",
        id: 0x59,
    },
    LedDefinition {
        name: "Key: Number Pad 2",
        id: 0x5A,
    },
    LedDefinition {
        name: "Key: Number Pad 3",
        id: 0x5B,
    },
    LedDefinition {
        name: "Key: Number Pad 4",
        id: 0x5C,
    },
    LedDefinition {
        name: "Key: Number Pad 5",
        id: 0x5D,
    },
    LedDefinition {
        name: "Key: Number Pad 6",
        id: 0x5E,
    },
    LedDefinition {
        name: "Key: Number Pad 7",
        id: 0x5F,
    },
    LedDefinition {
        name: "Key: Number Pad 8",
        id: 0x60,
    },
    LedDefinition {
        name: "Key: Number Pad 9",
        id: 0x61,
    },
    LedDefinition {
        name: "Key: Number Pad 0",
        id: 0x62,
    },
    LedDefinition {
        name: "Key: Number Pad .",
        id: 0x63,
    },
    LedDefinition {
        name: "Key: Power",
        id: 0x66,
    },
    LedDefinition {
        name: "Key: Left Control",
        id: 0xE0,
    },
    LedDefinition {
        name: "Key: Left Shift",
        id: 0xE1,
    },
    LedDefinition {
        name: "Key: Left Alt",
        id: 0xE2,
    },
    LedDefinition {
        name: "Key: Left Windows",
        id: 0xE3,
    },
    LedDefinition {
        name: "Key: Right Windows",
        id: 0xE4,
    },
    LedDefinition {
        name: "Key: Right Fn",
        id: 0xF0,
    },
    LedDefinition {
        name: "Key: Enter",
        id: 0x28,
    },
    LedDefinition {
        name: "Key: Backspace",
        id: 0x2A,
    },
    LedDefinition {
        name: "Key: \\",
        id: 0x31,
    },
    LedDefinition {
        name: "Key: \\ (ISO)",
        id: 0x64,
    },
    LedDefinition {
        name: "Key: Right Shift",
        id: 0xE5,
    },
    LedDefinition {
        name: "Key: Right Alt",
        id: 0xE6,
    },
];

pub const LIGHTBAR_LEDS: [LedDefinition; 4] = [
    LedDefinition {
        name: "Lightbar 1",
        id: 0x00,
    },
    LedDefinition {
        name: "Lightbar 2",
        id: 0x01,
    },
    LedDefinition {
        name: "Lightbar 3",
        id: 0x02,
    },
    LedDefinition {
        name: "Logo",
        id: 0x03,
    },
];

pub const KEYBOARD_MATRIX: [[Option<u8>; 23]; 6] = [
    [
        Some(36),
        None,
        Some(50),
        Some(51),
        Some(52),
        Some(53),
        None,
        Some(54),
        Some(55),
        Some(56),
        Some(57),
        None,
        Some(58),
        Some(59),
        Some(60),
        Some(61),
        Some(62),
        Some(63),
        None,
        None,
        None,
        None,
        Some(89),
    ],
    [
        Some(45),
        Some(26),
        Some(27),
        Some(28),
        Some(29),
        Some(30),
        Some(31),
        Some(32),
        Some(33),
        Some(34),
        Some(35),
        Some(39),
        Some(40),
        Some(97),
        None,
        Some(64),
        Some(65),
        None,
        Some(72),
        Some(73),
        Some(74),
        Some(75),
        None,
    ],
    [
        Some(37),
        Some(16),
        Some(22),
        Some(4),
        Some(17),
        Some(19),
        Some(24),
        Some(20),
        Some(8),
        Some(14),
        Some(15),
        Some(41),
        Some(42),
        Some(98),
        None,
        Some(66),
        Some(67),
        None,
        Some(84),
        Some(85),
        Some(86),
        Some(76),
        None,
    ],
    [
        Some(49),
        Some(0),
        Some(18),
        Some(3),
        Some(5),
        Some(6),
        Some(7),
        Some(9),
        Some(10),
        Some(11),
        Some(43),
        Some(44),
        Some(96),
        None,
        None,
        None,
        None,
        None,
        Some(81),
        Some(82),
        Some(83),
        None,
        None,
    ],
    [
        Some(91),
        Some(25),
        Some(23),
        Some(2),
        Some(21),
        Some(1),
        Some(13),
        Some(12),
        Some(46),
        Some(47),
        Some(48),
        Some(100),
        None,
        None,
        None,
        None,
        Some(71),
        None,
        Some(78),
        Some(79),
        Some(80),
        Some(77),
        None,
    ],
    [
        Some(90),
        Some(93),
        Some(92),
        None,
        None,
        None,
        Some(38),
        None,
        None,
        None,
        Some(101),
        Some(94),
        Some(95),
        None,
        Some(69),
        Some(70),
        Some(68),
        None,
        Some(87),
        None,
        Some(88),
        None,
        None,
    ],
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidColorCount {
    pub device: MsiLaptopDevice,
    pub actual: usize,
}

impl fmt::Display for InvalidColorCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} requires exactly {} colors, got {}",
            self.device.name(),
            self.device.led_count(),
            self.actual
        )
    }
}

impl std::error::Error for InvalidColorCount {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectColorReport(OutputReport<FEATURE_REPORT_LEN>);

impl DirectColorReport {
    /// Builds the native KLC or ALC feature report.
    ///
    /// # Errors
    /// Returns an error unless every logical LED color is supplied.
    pub fn new(device: MsiLaptopDevice, colors: &[Rgb8]) -> Result<Self, InvalidColorCount> {
        if colors.len() != device.led_count() {
            return Err(InvalidColorCount {
                device,
                actual: colors.len(),
            });
        }

        let mut report = [0; FEATURE_REPORT_LEN];
        report[1] = 0x0C;
        report[3] = device.packet_id();
        for offset in (5..FEATURE_REPORT_LEN).step_by(4) {
            report[offset] = 0xFF;
        }
        for ((definition, color), slot) in device
            .leds()
            .iter()
            .zip(colors)
            .zip(report[5..].chunks_exact_mut(4))
        {
            slot.copy_from_slice(&[definition.id, color.r, color.g, color.b]);
        }
        Ok(Self(OutputReport::from_array(report)))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<FEATURE_REPORT_LEN> {
        &self.0
    }

    /// Sends the native feature report.
    ///
    /// # Errors
    /// Returns the feature transport error.
    pub fn apply<T: FeatureWriter<FEATURE_REPORT_LEN>>(
        &self,
        transport: &mut T,
    ) -> Result<(), T::Error> {
        send_feature(transport, &self.0)
    }
}

#[must_use]
pub fn match_device(
    endpoint: &HidEndpointInfo,
    system: &SystemIdentity,
) -> Option<MsiLaptopDevice> {
    system.is_supported().then(|| {
        DEVICES
            .iter()
            .copied()
            .find(|device| device.matcher().matches(endpoint))
    })?
}

#[must_use]
pub fn description(device: MsiLaptopDevice) -> ControllerDescription {
    ControllerDescription {
        name: device.name().into(),
        vendor: "SteelSeries".into(),
        description: format!("{SUPPORTED_MANUFACTURER} {SUPPORTED_PRODUCT} RGB Device"),
        device_type: match device {
            MsiLaptopDevice::Keyboard => DeviceType::Keyboard,
            MsiLaptopDevice::Lightbar => DeviceType::LedStrip,
        },
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 0,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: None,
        }],
        zone_names: match device {
            MsiLaptopDevice::Keyboard => vec!["Keyboard".into()],
            MsiLaptopDevice::Lightbar => vec!["Lightbar".into(), "Logo".into()],
        },
        led_names: device
            .leds()
            .iter()
            .map(|definition| definition.name.into())
            .collect(),
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::sync::Arc;

    #[derive(Debug, Default)]
    struct RecordingTransport(Vec<[u8; FEATURE_REPORT_LEN]>);

    impl FeatureWriter<FEATURE_REPORT_LEN> for RecordingTransport {
        type Error = io::Error;

        fn send_feature_report(
            &mut self,
            report: &OutputReport<FEATURE_REPORT_LEN>,
        ) -> Result<(), Self::Error> {
            self.0.push(*report.as_bytes());
            Ok(())
        }
    }

    fn endpoint(product_id: u16, interface: i32) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"msi-laptop-test"[..]),
            0x1038,
            product_id,
            interface,
            0xFF00,
            1,
            None,
            None,
            None,
        )
    }

    fn supported_system() -> SystemIdentity {
        SystemIdentity {
            manufacturer: SUPPORTED_MANUFACTURER.into(),
            product_name: SUPPORTED_PRODUCT.into(),
        }
    }

    #[test]
    fn matcher_preserves_dmi_and_interface_constraints() {
        let system = supported_system();
        assert_eq!(
            match_device(&endpoint(0x1122, 7), &system),
            Some(MsiLaptopDevice::Keyboard)
        );
        assert_eq!(
            match_device(&endpoint(0x1161, 0), &system),
            Some(MsiLaptopDevice::Lightbar)
        );
        assert_eq!(match_device(&endpoint(0x1161, 1), &system), None);
        assert_eq!(
            match_device(
                &endpoint(0x1122, 7),
                &SystemIdentity {
                    manufacturer: "Other".into(),
                    product_name: SUPPORTED_PRODUCT.into()
                }
            ),
            None
        );
    }

    #[test]
    fn keyboard_packet_keeps_native_ids_rgb_order_and_ignored_slots() {
        let mut colors = [Rgb8::BLACK; 102];
        colors[0] = Rgb8::new(1, 2, 3);
        colors[101] = Rgb8::new(4, 5, 6);
        let report = DirectColorReport::new(MsiLaptopDevice::Keyboard, &colors).unwrap();
        let bytes = report.report().as_bytes();
        assert_eq!(&bytes[..5], &[0, 0x0C, 0, 0x66, 0]);
        assert_eq!(&bytes[5..9], &[0x04, 1, 2, 3]);
        assert_eq!(&bytes[409..413], &[0xE6, 4, 5, 6]);
        assert_eq!(bytes[413], 0xFF);
        assert_eq!(bytes[521], 0xFF);
        assert_eq!(&bytes[522..], &[0, 0, 0]);
    }

    #[test]
    fn lightbar_packet_uses_alc_id_and_four_native_leds() {
        let colors = [
            Rgb8::new(1, 2, 3),
            Rgb8::new(4, 5, 6),
            Rgb8::new(7, 8, 9),
            Rgb8::new(10, 11, 12),
        ];
        let report = DirectColorReport::new(MsiLaptopDevice::Lightbar, &colors).unwrap();
        let bytes = report.report().as_bytes();
        assert_eq!(bytes[3], 0x06);
        assert_eq!(
            &bytes[5..21],
            &[0, 1, 2, 3, 1, 4, 5, 6, 2, 7, 8, 9, 3, 10, 11, 12]
        );
        assert_eq!(bytes[21], 0xFF);
    }

    #[test]
    fn count_matrix_descriptions_and_transport_are_checked() {
        assert!(DirectColorReport::new(MsiLaptopDevice::Keyboard, &[]).is_err());
        assert_eq!(KEYBOARD_MATRIX.len(), 6);
        assert!(KEYBOARD_MATRIX.iter().all(|row| row.len() == 23));
        let mut mapped: Vec<_> = KEYBOARD_MATRIX
            .iter()
            .flatten()
            .flatten()
            .copied()
            .collect();
        mapped.sort_unstable();
        let expected: Vec<_> = (0..99).chain(100..102).collect();
        assert_eq!(mapped, expected);
        assert_eq!(description(MsiLaptopDevice::Keyboard).led_names.len(), 102);
        assert_eq!(
            description(MsiLaptopDevice::Lightbar).zone_names,
            ["Lightbar", "Logo"]
        );

        let report = DirectColorReport::new(MsiLaptopDevice::Lightbar, &[Rgb8::BLACK; 4]).unwrap();
        let mut transport = RecordingTransport::default();
        report.apply(&mut transport).unwrap();
        assert_eq!(transport.0, [*report.report().as_bytes()]);
    }
}
