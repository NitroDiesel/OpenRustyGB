#![forbid(unsafe_code)]

use std::env;
use std::error::Error;
use std::fmt;
use std::num::{NonZeroU32, NonZeroU64};
use std::path::Path;

use openrustygb_domain::{ControllerId, ControllerRef, Incarnation, Rgb8};
use openrustygb_driver_api::{ExactWriteError, PrefixTooLong};
use openrustygb_driver_asus_monitor::{
    DirectColorTransaction as AsusMonitorColorTransaction,
    Initialization as AsusMonitorInitialization, InvalidLedCount as AsusMonitorInvalidLedCount,
    LedCountQuery as AsusMonitorLedCountQuery, REPORT_LEN as ASUS_MONITOR_REPORT_LEN,
    match_model as match_asus_monitor,
};
use openrustygb_driver_dream_cheeky_webmail_notifier::{
    DirectColorTransaction as DreamColorTransaction, Initialization as DreamInitialization,
    MATCH as DREAM_MATCH, OUTPUT_REPORT_LEN as DREAM_REPORT_LEN, matches as matches_dream,
};
use openrustygb_driver_faustus_keyboard::{
    DEFAULT_BASE_PATH as FAUSTUS_BASE_PATH, FaustusMode, FaustusUpdate, detect_at as detect_faustus,
};
use openrustygb_driver_gamesir_nova_lite_2::{
    MATCH as GAMESIR_MATCH, OUTPUT_REPORT_LEN as GAMESIR_REPORT_LEN,
    StaticColorTransaction as GameSirColorTransaction, matches as matches_gamesir,
};
use openrustygb_driver_hyperx_mousemat::{
    DirectColorTransaction as HyperXMousematColorTransaction,
    FEATURE_REPORT_LEN as HYPERX_MOUSEMAT_REPORT_LEN, HyperXMousematModel,
    InvalidColorCount as HyperXMousematInvalidColorCount, match_model as match_hyperx_mousemat,
};
use openrustygb_driver_hyperx_pulsefire_haste2::{
    MATCH, OUTPUT_REPORT_LEN, WheelColorTransaction, matches,
};
use openrustygb_driver_lexip_np93_alpha::{
    DirectColorTransaction as LexipColorTransaction, MATCH as LEXIP_MATCH,
    OUTPUT_REPORT_LEN as LEXIP_REPORT_LEN, matches as matches_lexip,
};
use openrustygb_driver_madcatz_cyborg_light::{
    DirectColorTransaction as MadCatzColorTransaction, EnableTransaction as MadCatzEnable,
    IntensityTransaction as MadCatzIntensity, MATCH as MADCATZ_MATCH,
    OPEN_REPORT_LEN as MADCATZ_REPORT_LEN, matches as matches_madcatz,
};
use openrustygb_driver_msi_3_zone_keyboard::{
    FEATURE_REPORT_LEN as MSI_REPORT_LEN, MATCH as MSI_MATCH,
    PerLedColorTransaction as MsiColorTransaction, matches as matches_msi,
};
use openrustygb_driver_n5312a_mouse::{
    ColorTransaction as N5312ColorTransaction, FEATURE_REPORT_LEN as N5312_REPORT_LEN,
    Initialization as N5312Initialization, InvalidModeSettings, MATCH as N5312_MATCH,
    ModeTransaction as N5312ModeTransaction, N5312Mode, matches as matches_n5312,
};
use openrustygb_driver_nvidia_esa_xps_730x::{
    AllZonesTransaction as NvidiaEsaColorTransaction, MATCH as NVIDIA_ESA_MATCH,
    OUTPUT_REPORT_LEN as NVIDIA_ESA_REPORT_LEN, matches as matches_nvidia_esa,
};
use openrustygb_driver_nzxt_lift_mouse::{
    FirmwareHandshake as NzxtFirmwareHandshake, MATCH as NZXT_MATCH,
    PerLedColorTransaction as NzxtColorTransaction, REPORT_LEN as NZXT_REPORT_LEN,
    matches as matches_nzxt,
};
use openrustygb_driver_patriot_viper_v550::{
    FEATURE_REPORT_LEN as VIPER_REPORT_LEN, Initialization as ViperInitialization,
    MATCH as VIPER_MATCH, PerLedColorTransaction as ViperColorTransaction,
    matches as matches_viper,
};
use openrustygb_driver_tecknet_m008::{
    FEATURE_REPORT_LEN as TECKNET_REPORT_LEN, InvalidSpeed as TecknetInvalidSpeed,
    MATCH as TECKNET_MATCH, ModeColorTransaction as TecknetTransaction, TecknetMode,
    matches as matches_tecknet,
};
use openrustygb_driver_thingm_blink1_mk2::{
    BlinkMode, FEATURE_REPORT_LEN as THINGM_REPORT_LEN, MATCH as THINGM_MATCH,
    ModeTransaction as ThingMModeTransaction, matches as matches_thingm,
};
use openrustygb_runtime::{CommandOutcome, ControllerActor, ControllerBackend};
use openrustygb_transport_hid::{HidInventory, HidOutput, HidTransportError};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if let Some(result) = dispatch_probe(&args) {
        return result;
    }
    if dispatch_write(&args)? {
        return Ok(());
    }
    print_usage();
    Ok(())
}

