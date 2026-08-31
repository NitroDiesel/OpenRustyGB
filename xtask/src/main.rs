#![forbid(unsafe_code)]

use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

const PINNED_CONTROLLER_FAMILIES: usize = 197;
const PINNED_DETECTOR_SOURCES: usize = 224;
const PORTED_FAMILIES: &[(&str, &str)] = &[
    ("AOCMouseController", "aoc-gm500-mouse"),
    ("AOCMousematController", "aoc-amm700-mousemat"),
    ("AnnePro2Controller", "anne-pro-2"),
    ("AresonController", "areson-mice"),
    ("AsusMonitorController", "asus-monitor"),
    ("ClevoLightbarController", "clevo-lightbar"),
    ("DarkProject", "dark-project-kd3b-v2"),
    ("DreamCheekyController", "dream-cheeky-webmail-notifier"),
    ("DuckyKeyboardController", "ducky-keyboard"),
    ("EKController", "ek-loop-connect"),
    ("StreamDeckController", "elgato-stream-deck-mk2"),
    ("GameSirController", "gamesir-nova-lite-2"),
    ("GigabyteAorusMouseController", "gigabyte-aorus-m2"),
    ("GigabyteAorusPCCaseController", "gigabyte-aorus-c300-glass"),
    ("FaustusController", "faustus-keyboard"),
    ("HyperXMousematController", "hyperx-mousemat"),
    ("HYTEKeyboardController", "hyte-keeb-tkl"),
    ("InstantMouseController", "instant-mice"),
    ("IntelArcA770LEController", "intel-arc-a770-le"),
    ("IonicoController", "ionico"),
    ("LaviewTechnologyController", "glorious-model-i"),
    ("LexipMouseController", "lexip-np93-alpha"),
    (
        "LegoDimensionsToypadBaseController",
        "lego-dimensions-toypad",
    ),
    ("LuxaforController", "luxafor-flag"),
    ("MadCatzCyborgController", "madcatz-cyborg-light"),
    ("MSI3ZoneController", "msi-3-zone-keyboard"),
    ("MSILaptopController", "msi-laptop"),
    ("N5312AController", "n5312a-mouse"),
    ("NvidiaESAController", "nvidia-esa-xps-730x"),
    ("NZXTMouseController", "nzxt-lift-mouse"),
    ("PatriotViperMouseController", "patriot-viper-v550"),
    ("RedragonController", "redragon-mice"),
    ("RedSquareKeyroxController", "red-square-keyrox"),
    ("SayoDeviceController", "sayodevice-e1"),
    ("SkyloongController", "skyloong-gk104-pro"),
    ("SkydimoHIDController", "skydimo-sk0902"),
    ("ThingMController", "thingm-blink1-mk2"),
    ("TecknetController", "tecknet-m008"),
    (
        "ThermaltakePoseidonZRGBController",
        "thermaltake-poseidon-z-rgb",
    ),
    ("ValkyrieKeyboardController", "valkyrie-vk99"),
    ("WushiController", "wushi-l50"),
    ("XPGSummonerKeyboardController", "xpg-summoner"),
];
const PORTED_DETECTOR_SOURCES: &[&str] = &[
    "Controllers/AOCMouseController/AOCMouseControllerDetect.cpp",
    "Controllers/AOCMousematController/AOCMousematControllerDetect.cpp",
    "Controllers/AnnePro2Controller/AnnePro2ControllerDetect.cpp",
    "Controllers/AresonController/AresonControllerDetect.cpp",
    "Controllers/AsusMonitorController/AsusMonitorControllerDetect.cpp",
    "Controllers/ClevoLightbarController/ClevoLightbarControllerDetect.cpp",
    "Controllers/DarkProject/DarkProjectControllerDetect.cpp",
    "Controllers/DreamCheekyController/DreamCheekyControllerDetect.cpp",
    "Controllers/DuckyKeyboardController/DuckyKeyboardControllerDetect.cpp",
    "Controllers/EKController/EKControllerDetect.cpp",
    "Controllers/StreamDeckController/ElgatoStreamDeckControllerDetect.cpp",
    "Controllers/GameSirController/GameSirControllerDetect.cpp",
    "Controllers/GigabyteAorusMouseController/GigabyteAorusMouseControllerDetect.cpp",
    "Controllers/GigabyteAorusPCCaseController/GigabyteAorusPCCaseControllerDetect.cpp",
    "Controllers/HyperXMousematController/HyperXMousematControllerDetect.cpp",
    "Controllers/HYTEKeyboardController/HYTEKeyboardControllerDetect.cpp",
    "Controllers/InstantMouseController/InstantMouseControllerDetect.cpp",
    "Controllers/IntelArcA770LEController/IntelArcA770LEControllerDetect.cpp",
    "Controllers/IonicoController/IonicoControllerDetect.cpp",
    "Controllers/LaviewTechnologyController/LaviewTechnologyDetector.cpp",
    "Controllers/LexipMouseController/LexipMouseControllerDetect.cpp",
    "Controllers/LegoDimensionsToypadBaseController/LegoDimensionsToypadBaseControllerDetect.cpp",
    "Controllers/LuxaforController/LuxaforControllerDetect.cpp",
    "Controllers/MadCatzCyborgController/MadCatzCyborgControllerDetect.cpp",
    "Controllers/MSI3ZoneController/MSI3ZoneControllerDetect.cpp",
    "Controllers/MSILaptopController/MSILaptopControllerDetect.cpp",
    "Controllers/N5312AController/N5312AControllerDetect.cpp",
    "Controllers/NvidiaESAController/NvidiaESAControllerDetect.cpp",
    "Controllers/NZXTMouseController/NZXTMouseControllerDetect.cpp",
    "Controllers/PatriotViperMouseController/PatriotViperMouseControllerDetect.cpp",
    "Controllers/RedragonController/RedragonControllerDetect.cpp",
    "Controllers/RedSquareKeyroxController/RedSquareKeyroxControllerDetect.cpp",
    "Controllers/SayoDeviceController/SayoDeviceControllerDetect.cpp",
    "Controllers/SkyloongController/SkyloongControllerDetect.cpp",
    "Controllers/SkydimoHIDController/SkydimoHIDControllerDetect.cpp",
    "Controllers/ThingMController/ThingMControllerDetect.cpp",
    "Controllers/TecknetController/TecknetControllerDetect.cpp",
    "Controllers/ThermaltakePoseidonZRGBController/ThermaltakePoseidonZRGBControllerDetect.cpp",
    "Controllers/ValkyrieKeyboardController/ValkyrieKeyboardControllerDetect.cpp",
    "Controllers/WushiController/WushiL50USBDetect.cpp",
    "Controllers/XPGSummonerKeyboardController/XPGSummonerControllerDetect.cpp",
];

fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    match arguments.as_slice() {
        [command] if command == "inventory" => inventory(false),
        [command, flag] if command == "inventory" && flag == "--require-parity" => inventory(true),
        [command] if command == "source-audit" => source_audit(false),
        [command, flag]
            if command == "source-audit" && flag == "--require-rust-only" =>
        {
            source_audit(true)
        }
        _ => Err(
            "usage: cargo xtask inventory [--require-parity]\n       cargo xtask source-audit [--require-rust-only]"
                .into(),
        ),
    }
}

fn inventory(require_parity: bool) -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask must be located beneath the repository root")?;
    let controllers = root.join("Controllers");
    let families = immediate_directories(&controllers)?;
    let detector_sources = detector_sources(&controllers)?;
    let rust_drivers = rust_driver_packages(&root.join("crates").join("drivers"))?;

    if families.len() + PORTED_FAMILIES.len() != PINNED_CONTROLLER_FAMILIES {
        return Err(format!(
            "pinned family inventory drifted: expected {PINNED_CONTROLLER_FAMILIES}, found {} native plus {} ported",
            families.len(),
            PORTED_FAMILIES.len()
        )
        .into());
    }
    if detector_sources.len() + PORTED_DETECTOR_SOURCES.len() != PINNED_DETECTOR_SOURCES {
        return Err(format!(
            "pinned detector-source inventory drifted: expected {PINNED_DETECTOR_SOURCES}, found {} native plus {} ported",
            detector_sources.len(),
            PORTED_DETECTOR_SOURCES.len()
        )
        .into());
    }

    for (native_family, rust_package) in PORTED_FAMILIES {
        if controllers.join(native_family).exists() {
            return Err(format!(
                "ported family {native_family} still has a native source directory"
            )
            .into());
        }
        if !root
            .join("crates")
            .join("drivers")
            .join(rust_package)
            .join("Cargo.toml")
            .is_file()
        {
            return Err(format!(
                "ported family {native_family} is missing Rust package {rust_package}"
            )
            .into());
        }
    }

    println!("Pinned upstream controller families: {PINNED_CONTROLLER_FAMILIES}");
    println!(
        "Native controller-family directories remaining: {}",
        families.len()
    );
    println!(
        "Contracted Rust controller families: {}",
        PORTED_FAMILIES.len()
    );
    println!("Pinned upstream detector source files: {PINNED_DETECTOR_SOURCES}");
    println!(
        "Native detector source files remaining: {}",
        detector_sources.len()
    );
    println!(
        "Rust driver packages currently present: {}",
        rust_drivers.len()
    );
    let progress_tenths = PORTED_FAMILIES.len() * 1_000 / PINNED_CONTROLLER_FAMILIES;
    println!(
        "Family-package progress: {}/{} ({}.{:01}%)",
        PORTED_FAMILIES.len(),
        PINNED_CONTROLLER_FAMILIES,
        progress_tenths / 10,
        progress_tenths % 10
    );

    if require_parity && PORTED_FAMILIES.len() < PINNED_CONTROLLER_FAMILIES {
        return Err("driver parity gate is not complete; release remains blocked".into());
    }
    Ok(())
}

