#![forbid(unsafe_code)]

use std::fmt;

use openrustygb_domain::{
    BrightnessRange, ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode,
    ModeDescription, Rgb8,
};
use openrustygb_driver_api::{
    ExactWriteError, HidDeviceMatch, HidEndpointInfo, InputReader, OutputReport, OutputWriter,
    write_exact,
};

pub const OUTPUT_REPORT_LEN: usize = 65;
pub const INPUT_REPORT_LEN: usize = 64;
pub const LED_COUNT: usize = 91;
const LEDS_PER_PACKET: usize = 15;
const DIRECT_PACKET_COUNT: usize = LED_COUNT.div_ceil(LEDS_PER_PACKET);

pub const MATCH: HidDeviceMatch = HidDeviceMatch {
    vendor_id: 0x2516,
    product_id: 0x01B5,
    interface_number: Some(1),
    usage_page: Some(0xFF00),
    usage: None,
};

const FAN_1_IDS: [u8; 16] = [
    0x01, 0x04, 0x07, 0x0A, 0x0D, 0x10, 0x13, 0x16, 0x19, 0x1C, 0x1F, 0x22, 0x25, 0x28, 0x2B, 0x2E,
];
const FAN_2_IDS: [u8; 16] = [
    0x31, 0x34, 0x37, 0x3A, 0x3D, 0x40, 0x43, 0x46, 0x49, 0x4C, 0x4F, 0x52, 0x55, 0x58, 0x5B, 0x5E,
];
const BACK_IDS: [u8; 8] = [0x02, 0x05, 0x08, 0x0B, 0x0E, 0x11, 0x14, 0x17];
const RING_IDS: [u8; 50] = [
    0x00, 0x03, 0x06, 0x09, 0x0C, 0x0F, 0x12, 0x15, 0x18, 0x1B, 0x1E, 0x21, 0x24, 0x27, 0x2A, 0x2D,
    0x30, 0x33, 0x36, 0x39, 0x3C, 0x3F, 0x42, 0x45, 0x48, 0x4B, 0x4E, 0x51, 0x54, 0x57, 0x5A, 0x5D,
    0x60, 0x63, 0x66, 0x69, 0x6C, 0x6F, 0x72, 0x75, 0x78, 0x7B, 0x7E, 0x81, 0x84, 0x87, 0x8A, 0x8D,
    0x90, 0x93,
];
const LOGO_ID: u8 = 0x96;

const LED_IDS: [u8; LED_COUNT] = {
    let mut ids = [0; LED_COUNT];
    let mut target = 0;
    let mut source = 0;
    while source < FAN_1_IDS.len() {
        ids[target] = FAN_1_IDS[source];
        target += 1;
        source += 1;
    }
    source = 0;
    while source < FAN_2_IDS.len() {
        ids[target] = FAN_2_IDS[source];
        target += 1;
        source += 1;
    }
    source = 0;
    while source < BACK_IDS.len() {
        ids[target] = BACK_IDS[source];
        target += 1;
        source += 1;
    }
    source = 0;
    while source < RING_IDS.len() {
        ids[target] = RING_IDS[source];
        target += 1;
        source += 1;
    }
    ids[target] = LOGO_ID;
    ids
};

#[derive(Debug)]
pub enum ExchangeError<E> {
    Output(ExactWriteError<E>),
    Input(E),
    ShortRead { expected: usize, actual: usize },
}

impl<E: fmt::Display> fmt::Display for ExchangeError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Output(error) => write!(f, "{error}"),
            Self::Input(error) => write!(f, "input transport failed: {error}"),
            Self::ShortRead { expected, actual } => {
                write!(
                    f,
                    "short input read: expected {expected} bytes, read {actual}"
                )
            }
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for ExchangeError<E> {}

