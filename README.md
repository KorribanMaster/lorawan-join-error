# LoRa Experiment - Victron BLE to LoRaWAN Gateway

This project implements a gateway that reads Victron Energy device data via Bluetooth LE and transmits it over LoRaWAN. It's structured following the Ferrous Systems nested workspace pattern for embedded applications.

## Project Structure

The codebase is organized into a nested workspace structure that separates hardware-independent and hardware-dependent code:

```
victron-ttn-gateway/                      # Root workspace (host-testable)
├── Cargo.toml                        # Root workspace manifest
├── victron-protocol/                 # Hardware-independent protocol library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── bitreader.rs             # Bit-level parsing utilities
│       ├── crypto.rs                # AES-CTR encryption/decryption
│       ├── types.rs                 # Common enums and types
│       ├── device/                  # Device-specific parsers (14 types)
│       └── victron_payload.rs       # LoRaWAN payload encoding
├── cross/                            # Inner workspace (embedded)
│   ├── Cargo.toml                   # Cross workspace manifest
│   ├── rust-toolchain.toml          # ESP toolchain specification
│   ├── .cargo/config.toml           # ESP32-S3 build configuration
│   └── app/                         # Embedded application
│       ├── Cargo.toml
│       ├── build.rs
│       └── src/
│           ├── lib.rs               # Embedded utilities
│           ├── bin/main.rs          # Application entry point
│           ├── lorawan.rs           # LoRaWAN networking
│           └── scanner.rs           # BLE scanner integration
└── xtask/                            # Build orchestration tool
    └── src/main.rs
```

## Building and Testing

This project uses the `xtask` pattern for build orchestration. All commands can be run from the root directory without needing to change directories.

### Available Commands

```bash
# Run host tests only
cargo xtask test host

# Run all tests (host + embedded build verification)
cargo xtask test all

# Build embedded application (debug)
cargo xtask build

# Build embedded application (release)
cargo xtask build --release

# Flash to device (debug)
cargo xtask flash

# Flash to device (release)
cargo xtask flash --release
```

### Quick Start

```bash
# Test the protocol library
cargo xtask test host

# Build and flash the release version to your device
cargo xtask flash --release
```

## Hardware Requirements
- Heltec Lora32 v2

Or 
- ESP32-S3 development board
- SX1262 LoRa radio module connected via SPI:
  - NSS: GPIO8
  - SCK: GPIO9
  - MOSI: GPIO10
  - MISO: GPIO11
  - RESET: GPIO12
  - BUSY: GPIO13
  - DIO1: GPIO14

## Configuration

### Victron Encryption Keys

Create a `.env` file in the root directory:

```
VICTRON_KEY_{}= "0102030405060708090A0B0C0D0E0F10"
```

### LoRaWAN Credentials

Create a `.env` file in the root directory:

```
LORAWAN_DEVEUI="0000000000000000"
LORAWAN_APPEUI="0000000000000000"
LORAWAN_APPKEY="00000000000000000000000000000000"
```

## Supported Victron Devices

The protocol library supports parsing data from 14 different Victron device types:

- Solar Chargers (SmartSolar, BlueSolar)
- Battery Monitors (BMV series)
- Battery Sense
- DC-DC Converters (Orion, Orion-Tr)
- DC Energy Meters
- Inverters
- Lynx Smart BMS
- Orion XS
- Smart Battery Protect
- Smart Lithium
- AC Chargers
- VE.Bus devices

## Development

### Adding Tests

Tests should be added to the `victron-protocol` package where they can run on the host:

```rust
// In victron-protocol/src/crypto.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decryption() {
        // Test code here
    }
}
```

### Adding Device Support

1. Add parser in `victron-protocol/src/device/`
2. Add variant to `DeviceData` enum in `victron-protocol/src/device/mod.rs`
3. Add packing logic in `victron-protocol/src/victron_payload.rs`
4. Write tests in the parser module

## License

MIT

## Resources

- [Ferrous Systems Testing Article](https://ferrous-systems.com/blog/test-embedded-app/)
- [Victron BLE Protocol](https://github.com/keshavdv/victron-ble)
- [LoRaWAN Specification](https://lora-alliance.org/resource_hub/lorawan-specification-v1-1/)