fn dispatch_probe(args: &[String]) -> Option<Result<(), Box<dyn Error>>> {
    match args {
        [] => Some(probe()),
        [command] if command == "probe-asus-monitor" => Some(probe_asus_monitor()),
        [command] if command == "probe-haste2" => Some(probe()),
        [command] if command == "probe-dream-cheeky" => Some(probe_dream()),
        [command] if command == "probe-gamesir" => Some(probe_gamesir()),
        [command] if command == "probe-hyperx-mousemat" => Some(probe_hyperx_mousemat()),
        [command] if command == "probe-lexip" => Some(probe_lexip()),
        [command] if command == "probe-madcatz" => Some(probe_madcatz()),
        [command] if command == "probe-msi-3-zone" => Some(probe_msi()),
        [command] if command == "probe-n5312" => Some(probe_n5312()),
        [command] if command == "probe-nvidia-esa" => Some(probe_nvidia_esa()),
        [command] if command == "probe-nzxt-lift" => Some(probe_nzxt()),
        [command] if command == "probe-viper-v550" => Some(probe_viper()),
        [command] if command == "probe-thingm-blink" => Some(probe_thingm()),
        [command] if command == "probe-tecknet-m008" => Some(probe_tecknet()),
        [command] if command == "probe-faustus" => {
            probe_faustus();
            Some(Ok(()))
        }
        _ => None,
    }
}