fn exchange<IO, E>(
    io: &mut IO,
    report: &OutputReport<OUTPUT_REPORT_LEN>,
) -> Result<[u8; INPUT_REPORT_LEN], ExchangeError<E>>
where
    IO: OutputWriter<OUTPUT_REPORT_LEN, Error = E> + InputReader<INPUT_REPORT_LEN, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    write_exact(io, report).map_err(ExchangeError::Output)?;
    let mut response = [0; INPUT_REPORT_LEN];
    let actual = io.read_input(&mut response).map_err(ExchangeError::Input)?;
    if actual != INPUT_REPORT_LEN {
        return Err(ExchangeError::ShortRead {
            expected: INPUT_REPORT_LEN,
            actual,
        });
    }
    Ok(response)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FirmwareQuery(OutputReport<OUTPUT_REPORT_LEN>);

impl FirmwareQuery {
    #[must_use]
    pub const fn new() -> Self {
        let mut bytes = [0; OUTPUT_REPORT_LEN];
        bytes[1] = 0x12;
        bytes[2] = 0x20;
        Self(OutputReport::from_array(bytes))
    }

    #[must_use]
    pub const fn report(&self) -> &OutputReport<OUTPUT_REPORT_LEN> {
        &self.0
    }

    /// Sends the native firmware query and decodes its interleaved ASCII response.
    ///
    /// # Errors
    /// Returns the first write, read, or exact-length failure.
    pub fn apply<IO, E>(&self, io: &mut IO) -> Result<String, ExchangeError<E>>
    where
        IO: OutputWriter<OUTPUT_REPORT_LEN, Error = E> + InputReader<INPUT_REPORT_LEN, Error = E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let response = exchange(io, &self.0)?;
        let mut firmware = [0; 8];
        let mut length = 0;
        for source in (0x08..0x18).step_by(2) {
            let value = response[source];
            if value == 0 {
                break;
            }
            firmware[length] = value;
            length += 1;
        }
        Ok(String::from_utf8_lossy(&firmware[..length]).into_owned())
    }
}

impl Default for FirmwareQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Initialization([OutputReport<OUTPUT_REPORT_LEN>; 2]);

impl Initialization {
    #[must_use]
    pub const fn new() -> Self {
        let mut enable = [0; OUTPUT_REPORT_LEN];
        enable[1] = 0x41;
        enable[2] = 0x03;
        let mut apply = [0; OUTPUT_REPORT_LEN];
        apply[1] = 0x51;
        apply[2] = 0x28;
        Self([
            OutputReport::from_array(enable),
            OutputReport::from_array(apply),
        ])
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<OUTPUT_REPORT_LEN>; 2] {
        &self.0
    }

    /// Sends the native enable and apply exchanges in order.
    ///
    /// # Errors
    /// Returns the first write, read, or exact-length failure.
    pub fn apply<IO, E>(&self, io: &mut IO) -> Result<(), ExchangeError<E>>
    where
        IO: OutputWriter<OUTPUT_REPORT_LEN, Error = E> + InputReader<INPUT_REPORT_LEN, Error = E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        for report in &self.0 {
            exchange(io, report)?;
        }
        Ok(())
    }
}

impl Default for Initialization {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidColorCount(pub usize);

impl fmt::Display for InvalidColorCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Intel Arc A770 Limited Edition requires exactly {LED_COUNT} colors, got {}",
            self.0
        )
    }
}

impl std::error::Error for InvalidColorCount {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectColorTransaction([OutputReport<OUTPUT_REPORT_LEN>; DIRECT_PACKET_COUNT]);

impl DirectColorTransaction {
    /// Serializes all 91 LEDs into the native sequence of at most 15 LEDs per packet.
    ///
    /// # Errors
    /// Returns an error unless the complete five-zone color model is supplied.
    pub fn new(colors: &[Rgb8]) -> Result<Self, InvalidColorCount> {
        if colors.len() != LED_COUNT {
            return Err(InvalidColorCount(colors.len()));
        }
        let mut packets = [[0; OUTPUT_REPORT_LEN]; DIRECT_PACKET_COUNT];
        for (packet_index, packet) in packets.iter_mut().enumerate() {
            let start = packet_index * LEDS_PER_PACKET;
            let end = (start + LEDS_PER_PACKET).min(LED_COUNT);
            let packet_led_count =
                u8::try_from(end - start).map_err(|_| InvalidColorCount(colors.len()))?;
            packet[1] = 0xC0;
            packet[2] = 0x01;
            packet[3] = packet_led_count;
            for (local_index, global_index) in (start..end).enumerate() {
                let id = LED_IDS[global_index];
                let color = colors[global_index];
                let offset = 5 + local_index * 4;
                packet[offset] = id;
                if id == LOGO_ID {
                    packet[offset + 1] = color.r.max(color.g).max(color.b);
                } else {
                    packet[offset + 1..offset + 4].copy_from_slice(&[color.r, color.g, color.b]);
                }
            }
        }
        Ok(Self(packets.map(OutputReport::from_array)))
    }

