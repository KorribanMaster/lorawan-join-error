//! AC Charger device implementation

use crate::Result;
use crate::bitreader::BitReader;
use crate::device::base::Device;
use crate::types::{ChargerError, DeviceType, OperationMode};

/// AC Charger data structure
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AcChargerData {
    /// Charge state / operation mode
    pub charge_state: Option<OperationMode>,
    /// Charger error code
    pub charger_error: Option<ChargerError>,
    /// Output voltage 1 in volts
    pub output_voltage1: Option<f32>,
    /// Output current 1 in amperes
    pub output_current1: Option<f32>,
    /// Output voltage 2 in volts
    pub output_voltage2: Option<f32>,
    /// Output current 2 in amperes
    pub output_current2: Option<f32>,
    /// Output voltage 3 in volts
    pub output_voltage3: Option<f32>,
    /// Output current 3 in amperes
    pub output_current3: Option<f32>,
    /// Temperature in Celsius
    pub temperature: Option<i8>,
    /// AC input current in amperes
    pub ac_current: Option<f32>,
}

impl Device for AcChargerData {
    fn device_type() -> DeviceType {
        DeviceType::AcCharger
    }

    fn parse_decrypted(reader: &mut BitReader) -> Result<Self> {
        // Parse charge state (8 bits)
        let charge_state_raw = reader.read_u8(8)?;
        let charge_state = if charge_state_raw == 0xFF {
            None
        } else {
            OperationMode::from_u8(charge_state_raw)
        };

        // Parse charger error (8 bits)
        let charger_error_raw = reader.read_u8(8)?;
        let charger_error = if charger_error_raw == 0xFF {
            None
        } else {
            ChargerError::from_u8(charger_error_raw)
        };

        // Parse output voltage 1 (13 bits, 0.01V resolution)
        let output_voltage1_raw = reader.read_u16(13)?;
        let output_voltage1 = if output_voltage1_raw == 0x1FFF {
            None
        } else {
            Some((output_voltage1_raw as f32) * 0.01)
        };

        // Parse output current 1 (11 bits, 0.1A resolution)
        let output_current1_raw = reader.read_u16(11)?;
        let output_current1 = if output_current1_raw == 0x7FF {
            None
        } else {
            Some((output_current1_raw as f32) * 0.1)
        };

        // Parse output voltage 2 (13 bits, 0.01V resolution)
        let output_voltage2_raw = reader.read_u16(13)?;
        let output_voltage2 = if output_voltage2_raw == 0x1FFF {
            None
        } else {
            Some((output_voltage2_raw as f32) * 0.01)
        };

        // Parse output current 2 (11 bits, 0.1A resolution)
        let output_current2_raw = reader.read_u16(11)?;
        let output_current2 = if output_current2_raw == 0x7FF {
            None
        } else {
            Some((output_current2_raw as f32) * 0.1)
        };

        // Parse output voltage 3 (13 bits, 0.01V resolution)
        let output_voltage3_raw = reader.read_u16(13)?;
        let output_voltage3 = if output_voltage3_raw == 0x1FFF {
            None
        } else {
            Some((output_voltage3_raw as f32) * 0.01)
        };

        // Parse output current 3 (11 bits, 0.1A resolution)
        let output_current3_raw = reader.read_u16(11)?;
        let output_current3 = if output_current3_raw == 0x7FF {
            None
        } else {
            Some((output_current3_raw as f32) * 0.1)
        };

        // Parse temperature (7 bits, Celsius with offset)
        let temperature_raw = reader.read_u8(7)?;
        let temperature = if temperature_raw == 0x7F {
            None
        } else {
            Some((temperature_raw as i8) - 40)
        };

        // Parse AC current (9 bits, 0.1A resolution)
        let ac_current_raw = reader.read_u16(9)?;
        let ac_current = if ac_current_raw == 0x1FF {
            None
        } else {
            Some((ac_current_raw as f32) * 0.1)
        };

        Ok(Self {
            charge_state,
            charger_error,
            output_voltage1,
            output_current1,
            output_voltage2,
            output_current2,
            output_voltage3,
            output_current3,
            temperature,
            ac_current,
        })
    }
}
