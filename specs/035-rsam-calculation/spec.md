# Feature Specification: RSAM Calculation and UDP Forwarding

**Feature Branch**: `035-rsam-calculation`
**Created**: 2026-02-10
**Status**: Draft
**Input**: User description: "references/rsudpのPython実装と同等のrsamの機能を実装してください。E2Eテストも実装して、同じように動作することも確認してほしいです。"

## Clarifications

### Session 2026-02-10

- Q: Should deconvolution (unit conversion from raw counts to physical units) be supported? → A: Yes, deconvolution is in scope. The system uses the existing sensitivity map (fetched from FDSN StationXML) to convert raw counts to physical units (velocity, acceleration, displacement, gravity) before RSAM calculation when `deconvolve` is enabled.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Calculate and Forward RSAM Values (Priority: P1)

As a seismograph operator, I want the system to continuously calculate Real-time Seismic Amplitude Measurement (RSAM) statistics from a configured channel and periodically send the results via UDP to a remote receiver, so that I can monitor ground motion magnitude in real time from a separate monitoring system.

The system calculates four statistics over a configurable time window: mean, median, minimum, and maximum of the absolute amplitude values. When deconvolution is enabled, raw counts are converted to physical units (using the sensitivity map) before calculation. These results are formatted (LITE, JSON, or CSV) and sent via UDP to the configured destination at each interval.

**Why this priority**: This is the core functionality of the RSAM module. Without it, there is no value delivered. It matches the Python rsudp `c_rsam.py` behavior.

**Independent Test**: Configure RSAM with a destination UDP listener, send seismic data through the pipeline, and verify the listener receives correctly formatted RSAM statistics at the configured interval.

**Acceptance Scenarios**:

1. **Given** RSAM is enabled with `interval=10` and a destination address, **When** 10 seconds of seismic data arrives on the configured channel, **Then** the system sends one UDP packet containing mean, median, min, and max of absolute amplitudes to the destination.
2. **Given** RSAM is enabled with `fwformat=LITE`, **When** the interval elapses, **Then** the destination receives a pipe-delimited string: `stn:{station}|ch:{channel}|mean:{value}|med:{value}|min:{value}|max:{value}`.
3. **Given** RSAM is enabled with `fwformat=JSON`, **When** the interval elapses, **Then** the destination receives a valid JSON object with keys: station, channel, mean, median, min, max.
4. **Given** RSAM is enabled with `fwformat=CSV`, **When** the interval elapses, **Then** the destination receives a comma-separated line: `{station},{channel},{mean},{median},{min},{max}`.
5. **Given** RSAM is enabled, **When** data arrives on a channel that does not match the configured channel filter, **Then** that data is ignored and not included in the RSAM calculation.
6. **Given** RSAM is enabled with `deconvolve=true` and `units=VEL`, **When** RSAM calculates, **Then** raw sample counts are divided by the channel's sensitivity value (from the sensitivity map) to produce velocity units before computing statistics.

---

### User Story 2 - Channel Matching and Filtering (Priority: P2)

As a seismograph operator, I want to specify which channel to monitor for RSAM using a partial channel name (e.g., "HZ" matches "EHZ" or "SHZ"), so that the RSAM calculation focuses on the channel I care about without needing to know the exact full channel code.

**Why this priority**: Channel matching is essential for usability, as operators may not know the exact channel code transmitted by their Raspberry Shake sensor. This matches the Python implementation's suffix-matching behavior.

**Independent Test**: Configure RSAM with `channel = "HZ"`, send data for multiple channels (EHZ, EHN, EHE), and verify only EHZ data is included in the RSAM calculation.

**Acceptance Scenarios**:

1. **Given** RSAM is configured with `channel = "HZ"`, **When** data for channels EHZ, EHN, and EHE arrives, **Then** only EHZ data is used for RSAM calculation.
2. **Given** RSAM is configured with `channel = "ENZ"`, **When** data for ENZ and ENE arrives, **Then** only ENZ data is used.
3. **Given** RSAM is configured with `channel = "HZ"` and data arrives for SHZ, **Then** SHZ data is used (suffix matching).

