//! Lynx Smart BMS device implementation

use crate::Result;
use crate::bitreader::BitReader;
use crate::device::base::Device;
use crate::types::DeviceType;

/// Lynx Smart BMS data structure
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LynxSmartBMSData {
    /// Error flags (meaning not documented)
    pub error_flags: u8,
    /// Remaining battery life in minutes
    pub remaining_mins: Option<u16>,
    /// Battery voltage in volts
    pub voltage: Option<f32>,
    /// Battery current in amperes
    pub current: Option<f32>,
    /// IO status (meaning not documented)
    pub io_status: u16,
    /// Alarm flags (meaning not documented)
    pub alarm_flags: u32,
    /// State of charge in percentage
    pub soc: Option<f32>,
    /// Consumed amp-hours
    pub consumed_ah: Option<f32>,
    /// Battery temperature in Celsius
    pub battery_temperature: Option<i8>,
}

impl Device for LynxSmartBMSData {
    fn device_type() -> DeviceType {
        DeviceType::LynxSmartBMS
    }

    fn parse_decrypted(reader: &mut BitReader) -> Result<Self> {
        // Parse error flags (8 bits)
        let error_flags = reader.read_u8(8)?;

        // Parse remaining minutes (16 bits)
        let remaining_mins_raw = reader.read_u16(16)?;
        let remaining_mins = if remaining_mins_raw == 0xFFFF {
            None
        } else {
            Some(remaining_mins_raw)
        };

        // Parse voltage (16 bits signed, 0.01V resolution)
        let voltage_raw = reader.read_i16(16)?;
        let voltage = if voltage_raw == 0x7FFF {
            None
        } else {
            Some((voltage_raw as f32) * 0.01)
        };

        // Parse current (16 bits signed, 0.1A resolution)
        let current_raw = reader.read_i16(16)?;
        let current = if current_raw == 0x7FFF {
            None
        } else {
            Some((current_raw as f32) * 0.1)
        };

        // Parse IO status (16 bits)
        let io_status = reader.read_u16(16)?;

        // Parse alarm flags (18 bits)
        let alarm_flags = reader.read_u32(18)?;

        // Parse SOC (10 bits, 0.1% resolution)
        let soc_raw = reader.read_u16(10)?;
        let soc = if soc_raw == 0x3FF {
            None
        } else {
            Some((soc_raw as f32) * 0.1)
        };

        // Parse consumed Ah (20 bits, 0.1Ah resolution)
        // Note: Documentation specifies "Consumed Ah = -Record value"
        let consumed_ah_raw = reader.read_u32(20)?;
        let consumed_ah = if consumed_ah_raw == 0xFFFFF {
            None
        } else {
            Some((consumed_ah_raw as f32) * -0.1)
        };

        // Parse battery temperature (7 bits, Celsius with offset)
        let battery_temperature_raw = reader.read_u8(7)?;
        let battery_temperature = if battery_temperature_raw == 0x7F {
            None
        } else {
            Some((battery_temperature_raw as i8) - 40)
        };

        Ok(Self {
            error_flags,
            remaining_mins,
            voltage,
            current,
            io_status,
            alarm_flags,
            soc,
            consumed_ah,
            battery_temperature,
        })
    }
}
