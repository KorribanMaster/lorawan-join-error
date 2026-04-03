//! Comprehensive tests for binary payload packing
//!
//! Tests pack_device_data() and pack_device_with_metadata() to ensure
//! correct binary encoding of all device types for LoRaWAN transmission.

use victron_ble::device::{
    BatteryMonitorData, DcDcConverterData, DeviceData, InverterData, SmartLithiumData,
    SolarChargerData,
};
use victron_ble::victron_payload::{pack_device_data, pack_device_with_metadata};
use victron_ble::{AlarmNotification, AlarmReason, BalancerStatus, OffReason, OperationMode};

// ============================================================================
// Solar Charger Packing Tests
// ============================================================================

#[test]
fn test_pack_solar_charger_basic() {
    let data = SolarChargerData {
        charge_state: Some(OperationMode::Float),
        battery_voltage: 13.5,
        battery_current: 5.2,
        yield_today: 1500,
        pv_power: 100,
        load_current: None,
        error: None,
    };

    let mut output = vec![0u8; 20];
    let bytes_written = pack_device_data(&DeviceData::SolarCharger(data), &mut output);

    assert_eq!(bytes_written, 11, "Should write 11 bytes");
    assert_eq!(output[0], 0x01, "Type ID should be 0x01");

    // Voltage: 13.5V = 13500mV = 0x34BC
    assert_eq!(output[1], 0x34, "Voltage high byte");
    assert_eq!(output[2], 0xBC, "Voltage low byte");

    // Current: 5.2A = 5200mA = 0x1450
    assert_eq!(output[3], 0x14, "Current high byte");
    assert_eq!(output[4], 0x50, "Current low byte");

    // Yield: 1500Wh = 0x0005DC (24-bit, only 3 bytes used)
    assert_eq!(output[5], 0x00, "Yield byte 2");
    assert_eq!(output[6], 0x05, "Yield byte 1");
    assert_eq!(output[7], 0xDC, "Yield byte 0");

    // PV Power: 100W = 0x0064
    assert_eq!(output[8], 0x00, "PV power high byte");
    assert_eq!(output[9], 0x64, "PV power low byte");

    // State: Float = 5
    assert_eq!(output[10], 5, "State should be 5 (Float)");
}

#[test]
fn test_pack_solar_charger_negative_current() {
    let data = SolarChargerData {
        charge_state: Some(OperationMode::Off),
        battery_voltage: 12.0,
        battery_current: -2.5, // Negative current
        yield_today: 0,
        pv_power: 0,
        load_current: None,
        error: None,
    };

    let mut output = vec![0u8; 20];
    pack_device_data(&DeviceData::SolarCharger(data), &mut output);

    // Current: -2.5A = -2500mA = 0xF63C (two's complement)
    let current_bytes = i16::from_be_bytes([output[3], output[4]]);
    assert_eq!(current_bytes, -2500, "Negative current should be preserved");
}

#[test]
fn test_pack_solar_charger_large_yield() {
    let data = SolarChargerData {
        charge_state: Some(OperationMode::Bulk),
        battery_voltage: 14.2,
        battery_current: 10.0,
        yield_today: 65535, // Max 16-bit value
        pv_power: 500,
        load_current: None,
        error: None,
    };

    let mut output = vec![0u8; 20];
    pack_device_data(&DeviceData::SolarCharger(data), &mut output);

    // Yield: 65535 = 0x00FFFF (24-bit)
    assert_eq!(output[5], 0x00);
    assert_eq!(output[6], 0xFF);
    assert_eq!(output[7], 0xFF);
}

#[test]
fn test_pack_solar_charger_no_state() {
    let data = SolarChargerData {
        charge_state: None,
        battery_voltage: 12.5,
        battery_current: 1.0,
        yield_today: 100,
        pv_power: 50,
        load_current: None,
        error: None,
    };

    let mut output = vec![0u8; 20];
    pack_device_data(&DeviceData::SolarCharger(data), &mut output);

    assert_eq!(output[10], 0xFF, "No state should be 0xFF");
}

#[test]
fn test_pack_solar_charger_buffer_too_small() {
    let data = SolarChargerData {
        charge_state: Some(OperationMode::Float),
        battery_voltage: 13.5,
        battery_current: 5.0,
        yield_today: 1000,
        pv_power: 100,
        load_current: None,
        error: None,
    };

    let mut output = vec![0u8; 10]; // Too small
    let bytes_written = pack_device_data(&DeviceData::SolarCharger(data), &mut output);

    assert_eq!(bytes_written, 0, "Should return 0 for insufficient buffer");
}

// ============================================================================
// Battery Monitor Packing Tests
// ============================================================================

