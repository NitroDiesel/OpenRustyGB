#![forbid(unsafe_code)]

use std::fmt;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HidEndpointInfo {
    path: Arc<[u8]>,
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface_number: i32,
    pub usage_page: u16,
    pub usage: u16,
    pub manufacturer: Option<Arc<str>>,
    pub product: Option<Arc<str>>,
    pub serial_number: Option<Arc<str>>,
}

impl HidEndpointInfo {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        path: Arc<[u8]>,
        vendor_id: u16,
        product_id: u16,
        interface_number: i32,
        usage_page: u16,
        usage: u16,
        manufacturer: Option<Arc<str>>,
        product: Option<Arc<str>>,
        serial_number: Option<Arc<str>>,
    ) -> Self {
        Self {
            path,
            vendor_id,
            product_id,
            interface_number,
            usage_page,
            usage,
            manufacturer,
            product,
            serial_number,
        }
    }

    #[must_use]
    pub fn path_bytes(&self) -> &[u8] {
        &self.path
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExactHidMatch {
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface_number: i32,
    pub usage_page: u16,
    pub usage: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HidDeviceMatch {
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface_number: Option<i32>,
    pub usage_page: Option<u16>,
    pub usage: Option<u16>,
}

impl HidDeviceMatch {
    #[must_use]
    pub fn matches(self, endpoint: &HidEndpointInfo) -> bool {
        endpoint.vendor_id == self.vendor_id
            && endpoint.product_id == self.product_id
            && self
                .interface_number
                .is_none_or(|value| endpoint.interface_number == value)
            && self
                .usage_page
                .is_none_or(|value| endpoint.usage_page == value)
            && self.usage.is_none_or(|value| endpoint.usage == value)
    }
}

impl From<ExactHidMatch> for HidDeviceMatch {
    fn from(value: ExactHidMatch) -> Self {
        Self {
            vendor_id: value.vendor_id,
            product_id: value.product_id,
            interface_number: Some(value.interface_number),
            usage_page: Some(value.usage_page),
            usage: Some(value.usage),
        }
    }
}

impl ExactHidMatch {
    #[must_use]
    pub fn matches(self, endpoint: &HidEndpointInfo) -> bool {
        endpoint.vendor_id == self.vendor_id
            && endpoint.product_id == self.product_id
            && endpoint.interface_number == self.interface_number
            && endpoint.usage_page == self.usage_page
            && endpoint.usage == self.usage
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputReport<const N: usize>([u8; N]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrefixTooLong {
    pub prefix_len: usize,
    pub report_len: usize,
}

impl fmt::Display for PrefixTooLong {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-byte prefix cannot fit in {}-byte output report",
            self.prefix_len, self.report_len
        )
    }
}

impl std::error::Error for PrefixTooLong {}

impl<const N: usize> OutputReport<N> {
    #[must_use]
    pub const fn from_array(bytes: [u8; N]) -> Self {
        Self(bytes)
    }

    /// Builds a fixed-size output report and zero-fills the unused tail.
    ///
    /// # Errors
    ///
    /// Returns [`PrefixTooLong`] when `prefix` is larger than the report.
    pub fn zero_padded(prefix: &[u8]) -> Result<Self, PrefixTooLong> {
        if prefix.len() > N {
            return Err(PrefixTooLong {
                prefix_len: prefix.len(),
                report_len: N,
            });
        }
        let mut bytes = [0; N];
        bytes[..prefix.len()].copy_from_slice(prefix);
        Ok(Self(bytes))
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; N] {
        &self.0
    }
}

pub trait OutputWriter<const N: usize> {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Sends one fixed-size HID output report.
    ///
    /// # Errors
    ///
    /// Returns the transport's error when the output operation fails.
    fn write_output(&mut self, report: &OutputReport<N>) -> Result<usize, Self::Error>;
}

pub trait FeatureWriter<const N: usize> {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Sends one fixed-size HID feature report.
    ///
    /// # Errors
    ///
    /// Returns the transport's error when the feature operation fails.
    fn send_feature_report(&mut self, report: &OutputReport<N>) -> Result<(), Self::Error>;
}

pub trait InputReader<const N: usize> {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Reads one HID input report into the caller's fixed-size buffer.
    ///
    /// # Errors
    ///
    /// Returns the transport's error when the input operation fails.
    fn read_input(&mut self, report: &mut [u8; N]) -> Result<usize, Self::Error>;
}

pub trait PendingInputReader<const N: usize> {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Attempts one non-blocking HID input read.
    ///
    /// A return value of zero means no pending report remains.
    ///
    /// # Errors
    ///
    /// Returns the transport's error when the non-blocking input operation fails.
    fn read_pending(&mut self, report: &mut [u8; N]) -> Result<usize, Self::Error>;
}

#[derive(Debug, Eq, PartialEq)]
pub enum ExactWriteError<E> {
    Transport(E),
    ShortWrite { expected: usize, actual: usize },
}

impl<E: fmt::Display> fmt::Display for ExactWriteError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(error) => write!(f, "output transport failed: {error}"),
            Self::ShortWrite { expected, actual } => {
                write!(
                    f,
                    "short output write: expected {expected} bytes, wrote {actual}"
                )
            }
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for ExactWriteError<E> {}

/// Sends one report and verifies that the transport accepted every byte.
///
/// # Errors
///
/// Returns [`ExactWriteError::Transport`] for a transport failure or
/// [`ExactWriteError::ShortWrite`] when fewer than `N` bytes were accepted.
pub fn write_exact<const N: usize, W: OutputWriter<N>>(
    writer: &mut W,
    report: &OutputReport<N>,
) -> Result<(), ExactWriteError<W::Error>> {
    let actual = writer
        .write_output(report)
        .map_err(ExactWriteError::Transport)?;
    if actual != N {
        return Err(ExactWriteError::ShortWrite {
            expected: N,
            actual,
        });
    }
    Ok(())
}

/// Sends one feature report through the platform HID transport.
///
/// # Errors
///
/// Returns the transport's feature-report error.
pub fn send_feature<const N: usize, W: FeatureWriter<N>>(
    writer: &mut W,
    report: &OutputReport<N>,
) -> Result<(), W::Error> {
    writer.send_feature_report(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_report_is_zero_padded_and_fixed_size() {
        assert_eq!(OutputReport::from_array([4, 5]).as_bytes(), &[4, 5]);
        let report = OutputReport::<8>::zero_padded(&[1, 2, 3]).unwrap();
        assert_eq!(report.as_bytes(), &[1, 2, 3, 0, 0, 0, 0, 0]);
        assert!(OutputReport::<2>::zero_padded(&[1, 2, 3]).is_err());
    }

    #[test]
    fn optional_hid_fields_match_like_native_detector_variants() {
        let endpoint = HidEndpointInfo::new(
            Arc::from(&b"match-test"[..]),
            0x1234,
            0x5678,
            3,
            0xFF00,
            2,
            None,
            None,
            None,
        );
        let product_only = HidDeviceMatch {
            vendor_id: 0x1234,
            product_id: 0x5678,
            interface_number: None,
            usage_page: None,
            usage: None,
        };
        assert!(product_only.matches(&endpoint));
        assert!(
            !HidDeviceMatch {
                usage: Some(1),
                ..product_only
            }
            .matches(&endpoint)
        );
    }
}
