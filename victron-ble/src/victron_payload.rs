//! Binary payload packing for sending Victron data over LoRaWAN
//!
//! LoRaWAN has limited payload sizes (typically 51 bytes for EU868).
//! This module efficiently packs Victron device telemetry into compact binary format.

use crate::device::DeviceData;

/// Maximum LoRaWAN payload size for EU868
pub const MAX_LORAWAN_PAYLOAD: usize = 51;

/// Pack Victron device data into a binary payload
///
/// Format:
/// - Byte 0: Device type ID
/// - Bytes 1-N: Device-specific data (packed efficiently)
///
/// Returns the number of bytes written to the output buffer
pub fn pack_device_data(device_data: &DeviceData, output: &mut [u8]) -> usize {
    if output.is_empty() {
        return 0;
    }

    match device_data {
        DeviceData::SolarCharger(data) => {
            // Format: [type_id, voltage_h, voltage_l, current_h, current_l, yield_h, yield_l, yield_m, yield_l, pv_power_h, pv_power_l, state]
            if output.len() < 12 {
                return 0;
            }
            output[0] = 0x01; // Solar Charger type ID

            // Pack voltage (mV) as 16-bit
            let voltage = (data.battery_voltage * 1000.0) as u16;
            output[1] = (voltage >> 8) as u8;
            output[2] = (voltage & 0xFF) as u8;

            // Pack current (mA) as signed 16-bit
            let current = (data.battery_current * 1000.0) as i16;
            output[3] = (current >> 8) as u8;
            output[4] = (current & 0xFF) as u8;

            // Pack yield today (Wh) as 32-bit (use 3 bytes for space efficiency)
            output[5] = (data.yield_today >> 16) as u8;
            output[6] = (data.yield_today >> 8) as u8;
            output[7] = (data.yield_today & 0xFF) as u8;

            // Pack PV power (W) as 16-bit
            output[8] = (data.pv_power >> 8) as u8;
            output[9] = (data.pv_power & 0xFF) as u8;

            // Pack state
            output[10] = data.charge_state.map(|s| s as u8).unwrap_or(0xFF);

            11
        }
        DeviceData::BatteryMonitor(data) => {
            // Format: [type_id, voltage_h, voltage_l, current_h, current_l, soc_h, soc_l, ttg_h, ttg_l, consumed_h, consumed_l, alarm]
            if output.len() < 12 {
                return 0;
            }
            output[0] = 0x02; // Battery Monitor type ID

            // Pack voltage (mV) as 16-bit
            let voltage = (data.voltage * 1000.0) as u16;
            output[1] = (voltage >> 8) as u8;
            output[2] = (voltage & 0xFF) as u8;

            // Pack current (mA) as signed 16-bit
            let current = (data.current * 1000.0) as i16;
            output[3] = (current >> 8) as u8;
            output[4] = (current & 0xFF) as u8;

            // Pack SOC (0.1% resolution) as 16-bit
            let soc = data.soc.map(|s| (s * 10.0) as u16).unwrap_or(0xFFFF);
            output[5] = (soc >> 8) as u8;
            output[6] = (soc & 0xFF) as u8;

            // Pack time to go (minutes) as 16-bit
            let ttg = data.time_to_go.unwrap_or(0xFFFF);
            output[7] = (ttg >> 8) as u8;
            output[8] = (ttg & 0xFF) as u8;

            // Pack consumed Ah (0.1 Ah resolution) as signed 16-bit
            let consumed = (data.consumed_ah * 10.0) as i16;
            output[9] = (consumed >> 8) as u8;
            output[10] = (consumed & 0xFF) as u8;

            // Pack alarm status
            output[11] = data.alarm as u8;

            12
        }
        DeviceData::SmartLithium(data) => {
            // Format: [type_id, voltage_h, voltage_l, temp, balancer_status, error_flags_h, error_flags_l]
            if output.len() < 7 {
                return 0;
            }
            output[0] = 0x03; // Smart Lithium type ID

            // Pack voltage (mV) as 16-bit - handle Option
            let voltage = data
                .battery_voltage
                .map(|v| (v * 1000.0) as u16)
                .unwrap_or(0);
            output[1] = (voltage >> 8) as u8;
            output[2] = (voltage & 0xFF) as u8;

            // Pack temperature as signed 8-bit (direct celsius value)
            output[3] = data.battery_temperature.unwrap_or(-128) as u8;

            // Pack balancer status
            output[4] = data.balancer_status.map(|s| s as u8).unwrap_or(0xFF);

            // Pack error flags (16-bit field)
            output[5] = (data.error_flags >> 8) as u8;
            output[6] = (data.error_flags & 0xFF) as u8;

            7
        }
        DeviceData::DcDcConverter(data) => {
            // Format: [type_id, input_v_h, input_v_l, output_v_h, output_v_l, off_reason_0, off_reason_1, off_reason_2, off_reason_3]
            if output.len() < 9 {
                return 0;
            }
            output[0] = 0x04; // DC-DC Converter type ID

            let input_v = (data.input_voltage * 1000.0) as u16;
            output[1] = (input_v >> 8) as u8;
            output[2] = (input_v & 0xFF) as u8;

            let output_v = (data.output_voltage * 1000.0) as i16; // Can be negative
            output[3] = (output_v >> 8) as u8;
            output[4] = (output_v & 0xFF) as u8;

            // Pack off_reason as 32-bit
            let off_reason = data.off_reason.0;
            output[5] = (off_reason >> 24) as u8;
            output[6] = (off_reason >> 16) as u8;
            output[7] = (off_reason >> 8) as u8;
            output[8] = (off_reason & 0xFF) as u8;

            9
        }
        DeviceData::Inverter(data) => {
            // Format: [type_id, voltage_h, voltage_l, ac_power_h, ac_power_l, state, alarm_h, alarm_l, alarm_m, alarm_n]
            if output.len() < 10 {
                return 0;
            }
            output[0] = 0x05; // Inverter type ID

            let voltage = data
                .battery_voltage
                .map(|v| (v * 1000.0) as u16)
                .unwrap_or(0);
            output[1] = (voltage >> 8) as u8;
            output[2] = (voltage & 0xFF) as u8;

            let power = data.ac_apparent_power.unwrap_or(0);
            output[3] = (power >> 8) as u8;
            output[4] = (power & 0xFF) as u8;

            output[5] = data.device_state.map(|s| s as u8).unwrap_or(0xFF);

            // Pack alarm reason as 32-bit
            let alarm = data.alarm.0;
            output[6] = (alarm >> 24) as u8;
            output[7] = (alarm >> 16) as u8;
            output[8] = (alarm >> 8) as u8;
            output[9] = (alarm & 0xFF) as u8;

            10
        }
        // Add more device types as needed
        _ => {
            // Unsupported device type
            output[0] = 0xFF;
            1
        }
    }
}

