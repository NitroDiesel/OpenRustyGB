#![forbid(unsafe_code)]

use std::env;
use std::error::Error;
use std::fmt;
use std::num::{NonZeroU32, NonZeroU64};
use std::path::Path;

use openrustygb_domain::{ControllerId, ControllerRef, Incarnation, Rgb8};
use openrustygb_driver_anne_pro_2::{
    DirectColorTransaction as AnnePro2ColorTransaction,
    InvalidColorCount as AnnePro2InvalidColorCount, LED_COUNT as ANNE_PRO_2_LED_COUNT,
    OUTPUT_REPORT_LEN as ANNE_PRO_2_REPORT_LEN, match_model as match_anne_pro_2,
};
use openrustygb_driver_aoc_amm700_mousemat::{
    AocMode, Direction as AocDirection, FEATURE_REPORT_LEN as AOC_REPORT_LEN,
    InvalidSettings as AocInvalidSettings, MATCH as AOC_MATCH,
    ModeTransaction as AocModeTransaction, matches as matches_aoc,
};
use openrustygb_driver_aoc_gm500_mouse::{
    AocMouseMode, Direction as AocMouseDirection, FEATURE_REPORT_LEN as AOC_MOUSE_REPORT_LEN,
    InvalidSettings as AocMouseInvalidSettings, MATCH as AOC_MOUSE_MATCH,
    ModeTransaction as AocMouseModeTransaction, matches as matches_aoc_mouse,
};
use openrustygb_driver_api::{ExactWriteError, PrefixTooLong};
use openrustygb_driver_areson_mice::{
    AresonMode, FEATURE_REPORT_LEN as ARESON_REPORT_LEN, InvalidSettings as AresonInvalidSettings,
    ModeTransaction as AresonModeTransaction, match_model as match_areson,
};
use openrustygb_driver_asus_monitor::{
    DirectColorTransaction as AsusMonitorColorTransaction,
    Initialization as AsusMonitorInitialization, InvalidLedCount as AsusMonitorInvalidLedCount,
    LedCountQuery as AsusMonitorLedCountQuery, REPORT_LEN as ASUS_MONITOR_REPORT_LEN,
    match_model as match_asus_monitor,
};
use openrustygb_driver_clevo_lightbar::{
    ClevoMode, FEATURE_REPORT_LEN as CLEVO_REPORT_LEN, InvalidSettings as ClevoInvalidSettings,
    MATCH as CLEVO_MATCH, ModeTransaction as ClevoModeTransaction,
    firmware_version as clevo_firmware_version, matches as matches_clevo,
};
use openrustygb_driver_dark_project_kd3b_v2::{
    InvalidColorCount as DarkProjectInvalidColorCount, LED_COUNT as DARK_PROJECT_LED_COUNT,
    MATCH as DARK_PROJECT_MATCH, OUTPUT_REPORT_LEN as DARK_PROJECT_REPORT_LEN,
    PerLedColorTransaction as DarkProjectColorTransaction, matches as matches_dark_project,
};
use openrustygb_driver_dream_cheeky_webmail_notifier::{
    DirectColorTransaction as DreamColorTransaction, Initialization as DreamInitialization,
    MATCH as DREAM_MATCH, OUTPUT_REPORT_LEN as DREAM_REPORT_LEN, matches as matches_dream,
};
use openrustygb_driver_ducky_keyboard::{
    DirectColorTransaction as DuckyColorTransaction, DuckyModel,
    Initialization as DuckyInitialization, InvalidColorCount as DuckyInvalidColorCount,
    OUTPUT_REPORT_LEN as DUCKY_REPORT_LEN, match_model as match_ducky,
};
use openrustygb_driver_ek_loop_connect::{
    EkMode, InvalidSpeed as EkInvalidSpeed, MATCH as EK_MATCH,
    ModeTransaction as EkModeTransaction, OUTPUT_REPORT_LEN as EK_REPORT_LEN,
    matches as matches_ek,
};
use openrustygb_driver_elgato_stream_deck_mk2::{
    BUTTON_COUNT as STREAM_DECK_BUTTON_COUNT, FrameBuildError as StreamDeckFrameBuildError,
    FullFrameTransaction as StreamDeckFrameTransaction, MATCH as STREAM_DECK_MATCH,
    OUTPUT_REPORT_LEN as STREAM_DECK_REPORT_LEN, matches as matches_stream_deck,
};
use openrustygb_driver_faustus_keyboard::{
    DEFAULT_BASE_PATH as FAUSTUS_BASE_PATH, FaustusMode, FaustusUpdate, detect_at as detect_faustus,
};
use openrustygb_driver_gamesir_nova_lite_2::{
    MATCH as GAMESIR_MATCH, OUTPUT_REPORT_LEN as GAMESIR_REPORT_LEN,
    StaticColorTransaction as GameSirColorTransaction, matches as matches_gamesir,
};
use openrustygb_driver_gigabyte_aorus_c300_glass::{
    AorusCaseMode, FEATURE_REPORT_LEN as AORUS_CASE_REPORT_LEN,
    InvalidSettings as AorusCaseInvalidSettings, MATCH as AORUS_CASE_MATCH,
    ModeTransaction as AorusCaseModeTransaction, matches as matches_aorus_case,
};
use openrustygb_driver_gigabyte_aorus_m2::{
    AorusMode, DirectColorTransaction as AorusDirectColorTransaction,
    FEATURE_REPORT_LEN as AORUS_REPORT_LEN, InvalidSettings as AorusInvalidSettings,
    MATCH as AORUS_MATCH, ModeTransaction as AorusModeTransaction, matches as matches_aorus,
};
use openrustygb_driver_glorious_model_i::{
    FEATURE_REPORT_LEN as GLORIOUS_REPORT_LEN, GloriousMode,
    InvalidSettings as GloriousInvalidSettings, MATCH as GLORIOUS_MATCH,
    ModeTransaction as GloriousModeTransaction, firmware_version as glorious_firmware_version,
    matches as matches_glorious,
};
use openrustygb_driver_hyperx_mousemat::{
    DirectColorTransaction as HyperXMousematColorTransaction,
    FEATURE_REPORT_LEN as HYPERX_MOUSEMAT_REPORT_LEN, HyperXMousematModel,
    InvalidColorCount as HyperXMousematInvalidColorCount, match_model as match_hyperx_mousemat,
};
use openrustygb_driver_hyperx_pulsefire_haste2::{
    MATCH, OUTPUT_REPORT_LEN, WheelColorTransaction, matches,
};
use openrustygb_driver_hyte_keeb_tkl::{
    ApplyError as HyteApplyError, DirectColorTransaction as HyteColorTransaction,
    InvalidColorCounts as HyteInvalidColorCounts, KEY_LED_COUNT as HYTE_KEY_LED_COUNT,
    MATCH as HYTE_MATCH, OUTPUT_REPORT_LEN as HYTE_REPORT_LEN,
    UNDERGLOW_LED_COUNT as HYTE_UNDERGLOW_LED_COUNT, matches as matches_hyte,
};
use openrustygb_driver_instant_mice::{
    Direction as InstantDirection, FEATURE_REPORT_LEN as INSTANT_REPORT_LEN, InstantMode,
    InstantMouseModel, InvalidSettings as InstantInvalidSettings,
    ModeTransaction as InstantModeTransaction, match_model as match_instant_mouse,
};
use openrustygb_driver_intel_arc_a770_le::{
    DirectColorTransaction as IntelArcColorTransaction, ExchangeError as IntelArcExchangeError,
    FirmwareQuery as IntelArcFirmwareQuery, Initialization as IntelArcInitialization,
    InvalidColorCount as IntelArcInvalidColorCount, LED_COUNT as INTEL_ARC_LED_COUNT,
    MATCH as INTEL_ARC_MATCH, OUTPUT_REPORT_LEN as INTEL_ARC_REPORT_LEN,
    matches as matches_intel_arc,
};
use openrustygb_driver_ionico::{
    ApplyError as IonicoApplyError, InvalidSettings as IonicoInvalidSettings, IonicoMode,
    IonicoModel, ModeTransaction as IonicoModeTransaction,
    OUTPUT_REPORT_LEN as IONICO_OUTPUT_REPORT_LEN, SaveTransaction as IonicoSaveTransaction,
    match_model as match_ionico,
};
use openrustygb_driver_lego_dimensions_toypad::{
    Activation as LegoActivation, DirectColorTransaction as LegoDirectColorTransaction,
    MATCH as LEGO_MATCH, ModeTransaction as LegoModeTransaction,
    OUTPUT_REPORT_LEN as LEGO_REPORT_LEN, ToypadMode, matches as matches_lego,
};
use openrustygb_driver_lexip_np93_alpha::{
    DirectColorTransaction as LexipColorTransaction, MATCH as LEXIP_MATCH,
    OUTPUT_REPORT_LEN as LEXIP_REPORT_LEN, matches as matches_lexip,
};
use openrustygb_driver_luxafor_flag::{
    DirectTransaction as LuxaforDirectTransaction, InvalidColorCount as LuxaforInvalidColorCount,
    LED_COUNT as LUXAFOR_LED_COUNT, MATCH as LUXAFOR_MATCH,
    OUTPUT_REPORT_LEN as LUXAFOR_REPORT_LEN, Pattern as LuxaforPattern,
    PatternTransaction as LuxaforPatternTransaction, matches as matches_luxafor,
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
use openrustygb_driver_msi_laptop::{
    DirectColorReport as MsiLaptopColorReport, FEATURE_REPORT_LEN as MSI_LAPTOP_REPORT_LEN,
    InvalidColorCount as MsiLaptopInvalidColorCount, MsiLaptopDevice, SystemIdentity,
    match_device as match_msi_laptop,
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
use openrustygb_driver_red_square_keyrox::{
    CustomColorTransaction as KeyroxCustomTransaction, Direction as KeyroxDirection,
    FEATURE_REPORT_LEN as KEYROX_REPORT_LEN, HardwareModeTransaction as KeyroxModeTransaction,
    InvalidSettings as KeyroxInvalidSettings, KeyroxMode, LED_COUNT as KEYROX_LED_COUNT,
    MATCH_TKL as KEYROX_TKL_MATCH, MATCH_TKL_V2 as KEYROX_TKL_V2_MATCH,
    ModeColor as KeyroxModeColor, match_model as match_keyrox,
};
use openrustygb_driver_redragon_mice::{
    FEATURE_REPORT_LEN as REDRAGON_REPORT_LEN, Initialization as RedragonInitialization,
    ModeTransaction as RedragonModeTransaction, RedragonMode, match_model as match_redragon,
};
use openrustygb_driver_sayodevice_e1::{
    ApplyError as SayoApplyError, InvalidSpeed as SayoInvalidSpeed, MATCH as SAYO_MATCH,
    ModeTransaction as SayoModeTransaction, REPORT_LEN as SAYO_REPORT_LEN,
    SaveTransaction as SayoSaveTransaction, SayoMode, matches as matches_sayo,
};
use openrustygb_driver_skydimo_sk0902::{
    FrameTransaction as SkydimoFrameTransaction, InvalidColorCount as SkydimoInvalidColorCount,
    LED_COUNT as SKYDIMO_LED_COUNT, MATCH as SKYDIMO_MATCH,
    OUTPUT_REPORT_LEN as SKYDIMO_REPORT_LEN, matches as matches_skydimo,
};
use openrustygb_driver_skyloong_gk104_pro::{
    DirectColorTransaction as SkyloongColorTransaction, Initialization as SkyloongInitialization,
    InvalidSettings as SkyloongInvalidSettings, LED_COUNT as SKYLOONG_LED_COUNT,
    MATCH as SKYLOONG_MATCH, OUTPUT_REPORT_LEN as SKYLOONG_REPORT_LEN,
    Shutdown as SkyloongShutdown, matches as matches_skyloong,
};
use openrustygb_driver_tecknet_m008::{
    FEATURE_REPORT_LEN as TECKNET_REPORT_LEN, InvalidSpeed as TecknetInvalidSpeed,
    MATCH as TECKNET_MATCH, ModeColorTransaction as TecknetTransaction, TecknetMode,
    matches as matches_tecknet,
};
use openrustygb_driver_thermaltake_poseidon_z_rgb::{
    DirectColorTransaction as PoseidonDirectTransaction, Direction as PoseidonDirection,
    FEATURE_REPORT_LEN as POSEIDON_REPORT_LEN, InvalidSettings as PoseidonInvalidSettings,
    LED_COUNT as POSEIDON_LED_COUNT, MATCH as POSEIDON_MATCH,
    ModeTransaction as PoseidonModeTransaction, PoseidonMode,
    ProfileColorTransaction as PoseidonProfileTransaction, matches as matches_poseidon,
};
use openrustygb_driver_thingm_blink1_mk2::{
    BlinkMode, FEATURE_REPORT_LEN as THINGM_REPORT_LEN, MATCH as THINGM_MATCH,
    ModeTransaction as ThingMModeTransaction, matches as matches_thingm,
};
use openrustygb_driver_valkyrie_vk99::{
    DirectColorTransaction as ValkyrieColorTransaction, FEATURE_REPORT_LEN as VALKYRIE_REPORT_LEN,
    InvalidColorCount as ValkyrieInvalidColorCount, ValkyrieModel, match_model as match_valkyrie,
};
use openrustygb_driver_wushi_l50::{
    Direction as WushiDirection, InvalidSettings as WushiInvalidSettings,
    LED_COUNT as WUSHI_LED_COUNT, MATCH as WUSHI_MATCH, ModeTransaction as WushiModeTransaction,
    REPORT_LEN as WUSHI_REPORT_LEN, WushiMode, matches as matches_wushi,
};
use openrustygb_driver_xpg_summoner::{
    DirectColorTransaction as XpgSummonerColorTransaction,
    Initialization as XpgSummonerInitialization, InvalidColorCount as XpgSummonerInvalidColorCount,
    LED_COUNT as XPG_SUMMONER_LED_COUNT, MATCH as XPG_SUMMONER_MATCH,
    OUTPUT_REPORT_LEN as XPG_SUMMONER_REPORT_LEN, Shutdown as XpgSummonerShutdown,
    matches as matches_xpg_summoner,
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
    print_skyloong_usage();
    print_anne_pro_2_usage();
    print_ionico_usage();
    print_xpg_summoner_usage();
    print_ducky_usage();
    print_poseidon_usage();
    print_keyrox_usage();
    print_valkyrie_usage();
    print_msi_laptop_usage();
    Ok(())
}

fn dispatch_probe(args: &[String]) -> Option<Result<(), Box<dyn Error>>> {
    match args {
        [] => Some(probe()),
        [command] if command == "probe-aoc-amm700" => Some(probe_aoc()),
        [command] if command == "probe-aoc-gm500" => Some(probe_aoc_mouse()),
        [command] if command == "probe-anne-pro-2" => Some(probe_anne_pro_2()),
        [command] if command == "probe-asus-monitor" => Some(probe_asus_monitor()),
        [command] if command == "probe-clevo-lightbar" => Some(probe_clevo()),
        [command] if command == "probe-areson" => Some(probe_areson()),
        [command] if command == "probe-redragon" => Some(probe_redragon()),
        [command] if command == "probe-haste2" => Some(probe()),
        [command] if command == "probe-dream-cheeky" => Some(probe_dream()),
        [command] if command == "probe-ducky" => Some(probe_ducky()),
        [command] if command == "probe-poseidon-z-rgb" => Some(probe_poseidon()),
        [command] if command == "probe-keyrox" => Some(probe_keyrox()),
        [command] if command == "probe-valkyrie-vk99" => Some(probe_valkyrie()),
        [command] if command == "probe-msi-laptop" => Some(probe_msi_laptop()),
        [command] if command == "probe-ek-loop-connect" => Some(probe_ek()),
        [command] if command == "probe-dark-project" => Some(probe_dark_project()),
        [command] if command == "probe-stream-deck" => Some(probe_stream_deck()),
        [command] if command == "probe-sayo" => Some(probe_sayo()),
        [command] if command == "probe-skydimo" => Some(probe_skydimo()),
        [command] if command == "probe-wushi" => Some(probe_wushi()),
        [command] if command == "probe-gamesir" => Some(probe_gamesir()),
        [command] if command == "probe-glorious-model-i" => Some(probe_glorious()),
        [command] if command == "probe-hyte-keeb-tkl" => Some(probe_hyte()),
        [command] if command == "probe-intel-arc-a770-le" => Some(probe_intel_arc()),
        [command] if command == "probe-skyloong-gk104-pro" => Some(probe_skyloong()),
        [command] if command == "probe-aorus-m2" => Some(probe_aorus()),
        [command] if command == "probe-aorus-case" => Some(probe_aorus_case()),
        [command] if command == "probe-hyperx-mousemat" => Some(probe_hyperx_mousemat()),
        [command] if command == "probe-instant-mouse" => Some(probe_instant_mouse()),
        [command] if command == "probe-ionico" => Some(probe_ionico()),
        [command] if command == "probe-xpg-summoner" => Some(probe_xpg_summoner()),
        [command] if command == "probe-lego-toypad" => Some(probe_lego()),
        [command] if command == "probe-luxafor" => Some(probe_luxafor()),
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

fn probe_aorus() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_aorus(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact Gigabyte Aorus M2 lighting endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found Gigabyte Aorus M2 endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_aorus_case() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_aorus_case(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact Gigabyte AORUS C300 GLASS lighting endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found Gigabyte AORUS C300 GLASS endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_clevo() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_clevo(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact CLEVO Lightbar endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found CLEVO Lightbar endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}, firmware {}",
                endpoint.vendor_id,
                endpoint.product_id,
                endpoint.interface_number,
                endpoint.usage_page,
                endpoint.usage,
                clevo_firmware_version(endpoint.release_number)
            );
        }
    }
    println!("Probe completed without opening a device or writing a report.");
    Ok(())
}

fn probe_skydimo() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_skydimo(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact Skydimo SK0902 endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found Skydimo SK0902 endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_ek() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_ek(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact EK Loop Connect endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found EK Loop Connect endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_luxafor() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_luxafor(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No Luxafor Flag endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found Luxafor Flag endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_areson() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter_map(|endpoint| match_areson(endpoint).map(|model| (endpoint, model)))
        .collect();
    if exact.is_empty() {
        println!("No exact supported Areson mouse endpoint found.");
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

fn probe_redragon() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter_map(|endpoint| match_redragon(endpoint).map(|model| (endpoint, model)))
        .collect();
    if exact.is_empty() {
        println!("No exact supported Redragon mouse endpoint found.");
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

fn probe_dark_project() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_dark_project(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact Dark Project KD3B V2 lighting endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found Dark Project KD3B V2 endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_stream_deck() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_stream_deck(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact Elgato Stream Deck MK.2 endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found Elgato Stream Deck MK.2 endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_sayo() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_sayo(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact SayoDevice E1 endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found SayoDevice E1 endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_wushi() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_wushi(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact JSAUX RGB Docking Station endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found JSAUX RGB Docking Station endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_aoc() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_aoc(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact AOC AGON AMM700 lighting endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found AOC AGON AMM700 endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_aoc_mouse() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_aoc_mouse(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact AOC GM500 lighting endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found AOC GM500 endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_anne_pro_2() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter_map(|endpoint| match_anne_pro_2(endpoint).map(|model| (endpoint, model)))
        .collect();
    if exact.is_empty() {
        println!("No exact Anne Pro 2 lighting endpoint found.");
    } else {
        for (endpoint, _) in exact {
            println!(
                "Found Anne Pro 2 endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_instant_mouse() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter_map(|endpoint| match_instant_mouse(endpoint).map(|model| (endpoint, model)))
        .collect();
    if exact.is_empty() {
        println!("No exact supported Instant mouse lighting endpoint found.");
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

fn probe_ionico() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter_map(|endpoint| match_ionico(endpoint).map(|model| (endpoint, model)))
        .collect();
    if exact.is_empty() {
        println!("No exact Ionico keyboard or front-bar endpoint found.");
    } else {
        for (endpoint, model) in exact {
            println!(
                "Found {} endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
                model.name(),
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

fn probe_xpg_summoner() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_xpg_summoner(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact XPG Summoner lighting endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found XPG Summoner endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_glorious() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_glorious(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact Glorious Model I lighting endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found Glorious Model I endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}, firmware {}",
                endpoint.vendor_id,
                endpoint.product_id,
                endpoint.interface_number,
                endpoint.usage_page,
                endpoint.usage,
                glorious_firmware_version(endpoint.release_number)
            );
        }
    }
    println!("Probe completed without opening a device or writing a report.");
    Ok(())
}

fn probe_hyte() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_hyte(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact HYTE Keeb TKL lighting endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found HYTE Keeb TKL endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_intel_arc() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_intel_arc(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact Intel Arc A770 Limited Edition lighting endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found Intel Arc A770 Limited Edition endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_skyloong() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_skyloong(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact Skyloong GK104 Pro lighting endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found Skyloong GK104 Pro endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_lego() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_lego(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No Lego Dimensions Toypad Base endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found Lego Dimensions Toypad Base endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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
    if dispatch_structured_write(args)? {
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

fn dispatch_structured_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    if dispatch_keyrox_write(args)? {
        return Ok(true);
    }
    if dispatch_poseidon_write(args)? {
        return Ok(true);
    }
    if dispatch_ionico_write(args)? {
        return Ok(true);
    }
    if dispatch_skyloong_write(args)? {
        return Ok(true);
    }
    if dispatch_glorious_write(args)? {
        return Ok(true);
    }
    if dispatch_instant_mouse_write(args)? {
        return Ok(true);
    }
    if dispatch_aoc_mouse_write(args)? {
        return Ok(true);
    }
    if dispatch_luxafor_write(args)? {
        return Ok(true);
    }
    if dispatch_wushi_write(args)? {
        return Ok(true);
    }
    if dispatch_sayo_write(args)? {
        return Ok(true);
    }
    if dispatch_areson_write(args)? {
        return Ok(true);
    }
    if dispatch_redragon_write(args)? {
        return Ok(true);
    }
    if dispatch_aorus_case_write(args)? {
        return Ok(true);
    }
    if dispatch_clevo_write(args)? {
        return Ok(true);
    }
    if dispatch_ek_write(args)? {
        return Ok(true);
    }
    if dispatch_variable_write(args)? {
        return Ok(true);
    }
    Ok(false)
}

fn dispatch_keyrox_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    match args {
        [command, confirmation, brightness, colors @ ..]
            if command == "set-keyrox-custom" && confirmation == "--confirm-reversible-write" =>
        {
            set_keyrox(KeyroxCommand::Custom {
                brightness: parse_u8_decimal(brightness, "brightness")?,
                colors: parse_rgb_colors(colors)?,
            })?;
            Ok(true)
        }
        [
            command,
            confirmation,
            mode,
            brightness,
            speed,
            direction,
            color,
        ] if command == "set-keyrox-mode" && confirmation == "--confirm-persistent-write" => {
            set_keyrox(KeyroxCommand::HardwareMode {
                mode: parse_keyrox_mode(mode)?,
                brightness: parse_u8_decimal(brightness, "brightness")?,
                speed: parse_u8_decimal(speed, "speed")?,
                direction: parse_keyrox_direction(direction)?,
                color: parse_keyrox_color(color)?,
            })?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn dispatch_poseidon_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    match args {
        [command, confirmation, colors @ ..]
            if command == "set-poseidon-direct" && confirmation == "--confirm-reversible-write" =>
        {
            set_poseidon(PoseidonCommand::Direct {
                colors: parse_rgb_colors(colors)?,
            })?;
            Ok(true)
        }
        [command, confirmation, mode, direction, speed, colors @ ..]
            if command == "set-poseidon-profile"
                && confirmation == "--confirm-persistent-write" =>
        {
            set_poseidon(PoseidonCommand::Profile {
                mode: parse_poseidon_mode(mode)?,
                direction: parse_poseidon_direction(direction)?,
                speed: parse_u8_decimal(speed, "speed")?,
                colors: parse_rgb_colors(colors)?,
            })?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn dispatch_ionico_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    match args {
        [command, confirmation, model]
            if command == "save-ionico" && confirmation == "--confirm-persistent-write" =>
        {
            set_ionico(parse_ionico_model(model)?, IonicoCommand::Save)?;
        }
        [
            command,
            confirmation,
            model,
            mode,
            brightness,
            speed,
            colors @ ..,
        ] if command == "set-ionico" && confirmation == "--confirm-reversible-write" => {
            set_ionico(
                parse_ionico_model(model)?,
                IonicoCommand::Mode {
                    mode: parse_ionico_mode(mode)?,
                    colors: parse_rgb_colors(colors)?,
                    brightness: parse_u8_decimal(brightness, "brightness")?,
                    speed: parse_u8_decimal(speed, "speed")?,
                },
            )?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn dispatch_skyloong_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    let [command, confirmation, brightness, colors @ ..] = args else {
        return Ok(false);
    };
    if command != "set-skyloong-gk104-pro" || confirmation != "--confirm-persistent-write" {
        return Ok(false);
    }
    set_skyloong_colors(
        parse_u8_decimal(brightness, "brightness")?,
        &parse_rgb_colors(colors)?,
    )?;
    Ok(true)
}

fn dispatch_glorious_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    let [command, confirmation, mode, color, brightness, speed] = args else {
        return Ok(false);
    };
    if command != "set-glorious-model-i" || confirmation != "--confirm-reversible-write" {
        return Ok(false);
    }
    set_glorious_mode(GloriousCommand {
        mode: parse_glorious_mode(mode)?,
        color: parse_rgb(color)?,
        brightness: parse_u8_decimal(brightness, "brightness")?,
        speed: parse_u8_decimal(speed, "speed")?,
    })?;
    Ok(true)
}

fn dispatch_instant_mouse_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    let [
        command,
        confirmation,
        mode,
        color,
        brightness,
        speed,
        direction,
    ] = args
    else {
        return Ok(false);
    };
    if command != "set-instant-mouse" || confirmation != "--confirm-reversible-write" {
        return Ok(false);
    }
    set_instant_mouse_mode(InstantCommand {
        mode: parse_instant_mode(mode)?,
        color: parse_rgb(color)?,
        brightness: parse_u8_decimal(brightness, "brightness")?,
        speed: parse_u8_decimal(speed, "speed")?,
        direction: parse_instant_direction(direction)?,
    })?;
    Ok(true)
}

fn dispatch_aoc_mouse_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    let [
        command,
        confirmation,
        mode,
        logo,
        wheel,
        brightness,
        speed,
        direction,
    ] = args
    else {
        return Ok(false);
    };
    if command != "set-aoc-gm500" || confirmation != "--confirm-reversible-write" {
        return Ok(false);
    }
    set_aoc_mouse_mode(AocMouseCommand {
        mode: parse_aoc_mouse_mode(mode)?,
        colors: [parse_rgb(logo)?, parse_rgb(wheel)?],
        brightness: parse_u8_decimal(brightness, "brightness")?,
        speed: parse_u8_decimal(speed, "speed")?,
        direction: parse_aoc_mouse_direction(direction)?,
    })?;
    Ok(true)
}

fn dispatch_luxafor_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    match args {
        [command, confirmation, mode, colors @ ..]
            if command == "set-luxafor"
                && confirmation == "--confirm-reversible-write"
                && mode == "direct" =>
        {
            set_luxafor(LuxaforCommand::Direct(parse_rgb_colors(colors)?))?;
        }
        [command, confirmation, mode, pattern]
            if command == "set-luxafor"
                && confirmation == "--confirm-reversible-write"
                && mode == "pattern" =>
        {
            set_luxafor(LuxaforCommand::Pattern(parse_luxafor_pattern(pattern)?))?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn dispatch_wushi_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    let [
        command,
        confirmation,
        mode,
        zone_1,
        zone_2,
        zone_3,
        zone_4,
        brightness,
        speed,
        direction,
    ] = args
    else {
        return Ok(false);
    };
    if command != "set-wushi" || confirmation != "--confirm-reversible-write" {
        return Ok(false);
    }
    set_wushi(WushiCommand {
        mode: parse_wushi_mode(mode)?,
        colors: [
            parse_rgb(zone_1)?,
            parse_rgb(zone_2)?,
            parse_rgb(zone_3)?,
            parse_rgb(zone_4)?,
        ],
        brightness: parse_u8_decimal(brightness, "brightness")?,
        speed: parse_u8_decimal(speed, "speed")?,
        direction: parse_wushi_direction(direction)?,
    })?;
    Ok(true)
}

fn dispatch_sayo_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    match args {
        [command, confirmation]
            if command == "save-sayo" && confirmation == "--confirm-persistent-write" =>
        {
            set_sayo(SayoCommand::Save)?;
        }
        [command, confirmation, mode, color, speed, color_behavior]
            if command == "set-sayo" && confirmation == "--confirm-reversible-write" =>
        {
            set_sayo(SayoCommand::Mode {
                mode: parse_sayo_mode(mode)?,
                color: parse_rgb(color)?,
                speed: parse_u8_decimal(speed, "speed")?,
                random: parse_sayo_random(color_behavior)?,
            })?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn dispatch_areson_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    let [command, confirmation, mode, color, brightness, speed] = args else {
        return Ok(false);
    };
    if command != "set-areson" || confirmation != "--confirm-reversible-write" {
        return Ok(false);
    }
    set_areson_mode(AresonCommand {
        mode: parse_areson_mode(mode)?,
        color: parse_rgb(color)?,
        brightness: parse_u8_decimal(brightness, "brightness")?,
        speed: parse_u8_decimal(speed, "speed")?,
    })?;
    Ok(true)
}

fn dispatch_redragon_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    let [command, confirmation, mode, color] = args else {
        return Ok(false);
    };
    if command != "set-redragon" || confirmation != "--confirm-reversible-write" {
        return Ok(false);
    }
    set_redragon_mode(RedragonCommand {
        mode: parse_redragon_mode(mode)?,
        color: parse_rgb(color)?,
    })?;
    Ok(true)
}

fn dispatch_aorus_case_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    let [command, confirmation, mode, color, brightness, speed] = args else {
        return Ok(false);
    };
    if command != "set-aorus-case" || confirmation != "--confirm-reversible-write" {
        return Ok(false);
    }
    set_aorus_case_mode(AorusCaseCommand {
        mode: parse_aorus_case_mode(mode)?,
        color: parse_rgb(color)?,
        brightness: parse_u8_decimal(brightness, "brightness")?,
        speed: parse_u8_decimal(speed, "speed")?,
    })?;
    Ok(true)
}

fn dispatch_clevo_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    let [command, confirmation, mode, color, brightness, speed] = args else {
        return Ok(false);
    };
    if command != "set-clevo-lightbar" || confirmation != "--confirm-reversible-write" {
        return Ok(false);
    }
    set_clevo_mode(ClevoCommand {
        mode: parse_clevo_mode(mode)?,
        color: parse_rgb(color)?,
        brightness: parse_u8_decimal(brightness, "brightness")?,
        speed: parse_u8_decimal(speed, "speed")?,
    })?;
    Ok(true)
}

fn dispatch_ek_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    let [command, confirmation, mode, color, speed] = args else {
        return Ok(false);
    };
    if command != "set-ek-loop-connect" || confirmation != "--confirm-reversible-write" {
        return Ok(false);
    }
    set_ek_mode(EkCommand {
        mode: parse_ek_mode(mode)?,
        color: parse_rgb(color)?,
        speed: parse_u8_decimal(speed, "speed")?,
    })?;
    Ok(true)
}

fn dispatch_variable_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    if dispatch_per_led_write(args)? {
        return Ok(true);
    }

    match args {
        [command, confirmation, mode, color, brightness, speed]
            if command == "set-aorus-m2" && confirmation == "--confirm-reversible-write" =>
        {
            set_aorus_mode(AorusCommand {
                mode: parse_aorus_mode(mode)?,
                color: parse_rgb(color)?,
                brightness: parse_u8_decimal(brightness, "brightness")?,
                speed: parse_u8_decimal(speed, "speed")?,
            })?;
            Ok(true)
        }
        [
            command,
            confirmation,
            mode,
            color,
            brightness,
            speed,
            direction,
        ] if command == "set-aoc-amm700" && confirmation == "--confirm-reversible-write" => {
            set_aoc_mode(AocCommand {
                mode: parse_aoc_mode(mode)?,
                color: parse_rgb(color)?,
                brightness: parse_u8_decimal(brightness, "brightness")?,
                speed: parse_u8_decimal(speed, "speed")?,
                direction: parse_aoc_direction(direction)?,
            })?;
            Ok(true)
        }
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
        [command, confirmation, mode, center, left, right]
            if command == "set-lego-toypad"
                && confirmation == "--confirm-reversible-write"
                && mode == "direct" =>
        {
            set_lego_command(LegoCommand::Direct([
                parse_rgb(center)?,
                parse_rgb(left)?,
                parse_rgb(right)?,
            ]))?;
            Ok(true)
        }
        [command, confirmation, mode, color, speed]
            if command == "set-lego-toypad" && confirmation == "--confirm-reversible-write" =>
        {
            set_lego_command(LegoCommand::Effect {
                mode: parse_lego_mode(mode)?,
                color: parse_rgb(color)?,
                speed: parse_u8_decimal(speed, "speed")?,
            })?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn dispatch_per_led_write(args: &[String]) -> Result<bool, Box<dyn Error>> {
    let [command, confirmation, colors @ ..] = args else {
        return Ok(false);
    };
    if confirmation != "--confirm-reversible-write" {
        return Ok(false);
    }

    if !matches!(
        command.as_str(),
        "set-anne-pro-2"
            | "set-asus-monitor"
            | "set-hyperx-mousemat"
            | "set-dark-project"
            | "set-stream-deck"
            | "set-skydimo"
            | "set-hyte-keeb-tkl"
            | "set-intel-arc-a770-le"
            | "set-xpg-summoner"
            | "set-ducky"
            | "set-valkyrie-vk99-pro"
            | "set-valkyrie-vk99"
            | "set-msi-laptop-keyboard"
            | "set-msi-laptop-lightbar"
    ) {
        return Ok(false);
    }
    let colors = parse_rgb_colors(colors)?;
    match command.as_str() {
        "set-anne-pro-2" => set_anne_pro_2_colors(&colors)?,
        "set-asus-monitor" => set_asus_monitor_colors(colors)?,
        "set-hyperx-mousemat" => set_hyperx_mousemat_colors(colors)?,
        "set-dark-project" => set_dark_project_colors(colors)?,
        "set-stream-deck" => set_stream_deck_colors(colors)?,
        "set-skydimo" => set_skydimo_colors(colors)?,
        "set-hyte-keeb-tkl" => set_hyte_colors(&colors)?,
        "set-intel-arc-a770-le" => set_intel_arc_colors(&colors)?,
        "set-xpg-summoner" => set_xpg_summoner_colors(&colors)?,
        "set-ducky" => set_ducky_colors(&colors)?,
        "set-valkyrie-vk99-pro" => {
            set_valkyrie_colors(ValkyrieModel::Vk99Pro, &colors)?;
        }
        "set-valkyrie-vk99" => set_valkyrie_colors(ValkyrieModel::Vk99, &colors)?,
        "set-msi-laptop-keyboard" => {
            set_msi_laptop_colors(MsiLaptopDevice::Keyboard, &colors)?;
        }
        "set-msi-laptop-lightbar" => {
            set_msi_laptop_colors(MsiLaptopDevice::Lightbar, &colors)?;
        }
        _ => unreachable!("command was checked above"),
    }
    Ok(true)
}

fn print_usage() {
    eprintln!(
        "Usage:\n  openrustygb probe-aoc-amm700\n  openrustygb probe-aoc-gm500\n  openrustygb probe-aorus-m2\n  openrustygb probe-asus-monitor\n  openrustygb probe-haste2\n  \
         openrustygb probe-aorus-case\n  \
         openrustygb probe-clevo-lightbar\n  \
         openrustygb probe-areson\n  \
         openrustygb probe-redragon\n  \
         openrustygb probe-dream-cheeky\n  \
         openrustygb probe-ek-loop-connect\n  \
         openrustygb probe-dark-project\n  \
         openrustygb probe-stream-deck\n  \
         openrustygb probe-sayo\n  \
         openrustygb probe-skydimo\n  \
         openrustygb probe-wushi\n  \
         openrustygb probe-gamesir\n  \
         openrustygb probe-glorious-model-i\n  \
         openrustygb probe-hyte-keeb-tkl\n  \
         openrustygb probe-intel-arc-a770-le\n  \
         openrustygb probe-hyperx-mousemat\n  \
         openrustygb probe-instant-mouse\n  \
         openrustygb probe-lego-toypad\n  \
         openrustygb probe-luxafor\n  \
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
         openrustygb set-aoc-amm700 --confirm-reversible-write \
         <static|spectrum|breathing|breathing-random|flashing|flashing-random|wave|rainbow-wave> \
         RRGGBB <brightness> <speed> <cw|ccw>\n  \
         openrustygb set-aoc-gm500 --confirm-reversible-write \
         <direct|spectrum|breathing|breathing-random|flashing|flashing-random|wave|rainbow-wave|dpi> \
         <LOGO-RRGGBB> <WHEEL-RRGGBB> <brightness> <speed> <cw|ccw>\n  \
         openrustygb set-aorus-m2 --confirm-reversible-write \
         <direct|static|breathing|spectrum|flashing|double-flash|off> \
         RRGGBB <brightness> <speed>\n  \
         openrustygb set-aorus-case --confirm-reversible-write \
         <custom|off|breathing|spectrum|flashing|double-flashing> \
         RRGGBB <brightness> <speed>\n  \
         openrustygb set-clevo-lightbar --confirm-reversible-write \
         <direct|breathing|wave|bounce|marquee|scan|off> \
         RRGGBB <brightness> <speed>\n  \
         openrustygb set-areson --confirm-reversible-write \
         <static|rainbow-wave|breathing|spectrum|single-color-wave|colorful-breathing|off> \
         RRGGBB <brightness> <speed>\n  \
         openrustygb set-redragon --confirm-reversible-write \
         <static|wave|breathing|breathing-random|rainbow|flashing> RRGGBB\n  \
         openrustygb set-haste2-color --confirm-reversible-write RRGGBB\n  \
         openrustygb set-hyperx-mousemat --confirm-reversible-write <RRGGBB...>\n  \
         openrustygb set-hyte-keeb-tkl --confirm-reversible-write <98-key-RRGGBB-colors> <63-underglow-RRGGBB-colors>\n  \
         openrustygb set-intel-arc-a770-le --confirm-reversible-write <91-RRGGBB-colors>\n  \
         openrustygb set-instant-mouse --confirm-reversible-write \
         <direct|rainbow-wave|spectrum|breathing|fill|loop|enraptured|flicker|ripple|star-treck|off> \
         RRGGBB <brightness> <speed> <left|right>\n  \
         openrustygb set-lego-toypad --confirm-reversible-write direct \
         <CENTER> <LEFT> <RIGHT>\n  \
         openrustygb set-lego-toypad --confirm-reversible-write \
         <flash|fade> RRGGBB <speed>\n  \
         openrustygb set-luxafor --confirm-reversible-write direct <6-RRGGBB-colors>\n  \
         openrustygb set-luxafor --confirm-reversible-write pattern \
         <traffic-lights|2|3|4|police|6|7|8>\n  \
         openrustygb set-dream-cheeky-color --confirm-reversible-write RRGGBB\n  \
         openrustygb set-ek-loop-connect --confirm-reversible-write \
         <static|breathing|fading|marquee|covering-marquee|pulse|spectrum-wave|alternating|candle> \
         RRGGBB <speed>\n  \
         openrustygb set-dark-project --confirm-reversible-write <87-RRGGBB-colors>\n  \
         openrustygb set-stream-deck --confirm-reversible-write <15-RRGGBB-colors>\n  \
         openrustygb set-skydimo --confirm-reversible-write <49-RRGGBB-colors>\n  \
         openrustygb set-sayo --confirm-reversible-write \
         <direct|breathing|wave|switch|blink> RRGGBB <speed> <static|random>\n  \
         openrustygb save-sayo --confirm-persistent-write\n  \
         openrustygb set-wushi --confirm-reversible-write \
         <direct|breathing|rainbow-wave|spectrum|race|stacking> \
         <ZONE1> <ZONE2> <ZONE3> <ZONE4> <brightness> <speed> <left|right>\n  \
         openrustygb set-gamesir-color --confirm-reversible-write RRGGBB\n  \
         openrustygb set-glorious-model-i --confirm-reversible-write \
         <custom|flashing|chase|wave|spectrum|breathing|spectrum-breathing|rainbow-wave|off> \
         RRGGBB <brightness> <speed>\n  \
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

fn print_skyloong_usage() {
    eprintln!(
        "  openrustygb probe-skyloong-gk104-pro\n  \
         openrustygb set-skyloong-gk104-pro --confirm-persistent-write <brightness-0-127> <106-RRGGBB-colors>"
    );
}

fn print_anne_pro_2_usage() {
    eprintln!(
        "  openrustygb probe-anne-pro-2\n  \
         openrustygb set-anne-pro-2 --confirm-reversible-write <61-RRGGBB-colors>"
    );
}

fn print_ionico_usage() {
    eprintln!(
        "  openrustygb probe-ionico\n  \
         openrustygb set-ionico --confirm-reversible-write <keyboard|front-bar> \
         <direct|breathing|wave|raindrops|flashing|off> <brightness-0-50> <speed-0-10> <RRGGBB...>\n  \
         openrustygb save-ionico --confirm-persistent-write <keyboard|front-bar>"
    );
}

fn print_xpg_summoner_usage() {
    eprintln!(
        "  openrustygb probe-xpg-summoner\n  \
         openrustygb set-xpg-summoner --confirm-reversible-write <104-RRGGBB-colors>"
    );
}

fn print_ducky_usage() {
    eprintln!(
        "  openrustygb probe-ducky\n  \
         openrustygb set-ducky --confirm-reversible-write <108-or-132-RRGGBB-colors>"
    );
}

fn print_poseidon_usage() {
    eprintln!(
        "  openrustygb probe-poseidon-z-rgb\n  \
         openrustygb set-poseidon-direct --confirm-reversible-write <104-RRGGBB-colors>\n  \
         openrustygb set-poseidon-profile --confirm-persistent-write \
         <static|wave|ripple|reactive> <left|right> <speed-5-16> <104-RRGGBB-colors>"
    );
}

fn print_keyrox_usage() {
    eprintln!(
        "  openrustygb probe-keyrox\n  \
         openrustygb set-keyrox-custom --confirm-reversible-write \
         <brightness-0-255> <87-RRGGBB-colors>\n  \
         openrustygb set-keyrox-mode --confirm-persistent-write \
         <wave|const|breathe|heartrate|point|winnower|stars|spectrum|plumflower|shoot|ambilight-rotate|ripple> \
         <brightness-0-127> <speed-0-4> <left|right|up|down> <none|random|RRGGBB>"
    );
}

fn print_valkyrie_usage() {
    eprintln!(
        "  openrustygb probe-valkyrie-vk99\n  \
         openrustygb set-valkyrie-vk99-pro --confirm-reversible-write <98-RRGGBB-colors>\n  \
         openrustygb set-valkyrie-vk99 --confirm-reversible-write <102-RRGGBB-colors>"
    );
}

fn print_msi_laptop_usage() {
    eprintln!(
        "  openrustygb probe-msi-laptop\n  \
         openrustygb set-msi-laptop-keyboard --confirm-reversible-write <102-RRGGBB-colors>\n  \
         openrustygb set-msi-laptop-lightbar --confirm-reversible-write <4-RRGGBB-colors>"
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

fn probe_ducky() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter_map(|endpoint| match_ducky(endpoint).map(|model| (endpoint, model)))
        .collect();
    if exact.is_empty() {
        println!("No exact supported Ducky keyboard endpoint found.");
    } else {
        for (endpoint, model) in exact {
            println!(
                "Found {} endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
                model.name(),
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

fn probe_poseidon() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| matches_poseidon(endpoint))
        .collect();
    if exact.is_empty() {
        println!("No exact Thermaltake Poseidon Z RGB endpoint found.");
    } else {
        for endpoint in exact {
            println!(
                "Found Thermaltake Poseidon Z RGB endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_keyrox() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter_map(|endpoint| match_keyrox(endpoint).map(|model| (endpoint, model)))
        .collect();
    if exact.is_empty() {
        println!("No exact Red Square Keyrox TKL endpoint found.");
    } else {
        for (endpoint, model) in exact {
            println!(
                "Found {model} endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
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

fn probe_valkyrie() -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter_map(|endpoint| match_valkyrie(endpoint).map(|model| (endpoint, model)))
        .collect();
    if exact.is_empty() {
        println!("No exact Valkyrie VK99 endpoint found.");
    } else {
        for (endpoint, model) in exact {
            println!(
                "Found {} endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
                model.name(),
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

fn probe_msi_laptop() -> Result<(), Box<dyn Error>> {
    let system = detect_system_identity();
    let endpoints = HidInventory::enumerate()?;
    let exact: Vec<_> = endpoints
        .iter()
        .filter_map(|endpoint| match_msi_laptop(endpoint, &system).map(|device| (endpoint, device)))
        .collect();
    if exact.is_empty() {
        println!(
            "No exact MSI Raider A18 laptop lighting endpoint found for system '{}' '{}'.",
            system.manufacturer, system.product_name
        );
    } else {
        for (endpoint, device) in exact {
            println!(
                "Found {} endpoint: {:04X}:{:04X}, interface {}, usage {:04X}:{:04X}",
                device.name(),
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

#[cfg(target_os = "windows")]
fn detect_system_identity() -> SystemIdentity {
    SystemIdentity {
        manufacturer: read_windows_bios_value("SystemManufacturer").unwrap_or_default(),
        product_name: read_windows_bios_value("SystemProductName").unwrap_or_default(),
    }
}

#[cfg(target_os = "windows")]
fn read_windows_bios_value(name: &str) -> Option<String> {
    let output = std::process::Command::new("reg.exe")
        .args([
            "query",
            r"HKLM\HARDWARE\DESCRIPTION\System\BIOS",
            "/v",
            name,
        ])
        .output()
        .ok()?;
    output.status.success().then_some(())?;
    parse_windows_registry_string(&String::from_utf8_lossy(&output.stdout), name)
}

#[cfg(target_os = "windows")]
fn parse_windows_registry_string(output: &str, name: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let line = line.trim();
        let rest = line.strip_prefix(name)?.trim_start();
        rest.strip_prefix("REG_SZ")
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    })
}

#[cfg(target_os = "linux")]
fn detect_system_identity() -> SystemIdentity {
    SystemIdentity {
        manufacturer: read_linux_dmi_value("sys_vendor"),
        product_name: read_linux_dmi_value("product_name"),
    }
}

#[cfg(target_os = "linux")]
fn read_linux_dmi_value(name: &str) -> String {
    std::fs::read_to_string(format!("/sys/class/dmi/id/{name}"))
        .unwrap_or_default()
        .trim()
        .to_owned()
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn detect_system_identity() -> SystemIdentity {
    SystemIdentity {
        manufacturer: String::new(),
        product_name: String::new(),
    }
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

fn set_lego_command(command: LegoCommand) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_lego);
    let endpoint = exact
        .next()
        .ok_or("Lego Dimensions Toypad Base endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one Lego Toy Pad endpoint found; refusing to choose".into());
    }
    let mut output = HidOutput::<LEGO_REPORT_LEN>::open_matching(&endpoint, LEGO_MATCH)?;
    LegoActivation::new().apply(&mut output)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(15).expect("fifteen is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, LegoBackend { output }, 4)?;
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Lego command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible Lego Dimensions Toy Pad lighting transaction.");
    Ok(())
}

fn set_luxafor(command: LuxaforCommand) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_luxafor);
    let endpoint = exact.next().ok_or("Luxafor Flag endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one Luxafor Flag endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<LUXAFOR_REPORT_LEN>::open_matching(&endpoint, LUXAFOR_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(28).expect("twenty-eight is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, LuxaforBackend { output }, 4)?;
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Luxafor command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible Luxafor Flag lighting transaction.");
    Ok(())
}

fn set_aoc_mode(command: AocCommand) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_aoc);
    let endpoint = exact
        .next()
        .ok_or("exact AOC AGON AMM700 lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one AOC AMM700 endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<AOC_REPORT_LEN>::open_matching(&endpoint, AOC_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(16).expect("sixteen is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, AocBackend { output }, 4)?;
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("AOC mode command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible AOC AGON AMM700 mode transaction.");
    Ok(())
}

fn set_aoc_mouse_mode(command: AocMouseCommand) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_aoc_mouse);
    let endpoint = exact
        .next()
        .ok_or("exact AOC GM500 lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one AOC GM500 endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<AOC_MOUSE_REPORT_LEN>::open_matching(&endpoint, AOC_MOUSE_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(29).expect("twenty-nine is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, AocMouseBackend { output }, 4)?;
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("AOC GM500 mode command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible AOC GM500 mode transaction.");
    Ok(())
}

fn set_instant_mouse_mode(command: InstantCommand) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter_map(|endpoint| {
        let model = match_instant_mouse(&endpoint)?;
        Some((endpoint, model))
    });
    let (endpoint, model) = exact
        .next()
        .ok_or("exact supported Instant mouse lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one Instant mouse endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<INSTANT_REPORT_LEN>::open_matching(&endpoint, model.matcher)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(30).expect("thirty is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, InstantBackend { model, output }, 4)?;
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Instant mouse mode command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible Instant mouse lighting transaction.");
    Ok(())
}

fn set_glorious_mode(command: GloriousCommand) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_glorious);
    let endpoint = exact
        .next()
        .ok_or("exact Glorious Model I lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one Glorious Model I endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<GLORIOUS_REPORT_LEN>::open_matching(&endpoint, GLORIOUS_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(31).expect("thirty-one is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, GloriousBackend { output }, 4)?;
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Glorious Model I mode command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible Glorious Model I mode transaction.");
    Ok(())
}

fn set_hyte_colors(colors: &[Rgb8]) -> Result<(), Box<dyn Error>> {
    let expected = HYTE_KEY_LED_COUNT + HYTE_UNDERGLOW_LED_COUNT;
    if colors.len() != expected {
        return Err(format!(
            "HYTE Keeb TKL requires {expected} colors: {HYTE_KEY_LED_COUNT} keyboard colors followed by {HYTE_UNDERGLOW_LED_COUNT} underglow colors; got {}",
            colors.len()
        )
        .into());
    }
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_hyte);
    let endpoint = exact
        .next()
        .ok_or("exact HYTE Keeb TKL lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one HYTE Keeb TKL endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<HYTE_REPORT_LEN>::open_matching(&endpoint, HYTE_MATCH)?;
    let (keyboard, underglow) = colors.split_at(HYTE_KEY_LED_COUNT);
    let command = HyteCommand {
        keyboard: keyboard.to_vec(),
        underglow: underglow.to_vec(),
    };
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(32).expect("thirty-two is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, HyteBackend { output }, 4)?;
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("HYTE Keeb TKL color command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible HYTE Keeb TKL per-LED transaction.");
    Ok(())
}

fn set_intel_arc_colors(colors: &[Rgb8]) -> Result<(), Box<dyn Error>> {
    if colors.len() != INTEL_ARC_LED_COUNT {
        return Err(format!(
            "Intel Arc A770 Limited Edition requires {INTEL_ARC_LED_COUNT} colors, got {}",
            colors.len()
        )
        .into());
    }
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_intel_arc);
    let endpoint = exact
        .next()
        .ok_or("exact Intel Arc A770 Limited Edition lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err(
            "more than one Intel Arc A770 Limited Edition endpoint found; refusing to choose"
                .into(),
        );
    }
    let output = HidOutput::<INTEL_ARC_REPORT_LEN>::open_matching(&endpoint, INTEL_ARC_MATCH)?;
    let (backend, firmware) = IntelArcBackend::initialize(output)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(33).expect("thirty-three is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, backend, 4)?;
    let outcome = actor
        .submit_barrier(
            target,
            IntelArcCommand {
                colors: colors.to_vec(),
            },
        )?
        .wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Intel Arc color command was unexpectedly superseded".into());
        }
    }
    println!(
        "Applied one reversible Intel Arc A770 Limited Edition per-LED transaction (firmware {firmware})."
    );
    Ok(())
}

fn set_skyloong_colors(brightness: u8, colors: &[Rgb8]) -> Result<(), Box<dyn Error>> {
    SkyloongColorTransaction::new(colors, brightness)?;
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_skyloong);
    let endpoint = exact
        .next()
        .ok_or("exact Skyloong GK104 Pro lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one Skyloong GK104 Pro endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<SKYLOONG_REPORT_LEN>::open_matching(&endpoint, SKYLOONG_MATCH)?;
    let backend = SkyloongBackend::initialize(output)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(34).expect("thirty-four is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, backend, 4)?;
    let outcome = actor
        .submit_barrier(
            target,
            SkyloongCommand {
                brightness,
                colors: colors.to_vec(),
            },
        )?
        .wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Skyloong GK104 Pro color command was unexpectedly superseded".into());
        }
    }
    println!("Applied and persistently saved one Skyloong GK104 Pro per-LED transaction.");
    Ok(())
}

fn set_anne_pro_2_colors(colors: &[Rgb8]) -> Result<(), Box<dyn Error>> {
    AnnePro2ColorTransaction::new(colors)?;
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints
        .into_iter()
        .filter_map(|endpoint| match_anne_pro_2(&endpoint).map(|model| (endpoint, model)));
    let (endpoint, model) = exact
        .next()
        .ok_or("exact Anne Pro 2 lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one Anne Pro 2 endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<ANNE_PRO_2_REPORT_LEN>::open_matching(&endpoint, model.matcher)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(35).expect("thirty-five is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, AnnePro2Backend { output }, 4)?;
    let outcome = actor
        .submit_barrier(
            target,
            AnnePro2Command {
                colors: colors.to_vec(),
            },
        )?
        .wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Anne Pro 2 color command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible Anne Pro 2 per-LED transaction.");
    Ok(())
}

fn set_ionico(model: IonicoModel, command: IonicoCommand) -> Result<(), Box<dyn Error>> {
    if let IonicoCommand::Mode {
        mode,
        colors,
        brightness,
        speed,
    } = &command
    {
        IonicoModeTransaction::new(model, *mode, colors, *brightness, *speed)?;
    }
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints
        .into_iter()
        .filter(|endpoint| model.matcher().matches(endpoint));
    let endpoint = exact
        .next()
        .ok_or("exact selected Ionico endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one selected Ionico endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<IONICO_OUTPUT_REPORT_LEN>::open_matching(&endpoint, model.matcher())?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(36).expect("thirty-six is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, IonicoBackend { model, output }, 4)?;
    let persistent = matches!(command, IonicoCommand::Save);
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Ionico command was unexpectedly superseded".into());
        }
    }
    if persistent {
        println!("Persisted the current selected Ionico state to BIOS.");
    } else {
        println!("Applied one reversible Ionico mode transaction.");
    }
    Ok(())
}

fn set_xpg_summoner_colors(colors: &[Rgb8]) -> Result<(), Box<dyn Error>> {
    XpgSummonerColorTransaction::new(colors)?;
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_xpg_summoner);
    let endpoint = exact
        .next()
        .ok_or("exact XPG Summoner lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one XPG Summoner endpoint found; refusing to choose".into());
    }
    let output =
        HidOutput::<XPG_SUMMONER_REPORT_LEN>::open_matching(&endpoint, XPG_SUMMONER_MATCH)?;
    let backend = XpgSummonerBackend::initialize(output)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(37).expect("thirty-seven is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, backend, 4)?;
    let outcome = actor
        .submit_barrier(
            target,
            XpgSummonerCommand {
                colors: colors.to_vec(),
            },
        )?
        .wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("XPG Summoner color command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible XPG Summoner per-LED transaction.");
    Ok(())
}

fn set_ducky_colors(colors: &[Rgb8]) -> Result<(), Box<dyn Error>> {
    if !matches!(colors.len(), 108 | 132) {
        return Err(format!(
            "Ducky keyboard requires 108 or 132 colors, got {}",
            colors.len()
        )
        .into());
    }
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints
        .into_iter()
        .filter_map(|endpoint| match_ducky(&endpoint).map(|model| (endpoint, model)));
    let (endpoint, model) = exact
        .next()
        .ok_or("exact supported Ducky keyboard endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one supported Ducky endpoint found; refusing to choose".into());
    }
    DuckyColorTransaction::new(model, colors)?;
    let output = HidOutput::<DUCKY_REPORT_LEN>::open_matching(&endpoint, model.matcher())?;
    let backend = DuckyBackend::initialize(model, output)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(38).expect("thirty-eight is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, backend, 4)?;
    let outcome = actor
        .submit_barrier(
            target,
            DuckyCommand {
                colors: colors.to_vec(),
            },
        )?
        .wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Ducky color command was unexpectedly superseded".into());
        }
    }
    println!(
        "Applied one reversible {} per-LED transaction.",
        model.name()
    );
    Ok(())
}

fn set_poseidon(command: PoseidonCommand) -> Result<(), Box<dyn Error>> {
    match &command {
        PoseidonCommand::Direct { colors } => {
            PoseidonDirectTransaction::new(colors)?;
        }
        PoseidonCommand::Profile {
            mode,
            direction,
            speed,
            colors,
        } => {
            PoseidonModeTransaction::new(*mode, *direction, *speed)?;
            PoseidonProfileTransaction::new(*mode, *direction, *speed, colors)?;
        }
    }
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_poseidon);
    let endpoint = exact
        .next()
        .ok_or("exact Thermaltake Poseidon Z RGB endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one Poseidon Z RGB endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<POSEIDON_REPORT_LEN>::open_matching(&endpoint, POSEIDON_MATCH)?;
    let persistent = matches!(command, PoseidonCommand::Profile { .. });
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(39).expect("thirty-nine is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, PoseidonBackend { output }, 4)?;
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Poseidon Z RGB command was unexpectedly superseded".into());
        }
    }
    if persistent {
        println!("Applied and saved one Thermaltake Poseidon Z RGB profile transaction.");
    } else {
        println!("Applied one reversible Thermaltake Poseidon Z RGB direct transaction.");
    }
    Ok(())
}

fn set_keyrox(command: KeyroxCommand) -> Result<(), Box<dyn Error>> {
    match &command {
        KeyroxCommand::Custom { brightness, colors } => {
            KeyroxCustomTransaction::new(colors, *brightness)?;
        }
        KeyroxCommand::HardwareMode {
            mode,
            brightness,
            speed,
            direction,
            color,
        } => {
            KeyroxModeTransaction::new(*mode, *brightness, *speed, *direction, *color)?;
        }
    }
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints
        .into_iter()
        .filter_map(|endpoint| match_keyrox(&endpoint).map(|model| (endpoint, model)));
    let (endpoint, model) = exact
        .next()
        .ok_or("exact Red Square Keyrox TKL endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one Keyrox TKL endpoint found; refusing to choose".into());
    }
    let matcher = if endpoint.product_id == KEYROX_TKL_MATCH.product_id {
        KEYROX_TKL_MATCH
    } else {
        KEYROX_TKL_V2_MATCH
    };
    let output = HidOutput::<KEYROX_REPORT_LEN>::open_matching(&endpoint, matcher)?;
    let persistent = matches!(command, KeyroxCommand::HardwareMode { .. });
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(40).expect("forty is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, KeyroxBackend { output }, 4)?;
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Keyrox command was unexpectedly superseded".into());
        }
    }
    if persistent {
        println!("Applied one guarded {model} hardware-mode transaction.");
    } else {
        println!("Applied one reversible {model} Custom color transaction.");
    }
    Ok(())
}

fn set_valkyrie_colors(
    requested_model: ValkyrieModel,
    colors: &[Rgb8],
) -> Result<(), Box<dyn Error>> {
    ValkyrieColorTransaction::new(requested_model, colors)?;
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints
        .into_iter()
        .filter(|endpoint| match_valkyrie(endpoint).is_some_and(|model| model == requested_model));
    let endpoint = exact
        .next()
        .ok_or("exact requested Valkyrie VK99 endpoint not found")?;
    if exact.next().is_some() {
        return Err(
            "more than one matching Valkyrie VK99 endpoint found; refusing to choose".into(),
        );
    }
    let output =
        HidOutput::<VALKYRIE_REPORT_LEN>::open_matching(&endpoint, requested_model.matcher())?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(41).expect("forty-one is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(
        target,
        ValkyrieBackend {
            model: requested_model,
            output,
        },
        4,
    )?;
    let outcome = actor
        .submit_barrier(
            target,
            ValkyrieCommand {
                colors: colors.to_vec(),
            },
        )?
        .wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Valkyrie color command was unexpectedly superseded".into());
        }
    }
    println!(
        "Applied one reversible {} per-key color transaction.",
        requested_model.name()
    );
    Ok(())
}

fn set_msi_laptop_colors(
    requested_device: MsiLaptopDevice,
    colors: &[Rgb8],
) -> Result<(), Box<dyn Error>> {
    MsiLaptopColorReport::new(requested_device, colors)?;
    let system = detect_system_identity();
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(|endpoint| {
        match_msi_laptop(endpoint, &system).is_some_and(|device| device == requested_device)
    });
    let endpoint = exact
        .next()
        .ok_or("exact requested MSI Raider A18 laptop lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err(
            "more than one matching MSI Raider A18 laptop endpoint found; refusing to choose"
                .into(),
        );
    }
    let output =
        HidOutput::<MSI_LAPTOP_REPORT_LEN>::open_matching(&endpoint, requested_device.matcher())?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(42).expect("forty-two is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(
        target,
        MsiLaptopBackend {
            device: requested_device,
            output,
        },
        4,
    )?;
    let outcome = actor
        .submit_barrier(
            target,
            MsiLaptopCommand {
                colors: colors.to_vec(),
            },
        )?
        .wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("MSI laptop color command was unexpectedly superseded".into());
        }
    }
    println!(
        "Applied one reversible {} per-LED color transaction.",
        requested_device.name()
    );
    Ok(())
}

fn set_aorus_mode(command: AorusCommand) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_aorus);
    let endpoint = exact
        .next()
        .ok_or("exact Gigabyte Aorus M2 lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one Aorus M2 endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<AORUS_REPORT_LEN>::open_matching(&endpoint, AORUS_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(17).expect("seventeen is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, AorusBackend { output }, 4)?;
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Aorus mode command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible Gigabyte Aorus M2 mode transaction.");
    Ok(())
}

fn set_aorus_case_mode(command: AorusCaseCommand) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_aorus_case);
    let endpoint = exact
        .next()
        .ok_or("exact Gigabyte AORUS C300 GLASS lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one AORUS C300 GLASS endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<AORUS_CASE_REPORT_LEN>::open_matching(&endpoint, AORUS_CASE_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(24).expect("twenty-four is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, AorusCaseBackend { output }, 4)?;
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("AORUS C300 GLASS mode command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible Gigabyte AORUS C300 GLASS mode transaction.");
    Ok(())
}

fn set_clevo_mode(command: ClevoCommand) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_clevo);
    let endpoint = exact
        .next()
        .ok_or("exact CLEVO Lightbar endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one CLEVO Lightbar endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<CLEVO_REPORT_LEN>::open_matching(&endpoint, CLEVO_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(25).expect("twenty-five is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, ClevoBackend { output }, 4)?;
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("CLEVO Lightbar mode command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible CLEVO Lightbar mode transaction.");
    Ok(())
}

fn set_areson_mode(command: AresonCommand) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter_map(|endpoint| {
        let model = match_areson(&endpoint)?;
        Some((endpoint, model))
    });
    let (endpoint, model) = exact
        .next()
        .ok_or("exact supported Areson mouse endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one supported Areson endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<ARESON_REPORT_LEN>::open_matching(&endpoint, model.matcher)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(20).expect("twenty is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, AresonBackend { output }, 4)?;
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Areson mode command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible Areson mouse hardware-mode transaction.");
    Ok(())
}

fn set_redragon_mode(command: RedragonCommand) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter_map(|endpoint| {
        let model = match_redragon(&endpoint)?;
        Some((endpoint, model))
    });
    let (endpoint, model) = exact
        .next()
        .ok_or("exact supported Redragon mouse endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one supported Redragon endpoint found; refusing to choose".into());
    }
    let mut output = HidOutput::<REDRAGON_REPORT_LEN>::open_matching(&endpoint, model.matcher)?;
    RedragonInitialization::new().apply(&mut output)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(23).expect("twenty-three is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, RedragonBackend { output }, 4)?;
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Redragon mode command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible Redragon mouse hardware-mode transaction.");
    Ok(())
}

fn set_dark_project_colors(colors: Vec<Rgb8>) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_dark_project);
    let endpoint = exact
        .next()
        .ok_or("exact Dark Project KD3B V2 lighting endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one Dark Project endpoint found; refusing to choose".into());
    }
    let output =
        HidOutput::<DARK_PROJECT_REPORT_LEN>::open_matching(&endpoint, DARK_PROJECT_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(18).expect("eighteen is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, DarkProjectBackend { output }, 4)?;
    let outcome = actor
        .submit_barrier(target, DarkProjectCommand { colors })?
        .wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Dark Project color command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible Dark Project KD3B V2 per-key transaction.");
    Ok(())
}

fn set_stream_deck_colors(colors: Vec<Rgb8>) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_stream_deck);
    let endpoint = exact
        .next()
        .ok_or("exact Elgato Stream Deck MK.2 endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one Stream Deck MK.2 endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<STREAM_DECK_REPORT_LEN>::open_matching(&endpoint, STREAM_DECK_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(19).expect("nineteen is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, StreamDeckBackend { output }, 4)?;
    let outcome = actor
        .submit_barrier(target, StreamDeckCommand { colors })?
        .wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Stream Deck color command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible Elgato Stream Deck MK.2 per-button transaction.");
    Ok(())
}

fn set_skydimo_colors(colors: Vec<Rgb8>) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_skydimo);
    let endpoint = exact
        .next()
        .ok_or("exact Skydimo SK0902 endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one Skydimo SK0902 endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<SKYDIMO_REPORT_LEN>::open_matching(&endpoint, SKYDIMO_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(26).expect("twenty-six is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, SkydimoBackend { output }, 4)?;
    let outcome = actor
        .submit_barrier(target, SkydimoCommand { colors })?
        .wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Skydimo color command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible Skydimo SK0902 matrix transaction.");
    Ok(())
}

fn set_ek_mode(command: EkCommand) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_ek);
    let endpoint = exact
        .next()
        .ok_or("exact EK Loop Connect endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one EK Loop Connect endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<EK_REPORT_LEN>::open_matching(&endpoint, EK_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(27).expect("twenty-seven is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, EkBackend { output }, 4)?;
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("EK Loop Connect mode command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible EK Loop Connect hardware-mode transaction.");
    Ok(())
}