    #[must_use]
    pub const fn reports(&self) -> &[OutputReport<OUTPUT_REPORT_LEN>; DIRECT_PACKET_COUNT] {
        &self.0
    }

    /// Sends every direct-color packet and waits for its acknowledgement.
    ///
    /// # Errors
    /// Returns the first write, read, or exact-length failure.
    pub fn apply<IO, E>(&self, io: &mut IO) -> Result<(), ExchangeError<E>>
    where
        IO: OutputWriter<OUTPUT_REPORT_LEN, Error = E> + InputReader<INPUT_REPORT_LEN, Error = E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        for report in &self.0 {
            exchange(io, report)?;
        }
        Ok(())
    }
}

#[must_use]
pub fn matches(endpoint: &HidEndpointInfo) -> bool {
    MATCH.matches(endpoint)
}

#[must_use]
pub fn description() -> ControllerDescription {
    let mut led_names = Vec::with_capacity(LED_COUNT);
    add_led_names(&mut led_names, "Fan 1", FAN_1_IDS.len());
    add_led_names(&mut led_names, "Fan 2", FAN_2_IDS.len());
    add_led_names(&mut led_names, "Back", BACK_IDS.len());
    add_led_names(&mut led_names, "Ring", RING_IDS.len());
    add_led_names(&mut led_names, "Logo", 1);
    ControllerDescription {
        name: "Intel Arc A770 Limited Edition".into(),
        vendor: "Cooler Master".into(),
        description: "Intel Arc A770 Limited Edition".into(),
        device_type: DeviceType::Gpu,
        modes: vec![ModeDescription {
            name: "Direct".into(),
            value: 0,
            color_mode: ModeColorMode::PerLed,
            speed: None,
            brightness: Some(BrightnessRange {
                min: 0,
                max: 0,
                current: 0,
            }),
        }],
        zone_names: vec![
            "Fan 1".into(),
            "Fan 2".into(),
            "Back".into(),
            "Ring".into(),
            "Logo".into(),
        ],
        led_names,
        capabilities: ControllerCapabilities::DIRECT_COLOR
            .union(ControllerCapabilities::PER_LED_COLOR)
            .union(ControllerCapabilities::BRIGHTNESS),
    }
}

