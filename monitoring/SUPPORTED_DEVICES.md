# Supported Victron Device Types

The TTN ingestor now supports **all 12 Victron device types** with comprehensive data tracking.

## Device Support Matrix

| Device Type | InfluxDB Measurement | Status | Fields Tracked |
|-------------|---------------------|--------|----------------|
| Battery Monitor (BMV) | `battery_monitor` | ✅ Full | 6 fields |
| Solar Charger (MPPT) | `solar_charger` | ✅ Full | 5 fields |
| DC-DC Converter | `dc_dc_converter` | ✅ Full | 4 fields |
| AC Charger | `ac_charger` | ✅ Full | 9 fields |
| Battery Sense | `battery_sense` | ✅ Full | 2 fields |
| DC Energy Meter | `dc_energy_meter` | ✅ Full | 3 fields |
| Inverter | `inverter` | ✅ Full | 4 fields |
| Lynx Smart BMS | `lynx_smart_bms` | ✅ Full | 6 fields |
| Orion XS | `orion_xs` | ✅ Full | 4 fields |
| Smart Battery Protect | `smart_battery_protect` | ✅ Full | 3 fields |
| Smart Lithium | `smart_lithium` | ✅ Full | 3 fields |
| VE.Bus | `vebus` | ✅ Full | 6 fields |

---

## 1. Battery Monitor (BMV series)

**Examples**: BMV-712, BMV-700, BMV-702

**Measurement**: `battery_monitor`

**Fields**:
- `voltage` (V) - Battery voltage
- `current` (A) - Battery current (+ charging, - discharging)
- `soc` (%) - State of charge (0-100%)
- `consumed_ah` (Ah) - Consumed amp-hours
- `device_rssi` (dBm) - BLE signal strength
- Additional: `time_to_go`, `alarm`, `aux_input`

**Example JSON from TTN**:
```json
{
  "device_type": "BatteryMonitor",
  "voltage": 12.65,
  "current": -5.2,
  "soc": 87.5,
  "consumed_ah": 12.3,
  "rssi": -68
}
```

---

## 2. Solar Charger (MPPT series)

**Examples**: SmartSolar MPPT 75/15, 100/30, 150/35

**Measurement**: `solar_charger`

**Fields**:
- `battery_voltage` (V) - Battery voltage
- `battery_current` (A) - Charging current
- `pv_power` (W) - Solar panel power
- `yield_today` (Wh) - Energy harvested today
- `device_rssi` (dBm) - BLE signal strength
- Additional: `charge_state`, `load_current`, `error`

**Example JSON**:
```json
{
  "device_type": "SolarCharger",
  "battery_voltage": 13.8,
  "battery_current": 8.5,
  "pv_power": 125,
  "yield_today": 850,
  "rssi": -65
}
```

---

## 3. DC-DC Converter

**Examples**: Basic Orion-Tr converters

**Measurement**: `dc_dc_converter`

**Fields**:
- `input_voltage` (V) - Input voltage
- `output_voltage` (V) - Output voltage
- `off_reason` - Reason code if off
- `device_rssi` (dBm) - BLE signal strength

**Example JSON**:
```json
{
  "device_type": "DcDcConverter",
  "input_voltage": 13.98,
  "output_voltage": 14.099,
  "off_reason": 0,
  "rssi": -60
}
```

---

## 4. AC Charger

**Examples**: Phoenix Charger, Skylla-TG

**Measurement**: `ac_charger`

**Fields**:
- `output_voltage1` (V) - Output 1 voltage
- `output_current1` (A) - Output 1 current
- `output_voltage2` (V) - Output 2 voltage (if present)
- `output_current2` (A) - Output 2 current
- `output_voltage3` (V) - Output 3 voltage (if present)
- `output_current3` (A) - Output 3 current
- `temperature` (°C) - Device temperature
- `ac_current` (A) - AC input current
- Additional: `charge_state`, `charger_error`

**Example JSON**:
```json
{
  "device_type": "AcCharger",
  "output_voltage1": 14.2,
  "output_current1": 25.5,
  "temperature": 45,
  "ac_current": 3.2
}
```

---

## 5. Battery Sense

**Examples**: Smart Battery Sense

**Measurement**: `battery_sense`

**Fields**:
- `voltage` (V) - Battery voltage
- `temperature` (°C) - Battery temperature

**Example JSON**:
```json
{
  "device_type": "BatterySense",
  "voltage": 12.75,
  "temperature": 22.5
}
```

---

## 6. DC Energy Meter

**Examples**: SmartShunt

**Measurement**: `dc_energy_meter`

**Fields**:
- `voltage` (V) - DC voltage
- `current` (A) - DC current
- `power` (W) - DC power

**Example JSON**:
```json
{
  "device_type": "DcEnergyMeter",
  "voltage": 12.8,
  "current": 15.2,
  "power": 194
}
```

---

## 7. Inverter

**Examples**: Phoenix Inverter

**Measurement**: `inverter`

**Fields**:
- `battery_voltage` (V) - DC battery voltage
- `ac_apparent_power` (VA) - AC apparent power
- `ac_voltage` (V) - AC output voltage
- `ac_current` (A) - AC output current
- Additional: `device_state`, `alarm`

