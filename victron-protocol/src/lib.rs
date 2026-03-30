//! Victron BLE Protocol Implementation
//!
//! This library provides hardware-independent parsing and encoding of Victron Energy
//! BLE advertisement data. It can be compiled for both embedded (no_std) and host (std)
//! targets, enabling comprehensive testing on development machines.

#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod bitreader;
pub mod crypto;
pub mod device;
pub mod types;
pub mod victron_payload;

// Re-export commonly used types
pub use device::{DeviceData, detect_and_parse};
pub use types::*;

/// Victron BLE manufacturer ID
pub const VICTRON_MANUFACTURER_ID: u16 = 0x02E1;

/// Product advertisement type identifier
pub const PRODUCT_ADVERTISEMENT_TYPE: u8 = 0x10;

/// Encryption key size in bytes
pub const ENCRYPTION_KEY_SIZE: usize = 16;

/// Result type for Victron protocol operations
pub type Result<T> = core::result::Result<T, Error>;

/// Error types for Victron protocol operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    InvalidKey,
    DecryptionFailed,
    InvalidAdvertisement,
    UnsupportedDevice,
    InvalidDeviceData,
    BufferTooSmall,
    ParseError,
}
