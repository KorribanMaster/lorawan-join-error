//! Smart Lithium device implementation

use crate::Result;
use crate::bitreader::BitReader;
use crate::device::base::Device;
use crate::types::{BalancerStatus, DeviceType};

/// Smart Lithium data structure
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SmartLithiumData {
    /// BMS flags (meaning not documented)
    pub bms_flags: u32,
    /// Error flags (meaning not documented)
    pub error_flags: u16,
    /// Cell voltages (array of 8 cells)
    /// Special values: 0.0 = <2.61V, 40.0 = >3.85V, -1.0 = N/A
    pub cell_voltages: [f32; 8],
    /// Battery voltage in volts
    pub battery_voltage: Option<f32>,
    /// Balancer status
    pub balancer_status: Option<BalancerStatus>,
    /// Battery temperature in Celsius
    pub battery_temperature: Option<i8>,
}

impl Device for SmartLithiumData {
    fn device_type() -> DeviceType {
        DeviceType::SmartLithium
    }

    fn parse_decrypted(reader: &mut BitReader) -> Result<Self> {
        // Parse BMS flags (32 bits)
        let bms_flags = reader.read_u32(32)?;

        // Parse error flags (16 bits)
        let error_flags = reader.read_u16(16)?;

        // Parse 8 cell voltages (7 bits each)
        let mut cell_voltages = [0.0f32; 8];
        for cell_voltage in &mut cell_voltages {
            let cell_raw = reader.read_u8(7)?;
            *cell_voltage = parse_cell_voltage(cell_raw);
        }

        // Parse battery voltage (12 bits, 0.01V resolution)
        let battery_voltage_raw = reader.read_u16(12)?;
        let battery_voltage = if battery_voltage_raw == 0x0FFF {
            None
        } else {
            Some((battery_voltage_raw as f32) * 0.01)
        };

        // Parse balancer status (4 bits)
        let balancer_status_raw = reader.read_u8(4)?;
        let balancer_status = if balancer_status_raw == 0xF {
            None
        } else {
            BalancerStatus::from_u8(balancer_status_raw)
        };

        // Parse battery temperature (7 bits, Celsius with offset)
        let battery_temperature_raw = reader.read_u8(7)?;
        let battery_temperature = if battery_temperature_raw == 0x7F {
            None
        } else {
            Some((battery_temperature_raw as i8) - 40)
        };

        Ok(Self {
            bms_flags,
            error_flags,
            cell_voltages,
            battery_voltage,
            balancer_status,
            battery_temperature,
        })
    }
}

/// Parse a cell voltage from raw 7-bit value
///
/// Special values:
/// - 0x00: <2.61V (represented as 0.0)
/// - 0x7E: >3.85V (represented as 40.0)
/// - 0x7F: N/A (represented as -1.0)
/// - Other: (260 + value) / 100.0
fn parse_cell_voltage(raw: u8) -> f32 {
    match raw {
        0x00 => 0.0,  // < 2.61V
        0x7E => 40.0, // > 3.85V
        0x7F => -1.0, // N/A
        v => (260 + v as u16) as f32 / 100.0,
    }
}
