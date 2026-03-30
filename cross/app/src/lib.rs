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
