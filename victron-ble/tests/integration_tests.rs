//! Integration tests covering the full encryption/decryption flow
//! These tests use the crypto module to create encrypted data and verify
//! end-to-end parsing works correctly.

use victron_ble::bitreader::BitReader;
use victron_ble::crypto::{decrypt, encrypt_for_test};
use victron_ble::device::{
    Device, battery_monitor::BatteryMonitorData, solar_charger::SolarChargerData,
};
use victron_ble::{ChargerError, OperationMode};

#[test]
fn test_solar_charger_end_to_end() {
    // Test encryption and decryption roundtrip for solar charger data
    let key = [
        0xE2, 0xAA, 0x34, 0x74, 0x68, 0x30, 0x7B, 0x9E, 0xAE, 0x8B, 0x2E, 0x87, 0xCE, 0x86, 0x0C,
        0xFD,
    ];
    let nonce: u16 = 0xF681;

    // Real decrypted data from logs
    let plaintext = vec![5, 0, 90, 5, 17, 0, 22, 0, 24, 0, 255, 255];

    // Encrypt it
    let mut encrypted = vec![0u8; plaintext.len() + 1];
    let result = encrypt_for_test(&key, nonce, &plaintext, &mut encrypted);
    assert!(result.is_ok(), "Encryption failed");

    // Decrypt it back
    let mut decrypted = vec![0u8; plaintext.len()];
    let result = decrypt(&key, nonce, &encrypted, &mut decrypted);
    assert!(result.is_ok(), "Decryption failed");

    // Verify roundtrip
    assert_eq!(&decrypted[..], &plaintext[..], "Roundtrip mismatch");

    // Parse the decrypted data
    let mut reader = BitReader::new(&decrypted);
    let result = SolarChargerData::parse_decrypted(&mut reader);
    assert!(result.is_ok(), "Parsing failed");

    let data = result.unwrap();
    assert_eq!(data.charge_state, Some(OperationMode::Float));
    assert_eq!(data.error, Some(ChargerError::NoError));
    assert!((data.battery_voltage - 13.70).abs() < 0.01);
    assert!((data.battery_current - 1.7).abs() < 0.01);
    assert_eq!(data.yield_today, 220);
    assert_eq!(data.pv_power, 24);
}

#[test]
fn test_battery_monitor_end_to_end() {
    // Test encryption and decryption roundtrip for battery monitor data
    let key = [
        0xB9, 0xB8, 0x2D, 0x11, 0x5E, 0x7E, 0x64, 0x0C, 0x0D, 0x75, 0x0E, 0x79, 0x44, 0x66, 0x86,
        0xDF,
    ];
    let nonce: u16 = 0x9FEE;

    // Real decrypted data from logs
    let plaintext = vec![255, 255, 90, 5, 0, 0, 0, 0, 3, 15, 0, 0, 0, 128, 254];

    // Encrypt it
    let mut encrypted = vec![0u8; plaintext.len() + 1];
    let result = encrypt_for_test(&key, nonce, &plaintext, &mut encrypted);
    assert!(result.is_ok(), "Encryption failed");

    // Decrypt it back
    let mut decrypted = vec![0u8; plaintext.len()];
    let result = decrypt(&key, nonce, &encrypted, &mut decrypted);
    assert!(result.is_ok(), "Decryption failed");

    // Verify roundtrip
    assert_eq!(&decrypted[..], &plaintext[..], "Roundtrip mismatch");

    // Parse the decrypted data
    let mut reader = BitReader::new(&decrypted);
    let result = BatteryMonitorData::parse_decrypted(&mut reader);
    assert!(result.is_ok(), "Parsing failed");

    let data = result.unwrap();
    assert_eq!(data.time_to_go, None);
    assert!((data.voltage - 13.70).abs() < 0.01);
}

#[test]
fn test_wrong_key_detection() {
    // Verify that using the wrong key for decryption is detected
    let correct_key = [
        0xE2, 0xAA, 0x34, 0x74, 0x68, 0x30, 0x7B, 0x9E, 0xAE, 0x8B, 0x2E, 0x87, 0xCE, 0x86, 0x0C,
        0xFD,
    ];
    let wrong_key = [
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE,
        0xFF,
    ];
    let nonce: u16 = 0x1234;
    let plaintext = vec![1, 2, 3, 4, 5];

    // Encrypt with correct key
    let mut encrypted = vec![0u8; plaintext.len() + 1];
    encrypt_for_test(&correct_key, nonce, &plaintext, &mut encrypted).unwrap();

    // Try to decrypt with wrong key - should fail
    let mut decrypted = vec![0u8; plaintext.len()];
    let result = decrypt(&wrong_key, nonce, &encrypted, &mut decrypted);
    assert!(result.is_err(), "Decryption with wrong key should fail");
}

