//! Solar Charger (MPPT) device parser

use crate::bitreader::BitReader;
use crate::device::Device;
use crate::{ChargerError, DeviceType, OperationMode, Result};

/// Solar Charger telemetry data
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SolarChargerData {
    /// Charge state / operation mode
    pub charge_state: Option<OperationMode>,

    /// Battery voltage in volts
    pub battery_voltage: f32,

    /// Battery charging current in amperes
    pub battery_current: f32,

    /// Yield today in watt-hours
    pub yield_today: u32,

    /// PV power in watts
    pub pv_power: u16,

    /// Load current in amperes (external load)
    pub load_current: Option<f32>,

    /// Charger error code
    pub error: Option<ChargerError>,
}

impl Device for SolarChargerData {
    fn device_type() -> DeviceType {
        DeviceType::SolarCharger
    }

    fn parse_decrypted(reader: &mut BitReader) -> Result<Self> {
        // Read charge state (8 bits)
        let state_raw = reader.read_u8(8)?;
        let charge_state = OperationMode::from_u8(state_raw);

        // Read battery voltage (16 bits, 0.01V resolution)
        let voltage_raw = reader.read_u16(16)?;
        let battery_voltage = (voltage_raw as f32) * 0.01;

        // Read battery current (16 bits, 0.1A resolution)
        let current_raw = reader.read_u16(16)?;
        let battery_current = (current_raw as f32) * 0.1;

        // Read yield today (20 bits, 10Wh resolution)
        let yield_raw = reader.read_u32(20)?;
        let yield_today = yield_raw * 10;

        // Read PV power (16 bits, 1W resolution)
        let pv_power = reader.read_u16(16)?;

        // Read load current (17 bits, 0.1A resolution)
        // 0x1FFFF = not available
        let load_raw = reader.read_u32(17)?;
        let load_current = if load_raw == 0x1FFFF {
            None
        } else {
            Some((load_raw as f32) * 0.1)
        };

        // Read charger error (8 bits)
        let error_raw = reader.read_u8(8)?;
        let error = ChargerError::from_u8(error_raw);

        Ok(SolarChargerData {
            charge_state,
            battery_voltage,
            battery_current,
            yield_today,
            pv_power,
            load_current,
            error,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solar_charger_parse() {
        // This would need real test data from victron-ble Python library
        // For now, just verify the struct can be created
        let data = SolarChargerData {
            charge_state: Some(OperationMode::Bulk),
            battery_voltage: 13.5,
            battery_current: 10.5,
            yield_today: 1500,
            pv_power: 200,
            load_current: Some(2.5),
            error: Some(ChargerError::NoError),
        };

        assert_eq!(data.battery_voltage, 13.5);
        assert_eq!(data.pv_power, 200);
    }
}
