# Feature Specification: Playwright Alert Capture

**Feature Branch**: `041-playwright-alert-capture`
**Created**: 2026-02-16
**Status**: Draft
**Input**: Replace the current Rust backend plot generation (plotters) with a Playwright headless browser screenshot approach, producing alert plot images that are pixel-identical to the WebUI.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - High-Quality Plot Image Generation on Alert Reset (Priority: P1)

As a seismic monitoring operator, when an alert resets, I want a plot image (Waveform + Spectrogram for 4 channels) automatically generated at the same visual quality as the WebUI and delivered through existing notification channels (Discord, LINE, Google Chat, etc.), so that I can accurately assess event details even when away from the monitoring station.

**Why this priority**: This is the core value of the feature. The current plotters-based images lack anti-aliasing and use rectangle-based rendering, resulting in visual discrepancy from the WebUI. Using the same rendering code ensures visual consistency and accuracy for remote monitoring decisions.

**Independent Test**: Start the capture service, trigger an alert reset in rsudp-rust, and verify that the generated PNG is visually identical to a WebUI screenshot and that it is correctly delivered to notification channels.

**Acceptance Scenarios**:

1. **Given** rsudp-rust is receiving seismic data and the capture service is running, **When** an STA/LTA alert resets, **Then** a plot PNG containing 4 channels (Waveform + Spectrogram) covering the pre-trigger and post-trigger time range is generated within 30 seconds
2. **Given** a plot PNG has been generated, **When** notifications are dispatched, **Then** the image is attached to Discord, LINE, Google Chat, and other existing notification channels using the same delivery flow as before
3. **Given** WebUI filter or spectrogram settings have been changed, **When** a capture is executed on alert reset, **Then** the generated image reflects the current WebUI settings (bandpass filter, frequency range, deconvolution, etc.)

---

### User Story 2 - Capture Service Installation and Operation (Priority: P2)

As a system administrator, I want a single `make install` command to install all components including the capture service and its browser engine, managed via systemd, so that deployment friction is minimized across both server and Raspberry Pi environments.

**Why this priority**: The target environments include both x86_64 servers and Raspberry Pi 4 (ARM64). Complex installation procedures raise the deployment barrier significantly.

**Independent Test**: Run `make install` on a clean Raspberry Pi 4 (ARM64, 4GB RAM) and verify the capture service can be started and stopped via systemd.

**Acceptance Scenarios**:

1. **Given** a system with Node.js pre-installed, **When** `make install` is executed, **Then** the capture service including the browser engine is automatically installed
2. **Given** installation is complete, **When** `systemctl start rsudp-capture` is executed, **Then** the capture service starts as a resident process and begins accepting requests
3. **Given** the capture service is running, **When** `systemctl stop rsudp-capture` is executed, **Then** all child processes including the browser engine are cleanly terminated

---

### User Story 3 - Non-Interference with Monitoring Process (Priority: P3)

As a seismic monitoring operator, I want the capture service to operate without impacting rsudp-rust's UDP packet reception, STA/LTA calculation, or other critical monitoring functions, so that continuous monitoring reliability is maintained.

**Why this priority**: On resource-constrained environments like Raspberry Pi 4, image generation load could interfere with monitoring, causing packet loss or alert delays.

**Independent Test**: On a Raspberry Pi 4, trigger a capture while continuously streaming data, and verify that no UDP packet loss or STA/LTA calculation interruptions occur.

**Acceptance Scenarios**:

1. **Given** rsudp-rust is processing real-time data, **When** the capture service generates an image, **Then** no UDP packet reception loss occurs in rsudp-rust
2. **Given** a Raspberry Pi 4 (4GB RAM) environment, **When** the capture service is resident, **Then** the capture service memory usage does not exceed 300MB
3. **Given** the capture service has crashed, **When** rsudp-rust detects an alert reset, **Then** the failure is logged and rsudp-rust continues operating normally (notifications are sent without an image)

---

### Edge Cases

- What happens when the capture service is not running or not responding? The monitoring system sends notifications without an image. No legacy plotters fallback is maintained
- What happens when alerts reset in rapid succession (e.g., seconds apart)? Capture requests are queued and processed sequentially. When the queue depth exceeds 3, new requests are rejected (HTTP 503) to prevent overload
- What happens when the browser engine process grows due to memory leaks? The service manager's memory limit terminates the process, and automatic restart recovers the service
- What happens when a capture is requested but the data buffer is empty (no data received yet)? An error response is returned, and the monitoring system performs its fallback behavior

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST send an image generation request to the capture service on alert reset. The request MUST include the target time range, channel list, intensity information, and backend URL. The generated image MUST reflect the current display settings (bandpass filter, spectrogram frequency range, deconvolution), which the capture page retrieves from the backend data API
- **FR-002**: The capture service MUST generate and return a PNG image within 30 seconds of receiving a request
- **FR-003**: The generated image MUST contain Waveform and Spectrogram for 4 channels from 1 station, rendered using the same drawing code as the WebUI. The output dimensions MUST be 1000 x (500 * N_channels) pixels (e.g., 1000x2000 for 4 channels), matching the tall vertical layout of the current system
- **FR-004**: The generated image MUST include a JMA seismic intensity badge (equivalent to the current plotters version)
- **FR-005**: The monitoring system MUST send alert notifications without an image when the capture service is unavailable. The legacy plotters-based rendering code MUST be removed as part of this feature
- **FR-006**: The capture service MUST operate as an independent process, separate from the monitoring system
- **FR-007**: The capture service MUST run at lower priority than the monitoring process (reduced CPU priority, memory ceiling)
- **FR-008**: `make install` MUST install the monitoring system, capture service, and browser engine together
- **FR-009**: The capture service MUST work on both x86_64 Linux and ARM64 Linux (Raspberry Pi 4)
- **FR-010**: The capture service MUST be manageable via the OS service manager (start, stop, auto-restart)

### Key Entities

- **Capture Request**: A request for image generation. Contains time range (start/end), target channel list, and display settings (filter on/off, frequency range, deconvolution, intensity badge value, etc.)
- **Capture Response**: The generation result. On success: PNG binary data. On failure: error reason
- **Capture Service**: A resident process that holds a browser engine internally and, upon request, renders the WebUI page and takes a screenshot

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Alert reset image generation completes within 30 seconds (including Raspberry Pi 4 environments)
- **SC-002**: Generated images are visually identical to the WebUI display (same rendering logic, colormap, fonts, and layout)
- **SC-003**: No UDP packet reception loss occurs in the monitoring system during capture generation
- **SC-004**: Capture service memory usage stays under 300MB at all times (Raspberry Pi 4 environment)
- **SC-005**: A single `make install` execution installs all components on a clean environment (no additional manual commands required)
- **SC-006**: Capture service failures do not affect monitoring functionality (process isolation provides fault isolation)

## Clarifications

### Session 2026-02-16

- Q: What should happen when the capture service is unavailable at alert reset? → A: Remove legacy plotters code entirely; send notifications without image if capture service is down.
- Q: What should the output image dimensions be? → A: Match current plotters dimensions: 1000 x (500 * N_channels), tall vertical layout.

## Assumptions

- Node.js (v18 or later) is pre-installed on the target environment
- Raspberry Pi 4 runs a 64-bit OS (Raspberry Pi OS 64-bit, etc.) with 4GB or more RAM
- The WebUI (Next.js) is running on the same host as the capture service, or is network-accessible
- Alert resets are infrequent (typically a few per day or less), so simultaneous capture requests are rare
- Existing notification channel image delivery flows (Discord, LINE, Google Chat, AWS SNS) are not changed — the PNG file path interface is preserved
