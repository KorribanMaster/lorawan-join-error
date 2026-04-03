use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures::StreamExt;
use influxdb2::models::DataPoint;
use influxdb2::Client;
use reqwest;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use tracing::{debug, error, info, warn};

// TTN API Response structures
#[derive(Debug, Deserialize)]
struct TtnMessage {
    result: UplinkResult,
}

#[derive(Debug, Deserialize)]
struct UplinkResult {
    end_device_ids: EndDeviceIds,
    received_at: String,
    uplink_message: UplinkMessage,
}

#[derive(Debug, Deserialize)]
struct EndDeviceIds {
    device_id: String,
    #[serde(default)]
    dev_eui: Option<String>,
    #[serde(default)]
    dev_addr: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UplinkMessage {
    #[serde(default)]
    f_port: Option<u8>,
    #[serde(default)]
    f_cnt: Option<u32>,
    #[serde(default)]
    frm_payload: Option<String>,
    #[serde(default)]
    decoded_payload: Option<serde_json::Value>,
    #[serde(default)]
    rx_metadata: Vec<RxMetadata>,
    #[serde(default)]
    settings: Option<Settings>,
}

#[derive(Debug, Deserialize)]
struct RxMetadata {
    gateway_ids: GatewayIds,
    #[serde(default)]
    rssi: Option<i32>,
    #[serde(default)]
    channel_rssi: Option<i32>,
    #[serde(default)]
    snr: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct GatewayIds {
    gateway_id: String,
    #[serde(default)]
    eui: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Settings {
    #[serde(default)]
    data_rate: Option<DataRate>,
    #[serde(default)]
    frequency: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DataRate {
    #[serde(default)]
    lora: Option<LoRaSettings>,
}

#[derive(Debug, Deserialize)]
struct LoRaSettings {
    #[serde(default)]
    bandwidth: Option<u32>,
    #[serde(default)]
    spreading_factor: Option<u8>,
    #[serde(default)]
    coding_rate: Option<String>,
}

struct TtnIngestor {
    influx_client: Client,
    ttn_app_id: String,
    ttn_api_key: String,
    ttn_region: String,
    bucket: String,
}

impl TtnIngestor {
    fn new() -> Result<Self> {
        let influx_url = env::var("INFLUXDB_URL").unwrap_or_else(|_| "http://localhost:8086".to_string());
        let influx_token = env::var("INFLUXDB_TOKEN").context("INFLUXDB_TOKEN not set")?;
        let influx_org = env::var("INFLUXDB_ORG").unwrap_or_else(|_| "victron-monitoring".to_string());
        let bucket = env::var("INFLUXDB_BUCKET").unwrap_or_else(|_| "victron-data".to_string());

        let ttn_app_id = env::var("TTN_APP_ID").unwrap_or_else(|_| "vanman".to_string());
        let ttn_api_key = env::var("TTN_API_KEY").context("TTN_API_KEY not set")?;
        let ttn_region = env::var("TTN_REGION").unwrap_or_else(|_| "eu1".to_string());

        let influx_client = Client::new(&influx_url, &influx_org, &influx_token);

        info!("Initialized TTN Ingestor");
        info!("  InfluxDB: {}", influx_url);
        info!("  TTN App: {} ({})", ttn_app_id, ttn_region);
        info!("  Bucket: {}", bucket);

        Ok(Self {
            influx_client,
            ttn_app_id,
            ttn_api_key,
            ttn_region,
            bucket,
        })
    }

    async fn process_uplink(&self, message: TtnMessage) -> Result<()> {
        let device_id = &message.result.end_device_ids.device_id;
        let timestamp = DateTime::parse_from_rfc3339(&message.result.received_at)
            .context("Failed to parse timestamp")?
            .with_timezone(&Utc);

        debug!("Processing uplink from {}", device_id);

        let uplink = &message.result.uplink_message;

        // Write signal quality metrics from each gateway
        for rx_meta in &uplink.rx_metadata {
            if let (Some(rssi), Some(snr)) = (rx_meta.rssi, rx_meta.snr) {
                let point = DataPoint::builder("signal_quality")
                    .tag("device_id", device_id)
                    .tag("gateway_id", &rx_meta.gateway_ids.gateway_id)
                    .field("rssi", rssi as i64)
                    .field("snr", snr)
                    .timestamp(timestamp.timestamp_nanos_opt().unwrap_or(0))
                    .build()?;

                self.influx_client
                    .write(&self.bucket, futures::stream::once(async { point }))
                    .await
                    .context("Failed to write signal quality")?;
            }
        }

        // Write network settings
        if let Some(settings) = &uplink.settings {
            if let Some(data_rate) = &settings.data_rate {
                if let Some(lora) = &data_rate.lora {
                    let mut point_builder = DataPoint::builder("network_info")
                        .tag("device_id", device_id)
                        .timestamp(timestamp.timestamp_nanos_opt().unwrap_or(0));

                    if let Some(sf) = lora.spreading_factor {
                        point_builder = point_builder.field("spreading_factor", sf as i64);
                    }
                    if let Some(bw) = lora.bandwidth {
                        point_builder = point_builder.field("bandwidth", bw as i64);
                    }

                    let point = point_builder.build()?;
                    self.influx_client
                        .write(&self.bucket, futures::stream::once(async { point }))
                        .await
                        .context("Failed to write network info")?;
                }
            }
        }

        // Write decoded device data
        if let Some(decoded) = &uplink.decoded_payload {
            self.write_device_data(device_id, decoded, timestamp).await?;
        }

        info!("✓ Processed uplink from {}", device_id);
        Ok(())
    }

    async fn write_device_data(
        &self,
        device_id: &str,
        decoded: &serde_json::Value,
        timestamp: DateTime<Utc>,
    ) -> Result<()> {
        let device_type = decoded
            .get("device_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        debug!("Device type: {}", device_type);

        match device_type {
            "BatteryMonitor" => {
                let mut point = DataPoint::builder("battery_monitor")
                    .tag("device_id", device_id)
                    .timestamp(timestamp.timestamp_nanos_opt().unwrap_or(0));

                // Add fields based on decoded payload
                if let Some(voltage) = decoded.get("voltage").and_then(|v| v.as_f64()) {
                    point = point.field("voltage", voltage);
                }
                if let Some(current) = decoded.get("current").and_then(|v| v.as_f64()) {
                    point = point.field("current", current);
                }
                if let Some(soc) = decoded.get("soc").and_then(|v| v.as_f64()) {
                    point = point.field("soc", soc);
                }
                if let Some(consumed_ah) = decoded.get("consumed_ah").and_then(|v| v.as_f64()) {
                    point = point.field("consumed_ah", consumed_ah);
                }
                if let Some(rssi) = decoded.get("rssi").and_then(|v| v.as_i64()) {
                    point = point.field("device_rssi", rssi);
                }

                let data_point = point.build()?;
                self.influx_client
                    .write(&self.bucket, futures::stream::once(async { data_point }))
                    .await
                    .context("Failed to write battery monitor data")?;
            }
            "SolarCharger" => {
                let mut point = DataPoint::builder("solar_charger")
                    .tag("device_id", device_id)
                    .timestamp(timestamp.timestamp_nanos_opt().unwrap_or(0));

                if let Some(battery_voltage) = decoded.get("battery_voltage").and_then(|v| v.as_f64()) {
                    point = point.field("battery_voltage", battery_voltage);
                }
                if let Some(battery_current) = decoded.get("battery_current").and_then(|v| v.as_f64()) {
                    point = point.field("battery_current", battery_current);
                }
                if let Some(pv_power) = decoded.get("pv_power").and_then(|v| v.as_i64()) {
                    point = point.field("pv_power", pv_power);
                }
                if let Some(yield_today) = decoded.get("yield_today").and_then(|v| v.as_i64()) {
                    point = point.field("yield_today", yield_today);
                }
                if let Some(rssi) = decoded.get("rssi").and_then(|v| v.as_i64()) {
                    point = point.field("device_rssi", rssi);
                }

                let data_point = point.build()?;
                self.influx_client
                    .write(&self.bucket, futures::stream::once(async { data_point }))
                    .await
                    .context("Failed to write solar charger data")?;
            }
            "DcDcConverter" => {
                let mut point = DataPoint::builder("dc_dc_converter")
                    .tag("device_id", device_id)
                    .timestamp(timestamp.timestamp_nanos_opt().unwrap_or(0));

                if let Some(input_voltage) = decoded.get("input_voltage").and_then(|v| v.as_f64()) {
                    point = point.field("input_voltage", input_voltage);
                }
                if let Some(output_voltage) = decoded.get("output_voltage").and_then(|v| v.as_f64()) {
                    point = point.field("output_voltage", output_voltage);
                }
                if let Some(off_reason) = decoded.get("off_reason").and_then(|v| v.as_i64()) {
                    point = point.field("off_reason", off_reason);
                }
                if let Some(rssi) = decoded.get("rssi").and_then(|v| v.as_i64()) {
                    point = point.field("device_rssi", rssi);
                }

                let data_point = point.build()?;
                self.influx_client
                    .write(&self.bucket, futures::stream::once(async { data_point }))
                    .await
                    .context("Failed to write DC-DC converter data")?;
            }
            "AcCharger" => {
                let mut point = DataPoint::builder("ac_charger")
                    .tag("device_id", device_id)
                    .timestamp(timestamp.timestamp_nanos_opt().unwrap_or(0));

                if let Some(output_voltage1) = decoded.get("output_voltage1").and_then(|v| v.as_f64()) {
                    point = point.field("output_voltage1", output_voltage1);
                }
                if let Some(output_current1) = decoded.get("output_current1").and_then(|v| v.as_f64()) {
                    point = point.field("output_current1", output_current1);
                }
                if let Some(output_voltage2) = decoded.get("output_voltage2").and_then(|v| v.as_f64()) {
                    point = point.field("output_voltage2", output_voltage2);
                }
                if let Some(output_current2) = decoded.get("output_current2").and_then(|v| v.as_f64()) {
                    point = point.field("output_current2", output_current2);
                }
                if let Some(output_voltage3) = decoded.get("output_voltage3").and_then(|v| v.as_f64()) {
                    point = point.field("output_voltage3", output_voltage3);
                }
                if let Some(output_current3) = decoded.get("output_current3").and_then(|v| v.as_f64()) {
                    point = point.field("output_current3", output_current3);
                }
                if let Some(temperature) = decoded.get("temperature").and_then(|v| v.as_i64()) {
                    point = point.field("temperature", temperature);
                }
                if let Some(ac_current) = decoded.get("ac_current").and_then(|v| v.as_f64()) {
                    point = point.field("ac_current", ac_current);
                }

                let data_point = point.build()?;
                self.influx_client
                    .write(&self.bucket, futures::stream::once(async { data_point }))
                    .await
                    .context("Failed to write AC charger data")?;
            }
            "BatterySense" => {
                let mut point = DataPoint::builder("battery_sense")
                    .tag("device_id", device_id)
                    .timestamp(timestamp.timestamp_nanos_opt().unwrap_or(0));

                if let Some(voltage) = decoded.get("voltage").and_then(|v| v.as_f64()) {
                    point = point.field("voltage", voltage);
                }
                if let Some(temperature) = decoded.get("temperature").and_then(|v| v.as_f64()) {
                    point = point.field("temperature", temperature);
                }

                let data_point = point.build()?;
                self.influx_client
                    .write(&self.bucket, futures::stream::once(async { data_point }))
                    .await
                    .context("Failed to write battery sense data")?;
            }
            "DcEnergyMeter" => {
                let mut point = DataPoint::builder("dc_energy_meter")
                    .tag("device_id", device_id)
                    .timestamp(timestamp.timestamp_nanos_opt().unwrap_or(0));

                if let Some(voltage) = decoded.get("voltage").and_then(|v| v.as_f64()) {
                    point = point.field("voltage", voltage);
                }
                if let Some(current) = decoded.get("current").and_then(|v| v.as_f64()) {
                    point = point.field("current", current);
                }
                if let Some(power) = decoded.get("power").and_then(|v| v.as_i64()) {
                    point = point.field("power", power);
                }

                let data_point = point.build()?;
                self.influx_client
                    .write(&self.bucket, futures::stream::once(async { data_point }))
                    .await
                    .context("Failed to write DC energy meter data")?;
            }
            "Inverter" => {
                let mut point = DataPoint::builder("inverter")
                    .tag("device_id", device_id)
                    .timestamp(timestamp.timestamp_nanos_opt().unwrap_or(0));

                if let Some(battery_voltage) = decoded.get("battery_voltage").and_then(|v| v.as_f64()) {
                    point = point.field("battery_voltage", battery_voltage);
                }
                if let Some(ac_apparent_power) = decoded.get("ac_apparent_power").and_then(|v| v.as_i64()) {
                    point = point.field("ac_apparent_power", ac_apparent_power);
                }
                if let Some(ac_voltage) = decoded.get("ac_voltage").and_then(|v| v.as_f64()) {
                    point = point.field("ac_voltage", ac_voltage);
                }
                if let Some(ac_current) = decoded.get("ac_current").and_then(|v| v.as_f64()) {
                    point = point.field("ac_current", ac_current);
                }

                let data_point = point.build()?;
                self.influx_client
                    .write(&self.bucket, futures::stream::once(async { data_point }))
                    .await
                    .context("Failed to write inverter data")?;
            }
            "LynxSmartBMS" => {
                let mut point = DataPoint::builder("lynx_smart_bms")
                    .tag("device_id", device_id)
                    .timestamp(timestamp.timestamp_nanos_opt().unwrap_or(0));

                if let Some(voltage) = decoded.get("voltage").and_then(|v| v.as_f64()) {
                    point = point.field("voltage", voltage);
                }
                if let Some(current) = decoded.get("current").and_then(|v| v.as_f64()) {
                    point = point.field("current", current);
                }
                if let Some(soc) = decoded.get("soc").and_then(|v| v.as_f64()) {
                    point = point.field("soc", soc);
                }
                if let Some(consumed_ah) = decoded.get("consumed_ah").and_then(|v| v.as_f64()) {
                    point = point.field("consumed_ah", consumed_ah);
                }
                if let Some(temperature) = decoded.get("temperature").and_then(|v| v.as_f64()) {
                    point = point.field("temperature", temperature);
                }
                if let Some(battery_temperature) = decoded.get("battery_temperature").and_then(|v| v.as_f64()) {
                    point = point.field("battery_temperature", battery_temperature);
                }

                let data_point = point.build()?;
                self.influx_client
                    .write(&self.bucket, futures::stream::once(async { data_point }))
                    .await
                    .context("Failed to write Lynx Smart BMS data")?;
            }
            "OrionXS" => {
                let mut point = DataPoint::builder("orion_xs")
                    .tag("device_id", device_id)
                    .timestamp(timestamp.timestamp_nanos_opt().unwrap_or(0));

                if let Some(input_voltage) = decoded.get("input_voltage").and_then(|v| v.as_f64()) {
                    point = point.field("input_voltage", input_voltage);
                }
                if let Some(input_current) = decoded.get("input_current").and_then(|v| v.as_f64()) {
                    point = point.field("input_current", input_current);
                }
                if let Some(output_voltage) = decoded.get("output_voltage").and_then(|v| v.as_f64()) {
                    point = point.field("output_voltage", output_voltage);
                }
                if let Some(output_current) = decoded.get("output_current").and_then(|v| v.as_f64()) {
                    point = point.field("output_current", output_current);
                }

                let data_point = point.build()?;
                self.influx_client
                    .write(&self.bucket, futures::stream::once(async { data_point }))
                    .await
                    .context("Failed to write Orion XS data")?;
            }
            "SmartBatteryProtect" => {
                let mut point = DataPoint::builder("smart_battery_protect")
                    .tag("device_id", device_id)
                    .timestamp(timestamp.timestamp_nanos_opt().unwrap_or(0));

                if let Some(input_voltage) = decoded.get("input_voltage").and_then(|v| v.as_f64()) {
                    point = point.field("input_voltage", input_voltage);
                }
                if let Some(output_voltage) = decoded.get("output_voltage").and_then(|v| v.as_f64()) {
                    point = point.field("output_voltage", output_voltage);
                }
                if let Some(error_code) = decoded.get("error_code").and_then(|v| v.as_i64()) {
                    point = point.field("error_code", error_code);
                }

                let data_point = point.build()?;
                self.influx_client
                    .write(&self.bucket, futures::stream::once(async { data_point }))
                    .await
                    .context("Failed to write Smart Battery Protect data")?;
            }
            "SmartLithium" => {
                let mut point = DataPoint::builder("smart_lithium")
                    .tag("device_id", device_id)
                    .timestamp(timestamp.timestamp_nanos_opt().unwrap_or(0));

                if let Some(battery_voltage) = decoded.get("battery_voltage").and_then(|v| v.as_f64()) {
                    point = point.field("battery_voltage", battery_voltage);
                }
                if let Some(cell_count) = decoded.get("cell_count").and_then(|v| v.as_i64()) {
                    point = point.field("cell_count", cell_count);
                }
                if let Some(temperature) = decoded.get("temperature").and_then(|v| v.as_f64()) {
                    point = point.field("temperature", temperature);
                }

                let data_point = point.build()?;
                self.influx_client
                    .write(&self.bucket, futures::stream::once(async { data_point }))
                    .await
                    .context("Failed to write Smart Lithium data")?;
            }
            "VEBus" => {
                let mut point = DataPoint::builder("vebus")
                    .tag("device_id", device_id)
                    .timestamp(timestamp.timestamp_nanos_opt().unwrap_or(0));

                if let Some(battery_voltage) = decoded.get("battery_voltage").and_then(|v| v.as_f64()) {
                    point = point.field("battery_voltage", battery_voltage);
                }
                if let Some(battery_current) = decoded.get("battery_current").and_then(|v| v.as_f64()) {
                    point = point.field("battery_current", battery_current);
                }
                if let Some(soc) = decoded.get("soc").and_then(|v| v.as_i64()) {
                    point = point.field("soc", soc);
                }
                if let Some(ac_in_power) = decoded.get("ac_in_power").and_then(|v| v.as_i64()) {
                    point = point.field("ac_in_power", ac_in_power);
                }
                if let Some(ac_out_power) = decoded.get("ac_out_power").and_then(|v| v.as_i64()) {
                    point = point.field("ac_out_power", ac_out_power);
                }
                if let Some(battery_temperature) = decoded.get("battery_temperature").and_then(|v| v.as_i64()) {
                    point = point.field("battery_temperature", battery_temperature);
                }

                let data_point = point.build()?;
                self.influx_client
                    .write(&self.bucket, futures::stream::once(async { data_point }))
                    .await
                    .context("Failed to write VE.Bus data")?;
            }
            _ => {
                // Store as generic device data
                warn!("Unknown device type: {}", device_type);
                let point = DataPoint::builder("unknown_device")
                    .tag("device_id", device_id)
                    .tag("device_type", device_type)
                    .field("raw_json", decoded.to_string())
                    .timestamp(timestamp.timestamp_nanos_opt().unwrap_or(0))
                    .build()?;

                self.influx_client
                    .write(&self.bucket, futures::stream::once(async { point }))
                    .await
                    .context("Failed to write unknown device data")?;
            }
        }

        Ok(())
    }

    async fn stream_uplinks(&self) -> Result<()> {
        let url = format!(
            "https://{}.cloud.thethings.network/api/v3/as/applications/{}/packages/storage/uplink_message",
            self.ttn_region, self.ttn_app_id
        );

        info!("Connecting to TTN: {}", url);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()?;

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.ttn_api_key))
            .header("Accept", "text/event-stream")
            .query(&[("last", "24h")]) // Get last 24h on startup
            .send()
            .await
            .context("Failed to connect to TTN")?;

        if !response.status().is_success() {
            anyhow::bail!("TTN API returned status: {}", response.status());
        }

        info!("Connected to TTN stream");

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    buffer.push_str(&text);

                    // Process complete lines
                    while let Some(newline_pos) = buffer.find('\n') {
                        let line = buffer[..newline_pos].trim().to_string();
                        buffer = buffer[newline_pos + 1..].to_string();

                        if line.is_empty() {
                            continue;
                        }

                        // SSE format can have "data: " prefix or just JSON
                        let json_str = if line.starts_with("data:") {
                            line.strip_prefix("data:").unwrap_or(&line).trim()
                        } else {
                            &line
                        };

                        if json_str.is_empty() {
                            continue;
                        }

                        // Try to parse as TtnMessage
                        match serde_json::from_str::<TtnMessage>(json_str) {
                            Ok(message) => {
                                if let Err(e) = self.process_uplink(message).await {
                                    error!("Failed to process uplink: {}", e);
                                }
                            }
                            Err(e) => {
                                // Only log if it's not a keep-alive or comment
                                if !json_str.starts_with(':') {
                                    debug!("Failed to parse JSON (might be keep-alive): {}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Stream error: {}", e);
                    anyhow::bail!("Stream interrupted: {}", e);
                }
            }
        }

        warn!("Stream ended");
        Ok(())
    }

    async fn run(&self) -> Result<()> {
        loop {
            match self.stream_uplinks().await {
                Ok(_) => {
                    warn!("Stream ended normally, reconnecting...");
                }
                Err(e) => {
                    error!("Stream error: {}, reconnecting in 5s...", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting TTN Data Ingestor");

    let ingestor = TtnIngestor::new()?;
    ingestor.run().await
}
