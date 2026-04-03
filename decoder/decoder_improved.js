// Enhanced LoRaWAN Payload Decoder for Victron Energy Devices
// Includes human-readable string representations for all enum values
//
// Payload Format (Version with MAC address and RSSI metadata):
// Bytes 0-5:   MAC address (6 bytes) - Device identifier
// Byte 6:      RSSI (signed 8-bit) - Signal strength in dBm
// Byte 7:      Device Type ID
// Bytes 8-N:   Device-specific data (varies by device type)
//
// Total overhead: 7 bytes (MAC + RSSI + Type) before device data

// ===== ENUM LOOKUP TABLES =====

var OPERATION_MODE = {
  0: "Off",
  1: "LowPower",
  2: "Fault",
  3: "Bulk",
  4: "Absorption",
  5: "Float",
  6: "Storage",
  7: "Equalize",
  9: "Inverting",
  11: "PowerSupply",
  245: "StartingUp",
  246: "RepeatedAbsorption",
  247: "AutoEqualize",
  248: "BatterySafe",
  252: "ExternalControl"
};

var CHARGER_ERROR = {
  0: "NoError",
  1: "BatteryTemperatureTooHigh",
  2: "BatteryVoltageTooHigh",
  3: "BatteryTemperatureSensorMiswired",
  4: "RemoteTemperatureSensorFailure",
  5: "RemoteTemperatureSensorMiswired",
  6: "RemoteVoltageSenseMiswired",
  7: "RemoteVoltageWireLost",
  17: "ChargerTemperatureTooHigh",
  18: "ChargerOverCurrent",
  19: "ChargerCurrentReversed",
  20: "BulkTimeLimitExceeded",
  21: "CurrentSensorIssue",
  26: "InternalTemperatureSensorFailure",
  27: "FanFailure",
  28: "InternalDCVoltageError",
  29: "InternalSupplyVoltageError",
  33: "InternalBatteryVoltageSensorError",
  34: "InternalDCVoltageSensorError",
  35: "PVInputShutdownExcessiveCurrent",
  36: "PVInputShutdownOverVoltage",
  38: "PVInputShutdown",
  39: "PVInputFailedToShutdown",
  65: "InverterShutdownPanelVoltage",
  66: "InverterShutdownVoltageRange",
  67: "InverterShutdownWiring",
  68: "InverterShutdownConverterIssue",
  114: "InverterShutdownOverCurrent",
  116: "InverterShutdownBatteryVoltage",
  117: "InverterShutdownHighBatteryVoltage",
  119: "InverterOverload",
  121: "CPUTemperatureTooHigh",
  200: "CommunicationLost",
  201: "SynchronizationCalibration",
  202: "BmsTempTransmitError",
  203: "BMSConnectionLost"
};

var BALANCER_STATUS = {
  0: "Unknown",
  1: "Balanced",
  2: "Balancing",
  3: "Imbalance"
};

// Off Reason bitfield flags
var OFF_REASON_FLAGS = {
  0x01: "NoInputPower",
  0x02: "SwitchedOffSwitch",
  0x04: "SwitchedOffRegister",
  0x08: "RemoteInput",
  0x10: "ProtectionActive",
  0x20: "Paygo",
  0x40: "BMS",
  0x80: "EngineShutdown",
  0x100: "AnalysingInputVoltage"
};

// Alarm Reason bitfield flags
var ALARM_REASON_FLAGS = {
  0x01: "LowVoltage",
  0x02: "HighVoltage",
  0x04: "LowSOC",
  0x08: "LowStarterVoltage",
  0x10: "HighStarterVoltage",
  0x20: "LowTemperature",
  0x40: "HighTemperature",
  0x80: "MidVoltage",
  0x100: "Overload",
  0x200: "DCRipple",
  0x400: "LowACOutVoltage",
  0x800: "HighACOutVoltage"
};

// ===== HELPER FUNCTIONS =====

/**
 * Decode a bitfield into an array of active flag names
 */
function decodeBitfield(value, flagMap) {
  if (value === 0) {
    return ["None"];
  }

  var activeFlags = [];
  for (var flag in flagMap) {
    if (value & parseInt(flag)) {
      activeFlags.push(flagMap[flag]);
    }
  }

  return activeFlags.length > 0 ? activeFlags : ["Unknown"];
}