fn set_sayo(command: SayoCommand) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_sayo);
    let endpoint = exact
        .next()
        .ok_or("exact SayoDevice E1 endpoint not found")?;
    if exact.next().is_some() {
        return Err("more than one SayoDevice E1 endpoint found; refusing to choose".into());
    }
    let output = HidOutput::<SAYO_REPORT_LEN>::open_matching(&endpoint, SAYO_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(21).expect("twenty-one is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, SayoBackend { output }, 4)?;
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("SayoDevice command was unexpectedly superseded".into());
        }
    }
    println!("Applied one confirmed SayoDevice E1 transaction.");
    Ok(())
}

fn set_wushi(command: WushiCommand) -> Result<(), Box<dyn Error>> {
    let endpoints = HidInventory::enumerate()?;
    let mut exact = endpoints.into_iter().filter(matches_wushi);
    let endpoint = exact
        .next()
        .ok_or("exact JSAUX RGB Docking Station endpoint not found")?;
    if exact.next().is_some() {
        return Err(
            "more than one JSAUX RGB Docking Station endpoint found; refusing to choose".into(),
        );
    }
    let output = HidOutput::<WUSHI_REPORT_LEN>::open_matching(&endpoint, WUSHI_MATCH)?;
    let target = ControllerRef {
        id: ControllerId::new(NonZeroU64::new(22).expect("twenty-two is non-zero")),
        incarnation: Incarnation::new(NonZeroU32::new(1).expect("one is non-zero")),
    };
    let actor = ControllerActor::start(target, WushiBackend { output }, 4)?;
    let outcome = actor.submit_barrier(target, command)?.wait()?;
    actor.shutdown()?;
    match outcome {
        CommandOutcome::Applied { .. } => {}
        CommandOutcome::Failed { error, .. } => return Err(error.into()),
        CommandOutcome::Superseded { .. } => {
            return Err("Wushi mode command was unexpectedly superseded".into());
        }
    }
    println!("Applied one reversible JSAUX RGB Docking Station transaction.");
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

#[derive(Clone, Debug)]
struct DarkProjectCommand {
    colors: Vec<Rgb8>,
}

#[derive(Debug)]
struct DarkProjectBackend {
    output: HidOutput<DARK_PROJECT_REPORT_LEN>,
}

#[derive(Debug)]
enum DarkProjectBackendError {
    Settings(DarkProjectInvalidColorCount),
    Output(ExactWriteError<HidTransportError>),
}

impl fmt::Display for DarkProjectBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid Dark Project colors: {error}"),
            Self::Output(error) => write!(f, "could not apply Dark Project colors: {error}"),
        }
    }
}