fn probe_nzxt() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_nzxt(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact NZXT Lift lighting endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found exact NZXT Lift endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_asus_monitor() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter_map(|endpoint| match_asus_monitor(endpoint).map(|model| (endpoint, model)))
        .collect();
    if exact.is_empty() {
        println!("No exact supported ASUS monitor lighting endpoint found.");
    } else {
        for (endpoint, model) in exact {
            println!(
                "Found {} endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
                model.name,
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

fn probe_tecknet() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_tecknet(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact Tecknet M008 lighting endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found Tecknet M008 endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_hyperx_mousemat() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter_map(|endpoint| match_hyperx_mousemat(endpoint).map(|model| (endpoint, model)))
        .collect();
    if exact.is_empty() {
        println!("No exact supported HyperX mousemat lighting endpoint found.");
    } else {
        for (endpoint, model) in exact {
            println!(
                "Found {} endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
                model.name,
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

fn dispatch_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    if dispatch_variable_write(args)? {
        return Ok(true);
    }
    match args {
        [command, confirmation, color]
            if command == "set-haste2-color" && confirmation == "--confirm-reversible-write" =>
        {
            set_color(parse_rgb(color)?)?;
        }
        [command, confirmation, color]
            if command == "set-dream-cheeky-color"
                && confirmation == "--confirm-reversible-write" =>
        {
            set_dream_color(parse_rgb(color)?)?;
        }
        [command, confirmation, color]
            if command == "set-gamesir-color" && confirmation == "--confirm-reversible-write" =>
        {
            set_gamesir_color(parse_rgb(color)?)?;
        }
        [command, confirmation, color]
            if command == "set-lexip-color" && confirmation == "--confirm-reversible-write" =>
        {
            set_lexip_color(parse_rgb(color)?)?;
        }
        [command, confirmation, color, brightness]
            if command == "set-madcatz-color" && confirmation == "--confirm-reversible-write" =>
        {
            set_madcatz_color(
                parse_rgb(color)?,
                parse_u8_decimal(brightness, "brightness")?,
            )?;
        }
        [command, confirmation, color]
            if command == "set-viper-v550-color"
                && confirmation == "--confirm-reversible-write" =>
        {
            set_viper_color(parse_rgb(color)?)?;
        }
        [command, confirmation, color]
            if command == "set-nvidia-esa-color"
                && confirmation == "--confirm-reversible-write" =>
        {
            set_nvidia_esa_color(parse_rgb(color)?)?;
        }
        [command, confirmation, mode, color]
            if command == "set-faustus-mode" && confirmation == "--confirm-reversible-write" =>
        {
            set_faustus_mode(parse_faustus_mode(mode)?, parse_rgb(color)?)?;
        }
        [command, confirmation, mode, color, brightness, speed]
            if command == "set-n5312-mode" && confirmation == "--confirm-reversible-write" =>
        {
            set_n5312_mode(
                parse_n5312_mode(mode)?,
                parse_rgb(color)?,
                parse_u8_decimal(brightness, "brightness")?,
                parse_u8_decimal(speed, "speed")?,
            )?;
        }
        [command, confirmation, mode, led_a, led_b, speed]
            if command == "set-thingm-blink" && confirmation == "--confirm-reversible-write" =>
        {
            set_thingm_mode(
                parse_thingm_mode(mode)?,
                [parse_rgb(led_a)?, parse_rgb(led_b)?],
                parse_u32_decimal(speed, "speed")?,
            )?;
        }
        [command, confirmation, left, middle, right, aux]
            if command == "set-msi-3-zone" && confirmation == "--confirm-reversible-write" =>
        {
            set_msi_colors([
                parse_rgb(left)?,
                parse_rgb(middle)?,
                parse_rgb(right)?,
                parse_rgb(aux)?,
            ])?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn dispatch_variable_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    match args {
        [command, confirmation, mode, color, speed]
            if command == "set-tecknet-m008" && confirmation == "--confirm-reversible-write" =>
        {
            set_tecknet_mode(
                parse_tecknet_mode(mode)?,
                parse_rgb(color)?,
                parse_u8_decimal(speed, "speed")?,
            )?;
            Ok(true)
        }
        [
            command,
            confirmation,
            left_0,
            left_1,
            left_2,
            right_0,
            right_1,
            right_2,
        ] if command == "set-nzxt-lift" && confirmation == "--confirm-reversible-write" => {
            set_nzxt_colors([
                parse_rgb(left_0)?,
                parse_rgb(left_1)?,
                parse_rgb(left_2)?,
                parse_rgb(right_0)?,
                parse_rgb(right_1)?,
                parse_rgb(right_2)?,
            ])?;
            Ok(true)
        }
        [command, confirmation, colors @ ..]
            if command == "set-asus-monitor" && confirmation == "--confirm-reversible-write" =>
        {
            set_asus_monitor_colors(parse_rgb_colors(colors)?)?;
            Ok(true)
        }
        [command, confirmation, colors @ ..]
            if command == "set-hyperx-mousemat" && confirmation == "--confirm-reversible-write" =>
        {
            set_hyperx_mousemat_colors(parse_rgb_colors(colors)?)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn print_usage() {
    eprintln!(
        "Usage:\n  openrustygb probe-asus-monitor\n  openrustygb probe-haste2\n  \
         openrustygb probe-dream-cheeky\n  \
         openrustygb probe-gamesir\n  \
         openrustygb probe-hyperx-mousemat\n  \
         openrustygb probe-lexip\n  \
         openrustygb probe-madcatz\n  \
         openrustygb probe-msi-3-zone\n  \
         openrustygb probe-n5312\n  \
         openrustygb probe-nvidia-esa\n  \
         openrustygb probe-nzxt-lift\n  \
         openrustygb probe-viper-v550\n  \
         openrustygb probe-thingm-blink\n  \
         openrustygb probe-tecknet-m008\n  \
         openrustygb probe-faustus\n  \
         openrustygb set-asus-monitor --confirm-reversible-write <RRGGBB...>\n  \
         openrustygb set-haste2-color --confirm-reversible-write RRGGBB\n  \
         openrustygb set-hyperx-mousemat --confirm-reversible-write <RRGGBB...>\n  \
         openrustygb set-dream-cheeky-color --confirm-reversible-write RRGGBB\n  \
         openrustygb set-gamesir-color --confirm-reversible-write RRGGBB\n  \
         openrustygb set-lexip-color --confirm-reversible-write RRGGBB\n  \
         openrustygb set-madcatz-color --confirm-reversible-write RRGGBB <brightness>\n  \
         openrustygb set-msi-3-zone --confirm-reversible-write \
         <LEFT-RRGGBB> <MIDDLE-RRGGBB> <RIGHT-RRGGBB> <AUX-RRGGBB>\n  \
         openrustygb set-viper-v550-color --confirm-reversible-write RRGGBB\n  \
         openrustygb set-nvidia-esa-color --confirm-reversible-write RRGGBB\n  \
         openrustygb set-nzxt-lift --confirm-reversible-write \
         <LEFT-0> <LEFT-1> <LEFT-2> <RIGHT-0> <RIGHT-1> <RIGHT-2>\n  \
         openrustygb set-thingm-blink --confirm-reversible-write \
         <off|direct|fade> <LED-A-RRGGBB> <LED-B-RRGGBB> <speed>\n  \
         openrustygb set-tecknet-m008 --confirm-reversible-write \
         <direct|off|breathing> RRGGBB <speed>\n  \
         openrustygb set-n5312-mode --confirm-reversible-write \
         <direct|breathing|single-breath|off> RRGGBB <brightness> <speed>\n  \
         openrustygb set-faustus-mode --confirm-reversible-write \
         <static|breathing|color-cycle|strobe> RRGGBB"
    );
}

fn probe_msi() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_msi(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No MSI 3-Zone keyboard endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found MSI 3-Zone endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_thingm() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_thingm(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No ThingM blink(1) mk2 endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found ThingM blink(1) mk2 endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_madcatz() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_madcatz(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No MadCatz Cyborg Gaming Light endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found MadCatz Cyborg endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_nvidia_esa() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_nvidia_esa(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No NVIDIA ESA Dell XPS 730x lighting endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found NVIDIA ESA endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_dream() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_dream(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No Dream Cheeky Webmail Notifier endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found Dream Cheeky Webmail Notifier endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_viper() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_viper(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact Patriot Viper V550 lighting endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found exact Patriot Viper V550 endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_n5312() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_n5312(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact N5312A mouse lighting endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found exact N5312A endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_lexip() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_lexip(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact Lexip NP93 Alpha lighting endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found exact Lexip NP93 Alpha endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_faustus() {
    let base = Path::new(FAUSTUS_BASE_PATH);
    if detect_faustus(base) {
        println!("Found exact Faustus ASUS TUF keyboard sysfs interface.");
    } else {
        println!("No exact Faustus ASUS TUF keyboard sysfs interface found.");
    }
    println!("Probe completed without opening an attribute or writing a value.");
}

fn set_faustus_mode(mode: FaustusMode, color: Rgb8) -> Result<(), Box<dyn Error>> {
    let base = Path::new(FAUSTUS_BASE_PATH);
    if !detect_faustus(base) {
        return Err("exact Faustus ASUS TUF keyboard sysfs interface not found".into());
    }
    FaustusUpdate { mode, color }.apply(base)?;
    println!("Applied one reversible Faustus keyboard mode transaction.");
    Ok(())
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

fn set_dream_color(color: Rgb8) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_dream);
    let endpoint = exact
        .next()
        .ok_or("Dream Cheeky Webmail Notifier endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one Dream Cheeky endpoint found; refusing to choose".into());
    }

    let mut output = HidOutput::<DREAM_REPORT_LEN>::open_matching(&endpoint, DREAM_MATCH)?;
    DreamInitialization::new().apply(&mut output)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(6).expect("six is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, DreamBackend { output }, 4)?;
    let outcome = actor.submit_whole_color(target, color)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("one-shot color command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible Dream Cheeky color transaction.");
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

fn set_lexip_color(color: Rgb8) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_lexip);
    let endpoint = exact
        .next()
        .ok_or("exact Lexip NP93 Alpha lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one exact Lexip endpoint found; refusing to choose".into());
    }

    let output = HidOutput::<LEXIP_REPORT_LEN>::open_exact(&endpoint, LEXIP_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(3).expect("three is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, LexipBackend { output }, 4)?;
    let outcome = actor.submit_whole_color(target, color)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("one-shot color command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible Lexip direct-color transaction.");
    Ok(())
}

fn set_madcatz_color(color: Rgb8, brightness: u8) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_madcatz);
    let endpoint = exact
        .next()
        .ok_or("MadCatz Cyborg Gaming Light endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one MadCatz Cyborg endpoint found; refusing to choose".into());
    }

    let mut output = HidOutput::<MADCATZ_REPORT_LEN>::open_matching(&endpoint, MADCATZ_MATCH)?;
    MadCatzEnable::new().apply(&mut output)?;
    MadCatzIntensity::new(brightness).apply(&mut output)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(8).expect("eight is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, MadCatzBackend { output }, 4)?;
    let outcome = actor.submit_whole_color(target, color)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("one-shot color command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible MadCatz Cyborg color transaction.");
    Ok(())
}

fn set_n5312_mode(
    mode: N5312Mode,
    color: Rgb8,
    brightness: u8,
    speed: u8,
) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_n5312);
    let endpoint = exact
        .next()
        .ok_or("exact N5312A mouse lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one exact N5312A endpoint found; refusing to choose".into());
    }

    let output = HidOutput::<N5312_REPORT_LEN>::open_exact(&endpoint, N5312_MATCH)?;
    let backend = N5312Backend::initialize(output)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(4).expect("four is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, backend, 4)?;
    let outcome = actor
        .submit_barrier(
            target,
            N5312Command {
                mode,
                color,
                brightness,
                speed,
            },
        )?
        .wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("mode command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible N5312A mode transaction.");
    Ok(())
}

fn set_viper_color(color: Rgb8) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_viper);
    let endpoint = exact
        .next()
        .ok_or("exact Patriot Viper V550 lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one exact Viper V550 endpoint found; refusing to choose".into());
    }

    let mut output = HidOutput::<VIPER_REPORT_LEN>::open_exact(&endpoint, VIPER_MATCH)?;
    ViperInitialization::new().apply(&mut output)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(5).expect("five is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, ViperBackend { output }, 4)?;
    let outcome = actor.submit_whole_color(target, color)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("one-shot color command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible Viper V550 seven-LED color transaction.");
    Ok(())
}

fn set_nvidia_esa_color(color: Rgb8) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_nvidia_esa);
    let endpoint = exact
        .next()
        .ok_or("NVIDIA ESA Dell XPS 730x lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one NVIDIA ESA endpoint found; refusing to choose".into());
    }

    let output = HidOutput::<NVIDIA_ESA_REPORT_LEN>::open_matching(&endpoint, NVIDIA_ESA_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(7).expect("seven is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, NvidiaEsaBackend { output }, 4)?;
    let outcome = actor.submit_whole_color(target, color)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("one-shot color command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible NVIDIA ESA five-zone color transaction.");
    Ok(())
}

fn set_thingm_mode(mode: BlinkMode, colors: [Rgb8; 2], speed: u32) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_thingm);
    let endpoint = exact
        .next()
        .ok_or("ThingM blink(1) mk2 endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one ThingM blink(1) mk2 endpoint found; refusing to choose".into());
    }

    let output = HidOutput::<THINGM_REPORT_LEN>::open_matching(&endpoint, THINGM_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(9).expect("nine is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, ThingMBackend { output }, 4)?;
    let outcome = actor
        .submit_barrier(
            target,
            ThingMCommand {
                mode,
                colors,
                speed,
            },
        )?
        .wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("mode command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible ThingM blink(1) mk2 mode transaction.");
    Ok(())
}

fn set_msi_colors(colors: [Rgb8; 4]) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_msi);
    let endpoint = exact
        .next()
        .ok_or("MSI 3-Zone keyboard endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one MSI 3-Zone endpoint found; refusing to choose".into());
    }

    let output = HidOutput::<MSI_REPORT_LEN>::open_matching(&endpoint, MSI_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(10).expect("ten is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, MsiBackend { output }, 4)?;
    let outcome = actor
        .submit_barrier(target, MsiCommand { colors })?
        .wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("color command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible MSI 3-Zone color transaction.");
    Ok(())
}

fn set_nzxt_colors(colors: [Rgb8; 6]) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_nzxt);
    let endpoint = exact
        .next()
        .ok_or("exact NZXT Lift lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one exact NZXT Lift endpoint found; refusing to choose".into());
    }

    let mut output = HidOutput::<NZXT_REPORT_LEN>::open_matching(&endpoint, NZXT_MATCH)?;
    let firmware = NzxtFirmwareHandshake::new().apply(&mut output)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(11).expect("eleven is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, NzxtBackend { output }, 4)?;
    let outcome = actor
        .submit_barrier(target, NzxtCommand { colors })?
        .wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("color command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible NZXT Lift color transaction using firmware {firmware}.");
    Ok(())
}

fn set_asus_monitor_colors(colors: Vec<Rgb8>) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints
        .into_iter()
        .filter_map(|endpoint| match_asus_monitor(&endpoint).map(|model| (endpoint, model)));
    let (endpoint, model) = exact
        .next()
        .ok_or("exact supported ASUS monitor lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err(
            "more than one supported ASUS monitor endpoint found; refusing to choose".into(),
        );
    }

    let mut output = HidOutput::<ASUS_MONITOR_REPORT_LEN>::open_matching(&endpoint, model.matcher)?;
    let led_count = AsusMonitorLedCountQuery::new().apply(&mut output)?;
    if colors.len() != usize::from(led_count) {
        return Err(format!(
            "{} reported {led_count} LEDs, but {} colors were supplied",
            model.name,
            colors.len()
        )
        .into());
    }
    AsusMonitorInitialization::new().apply(&mut output)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(12).expect("twelve is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(
        target,
        AsusMonitorBackend {
            output,
            led_count: usize::from(led_count),
        },
        4,
    )?;
    let outcome = actor
        .submit_barrier(target, AsusMonitorCommand { colors })?
        .wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("color command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible {} per-LED transaction.", model.name);
    Ok(())
}

fn set_tecknet_mode(mode: TecknetMode, color: Rgb8, speed: u8) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_tecknet);
    let endpoint = exact
        .next()
        .ok_or("exact Tecknet M008 lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one Tecknet M008 endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<TECKNET_REPORT_LEN>::open_matching(&endpoint, TECKNET_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(13).expect("thirteen is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, TecknetBackend { output }, 4)?;
    let outcome = actor
        .submit_barrier(target, TecknetCommand { mode, color, speed })?
        .wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("mode command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible Tecknet M008 mode transaction.");
    Ok(())
}

fn set_hyperx_mousemat_colors(colors: Vec<Rgb8>) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints
        .into_iter()
        .filter_map(|endpoint| match_hyperx_mousemat(&endpoint).map(|model| (endpoint, model)));
    let (endpoint, model) = exact
        .next()
        .ok_or("exact supported HyperX mousemat lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err(
            "more than one supported HyperX mousemat endpoint found; refusing to choose".into(),
        );
    }
    let output = HidOutput::<HYPERX_MOUSEMAT_REPORT_LEN>::open_matching(&endpoint, model.matcher)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(14).expect("fourteen is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, HyperXMousematBackend { output, model }, 4)?;
    let outcome = actor
        .submit_barrier(target, HyperXMousematCommand { colors })?
        .wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("color command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible {} per-LED transaction.", model.name);
    Ok(())
}

#[derive(Debug)]
struct Haste2Backend {
    output: HidOutput<OUTPUT_REPORT_LEN>,
}

#[derive(Debug)]
struct DreamBackend {
    output: HidOutput<DREAM_REPORT_LEN>,
}

#[derive(Debug)]
struct DreamBackendError(ExactWriteError<HidTransportError>);

impl fmt::Display for DreamBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not apply Dream Cheeky color: {}", self.0)
    }
}

impl Error for DreamBackendError {}

impl ControllerBackend for DreamBackend {
    type Barrier = std::convert::Infallible;
    type Error = DreamBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        DreamColorTransaction::new(color)
            .apply(&mut self.output)
            .map_err(DreamBackendError)
    }

    fn apply_barrier(&mut self, barrier: Self::Barrier) -> Result<(), Self::Error> {
        match barrier {}
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
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

#[derive(Debug)]
struct LexipBackend {
    output: HidOutput<LEXIP_REPORT_LEN>,
}

#[derive(Debug)]
enum LexipBackendError {
    Serialization(PrefixTooLong),
    Output(ExactWriteError<HidTransportError>),
}

impl fmt::Display for LexipBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialization(error) => write!(f, "could not serialize Lexip color: {error}"),
            Self::Output(error) => write!(f, "could not apply Lexip color: {error}"),
        }
    }
}

