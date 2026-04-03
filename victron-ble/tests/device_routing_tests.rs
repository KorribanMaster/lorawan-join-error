//! Comprehensive tests for device detection and routing
//!
//! Tests the detect_and_parse() function and DeviceData enum to ensure
//! correct routing of advertisements to device-specific parsers.

use victron_ble::crypto::encrypt_for_test;
use victron_ble::device::{detect_and_parse, Advertisement, DeviceData};

// Test encryption key
const TEST_KEY: [u8; 16] = [
    0xE2, 0xAA, 0x34, 0x74, 0x68, 0x30, 0x7B, 0x9E, 0xAE, 0x8B, 0x2E, 0x87, 0xCE, 0x86, 0x0C,
    0xFD,
];

/// Helper to create a valid advertisement with encrypted data
fn create_advertisement(
    model_id: u16,
    readout_type: u8,
    nonce: u16,
    plaintext: &[u8],
) -> Vec<u8> {
    let mut adv = vec![0u8; 7 + plaintext.len() + 1]; // +1 for key check byte

    // Prefix (0x10 = PRODUCT_ADVERTISEMENT_TYPE, little-endian)
    adv[0] = 0x10;
    adv[1] = 0x02;

    // Model ID (little-endian)
    adv[2] = (model_id & 0xFF) as u8;
    adv[3] = (model_id >> 8) as u8;

    // Readout type
    adv[4] = readout_type;

    // Nonce (little-endian)
    adv[5] = (nonce & 0xFF) as u8;
    adv[6] = (nonce >> 8) as u8;

    // Encrypt the data
    let mut encrypted = vec![0u8; plaintext.len() + 1];
    encrypt_for_test(&TEST_KEY, nonce, plaintext, &mut encrypted).unwrap();

    // Copy encrypted data (including key check byte)
    adv[7..].copy_from_slice(&encrypted);

    adv
}

// ============================================================================
// Solar Charger (0x01) Tests
// ============================================================================

#[test]
fn test_detect_solar_charger() {
    // Valid solar charger data
    let plaintext = vec![5, 0, 90, 5, 17, 0, 22, 0, 24, 0, 255, 255];
    let adv_data = create_advertisement(0xA056, 0x01, 0x1234, &plaintext);

    let adv = Advertisement::parse(&adv_data).unwrap();
    let result = detect_and_parse(&adv, &TEST_KEY);

    assert!(result.is_ok(), "Should parse solar charger");
    match result.unwrap() {
        DeviceData::SolarCharger(data) => {
            assert!((data.battery_voltage - 13.70).abs() < 0.01);
        }
        _ => panic!("Expected SolarCharger variant"),
    }
}

// ============================================================================
// Battery Monitor (0x02) Tests
// ============================================================================

#[test]
fn test_detect_battery_monitor() {
    // Valid battery monitor data
    let plaintext = vec![255, 255, 90, 5, 0, 0, 0, 0, 3, 15, 0, 0, 0, 128, 254];
    let adv_data = create_advertisement(0x0203, 0x02, 0x5678, &plaintext);

    let adv = Advertisement::parse(&adv_data).unwrap();
    let result = detect_and_parse(&adv, &TEST_KEY);

    assert!(result.is_ok(), "Should parse battery monitor");
    match result.unwrap() {
        DeviceData::BatteryMonitor(data) => {
            assert!((data.voltage - 13.70).abs() < 0.01);
        }
        _ => panic!("Expected BatteryMonitor variant"),
    }
}

// ============================================================================
// Battery Sense Special Case (0x02 with specific model IDs)
// ============================================================================

#[test]
fn test_detect_battery_sense_0xa3a4() {
    // Battery Sense uses Battery Monitor readout type but special model ID
    let plaintext = vec![255, 255, 90, 5, 0, 0, 0, 0, 3, 15, 0, 0, 0, 128, 254];
    let adv_data = create_advertisement(0xA3A4, 0x02, 0xABCD, &plaintext);

    let adv = Advertisement::parse(&adv_data).unwrap();
    let result = detect_and_parse(&adv, &TEST_KEY);

    assert!(result.is_ok(), "Should parse battery sense");
    match result.unwrap() {
        DeviceData::BatterySense(_data) => {
            // Correct variant for Battery Sense
        }
        _ => panic!("Expected BatterySense variant for model 0xA3A4"),
    }
}