impl Error for DarkProjectBackendError {}

impl ControllerBackend for DarkProjectBackend {
    type Barrier = DarkProjectCommand;
    type Error = DarkProjectBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        DarkProjectColorTransaction::new(&vec![color; DARK_PROJECT_LED_COUNT])
            .map_err(DarkProjectBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(DarkProjectBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        DarkProjectColorTransaction::new(&command.colors)
            .map_err(DarkProjectBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(DarkProjectBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct StreamDeckCommand {
    colors: Vec<Rgb8>,
}

#[derive(Clone, Copy, Debug)]
struct AresonCommand {
    mode: AresonMode,
    color: Rgb8,
    brightness: u8,
    speed: u8,
}

#[derive(Debug)]
struct AresonBackend {
    output: HidOutput<ARESON_REPORT_LEN>,
}

#[derive(Debug)]
enum AresonBackendError {
    Settings(AresonInvalidSettings),
    Output(HidTransportError),
}

impl fmt::Display for AresonBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid Areson settings: {error}"),
            Self::Output(error) => write!(f, "could not apply Areson mode: {error}"),
        }
    }
}

impl Error for AresonBackendError {}

impl ControllerBackend for AresonBackend {
    type Barrier = AresonCommand;
    type Error = AresonBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        AresonModeTransaction::new(AresonMode::Static, color, 10, 1)
            .map_err(AresonBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(AresonBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        AresonModeTransaction::new(
            command.mode,
            command.color,
            command.brightness,
            command.speed,
        )
        .map_err(AresonBackendError::Settings)?
        .apply(&mut self.output)
        .map_err(AresonBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct RedragonCommand {
    mode: RedragonMode,
    color: Rgb8,
}

#[derive(Debug)]
struct RedragonBackend {
    output: HidOutput<REDRAGON_REPORT_LEN>,
}

impl ControllerBackend for RedragonBackend {
    type Barrier = RedragonCommand;
    type Error = HidTransportError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        RedragonModeTransaction::new(RedragonMode::Static, color).apply(&mut self.output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        RedragonModeTransaction::new(command.mode, command.color).apply(&mut self.output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
struct StreamDeckBackend {
    output: HidOutput<STREAM_DECK_REPORT_LEN>,
}

#[derive(Debug)]
enum StreamDeckBackendError {
    Frame(StreamDeckFrameBuildError),
    Output(ExactWriteError<HidTransportError>),
}

impl fmt::Display for StreamDeckBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Frame(error) => write!(f, "invalid Stream Deck frame: {error}"),
            Self::Output(error) => write!(f, "could not apply Stream Deck frame: {error}"),
        }
    }
}

impl Error for StreamDeckBackendError {}

impl ControllerBackend for StreamDeckBackend {
    type Barrier = StreamDeckCommand;
    type Error = StreamDeckBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        StreamDeckFrameTransaction::new(&[color; STREAM_DECK_BUTTON_COUNT])
            .map_err(StreamDeckBackendError::Frame)?
            .apply(&mut self.output)
            .map_err(StreamDeckBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        StreamDeckFrameTransaction::new(&command.colors)
            .map_err(StreamDeckBackendError::Frame)?
            .apply(&mut self.output)
            .map_err(StreamDeckBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct SkydimoCommand {
    colors: Vec<Rgb8>,
}

#[derive(Debug)]
struct SkydimoBackend {
    output: HidOutput<SKYDIMO_REPORT_LEN>,
}

#[derive(Debug)]
enum SkydimoBackendError {
    Settings(SkydimoInvalidColorCount),
    Output(ExactWriteError<HidTransportError>),
}

impl fmt::Display for SkydimoBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid Skydimo frame: {error}"),
            Self::Output(error) => write!(f, "could not apply Skydimo frame: {error}"),
        }
    }
}

impl Error for SkydimoBackendError {}

impl ControllerBackend for SkydimoBackend {
    type Barrier = SkydimoCommand;
    type Error = SkydimoBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        SkydimoFrameTransaction::new(&[color; SKYDIMO_LED_COUNT])
            .map_err(SkydimoBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(SkydimoBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        SkydimoFrameTransaction::new(&command.colors)
            .map_err(SkydimoBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(SkydimoBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct EkCommand {
    mode: EkMode,
    color: Rgb8,
    speed: u8,
}

#[derive(Debug)]
struct EkBackend {
    output: HidOutput<EK_REPORT_LEN>,
}

#[derive(Debug)]
enum EkBackendError {
    Settings(EkInvalidSpeed),
    Output(ExactWriteError<HidTransportError>),
}

impl fmt::Display for EkBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid EK Loop Connect mode: {error}"),
            Self::Output(error) => write!(f, "could not apply EK Loop Connect mode: {error}"),
        }
    }
}

impl Error for EkBackendError {}

impl ControllerBackend for EkBackend {
    type Barrier = EkCommand;
    type Error = EkBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        EkModeTransaction::new(EkMode::Static, color, 0)
            .map_err(EkBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(EkBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        EkModeTransaction::new(command.mode, command.color, command.speed)
            .map_err(EkBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(EkBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
enum SayoCommand {
    Mode {
        mode: SayoMode,
        color: Rgb8,
        speed: u8,
        random: bool,
    },
    Save,
}

#[derive(Debug)]
struct SayoBackend {
    output: HidOutput<SAYO_REPORT_LEN>,
}

#[derive(Debug)]
enum SayoBackendError {
    Settings(SayoInvalidSpeed),
    Transport(SayoApplyError<HidTransportError>),
}

impl fmt::Display for SayoBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid SayoDevice settings: {error}"),
            Self::Transport(error) => write!(f, "could not apply SayoDevice command: {error}"),
        }
    }
}

impl Error for SayoBackendError {}

impl ControllerBackend for SayoBackend {
    type Barrier = SayoCommand;
    type Error = SayoBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        SayoModeTransaction::new(SayoMode::Direct, 0, color, false)
            .map_err(SayoBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(SayoBackendError::Transport)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        match command {
            SayoCommand::Mode {
                mode,
                color,
                speed,
                random,
            } => SayoModeTransaction::new(mode, speed, color, random)
                .map_err(SayoBackendError::Settings)?
                .apply(&mut self.output)
                .map_err(SayoBackendError::Transport),
            SayoCommand::Save => SayoSaveTransaction::default()
                .apply(&mut self.output)
                .map_err(SayoBackendError::Transport),
        }
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct WushiCommand {
    mode: WushiMode,
    colors: [Rgb8; WUSHI_LED_COUNT],
    brightness: u8,
    speed: u8,
    direction: WushiDirection,
}

#[derive(Debug)]
struct WushiBackend {
    output: HidOutput<WUSHI_REPORT_LEN>,
}

#[derive(Debug)]
enum WushiBackendError {
    Settings(WushiInvalidSettings),
    Output(HidTransportError),
}

impl fmt::Display for WushiBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid Wushi settings: {error}"),
            Self::Output(error) => write!(f, "could not apply Wushi mode: {error}"),
        }
    }
}

impl Error for WushiBackendError {}

impl ControllerBackend for WushiBackend {
    type Barrier = WushiCommand;
    type Error = WushiBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        WushiModeTransaction::new(
            WushiMode::Direct,
            [color; WUSHI_LED_COUNT],
            2,
            1,
            WushiDirection::Left,
        )
        .map_err(WushiBackendError::Settings)?
        .apply(&mut self.output)
        .map_err(WushiBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        WushiModeTransaction::new(
            command.mode,
            command.colors,
            command.brightness,
            command.speed,
            command.direction,
        )
        .map_err(WushiBackendError::Settings)?
        .apply(&mut self.output)
        .map_err(WushiBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct AorusCommand {
    mode: AorusMode,
    color: Rgb8,
    brightness: u8,
    speed: u8,
}

#[derive(Debug)]
struct AorusBackend {
    output: HidOutput<AORUS_REPORT_LEN>,
}

#[derive(Debug)]
enum AorusBackendError {
    Settings(AorusInvalidSettings),
    Output(HidTransportError),
}

impl fmt::Display for AorusBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid Aorus M2 mode: {error}"),
            Self::Output(error) => write!(f, "could not apply Aorus M2 mode: {error}"),
        }
    }
}

impl Error for AorusBackendError {}

impl ControllerBackend for AorusBackend {
    type Barrier = AorusCommand;
    type Error = AorusBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        AorusDirectColorTransaction::new(color)
            .apply(&mut self.output)
            .map_err(AorusBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        AorusModeTransaction::new(
            command.mode,
            command.color,
            command.brightness,
            command.speed,
        )
        .map_err(AorusBackendError::Settings)?
        .apply(&mut self.output)
        .map_err(AorusBackendError::Output)?;
        if command.mode == AorusMode::Direct {
            AorusDirectColorTransaction::new(command.color)
                .apply(&mut self.output)
                .map_err(AorusBackendError::Output)?;
        }
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct AorusCaseCommand {
    mode: AorusCaseMode,
    color: Rgb8,
    brightness: u8,
    speed: u8,
}

#[derive(Debug)]
struct AorusCaseBackend {
    output: HidOutput<AORUS_CASE_REPORT_LEN>,
}

#[derive(Debug)]
enum AorusCaseBackendError {
    Settings(AorusCaseInvalidSettings),
    Output(HidTransportError),
}

impl fmt::Display for AorusCaseBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid AORUS case mode: {error}"),
            Self::Output(error) => write!(f, "could not apply AORUS case mode: {error}"),
        }
    }
}

impl Error for AorusCaseBackendError {}

impl ControllerBackend for AorusCaseBackend {
    type Barrier = AorusCaseCommand;
    type Error = AorusCaseBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        AorusCaseModeTransaction::new(AorusCaseMode::Custom, color, 9, 9)
            .map_err(AorusCaseBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(AorusCaseBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        AorusCaseModeTransaction::new(
            command.mode,
            command.color,
            command.brightness,
            command.speed,
        )
        .map_err(AorusCaseBackendError::Settings)?
        .apply(&mut self.output)
        .map_err(AorusCaseBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct ClevoCommand {
    mode: ClevoMode,
    color: Rgb8,
    brightness: u8,
    speed: u8,
}

#[derive(Debug)]
struct ClevoBackend {
    output: HidOutput<CLEVO_REPORT_LEN>,
}

#[derive(Debug)]
enum ClevoBackendError {
    Settings(ClevoInvalidSettings),
    Output(HidTransportError),
}

impl fmt::Display for ClevoBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid CLEVO Lightbar mode: {error}"),
            Self::Output(error) => write!(f, "could not apply CLEVO Lightbar mode: {error}"),
        }
    }
}

impl Error for ClevoBackendError {}

impl ControllerBackend for ClevoBackend {
    type Barrier = ClevoCommand;
    type Error = ClevoBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        ClevoModeTransaction::new(ClevoMode::Direct, color, 100, 0)
            .map_err(ClevoBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(ClevoBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        ClevoModeTransaction::new(
            command.mode,
            command.color,
            command.brightness,
            command.speed,
        )
        .map_err(ClevoBackendError::Settings)?
        .apply(&mut self.output)
        .map_err(ClevoBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct AocCommand {
    mode: AocMode,
    color: Rgb8,
    brightness: u8,
    speed: u8,
    direction: AocDirection,
}

#[derive(Clone, Copy, Debug)]
struct AocMouseCommand {
    mode: AocMouseMode,
    colors: [Rgb8; 2],
    brightness: u8,
    speed: u8,
    direction: AocMouseDirection,
}

#[derive(Clone, Copy, Debug)]
struct InstantCommand {
    mode: InstantMode,
    color: Rgb8,
    brightness: u8,
    speed: u8,
    direction: InstantDirection,
}

#[derive(Clone, Copy, Debug)]
struct GloriousCommand {
    mode: GloriousMode,
    color: Rgb8,
    brightness: u8,
    speed: u8,
}

#[derive(Debug)]
struct GloriousBackend {
    output: HidOutput<GLORIOUS_REPORT_LEN>,
}

#[derive(Debug)]
enum GloriousBackendError {
    Settings(GloriousInvalidSettings),
    Output(HidTransportError),
}

impl fmt::Display for GloriousBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid Glorious Model I mode: {error}"),
            Self::Output(error) => write!(f, "could not apply Glorious Model I mode: {error}"),
        }
    }
}

impl Error for GloriousBackendError {}

impl ControllerBackend for GloriousBackend {
    type Barrier = GloriousCommand;
    type Error = GloriousBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        GloriousModeTransaction::new(GloriousMode::Custom, color, 50, 0)
            .map_err(GloriousBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(GloriousBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        GloriousModeTransaction::new(
            command.mode,
            command.color,
            command.brightness,
            command.speed,
        )
        .map_err(GloriousBackendError::Settings)?
        .apply(&mut self.output)
        .map_err(GloriousBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct HyteCommand {
    keyboard: Vec<Rgb8>,
    underglow: Vec<Rgb8>,
}

#[derive(Debug)]
struct HyteBackend {
    output: HidOutput<HYTE_REPORT_LEN>,
}

#[derive(Debug)]
enum HyteBackendError {
    Settings(HyteInvalidColorCounts),
    Output(HyteApplyError<HidTransportError>),
}

impl fmt::Display for HyteBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid HYTE Keeb TKL colors: {error}"),
            Self::Output(error) => write!(f, "could not apply HYTE Keeb TKL colors: {error}"),
        }
    }
}

impl Error for HyteBackendError {}

impl ControllerBackend for HyteBackend {
    type Barrier = HyteCommand;
    type Error = HyteBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        HyteColorTransaction::new(
            &[color; HYTE_KEY_LED_COUNT],
            &[color; HYTE_UNDERGLOW_LED_COUNT],
        )
        .map_err(HyteBackendError::Settings)?
        .apply(&mut self.output)
        .map_err(HyteBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        HyteColorTransaction::new(&command.keyboard, &command.underglow)
            .map_err(HyteBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(HyteBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct IntelArcCommand {
    colors: Vec<Rgb8>,
}

#[derive(Debug)]
struct IntelArcBackend {
    output: HidOutput<INTEL_ARC_REPORT_LEN>,
}

impl IntelArcBackend {
    fn initialize(
        mut output: HidOutput<INTEL_ARC_REPORT_LEN>,
    ) -> Result<(Self, String), IntelArcExchangeError<HidTransportError>> {
        let firmware = IntelArcFirmwareQuery::new().apply(&mut output)?;
        IntelArcInitialization::new().apply(&mut output)?;
        Ok((Self { output }, firmware))
    }
}

#[derive(Debug)]
enum IntelArcBackendError {
    Settings(IntelArcInvalidColorCount),
    Output(IntelArcExchangeError<HidTransportError>),
}

impl fmt::Display for IntelArcBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid Intel Arc colors: {error}"),
            Self::Output(error) => write!(f, "could not apply Intel Arc colors: {error}"),
        }
    }
}

impl Error for IntelArcBackendError {}

impl ControllerBackend for IntelArcBackend {
    type Barrier = IntelArcCommand;
    type Error = IntelArcBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        IntelArcColorTransaction::new(&[color; INTEL_ARC_LED_COUNT])
            .map_err(IntelArcBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(IntelArcBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        IntelArcColorTransaction::new(&command.colors)
            .map_err(IntelArcBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(IntelArcBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
enum IonicoCommand {
    Mode {
        mode: IonicoMode,
        colors: Vec<Rgb8>,
        brightness: u8,
        speed: u8,
    },
    Save,
}

#[derive(Debug)]
struct IonicoBackend {
    model: IonicoModel,
    output: HidOutput<IONICO_OUTPUT_REPORT_LEN>,
}

#[derive(Debug)]
enum IonicoBackendError {
    Settings(IonicoInvalidSettings),
    Apply(IonicoApplyError<HidTransportError>),
    Save(HidTransportError),
}

impl fmt::Display for IonicoBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid Ionico mode: {error}"),
            Self::Apply(error) => write!(f, "could not apply Ionico mode: {error}"),
            Self::Save(error) => write!(f, "could not persist Ionico state: {error}"),
        }
    }
}

impl Error for IonicoBackendError {}

impl ControllerBackend for IonicoBackend {
    type Barrier = IonicoCommand;
    type Error = IonicoBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        let colors = vec![color; self.model.led_count()];
        IonicoModeTransaction::new(self.model, IonicoMode::Direct, &colors, 50, 0)
            .map_err(IonicoBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(IonicoBackendError::Apply)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        match command {
            IonicoCommand::Mode {
                mode,
                colors,
                brightness,
                speed,
            } => IonicoModeTransaction::new(self.model, mode, &colors, brightness, speed)
                .map_err(IonicoBackendError::Settings)?
                .apply(&mut self.output)
                .map_err(IonicoBackendError::Apply),
            IonicoCommand::Save => IonicoSaveTransaction::new()
                .apply(&mut self.output)
                .map_err(IonicoBackendError::Save),
        }
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct AnnePro2Command {
    colors: Vec<Rgb8>,
}

#[derive(Debug)]
struct AnnePro2Backend {
    output: HidOutput<ANNE_PRO_2_REPORT_LEN>,
}

#[derive(Debug)]
enum AnnePro2BackendError {
    Settings(AnnePro2InvalidColorCount),
    Output(ExactWriteError<HidTransportError>),
}

impl fmt::Display for AnnePro2BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid Anne Pro 2 colors: {error}"),
            Self::Output(error) => write!(f, "could not apply Anne Pro 2 colors: {error}"),
        }
    }
}

impl Error for AnnePro2BackendError {}

impl ControllerBackend for AnnePro2Backend {
    type Barrier = AnnePro2Command;
    type Error = AnnePro2BackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        AnnePro2ColorTransaction::new(&[color; ANNE_PRO_2_LED_COUNT])
            .map_err(AnnePro2BackendError::Settings)?
            .apply(&mut self.output)
            .map_err(AnnePro2BackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        AnnePro2ColorTransaction::new(&command.colors)
            .map_err(AnnePro2BackendError::Settings)?
            .apply(&mut self.output)
            .map_err(AnnePro2BackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct XpgSummonerCommand {
    colors: Vec<Rgb8>,
}

#[derive(Debug)]
struct XpgSummonerBackend {
    output: HidOutput<XPG_SUMMONER_REPORT_LEN>,
}

impl XpgSummonerBackend {
    fn initialize(
        mut output: HidOutput<XPG_SUMMONER_REPORT_LEN>,
    ) -> Result<Self, ExactWriteError<HidTransportError>> {
        XpgSummonerInitialization::new().apply(&mut output)?;
        Ok(Self { output })
    }
}

#[derive(Debug)]
enum XpgSummonerBackendError {
    Settings(XpgSummonerInvalidColorCount),
    Output(ExactWriteError<HidTransportError>),
}

impl fmt::Display for XpgSummonerBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid XPG Summoner colors: {error}"),
            Self::Output(error) => write!(f, "could not communicate with XPG Summoner: {error}"),
        }
    }
}

impl Error for XpgSummonerBackendError {}

impl ControllerBackend for XpgSummonerBackend {
    type Barrier = XpgSummonerCommand;
    type Error = XpgSummonerBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        XpgSummonerColorTransaction::new(&[color; XPG_SUMMONER_LED_COUNT])
            .map_err(XpgSummonerBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(XpgSummonerBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        XpgSummonerColorTransaction::new(&command.colors)
            .map_err(XpgSummonerBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(XpgSummonerBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        XpgSummonerShutdown::new()
            .apply(&mut self.output)
            .map_err(XpgSummonerBackendError::Output)
    }
}

#[derive(Clone, Debug)]
struct DuckyCommand {
    colors: Vec<Rgb8>,
}

#[derive(Debug)]
struct DuckyBackend {
    model: DuckyModel,
    output: HidOutput<DUCKY_REPORT_LEN>,
}

impl DuckyBackend {
    fn initialize(
        model: DuckyModel,
        mut output: HidOutput<DUCKY_REPORT_LEN>,
    ) -> Result<Self, ExactWriteError<HidTransportError>> {
        DuckyInitialization::new().apply(&mut output)?;
        Ok(Self { model, output })
    }
}

#[derive(Debug)]
enum DuckyBackendError {
    Settings(DuckyInvalidColorCount),
    Output(ExactWriteError<HidTransportError>),
}

impl fmt::Display for DuckyBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid Ducky colors: {error}"),
            Self::Output(error) => write!(f, "could not communicate with Ducky keyboard: {error}"),
        }
    }
}

impl Error for DuckyBackendError {}

impl ControllerBackend for DuckyBackend {
    type Barrier = DuckyCommand;
    type Error = DuckyBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        let colors = vec![color; self.model.led_count()];
        DuckyColorTransaction::new(self.model, &colors)
            .map_err(DuckyBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(DuckyBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        DuckyColorTransaction::new(self.model, &command.colors)
            .map_err(DuckyBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(DuckyBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct ValkyrieCommand {
    colors: Vec<Rgb8>,
}

#[derive(Debug)]
struct ValkyrieBackend {
    model: ValkyrieModel,
    output: HidOutput<VALKYRIE_REPORT_LEN>,
}

#[derive(Debug)]
enum ValkyrieBackendError {
    Colors(ValkyrieInvalidColorCount),
    Output(HidTransportError),
}

impl fmt::Display for ValkyrieBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Colors(error) => write!(f, "invalid Valkyrie color count: {error}"),
            Self::Output(error) => {
                write!(f, "could not communicate with Valkyrie keyboard: {error}")
            }
        }
    }
}

impl Error for ValkyrieBackendError {}

impl ControllerBackend for ValkyrieBackend {
    type Barrier = ValkyrieCommand;
    type Error = ValkyrieBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        ValkyrieColorTransaction::new(self.model, &vec![color; self.model.led_count()])
            .map_err(ValkyrieBackendError::Colors)?
            .apply(&mut self.output)
            .map_err(ValkyrieBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        ValkyrieColorTransaction::new(self.model, &command.colors)
            .map_err(ValkyrieBackendError::Colors)?
            .apply(&mut self.output)
            .map_err(ValkyrieBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct MsiLaptopCommand {
    colors: Vec<Rgb8>,
}

#[derive(Debug)]
struct MsiLaptopBackend {
    device: MsiLaptopDevice,
    output: HidOutput<MSI_LAPTOP_REPORT_LEN>,
}

#[derive(Debug)]
enum MsiLaptopBackendError {
    Colors(MsiLaptopInvalidColorCount),
    Output(HidTransportError),
}

impl fmt::Display for MsiLaptopBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Colors(error) => write!(f, "invalid MSI laptop color count: {error}"),
            Self::Output(error) => {
                write!(f, "could not communicate with MSI laptop lighting: {error}")
            }
        }
    }
}

impl Error for MsiLaptopBackendError {}

impl ControllerBackend for MsiLaptopBackend {
    type Barrier = MsiLaptopCommand;
    type Error = MsiLaptopBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        MsiLaptopColorReport::new(self.device, &vec![color; self.device.led_count()])
            .map_err(MsiLaptopBackendError::Colors)?
            .apply(&mut self.output)
            .map_err(MsiLaptopBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        MsiLaptopColorReport::new(self.device, &command.colors)
            .map_err(MsiLaptopBackendError::Colors)?
            .apply(&mut self.output)
            .map_err(MsiLaptopBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
enum KeyroxCommand {
    Custom {
        brightness: u8,
        colors: Vec<Rgb8>,
    },
    HardwareMode {
        mode: KeyroxMode,
        brightness: u8,
        speed: u8,
        direction: KeyroxDirection,
        color: KeyroxModeColor,
    },
}

#[derive(Debug)]
struct KeyroxBackend {
    output: HidOutput<KEYROX_REPORT_LEN>,
}

#[derive(Debug)]
enum KeyroxBackendError {
    Settings(KeyroxInvalidSettings),
    Output(HidTransportError),
}

impl fmt::Display for KeyroxBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid Red Square Keyrox settings: {error}"),
            Self::Output(error) => {
                write!(f, "could not communicate with Red Square Keyrox: {error}")
            }
        }
    }
}

impl Error for KeyroxBackendError {}

impl ControllerBackend for KeyroxBackend {
    type Barrier = KeyroxCommand;
    type Error = KeyroxBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        KeyroxCustomTransaction::new(&[color; KEYROX_LED_COUNT], 255)
            .map_err(KeyroxBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(KeyroxBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        match command {
            KeyroxCommand::Custom { brightness, colors } => {
                KeyroxCustomTransaction::new(&colors, brightness)
                    .map_err(KeyroxBackendError::Settings)?
                    .apply(&mut self.output)
                    .map_err(KeyroxBackendError::Output)
            }
            KeyroxCommand::HardwareMode {
                mode,
                brightness,
                speed,
                direction,
                color,
            } => KeyroxModeTransaction::new(mode, brightness, speed, direction, color)
                .map_err(KeyroxBackendError::Settings)?
                .apply(&mut self.output)
                .map_err(KeyroxBackendError::Output),
        }
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
enum PoseidonCommand {
    Direct {
        colors: Vec<Rgb8>,
    },
    Profile {
        mode: PoseidonMode,
        direction: PoseidonDirection,
        speed: u8,
        colors: Vec<Rgb8>,
    },
}

#[derive(Debug)]
struct PoseidonBackend {
    output: HidOutput<POSEIDON_REPORT_LEN>,
}

#[derive(Debug)]
enum PoseidonBackendError {
    Settings(PoseidonInvalidSettings),
    Output(HidTransportError),
}

impl fmt::Display for PoseidonBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid Poseidon Z RGB settings: {error}"),
            Self::Output(error) => write!(f, "could not communicate with Poseidon Z RGB: {error}"),
        }
    }
}

impl Error for PoseidonBackendError {}

impl ControllerBackend for PoseidonBackend {
    type Barrier = PoseidonCommand;
    type Error = PoseidonBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        PoseidonDirectTransaction::new(&[color; POSEIDON_LED_COUNT])
            .map_err(PoseidonBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(PoseidonBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        match command {
            PoseidonCommand::Direct { colors } => PoseidonDirectTransaction::new(&colors)
                .map_err(PoseidonBackendError::Settings)?
                .apply(&mut self.output)
                .map_err(PoseidonBackendError::Output),
            PoseidonCommand::Profile {
                mode,
                direction,
                speed,
                colors,
            } => {
                PoseidonModeTransaction::new(mode, direction, speed)
                    .map_err(PoseidonBackendError::Settings)?
                    .apply(&mut self.output)
                    .map_err(PoseidonBackendError::Output)?;
                PoseidonProfileTransaction::new(mode, direction, speed, &colors)
                    .map_err(PoseidonBackendError::Settings)?
                    .apply(&mut self.output)
                    .map_err(PoseidonBackendError::Output)
            }
        }
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct SkyloongCommand {
    brightness: u8,
    colors: Vec<Rgb8>,
}

#[derive(Debug)]
struct SkyloongBackend {
    output: HidOutput<SKYLOONG_REPORT_LEN>,
}

impl SkyloongBackend {
    fn initialize(
        mut output: HidOutput<SKYLOONG_REPORT_LEN>,
    ) -> Result<Self, ExactWriteError<HidTransportError>> {
        SkyloongInitialization::new().apply(&mut output)?;
        Ok(Self { output })
    }
}

#[derive(Debug)]
enum SkyloongBackendError {
    Settings(SkyloongInvalidSettings),
    Output(ExactWriteError<HidTransportError>),
}

impl fmt::Display for SkyloongBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid Skyloong colors: {error}"),
            Self::Output(error) => {
                write!(f, "could not communicate with Skyloong keyboard: {error}")
            }
        }
    }
}

impl Error for SkyloongBackendError {}

impl ControllerBackend for SkyloongBackend {
    type Barrier = SkyloongCommand;
    type Error = SkyloongBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        SkyloongColorTransaction::new(&[color; SKYLOONG_LED_COUNT], 127)
            .map_err(SkyloongBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(SkyloongBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        SkyloongColorTransaction::new(&command.colors, command.brightness)
            .map_err(SkyloongBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(SkyloongBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        SkyloongShutdown::new()
            .apply(&mut self.output)
            .map_err(SkyloongBackendError::Output)
    }
}

#[derive(Debug)]
struct InstantBackend {
    model: InstantMouseModel,
    output: HidOutput<INSTANT_REPORT_LEN>,
}

#[derive(Debug)]
enum InstantBackendError {
    Settings(InstantInvalidSettings),
    Output(HidTransportError),
}

impl fmt::Display for InstantBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid Instant mouse mode: {error}"),
            Self::Output(error) => write!(f, "could not apply Instant mouse mode: {error}"),
        }
    }
}

impl Error for InstantBackendError {}

impl ControllerBackend for InstantBackend {
    type Barrier = InstantCommand;
    type Error = InstantBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        InstantModeTransaction::new(
            self.model,
            InstantMode::Direct,
            color,
            0,
            7,
            InstantDirection::Right,
        )
        .map_err(InstantBackendError::Settings)?
        .apply(&mut self.output)
        .map_err(InstantBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        let mode = if self.model.ant_effects && command.mode == InstantMode::Breathing {
            InstantMode::AntBreathing
        } else {
            command.mode
        };
        InstantModeTransaction::new(
            self.model,
            mode,
            command.color,
            command.speed,
            command.brightness,
            command.direction,
        )
        .map_err(InstantBackendError::Settings)?
        .apply(&mut self.output)
        .map_err(InstantBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
struct AocMouseBackend {
    output: HidOutput<AOC_MOUSE_REPORT_LEN>,
}

#[derive(Debug)]
enum AocMouseBackendError {
    Settings(AocMouseInvalidSettings),
    Output(HidTransportError),
}

impl fmt::Display for AocMouseBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid AOC GM500 mode: {error}"),
            Self::Output(error) => write!(f, "could not apply AOC GM500 mode: {error}"),
        }
    }
}

impl Error for AocMouseBackendError {}

impl ControllerBackend for AocMouseBackend {
    type Barrier = AocMouseCommand;
    type Error = AocMouseBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        AocMouseModeTransaction::direct([color; 2])
            .apply(&mut self.output)
            .map_err(AocMouseBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        let transaction = if command.mode == AocMouseMode::Static {
            AocMouseModeTransaction::direct(command.colors)
        } else {
            AocMouseModeTransaction::new(
                command.mode,
                command.colors,
                command.brightness,
                command.speed,
                command.direction,
            )
            .map_err(AocMouseBackendError::Settings)?
        };
        transaction
            .apply(&mut self.output)
            .map_err(AocMouseBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
struct AocBackend {
    output: HidOutput<AOC_REPORT_LEN>,
}

#[derive(Debug)]
enum AocBackendError {
    Settings(AocInvalidSettings),
    Output(HidTransportError),
}

impl fmt::Display for AocBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid AOC mode: {error}"),
            Self::Output(error) => write!(f, "could not apply AOC mode: {error}"),
        }
    }
}

impl Error for AocBackendError {}

impl ControllerBackend for AocBackend {
    type Barrier = AocCommand;
    type Error = AocBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        AocModeTransaction::direct(color)
            .apply(&mut self.output)
            .map_err(AocBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        AocModeTransaction::new(
            command.mode,
            command.color,
            command.brightness,
            command.speed,
            command.direction,
        )
        .map_err(AocBackendError::Settings)?
        .apply(&mut self.output)
        .map_err(AocBackendError::Output)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
enum LegoCommand {
    Direct([Rgb8; 3]),
    Effect {
        mode: ToypadMode,
        color: Rgb8,
        speed: u8,
    },
}

#[derive(Debug)]
struct LegoBackend {
    output: HidOutput<LEGO_REPORT_LEN>,
}

impl ControllerBackend for LegoBackend {
    type Barrier = LegoCommand;
    type Error = ExactWriteError<HidTransportError>;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        LegoDirectColorTransaction::new([color; 3]).apply(&mut self.output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        match command {
            LegoCommand::Direct(colors) => {
                LegoDirectColorTransaction::new(colors).apply(&mut self.output)
            }
            LegoCommand::Effect { mode, color, speed } => {
                LegoModeTransaction::new(mode, speed, color).apply(&mut self.output)
            }
        }
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
enum LuxaforCommand {
    Direct(Vec<Rgb8>),
    Pattern(LuxaforPattern),
}

#[derive(Debug)]
struct LuxaforBackend {
    output: HidOutput<LUXAFOR_REPORT_LEN>,
}

#[derive(Debug)]
enum LuxaforBackendError {
    Settings(LuxaforInvalidColorCount),
    Output(ExactWriteError<HidTransportError>),
}

impl fmt::Display for LuxaforBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(error) => write!(f, "invalid Luxafor colors: {error}"),
            Self::Output(error) => write!(f, "could not apply Luxafor command: {error}"),
        }
    }
}

impl Error for LuxaforBackendError {}

impl ControllerBackend for LuxaforBackend {
    type Barrier = LuxaforCommand;
    type Error = LuxaforBackendError;

    fn apply_whole_color(&mut self, color: Rgb8) -> Result<(), Self::Error> {
        LuxaforDirectTransaction::new(&[color; LUXAFOR_LED_COUNT])
            .map_err(LuxaforBackendError::Settings)?
            .apply(&mut self.output)
            .map_err(LuxaforBackendError::Output)
    }

    fn apply_barrier(&mut self, command: Self::Barrier) -> Result<(), Self::Error> {
        match command {
            LuxaforCommand::Direct(colors) => LuxaforDirectTransaction::new(&colors)
                .map_err(LuxaforBackendError::Settings)?
                .apply(&mut self.output)
                .map_err(LuxaforBackendError::Output),
            LuxaforCommand::Pattern(pattern) => LuxaforPatternTransaction::new(pattern)
                .apply(&mut self.output)
                .map_err(LuxaforBackendError::Output),
        }
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

fn parse_ionico_model(input: &str) -> Result<IonicoModel, Box<dyn Error>> {
    match input {
        "keyboard" => Ok(IonicoModel::Keyboard),
        "front-bar" => Ok(IonicoModel::FrontBar),
        _ => Err(format!("unknown Ionico model: {input}").into()),
    }
}

fn parse_ionico_mode(input: &str) -> Result<IonicoMode, Box<dyn Error>> {
    match input {
        "direct" => Ok(IonicoMode::Direct),
        "breathing" => Ok(IonicoMode::Breathing),
        "wave" => Ok(IonicoMode::Wave),
        "raindrops" => Ok(IonicoMode::Raindrops),
        "flashing" => Ok(IonicoMode::Flashing),
        "off" => Ok(IonicoMode::Off),
        _ => Err(format!("unknown Ionico mode: {input}").into()),
    }
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

fn parse_poseidon_mode(input: &str) -> Result<PoseidonMode, Box<dyn Error>> {
    match input {
        "static" => Ok(PoseidonMode::Static),
        "wave" => Ok(PoseidonMode::Wave),
        "ripple" => Ok(PoseidonMode::Ripple),
        "reactive" => Ok(PoseidonMode::Reactive),
        _ => Err("Poseidon Z RGB mode must be static, wave, ripple, or reactive".into()),
    }
}

fn parse_keyrox_mode(input: &str) -> Result<KeyroxMode, Box<dyn Error>> {
    match input {
        "wave" => Ok(KeyroxMode::Wave),
        "const" => Ok(KeyroxMode::Const),
        "breathe" => Ok(KeyroxMode::Breathe),
        "heartrate" => Ok(KeyroxMode::Heartrate),
        "point" => Ok(KeyroxMode::Point),
        "winnower" => Ok(KeyroxMode::Winnower),
        "stars" => Ok(KeyroxMode::Stars),
        "spectrum" => Ok(KeyroxMode::Spectrum),
        "plumflower" => Ok(KeyroxMode::Plumflower),
        "shoot" => Ok(KeyroxMode::Shoot),
        "ambilight-rotate" => Ok(KeyroxMode::AmbilightRotate),
        "ripple" => Ok(KeyroxMode::Ripple),
        _ => Err("unknown Red Square Keyrox hardware mode".into()),
    }
}

fn parse_keyrox_direction(input: &str) -> Result<KeyroxDirection, Box<dyn Error>> {
    match input {
        "left" => Ok(KeyroxDirection::Left),
        "right" => Ok(KeyroxDirection::Right),
        "up" => Ok(KeyroxDirection::Up),
        "down" => Ok(KeyroxDirection::Down),
        _ => Err("Keyrox direction must be left, right, up, or down".into()),
    }
}

fn parse_keyrox_color(input: &str) -> Result<KeyroxModeColor, Box<dyn Error>> {
    match input {
        "none" => Ok(KeyroxModeColor::None),
        "random" => Ok(KeyroxModeColor::Random),
        value => Ok(KeyroxModeColor::Fixed(parse_rgb(value)?)),
    }
}

fn parse_poseidon_direction(input: &str) -> Result<PoseidonDirection, Box<dyn Error>> {
    match input {
        "left" => Ok(PoseidonDirection::Left),
        "right" => Ok(PoseidonDirection::Right),
        _ => Err("Poseidon Z RGB direction must be left or right".into()),
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

fn parse_lego_mode(input: &str) -> Result<ToypadMode, Box<dyn Error>> {
    match input {
        "flash" => Ok(ToypadMode::Flash),
        "fade" => Ok(ToypadMode::Fade),
        _ => Err("Lego Toy Pad effect must be flash or fade".into()),
    }
}

fn parse_luxafor_pattern(input: &str) -> Result<LuxaforPattern, Box<dyn Error>> {
    match input {
        "traffic-lights" => Ok(LuxaforPattern::TrafficLights),
        "2" => Ok(LuxaforPattern::Pattern2),
        "3" => Ok(LuxaforPattern::Pattern3),
        "4" => Ok(LuxaforPattern::Pattern4),
        "police" => Ok(LuxaforPattern::Police),
        "6" => Ok(LuxaforPattern::Pattern6),
        "7" => Ok(LuxaforPattern::Pattern7),
        "8" => Ok(LuxaforPattern::Pattern8),
        _ => Err("unknown Luxafor pattern".into()),
    }
}

fn parse_aoc_mode(input: &str) -> Result<AocMode, Box<dyn Error>> {
    match input {
        "static" => Ok(AocMode::Static),
        "spectrum" => Ok(AocMode::SpectrumCycle),
        "breathing" => Ok(AocMode::Breathing),
        "breathing-random" => Ok(AocMode::BreathingRandom),
        "flashing" => Ok(AocMode::Flashing),
        "flashing-random" => Ok(AocMode::FlashingRandom),
        "wave" => Ok(AocMode::Wave),
        "rainbow-wave" => Ok(AocMode::RainbowWave),
        _ => Err("unknown AOC AMM700 mode".into()),
    }
}

fn parse_aoc_mouse_mode(input: &str) -> Result<AocMouseMode, Box<dyn Error>> {
    match input {
        "direct" => Ok(AocMouseMode::Static),
        "spectrum" => Ok(AocMouseMode::SpectrumCycle),
        "breathing" => Ok(AocMouseMode::Breathing),
        "breathing-random" => Ok(AocMouseMode::BreathingRandom),
        "flashing" => Ok(AocMouseMode::Flashing),
        "flashing-random" => Ok(AocMouseMode::FlashingRandom),
        "wave" => Ok(AocMouseMode::Wave),
        "rainbow-wave" => Ok(AocMouseMode::RainbowWave),
        "dpi" => Ok(AocMouseMode::Dpi),
        _ => Err("unknown AOC GM500 mode".into()),
    }
}

fn parse_aoc_mouse_direction(input: &str) -> Result<AocMouseDirection, Box<dyn Error>> {
    match input {
        "cw" => Ok(AocMouseDirection::Clockwise),
        "ccw" => Ok(AocMouseDirection::CounterClockwise),
        _ => Err("AOC GM500 direction must be cw or ccw".into()),
    }
}

fn parse_instant_mode(input: &str) -> Result<InstantMode, Box<dyn Error>> {
    match input {
        "direct" => Ok(InstantMode::Direct),
        "rainbow-wave" => Ok(InstantMode::RainbowWave),
        "spectrum" => Ok(InstantMode::SpectrumCycle),
        "breathing" => Ok(InstantMode::Breathing),
        "fill" => Ok(InstantMode::Fill),
        "loop" => Ok(InstantMode::Loop),
        "enraptured" => Ok(InstantMode::Enraptured),
        "flicker" => Ok(InstantMode::Flicker),
        "ripple" => Ok(InstantMode::Ripple),
        "star-treck" => Ok(InstantMode::StarTreck),
        "off" => Ok(InstantMode::Off),
        _ => Err("unknown Instant mouse mode".into()),
    }
}

fn parse_instant_direction(input: &str) -> Result<InstantDirection, Box<dyn Error>> {
    match input {
        "right" => Ok(InstantDirection::Right),
        "left" => Ok(InstantDirection::Left),
        _ => Err("Instant mouse direction must be left or right".into()),
    }
}

fn parse_glorious_mode(input: &str) -> Result<GloriousMode, Box<dyn Error>> {
    match input {
        "custom" => Ok(GloriousMode::Custom),
        "flashing" => Ok(GloriousMode::Flashing),
        "chase" => Ok(GloriousMode::Chase),
        "wave" => Ok(GloriousMode::Wave),
        "spectrum" => Ok(GloriousMode::SpectrumCycle),
        "breathing" => Ok(GloriousMode::Breathing),
        "spectrum-breathing" => Ok(GloriousMode::SpectrumBreathing),
        "rainbow-wave" => Ok(GloriousMode::RainbowWave),
        "off" => Ok(GloriousMode::Off),
        _ => Err("unknown Glorious Model I mode".into()),
    }
}

fn parse_aorus_mode(input: &str) -> Result<AorusMode, Box<dyn Error>> {
    match input {
        "direct" => Ok(AorusMode::Direct),
        "static" => Ok(AorusMode::Static),
        "breathing" => Ok(AorusMode::Breathing),
        "spectrum" => Ok(AorusMode::SpectrumCycle),
        "flashing" => Ok(AorusMode::Flashing),
        "double-flash" => Ok(AorusMode::DoubleFlash),
        "off" => Ok(AorusMode::Off),
        _ => Err("unknown Gigabyte Aorus M2 mode".into()),
    }
}

fn parse_aorus_case_mode(input: &str) -> Result<AorusCaseMode, Box<dyn Error>> {
    match input {
        "custom" => Ok(AorusCaseMode::Custom),
        "off" => Ok(AorusCaseMode::Off),
        "breathing" => Ok(AorusCaseMode::Breathing),
        "spectrum" => Ok(AorusCaseMode::SpectrumCycle),
        "flashing" => Ok(AorusCaseMode::Flashing),
        "double-flashing" => Ok(AorusCaseMode::DoubleFlashing),
        _ => Err("unknown Gigabyte AORUS case mode".into()),
    }
}

fn parse_clevo_mode(input: &str) -> Result<ClevoMode, Box<dyn Error>> {
    match input {
        "direct" => Ok(ClevoMode::Direct),
        "breathing" => Ok(ClevoMode::Breathing),
        "wave" => Ok(ClevoMode::Wave),
        "bounce" => Ok(ClevoMode::Bounce),
        "marquee" => Ok(ClevoMode::Marquee),
        "scan" => Ok(ClevoMode::Scan),
        "off" => Ok(ClevoMode::Off),
        _ => Err("unknown CLEVO Lightbar mode".into()),
    }
}

fn parse_ek_mode(input: &str) -> Result<EkMode, Box<dyn Error>> {
    match input {
        "static" => Ok(EkMode::Static),
        "breathing" => Ok(EkMode::Breathing),
        "fading" => Ok(EkMode::Fading),
        "marquee" => Ok(EkMode::Marquee),
        "covering-marquee" => Ok(EkMode::CoveringMarquee),
        "pulse" => Ok(EkMode::Pulse),
        "spectrum-wave" => Ok(EkMode::SpectrumWave),
        "alternating" => Ok(EkMode::Alternating),
        "candle" => Ok(EkMode::Candle),
        _ => Err("unknown EK Loop Connect mode".into()),
    }
}

fn parse_areson_mode(input: &str) -> Result<AresonMode, Box<dyn Error>> {
    match input {
        "static" => Ok(AresonMode::Static),
        "rainbow-wave" => Ok(AresonMode::RainbowWave),
        "breathing" => Ok(AresonMode::Breathing),
        "spectrum" => Ok(AresonMode::SpectrumCycle),
        "single-color-wave" => Ok(AresonMode::SingleColorWave),
        "colorful-breathing" => Ok(AresonMode::ColorfulBreathing),
        "off" => Ok(AresonMode::Off),
        _ => Err("unknown Areson mouse mode".into()),
    }
}

fn parse_redragon_mode(input: &str) -> Result<RedragonMode, Box<dyn Error>> {
    match input {
        "static" => Ok(RedragonMode::Static),
        "wave" => Ok(RedragonMode::Wave),
        "breathing" => Ok(RedragonMode::Breathing),
        "breathing-random" => Ok(RedragonMode::RandomBreathing),
        "rainbow" => Ok(RedragonMode::Rainbow),
        "flashing" => Ok(RedragonMode::Flashing),
        _ => Err("unknown Redragon mouse mode".into()),
    }
}

fn parse_sayo_mode(input: &str) -> Result<SayoMode, Box<dyn Error>> {
    match input {
        "direct" => Ok(SayoMode::Direct),
        "breathing" => Ok(SayoMode::Breathing),
        "wave" => Ok(SayoMode::Wave),
        "switch" => Ok(SayoMode::Switch),
        "blink" => Ok(SayoMode::Blink),
        _ => Err("unknown SayoDevice E1 mode".into()),
    }
}

fn parse_sayo_random(input: &str) -> Result<bool, Box<dyn Error>> {
    match input {
        "static" => Ok(false),
        "random" => Ok(true),
        _ => Err("SayoDevice color behavior must be static or random".into()),
    }
}

fn parse_wushi_mode(input: &str) -> Result<WushiMode, Box<dyn Error>> {
    match input {
        "direct" => Ok(WushiMode::Direct),
        "breathing" => Ok(WushiMode::Breathing),
        "rainbow-wave" => Ok(WushiMode::RainbowWave),
        "spectrum" => Ok(WushiMode::SpectrumCycle),
        "race" => Ok(WushiMode::RaceCycle),
        "stacking" => Ok(WushiMode::Stacking),
        _ => Err("unknown Wushi L50 mode".into()),
    }
}

fn parse_wushi_direction(input: &str) -> Result<WushiDirection, Box<dyn Error>> {
    match input {
        "left" => Ok(WushiDirection::Left),
        "right" => Ok(WushiDirection::Right),
        _ => Err("Wushi direction must be left or right".into()),
    }
}

fn parse_aoc_direction(input: &str) -> Result<AocDirection, Box<dyn Error>> {
    match input {
        "cw" => Ok(AocDirection::Clockwise),
        "ccw" => Ok(AocDirection::CounterClockwise),
        _ => Err("AOC direction must be cw or ccw".into()),
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

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_bios_registry_parser_keeps_spaced_values() {
        let output = r"
HKEY_LOCAL_MACHINE\HARDWARE\DESCRIPTION\System\BIOS
    SystemManufacturer    REG_SZ    Micro-Star International Co., Ltd.
    SystemProductName     REG_SZ    Raider A18 HX A9WJG
";
        assert_eq!(
            parse_windows_registry_string(output, "SystemManufacturer").as_deref(),
            Some("Micro-Star International Co., Ltd.")
        );
        assert_eq!(
            parse_windows_registry_string(output, "SystemProductName").as_deref(),
            Some("Raider A18 HX A9WJG")
        );
    }
}
