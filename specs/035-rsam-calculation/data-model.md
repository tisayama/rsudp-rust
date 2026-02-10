# Data Model: RSAM Calculation and UDP Forwarding

**Feature**: 035-rsam-calculation
**Date**: 2026-02-10

## Entities

### RsamSettings (existing in settings.rs)

Configuration for the RSAM module. Already defined — no changes needed.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| enabled | bool | false | Enable/disable RSAM module |
| quiet | bool | true | Suppress periodic log output |
| fwaddr | String | "192.168.1.254" | UDP destination address |
| fwport | u16 | 8887 | UDP destination port |
| fwformat | String | "LITE" | Output format: LITE, JSON, CSV |
| channel | String | "HZ" | Channel filter (suffix matching) |
| interval | u32 | 10 | Calculation window in seconds |
| deconvolve | bool | false | Enable sensitivity conversion |
| units | String | "VEL" | Unit mode: VEL, ACC, DISP, GRAV, CHAN |

### RsamManager

Main module struct managing RSAM calculation and UDP forwarding.

| Field | Type | Description |
|-------|------|-------------|
| settings | RsamSettings | Module configuration (cloned from settings) |
| socket | Option\<UdpSocket\> | UDP socket for sending results (None if addr invalid) |
| dest_addr | Option\<SocketAddr\> | Resolved destination address |
| buffer | Vec\<f64\> | Accumulated absolute amplitude samples |
| last_calc_time | Instant | Time of last RSAM calculation |
| station | String | Station name from first matching segment |
| matched_channel | String | Full channel code of first matching segment |
| sensitivity | Option\<f64\> | Sensitivity value for matched channel (from sensitivity map) |
| sensitivity_map | HashMap\<String, f64\> | Full sensitivity map (for late channel resolution) |
| warm | bool | Whether first segment has been received |

### RsamResult

Output of a single RSAM calculation.

| Field | Type | Description |
|-------|------|-------------|
| station | String | Station name (e.g., "R6E01") |
| channel | String | Full channel code (e.g., "EHZ") |
| mean | f64 | Mean of absolute amplitude values |
| median | f64 | Median of absolute amplitude values |
| min | f64 | Minimum absolute amplitude value |
| max | f64 | Maximum absolute amplitude value |

## Relationships

```
RsamSettings ──configures──> RsamManager
RsamManager ──produces──> RsamResult ──formats──> UDP packet (LITE/JSON/CSV)
TraceSegment ──feeds──> RsamManager (filtered by channel suffix match)
SensitivityMap ──provides──> RsamManager (for deconvolution)
```

## State Transitions

```
RsamManager States:
  Cold (no data received)
    → Warm (first matching segment received, station/channel resolved)
    → Calculating (interval elapsed, buffer has samples)
    → Sending (UDP packet formatted and sent)
    → Warm (buffer cleared, timer reset)
```

## Deconvolution Conversion Logic

| Units Mode | Channel Match | Conversion Formula |
|------------|--------------|-------------------|
| VEL | any | `sample / sensitivity` → m/s |
| ACC | any | `sample / sensitivity` → m/s² |
| DISP | any | `sample / sensitivity` → m (note: simplified, no integration) |
| GRAV | any | `sample / sensitivity / 9.81` → g |
| CHAN | EH* (geophone) | `sample / sensitivity` → m/s (VEL) |
| CHAN | EN* (accelerometer) | `sample / sensitivity` → m/s² (ACC) |
| (disabled) | any | raw counts (no conversion) |

When sensitivity is not available for the matched channel, all modes fall back to raw counts with a warning log.
