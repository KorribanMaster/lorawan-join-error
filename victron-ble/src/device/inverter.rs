//! Inverter device implementation

use crate::Result;
use crate::bitreader::BitReader;
use crate::device::base::Device;
use crate::types::{AlarmReason, DeviceType, OperationMode};

/// Inverter data structure
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InverterData {
    /// Device operation state
    pub device_state: Option<OperationMode>,
    /// Alarm reason flags
    pub alarm: AlarmReason,
    /// Battery voltage in volts
    pub battery_voltage: Option<f32>,
    /// AC apparent power in voltampere (VA)
    pub ac_apparent_power: Option<u16>,
    /// AC output voltage in volts
    pub ac_voltage: Option<f32>,
    /// AC output current in amperes
    pub ac_current: Option<f32>,
}

impl Device for InverterData {
    fn device_type() -> DeviceType {
        DeviceType::Inverter
    }

    fn parse_decrypted(reader: &mut BitReader) -> Result<Self> {
        // Parse device state (8 bits)
        let device_state_raw = reader.read_u8(8)?;
        let device_state = if device_state_raw == 0xFF {
            None
        } else {
            OperationMode::from_u8(device_state_raw)
        };

        // Parse alarm reason (16 bits)
        let alarm_raw = reader.read_u16(16)?;
        let alarm = AlarmReason::new(alarm_raw as u32);

        // Parse battery voltage (16 bits signed, 0.01V resolution)
        let battery_voltage_raw = reader.read_i16(16)?;
        let battery_voltage = if battery_voltage_raw == 0x7FFF {
            None
        } else {
            Some((battery_voltage_raw as f32) * 0.01)
        };

        // Parse AC apparent power (16 bits, 1VA resolution)
        let ac_apparent_power_raw = reader.read_u16(16)?;
        let ac_apparent_power = if ac_apparent_power_raw == 0xFFFF {
            None
        } else {
            Some(ac_apparent_power_raw)
        };

        // Parse AC voltage (15 bits, 0.01V resolution)
        let ac_voltage_raw = reader.read_u16(15)?;
        let ac_voltage = if ac_voltage_raw == 0x7FFF {
            None
        } else {
            Some((ac_voltage_raw as f32) * 0.01)
        };

        // Parse AC current (11 bits, 0.1A resolution)
        let ac_current_raw = reader.read_u16(11)?;
        let ac_current = if ac_current_raw == 0x7FF {
            None
        } else {
            Some((ac_current_raw as f32) * 0.1)
        };

        Ok(Self {
            device_state,
            alarm,
            battery_voltage,
            ac_apparent_power,
            ac_voltage,
            ac_current,
        })
    }
}
