# Feature Specification: WebUI Plot System

**Feature Branch**: `007-webui-plot`  
**Created**: 2025-12-19  
**Status**: Draft  
**Input**: User description: "rsudpに実装されているplotシステムを実装したいです。ただし、Xは使いたくないので、WebUIで実装してほしいです。サーバサイド（Rust)との通信はREST APIとWebSocketsで、フロントエンドはNext.JSとTailwindで組んでください。"

## Summary

Implement a real-time seismic data visualization system using a modern WebUI. This feature replaces the legacy X11-based Matplotlib display found in `rsudp` with a high-performance web interface. The system uses a Rust backend to stream seismic data via WebSockets and manage configurations through a REST API, providing users with a responsive and accessible platform for monitoring seismic activity.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Real-time Waveform Monitoring (Priority: P1)

A seismic station operator wants to see live waveform data from their station without needing a local display (X11). They open the system's web URL in a browser and immediately see smooth, real-time scrolling waveforms for all active channels (e.g., SHZ, EHZ).

**Why this priority**: Core functionality that delivers the primary value of the plot system.

**Independent Test**: Can be tested by launching the Rust backend and Next.js frontend, then verifying that waveform data packets arriving at the backend are rendered on the web page with minimal latency.

**Acceptance Scenarios**:

1. **Given** the backend is receiving seismic samples, **When** a user navigates to the dashboard, **Then** the waveforms for active channels are displayed and scroll in real-time.
2. **Given** a live plot is running, **When** a new data packet arrives via WebSocket, **Then** the plot is updated within 100ms.

---

### User Story 2 - Plot Configuration & Channel Selection (Priority: P2)

A user wants to customize which channels are displayed and adjust the time scale (e.g., viewing the last 5 minutes instead of 1 minute). They use a settings panel in the WebUI to select channels and change the view duration.

**Why this priority**: Essential for usability and managing screen real estate in multi-channel environments.

**Independent Test**: Can be tested by changing settings in the UI and verifying that the REST API calls update the backend state and the frontend rendering logic accordingly.

**Acceptance Scenarios**:

1. **Given** multiple channels are available, **When** a user deselects a channel in the settings, **Then** that channel's plot is removed from the view.
2. **Given** a 1-minute time window, **When** a user changes the setting to 5 minutes, **Then** the X-axis scale adjusts to show a 5-minute history.

---

### User Story 3 - Visualizing STA/LTA Triggers (Priority: P3)

An operator wants to know when an earthquake trigger has occurred. When the STA/LTA ratio exceeds the threshold, the WebUI highlights the event on the waveform and displays a visual indicator (e.g., a vertical line or background color change).

**Why this priority**: Critical for event detection and situational awareness.

**Independent Test**: Can be tested by simulating a seismic event (sending high-amplitude samples) and verifying that the "ALARM" event sent via WebSocket triggers a visual change in the UI.

**Acceptance Scenarios**:

1. **Given** an active monitoring session, **When** an "ALARM" event is received from the backend, **Then** a vertical red line is drawn at the trigger timestamp on the waveform.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide a WebSocket server (Rust) to stream real-time seismic samples to the frontend.
- **FR-002**: The frontend (Next.js) MUST render waveforms using a high-performance drawing method (e.g., HTML5 Canvas) to handle continuous updates.
- **FR-003**: The system MUST provide a REST API (Rust) for retrieving current station metadata and active plot configurations.
- **FR-004**: The UI MUST be responsive and styled using Tailwind CSS, supporting both desktop and tablet screen sizes.
- **FR-005**: The system MUST support simultaneous display of at least 3 components (Z, N, E) per station.
- **FR-006**: The backend MUST handle multiple concurrent WebUI clients without significant performance degradation.
- **FR-007**: The UI MUST allow users to toggle "Auto-scale" for waveform amplitudes.

### Key Entities

- **WaveformPacket**: A data structure containing a batch of samples, channel ID, and start timestamp.
- **PlotSettings**: Configuration object containing active channels, time window length, and scaling options.
- **AlertEvent**: A message indicating a trigger onset or reset (Alarm/Reset).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Waveform data is rendered on the frontend within 200ms of arrival at the Rust backend.
- **SC-002**: The system supports streaming data at 100Hz for up to 6 channels simultaneously without UI stuttering (maintaining 60 FPS).
- **SC-003**: Initial page load and first data render occur in under 2 seconds on a local network.
- **SC-004**: Configuration changes (e.g., channel toggle) are reflected in the UI in under 500ms.

## Edge Cases

- **Network Latency**: How the UI handles delayed or out-of-order WebSocket packets (e.g., using a buffer to ensure smooth scrolling).
- **Tab Backgrounding**: How the frontend manages the WebSocket connection and data buffer when the browser tab is not active.
- **Disconnect/Reconnect**: The system must automatically attempt to reconnect the WebSocket if the connection is lost.

## Assumptions

- The Rust backend has access to processed seismic data (Feature 004).
- Users are accessing the WebUI from a modern browser with Canvas and WebSocket support.
- Security (authentication) is handled by a separate layer or is out of scope for this initial prototype.