#![forbid(unsafe_code)]

use std::env;
use std::error::Error;
use std::fmt;
use std::num::{NonZeroU32, NonZeroU64};

use openrustygb_domain::{ControllerId, ControllerRef, Incarnation, Rgb8};
use openrustygb_driver_api::{ExactWriteError, PrefixTooLong};
use openrustygb_driver_gamesir_nova_lite_2::{
    MATCH as GAMESIR_MATCH, OUTPUT_REPORT_LEN as GAMESIR_REPORT_LEN,
    StaticColorTransaction as GameSirColorTransaction, matches as matches_gamesir,
};
use openrustygb_driver_hyperx_pulsefire_haste2::{
    MATCH, OUTPUT_REPORT_LEN, WheelColorTransaction, matches,
};
use openrustygb_runtime::{CommandOutcome, ControllerActor, ControllerBackend};
use openrustygb_transport_hid::{HidInventory, HidOutput, HidTransportError};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [] => probe(),
        [command] if command == "probe-haste2" => probe(),
        [command] if command == "probe-gamesir" => probe_gamesir(),
        [command, confirmation, color]
            if command == "set-haste2-color" && confirmation == "--confirm-reversible-write" =>
        {
            set_color(parse_rgb(color)?)
        }
        [command, confirmation, color]
            if command == "set-gamesir-color" && confirmation == "--confirm-reversible-write" =>
        {
            set_gamesir_color(parse_rgb(color)?)
        }
        _ => {
            eprintln!(
                "Usage:\n  openrustygb probe-haste2\n  openrustygb probe-gamesir\n  \
                 openrustygb set-haste2-color --confirm-reversible-write RRGGBB\n  \
                 openrustygb set-gamesir-color --confirm-reversible-write RRGGBB"
            );
            Ok(())
        }
    }
}

fn probe_gamesir() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let matches: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_gamesir(endpoint))
        .collect();
    if matches.is_empty() {
        println!("No exact GameSir Nova 2 Lite lighting endpoint found.");
    } else {
        for endpoint in matches {
            println!(
                "Found exact GameSir Nova 2 Lite endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
                endpoint.vendor_id,
                endpoint.product_id,
                endpoint.interface_number,
                endpoint.usage_page,
                endpoint.usage
            );
        }
    }
    println!("Probe completed without opening a device or writing a report.");
    Ok(())
}

fn probe() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let matches: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches(endpoint))
        .collect();
    if matches.is_empty() {
        println!("No exact Haste 2 lighting endpoint found (03F0:0B97, interface 2, FF90:FF00).");
    } else {
        for endpoint in matches {
            println!(
                "Found exact Haste 2 lighting endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
                endpoint.vendor_id,
                endpoint.product_id,
                endpoint.interface_number,
                endpoint.usage_page,
                endpoint.usage
            );
        }
    }
    println!("Probe completed without opening a device or writing a report.");
    Ok(())
}

fn set_color(color: Rgb8) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches);
    let endpoint = exact
        .next()
        .ok_or("exact Haste 2 lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err(
            "more than one exact Haste 2 lighting endpoint found; refusing to choose".into(),
        );
    }

    let output = HidOutput::<OUTPUT_REPORT_LEN>::open_exact(&endpoint, MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(1).expect("one is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, Haste2Backend { output }, 4)?;
    let outcome = actor.submit_whole_color(target, color)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("one-shot color command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible scroll-wheel color transaction.");
    Ok(())
}

fn set_gamesir_color(color: Rgb8) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_gamesir);
    let endpoint = exact
        .next()
        .ok_or("exact GameSir Nova 2 Lite lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one exact GameSir endpoint found; refusing to choose".into());
    }

    let output = HidOutput::<GAMESIR_REPORT_LEN>::open_exact(&endpoint, GAMESIR_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(2).expect("two is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, GameSirBackend { output }, 4)?;
    let outcome = actor.submit_whole_color(target, color)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("one-shot color command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible GameSir static-color transaction.");
    Ok(())
}

#[derive(Debug)]
struct Haste2Backend {
    output: HidOutput<OUTPUT_REPORT_LEN>,
}

#[derive(Debug)]
enum Haste2BackendError {
    Serialization(PrefixTooLong),
    Output(ExactWriteError<HidTransportError>),
}

impl fmt::Display for Haste2BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialization(error) => write!(f, "could not serialize Haste 2 color: {error}"),
            Self::Output(error) => write!(f, "could not apply Haste 2 color: {error}"),
        }
    }
}

impl Error for Haste2BackendError {}

impl ControllerBackend for Haste2Backend {
    type Barrier = std::convert::Infallible;
    type Error = Haste2BackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        WheelColorTransaction::new(color)
            .map_err(Haste2BackendError::Serialization)?
            .apply(&mut self.output)
            .map_err(Haste2BackendError::Output)
    }

    fn apply_barrier(&mut self, barrier: Self::Barrier) -> Result<(), Self::Error> {
        match barrier {}
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
struct GameSirBackend {
    output: HidOutput<GAMESIR_REPORT_LEN>,
}

#[derive(Debug)]
enum GameSirBackendError {
    Serialization(PrefixTooLong),
    Output(ExactWriteError<HidTransportError>),
}

impl fmt::Display for GameSirBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialization(error) => write!(f, "could not serialize GameSir color: {error}"),
            Self::Output(error) => write!(f, "could not apply GameSir color: {error}"),
        }
    }
}

impl Error for GameSirBackendError {}

impl ControllerBackend for GameSirBackend {
    type Barrier = std::convert::Infallible;
    type Error = GameSirBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        GameSirColorTransaction::new(color)
            .map_err(GameSirBackendError::Serialization)?
            .apply(&mut self.output)
            .map_err(GameSirBackendError::Output)
    }

    fn apply_barrier(&mut self, barrier: Self::Barrier) -> Result<(), Self::Error> {
        match barrier {}
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn parse_rgb(input: &str) -> Result<Rgb8, Box<dyn Error>> {
    if input.len() != 6 || !input.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("color must be exactly six hexadecimal digits (RRGGBB)".into());
    }
    Ok(Rgb8::new(
        u8::from_str_radix(&input[0..2], 16)?,
        u8::from_str_radix(&input[2..4], 16)?,
        u8::from_str_radix(&input[4..6], 16)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_parser_is_strict() {
        assert_eq!(parse_rgb("1234aB").unwrap(), Rgb8::new(0x12, 0x34, 0xAB));
        assert!(parse_rgb("#123456").is_err());
        assert!(parse_rgb("12345").is_err());
        assert!(parse_rgb("xyzxyz").is_err());
    }
}
