//! Base device trait and advertisement parsing

use crate::bitreader::BitReader;
use crate::crypto::decrypt_aes_ctr;
use crate::{DeviceType, ENCRYPTION_KEY_SIZE, Error, Result};

/// Advertisement data structure
///
/// Structure of Victron advertisement (after company ID):
/// - Bytes 0-1: Prefix (typically 0x0010, where 0x10 is PRODUCT_ADVERTISEMENT_TYPE)
/// - Bytes 2-3: Model ID (little-endian)
/// - Byte 4: Readout type/mode
/// - Bytes 5-6: Nonce/IV (little-endian)
/// - Bytes 7+: Encrypted data (byte 7 is key check, bytes 8+ are AES-CTR encrypted)
#[derive(Debug)]
pub struct Advertisement<'a> {
    pub raw_data: &'a [u8],
    pub prefix: u16,
    pub model_id: u16,
    pub readout_type: u8,
    pub nonce: u16,
    pub encrypted_data: &'a [u8],
}

impl<'a> Advertisement<'a> {
    /// Parse advertisement from manufacturer data
    pub fn parse(data: &'a [u8]) -> Result<Self> {
        if data.len() < 7 {
            return Err(Error::InvalidAdvertisement);
        }

        // Parse fixed fields
        let prefix = u16::from_le_bytes([data[0], data[1]]);
        let model_id = u16::from_le_bytes([data[2], data[3]]);
        let readout_type = data[4];
        let nonce = u16::from_le_bytes([data[5], data[6]]);
        let encrypted_data = &data[7..];

        Ok(Self {
            raw_data: data,
            prefix,
            model_id,
            readout_type,
            nonce,
            encrypted_data,
        })
    }

    /// Decrypt the advertisement data
    pub fn decrypt(&self, key: &[u8; ENCRYPTION_KEY_SIZE], output: &mut [u8]) -> Result<()> {
        decrypt_aes_ctr(key, self.nonce, self.encrypted_data, output)
    }

    /// Get the device type from the readout type byte
    pub fn device_type(&self) -> Option<DeviceType> {
        DeviceType::from_u8(self.readout_type)
    }
}

/// Trait for device-specific data parsers
pub trait Device {
    /// Parse decrypted advertisement data
    fn parse_decrypted(reader: &mut BitReader) -> Result<Self>
    where
        Self: Sized;

    /// Get the device type
    fn device_type() -> DeviceType
    where
        Self: Sized;
}

/// Parse a complete advertisement with decryption
pub fn parse_advertisement<D: Device>(
    adv: &Advertisement,
    key: &[u8; ENCRYPTION_KEY_SIZE],
) -> Result<D> {
    // Decrypt data
    // Note: encrypted_data[0] is the key check byte (not encrypted)
    // decrypt() will verify it and decrypt the rest
    let mut decrypted = [0u8; 32]; // Max size for decrypted data
    if adv.encrypted_data.len() > decrypted.len() + 1 {
        return Err(Error::BufferTooSmall);
    }

    // Decrypt returns the data WITHOUT the first key-check byte
    let decrypted_len = adv.encrypted_data.len() - 1;
    adv.decrypt(key, &mut decrypted[..decrypted_len])?;

    // Create reader from decrypted data
    let mut reader = BitReader::new(&decrypted[..decrypted_len]);

    // Parse device-specific data
    D::parse_decrypted(&mut reader)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advertisement_parse() {
        // Minimal valid advertisement
        let data = [
            0x02, 0x10, // prefix (little-endian 0x1002)
            0x34, 0x12, // model_id (0x1234)
            0x02, // readout_type (BatteryMonitor)
            0x78, 0x56, // nonce (0x5678)
            0xAA, 0xBB, 0xCC, // encrypted data
        ];

        let adv = Advertisement::parse(&data).unwrap();
        assert_eq!(adv.prefix, 0x1002);
        assert_eq!(adv.model_id, 0x1234);
        assert_eq!(adv.readout_type, 0x02);
        assert_eq!(adv.nonce, 0x5678);
        assert_eq!(adv.encrypted_data.len(), 3);
        assert_eq!(adv.device_type(), Some(DeviceType::BatteryMonitor));
    }

    #[test]
    fn test_advertisement_too_short() {
        let data = [0x02, 0x10, 0x34]; // Only 3 bytes
        assert!(Advertisement::parse(&data).is_err());
    }
}
