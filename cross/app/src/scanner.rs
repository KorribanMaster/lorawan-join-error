//! BLE scanner for Victron devices

use crate::device::{Advertisement, DeviceData, detect_and_parse};
use crate::{ENCRYPTION_KEY_SIZE, Error, PRODUCT_ADVERTISEMENT_TYPE, Result};
use alloc::vec::Vec;
use trouble_host::prelude::*;

/// Victron BLE scanner
///
/// Scans for Victron device advertisements and parses them.
pub struct VictronScanner<'a> {
    /// Encryption keys for devices (MAC address -> key mapping)
    /// In real usage, you'd have multiple keys for different devices
    keys: &'a [[u8; ENCRYPTION_KEY_SIZE]],
}

impl<'a> VictronScanner<'a> {
    /// Create a new Victron scanner
    ///
    /// # Arguments
    /// * `keys` - Slice of encryption keys to try for decryption
    pub fn new(keys: &'a [[u8; ENCRYPTION_KEY_SIZE]]) -> Self {
        Self { keys }
    }

    /// Parse manufacturer data from a BLE advertisement
    ///
    /// # Arguments
    /// * `manufacturer_data` - Raw manufacturer data from advertisement
    ///
    /// # Returns
    /// Parsed DeviceData if successful
    pub fn parse_manufacturer_data(&self, manufacturer_data: &[u8]) -> Result<DeviceData> {
        // Check minimum length
        if manufacturer_data.is_empty() {
            defmt::debug!("Empty manufacturer data");
            return Err(Error::InvalidAdvertisement);
        }

        // First byte should be product advertisement type
        // Note: In Victron's format, this 0x10 byte is actually part of the prefix,
        // not a separate field to be skipped!
        if manufacturer_data[0] != PRODUCT_ADVERTISEMENT_TYPE {
            defmt::debug!(
                "Wrong product type: 0x{:02x}, expected 0x{:02x}",
                manufacturer_data[0],
                PRODUCT_ADVERTISEMENT_TYPE
            );
            return Err(Error::InvalidAdvertisement);
        }

        // Parse advertisement structure (includes the 0x10 byte as part of prefix)
        let adv = Advertisement::parse(manufacturer_data)?;

        defmt::debug!(
            "Advertisement: prefix=0x{:04x}, model=0x{:04x}, readout_type=0x{:02x}, nonce=0x{:04x}",
            adv.prefix,
            adv.model_id,
            adv.readout_type,
            adv.nonce
        );

        // Try each key until one works
        for key in self.keys {
            defmt::debug!("Advertisement \n device type = {},\n model id {},\n encyped_data {},\n key {}", adv.device_type(), adv.model_id, adv.encrypted_data, key);
           match detect_and_parse(&adv, key) {
                Ok(data) => return Ok(data),
                Err(Error::DecryptionFailed) => continue, // Try next key
                Err(e) => {
                    #[cfg(feature = "defmt")]
                    defmt::debug!("Parse error: {:?}", e);
                    return Err(e);
                }
            }
        }

        Err(Error::DecryptionFailed)
    }
}

/// Scan configuration for Victron devices
pub struct VictronScanConfig {
    /// Scan duration (use trouble's Duration)
    pub active: bool,
    /// Filter for specific device addresses (optional)
    pub device_filter: Option<Vec<Address>>,
}

impl Default for VictronScanConfig {
    fn default() -> Self {
        Self {
            active: true,
            device_filter: None,
        }
    }
}
