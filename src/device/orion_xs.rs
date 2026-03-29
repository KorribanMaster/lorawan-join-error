//! Orion XS DC-DC Converter device implementation
//!
//! Supports Orion XS DC-DC converters (readout type 0x0F)

use crate::Result;
use crate::bitreader::BitReader;
use crate::device::base::Device;
use crate::types::{ChargerError, DeviceType, OffReason, OperationMode};

/// Orion XS DC-DC converter data structure
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OrionXSData {
    /// Charge state / operation mode
    pub charge_state: Option<OperationMode>,
    /// Charger error code
    pub charger_error: Option<ChargerError>,
    /// Input voltage in volts
    pub input_voltage: f32,
    /// Input current in amperes
    pub input_current: f32,
    /// Output voltage in volts
    pub output_voltage: f32,
    /// Output current in amperes
    pub output_current: f32,
    /// Off reason flags
    pub off_reason: OffReason,
}

impl Device for OrionXSData {
    fn device_type() -> DeviceType {
        DeviceType::OrionXS
    }

    fn parse_decrypted(reader: &mut BitReader) -> Result<Self> {
        // Parse device state (8 bits)
        let device_state = reader.read_u8(8)?;
        let charge_state = OperationMode::from_u8(device_state);

        // Parse charger error (8 bits)
        let charger_error_raw = reader.read_u8(8)?;
        let charger_error = ChargerError::from_u8(charger_error_raw);

        // Parse output voltage (16 bits, unsigned, 0.01V resolution)
        let output_voltage_raw = reader.read_u16(16)?;
        let output_voltage = if output_voltage_raw == 0xFFFF {
            0.0
        } else {
            (output_voltage_raw as f32) * 0.01
        };

        // Parse output current (16 bits, unsigned, 0.1A resolution)
        let output_current_raw = reader.read_u16(16)?;
        let output_current = if output_current_raw == 0xFFFF {
            0.0
        } else {
            (output_current_raw as f32) * 0.1
        };

        // Parse input voltage (16 bits, unsigned, 0.01V resolution)
        let input_voltage_raw = reader.read_u16(16)?;
        let input_voltage = if input_voltage_raw == 0xFFFF {
            0.0
        } else {
            (input_voltage_raw as f32) * 0.01
        };

        // Parse input current (16 bits, unsigned, 0.1A resolution)
        let input_current_raw = reader.read_u16(16)?;
        let input_current = if input_current_raw == 0xFFFF {
            0.0
        } else {
            (input_current_raw as f32) * 0.1
        };

        // Parse off reason (32 bits)
        let off_reason_raw = reader.read_u32(32)?;
        let off_reason = OffReason::new(off_reason_raw);

        Ok(Self {
            charge_state,
            charger_error,
            input_voltage,
            input_current,
            output_voltage,
            output_current,
            off_reason,
        })
    }
}