#[test]
fn test_detect_battery_sense_0xa3a5() {
    // Battery Sense alternate model ID
    let plaintext = vec![255, 255, 90, 5, 0, 0, 0, 0, 3, 15, 0, 0, 0, 128, 254];
    let adv_data = create_advertisement(0xA3A5, 0x02, 0xABCD, &plaintext);

    let adv = Advertisement::parse(&adv_data).unwrap();
    let result = detect_and_parse(&adv, &TEST_KEY);

    assert!(result.is_ok(), "Should parse battery sense");
    match result.unwrap() {
        DeviceData::BatterySense(_data) => {
            // Correct variant
        }
        _ => panic!("Expected BatterySense variant for model 0xA3A5"),
    }
}

#[test]
fn test_battery_monitor_not_battery_sense() {
    // Regular battery monitor (not Battery Sense model ID)
    let plaintext = vec![255, 255, 90, 5, 0, 0, 0, 0, 3, 15, 0, 0, 0, 128, 254];
    let adv_data = create_advertisement(0x0203, 0x02, 0xABCD, &plaintext);

    let adv = Advertisement::parse(&adv_data).unwrap();
    let result = detect_and_parse(&adv, &TEST_KEY);

    assert!(result.is_ok());
    match result.unwrap() {
        DeviceData::BatteryMonitor(_) => {
            // Should be BatteryMonitor, not BatterySense
        }
        DeviceData::BatterySense(_) => {
            panic!("Should not be BatterySense for non-special model ID")
        }
        _ => panic!("Expected BatteryMonitor variant"),
    }
}

// ============================================================================
// Inverter (0x03) Tests
// ============================================================================

#[test]
fn test_detect_inverter() {
    // Minimal valid inverter data (8 bits state + 16 bits alarm + 16 bits voltage + 16 bits power + 15 bits ac_voltage + 11 bits ac_current)
    let plaintext = vec![
        9,    // State: Inverting
        0, 0, // Alarm: None
        0x5F, 0xB4, // Voltage: 24.5V
        0x05, 0xDC, // Power: 1500 VA
        0x47, 0xE8, // AC voltage: 230V (15 bits MSB)
        0x41, // AC current: 6.5A (partial, 11 bits total)
        0, 0, // Padding
    ];
    let adv_data = create_advertisement(0x1234, 0x03, 0x9999, &plaintext);

    let adv = Advertisement::parse(&adv_data).unwrap();
    let result = detect_and_parse(&adv, &TEST_KEY);

    assert!(result.is_ok(), "Should parse inverter");
    match result.unwrap() {
        DeviceData::Inverter(data) => {
            assert!(data.battery_voltage.is_some());
        }
        _ => panic!("Expected Inverter variant"),
    }
}

// ============================================================================
// DC-DC Converter (0x04) Tests
// ============================================================================

#[test]
fn test_detect_dcdc_converter() {
    // DC-DC converter data: state(8) + error(8) + input_v(16) + output_v(16) + off_reason(32) = 80 bits = 10 bytes
    let plaintext = vec![
        3,    // State: Bulk
        0,    // Error: None
        0x92, 0x09, // Input: 24.5V (2450 * 0.01, little-endian)
        0xC4, 0x04, // Output: 12.2V (1220 * 0.01, little-endian)
        0, 0, 0, 0, // Off reason: None
    ];
    let adv_data = create_advertisement(0xA380, 0x04, 0x1111, &plaintext);

    let adv = Advertisement::parse(&adv_data).unwrap();
    let result = detect_and_parse(&adv, &TEST_KEY);

    assert!(result.is_ok(), "Should parse DC-DC converter");
    match result.unwrap() {
        DeviceData::DcDcConverter(data) => {
            assert!((data.input_voltage - 24.5).abs() < 0.1);
        }
        _ => panic!("Expected DcDcConverter variant"),
    }
}

// ============================================================================
// Smart Lithium (0x05) Tests
// ============================================================================

#[test]
fn test_detect_smart_lithium() {
    // Smart Lithium: bms_flags(32) + error_flags(16) + cells(8*7=56) + voltage(12) + balancer(4) + temp(7) + unused(1) = 128 bits = 16 bytes
    let plaintext = vec![
        0x12, 0x34, 0x56, 0x78, // BMS flags
        0xAB, 0xCD, // Error flags
        0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, // Cells (7 bits each, packed)
        0x67, 0x20, // Voltage (12 bits) + balancer (4 bits) = 16 bits
        0x19, // Temperature (7 bits) + unused (1 bit)
        0, // Padding
    ];
    let adv_data = create_advertisement(0xA3E0, 0x05, 0x2222, &plaintext);

    let adv = Advertisement::parse(&adv_data).unwrap();
    let result = detect_and_parse(&adv, &TEST_KEY);

    assert!(result.is_ok(), "Should parse Smart Lithium");
    match result.unwrap() {
        DeviceData::SmartLithium(data) => {
            assert!(data.battery_voltage.is_some());
        }
        _ => panic!("Expected SmartLithium variant"),
    }
}

