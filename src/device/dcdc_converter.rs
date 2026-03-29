//! Orion Smart DC-DC Converter device implementation
//!
//! Supports generic Orion Smart DC-DC converters (readout type 0x04)
//! Model IDs: 0xA3C0-0xA3CF and 0xA3D0-0xA3D3

use crate::Result;
use crate::bitreader::BitReader;
use crate::device::base::Device;
use crate::types::{ChargerError, DeviceType, OffReason, OperationMode};

/// DC-DC converter data structure
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DcDcConverterData {
    /// Charge state / operation mode
    pub charge_state: Option<OperationMode>,
    /// Charger error code
    pub charger_error: Option<ChargerError>,
    /// Input voltage in volts
    pub input_voltage: f32,
    /// Output voltage in volts (can be negative)
    pub output_voltage: f32,
    /// Off reason flags
    pub off_reason: OffReason,
}

impl Device for DcDcConverterData {
    fn device_type() -> DeviceType {
        DeviceType::DcDcConverter
    }

    fn parse_decrypted(reader: &mut BitReader) -> Result<Self> {
        // Parse device state (8 bits)
        let device_state = reader.read_u8(8)?;
        let charge_state = OperationMode::from_u8(device_state);

        // Parse charger error (8 bits)
        let charger_error_raw = reader.read_u8(8)?;
        let charger_error = ChargerError::from_u8(charger_error_raw);

        // Parse input voltage (16 bits, unsigned, 0.01V resolution)
        let input_voltage_raw = reader.read_u16(16)?;
        let input_voltage = if input_voltage_raw == 0xFFFF {
            0.0
        } else {
            (input_voltage_raw as f32) * 0.01
        };

        // Parse output voltage (16 bits, signed, 0.01V resolution)
        // 0x7FFF indicates not available
        let output_voltage_raw = reader.read_i16(16)?;
        let output_voltage = if output_voltage_raw == 0x7FFF {
            0.0
        } else {
            (output_voltage_raw as f32) * 0.01
        };

        // Parse off reason (32 bits)
        let off_reason_raw = reader.read_u32(32)?;
        let off_reason = OffReason::new(off_reason_raw);

        Ok(Self {
            charge_state,
            charger_error,
            input_voltage,
            output_voltage,
            off_reason,
        })
    }
}