**Example JSON**:
```json
{
  "device_type": "Inverter",
  "battery_voltage": 12.6,
  "ac_apparent_power": 500,
  "ac_voltage": 230.5,
  "ac_current": 2.2
}
```

---

## 8. Lynx Smart BMS

**Examples**: Lynx Smart BMS

**Measurement**: `lynx_smart_bms`

**Fields**:
- `voltage` (V) - Battery voltage
- `current` (A) - Battery current
- `soc` (%) - State of charge
- `consumed_ah` (Ah) - Consumed amp-hours
- `temperature` (°C) - Ambient temperature
- `battery_temperature` (°C) - Battery temperature

**Example JSON**:
```json
{
  "device_type": "LynxSmartBMS",
  "voltage": 51.2,
  "current": 12.5,
  "soc": 92.0,
  "consumed_ah": 8.5,
  "temperature": 25.0,
  "battery_temperature": 28.5
}
```

---

## 9. Orion XS

**Examples**: Orion XS 12/12-50

**Measurement**: `orion_xs`

**Fields**:
- `input_voltage` (V) - Input voltage
- `input_current` (A) - Input current
- `output_voltage` (V) - Output voltage
- `output_current` (A) - Output current
- Additional: `charge_state`, `charger_error`, `off_reason`

**Example JSON**:
```json
{
  "device_type": "OrionXS",
  "input_voltage": 13.8,
  "input_current": 42.5,
  "output_voltage": 14.2,
  "output_current": 40.2
}
```

---

## 10. Smart Battery Protect

**Examples**: Smart BatteryProtect 65, 100, 220

**Measurement**: `smart_battery_protect`

**Fields**:
- `input_voltage` (V) - Input voltage
- `output_voltage` (V) - Output voltage
- `error_code` - Error/warning code
- Additional: `output_state`

**Example JSON**:
```json
{
  "device_type": "SmartBatteryProtect",
  "input_voltage": 12.8,
  "output_voltage": 12.75,
  "error_code": 0
}
```

---

## 11. Smart Lithium

**Examples**: Smart Lithium batteries

**Measurement**: `smart_lithium`

**Fields**:
- `battery_voltage` (V) - Battery voltage
- `cell_count` - Number of cells
- `temperature` (°C) - Battery temperature
- Additional: `balancer_status`

**Example JSON**:
```json
{
  "device_type": "SmartLithium",
  "battery_voltage": 13.2,
  "cell_count": 4,
  "temperature": 24.5
}
```

---

## 12. VE.Bus

**Examples**: MultiPlus, Quattro, MultiGrid

**Measurement**: `vebus`

**Fields**:
- `battery_voltage` (V) - DC battery voltage
- `battery_current` (A) - Battery current
- `soc` (%) - State of charge
- `ac_in_power` (W) - AC input power
- `ac_out_power` (W) - AC output power
- `battery_temperature` (°C) - Battery temperature
- Additional: `device_state`, `error`, `ac_in_state`, `alarm`

**Example JSON**:
```json
{
  "device_type": "VEBus",
  "battery_voltage": 12.7,
  "battery_current": 5.2,
  "soc": 85,
  "ac_in_power": 250,
  "ac_out_power": 180,
  "battery_temperature": 26
}
```

---

## Signal Quality Tracking (All Devices)

**Measurement**: `signal_quality`

**Tags**: `device_id`, `gateway_id`

**Fields**:
- `rssi` (dBm) - Received Signal Strength Indicator
- `snr` (dB) - Signal-to-Noise Ratio

---

## Network Metrics (All Devices)

**Measurement**: `network_info`

**Tag**: `device_id`

**Fields**:
- `spreading_factor` - LoRa SF (7-12)
- `bandwidth` (Hz) - LoRa bandwidth

---

## TTN Payload Decoder Requirements

Your TTN application must have a payload decoder that outputs JSON with:

1. **Required**: `device_type` field matching one of the types above
2. **Optional**: Relevant data fields for that device type
3. **Optional**: `rssi` field for device-level RSSI

**Example decoder output**:
```javascript
function decodeUplink(input) {
  return {
    data: {
      device_type: "BatteryMonitor",
      voltage: 12.65,
      current: -5.2,
      soc: 87.5,
      consumed_ah: 12.3,
      rssi: -68
    }
  };
}
```

---

## Querying Data in Grafana

Use Flux queries to visualize data:

```flux
// Battery voltage over time
from(bucket: "victron-data")
  |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
  |> filter(fn: (r) => r._measurement == "battery_monitor")
  |> filter(fn: (r) => r._field == "voltage")

// Solar power over time
from(bucket: "victron-data")
  |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
  |> filter(fn: (r) => r._measurement == "solar_charger")
  |> filter(fn: (r) => r._field == "pv_power")

// Latest SOC
from(bucket: "victron-data")
  |> range(start: -1h)
  |> filter(fn: (r) => r._measurement == "battery_monitor")
  |> filter(fn: (r) => r._field == "soc")
  |> last()
```

---

## Unknown Devices

If the `device_type` doesn't match any of the above, data is stored as:

**Measurement**: `unknown_device`

**Tags**: `device_id`, `device_type`

**Field**: `raw_json` - Complete JSON payload

This ensures no data is lost even if new device types are added.
