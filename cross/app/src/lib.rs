#![no_std]

extern crate alloc;

use rand::Rng as _;

// Re-export victron-protocol for convenience
pub use victron_protocol::*;

// LoRaWAN module
pub mod lorawan;
// BLE scanner module
pub mod scanner;

// Victron constants
pub const PRODUCT_ADVERTISEMENT_TYPE: u8 = 0x10;

/// Generate "jittered" delay for retry attempts up to maximum of 1 hour
pub fn generate_delay<R: rand::RngCore>(rng: &mut R, retries: u32) -> u32 {
    let base = core::cmp::min(10 + (10 * retries), 3600);
    let jitter = base / 5;
    (base - jitter).saturating_add(rng.gen_range(jitter..=2 * jitter))
}

/// Simple timer implementation using embassy_time for LoRaWAN device
pub struct SimpleTimer {
    start: embassy_time::Instant,
}

impl SimpleTimer {
    pub fn new() -> Self {
        Self {
            start: embassy_time::Instant::now(),
        }
    }
}

impl Default for SimpleTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl lorawan_device::async_device::radio::Timer for SimpleTimer {
    fn reset(&mut self) {
        self.start = embassy_time::Instant::now();
    }

    async fn at(&mut self, millis: u64) {
        let target = self.start + embassy_time::Duration::from_millis(millis);
        embassy_time::Timer::at(target).await;
    }

    async fn delay_ms(&mut self, millis: u64) {
        embassy_time::Timer::after(embassy_time::Duration::from_millis(millis)).await;
    }
}

/// Entry for a single tracked Victron device
pub struct VictronDeviceEntry {
    pub mac_address: [u8; 6], // We'll manually populate this from defmt output or hash
    pub rssi: i8,
    pub data: device::DeviceData,
    pub last_updated: u64,
    pub valid: bool,
    pub addr_hash: u32, // Hash of the BLE address for identification
}

impl VictronDeviceEntry {
    fn empty() -> Self {
        Self {
            mac_address: [0; 6],
            rssi: 0,
            data: device::DeviceData::SolarCharger(device::SolarChargerData {
                charge_state: None,
                battery_voltage: 0.0,
                battery_current: 0.0,
                yield_today: 0,
                pv_power: 0,
                load_current: None,
                error: None,
            }),
            last_updated: 0,
            valid: false,
            addr_hash: 0,
        }
    }
}

/// Storage for tracking up to 10 Victron devices
pub struct VictronDeviceStorage {
    devices: [VictronDeviceEntry; 10],
    next_transmit_index: usize,
}

impl VictronDeviceStorage {
    pub fn new() -> Self {
        Self {
            devices: core::array::from_fn(|_| VictronDeviceEntry::empty()),
            next_transmit_index: 0,
        }
    }

    /// Find device by address hash or allocate a new slot
    pub fn find_or_allocate_slot(&mut self, addr_hash: u32) -> Option<usize> {
        // First, look for existing device by hash
        for (i, entry) in self.devices.iter().enumerate() {
            if entry.valid && entry.addr_hash == addr_hash {
                return Some(i);
            }
        }

        // Look for empty slot
        for (i, entry) in self.devices.iter().enumerate() {
            if !entry.valid {
                return Some(i);
            }
        }

        // Find oldest device to replace (LRU)
        let mut oldest_index = 0;
        let mut oldest_time = u64::MAX;
        for (i, entry) in self.devices.iter().enumerate() {
            if entry.last_updated < oldest_time {
                oldest_time = entry.last_updated;
                oldest_index = i;
            }
        }
        Some(oldest_index)
    }

    /// Update device data
    pub fn update_device(
        &mut self,
        mac: [u8; 6],
        rssi: i8,
        data: device::DeviceData,
        timestamp: u64,
        addr_hash: u32,
    ) {
        if let Some(slot) = self.find_or_allocate_slot(addr_hash) {
            self.devices[slot] = VictronDeviceEntry {
                mac_address: mac,
                rssi,
                data,
                last_updated: timestamp,
                valid: true,
                addr_hash,
            };
        }
    }

    /// Get next valid device for round-robin transmission
    /// Returns reference to entry and advances the index
    pub fn get_next_valid_device(&mut self) -> Option<&VictronDeviceEntry> {
        // Try to find a valid device starting from next_transmit_index
        for _ in 0..10 {
            let index = self.next_transmit_index;

            // Move to next index for next call
            self.next_transmit_index = (self.next_transmit_index + 1) % 10;

            if self.devices[index].valid {
                return Some(&self.devices[index]);
            }
        }

        // No valid devices found
        None
    }
}
