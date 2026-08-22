#![forbid(unsafe_code)]

use std::fmt;
use std::num::{NonZeroU32, NonZeroU64};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Rgb8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb8 {
    pub const BLACK: Self = Self::new(0, 0, 0);

    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ControllerId(NonZeroU64);

impl ControllerId {
    #[must_use]
    pub const fn new(value: NonZeroU64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Incarnation(NonZeroU32);

impl Incarnation {
    #[must_use]
    pub const fn new(value: NonZeroU32) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ControllerRef {
    pub id: ControllerId,
    pub incarnation: Incarnation,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceType {
    Motherboard = 0,
    Dram = 1,
    Gpu = 2,
    Cooler = 3,
    LedStrip = 4,
    Keyboard = 5,
    Mouse = 6,
    MouseMat = 7,
    Headset = 8,
    HeadsetStand = 9,
    Gamepad = 10,
    Light = 11,
    Speaker = 12,
    Virtual = 13,
    Storage = 14,
    Case = 15,
    Microphone = 16,
    Accessory = 17,
    Keypad = 18,
    Laptop = 19,
    Monitor = 20,
    Unknown = 21,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LedIndex(u32);

impl LedIndex {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LedRange {
    pub start: LedIndex,
    pub len: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RangeOutOfBounds {
    pub start: u32,
    pub len: u32,
    pub led_count: u32,
}

impl fmt::Display for RangeOutOfBounds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LED range {}..{} exceeds LED count {}",
            self.start,
            self.start.saturating_add(self.len),
            self.led_count
        )
    }
}

impl std::error::Error for RangeOutOfBounds {}

impl LedRange {
    /// Creates a range into a flat LED sequence.
    ///
    /// # Errors
    ///
    /// Returns [`RangeOutOfBounds`] when the end overflows or exceeds `led_count`.
    pub fn checked(start: LedIndex, len: u32, led_count: u32) -> Result<Self, RangeOutOfBounds> {
        let end = start.get().checked_add(len).ok_or(RangeOutOfBounds {
            start: start.get(),
            len,
            led_count,
        })?;
        if end > led_count {
            return Err(RangeOutOfBounds {
                start: start.get(),
                len,
                led_count,
            });
        }
        Ok(Self { start, len })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ControllerCapabilities(u32);

impl ControllerCapabilities {
    pub const DIRECT_COLOR: Self = Self(1 << 0);
    pub const PER_LED_COLOR: Self = Self(1 << 1);
    pub const EFFECTS: Self = Self(1 << 2);
    pub const BRIGHTNESS: Self = Self(1 << 3);

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn contains(self, capability: Self) -> bool {
        self.0 & capability.0 == capability.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControllerDescription {
    pub name: String,
    pub vendor: String,
    pub description: String,
    pub device_type: DeviceType,
    pub modes: Vec<ModeDescription>,
    pub zone_names: Vec<String>,
    pub led_names: Vec<String>,
    pub capabilities: ControllerCapabilities,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeDescription {
    pub name: String,
    pub value: u32,
    pub color_mode: ModeColorMode,
    pub speed: Option<SpeedRange>,
    pub brightness: Option<BrightnessRange>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModeColorMode {
    None,
    PerLed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpeedRange {
    pub min: u8,
    pub max: u8,
    pub current: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BrightnessRange {
    pub min: u8,
    pub max: u8,
    pub current: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_device_type_discriminants_do_not_move() {
        assert_eq!(DeviceType::Motherboard as i32, 0);
        assert_eq!(DeviceType::Mouse as i32, 6);
        assert_eq!(DeviceType::Monitor as i32, 20);
        assert_eq!(DeviceType::Unknown as i32, 21);
    }

    #[test]
    fn led_ranges_reject_overflow_and_out_of_bounds() {
        assert!(LedRange::checked(LedIndex::new(2), 2, 4).is_ok());
        assert!(LedRange::checked(LedIndex::new(3), 2, 4).is_err());
        assert!(LedRange::checked(LedIndex::new(u32::MAX), 2, u32::MAX).is_err());
    }
}
