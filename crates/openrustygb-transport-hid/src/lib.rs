#![forbid(unsafe_code)]

use std::ffi::CString;
use std::fmt;
use std::sync::Arc;

use hidapi::{HidApi, HidDevice};
use openrustygb_driver_api::{ExactHidMatch, HidEndpointInfo, OutputReport, OutputWriter};

#[derive(Debug)]
pub enum HidTransportError {
    Hid(hidapi::HidError),
    InvalidPath,
    EndpointChanged,
}

impl fmt::Display for HidTransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hid(error) => write!(f, "HID operation failed: {error}"),
            Self::InvalidPath => f.write_str("HID path contains an interior NUL byte"),
            Self::EndpointChanged => {
                f.write_str("HID endpoint no longer matches the exact approved interface")
            }
        }
    }
}

impl std::error::Error for HidTransportError {}

impl From<hidapi::HidError> for HidTransportError {
    fn from(value: hidapi::HidError) -> Self {
        Self::Hid(value)
    }
}

#[derive(Debug, Default)]
pub struct HidInventory;

impl HidInventory {
    /// Enumerates HID metadata only. This function opens no device and writes no report.
    ///
    /// # Errors
    ///
    /// Returns [`HidTransportError`] if the platform HID inventory cannot be created.
    pub fn enumerate() -> Result<Vec<HidEndpointInfo>, HidTransportError> {
        let api = HidApi::new()?;
        Ok(api.device_list().map(to_endpoint_info).collect())
    }
}

fn to_endpoint_info(info: &hidapi::DeviceInfo) -> HidEndpointInfo {
    HidEndpointInfo::new(
        Arc::from(info.path().to_bytes()),
        info.vendor_id(),
        info.product_id(),
        info.interface_number(),
        info.usage_page(),
        info.usage(),
        info.manufacturer_string().map(Arc::from),
        info.product_string().map(Arc::from),
        info.serial_number().map(Arc::from),
    )
}

#[derive(Debug)]
pub struct HidOutput<const N: usize> {
    device: HidDevice,
}

impl<const N: usize> HidOutput<N> {
    /// Re-enumerates and revalidates every approved field immediately before opening.
    ///
    /// # Errors
    ///
    /// Returns [`HidTransportError::EndpointChanged`] if any identity field or
    /// path differs, or another [`HidTransportError`] if opening fails.
    pub fn open_exact(
        previously_seen: &HidEndpointInfo,
        approved: ExactHidMatch,
    ) -> Result<Self, HidTransportError> {
        if !approved.matches(previously_seen) {
            return Err(HidTransportError::EndpointChanged);
        }

        let path = CString::new(previously_seen.path_bytes())
            .map_err(|_| HidTransportError::InvalidPath)?;
        let api = HidApi::new()?;
        let still_exact = api.device_list().any(|info| {
            info.path().to_bytes() == previously_seen.path_bytes()
                && approved.matches(&to_endpoint_info(info))
        });
        if !still_exact {
            return Err(HidTransportError::EndpointChanged);
        }

        let device = api.open_path(&path)?;
        Ok(Self { device })
    }
}

impl<const N: usize> OutputWriter<N> for HidOutput<N> {
    type Error = HidTransportError;

    fn write_output(&mut self, report: &OutputReport<N>) -> Result<usize, Self::Error> {
        self.device.write(report.as_bytes()).map_err(Into::into)
    }
}