/// Pack Victron device data with MAC address and RSSI metadata
///
/// Format:
/// - Bytes 0-5: MAC address (6 bytes)
/// - Byte 6: RSSI (signed 8-bit)
/// - Byte 7: Device type ID
/// - Bytes 8-N: Device-specific data (packed efficiently)
///
/// Returns the number of bytes written to the output buffer
pub fn pack_device_with_metadata(
    mac_address: &[u8; 6],
    rssi: i8,
    device_data: &DeviceData,
    output: &mut [u8],
) -> usize {
    if output.len() < 8 {
        return 0;
    }

    // Pack MAC address (6 bytes)
    output[0..6].copy_from_slice(mac_address);

    // Pack RSSI (1 signed byte)
    output[6] = rssi as u8;

    // Pack device data starting at offset 7
    let device_bytes_written = pack_device_data(device_data, &mut output[7..]);

    if device_bytes_written == 0 {
        return 0;
    }

    7 + device_bytes_written
}

/// Unpack Victron device data from binary payload (for testing/verification)
///
/// This is mainly useful for debugging and verifying the packing format
#[allow(dead_code)]
pub fn unpack_device_data(payload: &[u8]) -> Option<alloc::string::String> {
    if payload.is_empty() {
        return None;
    }

    use alloc::format;
    use alloc::string::ToString;

    match payload[0] {
        0x01 => {
            // Solar Charger
            if payload.len() < 10 {
                return None;
            }
            let voltage = u16::from_be_bytes([payload[1], payload[2]]) as f32 / 1000.0;
            let current = i16::from_be_bytes([payload[3], payload[4]]) as f32 / 1000.0;
            let yield_today = u16::from_be_bytes([payload[5], payload[6]]);
            let pv_power = u16::from_be_bytes([payload[7], payload[8]]);
            let state = payload[9];
            Some(format!(
                "Solar: V={:.2}V I={:.2}A Yield={}Wh PV={}W State={}",
                voltage, current, yield_today, pv_power, state
            ))
        }
        0x02 => {
            // Battery Monitor
            if payload.len() < 12 {
                return None;
            }
            let voltage = u16::from_be_bytes([payload[1], payload[2]]) as f32 / 1000.0;
            let current = i16::from_be_bytes([payload[3], payload[4]]) as f32 / 1000.0;
            let soc = u16::from_be_bytes([payload[5], payload[6]]) as f32 / 10.0;
            let ttg = u16::from_be_bytes([payload[7], payload[8]]);
            Some(format!(
                "Battery: V={:.2}V I={:.2}A SOC={:.1}% TTG={}min",
                voltage, current, soc, ttg
            ))
        }
        0x03 => {
            // Smart Lithium
            if payload.len() < 7 {
                return None;
            }
            let voltage = u16::from_be_bytes([payload[1], payload[2]]) as f32 / 1000.0;
            let temp = i16::from_be_bytes([payload[3], payload[4]]) as f32 / 10.0;
            Some(format!("SmartLi: V={:.2}V T={:.1}°C", voltage, temp))
        }
        _ => Some("Unknown device".to_string()),
    }
}
