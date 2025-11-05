#![no_std]

use rand::Rng as _;

// Generate "jittered" delay for retry attempts up to maximum of 1 hour
pub fn generate_delay<R: rand::RngCore>(rng: &mut R, retries: u32) -> u32 {
    let base = core::cmp::min(10 + (10 * retries), 3600);
    let jitter = base / 5;
    (base - jitter).saturating_add(rng.gen_range(jitter..=2 * jitter))
}