// ============================================================================
// AC Charger (0x08) Tests
// ============================================================================

#[test]
fn test_detect_ac_charger() {
    // AC Charger: state(8) + error(8) + v1(13) + i1(11) + v2(13) + i2(11) + v3(13) + i3(11) + temp(7) + ac_current(9) + unused(24) = 128 bits
    let plaintext = vec![
        3,  // State: Bulk
        0,  // Error: None
        0x32, 0x00, // V1: 12.8V (13 bits MSB)
        0x00, 0x00, // I1: 0A (11 bits)
        0x00, 0x00, // V2, I2
        0x00, 0x00, // V3, I3
        0x19, // Temp: 25°C (7 bits)
        0x00, // AC current (9 bits)
        0, 0, 0, // Padding
    ];
    let adv_data = create_advertisement(0x1234, 0x08, 0x3333, &plaintext);

    let adv = Advertisement::parse(&adv_data).unwrap();
    let result = detect_and_parse(&adv, &TEST_KEY);

    assert!(result.is_ok(), "Should parse AC Charger");
    match result.unwrap() {
        DeviceData::AcCharger(_data) => {
            // Correct variant
        }
        _ => panic!("Expected AcCharger variant"),
    }
}

// ============================================================================
// Smart Battery Protect (0x09) Tests
// ============================================================================

#[test]
fn test_detect_smart_battery_protect() {
    // SBP: state(8) + output_state(8) + error(8) + alarm(16) + warning(16) + input_v(16) + output_v(16) + off_reason(32) + unused(32) = 152 bits = 19 bytes
    let plaintext = vec![
        3,  // State: On
        1,  // Output state: On
        0,  // Error: None
        0, 0, // Alarm: None
        0, 0, // Warning: None
        0x32, 0x00, // Input voltage: 12.8V
        0x32, 0x00, // Output voltage: 12.8V
        0, 0, 0, 0, // Off reason: None
        0, 0, 0, 0, // Unused
    ];
    let adv_data = create_advertisement(0xA3F0, 0x09, 0x4444, &plaintext);

    let adv = Advertisement::parse(&adv_data).unwrap();
    let result = detect_and_parse(&adv, &TEST_KEY);

    assert!(result.is_ok(), "Should parse Smart Battery Protect");
    match result.unwrap() {
        DeviceData::SmartBatteryProtect(_data) => {
            // Correct variant
        }
        _ => panic!("Expected SmartBatteryProtect variant"),
    }
}

// ============================================================================
// Lynx Smart BMS (0x0A) Tests
// ============================================================================

#[test]
fn test_detect_lynx_smart_bms() {
    // Lynx BMS: error(8) + ttg(16) + voltage(16) + current(16) + io(16) + warnings(18) + soc(10) + consumed(20) + temp(7) + unused(1) = 128 bits
    let plaintext = vec![
        0,    // Error: None
        0xFF, 0xFF, // TTG: NA
        0x32, 0x00, // Voltage: 12.8V
        0x00, 0x00, // Current: 0A
        0x00, 0x00, // IO status
        0x00, 0x00, 0x00, // Warnings/alarms (18 bits)
        0x03, 0xE8, // SOC: 100% (10 bits)
        0x00, 0x00, 0x00, // Consumed Ah (20 bits)
        0x19, // Temperature (7 bits)
        0,    // Unused
    ];
    let adv_data = create_advertisement(0xA3E1, 0x0A, 0x5555, &plaintext);

    let adv = Advertisement::parse(&adv_data).unwrap();
    let result = detect_and_parse(&adv, &TEST_KEY);

    assert!(result.is_ok(), "Should parse Lynx Smart BMS");
    match result.unwrap() {
        DeviceData::LynxSmartBMS(_data) => {
            // Correct variant
        }
        _ => panic!("Expected LynxSmartBMS variant"),
    }
}

// ============================================================================
// VE.Bus (0x0C) Tests
// ============================================================================

