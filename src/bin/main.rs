#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use bt_hci::controller::ExternalController;
use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use esp_hal::{clock::CpuClock, timer::timg::TimerGroup};
use esp_radio::ble::controller::BleConnector;
use lora_experiment::{device::DeviceData, VictronScanner, VICTRON_MANUFACTURER_ID};

// Load Victron encryption keys
include!(concat!(env!("OUT_DIR"), "/victron_keys.rs"));

use static_cell::StaticCell;
use trouble_host::prelude::*;
use {esp_backtrace as _, esp_println as _};

extern crate alloc;

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

// Channel for sharing Victron device data between BLE scanner and LoRaWAN sender
static VICTRON_DATA_CHANNEL: StaticCell<Channel<CriticalSectionRawMutex, Option<DeviceData>, 10>> = StaticCell::new();

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

/// Event handler for BLE advertisements - processes Victron device data
struct VictronEventHandler {
    channel: &'static Channel<CriticalSectionRawMutex, Option<DeviceData>, 10>,
}

impl EventHandler for VictronEventHandler {
    fn on_adv_reports(&self, mut it: LeAdvReportsIter<'_>) {
        let scanner = VictronScanner::new(&ENCRYPTION_KEYS);

        while let Some(Ok(report)) = it.next() {
            // Get raw advertisement data
            let adv_data = report.data;

            // Parse through advertisement data to find manufacturer specific data (type 0xFF)
            let mut i = 0;
            while i < adv_data.len() {
                if i + 1 >= adv_data.len() {
                    break;
                }

                let length = adv_data[i] as usize;
                if length == 0 || i + length >= adv_data.len() {
                    break;
                }

                let ad_type = adv_data[i + 1];

                // Check for manufacturer specific data (0xFF)
                if ad_type == 0xFF && length >= 3 {
                    let data_start = i + 2;
                    let data_end = i + 1 + length;
                    let manufacturer_data = &adv_data[data_start..data_end];

                    if manufacturer_data.len() >= 3 {
                        let company_id =
                            u16::from_le_bytes([manufacturer_data[0], manufacturer_data[1]]);

                        if company_id == VICTRON_MANUFACTURER_ID {
                            info!(
                                "Found Victron device: {:x}, RSSI: {} dBm, trying to send to channel",
                                report.addr, report.rssi
                            );

                            // Parse manufacturer data (skip company ID)
                            match scanner.parse_manufacturer_data(&manufacturer_data[2..]) {
                                Ok(device_data) => {
                                    // Log detailed device data
                                    match &device_data {
                                        DeviceData::AcCharger(data) => {
                                            info!("AC Charger:");
                                            if let Some(v) = data.output_voltage1 {
                                                info!("  Output1: {} V", v as i32);
                                            }
                                            if let Some(state) = data.charge_state {
                                                info!("  State: {:?}", state);
                                            }
                                        }
                                        DeviceData::BatteryMonitor(data) => {
                                            info!("Battery Monitor:");
                                            info!("  Voltage: {} V", data.voltage as i32);
                                            info!("  Current: {} A", data.current as i32);
                                            if let Some(soc) = data.soc {
                                                info!("  SOC: {} %", soc as i32);
                                            }
                                            info!("  Consumed: {} Ah", data.consumed_ah as i32);
                                        }
                                        DeviceData::BatterySense(data) => {
                                            info!("Battery Sense:");
                                            info!("  Voltage: {} V", data.voltage as i32);
                                            if let Some(temp) = data.temperature {
                                                info!("  Temperature: {} C", temp as i32);
                                            }
                                        }
                                        DeviceData::DcDcConverter(data) => {
                                            info!("DC-DC Converter:");
                                            info!("  Input: {} V", data.input_voltage as i32);
                                            info!("  Output: {} V", data.output_voltage as i32);
                                            if let Some(state) = data.charge_state {
                                                info!("  State: {:?}", state);
                                            }
                                        }
                                        DeviceData::DcEnergyMeter(data) => {
                                            info!("DC Energy Meter:");
                                            if let Some(v) = data.voltage {
                                                info!("  Voltage: {} V", v as i32);
                                            }
                                            if let Some(c) = data.current {
                                                info!("  Current: {} A", c as i32);
                                            }
                                        }
                                        DeviceData::Inverter(data) => {
                                            info!("Inverter:");
                                            if let Some(v) = data.battery_voltage {
                                                info!("  Battery: {} V", v as i32);
                                            }
                                            if let Some(p) = data.ac_apparent_power {
                                                info!("  AC Power: {} VA", p);
                                            }
                                        }
                                        DeviceData::LynxSmartBMS(data) => {
                                            info!("Lynx Smart BMS:");
                                            if let Some(v) = data.voltage {
                                                info!("  Voltage: {} V", v as i32);
                                            }
                                            if let Some(soc) = data.soc {
                                                info!("  SOC: {} %", soc as i32);
                                            }
                                        }
                                        DeviceData::OrionXS(data) => {
                                            info!("Orion XS:");
                                            info!(
                                                "  Input: {} V @ {} A",
                                                data.input_voltage as i32, data.input_current as i32
                                            );
                                            info!(
                                                "  Output: {} V @ {} A",
                                                data.output_voltage as i32, data.output_current as i32
                                            );
                                            if let Some(state) = data.charge_state {
                                                info!("  State: {:?}", state);
                                            }
                                        }
                                        DeviceData::SmartBatteryProtect(data) => {
                                            info!("Smart Battery Protect:");
                                            if let Some(v) = data.input_voltage {
                                                info!("  Input: {} V", v as i32);
                                            }
                                            if let Some(state) = data.output_state {
                                                info!("  Output: {:?}", state);
                                            }
                                        }
                                        DeviceData::SmartLithium(data) => {
                                            info!("Smart Lithium:");
                                            if let Some(v) = data.battery_voltage {
                                                info!("  Voltage: {} V", v as i32);
                                            }
                                            if let Some(status) = data.balancer_status {
                                                info!("  Balancer: {:?}", status);
                                            }
                                        }
                                        DeviceData::SolarCharger(data) => {
                                            info!("Solar Charger:");
                                            info!(
                                                "  Battery: {} V @ {} A",
                                                data.battery_voltage as i32,
                                                data.battery_current as i32
                                            );
                                            info!("  PV Power: {} W", data.pv_power);
                                            info!("  Yield today: {} Wh", data.yield_today);
                                            if let Some(state) = data.charge_state {
                                                info!("  State: {:?}", state);
                                            }
                                        }
                                        DeviceData::VEBus(data) => {
                                            info!("VE.Bus:");
                                            if let Some(v) = data.battery_voltage {
                                                info!("  Battery: {} V", v as i32);
                                            }
                                            if let Some(soc) = data.soc {
                                                info!("  SOC: {} %", soc);
                                            }
                                        }
                                    }

                                    // Send to channel (non-blocking, overwrites old data)
                                    match self.channel.try_send(Some(device_data)) {
                                        Ok(_) => {
                                            info!("Successfully sent data to channel");
                                        }
                                        Err(e) => {
                                            warn!("Failed to send to channel: {:?}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse Victron data: {:?}", e);
                                }
                            }
                        } else {
                            // info!("Unknown manufacturer id {:x}", company_id);
                        }
                    }
                }

                i += 1 + length;
            }
        }
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.1

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    info!("Embassy initialized!");

    // Initialize Victron data channel
    let victron_channel = VICTRON_DATA_CHANNEL.init(Channel::new());

    // let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    // // find more examples https://github.com/embassy-rs/trouble/tree/main/examples/esp32
    // let transport = BleConnector::new(&radio_init, peripherals.BT, Default::default()).unwrap();
    // let ble_controller = ExternalController::<_, 20>::new(transport);

    // let resources = HOST_RESOURCES.init(HostResources::new());
    //
    // // Create scanner address for BLE
    // let address: Address = Address::random([0xff, 0x8f, 0x1b, 0x05, 0xe4, 0xff]);
    // let stack = trouble_host::new(ble_controller, resources)
    //     .set_random_address(address);
    // let Host {
    //     central, mut runner, ..
    // } = stack.build();

    let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    let transport = BleConnector::new(&radio_init, peripherals.BT, Default::default()).unwrap();
    let ble_controller = ExternalController::<_, 20>::new(transport);
    
    // Create BLE host resources
    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();

    // Use a random address for scanning
    let address: Address = Address::random([0xff, 0x8f, 0x1b, 0x05, 0xe4, 0xff]);
    info!("Scanner address = {:?}", address);

    let stack = trouble_host::new(ble_controller, &mut resources).set_random_address(address);

    let Host {
        central,
        mut runner,
        ..
    } = stack.build();
    info!("Bluetooth initialized with scanning support");

    // Spawn LoRaWAN task
    spawner.spawn(lora_experiment::lorawan::lorawan_task(victron_channel)).unwrap();
    info!("LoRaWAN task spawned");

    // Run BLE using join pattern (keeps everything in main's stack frame)
    // Create event handler for processing advertisements
    let handler = VictronEventHandler {
        channel: victron_channel,
    };
    let mut scanner = Scanner::new(central);

    let scan_config = ScanConfig {
        active: true,
        phys: PhySet::M1,
        interval: Duration::from_millis(100),
        window: Duration::from_millis(100),
        ..Default::default()
    };

    info!("Starting BLE scan for Victron devices...");

    // Run BLE runner and scanner concurrently using join
    // This never returns, keeping the main function alive
    let _ = join(
        runner.run_with_handler(&handler),
        async {
            loop {
                info!("Starting BLE scan session...");
                match scanner.scan(&scan_config).await {
                    Ok(session) => {
                        info!("BLE scan session started successfully");
                        // Keep scanning for 5 seconds, then restart to clear duplicate filter
                        Timer::after(Duration::from_secs(25)).await;
                        drop(session);
                        info!("Restarting scan to clear duplicate filter...");
                    }
                    Err(e) => {
                        warn!("Failed to start BLE scan: {:?}", e);
                        Timer::after(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    ).await;

    // This point is never reached
    unreachable!()
}