fn add_led_names(names: &mut Vec<String>, zone: &str, count: usize) {
    names.extend((1..=count).map(|index| format!("{zone} LED {index}")));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::io;
    use std::sync::Arc;

    #[derive(Debug, Default)]
    struct FakeIo {
        outputs: Vec<Vec<u8>>,
        inputs: VecDeque<Vec<u8>>,
    }

    impl FakeIo {
        fn with_responses(responses: impl IntoIterator<Item = Vec<u8>>) -> Self {
            Self {
                outputs: Vec::new(),
                inputs: responses.into_iter().collect(),
            }
        }
    }

    impl OutputWriter<OUTPUT_REPORT_LEN> for FakeIo {
        type Error = io::Error;

        fn write_output(
            &mut self,
            report: &OutputReport<OUTPUT_REPORT_LEN>,
        ) -> Result<usize, Self::Error> {
            self.outputs.push(report.as_bytes().to_vec());
            Ok(OUTPUT_REPORT_LEN)
        }
    }

    impl InputReader<INPUT_REPORT_LEN> for FakeIo {
        type Error = io::Error;

        fn read_input(
            &mut self,
            report: &mut [u8; INPUT_REPORT_LEN],
        ) -> Result<usize, Self::Error> {
            let response = self
                .inputs
                .pop_front()
                .unwrap_or_else(|| vec![0; INPUT_REPORT_LEN]);
            report[..response.len()].copy_from_slice(&response);
            Ok(response.len())
        }
    }

    fn endpoint(interface: i32, page: u16, usage: u16) -> HidEndpointInfo {
        HidEndpointInfo::new(
            Arc::from(&b"intel-arc-test"[..]),
            0x2516,
            0x01B5,
            interface,
            page,
            usage,
            None,
            None,
            None,
        )
    }

    #[test]
    fn matcher_preserves_interface_and_page_without_requiring_usage() {
        assert!(matches(&endpoint(1, 0xFF00, 7)));
        assert!(!matches(&endpoint(0, 0xFF00, 7)));
        assert!(!matches(&endpoint(1, 0xFF01, 7)));
    }

    #[test]
    fn firmware_query_preserves_interleaved_ascii_format() {
        let mut response = vec![0; INPUT_REPORT_LEN];
        for (offset, byte) in b"1.2.3".iter().copied().enumerate() {
            response[8 + offset * 2] = byte;
        }
        let mut io = FakeIo::with_responses([response]);
        assert_eq!(FirmwareQuery::new().apply(&mut io).unwrap(), "1.2.3");
        assert_eq!(&io.outputs[0][..5], &[0, 0x12, 0x20, 0, 0]);
    }

    #[test]
    fn initialization_preserves_enable_then_apply_exchanges() {
        let mut io = FakeIo::default();
        Initialization::new().apply(&mut io).unwrap();
        assert_eq!(io.outputs.len(), 2);
        assert_eq!(&io.outputs[0][..4], &[0, 0x41, 0x03, 0]);
        assert_eq!(&io.outputs[1][..4], &[0, 0x51, 0x28, 0]);
    }

    #[test]
    fn direct_packets_preserve_ids_chunking_colors_and_logo_white_channel() {
        let mut colors = [Rgb8::BLACK; LED_COUNT];
        colors[0] = Rgb8::new(1, 2, 3);
        colors[15] = Rgb8::new(4, 5, 6);
        colors[LED_COUNT - 1] = Rgb8::new(7, 9, 8);
        let transaction = DirectColorTransaction::new(&colors).unwrap();
        assert_eq!(transaction.reports().len(), 7);
        assert_eq!(
            &transaction.reports()[0].as_bytes()[..9],
            &[0, 0xC0, 1, 15, 0, 1, 1, 2, 3]
        );
        assert_eq!(
            &transaction.reports()[1].as_bytes()[..9],
            &[0, 0xC0, 1, 15, 0, 0x2E, 4, 5, 6]
        );
        assert_eq!(transaction.reports()[6].as_bytes()[3], 1);
        assert_eq!(&transaction.reports()[6].as_bytes()[5..9], &[0x96, 9, 0, 0]);
    }

    #[test]
    fn direct_apply_requires_every_acknowledgement_to_be_complete() {
        let transaction = DirectColorTransaction::new(&[Rgb8::BLACK; LED_COUNT]).unwrap();
        let mut responses = vec![vec![0; INPUT_REPORT_LEN]; DIRECT_PACKET_COUNT];
        responses[3].truncate(8);
        let mut io = FakeIo::with_responses(responses);
        assert!(matches!(
            transaction.apply(&mut io),
            Err(ExchangeError::ShortRead {
                expected: INPUT_REPORT_LEN,
                actual: 8
            })
        ));
        assert_eq!(io.outputs.len(), 4);
    }

    #[test]
    fn model_counts_and_input_validation_are_preserved() {
        assert!(DirectColorTransaction::new(&[Rgb8::BLACK; LED_COUNT - 1]).is_err());
        let device = description();
        assert_eq!(
            device.zone_names,
            ["Fan 1", "Fan 2", "Back", "Ring", "Logo"]
        );
        assert_eq!(device.led_names.len(), LED_COUNT);
        assert_eq!(device.led_names[90], "Logo LED 1");
        assert_eq!(device.modes[0].brightness.unwrap().max, 0);
    }
}
