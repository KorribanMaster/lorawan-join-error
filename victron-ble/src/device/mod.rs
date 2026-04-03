//! Device type detection and parsers

pub mod ac_charger;
pub mod base;
pub mod battery_monitor;
pub mod battery_sense;
pub mod dc_energy_meter;
pub mod dcdc_converter;
pub mod inverter;
pub mod lynx_smart_bms;
pub mod orion_xs;
pub mod smart_battery_protect;
pub mod smart_lithium;
pub mod solar_charger;
pub mod vebus;

pub use ac_charger::AcChargerData;
pub use base::{Advertisement, Device, parse_advertisement};
pub use battery_monitor::BatteryMonitorData;
pub use battery_sense::BatterySenseData;
pub use dc_energy_meter::DcEnergyMeterData;
pub use dcdc_converter::DcDcConverterData;
pub use inverter::InverterData;
pub use lynx_smart_bms::LynxSmartBMSData;
pub use orion_xs::OrionXSData;
pub use smart_battery_protect::SmartBatteryProtectData;
pub use smart_lithium::SmartLithiumData;
pub use solar_charger::SolarChargerData;
pub use vebus::VEBusData;

use crate::{DeviceType, ENCRYPTION_KEY_SIZE, Error, Result};

/// Detect and parse device advertisement
///
/// This function detects the device type and returns the parsed data
/// as a DeviceData enum.
pub fn detect_and_parse(
    adv: &Advertisement,
    key: &[u8; ENCRYPTION_KEY_SIZE],
) -> Result<DeviceData> {
    // Special handling: Battery Sense uses Battery Monitor device type
    // but has specific model IDs (0xA3A4, 0xA3A5)
    if adv.device_type() == Some(DeviceType::BatteryMonitor)
        && (adv.model_id == 0xA3A4 || adv.model_id == 0xA3A5)
    {
        let data = parse_advertisement::<battery_sense::BatterySenseData>(adv, key)?;
        return Ok(DeviceData::BatterySense(data));
    }

    match adv.device_type() {
        Some(DeviceType::AcCharger) => {
            let data = parse_advertisement::<ac_charger::AcChargerData>(adv, key)?;
            Ok(DeviceData::AcCharger(data))
        }
        Some(DeviceType::BatteryMonitor) => {
            let data = parse_advertisement::<battery_monitor::BatteryMonitorData>(adv, key)?;
            Ok(DeviceData::BatteryMonitor(data))
        }
        Some(DeviceType::DcDcConverter) => {
            let data = parse_advertisement::<dcdc_converter::DcDcConverterData>(adv, key)?;
            Ok(DeviceData::DcDcConverter(data))
        }
        Some(DeviceType::DcEnergyMeter) => {
            let data = parse_advertisement::<dc_energy_meter::DcEnergyMeterData>(adv, key)?;
            Ok(DeviceData::DcEnergyMeter(data))
        }
        Some(DeviceType::Inverter) => {
            let data = parse_advertisement::<inverter::InverterData>(adv, key)?;
            Ok(DeviceData::Inverter(data))
        }
        Some(DeviceType::LynxSmartBMS) => {
            let data = parse_advertisement::<lynx_smart_bms::LynxSmartBMSData>(adv, key)?;
            Ok(DeviceData::LynxSmartBMS(data))
        }
        Some(DeviceType::OrionXS) => {
            let data = parse_advertisement::<orion_xs::OrionXSData>(adv, key)?;
            Ok(DeviceData::OrionXS(data))
        }
        Some(DeviceType::SmartBatteryProtect) => {
            let data =
                parse_advertisement::<smart_battery_protect::SmartBatteryProtectData>(adv, key)?;
            Ok(DeviceData::SmartBatteryProtect(data))
        }
        Some(DeviceType::SmartLithium) => {
            let data = parse_advertisement::<smart_lithium::SmartLithiumData>(adv, key)?;
            Ok(DeviceData::SmartLithium(data))
        }
        Some(DeviceType::SolarCharger) => {
            let data = parse_advertisement::<solar_charger::SolarChargerData>(adv, key)?;
            Ok(DeviceData::SolarCharger(data))
        }
        Some(DeviceType::VEBus) => {
            let data = parse_advertisement::<vebus::VEBusData>(adv, key)?;
            Ok(DeviceData::VEBus(data))
        }
        None => Err(Error::InvalidAdvertisement),
    }
}

/// Enumeration of all device data types
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DeviceData {
    AcCharger(AcChargerData),
    BatteryMonitor(BatteryMonitorData),
    BatterySense(BatterySenseData),
    DcDcConverter(DcDcConverterData),
    DcEnergyMeter(DcEnergyMeterData),
    Inverter(InverterData),
    LynxSmartBMS(LynxSmartBMSData),
    OrionXS(OrionXSData),
    SmartBatteryProtect(SmartBatteryProtectData),
    SmartLithium(SmartLithiumData),
    SolarCharger(SolarChargerData),
    VEBus(VEBusData),
}
