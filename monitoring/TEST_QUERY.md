# Test Flux Queries for Grafana

## Simple voltage query (for testing datasource)

```flux
from(bucket: "victron-data")
  |> range(start: -24h)
  |> filter(fn: (r) => r._measurement == "dc_dc_converter")
  |> filter(fn: (r) => r._field == "input_voltage" or r._field == "output_voltage")
```

## DC-DC Converter Voltages (for dashboard panel)

```flux
from(bucket: "victron-data")
  |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
  |> filter(fn: (r) => r["_measurement"] == "dc_dc_converter")
  |> filter(fn: (r) => r["_field"] == "input_voltage" or r["_field"] == "output_voltage")
  |> aggregateWindow(every: v.windowPeriod, fn: mean, createEmpty: false)
```

## Latest Output Voltage (for gauge)

```flux
from(bucket: "victron-data")
  |> range(start: -1h)
  |> filter(fn: (r) => r._measurement == "dc_dc_converter")
  |> filter(fn: (r) => r._field == "output_voltage")
  |> last()
```

## Gateway RSSI

```flux
from(bucket: "victron-data")
  |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
  |> filter(fn: (r) => r._measurement == "signal_quality")
  |> filter(fn: (r) => r._field == "rssi")
  |> aggregateWindow(every: v.windowPeriod, fn: mean, createEmpty: false)
```

## Gateway SNR

```flux
from(bucket: "victron-data")
  |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
  |> filter(fn: (r) => r._measurement == "signal_quality")
  |> filter(fn: (r) => r._field == "snr")
  |> aggregateWindow(every: v.windowPeriod, fn: mean, createEmpty: false)
```

## Verify Data Exists

To verify data exists in InfluxDB from command line:

```bash
docker-compose exec influxdb influx query \
  'from(bucket: "victron-data") |> range(start: -24h) |> limit(n: 10)' \
  --org victron-monitoring \
  --token my-super-secret-auth-token
```
