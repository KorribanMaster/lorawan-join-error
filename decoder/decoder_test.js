// Test script for the improved payload decoder
// Run with: node decoder_test.js

var decoder = require('./decoder_improved.js');

console.log("=== Victron Energy LoRaWAN Payload Decoder Tests ===");
console.log("Format: [MAC(6), RSSI(1), Type(1), Data...]\n");

// Test 1: Solar Charger in Bulk mode
console.log("Test 1: Solar Charger");
var solarPayload = [
  0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0x11, // MAC: AA:BB:CC:DD:EE:11
  0xC5,                                // RSSI: -59 dBm
  0x01,                                // Type: Solar Charger
  0x34, 0x50, 0x04, 0xD2, 0x00, 0x05, 0xDC, 0x00, 0xC8, 0x03
];
var result1 = decoder.decodeUplink({ bytes: solarPayload });
console.log(JSON.stringify(result1, null, 2));
console.log("\n---\n");

// Test 2: Battery Monitor with Low Voltage alarm
console.log("Test 2: Battery Monitor with Low Voltage Alarm");
var batteryPayload = [
  0x11, 0x22, 0x33, 0x44, 0x55, 0x66, // MAC: 11:22:33:44:55:66
  0xB4,                                // RSSI: -76 dBm
  0x02,                                // Type: Battery Monitor
  0x32, 0xC8, 0xFF, 0x9C, 0x03, 0x20, 0x00, 0x78, 0xFF, 0xEC, 0x01
];
var result2 = decoder.decodeUplink({ bytes: batteryPayload });
console.log(JSON.stringify(result2, null, 2));
console.log("\n---\n");

// Test 3: DC-DC Converter - Off due to Protection + BMS
console.log("Test 3: DC-DC Converter (Off Reason: Protection + BMS)");
var dcdcPayload = [
  0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, // MAC: AB:CD:EF:12:34:56
  0xD0,                                // RSSI: -48 dBm
  0x04,                                // Type: DC-DC Converter
  0x34, 0x50, 0x32, 0xC8, 0x00, 0x00, 0x00, 0x50 // 0x50 = 0x10 | 0x40
];
var result3 = decoder.decodeUplink({ bytes: dcdcPayload });
console.log(JSON.stringify(result3, null, 2));
console.log("\n---\n");

// Test 4: Inverter in Float mode with multiple alarms
console.log("Test 4: Inverter in Float mode with Overload + High Temperature alarms");
var inverterPayload = [
  0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, // MAC: DE:AD:BE:EF:CA:FE
  0xBE,                                // RSSI: -66 dBm
  0x05,                                // Type: Inverter
  0x34, 0x50, 0x01, 0x2C, 0x05, 0x00, 0x00, 0x01, 0x40 // alarm = 0x140
];
var result4 = decoder.decodeUplink({ bytes: inverterPayload });
console.log(JSON.stringify(result4, null, 2));
console.log("\n---\n");

// Test 5: Smart Lithium
console.log("Test 5: Smart Lithium Battery");
var lithiumPayload = [
  0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, // MAC: 88:99:AA:BB:CC:DD
  0xC0,                                // RSSI: -64 dBm
  0x03,                                // Type: Smart Lithium
  0x33, 0x98, 0x19, 0x02, 0x00, 0x00  // 13.2V, 25°C, Balancing
];
var result5 = decoder.decodeUplink({ bytes: lithiumPayload });
console.log(JSON.stringify(result5, null, 2));
console.log("\n---\n");

// Test 6: DC-DC with "No Input Power" off reason
console.log("Test 6: DC-DC Converter (Off Reason: No Input Power)");
var dcdcOffPayload = [
  0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // MAC: 00:11:22:33:44:55
  0xA0,                                // RSSI: -96 dBm (weak signal)
  0x04,                                // Type: DC-DC Converter
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01
];
var result6 = decoder.decodeUplink({ bytes: dcdcOffPayload });
console.log(JSON.stringify(result6, null, 2));
console.log("\n---\n");

// Test 7: Empty payload (error case)
console.log("Test 7: Empty Payload (Error Case)");
var result7 = decoder.decodeUplink({ bytes: [] });
console.log(JSON.stringify(result7, null, 2));
console.log("\n---\n");

// Test 8: Too short payload (error case)
console.log("Test 8: Payload Too Short (Error Case)");
var result8 = decoder.decodeUplink({ bytes: [0xAA, 0xBB, 0xCC] });
console.log(JSON.stringify(result8, null, 2));

console.log("\n=== Key Features ===");
console.log("1. MAC address extraction for device identification");
console.log("2. RSSI (signal strength) included in decoded data");
console.log("3. Numeric values have corresponding '_str' fields with human-readable names");
console.log("4. Bitfield values (off_reason, alarm) have '_flags' arrays showing active conditions");
console.log("5. Round-robin multi-device support on the ESP32 side!");