fn immediate_directories(path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut directories = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            directories.push(entry.path());
        }
    }
    directories.sort();
    Ok(directories)
}

fn detector_sources(root: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut found = Vec::new();
    visit_files(root, &mut |path| {
        let is_cpp = path.extension().is_some_and(|extension| extension == "cpp");
        let has_detect = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .is_some_and(|stem| stem.contains("Detect"));
        if is_cpp && has_detect {
            found.push(path.to_path_buf());
        }
    })?;
    found.sort();
    Ok(found)
}

fn rust_driver_packages(root: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut packages = Vec::new();
    for directory in immediate_directories(root)? {
        if directory.join("Cargo.toml").is_file() {
            packages.push(directory);
        }
    }
    Ok(packages)
}

fn source_audit(require_rust_only: bool) -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask must be located beneath the repository root")?;
    let mut native_sources = Vec::new();
    let mut native_build_files = Vec::new();
    visit_files(root, &mut |path| {
        if is_native_source(path) {
            native_sources.push(path.to_path_buf());
        }
        if is_native_build_file(path) {
            native_build_files.push(path.to_path_buf());
        }
    })?;
    native_sources.sort();
    native_build_files.sort();

    println!(
        "C/C++/Objective-C source and header files: {}",
        native_sources.len()
    );
    println!(
        "Native or Qt build-description files: {}",
        native_build_files.len()
    );
    if native_sources.is_empty() && native_build_files.is_empty() {
        println!("Rust-only source-tree gate: PASS");
        return Ok(());
    }

    println!("Rust-only source-tree gate: BLOCKED");
    if require_rust_only {
        let examples = native_sources
            .iter()
            .chain(&native_build_files)
            .take(10)
            .map(|path| {
                path.strip_prefix(root)
                    .unwrap_or(path)
                    .display()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "native source remains; release is forbidden. First remaining paths: {examples}"
        )
        .into());
    }
    Ok(())
}

fn is_native_source(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "c" | "cc"
                    | "cpp"
                    | "cxx"
                    | "h"
                    | "hh"
                    | "hpp"
                    | "hxx"
                    | "ipp"
                    | "inl"
                    | "m"
                    | "mm"
            )
        })
}

fn is_native_build_file(path: &Path) -> bool {
    let file_name = path.file_name().and_then(|name| name.to_str());
    if file_name.is_some_and(|name| name.eq_ignore_ascii_case("CMakeLists.txt")) {
        return true;
    }
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "cmake" | "pro" | "pri" | "qbs" | "qrc" | "ui"
            )
        })
}

fn visit_files(root: &Path, visitor: &mut dyn FnMut(&Path)) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            if entry.file_name() == ".git" || entry.file_name() == "target" {
                continue;
            }
            visit_files(&path, visitor)?;
        } else {
            visitor(&path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_source_classifier_covers_c_cpp_headers_and_objective_c() {
        for path in [
            "driver.c",
            "driver.cpp",
            "driver.hpp",
            "driver.inl",
            "platform.m",
            "platform.mm",
        ] {
            assert!(is_native_source(Path::new(path)), "missed {path}");
        }
        assert!(!is_native_source(Path::new("driver.rs")));
        assert!(!is_native_source(Path::new("README.md")));
    }

    #[test]
    fn native_build_classifier_covers_cmake_and_qt() {
        for path in [
            "CMakeLists.txt",
            "package.cmake",
            "OpenRGB.pro",
            "shared.pri",
            "resources.qrc",
            "dialog.ui",
        ] {
            assert!(is_native_build_file(Path::new(path)), "missed {path}");
        }
        assert!(!is_native_build_file(Path::new("Cargo.toml")));
    }
}
