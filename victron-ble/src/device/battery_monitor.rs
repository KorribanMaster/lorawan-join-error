//! Battery Monitor (BMV) device parser

use crate::bitreader::BitReader;
use crate::device::Device;
use crate::{AlarmNotification, DeviceType, Result};

/// Battery Monitor telemetry data
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BatteryMonitorData {
    /// Time to go in minutes (0xFFFF = not available)
    pub time_to_go: Option<u16>,

    /// Battery voltage in volts
    pub voltage: f32,

    /// Alarm status
    pub alarm: AlarmNotification,

    /// Auxiliary input value (temperature in Celsius, voltage in V, or midpoint in V)
    pub aux_input: Option<AuxiliaryInput>,

    /// Battery current in amperes (positive = charging)
    pub current: f32,

    /// Consumed amp-hours in Ah
    pub consumed_ah: f32,

    /// State of charge as percentage (0-100%)
    pub soc: Option<f32>,
}

/// Auxiliary input types
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AuxiliaryInput {
    /// Starter battery voltage in volts
    StarterVoltage(f32),
    /// Midpoint voltage in volts
    MidpointVoltage(f32),
    /// Temperature in Celsius
    Temperature(f32),
    /// Not available
    NotAvailable,
}

impl Device for BatteryMonitorData {
    fn device_type() -> DeviceType {
        DeviceType::BatteryMonitor
    }

    fn parse_decrypted(reader: &mut BitReader) -> Result<Self> {
        // Read time to go (16 bits, minutes)
        let ttg_raw = reader.read_u16(16)?;
        let time_to_go = if ttg_raw == 0xFFFF {
            None
        } else {
            Some(ttg_raw)
        };

        // Read voltage (16 bits, 0.01V resolution, signed)
        // Range: -327.68..327.66 V, NA: 0x7FFF
        let voltage_raw = reader.read_i16(16)?;
        let voltage = if voltage_raw == 0x7FFF {
            0.0 // NA value
        } else {
            (voltage_raw as f32) * 0.01
        };

        // Read alarm reason (16 bits) - per PDF page 3
        let alarm_reason = reader.read_u16(16)?;
        let alarm = if alarm_reason != 0 {
            AlarmNotification::Alarm
        } else {
            AlarmNotification::Off
        };

        // Read auxiliary input value (16 bits) - BEFORE aux mode per PDF
        let aux_value_raw = reader.read_u16(16)?;

        // Read auxiliary input mode (2 bits) - AFTER aux value per PDF
        let aux_mode = reader.read_u8(2)?;
        let aux_input = match aux_mode {
            0 => {
                // Starter voltage: 0.01V resolution
                if aux_value_raw == 0xFFFF {
                    Some(AuxiliaryInput::NotAvailable)
                } else {
                    Some(AuxiliaryInput::StarterVoltage(
                        (aux_value_raw as f32) * 0.01,
                    ))
                }
            }
            1 => {
                // Midpoint voltage: 0.01V resolution
                if aux_value_raw == 0xFFFF {
                    Some(AuxiliaryInput::NotAvailable)
                } else {
                    Some(AuxiliaryInput::MidpointVoltage(
                        (aux_value_raw as f32) * 0.01,
                    ))
                }
            }
            2 => {
                // Temperature: 0.01K resolution, offset by -273.15 for Celsius
                if aux_value_raw == 0xFFFF {
                    Some(AuxiliaryInput::NotAvailable)
                } else {
                    let kelvin = (aux_value_raw as f32) * 0.01;
                    Some(AuxiliaryInput::Temperature(kelvin - 273.15))
                }
            }
            _ => Some(AuxiliaryInput::NotAvailable),
        };

        // Read current (22 bits, 0.001A resolution, signed)
        let current_raw = reader.read_i32(22)?;
        let current = (current_raw as f32) * 0.001;

        // Read consumed Ah (20 bits, 0.1Ah resolution)
        // Per PDF: "Consumed Ah = -Record value"
        let consumed_raw = reader.read_u32(20)?;
        let consumed_ah = if consumed_raw == 0xFFFFF {
            0.0 // NA value
        } else {
            -((consumed_raw as f32) * 0.1) // Negate per spec
        };

        // Read state of charge (10 bits, 0.1% resolution)
        let soc_raw = reader.read_u16(10)?;
        let soc = if soc_raw == 0x3FF {
            None
        } else {
            Some((soc_raw as f32) * 0.1)
        };

        Ok(BatteryMonitorData {
            time_to_go,
            voltage,
            alarm,
            aux_input,
            current,
            consumed_ah,
            soc,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery_monitor_parse() {
        // This would need real test data from victron-ble Python library
        // For now, just verify the struct can be created
        let data = BatteryMonitorData {
            time_to_go: Some(120),
            voltage: 12.5,
            alarm: AlarmNotification::Off,
            aux_input: Some(AuxiliaryInput::Temperature(25.0)),
            current: 5.5,
            consumed_ah: 10.0,
            soc: Some(85.5),
        };

        assert_eq!(data.voltage, 12.5);
        assert_eq!(data.soc, Some(85.5));
    }
}
