//! AES-CTR encryption/decryption for Victron BLE advertisements
//!
//! Victron uses AES-128 in CTR mode with:
//! - 128-bit counter initialized with a 16-bit nonce from the advertisement
//! - Little-endian byte ordering
//! - Counter starts at the nonce value (bytes 5-6 of advertisement)

use crate::{ENCRYPTION_KEY_SIZE, Error, Result};
use aes::Aes128;
use aes::cipher::{BlockEncrypt, KeyInit};

/// Decrypt Victron advertisement data using AES-CTR
///
/// # Arguments
/// * `key` - 16-byte encryption key
/// * `nonce` - 16-bit nonce from advertisement (little-endian)
/// * `encrypted_data` - Encrypted data to decrypt (first byte is key check, rest is encrypted)
/// * `output` - Buffer for decrypted data (will be encrypted_data.len() - 1 bytes)
///
/// # Returns
/// Ok(()) if decryption succeeded and key verification passed, Error otherwise
///
/// # Note
/// Victron's encryption scheme:
/// 1. First byte of encrypted_data must match first byte of key (unencrypted check)
/// 2. Only bytes [1..] are actually encrypted and get decrypted
pub fn decrypt_aes_ctr(
    key: &[u8; ENCRYPTION_KEY_SIZE],
    nonce: u16,
    encrypted_data: &[u8],
    output: &mut [u8],
) -> Result<()> {
    if encrypted_data.is_empty() {
        return Err(Error::InvalidAdvertisement);
    }

    // Verify the first byte of encrypted data matches first byte of key
    // This is Victron's key verification mechanism (happens BEFORE decryption)
    if encrypted_data[0] != key[0] {
        return Err(Error::DecryptionFailed);
    }

    // Only decrypt from byte 1 onwards
    let data_to_decrypt = &encrypted_data[1..];
    if output.len() < data_to_decrypt.len() {
        return Err(Error::BufferTooSmall);
    }

    if data_to_decrypt.is_empty() {
        return Ok(());
    }

    // Initialize AES cipher
    let cipher = Aes128::new(key.into());

    // CTR mode: we encrypt the counter values and XOR with ciphertext
    let num_blocks = data_to_decrypt.len().div_ceil(16);

    for block_idx in 0..num_blocks {
        // Create counter block: 128-bit value starting at nonce
        // Counter = nonce + block_idx, in little-endian
        let counter_value = (nonce as u128) + (block_idx as u128);
        let counter_block = counter_value.to_le_bytes();

        // Encrypt the counter block to get keystream
        let mut keystream = counter_block.into();
        cipher.encrypt_block(&mut keystream);

        // XOR with encrypted data to get plaintext
        let block_start = block_idx * 16;
        let block_end = core::cmp::min(block_start + 16, data_to_decrypt.len());

        for i in block_start..block_end {
            output[i] = data_to_decrypt[i] ^ keystream[i - block_start];
        }
    }

    Ok(())
}
