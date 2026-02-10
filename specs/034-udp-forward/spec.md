# Feature Specification: UDP Data Forwarding

**Feature Branch**: `034-udp-forward`
**Created**: 2026-02-10
**Status**: Draft
**Input**: User description: "references/rsudpのPython実装でいうところの、forward機能を実装してほしいです。正常に動いていることを検証する仕組みも作ってください。"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Forward Seismic Data to Remote Receivers (Priority: P1)

As a seismologist operating a network of monitoring stations, I want to forward incoming seismic data packets to one or more remote receivers so that multiple systems can process the same data stream independently (e.g., secondary analysis stations, backup recorders, or partner institutions).

**Why this priority**: This is the core value of the forwarding feature. Without the ability to forward data packets, the feature has no purpose. Data distribution is the primary use case for seismic monitoring networks.

**Independent Test**: Can be fully tested by configuring a single forward destination, sending sample seismic data, and verifying the remote receiver receives the forwarded packets with correct content.

**Acceptance Scenarios**:

1. **Given** the system is running with forwarding enabled and one destination configured, **When** seismic data packets arrive from the Raspberry Shake sensor, **Then** the system forwards each matching packet to the configured destination address and port via UDP.
2. **Given** the system is running with multiple forward destinations configured, **When** data packets arrive, **Then** each destination independently receives a copy of the forwarded packets.
3. **Given** forwarding is disabled in configuration, **When** data packets arrive, **Then** no packets are forwarded, and the system operates normally without any forwarding overhead.

---

### User Story 2 - Filter Forwarded Data by Channel and Message Type (Priority: P2)

As a network operator, I want to control which channels and message types are forwarded to each destination so that I can reduce bandwidth usage and send only relevant data to each downstream consumer (e.g., forward only the vertical channel "EHZ" to a trigger-only station, or forward alarm messages to a notification aggregator).

**Why this priority**: Filtering is essential for practical deployments where bandwidth or processing capacity is limited. Without filtering, the feature works but may be impractical in real-world networks.

**Independent Test**: Can be tested by configuring channel filters and data/alarm flags, then verifying that only matching packets are forwarded while non-matching packets are silently dropped.

**Acceptance Scenarios**:

1. **Given** forwarding is configured with `channels = ["EHZ"]` and `fwd_data = true`, **When** data packets arrive for channels EHZ, EHN, and EHE, **Then** only EHZ data packets are forwarded.
2. **Given** forwarding is configured with `fwd_alarms = true` and `fwd_data = false`, **When** an alarm is triggered and data packets arrive, **Then** only ALARM and RESET messages are forwarded, not data packets.
3. **Given** forwarding is configured with `channels = ["all"]`, **When** data packets arrive for any channel, **Then** all channel data packets are forwarded.

---

### User Story 3 - Verify Forwarding via Runtime Log Monitoring (Priority: P2)

As a system operator, I want the system to periodically log forwarding statistics (packet counts per destination, errors, destination reachability) during normal operation so that I can verify forwarding is working correctly by reading log output, without needing separate tools or manual network inspection.

**Why this priority**: The user explicitly requested a verification mechanism. Runtime log-based monitoring provides always-on visibility with zero additional operator effort, and aligns with the existing observability patterns in the system.

**Independent Test**: Can be tested by starting the system with forwarding enabled, waiting for periodic stats to be emitted, and verifying the log output contains accurate packet counts and destination status.

**Acceptance Scenarios**:

1. **Given** the system is running with forwarding enabled, **When** the periodic stats interval elapses, **Then** the system logs a summary per destination including: packets forwarded, packets dropped/failed, and destination reachability status.
2. **Given** a forward destination is unreachable, **When** the periodic stats are logged, **Then** the log clearly identifies the failing destination and the number of failed send attempts.
3. **Given** the system starts with forwarding enabled, **When** the first data packet is forwarded, **Then** the system logs a startup message confirming active destinations and filter configuration.

---

### User Story 4 - Automated End-to-End Forwarding Test (Priority: P3)

As a developer, I want automated tests that verify the entire forwarding pipeline works end-to-end so that regressions are caught before deployment.