#[test]
fn test_pack_battery_monitor_basic() {
    let data = BatteryMonitorData {
        time_to_go: Some(120),
        voltage: 12.8,
        alarm: AlarmNotification::Off,
        aux_input: None,
        current: 5.5,
        consumed_ah: -10.5,
        soc: Some(85.5),
    };

    let mut output = vec![0u8; 20];
    let bytes_written = pack_device_data(&DeviceData::BatteryMonitor(data), &mut output);

    assert_eq!(bytes_written, 12, "Should write 12 bytes");
    assert_eq!(output[0], 0x02, "Type ID should be 0x02");

    // Voltage: 12.8V = 12800mV = 0x3200
    assert_eq!(output[1], 0x32);
    assert_eq!(output[2], 0x00);

    // Current: 5.5A = 5500mA = 0x157C
    assert_eq!(output[3], 0x15);
    assert_eq!(output[4], 0x7C);

    // SOC: 85.5% = 855 (0.1% res) = 0x0357
    assert_eq!(output[5], 0x03);
    assert_eq!(output[6], 0x57);

    // TTG: 120 min = 0x0078
    assert_eq!(output[7], 0x00);
    assert_eq!(output[8], 0x78);

    // Consumed: -10.5Ah = -105 (0.1Ah res) = 0xFF97 (two's complement)
    let consumed_bytes = i16::from_be_bytes([output[9], output[10]]);
    assert_eq!(consumed_bytes, -105);

    // Alarm: Off = 0
    assert_eq!(output[11], 0);
}

#[test]
fn test_pack_battery_monitor_no_soc() {
    let data = BatteryMonitorData {
        time_to_go: None,
        voltage: 13.0,
        alarm: AlarmNotification::Alarm,
        aux_input: None,
        current: 0.0,
        consumed_ah: 0.0,
        soc: None,
    };

    let mut output = vec![0u8; 20];
    pack_device_data(&DeviceData::BatteryMonitor(data), &mut output);

    // SOC: None = 0xFFFF
    assert_eq!(output[5], 0xFF);
    assert_eq!(output[6], 0xFF);

    // TTG: None = 0xFFFF
    assert_eq!(output[7], 0xFF);
    assert_eq!(output[8], 0xFF);

    // Alarm: Alarm = 1
    assert_eq!(output[11], 1);
}

#[test]
fn test_pack_battery_monitor_negative_current() {
    let data = BatteryMonitorData {
        time_to_go: Some(0),
        voltage: 11.5,
        alarm: AlarmNotification::Warning,
        aux_input: None,
        current: -15.2, // Discharging
        consumed_ah: -50.0,
        soc: Some(50.0),
    };

    let mut output = vec![0u8; 20];
    pack_device_data(&DeviceData::BatteryMonitor(data), &mut output);

    // Current: -15.2A = -15200mA = 0xC4B0 (two's complement)
    let current_bytes = i16::from_be_bytes([output[3], output[4]]);
    assert_eq!(current_bytes, -15200);

    // Alarm: Warning = 2
    assert_eq!(output[11], 2);
}

// ============================================================================
// Smart Lithium Packing Tests
// ============================================================================

#[test]
fn test_pack_smart_lithium_basic() {
    let data = SmartLithiumData {
        bms_flags: 0x12345678,
        error_flags: 0xABCD,
        cell_voltages: [3.3; 8],
        battery_voltage: Some(26.4),
        balancer_status: Some(BalancerStatus::Balanced),
        battery_temperature: Some(25),
    };

    let mut output = vec![0u8; 20];
    let bytes_written = pack_device_data(&DeviceData::SmartLithium(data), &mut output);

    assert_eq!(bytes_written, 7, "Should write 7 bytes");
    assert_eq!(output[0], 0x03, "Type ID should be 0x03");

    // Voltage: 26.4V = 26400mV = 0x6720
    assert_eq!(output[1], 0x67);
    assert_eq!(output[2], 0x20);

    // Temperature: 25°C (signed)
    assert_eq!(output[3], 25);

    // Balancer status: Balanced = 1
    assert_eq!(output[4], 1);

    // Error flags: 0xABCD
    assert_eq!(output[5], 0xAB);
    assert_eq!(output[6], 0xCD);
}

#[test]
fn test_pack_smart_lithium_no_voltage() {
    let data = SmartLithiumData {
        bms_flags: 0,
        error_flags: 0,
        cell_voltages: [0.0; 8],
        battery_voltage: None,
        balancer_status: None,
        battery_temperature: None,
    };

    let mut output = vec![0u8; 20];
    pack_device_data(&DeviceData::SmartLithium(data), &mut output);

    // Voltage: None = 0x0000
    assert_eq!(output[1], 0x00);
    assert_eq!(output[2], 0x00);

    // Temperature: None = -128 (0x80)
    assert_eq!(output[3], 0x80);

    // Balancer: None = 0xFF
    assert_eq!(output[4], 0xFF);
}