#[test]
fn test_detect_vebus() {
    // VE.Bus: state(8) + error(8) + current(16) + voltage(14) + active_ac(2) + ac_in_power(19) + ac_out_power(19) + alarm(2) + temp(7) + soc(7) + unused(26) = 128 bits
    let plaintext = vec![
        9,  // State: Inverting
        0,  // Error: None
        0x00, 0x00, // Current: 0A
        0x32, 0x00, // Voltage: 12.8V (14 bits MSB)
        0x00, // Active AC (2 bits)
        0x00, 0x00, 0x00, // AC in power (19 bits)
        0x00, 0x00, 0x00, // AC out power (19 bits)
        0x00, // Alarm (2 bits)
        0x19, // Temp (7 bits)
        0x64, // SOC: 100% (7 bits)
        0, 0, 0, // Unused
    ];
    let adv_data = create_advertisement(0xA340, 0x0C, 0x6666, &plaintext);

    let adv = Advertisement::parse(&adv_data).unwrap();
    let result = detect_and_parse(&adv, &TEST_KEY);

    assert!(result.is_ok(), "Should parse VE.Bus");
    match result.unwrap() {
        DeviceData::VEBus(_data) => {
            // Correct variant
        }
        _ => panic!("Expected VEBus variant"),
    }
}

// ============================================================================
// DC Energy Meter (0x0D) Tests
// ============================================================================

#[test]
fn test_detect_dc_energy_meter() {
    // DC Energy Meter: monitor_mode(16) + voltage(16) + alarm(16) + aux(16) + aux_input(2) + current(22) + unused(40) = 128 bits
    let plaintext = vec![
        0x00, 0x00, // Monitor mode
        0x32, 0x00, // Voltage: 12.8V
        0x00, 0x00, // Alarm: None
        0x00, 0x00, // Aux value
        0x00, // Aux input (2 bits)
        0x00, 0x00, 0x00, // Current (22 bits)
        0, 0, 0, 0, 0, // Unused
    ];
    let adv_data = create_advertisement(0xA381, 0x0D, 0x7777, &plaintext);

    let adv = Advertisement::parse(&adv_data).unwrap();
    let result = detect_and_parse(&adv, &TEST_KEY);

    assert!(result.is_ok(), "Should parse DC Energy Meter");
    match result.unwrap() {
        DeviceData::DcEnergyMeter(_data) => {
            // Correct variant
        }
        _ => panic!("Expected DcEnergyMeter variant"),
    }
}

// ============================================================================
// Orion XS (0x0F) Tests
// ============================================================================

#[test]
fn test_detect_orion_xs() {
    // Orion XS: state(8) + error(8) + out_v(16) + out_i(16) + in_v(16) + in_i(16) + off_reason(32) = 112 bits = 14 bytes
    let plaintext = vec![
        3,    // State: Bulk
        0,    // Error: None
        0xC4, 0x04, // Output voltage: 12.2V (1220 * 0.01, little-endian)
        0x37, 0x00, // Output current: 5.5A (55 * 0.1, little-endian)
        0x92, 0x09, // Input voltage: 24.5V (2450 * 0.01, little-endian)
        0x3D, 0x00, // Input current: 6.1A (61 * 0.1, little-endian)
        0, 0, 0, 0, // Off reason: None
    ];
    let adv_data = create_advertisement(0xA3C0, 0x0F, 0x8888, &plaintext);

    let adv = Advertisement::parse(&adv_data).unwrap();
    let result = detect_and_parse(&adv, &TEST_KEY);

    assert!(result.is_ok(), "Should parse Orion XS");
    match result.unwrap() {
        DeviceData::OrionXS(_data) => {
            // Correct variant
        }
        _ => panic!("Expected OrionXS variant"),
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_detect_invalid_device_type() {
    // Invalid readout type (0xFF)
    let plaintext = vec![0, 0, 0, 0];
    let adv_data = create_advertisement(0x1234, 0xFF, 0x9999, &plaintext);

    let adv = Advertisement::parse(&adv_data).unwrap();
    let result = detect_and_parse(&adv, &TEST_KEY);

    assert!(result.is_err(), "Should fail for invalid device type");
}

#[test]
fn test_detect_wrong_key() {
    // Valid advertisement but wrong key
    let plaintext = vec![5, 0, 90, 5, 17, 0, 22, 0, 24, 0, 255, 255];
    let adv_data = create_advertisement(0xA056, 0x01, 0x1234, &plaintext);

    let adv = Advertisement::parse(&adv_data).unwrap();

    // Wrong key
    let wrong_key = [0xFF; 16];
    let result = detect_and_parse(&adv, &wrong_key);

    assert!(result.is_err(), "Should fail with wrong key");
}

#[test]
fn test_detect_truncated_data() {
    // Advertisement too short
    let data = vec![0x10, 0x02, 0x34, 0x12, 0x01]; // Only 5 bytes, need at least 7

    let result = Advertisement::parse(&data);
    assert!(result.is_err(), "Should fail for truncated advertisement");
}