impl Error for LexipBackendError {}

impl ControllerBackend for LexipBackend {
    type Barrier = std::convert::Infallible;
    type Error = LexipBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        LexipColorTransaction::new(color)
            .map_err(LexipBackendError::Serialization)?
            .apply(&mut self.output)
            .map_err(LexipBackendError::Output)
    }

    fn apply_barrier(&mut self, barrier: Self::Barrier) -> Result<(), Self::Error> {
        match barrier {}
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
struct MadCatzBackend {
    output: HidOutput<MADCATZ_REPORT_LEN>,
}

impl ControllerBackend for MadCatzBackend {
    type Barrier = std::convert::Infallible;
    type Error = HidTransportError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        MadCatzColorTransaction::new(color).apply(&mut self.output)
    }

    fn apply_barrier(&mut self, barrier: Self::Barrier) -> Result<(), Self::Error> {
        match barrier {}
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct N5312Command {
    mode: N5312Mode,
    color: Rgb8,
    brightness: u8,
    speed: u8,
}

#[derive(Debug)]
struct N5312Backend {
    output: HidOutput<N5312_REPORT_LEN>,
}

impl N5312Backend {
    fn initialize(mut output: HidOutput<N5312_REPORT_LEN>) -> Result<Self, HidTransportError> {
        N5312Initialization::new().apply(&mut output)?;
        Ok(Self { output })
    }
}

#[derive(Debug)]
enum N5312BackendError {
    Settings(InvalidModeSettings),
    Output(HidTransportError),
}

impl fmt::Display for N5312BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid N5312A mode: {error}"),
            Self::Output(error) => write!(f, "could not apply N5312A command: {error}"),
        }
    }
}