#[test]
fn test_pack_smart_lithium_negative_temperature() {
    let data = SmartLithiumData {
        bms_flags: 0,
        error_flags: 0,
        cell_voltages: [3.5; 8],
        battery_voltage: Some(28.0),
        balancer_status: Some(BalancerStatus::Balancing),
        battery_temperature: Some(-10),
    };

    let mut output = vec![0u8; 20];
    pack_device_data(&DeviceData::SmartLithium(data), &mut output);

    // Temperature: -10°C as i8
    let temp = output[3] as i8;
    assert_eq!(temp, -10);
}

// ============================================================================
// DC-DC Converter Packing Tests
// ============================================================================

#[test]
fn test_pack_dcdc_converter_basic() {
    let data = DcDcConverterData {
        charge_state: Some(OperationMode::Bulk),
        charger_error: None,
        input_voltage: 24.5,
        output_voltage: 12.2,
        off_reason: OffReason::new(OffReason::NONE),
    };

    let mut output = vec![0u8; 20];
    let bytes_written = pack_device_data(&DeviceData::DcDcConverter(data), &mut output);

    assert_eq!(bytes_written, 9, "Should write 9 bytes");
    assert_eq!(output[0], 0x04, "Type ID should be 0x04");

    // Input voltage: 24.5V = 24500mV = 0x5FB4
    assert_eq!(output[1], 0x5F);
    assert_eq!(output[2], 0xB4);

    // Output voltage: 12.2V = 12200mV = 0x2FA8
    assert_eq!(output[3], 0x2F);
    assert_eq!(output[4], 0xA8);

    // Off reason: 0x00000000
    assert_eq!(output[5], 0x00);
    assert_eq!(output[6], 0x00);
    assert_eq!(output[7], 0x00);
    assert_eq!(output[8], 0x00);
}

#[test]
fn test_pack_dcdc_converter_with_off_reason() {
    let data = DcDcConverterData {
        charge_state: Some(OperationMode::Off),
        charger_error: None,
        input_voltage: 0.0,
        output_voltage: 0.0,
        off_reason: OffReason::new(OffReason::NO_INPUT_POWER | OffReason::PROTECTION_ACTIVE),
    };

    let mut output = vec![0u8; 20];
    pack_device_data(&DeviceData::DcDcConverter(data), &mut output);

    // Off reason: bit 0 | bit 4 = 0x00000011
    let off_reason = u32::from_be_bytes([output[5], output[6], output[7], output[8]]);
    assert_eq!(off_reason, 0x00000011);
}

#[test]
fn test_pack_dcdc_converter_negative_output() {
    let data = DcDcConverterData {
        charge_state: Some(OperationMode::Float),
        charger_error: None,
        input_voltage: 48.0,
        output_voltage: -5.0, // Negative output voltage
        off_reason: OffReason::new(OffReason::NONE),
    };

    let mut output = vec![0u8; 20];
    pack_device_data(&DeviceData::DcDcConverter(data), &mut output);

    // Output: -5.0V = -5000mV = 0xEC78 (two's complement)
    let voltage = i16::from_be_bytes([output[3], output[4]]);
    assert_eq!(voltage, -5000);
}

// ============================================================================
// Inverter Packing Tests
// ============================================================================

#[test]
fn test_pack_inverter_basic() {
    let data = InverterData {
        device_state: Some(OperationMode::Inverting),
        alarm: AlarmReason::new(AlarmReason::NONE),
        battery_voltage: Some(24.5),
        ac_apparent_power: Some(1500),
        ac_voltage: Some(230.0),
        ac_current: Some(6.5),
    };

    let mut output = vec![0u8; 20];
    let bytes_written = pack_device_data(&DeviceData::Inverter(data), &mut output);

    assert_eq!(bytes_written, 10, "Should write 10 bytes");
    assert_eq!(output[0], 0x05, "Type ID should be 0x05");

    // Voltage: 24.5V = 24500mV = 0x5FB4
    assert_eq!(output[1], 0x5F);
    assert_eq!(output[2], 0xB4);

    // AC Power: 1500VA = 0x05DC
    assert_eq!(output[3], 0x05);
    assert_eq!(output[4], 0xDC);

    // State: Inverting = 9
    assert_eq!(output[5], 9);

    // Alarm: 0x00000000
    assert_eq!(output[6], 0x00);
    assert_eq!(output[7], 0x00);
    assert_eq!(output[8], 0x00);
    assert_eq!(output[9], 0x00);
}

