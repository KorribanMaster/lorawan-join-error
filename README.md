# LoRa Experiment - Victron BLE to LoRaWAN Gateway

This project implements a gateway that reads Victron Energy device data via Bluetooth LE and transmits it over LoRaWAN. It's structured following the Ferrous Systems nested workspace pattern for embedded applications.

## Project Structure

The codebase is organized into a nested workspace structure that separates hardware-independent and hardware-dependent code:

```
lora-experiment/                      # Root workspace (host-testable)
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

## Architecture Benefits

This structure provides several advantages:

1. **Host-based Testing**: The `victron-protocol` library can be tested on your development machine using standard `cargo test`, enabling:
   - Fast test iteration
   - Use of standard Rust tooling
   - Property-based testing and fuzzing
   - ~70% of business logic is testable

2. **Clear Separation**: Hardware-dependent code (ESP32-S3 HAL, BLE, LoRa radio) is isolated in `cross/app`, while protocol logic is in `victron-protocol`

3. **Reusability**: The `victron-protocol` library could be used in other projects (desktop apps, web services, etc.)

4. **Standard Toolchain**: The root workspace uses standard Rust toolchain, only `cross/` requires ESP toolchain

## Building and Testing

### Run Host Tests

From the root directory:

```bash
# Run all tests for victron-protocol
cargo test -p victron-protocol

# Or use xtask
cargo xtask test
```

### Build Embedded Application

From the cross directory:

```bash
cd cross
cargo build --release --manifest-path app/Cargo.toml

# Or use xtask from root
cargo xtask build
```

### Flash to Device

```bash
cd cross
cargo run --release --manifest-path app/Cargo.toml
```

## Hardware Requirements

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

This project is for experimental and educational purposes.

## Resources

- [Ferrous Systems Testing Article](https://ferrous-systems.com/blog/test-embedded-app/)
- [Victron BLE Protocol](https://github.com/keshavdv/victron-ble)
- [LoRaWAN Specification](https://lora-alliance.org/resource_hub/lorawan-specification-v1-1/)
