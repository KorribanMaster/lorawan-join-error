//! DC Energy Meter device implementation

use crate::Result;
use crate::bitreader::BitReader;
use crate::device::base::Device;
use crate::device::battery_monitor::AuxiliaryInput;
use crate::types::{AlarmReason, DeviceType, MeterType};

/// DC Energy Meter data structure
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DcEnergyMeterData {
    /// Meter type
    pub meter_type: Option<MeterType>,
    /// Battery voltage in volts
    pub voltage: Option<f32>,
    /// Alarm reason flags
    pub alarm: AlarmReason,
    /// Current in amperes
    pub current: Option<f32>,
    /// Auxiliary input (starter voltage or temperature)
    pub aux: AuxiliaryInput,
}

impl Device for DcEnergyMeterData {
    fn device_type() -> DeviceType {
        DeviceType::DcEnergyMeter
    }

    fn parse_decrypted(reader: &mut BitReader) -> Result<Self> {
        // Parse meter type (16 bits signed)
        let meter_type_raw = reader.read_i16(16)?;
        let meter_type = MeterType::from_i16(meter_type_raw);

        // Parse voltage (16 bits signed, 0.01V resolution)
        let voltage_raw = reader.read_i16(16)?;
        let voltage = if voltage_raw == 0x7FFF {
            None
        } else {
            Some((voltage_raw as f32) * 0.01)
        };

        // Parse alarm reason (16 bits)
        let alarm_raw = reader.read_u16(16)?;
        let alarm = AlarmReason::new(alarm_raw as u32);

        // Parse aux value (16 bits)
        let aux_raw = reader.read_u16(16)?;

        // Parse aux mode (2 bits)
        let aux_mode = reader.read_u8(2)?;

        // Parse current (22 bits signed, 0.001A resolution)
        let current_raw = reader.read_i32(22)?;
        let current = if current_raw == 0x3FFFFF {
            None
        } else {
            Some((current_raw as f32) * 0.001)
        };

        // Parse auxiliary input based on mode
        let aux = match aux_mode {
            0 => {
                // Starter voltage (signed 16-bit value)
                if aux_raw == 0xFFFF {
                    AuxiliaryInput::NotAvailable
                } else {
                    // Treat as signed
                    let signed_val = if aux_raw & 0x8000 != 0 {
                        (aux_raw as i16) as i32
                    } else {
                        aux_raw as i32
                    };
                    AuxiliaryInput::StarterVoltage((signed_val as f32) * 0.01)
                }
            }
            2 => {
                // Temperature (Kelvin / 100)
                if aux_raw == 0xFFFF {
                    AuxiliaryInput::NotAvailable
                } else {
                    let temp_kelvin = (aux_raw as f32) * 0.01;
                    AuxiliaryInput::Temperature(kelvin_to_celsius(temp_kelvin))
                }
            }
            _ => AuxiliaryInput::NotAvailable,
        };

        Ok(Self {
            meter_type,
            voltage,
            alarm,
            current,
            aux,
        })
    }
}

/// Convert Kelvin to Celsius
fn kelvin_to_celsius(kelvin: f32) -> f32 {
    kelvin - 273.15
}
