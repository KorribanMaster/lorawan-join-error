//! Comprehensive tests for type conversions and enums
//!
//! Tests all enum from_u8/from_i16 conversions and bitfield operations
//! to ensure correct mapping between raw protocol values and Rust types.

use victron_ble::{
    ACInState, AlarmNotification, AlarmReason, BalancerStatus, ChargerError, DeviceType,
    MeterType, OffReason, OperationMode, OutputState,
};

// ============================================================================
// OperationMode Tests (15 valid values)
// ============================================================================

#[test]
fn test_operation_mode_all_valid_values() {
    let test_cases = vec![
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

    for (code, expected) in test_cases {
        let result = OperationMode::from_u8(code);
        assert_eq!(result, Some(expected), "Failed for code {}", code);
    }
}

#[test]
fn test_operation_mode_invalid_values() {
    let invalid_codes = vec![8, 10, 12, 100, 200, 244, 249, 250, 251, 253, 254, 255];

    for code in invalid_codes {
        let result = OperationMode::from_u8(code);
        assert_eq!(result, None, "Code {} should be invalid", code);
    }
}

// ============================================================================
// ChargerError Tests (30+ error codes)
// ============================================================================

#[test]
fn test_charger_error_all_valid_values() {
    let test_cases = vec![
        (0, ChargerError::NoError),
        (1, ChargerError::BatteryTemperatureTooHigh),
        (2, ChargerError::BatteryVoltageTooHigh),
        (3, ChargerError::BatteryTemperatureSensorMiswired),
        (4, ChargerError::RemoteTemperatureSensorFailure),
        (5, ChargerError::RemoteTemperatureSensorMiswired),
        (6, ChargerError::RemoteVoltageSenseMiswired),
        (7, ChargerError::RemoteVoltageWireLost),
        (17, ChargerError::ChargerTemperatureTooHigh),
        (18, ChargerError::ChargerOverCurrent),
        (19, ChargerError::ChargerCurrentReversed),
        (20, ChargerError::BulkTimeLimitExceeded),
        (21, ChargerError::CurrentSensorIssue),
        (26, ChargerError::InternalTemperatureSensorFailure),
        (27, ChargerError::FanFailure),
        (28, ChargerError::InternalDCVoltageError),
        (29, ChargerError::InternalSupplyVoltageError),
        (33, ChargerError::InternalBatteryVoltageSensorError),
        (34, ChargerError::InternalDCVoltageSensorError),
        (35, ChargerError::PVInputShutdownExcessiveCurrent),
        (36, ChargerError::PVInputShutdownOverVoltage),
        (38, ChargerError::PVInputShutdown),
        (39, ChargerError::PVInputFailedToShutdown),
        (65, ChargerError::InverterShutdownPanelVoltage),
        (66, ChargerError::InverterShutdownVoltageRange),
        (67, ChargerError::InverterShutdownWiring),
        (68, ChargerError::InverterShutdownConverterIssue),
        (114, ChargerError::InverterShutdownOverCurrent),
        (116, ChargerError::InverterShutdownBatteryVoltage),
        (117, ChargerError::InverterShutdownHighBatteryVoltage),
        (119, ChargerError::InverterOverload),
        (121, ChargerError::CPUTemperatureTooHigh),
        (200, ChargerError::CommunicationLost),
        (201, ChargerError::SynchronizationCalibration),
        (202, ChargerError::BmsTempTransmitError),
        (203, ChargerError::BMSConnectionLost),
    ];

    for (code, expected) in test_cases {
        let result = ChargerError::from_u8(code);
        assert_eq!(result, Some(expected), "Failed for code {}", code);
    }
}

#[test]
fn test_charger_error_invalid_values() {
    let invalid_codes = vec![8, 9, 10, 50, 100, 150, 199, 204, 255];

    for code in invalid_codes {
        let result = ChargerError::from_u8(code);
        assert_eq!(result, None, "Code {} should be invalid", code);
    }
}

// ============================================================================
// ACInState Tests (3 values)
// ============================================================================

#[test]
fn test_ac_in_state_all_values() {
    assert_eq!(ACInState::from_u8(0), Some(ACInState::NotConnected));
    assert_eq!(ACInState::from_u8(1), Some(ACInState::Connected));
    assert_eq!(ACInState::from_u8(2), Some(ACInState::Unknown));
}

#[test]
fn test_ac_in_state_invalid_values() {
    assert_eq!(ACInState::from_u8(3), None);
    assert_eq!(ACInState::from_u8(255), None);
}

// ============================================================================
// AlarmNotification Tests (3 values)
// ============================================================================

#[test]
fn test_alarm_notification_all_values() {
    assert_eq!(AlarmNotification::from_u8(0), Some(AlarmNotification::Off));
    assert_eq!(
        AlarmNotification::from_u8(1),
        Some(AlarmNotification::Alarm)
    );
    assert_eq!(
        AlarmNotification::from_u8(2),
        Some(AlarmNotification::Warning)
    );
}

#[test]
fn test_alarm_notification_invalid_values() {
    assert_eq!(AlarmNotification::from_u8(3), None);
    assert_eq!(AlarmNotification::from_u8(255), None);
}

// ============================================================================
// OutputState Tests (2 values)
// ============================================================================

#[test]
fn test_output_state_all_values() {
    assert_eq!(OutputState::from_u8(1), Some(OutputState::On));
    assert_eq!(OutputState::from_u8(4), Some(OutputState::Off));
}

#[test]
fn test_output_state_invalid_values() {
    assert_eq!(OutputState::from_u8(0), None);
    assert_eq!(OutputState::from_u8(2), None);
    assert_eq!(OutputState::from_u8(3), None);
    assert_eq!(OutputState::from_u8(255), None);
}

// ============================================================================
// BalancerStatus Tests (4 values)
// ============================================================================

#[test]
fn test_balancer_status_all_values() {
    assert_eq!(BalancerStatus::from_u8(0), Some(BalancerStatus::Unknown));
    assert_eq!(BalancerStatus::from_u8(1), Some(BalancerStatus::Balanced));
    assert_eq!(BalancerStatus::from_u8(2), Some(BalancerStatus::Balancing));
    assert_eq!(BalancerStatus::from_u8(3), Some(BalancerStatus::Imbalance));
}

#[test]
fn test_balancer_status_invalid_values() {
    assert_eq!(BalancerStatus::from_u8(4), None);
    assert_eq!(BalancerStatus::from_u8(255), None);
}

// ============================================================================
// DeviceType Tests (11 values)
// ============================================================================

#[test]
fn test_device_type_all_values() {
    assert_eq!(DeviceType::from_u8(0x01), Some(DeviceType::SolarCharger));
    assert_eq!(
        DeviceType::from_u8(0x02),
        Some(DeviceType::BatteryMonitor)
    );
    assert_eq!(DeviceType::from_u8(0x03), Some(DeviceType::Inverter));
    assert_eq!(
        DeviceType::from_u8(0x04),
        Some(DeviceType::DcDcConverter)
    );
    assert_eq!(DeviceType::from_u8(0x05), Some(DeviceType::SmartLithium));
    assert_eq!(DeviceType::from_u8(0x08), Some(DeviceType::AcCharger));
    assert_eq!(
        DeviceType::from_u8(0x09),
        Some(DeviceType::SmartBatteryProtect)
    );
    assert_eq!(DeviceType::from_u8(0x0A), Some(DeviceType::LynxSmartBMS));
    assert_eq!(DeviceType::from_u8(0x0C), Some(DeviceType::VEBus));
    assert_eq!(
        DeviceType::from_u8(0x0D),
        Some(DeviceType::DcEnergyMeter)
    );
    assert_eq!(DeviceType::from_u8(0x0F), Some(DeviceType::OrionXS));
}

#[test]
fn test_device_type_invalid_values() {
    assert_eq!(DeviceType::from_u8(0x00), None);
    assert_eq!(DeviceType::from_u8(0x06), None);
    assert_eq!(DeviceType::from_u8(0x07), None);
    assert_eq!(DeviceType::from_u8(0x0B), None);
    assert_eq!(DeviceType::from_u8(0x0E), None);
    assert_eq!(DeviceType::from_u8(0x10), None);
    assert_eq!(DeviceType::from_u8(0xFF), None);
}

// ============================================================================
// MeterType Tests (18 values, signed)
// ============================================================================

#[test]
fn test_meter_type_all_source_values() {
    // Negative values = sources
    assert_eq!(MeterType::from_i16(-9), Some(MeterType::SolarCharger));
    assert_eq!(MeterType::from_i16(-8), Some(MeterType::WindCharger));
    assert_eq!(MeterType::from_i16(-7), Some(MeterType::ShaftGenerator));
    assert_eq!(MeterType::from_i16(-6), Some(MeterType::Alternator));
    assert_eq!(MeterType::from_i16(-5), Some(MeterType::FuelCell));
    assert_eq!(MeterType::from_i16(-4), Some(MeterType::WaterGenerator));
    assert_eq!(MeterType::from_i16(-3), Some(MeterType::DcDcCharger));
    assert_eq!(MeterType::from_i16(-2), Some(MeterType::AcCharger));
    assert_eq!(MeterType::from_i16(-1), Some(MeterType::GenericSource));
}

#[test]
fn test_meter_type_all_load_values() {
    // Positive values = loads
    assert_eq!(MeterType::from_i16(1), Some(MeterType::GenericLoad));
    assert_eq!(MeterType::from_i16(2), Some(MeterType::ElectricDrive));
    assert_eq!(MeterType::from_i16(3), Some(MeterType::Fridge));
    assert_eq!(MeterType::from_i16(4), Some(MeterType::WaterPump));
    assert_eq!(MeterType::from_i16(5), Some(MeterType::BilgePump));
    assert_eq!(MeterType::from_i16(6), Some(MeterType::DcSystem));
    assert_eq!(MeterType::from_i16(7), Some(MeterType::Inverter));
    assert_eq!(MeterType::from_i16(8), Some(MeterType::WaterHeater));
}

#[test]
fn test_meter_type_invalid_values() {
    assert_eq!(MeterType::from_i16(0), None);
    assert_eq!(MeterType::from_i16(-10), None);
    assert_eq!(MeterType::from_i16(9), None);
    assert_eq!(MeterType::from_i16(100), None);
    assert_eq!(MeterType::from_i16(-100), None);
}

// ============================================================================
// AlarmReason Bitfield Tests
// ============================================================================

#[test]
fn test_alarm_reason_none() {
    let alarm = AlarmReason::new(AlarmReason::NONE);
    assert!(!alarm.has_flag(AlarmReason::LOW_VOLTAGE));
    assert!(!alarm.has_flag(AlarmReason::HIGH_VOLTAGE));
}

#[test]
fn test_alarm_reason_single_flags() {
    let flags = vec![
        AlarmReason::LOW_VOLTAGE,
        AlarmReason::HIGH_VOLTAGE,
        AlarmReason::LOW_SOC,
        AlarmReason::LOW_STARTER_VOLTAGE,
        AlarmReason::HIGH_STARTER_VOLTAGE,
        AlarmReason::LOW_TEMPERATURE,
        AlarmReason::HIGH_TEMPERATURE,
        AlarmReason::MID_VOLTAGE,
        AlarmReason::OVERLOAD,
        AlarmReason::DC_RIPPLE,
        AlarmReason::LOW_AC_OUT_VOLTAGE,
        AlarmReason::HIGH_AC_OUT_VOLTAGE,
    ];

    for flag in flags {
        let alarm = AlarmReason::new(flag);
        assert!(alarm.has_flag(flag), "Flag {} should be set", flag);
    }
}

#[test]
fn test_alarm_reason_multiple_flags() {
    let alarm = AlarmReason::new(
        AlarmReason::LOW_VOLTAGE | AlarmReason::HIGH_TEMPERATURE | AlarmReason::OVERLOAD,
    );

    assert!(alarm.has_flag(AlarmReason::LOW_VOLTAGE));
    assert!(alarm.has_flag(AlarmReason::HIGH_TEMPERATURE));
    assert!(alarm.has_flag(AlarmReason::OVERLOAD));
    assert!(!alarm.has_flag(AlarmReason::HIGH_VOLTAGE));
    assert!(!alarm.has_flag(AlarmReason::LOW_SOC));
}

#[test]
fn test_alarm_reason_all_flags() {
    let all_flags = AlarmReason::LOW_VOLTAGE
        | AlarmReason::HIGH_VOLTAGE
        | AlarmReason::LOW_SOC
        | AlarmReason::LOW_STARTER_VOLTAGE
        | AlarmReason::HIGH_STARTER_VOLTAGE
        | AlarmReason::LOW_TEMPERATURE
        | AlarmReason::HIGH_TEMPERATURE
        | AlarmReason::MID_VOLTAGE
        | AlarmReason::OVERLOAD
        | AlarmReason::DC_RIPPLE
        | AlarmReason::LOW_AC_OUT_VOLTAGE
        | AlarmReason::HIGH_AC_OUT_VOLTAGE;

    let alarm = AlarmReason::new(all_flags);

    assert!(alarm.has_flag(AlarmReason::LOW_VOLTAGE));
    assert!(alarm.has_flag(AlarmReason::HIGH_VOLTAGE));
    assert!(alarm.has_flag(AlarmReason::LOW_SOC));
    assert!(alarm.has_flag(AlarmReason::LOW_STARTER_VOLTAGE));
    assert!(alarm.has_flag(AlarmReason::HIGH_STARTER_VOLTAGE));
    assert!(alarm.has_flag(AlarmReason::LOW_TEMPERATURE));
    assert!(alarm.has_flag(AlarmReason::HIGH_TEMPERATURE));
    assert!(alarm.has_flag(AlarmReason::MID_VOLTAGE));
    assert!(alarm.has_flag(AlarmReason::OVERLOAD));
    assert!(alarm.has_flag(AlarmReason::DC_RIPPLE));
    assert!(alarm.has_flag(AlarmReason::LOW_AC_OUT_VOLTAGE));
    assert!(alarm.has_flag(AlarmReason::HIGH_AC_OUT_VOLTAGE));
}

// ============================================================================
// OffReason Bitfield Tests
// ============================================================================

#[test]
fn test_off_reason_none() {
    let off = OffReason::new(OffReason::NONE);
    assert!(!off.has_flag(OffReason::NO_INPUT_POWER));
    assert!(!off.has_flag(OffReason::SWITCHED_OFF_SWITCH));
}

#[test]
fn test_off_reason_single_flags() {
    let flags = vec![
        OffReason::NO_INPUT_POWER,
        OffReason::SWITCHED_OFF_SWITCH,
        OffReason::SWITCHED_OFF_REGISTER,
        OffReason::REMOTE_INPUT,
        OffReason::PROTECTION_ACTIVE,
        OffReason::PAYGO,
        OffReason::BMS,
        OffReason::ENGINE_SHUTDOWN,
        OffReason::ANALYSING_INPUT_VOLTAGE,
    ];

    for flag in flags {
        let off = OffReason::new(flag);
        assert!(off.has_flag(flag), "Flag {} should be set", flag);
    }
}

#[test]
fn test_off_reason_multiple_flags() {
    let off = OffReason::new(
        OffReason::NO_INPUT_POWER | OffReason::PROTECTION_ACTIVE | OffReason::BMS,
    );

    assert!(off.has_flag(OffReason::NO_INPUT_POWER));
    assert!(off.has_flag(OffReason::PROTECTION_ACTIVE));
    assert!(off.has_flag(OffReason::BMS));
    assert!(!off.has_flag(OffReason::SWITCHED_OFF_SWITCH));
    assert!(!off.has_flag(OffReason::REMOTE_INPUT));
}

#[test]
fn test_off_reason_all_flags() {
    let all_flags = OffReason::NO_INPUT_POWER
        | OffReason::SWITCHED_OFF_SWITCH
        | OffReason::SWITCHED_OFF_REGISTER
        | OffReason::REMOTE_INPUT
        | OffReason::PROTECTION_ACTIVE
        | OffReason::PAYGO
        | OffReason::BMS
        | OffReason::ENGINE_SHUTDOWN
        | OffReason::ANALYSING_INPUT_VOLTAGE;

    let off = OffReason::new(all_flags);

    assert!(off.has_flag(OffReason::NO_INPUT_POWER));
    assert!(off.has_flag(OffReason::SWITCHED_OFF_SWITCH));
    assert!(off.has_flag(OffReason::SWITCHED_OFF_REGISTER));
    assert!(off.has_flag(OffReason::REMOTE_INPUT));
    assert!(off.has_flag(OffReason::PROTECTION_ACTIVE));
    assert!(off.has_flag(OffReason::PAYGO));
    assert!(off.has_flag(OffReason::BMS));
    assert!(off.has_flag(OffReason::ENGINE_SHUTDOWN));
    assert!(off.has_flag(OffReason::ANALYSING_INPUT_VOLTAGE));
}
