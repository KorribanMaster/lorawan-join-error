//! Battery voltage monitoring for Heltec LoRa32 v3.2
//!
//! This module handles reading battery voltage via ADC1 on GPIO1.
//! The Heltec v3.2 board has a voltage divider (390kΩ + 100kΩ) connected
//! to the battery, with GPIO37 as the enable pin (must be HIGH for v3.2).

use esp_hal::{
    analog::adc::{Adc, AdcChannel, AdcConfig, AdcPin, Attenuation},
    gpio::{AnalogPin, Level, Output, OutputConfig, OutputPin},
    peripherals::ADC1,
    Blocking,
};

/// Battery voltage threshold for critical low battery warning (3.3V)
pub const CRITICAL_BATTERY_VOLTAGE: f32 = 3.3;

/// Battery voltage threshold to resume normal operation after critical state (3.4V with hysteresis)
pub const BATTERY_RECOVERY_VOLTAGE: f32 = 3.4;

/// Voltage divider ratio for Heltec V3.2: (390kΩ + 100kΩ) / 100kΩ = 4.9
/// As per Heltec documentation: multiply ADC millivolts by 490/100
const VOLTAGE_DIVIDER_RATIO: f32 = 4.9;

/// ESP32-S3 internal ADC reference voltage (ESP32-S3 TRM Section 39.3.5)
/// Vref = 1100 mV by design (actual range: 1000-1200mV between chips)
const ADC_REFERENCE_VOLTAGE_MV: f32 = 1100.0;

/// Maximum ADC value for 12-bit resolution
const ADC_MAX_VALUE: f32 = 4095.0;

/// ADC calibration factor for 0dB attenuation
///
/// ESP32-S3 TRM Formula (Section 39.3.5):
///   Vdata = (Vref / k) × (data / 4095)
///   where k ≈ 100% (1.0) for 0dB attenuation
///
/// Applied to battery measurement with 4.9x voltage divider:
///   voltage_at_adc_pin = (Vref / k) × (data / 4095)
///                      = 1100mV × (data / 4095)           [at 0dB, k=1.0]
///   battery_voltage = voltage_at_adc_pin × 4.9
///                   = 1100mV × (data / 4095) × 4.9
///                   = (data / 4095) × 5390mV
///                   = data / (4095 / 5.39) volts
///                   = data / 759.4 volts
///
/// Therefore: VOLTAGE_CALIBRATION_FACTOR = 4095 / (1.1 × 4.9) ≈ 759.4
const VOLTAGE_CALIBRATION_FACTOR: f32 = ADC_MAX_VALUE / ((ADC_REFERENCE_VOLTAGE_MV / 1000.0) * VOLTAGE_DIVIDER_RATIO);

/// Battery monitor structure
pub struct BatteryMonitor<'d, PIN> {
    adc: Adc<'d, ADC1<'d>, Blocking>,
    adc_pin: AdcPin<PIN, ADC1<'d>>,
    ctrl_pin: Output<'d>,
}

impl<'d, PIN> BatteryMonitor<'d, PIN>
where
    PIN: AdcChannel + AnalogPin,
{
    /// Create a new battery monitor
    ///
    /// # Arguments
    /// * `adc1` - ADC1 peripheral
    /// * `gpio1` - GPIO1 for ADC reading (VBAT_ADC)
    /// * `gpio37` - GPIO37 for ADC control (VBAT_CTRL, must be HIGH for v3.2)
    pub fn new<CTRL: OutputPin + 'd>(
        adc1: ADC1<'d>,
        gpio1: PIN,
        gpio37: CTRL,
    ) -> Self {
        // Initialize ADC with default configuration
        let mut adc_config = AdcConfig::new();

        // Configure ADC pin with 0dB attenuation (0-1.1V range)
        // Perfect for battery voltage divider: 3.0-4.2V battery → 612-857mV at ADC pin
        let adc_pin = adc_config.enable_pin(gpio1, Attenuation::_0dB);
        

        // Create ADC instance
        let adc = Adc::new(adc1, adc_config);

        // Configure control pin (start LOW to save power, set HIGH only when reading)
        let ctrl_pin = Output::new(gpio37, Level::Low, OutputConfig::default());

        Self {
            adc,
            adc_pin,
            ctrl_pin,
        }
    }

    /// Read the current battery voltage
    ///
    /// Returns voltage in volts (e.g., 3.7 for a nominal LiPo battery)
    pub async fn read_voltage(&mut self) -> f32 {
        // Enable voltage divider by setting control pin HIGH
        self.ctrl_pin.set_low();


        embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;
        // Read ADC value using oneshot mode
        let voltage = match nb::block!(self.adc.read_oneshot(&mut self.adc_pin)) {
            Ok(reading) => {
                // Apply calibration factor
                (reading as f32) / VOLTAGE_CALIBRATION_FACTOR
            }
            Err(_) => {
                // If ADC read fails, return 0.0
                0.0
            }
        };

        // Disable voltage divider to save power
        self.ctrl_pin.set_high();

        voltage
    }

    /// Check if battery is critically low
    ///
    /// Returns true if voltage is below the critical threshold (3.3V)
    pub async fn is_critical(&mut self) -> bool {
        let voltage = self.read_voltage().await;
        voltage > 0.0 && voltage < CRITICAL_BATTERY_VOLTAGE
    }

    /// Check if battery has recovered from critical state
    ///
    /// Returns true if voltage is above recovery threshold (3.4V)
    /// This provides hysteresis to prevent bouncing between states
    pub async fn has_recovered(&mut self) -> bool {
        let voltage = self.read_voltage().await;
        voltage >= BATTERY_RECOVERY_VOLTAGE
    }

    /// Read battery voltage in millivolts
    ///
    /// Convenience method for protocol encoding
    pub async fn read_voltage_mv(&mut self) -> u16 {
        (self.read_voltage().await * 1000.0) as u16
    }
}