#[test]
fn test_solar_charger_boundary_values() {
    // Test maximum and minimum values from PDF spec
    // Max voltage: 327.66V (0x7FFE), Max current: 3276.6A (0x7FFE),
    // Max yield: 655.34 kWh (0xFFFE), Max PV power: 65534W
    let test_data = vec![
        5, 0, // State: Float, Error: None
        0xFE, 0x7F, // Voltage: max valid (327.66V)
        0xFE, 0x7F, // Current: max valid (3276.6A)
        0xFE, 0xFF, // Yield: max valid (6553.4 kWh)
        0xFE, 0xFF, // PV Power: max valid (65534W)
        0xFF, 0x01, // Load: NA
    ];

    let mut reader = BitReader::new(&test_data);
    let result = SolarChargerData::parse_decrypted(&mut reader);
    assert!(result.is_ok(), "Failed to parse boundary values");

    let data = result.unwrap();
    assert!((data.battery_voltage - 327.66).abs() < 0.01);
    assert!((data.battery_current - 3276.6).abs() < 0.1);
    assert_eq!(data.yield_today, 655340); // 65534 * 10
    assert_eq!(data.pv_power, 65534);
}

#[test]
fn test_battery_monitor_voltage_range() {
    // Test voltage boundary values
    // Min: -327.68V (0x8000), Max: 327.66V (0x7FFE), NA: 0x7FFF
    let test_cases = vec![
        (vec![0x00, 0x80], -327.68), // Minimum
        (vec![0xFE, 0x7F], 327.66),  // Maximum
        (vec![0xFF, 0x7F], 0.0),     // NA value → 0.0
    ];

    for (voltage_bytes, expected_voltage) in test_cases {
        let mut test_data = vec![0u8; 19];
        test_data[0] = 0xFF;
        test_data[1] = 0xFF; // TTG: NA
        test_data[2] = voltage_bytes[0];
        test_data[3] = voltage_bytes[1];

        let mut reader = BitReader::new(&test_data);
        let result = BatteryMonitorData::parse_decrypted(&mut reader);
        assert!(result.is_ok(), "Failed to parse voltage");

        let data = result.unwrap();
        assert!(
            (data.voltage - expected_voltage).abs() < 0.01,
            "Voltage mismatch: {} vs {}",
            data.voltage,
            expected_voltage
        );
    }
}

#[test]
fn test_solar_charger_all_operation_modes() {
    // Test all valid operation modes from the PDF spec
    let modes = vec![
        (0, OperationMode::Off),
        (1, OperationMode::LowPower),
        (2, OperationMode::Fault),
        (3, OperationMode::Bulk),
        (4, OperationMode::Absorption),
        (5, OperationMode::Float),
        (6, OperationMode::Storage),
        (7, OperationMode::Equalize),
        (9, OperationMode::Inverting),
        (11, OperationMode::PowerSupply),
        (245, OperationMode::StartingUp),
        (246, OperationMode::RepeatedAbsorption),
        (247, OperationMode::AutoEqualize),
        (248, OperationMode::BatterySafe),
        (252, OperationMode::ExternalControl),
    ];

    for (mode_code, expected_mode) in modes {
        let mut test_data = vec![0u8; 12];
        test_data[0] = mode_code;
        test_data[1] = 0; // No error
        test_data[2] = 0x00;
        test_data[3] = 0x05; // 12.80V

        let mut reader = BitReader::new(&test_data);
        let result = SolarChargerData::parse_decrypted(&mut reader);
        assert!(result.is_ok(), "Failed for mode {}", mode_code);

        let data = result.unwrap();
        assert_eq!(
            data.charge_state,
            Some(expected_mode),
            "Mode mismatch for code {}",
            mode_code
        );
    }
}

#[test]
fn test_solar_charger_all_error_codes() {
    // Test a selection of error codes from the PDF spec
    let errors = vec![
        (0, ChargerError::NoError),
        (1, ChargerError::BatteryTemperatureTooHigh),
        (2, ChargerError::BatteryVoltageTooHigh),
        (17, ChargerError::ChargerTemperatureTooHigh),
        (18, ChargerError::ChargerOverCurrent),
        (35, ChargerError::PVInputShutdownExcessiveCurrent),
        (36, ChargerError::PVInputShutdownOverVoltage),
        (119, ChargerError::InverterOverload),
        (200, ChargerError::CommunicationLost),
        (203, ChargerError::BMSConnectionLost),
    ];

    for (error_code, expected_error) in errors {
        let mut test_data = vec![0u8; 12];
        test_data[0] = 5; // State: Float
        test_data[1] = error_code;
        test_data[2] = 0x00;
        test_data[3] = 0x05; // 12.80V

        let mut reader = BitReader::new(&test_data);
        let result = SolarChargerData::parse_decrypted(&mut reader);
        assert!(result.is_ok(), "Failed for error {}", error_code);

        let data = result.unwrap();
        assert_eq!(
            data.error,
            Some(expected_error),
            "Error mismatch for code {}",
            error_code
        );
    }
}
