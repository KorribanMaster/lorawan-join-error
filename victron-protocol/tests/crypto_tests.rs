//! Integration tests for Victron crypto module

use victron_protocol::crypto::{decrypt, encrypt_for_test};

#[test]
fn test_encryption_roundtrip() {
    let key = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
               0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10];
    let nonce = 0x1234u16;
    let plaintext = b"Hello, Victron!";

    // Encrypt
    let mut encrypted = vec![0u8; plaintext.len() + 1];
    let result = encrypt_for_test(&key, nonce, plaintext, &mut encrypted);
    assert!(result.is_ok());

    // Decrypt
    let mut decrypted = vec![0u8; plaintext.len()];
    let result = decrypt(&key, nonce, &encrypted, &mut decrypted);
    assert!(result.is_ok());
    assert_eq!(&decrypted[..], plaintext);
}

#[test]
fn test_decryption_wrong_key() {
    let correct_key = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
                       0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10];
    let wrong_key = [0xFF; 16];
    let nonce = 0x1234u16;
    let plaintext = b"Test data";

    // Encrypt with correct key
    let mut encrypted = vec![0u8; plaintext.len() + 1];
    encrypt_for_test(&correct_key, nonce, plaintext, &mut encrypted).unwrap();

    // Try to decrypt with wrong key - should fail
    let mut decrypted = vec![0u8; plaintext.len()];
    let result = decrypt(&wrong_key, nonce, &encrypted, &mut decrypted);
    assert!(result.is_err());
}

#[test]
fn test_empty_data() {
    let key = [0u8; 16];
    let nonce = 0x0000u16;
    let plaintext = b"";

    let mut encrypted = vec![0u8; 1]; // Just the key check byte
    let result = encrypt_for_test(&key, nonce, plaintext, &mut encrypted);
    assert!(result.is_ok());

    let mut decrypted = vec![];
    let result = decrypt(&key, nonce, &encrypted, &mut decrypted);
    assert!(result.is_ok());
    assert_eq!(decrypted.len(), 0);
}
