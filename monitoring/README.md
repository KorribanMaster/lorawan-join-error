# Victron LoRaWAN Monitoring Stack

A complete monitoring solution for Victron devices connected via LoRaWAN to The Things Network (TTN). This stack uses Rust for data ingestion, InfluxDB for storage, and Grafana for visualization.

## Features

- **Real-time data ingestion** from TTN via Server-Sent Events (SSE)
- **Multi-device support**: Battery Monitor, Solar Charger, DC-DC Converter, and more
- **Signal quality tracking**: RSSI, SNR per gateway
- **Network metrics**: Spreading factor, bandwidth, airtime
- **Beautiful dashboards** with Grafana
- **Docker-based** for easy deployment

## Architecture

```
TTN (LoRaWAN) → Rust Ingestor → InfluxDB → Grafana
```

- **TTN Ingestor** (Rust): Connects to TTN Storage API, parses uplink messages, and writes to InfluxDB
- **InfluxDB 2.7**: Time-series database for metrics storage
- **Grafana 10.4**: Visualization and dashboarding

## Prerequisites

- Docker and Docker Compose
- TTN Application with API key
- Your TTN payload decoder must output JSON with device data

## Quick Start

### 1. Clone and Configure

```bash
cd monitoring
cp .env.example .env
```

Edit `.env` and set your TTN API key:

```bash
TTN_API_KEY=NNSXS.YOUR_ACTUAL_API_KEY_HERE
```

### 2. Start the Stack

```bash
docker-compose up -d
```

This will:
- Start InfluxDB on port 8086
- Start Grafana on port 3000
- Build and start the TTN ingestor

### 3. Access Grafana

Open http://localhost:3000 in your browser.

**Default credentials:**
- Username: `admin`
- Password: `admin` (change this!)

The dashboard "Victron LoRaWAN Overview" will be automatically provisioned.

## Configuration

### Environment Variables

See `.env.example` for all available options:

| Variable | Description | Default |
|----------|-------------|---------|
| `TTN_APP_ID` | Your TTN application ID | `vanman` |
| `TTN_API_KEY` | TTN API key with read access | **Required** |
| `TTN_REGION` | TTN region (eu1, us-west1, etc.) | `eu1` |
| `INFLUXDB_TOKEN` | InfluxDB authentication token | Random |
| `GRAFANA_ADMIN_PASSWORD` | Grafana admin password | `admin` |
| `RUST_LOG` | Log level (debug, info, warn, error) | `info` |

### TTN Payload Decoder

Your TTN application must have a payload decoder that outputs JSON with these fields:

**For DC-DC Converter:**
```json
{
  "device_type": "DcDcConverter",
  "input_voltage": 13.98,
  "output_voltage": 14.099,
  "off_reason": 0,
  "rssi": -60
}
```

**For Battery Monitor:**
```json
{
  "device_type": "BatteryMonitor",
  "voltage": 12.5,
  "current": 5.5,
  "soc": 85.5,
  "consumed_ah": 10.0,
  "rssi": -65
}
```

**For Solar Charger:**
```json
{
  "device_type": "SolarCharger",
  "battery_voltage": 13.5,
  "battery_current": 10.5,
  "pv_power": 200,
  "yield_today": 1500,
  "rssi": -70
}
```

## Data Schema

### InfluxDB Measurements

- **`dc_dc_converter`**: Input/output voltages
- **`battery_monitor`**: Voltage, current, SOC, consumed Ah
- **`solar_charger`**: Battery voltage/current, PV power, yield
- **`signal_quality`**: Per-gateway RSSI and SNR
- **`network_info`**: LoRa settings (SF, bandwidth)

## Viewing Logs

```bash
# All services
docker-compose logs -f

# Just the ingestor
docker-compose logs -f ttn-ingest

# Just InfluxDB
docker-compose logs -f influxdb
```

## Troubleshooting

### No data appearing in Grafana

1. Check if the ingestor is running:
   ```bash
   docker-compose ps
   ```

2. Check ingestor logs for errors:
   ```bash
   docker-compose logs ttn-ingest
   ```

3. Verify TTN API key is correct in `.env`

4. Test TTN connection manually:
   ```bash
   curl -G "https://eu1.cloud.thethings.network/api/v3/as/applications/vanman/packages/storage/uplink_message" \
     -H "Authorization: Bearer YOUR_API_KEY" \
     -H "Accept: text/event-stream" \
     -d "last=1h"
   ```

### InfluxDB connection issues

Check if InfluxDB is healthy:
```bash
docker-compose ps influxdb
```

Verify InfluxDB is accessible:
```bash
curl http://localhost:8086/health
```

### Rebuilding after code changes

```bash
docker-compose down
docker-compose build --no-cache ttn-ingest
docker-compose up -d
```

## Customization

### Adding Custom Dashboards

1. Create a new JSON file in `grafana/dashboards/`
2. Restart Grafana: `docker-compose restart grafana`

### Modifying Data Ingestion

Edit `ttn-ingest/src/main.rs` to add support for more device types or fields.

Then rebuild:
```bash
docker-compose build ttn-ingest
docker-compose up -d ttn-ingest
```

## Data Retention

By default, InfluxDB keeps all data indefinitely. To configure retention:

1. Access InfluxDB UI: http://localhost:8086
2. Login with credentials from `.env`
3. Go to Data → Buckets → victron-data → Edit
4. Set retention period (e.g., 30 days, 1 year)

## Backup

### Backing up InfluxDB data

```bash
docker-compose exec influxdb influx backup /var/lib/influxdb2/backup
docker cp victron-influxdb:/var/lib/influxdb2/backup ./backup
```

### Restoring from backup

```bash
docker cp ./backup victron-influxdb:/var/lib/influxdb2/restore
docker-compose exec influxdb influx restore /var/lib/influxdb2/restore
```

## Production Deployment

For production use:

1. **Change default passwords** in `.env`
2. **Use strong InfluxDB token**
3. **Enable HTTPS** with a reverse proxy (nginx, Traefik)
4. **Set up persistent volumes** on reliable storage
5. **Configure data retention** policies
6. **Enable Grafana authentication** (disable anonymous access)
7. **Set up monitoring** for the stack itself

## License

See main project license.

## Support

For issues related to:
- TTN integration: Check TTN documentation
- InfluxDB: See https://docs.influxdata.com/
- Grafana: See https://grafana.com/docs/
- Rust ingestor: Check logs or open an issue
