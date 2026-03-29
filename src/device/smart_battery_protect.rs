//! Smart Battery Protect device implementation

use crate::Result;
use crate::bitreader::BitReader;
use crate::device::base::Device;
use crate::types::{AlarmReason, ChargerError, DeviceType, OffReason, OperationMode, OutputState};

/// Smart Battery Protect data structure
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SmartBatteryProtectData {
    /// Device state
    pub device_state: Option<OperationMode>,
    /// Output state (ON/OFF)
    pub output_state: Option<OutputState>,
    /// Error code
    pub error_code: Option<ChargerError>,
    /// Alarm reason flags
    pub alarm_reason: AlarmReason,
    /// Warning reason flags
    pub warning_reason: AlarmReason,
    /// Input voltage in volts
    pub input_voltage: Option<f32>,
    /// Output voltage in volts
    pub output_voltage: Option<f32>,
    /// Off reason flags
    pub off_reason: OffReason,
}

impl Device for SmartBatteryProtectData {
    fn device_type() -> DeviceType {
        DeviceType::SmartBatteryProtect
    }

    fn parse_decrypted(reader: &mut BitReader) -> Result<Self> {
        // Parse device state (8 bits)
        let device_state_raw = reader.read_u8(8)?;
        let device_state = if device_state_raw == 0xFF {
            None
        } else {
            OperationMode::from_u8(device_state_raw)
        };

        // Parse output state (8 bits)
        let output_state_raw = reader.read_u8(8)?;
        let output_state = if output_state_raw == 0xFF {
            None
        } else {
            OutputState::from_u8(output_state_raw)
        };

        // Parse error code (8 bits)
        let error_code_raw = reader.read_u8(8)?;
        let error_code = if error_code_raw == 0xFF {
            None
        } else {
            ChargerError::from_u8(error_code_raw)
        };

        // Parse alarm reason (16 bits)
        let alarm_reason_raw = reader.read_u16(16)?;
        let alarm_reason = AlarmReason::new(alarm_reason_raw as u32);

        // Parse warning reason (16 bits)
        let warning_reason_raw = reader.read_u16(16)?;
        let warning_reason = AlarmReason::new(warning_reason_raw as u32);

        // Parse input voltage (16 bits signed, 0.01V resolution)
        let input_voltage_raw = reader.read_i16(16)?;
        let input_voltage = if input_voltage_raw == 0x7FFF {
            None
        } else {
            Some((input_voltage_raw as f32) * 0.01)
        };

        // Parse output voltage (16 bits, 0.01V resolution)
        let output_voltage_raw = reader.read_u16(16)?;
        let output_voltage = if output_voltage_raw == 0xFFFF {
            None
        } else {
            Some((output_voltage_raw as f32) * 0.01)
        };

        // Parse off reason (32 bits)
        let off_reason_raw = reader.read_u32(32)?;
        let off_reason = OffReason::new(off_reason_raw);

        Ok(Self {
            device_state,
            output_state,
            error_code,
            alarm_reason,
            warning_reason,
            input_voltage,
            output_voltage,
            off_reason,
        })
    }
}
