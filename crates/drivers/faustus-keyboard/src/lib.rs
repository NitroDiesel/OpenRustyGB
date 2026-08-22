#![forbid(unsafe_code)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use openrustygb_domain::{
    ControllerCapabilities, ControllerDescription, DeviceType, ModeColorMode, ModeDescription,
    Rgb8, SpeedRange,
};

pub const DEFAULT_BASE_PATH: &str = "/sys/devices/platform/faustus/kbbl";
const REQUIRED_FILES: [&str; 6] = [
    "kbbl_red",
    "kbbl_green",
    "kbbl_blue",
    "kbbl_mode",
    "kbbl_flags",
    "kbbl_set",
];

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FaustusMode {
    Static = 0,
    Breathing = 1,
    ColorCycle = 2,
    Strobe = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FaustusUpdate {
    pub mode: FaustusMode,
    pub color: Rgb8,
}

impl FaustusUpdate {
    #[must_use]
    pub fn writes(self, base: &Path) -> [(PathBuf, String); 6] {
        [
            (base.join("kbbl_red"), format!("{:x}", self.color.r)),
            (base.join("kbbl_green"), format!("{:x}", self.color.g)),
            (base.join("kbbl_blue"), format!("{:x}", self.color.b)),
            (base.join("kbbl_mode"), format!("{:x}", self.mode as u8)),
            (base.join("kbbl_flags"), "2a".into()),
            (base.join("kbbl_set"), "2".into()),
        ]
    }

    /// Applies the same six sysfs values as the pinned native controller.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if a required attribute is missing or cannot be written.
    pub fn apply(self, base: &Path) -> io::Result<()> {
        let writes = self.writes(base);
        for (path, _) in &writes {
            if !path.is_file() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("missing Faustus attribute {}", path.display()),
                ));
            }
        }
        for (path, value) in writes {
            fs::write(path, value)?;
        }
        Ok(())
    }
}

#[must_use]
pub fn detect_at(base: &Path) -> bool {
    REQUIRED_FILES.iter().all(|file| base.join(file).is_file())
}

#[must_use]
pub fn description() -> ControllerDescription {
    let normal_speed = Some(SpeedRange {
        min: 0,
        max: 2,
        current: 1,
    });
    ControllerDescription {
        name: "ASUS TUF Laptop Keyboard".into(),
        vendor: "ASUS".into(),
        description: "Faustus Device".into(),
        device_type: DeviceType::Laptop,
        modes: vec![
            ModeDescription {
                name: "Static".into(),
                value: FaustusMode::Static as u32,
                color_mode: ModeColorMode::PerLed,
                speed: None,
            },
            ModeDescription {
                name: "Breathing".into(),
                value: FaustusMode::Breathing as u32,
                color_mode: ModeColorMode::PerLed,
                speed: normal_speed,
            },
            ModeDescription {
                name: "Color Cycle".into(),
                value: FaustusMode::ColorCycle as u32,
                color_mode: ModeColorMode::None,
                speed: normal_speed,
            },
            ModeDescription {
                name: "Strobe".into(),
                value: FaustusMode::Strobe as u32,
                color_mode: ModeColorMode::PerLed,
                speed: None,
            },
        ],
        zone_names: vec!["Keyboard Backlight zone".into()],
        led_names: vec!["Keyboard Backlight LED".into()],
        capabilities: ControllerCapabilities::PER_LED_COLOR.union(ControllerCapabilities::EFFECTS),
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "openrustygb-faustus-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir(&root).unwrap();
            for file in REQUIRED_FILES {
                fs::write(root.join(file), "").unwrap();
            }
            Self { root }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn detection_requires_all_six_attributes() {
        let fixture = Fixture::new();
        assert!(detect_at(&fixture.root));
        fs::remove_file(fixture.root.join("kbbl_set")).unwrap();
        assert!(!detect_at(&fixture.root));
    }

    #[test]
    fn update_preserves_native_hex_values_and_commit_flag() {
        let fixture = Fixture::new();
        FaustusUpdate {
            mode: FaustusMode::Breathing,
            color: Rgb8::new(0x12, 0xAB, 0xFF),
        }
        .apply(&fixture.root)
        .unwrap();

        assert_eq!(
            fs::read_to_string(fixture.root.join("kbbl_red")).unwrap(),
            "12"
        );
        assert_eq!(
            fs::read_to_string(fixture.root.join("kbbl_green")).unwrap(),
            "ab"
        );
        assert_eq!(
            fs::read_to_string(fixture.root.join("kbbl_blue")).unwrap(),
            "ff"
        );
        assert_eq!(
            fs::read_to_string(fixture.root.join("kbbl_mode")).unwrap(),
            "1"
        );
        assert_eq!(
            fs::read_to_string(fixture.root.join("kbbl_flags")).unwrap(),
            "2a"
        );
        assert_eq!(
            fs::read_to_string(fixture.root.join("kbbl_set")).unwrap(),
            "2"
        );
    }

    #[test]
    fn model_preserves_four_modes_and_single_led_shape() {
        let device = description();
        assert_eq!(device.modes.len(), 4);
        assert_eq!(device.modes[1].speed.unwrap().current, 1);
        assert_eq!(device.modes[2].color_mode, ModeColorMode::None);
        assert_eq!(device.zone_names, ["Keyboard Backlight zone"]);
        assert_eq!(device.led_names, ["Keyboard Backlight LED"]);
    }
}