impl Error for N5312BackendError {}

impl ControllerBackend for N5312Backend {
    type Barrier = N5312Command;
    type Error = N5312BackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        N5312ColorTransaction::new(color)
            .apply(&mut self.output)
            .map_err(N5312BackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        N5312ModeTransaction::new(
            command.mode,
            command.color,
            command.brightness,
            command.speed,
        )
        .map_err(N5312BackendError::Settings)?
        .apply(&mut self.output)
        .map_err(N5312BackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
struct ViperBackend {
    output: HidOutput<VIPER_REPORT_LEN>,
}

impl ControllerBackend for ViperBackend {
    type Barrier = std::convert::Infallible;
    type Error = HidTransportError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        ViperColorTransaction::new([color; 7]).apply(&mut self.output)
    }

    fn apply_barrier(&mut self, barrier: Self::Barrier) -> Result<(), Self::Error> {
        match barrier {}
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
struct NvidiaEsaBackend {
    output: HidOutput<NVIDIA_ESA_REPORT_LEN>,
}

#[derive(Clone, Copy, Debug)]
struct ThingMCommand {
    mode: BlinkMode,
    colors: [Rgb8; 2],
    speed: u32,
}

#[derive(Debug)]
struct ThingMBackend {
    output: HidOutput<THINGM_REPORT_LEN>,
}

#[derive(Clone, Copy, Debug)]
struct MsiCommand {
    colors: [Rgb8; 4],
}

#[derive(Debug)]
struct MsiBackend {
    output: HidOutput<MSI_REPORT_LEN>,
}

#[derive(Clone, Copy, Debug)]
struct NzxtCommand {
    colors: [Rgb8; 6],
}

#[derive(Debug)]
struct NzxtBackend {
    output: HidOutput<NZXT_REPORT_LEN>,
}

impl ControllerBackend for NzxtBackend {
    type Barrier = NzxtCommand;
    type Error = ExactWriteError<HidTransportError>;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        NzxtColorTransaction::new([color; 6]).apply(&mut self.output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        NzxtColorTransaction::new(command.colors).apply(&mut self.output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct AsusMonitorCommand {
    colors: Vec<Rgb8>,
}

#[derive(Debug)]
struct AsusMonitorBackend {
    output: HidOutput<ASUS_MONITOR_REPORT_LEN>,
    led_count: usize,
}

#[derive(Debug)]
enum AsusMonitorBackendError {
    Settings(AsusMonitorInvalidLedCount),
    Output(ExactWriteError<HidTransportError>),
}

impl fmt::Display for AsusMonitorBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid ASUS monitor colors: {error}"),
            Self::Output(error) => write!(f, "could not apply ASUS monitor colors: {error}"),
        }
    }
}

impl Error for AsusMonitorBackendError {}

impl ControllerBackend for AsusMonitorBackend {
    type Barrier = AsusMonitorCommand;
    type Error = AsusMonitorBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        AsusMonitorColorTransaction::new(&vec![color; self.led_count])
            .map_err(AsusMonitorBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(AsusMonitorBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        AsusMonitorColorTransaction::new(&command.colors)
            .map_err(AsusMonitorBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(AsusMonitorBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct TecknetCommand {
    mode: TecknetMode,
    color: Rgb8,
    speed: u8,
}

#[derive(Debug)]
struct TecknetBackend {
    output: HidOutput<TECKNET_REPORT_LEN>,
}

#[derive(Clone, Debug)]
struct HyperXMousematCommand {
    colors: Vec<Rgb8>,
}

#[derive(Debug)]
struct HyperXMousematBackend {
    output: HidOutput<HYPERX_MOUSEMAT_REPORT_LEN>,
    model: HyperXMousematModel,
}

#[derive(Debug)]
enum HyperXMousematBackendError {
    Settings(HyperXMousematInvalidColorCount),
    Output(HidTransportError),
}

impl fmt::Display for HyperXMousematBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid HyperX mousemat colors: {error}"),
            Self::Output(error) => write!(f, "could not apply HyperX mousemat colors: {error}"),
        }
    }
}

impl Error for HyperXMousematBackendError {}

impl ControllerBackend for HyperXMousematBackend {
    type Barrier = HyperXMousematCommand;
    type Error = HyperXMousematBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        HyperXMousematColorTransaction::new(self.model, &vec![color; self.model.led_count()])
            .map_err(HyperXMousematBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(HyperXMousematBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        HyperXMousematColorTransaction::new(self.model, &command.colors)
            .map_err(HyperXMousematBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(HyperXMousematBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
enum TecknetBackendError {
    Settings(TecknetInvalidSpeed),
    Output(HidTransportError),
}

impl fmt::Display for TecknetBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid Tecknet mode: {error}"),
            Self::Output(error) => write!(f, "could not apply Tecknet mode: {error}"),
        }
    }
}

impl Error for TecknetBackendError {}

impl ControllerBackend for TecknetBackend {
    type Barrier = TecknetCommand;
    type Error = TecknetBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        TecknetTransaction::new(TecknetMode::Direct, color, 0)
            .map_err(TecknetBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(TecknetBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        TecknetTransaction::new(command.mode, command.color, command.speed)
            .map_err(TecknetBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(TecknetBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl ControllerBackend for MsiBackend {
    type Barrier = MsiCommand;
    type Error = HidTransportError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        MsiColorTransaction::new([color; 4]).apply(&mut self.output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        MsiColorTransaction::new(command.colors).apply(&mut self.output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl ControllerBackend for ThingMBackend {
    type Barrier = ThingMCommand;
    type Error = HidTransportError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        ThingMModeTransaction::new(BlinkMode::Direct, [color; 2], 0).apply(&mut self.output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        ThingMModeTransaction::new(command.mode, command.colors, command.speed)
            .apply(&mut self.output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
struct NvidiaEsaBackendError(ExactWriteError<HidTransportError>);

impl fmt::Display for NvidiaEsaBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not apply NVIDIA ESA color: {}", self.0)
    }
}

impl Error for NvidiaEsaBackendError {}

impl ControllerBackend for NvidiaEsaBackend {
    type Barrier = std::convert::Infallible;
    type Error = NvidiaEsaBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        NvidiaEsaColorTransaction::new([color; 5])
            .apply(&mut self.output)
            .map_err(NvidiaEsaBackendError)
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

fn parse_rgb_colors(inputs: &[String]) -> Result<Vec<Rgb8>, Box<dyn Error>> {
    inputs.iter().map(|input| parse_rgb(input)).collect()
}

fn parse_faustus_mode(input: &str) -> Result<FaustusMode, Box<dyn Error>> {
    match input {
        "static" => Ok(FaustusMode::Static),
        "breathing" => Ok(FaustusMode::Breathing),
        "color-cycle" => Ok(FaustusMode::ColorCycle),
        "strobe" => Ok(FaustusMode::Strobe),
        _ => Err("Faustus mode must be static, breathing, color-cycle, or strobe".into()),
    }
}

fn parse_n5312_mode(input: &str) -> Result<N5312Mode, Box<dyn Error>> {
    match input {
        "direct" => Ok(N5312Mode::Direct),
        "breathing" => Ok(N5312Mode::Breathing),
        "single-breath" => Ok(N5312Mode::SingleBreath),
        "off" => Ok(N5312Mode::Off),
        _ => Err("N5312A mode must be direct, breathing, single-breath, or off".into()),
    }
}

fn parse_thingm_mode(input: &str) -> Result<BlinkMode, Box<dyn Error>> {
    match input {
        "off" => Ok(BlinkMode::Off),
        "direct" => Ok(BlinkMode::Direct),
        "fade" => Ok(BlinkMode::Fade),
        _ => Err("ThingM mode must be off, direct, or fade".into()),
    }
}

fn parse_tecknet_mode(input: &str) -> Result<TecknetMode, Box<dyn Error>> {
    match input {
        "direct" => Ok(TecknetMode::Direct),
        "off" => Ok(TecknetMode::Off),
        "breathing" => Ok(TecknetMode::Breathing),
        _ => Err("Tecknet mode must be direct, off, or breathing".into()),
    }
}

fn parse_u8_decimal(input: &str, field: &str) -> Result<u8, Box<dyn Error>> {
    input
        .parse()
        .map_err(|_| format!("{field} must be a decimal number from 0 through 255").into())
}

fn parse_u32_decimal(input: &str, field: &str) -> Result<u32, Box<dyn Error>> {
    input
        .parse()
        .map_err(|_| format!("{field} must be a decimal number from 0 through 4294967295").into())
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

    #[test]
    fn faustus_mode_parser_is_strict() {
        assert_eq!(
            parse_faustus_mode("breathing").unwrap(),
            FaustusMode::Breathing
        );
        assert!(parse_faustus_mode("cycle").is_err());
        assert!(parse_faustus_mode("Breathing").is_err());
    }

    #[test]
    fn n5312_mode_and_number_parsers_are_strict() {
        assert_eq!(
            parse_n5312_mode("single-breath").unwrap(),
            N5312Mode::SingleBreath
        );
        assert!(parse_n5312_mode("single").is_err());
        assert_eq!(parse_u8_decimal("100", "brightness").unwrap(), 100);
        assert!(parse_u8_decimal("256", "brightness").is_err());
    }

    #[test]
    fn thingm_mode_and_speed_parsers_are_strict() {
        assert_eq!(parse_thingm_mode("fade").unwrap(), BlinkMode::Fade);
        assert!(parse_thingm_mode("breathing").is_err());
        assert_eq!(parse_u32_decimal("65535", "speed").unwrap(), 65_535);
        assert!(parse_u32_decimal("4294967296", "speed").is_err());
    }
}
