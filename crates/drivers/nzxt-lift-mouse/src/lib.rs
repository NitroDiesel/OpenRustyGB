#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    ExactWriteError, HidDeviceMatch, HidEndpointInfo, InputReader, OutputReport, OutputWriter,
    write_exact,
};

pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x1E71,
    product_id: 0x2100,
    interface_number: Some(0),
    usage_page: Some(0xFFCA),
    usage: Some(0x0001),
};
pub const REPORT_LEN: usize = 64;
pub const LED_COUNT: usize = 6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug)]
pub enum FirmwareHandshakeError<E> {
    Output(ExactWriteError<E>),
    Input(E),
}

impl<E: fmt::Display> fmt::Display for FirmwareHandshakeError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Output(error) => write!(f, "could not send NZXT firmware request: {error}"),
            Self::Input(error) => write!(f, "could not read NZXT firmware response: {error}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for FirmwareHandshakeError<E> {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FirmwareHandshake(OutputReport<REPORT_LEN>);

impl FirmwareHandshake {
    #[must_use]
    pub const fn new() -> Self {
        let mut bytes = [0; REPORT_LEN];
        bytes[0] = 0x43;
        bytes[1] = 0x81;
        bytes[3] = 0x01;
        Self(OutputReport::from_array(bytes))
    }

    #[must_use]
    pub const fn request(&self) -> &OutputReport<REPORT_LEN> {
        &self.0
    }

    /// Sends the firmware request and reads until the native response signature arrives.
    ///
    /// # Errors
    ///
    /// Returns an output or input transport error.
    pub fn apply<IO, E>(&self, io: &mut IO) -> Result<FirmwareVersion, FirmwareHandshakeError<E>>
    where
        IO: OutputWriter<REPORT_LEN, Error = E> + InputReader<REPORT_LEN, Error = E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        write_exact(io, &self.0).map_err(FirmwareHandshakeError::Output)?;
        loop {
            let mut response = [0; REPORT_LEN];
            io.read_input(&mut response)
                .map_err(FirmwareHandshakeError::Input)?;
            if response[0] == 0x43 && response[1] == 0x86 {
                return Ok(FirmwareVersion {
                    major: response[3],
                    minor: response[4],
                    patch: response[5],
                });
            }
        }
    }
}

impl Default for FirmwareHandshake {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerLedColorTransaction(OutputReport<REPORT_LEN>);

impl PerLedColorTransaction {
    #[must_use]
    pub const fn new(colors: [Rgb8; LED_COUNT]) -> Self {
        let mut bytes = [0; REPORT_LEN];
        bytes[0] = 0x43;
        bytes[1] = 0xAE;
        bytes[3] = 0x10;
        bytes[4] = 0x02;
        bytes[5] = 0x3F;
        bytes[24] = 0x06;
        let order = [2, 1, 0, 3, 4, 5];
        let offsets = [25, 29, 33, 37, 41, 45];
        let mut index = 0;
        while index < LED_COUNT {
            let color = colors[order[index]];
            let offset = offsets[index];
            bytes[offset] = color.r;
            bytes[offset + 1] = color.g;
            bytes[offset + 2] = color.b;
            index += 1;
        }
        Self(OutputReport::from_array(bytes))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<REPORT_LEN> {
        &self.0
    }

    /// Sends one exact NZXT direct-color output report.
    ///
    /// # Errors
    ///
    /// Returns a transport or short-write error.
    pub fn apply<W: OutputWriter<REPORT_LEN>>(
        &self,
        writer: &mut W,
    ) -> Result<(), ExactWriteError<W::Error>> {
        write_exact(writer, &self.0)
    }
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description(device_name: &str) -> ControllerDescription {
    ControllerDescription {
        name: device_name.into(),
        vendor: "NZXT".into(),
        description: "NZXT Mouse Device".into(),
        device_type: DeviceType::Mouse,
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 0xFFFF,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: None,
        }],
        zone_names: vec!["Left".into(), "Right".into()],
        led_names: vec![
            "Left LED 0".into(),
            "Left LED 1".into(),
            "Left LED 2".into(),
            "Right LED 0".into(),
            "Right LED 1".into(),
            "Right LED 2".into(),
        ],
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::convert::Infallible;
    use std::sync::Arc;

    use super::*;

    #[derive(Debug, Default)]
    struct FakeIo {
        writes: Vec<[u8; REPORT_LEN]>,
        reads: VecDeque<[u8; REPORT_LEN]>,
    }

    impl OutputWriter<REPORT_LEN> for FakeIo {
        type Error = Infallible;

        fn write_output(
            &mut self,
            report: &OutputReport<REPORT_LEN>,
        ) -> Result<usize, Self::Error> {
            self.writes.push(*report.as_bytes());
            Ok(REPORT_LEN)
        }
    }

    impl InputReader<REPORT_LEN> for FakeIo {
        type Error = Infallible;

        fn read_input(&mut self, report: &mut [u8; REPORT_LEN]) -> Result<usize, Self::Error> {
            *report = self.reads.pop_front().expect("test response");
            Ok(REPORT_LEN)
        }
    }

    fn endpoint(interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"nzxt-test"[..]),
            0x1E71,
            0x2100,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn matcher_rejects_every_non_exact_interface_field() {
        assert!(matches(&endpoint(0, 0xFFCA, 1)));
        assert!(!matches(&endpoint(1, 0xFFCA, 1)));
        assert!(!matches(&endpoint(0, 0xFFCB, 1)));
        assert!(!matches(&endpoint(0, 0xFFCA, 2)));
    }

    #[test]
    fn firmware_handshake_preserves_request_filtering_and_version() {
        let mut ignored = [0; REPORT_LEN];
        ignored[0] = 0x43;
        ignored[1] = 0x85;
        let mut accepted = [0; REPORT_LEN];
        accepted[0] = 0x43;
        accepted[1] = 0x86;
        accepted[3..6].copy_from_slice(&[1, 2, 3]);
        let mut io = FakeIo {
            reads: VecDeque::from([ignored, accepted]),
            ..FakeIo::default()
        };
        assert_eq!(
            FirmwareHandshake::new().apply(&mut io).unwrap().to_string(),
            "1.2.3"
        );
        assert_eq!(&io.writes[0][..4], &[0x43, 0x81, 0, 1]);
    }

    #[test]
    fn direct_packet_preserves_native_led_mapping() {
        let colors = [
            Rgb8::new(1, 2, 3),
            Rgb8::new(4, 5, 6),
            Rgb8::new(7, 8, 9),
            Rgb8::new(10, 11, 12),
            Rgb8::new(13, 14, 15),
            Rgb8::new(16, 17, 18),
        ];
        let transaction = PerLedColorTransaction::new(colors);
        let report = transaction.report();
        assert_eq!(&report.as_bytes()[..6], &[0x43, 0xAE, 0, 0x10, 2, 0x3F]);
        assert_eq!(&report.as_bytes()[25..28], &[7, 8, 9]);
        assert_eq!(&report.as_bytes()[29..32], &[4, 5, 6]);
        assert_eq!(&report.as_bytes()[33..36], &[1, 2, 3]);
        assert_eq!(&report.as_bytes()[45..48], &[16, 17, 18]);
    }

    #[test]
    fn description_preserves_two_zones_and_six_leds() {
        let device = description("NZXT Lift");
        assert_eq!(device.device_type, DeviceType::Mouse);
        assert_eq!(device.modes[0].value, 0xFFFF);
        assert_eq!(device.zone_names, ["Left", "Right"]);
        assert_eq!(device.led_names.len(), LED_COUNT);
    }
}