/**
 * Add both numeric value and string representation to data object
 */
function addEnumValue(data, fieldName, numericValue, lookupTable) {
  data[fieldName] = numericValue;
  data[fieldName + "_str"] = lookupTable[numericValue] || "Unknown";
}

/**
 * Add bitfield value with decoded flags
 */
function addBitfieldValue(data, fieldName, numericValue, flagMap) {
  data[fieldName] = numericValue;
  data[fieldName + "_flags"] = decodeBitfield(numericValue, flagMap);
}

// ===== MAIN DECODER FUNCTION =====

function decodeUplink(input) {
  var bytes = input.bytes;
  var warnings = [];
  var errors = [];

  if (bytes.length === 0) {
    return {
      data: {},
      warnings: [],
      errors: ["Empty payload"]
    };
  }

  // Check if we have the minimum header (MAC + RSSI + Device Type)
  if (bytes.length < 8) {
    return {
      data: {},
      warnings: [],
      errors: ["Payload too short - expected at least 8 bytes (MAC + RSSI + Type), got " + bytes.length]
    };
  }

  // Extract MAC address (bytes 0-5)
  var macAddress = [];
  for (var i = 0; i < 6; i++) {
    macAddress.push(("0" + bytes[i].toString(16).toUpperCase()).slice(-2));
  }
  var macString = macAddress.join(":");

  // Extract RSSI (byte 6, signed)
  var rssi = (bytes[6] << 24 >> 24); // Sign extend

  // Extract device type (byte 7)
  var deviceType = bytes[7];

  var data = {
    mac_address: macString,
    rssi: rssi,
    device_type_id: deviceType
  };

  switch (deviceType) {
    case 0x01: // Solar Charger
      if (bytes.length < 18) { // 7 header + 11 data = 18
        errors.push("Solar Charger payload too short");
        break;
      }

      data.device_type = "SolarCharger";
      data.battery_voltage = ((bytes[8] << 8) | bytes[9]) / 1000.0; // V
      data.battery_current = (((bytes[10] << 8) | bytes[11]) << 16 >> 16) / 1000.0; // A (signed)
      data.yield_today = (bytes[12] << 16) | (bytes[13] << 8) | bytes[14]; // Wh
      data.pv_power = (bytes[15] << 8) | bytes[16]; // W

      if (bytes[17] === 0xFF) {
        data.charge_state = null;
        data.charge_state_str = null;
      } else {
        addEnumValue(data, "charge_state", bytes[17], OPERATION_MODE);
      }
      break;

    case 0x02: // Battery Monitor
      if (bytes.length < 19) { // 7 header + 12 data = 19
        errors.push("Battery Monitor payload too short");
        break;
      }

      data.device_type = "BatteryMonitor";
      data.voltage = ((bytes[8] << 8) | bytes[9]) / 1000.0; // V
      data.current = (((bytes[10] << 8) | bytes[11]) << 16 >> 16) / 1000.0; // A (signed)

      var soc = (bytes[12] << 8) | bytes[13];
      data.soc = soc === 0xFFFF ? null : soc / 10.0; // %

      var ttg = (bytes[14] << 8) | bytes[15];
      data.time_to_go = ttg === 0xFFFF ? null : ttg; // minutes

      data.consumed_ah = (((bytes[16] << 8) | bytes[17]) << 16 >> 16) / 10.0; // Ah (signed)

      addBitfieldValue(data, "alarm", bytes[18], ALARM_REASON_FLAGS);
      break;

    case 0x03: // Smart Lithium
      if (bytes.length < 14) { // 7 header + 7 data = 14
        errors.push("Smart Lithium payload too short");
        break;
      }

      data.device_type = "SmartLithium";
      var voltage = (bytes[8] << 8) | bytes[9];
      data.battery_voltage = voltage === 0 ? null : voltage / 1000.0; // V

      data.battery_temperature = bytes[10] === 0x80 ? null : (bytes[10] << 24 >> 24); // °C (signed)

      if (bytes[11] === 0xFF) {
        data.balancer_status = null;
        data.balancer_status_str = null;
      } else {
        addEnumValue(data, "balancer_status", bytes[11], BALANCER_STATUS);
      }

      var errorFlags = (bytes[12] << 8) | bytes[13];
      data.error_flags = errorFlags;
      // Note: Smart Lithium error flags are device-specific and not fully documented
      // in the standard Victron BLE spec, so we only provide the numeric value
      break;

    case 0x04: // DC-DC Converter
      if (bytes.length < 16) { // 7 header + 9 data = 16
        errors.push("DC-DC Converter payload too short");
        break;
      }

      data.device_type = "DcDcConverter";
      data.input_voltage = ((bytes[8] << 8) | bytes[9]) / 1000.0; // V
      data.output_voltage = (((bytes[10] << 8) | bytes[11]) << 16 >> 16) / 1000.0; // V (signed)

      var offReason = (bytes[12] << 24) | (bytes[13] << 16) | (bytes[14] << 8) | bytes[15];
      addBitfieldValue(data, "off_reason", offReason, OFF_REASON_FLAGS);
      break;

    case 0x05: // Inverter
      if (bytes.length < 17) { // 7 header + 10 data = 17
        errors.push("Inverter payload too short");
        break;
      }

      data.device_type = "Inverter";
      var batt_voltage = (bytes[8] << 8) | bytes[9];
      data.battery_voltage = batt_voltage === 0 ? null : batt_voltage / 1000.0; // V

      var power = (bytes[10] << 8) | bytes[11];
      data.ac_apparent_power = power === 0 ? null : power; // VA

      if (bytes[12] === 0xFF) {
        data.device_state = null;
        data.device_state_str = null;
      } else {
        addEnumValue(data, "device_state", bytes[12], OPERATION_MODE);
      }

      var alarm = (bytes[13] << 24) | (bytes[14] << 16) | (bytes[15] << 8) | bytes[16];
      addBitfieldValue(data, "alarm", alarm, ALARM_REASON_FLAGS);
      break;

    case 0xFF: // Unknown/Unsupported device
      data.device_type = "Unknown";
      warnings.push("Unknown or unsupported device type");
      break;

    default:
      data.device_type = "Unknown";
      warnings.push("Unrecognized device type: 0x" + deviceType.toString(16));
      break;
  }

  return {
    data: data,
    warnings: warnings,
    errors: errors
  };
}

