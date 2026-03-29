//! Battery Sense device implementation
//!
//! Battery Sense uses the same format as Battery Monitor but only exposes
//! temperature and voltage

use crate::Result;
use crate::bitreader::BitReader;
use crate::device::base::Device;
use crate::types::DeviceType;

/// Battery Sense data structure
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BatterySenseData {
    /// Battery voltage in volts
    pub voltage: f32,
    /// Temperature in Celsius (from auxiliary input)
    pub temperature: Option<f32>,
}

impl Device for BatterySenseData {
    fn device_type() -> DeviceType {
        // Battery Sense uses Battery Monitor device type
        DeviceType::BatteryMonitor
    }

    fn parse_decrypted(reader: &mut BitReader) -> Result<Self> {
        // Parse using Battery Monitor format
        // Time to go (16 bits)
        let _time_to_go = reader.read_u16(16)?;

        // Voltage (16 bits, 0.01V resolution)
        let voltage_raw = reader.read_u16(16)?;
        let voltage = (voltage_raw as f32) * 0.01;

        // Alarm (2 bits)
        let _alarm_raw = reader.read_u8(2)?;

        // Auxiliary input mode (2 bits)
        let aux_mode = reader.read_u8(2)?;

        // Auxiliary input value (16 bits)
        let aux_raw = reader.read_u16(16)?;

        // Current (22 bits signed)
        let _current = reader.read_i32(22)?;

        // Consumed Ah (20 bits)
        let _consumed_ah = reader.read_u32(20)?;

        // SOC (10 bits)
        let _soc = reader.read_u16(10)?;

        // Parse auxiliary input to extract temperature
        let temperature = match aux_mode {
            2 => {
                // Temperature mode
                if aux_raw == 0xFFFF {
                    None
                } else {
                    let temp_kelvin = (aux_raw as f32) * 0.01;
                    Some(temp_kelvin - 273.15) // Convert Kelvin to Celsius
                }
            }
            _ => None,
        };

        Ok(Self {
            voltage,
            temperature,
        })
    }
}
