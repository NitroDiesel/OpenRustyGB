#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    ExactWriteError, HidDeviceMatch, HidEndpointInfo, InputReader, OutputReport, OutputWriter,
    write_exact,
};

pub const REPORT_LEN: usize = 65;
pub const MAX_LED_COUNT: usize = 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AsusMonitorModel {
    pub name: &'static str,
    pub matcher: HidDeviceMatch,
}

const fn model(name: &'static str, product_id: u16) -> AsusMonitorModel {
    AsusMonitorModel {
        name,
        matcher: HidDeviceMatch {
            vendor_id: 0x0B05,
            product_id,
            interface_number: Some(1),
            usage_page: Some(0xFF72),
            usage: Some(0x00A1),
        },
    }
}

pub const MODELS: [AsusMonitorModel; 4] = [
    model("Asus ROG STRIX XG27AQDMG", 0x1BA3),
    model("Asus ROG STRIX XG27UCG", 0x1BB4),
    model("Asus ROG SWIFT PG32UCDM", 0x1B2B),
    model("Asus ROG SWIFT PG32UCDMR", 0x1C9B),
];

#[derive(Debug)]
pub enum LedCountQueryError<E> {
    Output(ExactWriteError<E>),
    Input(E),
}

impl<E: fmt::Display> fmt::Display for LedCountQueryError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Output(error) => write!(f, "could not send ASUS LED-count query: {error}"),
            Self::Input(error) => write!(f, "could not read ASUS LED-count response: {error}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for LedCountQueryError<E> {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedCountQuery(OutputReport<REPORT_LEN>);

impl LedCountQuery {
    #[must_use]
    pub const fn new() -> Self {
        let mut bytes = [0; REPORT_LEN];
        bytes[0] = 0xEC;
        bytes[1] = 0xB0;
        Self(OutputReport::from_array(bytes))
    }

    #[must_use]
    pub const fn request(&self) -> &OutputReport<REPORT_LEN> {
        &self.0
    }

    /// Sends the native LED-count request and reads one response.
    ///
    /// # Errors
    ///
    /// Returns an output or input transport error.
    pub fn apply<IO, E>(&self, io: &mut IO) -> Result<u8, LedCountQueryError<E>>
    where
        IO: OutputWriter<REPORT_LEN, Error = E> + InputReader<REPORT_LEN, Error = E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        write_exact(io, &self.0).map_err(LedCountQueryError::Output)?;
        let mut response = [0; REPORT_LEN];
        let read = io
            .read_input(&mut response)
            .map_err(LedCountQueryError::Input)?;
        Ok(if read == 0 { 0 } else { response[32] })
    }
}

impl Default for LedCountQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Initialization(OutputReport<REPORT_LEN>);

impl Initialization {
    #[must_use]
    pub const fn new() -> Self {
        let mut bytes = [0; REPORT_LEN];
        bytes[0] = 0xEC;
        bytes[1] = 0x35;
        bytes[5] = 0xFF;
        bytes[8] = 0x01;
        Self(OutputReport::from_array(bytes))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<REPORT_LEN> {
        &self.0
    }

    /// Sends the exact native initialization report.
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

impl Default for Initialization {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidLedCount(pub usize);

impl fmt::Display for InvalidLedCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ASUS monitor report supports at most {MAX_LED_COUNT} LEDs, got {}",
            self.0
        )
    }
}