#[test]
fn test_pack_inverter_with_alarms() {
    let data = InverterData {
        device_state: Some(OperationMode::Fault),
        alarm: AlarmReason::new(
            AlarmReason::LOW_VOLTAGE | AlarmReason::HIGH_TEMPERATURE | AlarmReason::OVERLOAD,
        ),
        battery_voltage: Some(10.5),
        ac_apparent_power: Some(0),
        ac_voltage: None,
        ac_current: None,
    };

    let mut output = vec![0u8; 20];
    pack_device_data(&DeviceData::Inverter(data), &mut output);

    // Alarm: bit 0 | bit 6 | bit 8 = 0x00000141
    let alarm = u32::from_be_bytes([output[6], output[7], output[8], output[9]]);
    assert_eq!(alarm, 0x00000141);
}

#[test]
fn test_pack_inverter_no_voltage() {
    let data = InverterData {
        device_state: None,
        alarm: AlarmReason::new(AlarmReason::NONE),
        battery_voltage: None,
        ac_apparent_power: None,
        ac_voltage: None,
        ac_current: None,
    };

    let mut output = vec![0u8; 20];
    pack_device_data(&DeviceData::Inverter(data), &mut output);

    // Voltage: None = 0x0000
    assert_eq!(output[1], 0x00);
    assert_eq!(output[2], 0x00);

    // Power: None = 0x0000
    assert_eq!(output[3], 0x00);
    assert_eq!(output[4], 0x00);

    // State: None = 0xFF
    assert_eq!(output[5], 0xFF);
}

// ============================================================================
// pack_device_with_metadata() Tests
// ============================================================================

#[test]
fn test_pack_with_metadata_solar_charger() {
    let data = SolarChargerData {
        charge_state: Some(OperationMode::Float),
        battery_voltage: 13.5,
        battery_current: 5.0,
        yield_today: 1000,
        pv_power: 100,
        load_current: None,
        error: None,
    };

    let mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    let rssi = -65i8;

    let mut output = vec![0u8; 30];
    let bytes_written =
        pack_device_with_metadata(&mac, rssi, &DeviceData::SolarCharger(data), &mut output);

    assert_eq!(bytes_written, 18, "Should write 7 + 11 = 18 bytes");

    // Check MAC address
    assert_eq!(&output[0..6], &mac);

    // Check RSSI
    assert_eq!(output[6] as i8, -65);

    // Check device type
    assert_eq!(output[7], 0x01);

    // Verify data starts at offset 7
    assert_eq!(output[8], 0x34); // Voltage high byte
}

#[test]
fn test_pack_with_metadata_buffer_too_small() {
    let data = SolarChargerData {
        charge_state: Some(OperationMode::Float),
        battery_voltage: 13.5,
        battery_current: 5.0,
        yield_today: 1000,
        pv_power: 100,
        load_current: None,
        error: None,
    };

    let mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    let rssi = -65i8;

    let mut output = vec![0u8; 5]; // Too small
    let bytes_written =
        pack_device_with_metadata(&mac, rssi, &DeviceData::SolarCharger(data), &mut output);

    assert_eq!(bytes_written, 0, "Should return 0 for too small buffer");
}

#[test]
fn test_pack_with_metadata_battery_monitor() {
    let data = BatteryMonitorData {
        time_to_go: Some(120),
        voltage: 12.5,
        alarm: AlarmNotification::Off,
        aux_input: None,
        current: 3.0,
        consumed_ah: -5.0,
        soc: Some(90.0),
    };

    let mac = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
    let rssi = -70i8;

    let mut output = vec![0u8; 30];
    let bytes_written =
        pack_device_with_metadata(&mac, rssi, &DeviceData::BatteryMonitor(data), &mut output);

    assert_eq!(bytes_written, 19, "Should write 7 + 12 = 19 bytes");
    assert_eq!(&output[0..6], &mac);
    assert_eq!(output[6] as i8, -70);
    assert_eq!(output[7], 0x02); // Battery Monitor type
}

// ============================================================================
// Edge Cases and Buffer Tests
// ============================================================================

#[test]
fn test_pack_empty_buffer() {
    let data = SolarChargerData {
        charge_state: Some(OperationMode::Float),
        battery_voltage: 13.5,
        battery_current: 5.0,
        yield_today: 1000,
        pv_power: 100,
        load_current: None,
        error: None,
    };

    let mut output = vec![];
    let bytes_written = pack_device_data(&DeviceData::SolarCharger(data), &mut output);

    assert_eq!(bytes_written, 0, "Should return 0 for empty buffer");
}

#[test]
fn test_pack_exact_size_buffer() {
    let data = SolarChargerData {
        charge_state: Some(OperationMode::Float),
        battery_voltage: 13.5,
        battery_current: 5.0,
        yield_today: 1000,
        pv_power: 100,
        load_current: None,
        error: None,
    };

    let mut output = vec![0u8; 12]; // Solar charger needs 12 bytes minimum
    let bytes_written = pack_device_data(&DeviceData::SolarCharger(data), &mut output);

    assert_eq!(bytes_written, 11, "Should work with sufficient buffer");
    assert_eq!(output[0], 0x01);
}