---

### User Story 3 - Verify RSAM Operation via Logs (Priority: P2)

As a seismograph operator, I want the system to log RSAM calculation results and startup status, so that I can verify RSAM is operating correctly without needing a separate receiver.

**Why this priority**: Operational visibility is critical for debugging and verifying correct behavior, especially during initial setup. When `quiet=false`, the operator should see RSAM values in the logs.

**Independent Test**: Start the system with RSAM enabled and `quiet=false`, send seismic data, and verify log output includes RSAM statistics at each interval.

**Acceptance Scenarios**:

1. **Given** RSAM is enabled, **When** the system starts, **Then** it logs the RSAM configuration (channel, interval, format, destination, deconvolve, units).
2. **Given** RSAM is enabled and `quiet=false`, **When** RSAM values are calculated, **Then** the values are logged at INFO level.
3. **Given** RSAM is enabled and `quiet=true`, **When** RSAM values are calculated, **Then** the values are NOT logged (only sent via UDP).

---

### User Story 4 - Automated End-to-End RSAM Test (Priority: P3)

As a developer, I want automated tests that verify RSAM calculation correctness and UDP delivery end-to-end, so that the feature is regression-proof and matches the Python implementation's behavior.

**Why this priority**: Explicitly requested by the user. E2E tests ensure correctness and prevent regressions.

**Independent Test**: Run `cargo test test_rsam` — tests bind local UDP listeners, send known data through the RSAM module, verify received values match expected calculations.

**Acceptance Scenarios**:

1. **Given** a known set of sample values, **When** RSAM calculates statistics, **Then** the mean, median, min, and max match expected values.
2. **Given** RSAM is configured with LITE format, **When** the test runs, **Then** the UDP listener receives a correctly formatted LITE packet with accurate values.
3. **Given** RSAM is configured with JSON format, **When** the test runs, **Then** the UDP listener receives valid JSON with accurate values.
4. **Given** RSAM is configured with a 2-second interval and data is sent for 5 seconds, **When** the test runs, **Then** at least 2 RSAM packets are received.
5. **Given** RSAM is configured with `deconvolve=true` and a known sensitivity value, **When** the test runs, **Then** RSAM values reflect the deconvolved (unit-converted) amplitudes.

---

### Edge Cases

- What happens when no data arrives for the configured channel within an interval? The system waits and does not send an RSAM packet until data is available.
- What happens when the destination address is unreachable? The system logs a warning and continues operating (does not crash or block the pipeline).
- What happens when RSAM is enabled but `fwaddr` is empty or invalid? The system logs an error at startup and disables RSAM UDP forwarding (calculation and logging still work if `quiet=false`).
- What happens when the interval is very short (e.g., 1 second) and not enough samples have accumulated? The system calculates from whatever samples are available in the window.
- What happens when `fwformat` is an unrecognized value? The system defaults to LITE format and logs a warning.
- What happens when `deconvolve=true` but no sensitivity value is available for the channel? The system falls back to raw counts and logs a warning.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST calculate RSAM statistics (mean, median, min, max of absolute amplitude values) over a sliding window of configurable duration (default: 10 seconds).
- **FR-002**: System MUST send RSAM results via UDP to the configured destination address and port at each interval.
- **FR-003**: System MUST support three output formats: LITE (pipe-delimited), JSON, and CSV.
- **FR-004**: System MUST match channels using case-insensitive suffix matching (e.g., configured "HZ" matches incoming "EHZ", "SHZ").
- **FR-005**: System MUST include station name and channel name in all output formats.
- **FR-006**: System MUST accumulate samples over the configured interval window before calculating statistics (warm-up period).
- **FR-007**: System MUST discard samples older than the configured interval from the calculation window.
- **FR-008**: System MUST log startup configuration (channel, interval, format, destination, deconvolve, units) at INFO level.
- **FR-009**: System MUST support `quiet` mode to suppress periodic RSAM value logging while still forwarding via UDP.
- **FR-010**: System MUST handle UDP send failures gracefully (log warning, do not crash or block pipeline).
- **FR-011**: System MUST only operate on a single configured channel (not multiple channels simultaneously).
- **FR-012**: System MUST be configurable via the existing settings file under the `[rsam]` section using existing `RsamSettings` fields.
- **FR-013**: When `deconvolve=true`, system MUST convert raw sample counts to physical units by dividing each sample by the channel's sensitivity value (from the sensitivity map) before computing RSAM statistics.
- **FR-014**: System MUST support the following deconvolution unit modes via the `units` setting: `VEL` (velocity, m/s), `ACC` (acceleration, m/s²), `DISP` (displacement, m), `GRAV` (fraction of gravity, divides acceleration by 9.81), and `CHAN` (channel-specific default: velocity for geophones EH*, acceleration for accelerometers EN*).
- **FR-015**: When `deconvolve=true` but no sensitivity value is available for the matched channel, system MUST fall back to raw counts and log a warning.