// ===== EXAMPLE USAGE =====
// Test with a sample payload
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { decodeUplink };
}

// Example test cases (uncomment to test):
/*
// Solar Charger example with MAC and RSSI
// Format: [MAC(6 bytes), RSSI(1 byte), Type(1 byte), Data...]
var test1 = decodeUplink({
  bytes: [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0xC5, 0x01, 0x34, 0x50, 0x04, 0xD2, 0x00, 0x05, 0xDC, 0x00, 0xC8, 0x03]
});
console.log("Solar Charger:", JSON.stringify(test1, null, 2));
// Expected: MAC=AA:BB:CC:DD:EE:FF, RSSI=-59, voltage=13.392V, current=1.234A, yield=1500Wh, power=200W, state=Bulk

// Battery Monitor example with MAC and RSSI
var test2 = decodeUplink({
  bytes: [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0xB4, 0x02, 0x32, 0xC8, 0xFF, 0x9C, 0x03, 0x20, 0x00, 0x78, 0xFF, 0xEC, 0x01]
});
console.log("Battery Monitor:", JSON.stringify(test2, null, 2));
// Expected: MAC=11:22:33:44:55:66, RSSI=-76, voltage=13.0V, current=-10.0A, SOC=80%, TTG=120min, consumed=-2.0Ah

// DC-DC Converter with MAC, RSSI, and off reason
var test3 = decodeUplink({
  bytes: [0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0xD0, 0x04, 0x34, 0x50, 0x32, 0xC8, 0x00, 0x00, 0x00, 0x05]
});
console.log("DC-DC Converter:", JSON.stringify(test3, null, 2));
// Expected: MAC=AB:CD:EF:12:34:56, RSSI=-48, input=13.392V, output=13.0V, off_reason flags
*/
