//! VE.Bus device implementation

use crate::Result;
use crate::bitreader::BitReader;
use crate::device::base::Device;
use crate::types::{ACInState, AlarmNotification, DeviceType, OperationMode};

/// VE.Bus data structure
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VEBusData {
    /// Device state
    pub device_state: Option<OperationMode>,
    /// VE.Bus error (interpretation not documented)
    pub error: Option<u8>,
    /// Battery voltage in volts
    pub battery_voltage: Option<f32>,
    /// Battery current in amperes (positive = charging, negative = inverting)
    pub battery_current: Option<f32>,
    /// AC input state
    pub ac_in_state: Option<ACInState>,
    /// AC input power in watts
    pub ac_in_power: Option<i32>,
    /// AC output power in watts
    pub ac_out_power: Option<i32>,
    /// Alarm notification
    pub alarm: Option<AlarmNotification>,
    /// Battery temperature in Celsius
    pub battery_temperature: Option<i8>,
    /// State of charge in percentage
    pub soc: Option<u8>,
}

impl Device for VEBusData {
    fn device_type() -> DeviceType {
        DeviceType::VEBus
    }

    fn parse_decrypted(reader: &mut BitReader) -> Result<Self> {
        // Parse device state (8 bits)
        let device_state_raw = reader.read_u8(8)?;
        let device_state = if device_state_raw == 0xFF {
            None
        } else {
            OperationMode::from_u8(device_state_raw)
        };

        // Parse error (8 bits)
        let error_raw = reader.read_u8(8)?;
        let error = if error_raw == 0xFF {
            None
        } else {
            Some(error_raw)
        };

        // Parse battery current (16 bits signed, 0.1A resolution)
        let battery_current_raw = reader.read_i16(16)?;
        let battery_current = if battery_current_raw == 0x7FFF {
            None
        } else {
            Some((battery_current_raw as f32) * 0.1)
        };

        // Parse battery voltage (14 bits, 0.01V resolution)
        let battery_voltage_raw = reader.read_u16(14)?;
        let battery_voltage = if battery_voltage_raw == 0x3FFF {
            None
        } else {
            Some((battery_voltage_raw as f32) * 0.01)
        };

        // Parse AC in state (2 bits)
        let ac_in_state_raw = reader.read_u8(2)?;
        let ac_in_state = if ac_in_state_raw == 3 {
            None
        } else {
            ACInState::from_u8(ac_in_state_raw)
        };

        // Parse AC in power (19 bits signed, 1W resolution)
        let ac_in_power_raw = reader.read_i32(19)?;
        let ac_in_power = if ac_in_power_raw == 0x3FFFF {
            None
        } else {
            Some(ac_in_power_raw)
        };

        // Parse AC out power (19 bits signed, 1W resolution)
        let ac_out_power_raw = reader.read_i32(19)?;
        let ac_out_power = if ac_out_power_raw == 0x3FFFF {
            None
        } else {
            Some(ac_out_power_raw)
        };

        // Parse alarm (2 bits)
        let alarm_raw = reader.read_u8(2)?;
        let alarm = if alarm_raw == 3 {
            None
        } else {
            AlarmNotification::from_u8(alarm_raw)
        };

        // Parse battery temperature (7 bits, Celsius with offset)
        let battery_temperature_raw = reader.read_u8(7)?;
        let battery_temperature = if battery_temperature_raw == 0x7F {
            None
        } else {
            Some((battery_temperature_raw as i8) - 40)
        };

        // Parse SOC (7 bits, 1% resolution)
        let soc_raw = reader.read_u8(7)?;
        let soc = if soc_raw == 0x7F { None } else { Some(soc_raw) };

        Ok(Self {
            device_state,
            error,
            battery_voltage,
            battery_current,
            ac_in_state,
            ac_in_power,
            ac_out_power,
            alarm,
            battery_temperature,
            soc,
        })
    }
}