### LITE Format Specification

```
stn:{station}|ch:{channel}|mean:{value}|med:{value}|min:{value}|max:{value}
```

### JSON Format Specification

```json
{"station":"{station}","channel":"{channel}","mean":{value},"median":{value},"min":{value},"max":{value}}
```

### CSV Format Specification

```
{station},{channel},{mean},{median},{min},{max}
```

### Key Entities

- **RsamCalculator**: Accumulates samples for a single channel over a time window, optionally applies deconvolution (sensitivity conversion), computes mean/median/min/max of absolute values, and formats results.
- **RsamSettings**: Configuration for the RSAM module (already exists in settings.rs) — enabled, quiet, fwaddr, fwport, fwformat, channel, interval, deconvolve, units.
- **RsamResult**: A single RSAM calculation output containing station, channel, mean, median, min, max, and timestamp.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: RSAM statistics are calculated and sent via UDP within 1 second after each interval elapses.
- **SC-002**: All three output formats (LITE, JSON, CSV) produce correctly parseable output verified by automated tests.
- **SC-003**: Channel suffix matching correctly filters data, verified by tests sending multi-channel data.
- **SC-004**: RSAM calculation values (mean, median, min, max) match expected values computed from known input data, within floating-point precision.
- **SC-005**: All automated E2E tests pass, covering single-channel RSAM, multi-format output, interval timing, and deconvolution.
- **SC-006**: RSAM module does not degrade pipeline throughput — forwarding is non-blocking.
- **SC-007**: Deconvolved RSAM values correctly reflect sensitivity-adjusted physical units, verified by tests with known sensitivity values.

## Assumptions

- Station name is derived from the existing pipeline's parsed segment metadata (network.station format or station field).
- The existing `RsamSettings` struct in `settings.rs` provides all necessary configuration fields.
- RSAM values use standard floating-point precision (no explicit rounding), matching the Python implementation.
- Deconvolution uses the existing sensitivity map (from `fetch_sensitivity()` / FDSN StationXML). The conversion is `sample / sensitivity` to get physical units. For GRAV mode, the result is further divided by 9.81.
- The sensitivity map provides values in Counts/(m/s) for geophones and Counts/(m/s²) for accelerometers.

## Dependencies

- Existing pipeline infrastructure (`run_pipeline` in pipeline.rs)
- Existing settings infrastructure (`RsamSettings` in settings.rs)
- Existing sensitivity map infrastructure (`fetch_sensitivity` in parser/stationxml.rs)
- Existing UDP/tokio networking (same as forward module)

## Scope Boundaries

### In Scope

- RSAM calculation (mean, median, min, max of absolute amplitudes)
- Deconvolution / sensitivity conversion (VEL, ACC, DISP, GRAV, CHAN modes)
- UDP forwarding in LITE, JSON, CSV formats
- Channel suffix matching
- Interval-based windowed calculation
- Startup and periodic logging
- Automated E2E tests

### Out of Scope

- Multiple simultaneous channel monitoring
- Persistent storage of RSAM values
- WebUI display of RSAM values