impl std::error::Error for InvalidLedCount {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectColorTransaction(OutputReport<REPORT_LEN>);

impl DirectColorTransaction {
    /// Serializes the native dynamic per-LED direct report.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidLedCount`] when RGB data cannot fit in the 65-byte report.
    pub fn new(colors: &[Rgb8]) -> Result<Self, InvalidLedCount> {
        if colors.len() > MAX_LED_COUNT {
            return Err(InvalidLedCount(colors.len()));
        }
        let mut bytes = [0; REPORT_LEN];
        bytes[0] = 0xEC;
        bytes[1] = 0x40;
        bytes[2] = 0x84;
        bytes[4] = u8::try_from(colors.len()).map_err(|_| InvalidLedCount(colors.len()))?;
        for (index, color) in colors.iter().enumerate() {
            let offset = 5 + index * 3;
            bytes[offset..offset + 3].copy_from_slice(&[color.r, color.g, color.b]);
        }
        Ok(Self(OutputReport::from_array(bytes)))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<REPORT_LEN> {
        &self.0
    }

    /// Sends one exact direct-color report.
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
pub fn match_model(endpoint: &HidEndpointInfo) -> Option<AsusMonitorModel> {
    MODELS
        .iter()
        .copied()
        .find(|model| model.matcher.matches(endpoint))
}

#[must_use]
pub fn description(device_name: &str, led_count: u8) -> ControllerDescription {
    ControllerDescription {
        name: device_name.into(),
        vendor: "ASUS".into(),
        description: "ASUS monitor".into(),
        device_type: DeviceType::Monitor,
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 0,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: None,
        }],
        zone_names: vec!["Monitor".into()],
        led_names: (0..led_count).map(|index| format!("LED {index}")).collect(),
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
        reads: VecDeque<(usize, [u8; REPORT_LEN])>,
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
            let (read, response) = self.reads.pop_front().expect("test response");
            *report = response;
            Ok(read)
        }
    }

    fn endpoint(product_id: u16, interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"asus-test"[..]),
            0x0B05,
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
    fn all_four_models_require_the_exact_interface_and_usage() {
        for model in MODELS {
            assert_eq!(
                match_model(&endpoint(model.matcher.product_id, 1, 0xFF72, 0xA1)),
                Some(model)
            );
        }
        assert!(match_model(&endpoint(0x1BA3, 0, 0xFF72, 0xA1)).is_none());
        assert!(match_model(&endpoint(0x1BA3, 1, 0xFF72, 0xA2)).is_none());
    }

    #[test]
    fn led_query_and_initialization_are_byte_exact() {
        let mut response = [0; REPORT_LEN];
        response[32] = 12;
        let mut io = FakeIo {
            reads: VecDeque::from([(REPORT_LEN, response)]),
            ..FakeIo::default()
        };
        assert_eq!(LedCountQuery::new().apply(&mut io).unwrap(), 12);
        assert_eq!(&io.writes[0][..2], &[0xEC, 0xB0]);
        Initialization::new().apply(&mut io).unwrap();
        assert_eq!(&io.writes[1][..9], &[0xEC, 0x35, 0, 0, 0, 0xFF, 0, 0, 1]);
    }

    #[test]
    fn zero_byte_query_response_preserves_native_zero_count() {
        let mut io = FakeIo {
            reads: VecDeque::from([(0, [0; REPORT_LEN])]),
            ..FakeIo::default()
        };
        assert_eq!(LedCountQuery::new().apply(&mut io).unwrap(), 0);
    }

    #[test]
    fn direct_packet_is_dynamic_and_rejects_native_overflow() {
        let colors = [Rgb8::new(1, 2, 3), Rgb8::new(4, 5, 6)];
        let transaction = DirectColorTransaction::new(&colors).unwrap();
        assert_eq!(
            &transaction.report().as_bytes()[..11],
            &[0xEC, 0x40, 0x84, 0, 2, 1, 2, 3, 4, 5, 6]
        );
        assert!(DirectColorTransaction::new(&[Rgb8::BLACK; MAX_LED_COUNT + 1]).is_err());
    }

    #[test]
    fn description_uses_queried_led_count() {
        let device = description("Asus ROG SWIFT PG32UCDM", 3);
        assert_eq!(device.device_type, DeviceType::Monitor);
        assert_eq!(device.zone_names, ["Monitor"]);
        assert_eq!(device.led_names, ["LED 0", "LED 1", "LED 2"]);
    }
}