**Why this priority**: Automated testing supports long-term reliability but is not required for initial deployment. It ensures the forward feature remains functional across code changes.

**Independent Test**: Can be tested by running the automated test suite, which starts a local UDP listener, configures forwarding to it, sends sample data, and asserts the listener received the expected packets.

**Acceptance Scenarios**:

1. **Given** the test suite is run, **When** the forwarding integration test executes, **Then** it starts a local UDP listener, configures a forward destination pointing to it, sends sample seismic packets through the pipeline, and verifies the listener received the correct packets.
2. **Given** the test suite is run with filtering configured, **When** filtered data is sent, **Then** the test verifies that only matching packets were received by the listener.

---

### Edge Cases

- What happens when a forward destination is unreachable (network down, host offline)? The system must continue operating normally and log errors without blocking the main data pipeline.
- What happens when the address and port configuration lists have mismatched lengths? The system must reject the configuration at startup with a clear error message.
- What happens when a configured channel name does not match any available channel? The system must fall back to forwarding all channels and log a warning.
- What happens when the forwarding queue fills up (slow destination)? The system must drop packets rather than blocking the main pipeline.
- What happens when the system receives a shutdown signal while forwarding is active? All forwarding tasks must terminate gracefully and close their sockets.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST forward incoming UDP seismic data packets to one or more configured remote destinations.
- **FR-002**: System MUST support configuring multiple forward destinations, each with an independent address and port.
- **FR-003**: System MUST support filtering forwarded data by channel name (specific channels or "all").
- **FR-004**: System MUST support independently enabling/disabling forwarding of raw data packets (`fwd_data` flag).
- **FR-005**: System MUST support independently enabling/disabling forwarding of ALARM and RESET messages (`fwd_alarms` flag).
- **FR-006**: System MUST validate that address and port configuration lists have matching lengths at startup, and reject invalid configurations with a clear error.
- **FR-007**: System MUST handle unreachable destinations gracefully without blocking or crashing the main data processing pipeline.
- **FR-008**: System MUST periodically log forwarding statistics including: packets forwarded per destination, packets failed/dropped, and destination reachability status. Logs MUST also include startup confirmation and shutdown events.
- **FR-009**: System MUST provide an automated end-to-end test that verifies data forwarding through the entire pipeline.
- **FR-010**: System MUST terminate all forwarding tasks gracefully on shutdown.
- **FR-011**: System MUST fall back to forwarding all channels when a configured channel name does not match any available channel, with a logged warning.
- **FR-012**: System MUST support enabling or disabling the entire forwarding feature via a single configuration toggle.

### Key Entities

- **Forward Destination**: A remote endpoint defined by an IP address and port number that receives forwarded data. Multiple destinations can be configured simultaneously.
- **Forward Filter**: A set of criteria (channel list, data flag, alarm flag) that determines which packets are forwarded to each destination.
- **Forward Configuration**: The complete forwarding setup including enabled state, list of destinations, and filter settings. Stored in the application configuration file.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of data packets matching the configured channel filter are delivered to all reachable forward destinations.
- **SC-002**: Forwarding to an unreachable destination does not increase processing latency of the main data pipeline by more than 1%.
- **SC-003**: Automated end-to-end tests pass with 100% reliability, verifying that forwarded packets arrive at a local test receiver with correct content.
- **SC-004**: The system correctly starts and stops forwarding within 2 seconds of configuration changes or shutdown signals.
- **SC-005**: Operators can verify forwarding status through log output within 30 seconds of system startup.

## Clarifications

### Session 2026-02-10

- Q: What form should the verification mechanism take? → A: Runtime log-based monitoring with periodic stats logging (packet counts, errors, destination status). No separate CLI subcommand or health-check endpoint needed.

## Assumptions

- Forwarding uses UDP (connectionless), matching the Raspberry Shake protocol. No TCP or reliability guarantees are needed.
- All forward destinations share the same channel filter and data/alarm flags, matching the Python rsudp behavior.
- The forwarded packet format is identical to the received packet format (raw pass-through, no transformation).
- The existing `ForwardSettings` structure in the configuration file is the source of truth for forward configuration.
- Channel matching is case-insensitive (e.g., "ehz" matches "EHZ").
