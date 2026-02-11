# Feature Specification: WebUI Spectrogram & rsudp-Compatible Plot

**Feature Branch**: `036-webui-spectrogram`
**Created**: 2026-02-10
**Status**: Draft
**Input**: User description: "rsudp実装のplot機能をwebuiで再現したいです。rsudpのplot実装の動きや画面表示をスクリーンショットで見ながら仕様を考えていただけませんか。"

## Visual Reference

Reference screenshots are stored in `references/` directory of this spec, generated from actual rsudp test data (AM.R24FA, 4 channels, 100 Hz, 110 seconds with earthquake event).

**Single channel view** (`references/rsudp_single_channel.png`):
- Dark background (#202530) filling the entire window
- Title at top: station name and view description
- One waveform panel (upper ~2/3 of height) showing amplitude in Counts with pinkish (#c28285) line (linewidth 0.45)
- Channel name legend box in upper-left corner of waveform panel (e.g., "EHZ")
- One spectrogram panel (lower ~1/3 of height) directly below, same width, sharing the time axis
- Spectrogram uses inferno colormap (dark purple for quiet, bright yellow/orange for energy)
- Y-axis of waveform: amplitude in Counts (with engineering formatter)
- Y-axis of spectrogram: Frequency (Hz), 0 to Nyquist (50 Hz at 100 Hz sample rate)
- X-axis: shared time axis in seconds
- At ~55 seconds, earthquake arrival: waveform amplitude spikes, spectrogram shows bright broadband energy band

**4-channel view** (`references/rsudp_4channel.png`):
- Same dark theme, title at top
- Four channel pairs stacked vertically (EHZ, ENE, ENN, ENZ)
- Each pair: waveform on top (~2/3 height), spectrogram below (~1/3 height)
- Each waveform has its own channel legend box in upper-left
- All pairs share the same time axis range
- EHZ (geophone): clear earthquake signal visible in both waveform and spectrogram
- EN* (accelerometers): higher constant noise floor, event less prominent — spectrogram shows this as a uniformly brighter background compared to EHZ
- Channels sorted: Z-ending first, then E-ending, then N-ending

**Alert markers** (`references/rsudp_alert_markers.png`):
- Vertical blue dashed line at trigger time (~55s) on BOTH waveform AND spectrogram
- Vertical red dashed line at reset time (~85s) on BOTH waveform AND spectrogram
- Lines span the full height of each panel

**Power scaling comparison** (`references/rsudp_spectrogram_scaling.png`):
- Left: Standard linear power — event overwhelms display, background noise invisible
- Right: rsudp scaling (magnitude^(1/10)) — rich detail visible in both quiet and active periods, low-frequency noise patterns and event characteristics both clearly distinguishable
- This scaling is essential for useful seismological visualization

**Existing Rust WebUI**:
- Already provides real-time waveform rendering via Canvas 2D at 30 FPS
- WebSocket streaming of binary waveform packets
- Client-side RingBuffer (Float32Array) per channel
- Alert markers (dashed vertical lines) for trigger/reset events
- Control panel with channel toggles and time window slider
- Dark theme with Tailwind CSS
- Missing: spectrogram visualization, combined waveform+spectrogram layout, channel sorting, event counter

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Real-Time Spectrogram Display (Priority: P1)

As a seismologist monitoring a Raspberry Shake station, I want to see a real-time spectrogram below each waveform channel in the WebUI, so that I can visually identify frequency content of incoming seismic signals without needing the Python rsudp desktop application.

**Why this priority**: The spectrogram is the defining visual feature of rsudp's plot that is currently missing from the WebUI. It enables frequency-domain analysis which is essential for distinguishing seismic events from noise.

**Independent Test**: Can be tested by connecting to a live Raspberry Shake or streamer simulation and observing the spectrogram update in real-time below the waveform.

**Acceptance Scenarios**:

1. **Given** the WebUI is connected via WebSocket and receiving data, **When** a channel is active, **Then** a spectrogram panel is rendered below the waveform panel for that channel, updating in real-time.
2. **Given** the spectrogram is rendering, **When** a seismic event arrives (increased energy across frequencies), **Then** the spectrogram shows a visible bright band (yellow/orange) at the event time, consistent with the inferno colormap.
3. **Given** the WebUI is running with default settings, **When** 60 seconds of data has been received, **Then** the spectrogram displays a continuous frequency-time image spanning the configured time window.

---

### User Story 2 - Combined Waveform + Spectrogram Layout (Priority: P1)

As a user familiar with rsudp's desktop plot, I want the WebUI to display waveform and spectrogram panels in the same paired layout as rsudp, so that the visual experience feels familiar and I can correlate time-domain and frequency-domain views at a glance.

**Why this priority**: The waveform+spectrogram pairing is the fundamental visual identity of rsudp plots. Without this layout, the spectrogram alone would feel disconnected.

**Independent Test**: Can be tested by viewing the WebUI with 1, 2, 3, or 4 channels active and verifying the paired layout.

**Acceptance Scenarios**:

1. **Given** 1 channel is active, **When** the dashboard loads, **Then** the display shows one waveform panel on top and one spectrogram panel below, sharing the same time axis.
2. **Given** 3 channels are active (e.g., EHZ, EHN, EHE), **When** the dashboard loads, **Then** channels are sorted Z-first, E-second, N-third, each with its own waveform+spectrogram pair stacked vertically.
3. **Given** the time window is set to 90 seconds, **When** new data arrives, **Then** both waveform and spectrogram scroll together, maintaining aligned time axes.

---

### User Story 3 - Spectrogram Configuration (Priority: P2)

As a user, I want to configure spectrogram display options (frequency range, logarithmic scale, toggle on/off) from the control panel, so that I can customize the visualization to my monitoring needs.

**Why this priority**: Configurability is important but secondary to having the spectrogram render correctly. Most users will be satisfied with defaults initially.

**Independent Test**: Can be tested by toggling spectrogram on/off, adjusting frequency range, and switching between linear and logarithmic Y-axis.

**Acceptance Scenarios**:

1. **Given** the control panel is open, **When** I toggle "Show Spectrogram" off, **Then** the spectrogram panels disappear and only waveform panels remain.
2. **Given** the spectrogram is displayed, **When** I set the frequency range to 0.5-10 Hz, **Then** the spectrogram Y-axis adjusts to show only that frequency band.
3. **Given** the spectrogram is displayed, **When** I enable "Logarithmic Y-axis", **Then** the frequency scale switches to logarithmic with appropriate tick marks (0.5, 1, 2, 5, 10, 20, 50 Hz).

---

### User Story 4 - Event Markers on Waveform (Priority: P2)

As a user monitoring for earthquakes, I want to see clearly labeled vertical markers on the waveform at STA/LTA trigger and reset times, matching rsudp's blue (trigger) and red (reset) marker colors, so that I can quickly identify detected events in the live data stream.

**Why this priority**: Event markers enhance situational awareness. The existing WebUI already has basic alert markers (dashed lines), but they need to match rsudp's color convention.

**Independent Test**: Can be tested by running simulation data that triggers an STA/LTA event and observing markers appear on the waveform with correct colors.

**Acceptance Scenarios**:

1. **Given** the waveform is rendering live data, **When** an STA/LTA trigger fires, **Then** a vertical blue dashed line appears at the trigger time position on both the waveform and spectrogram panels.
2. **Given** an active trigger is displayed, **When** the trigger resets, **Then** a vertical red dashed line appears at the reset time position on both panels.
3. **Given** multiple events have occurred within the visible time window, **When** the user looks at the waveform and spectrogram, **Then** all trigger/reset markers within the window are visible with distinct colors on both panels.

---

### User Story 5 - Event Counter in Header (Priority: P3)

As a user, I want to see a running count of detected events displayed in the dashboard header, matching the rsudp title bar format, so that I have at-a-glance awareness of seismic activity.

**Why this priority**: Cosmetic parity feature that improves the rsudp-like experience but is not critical for core visualization.

**Independent Test**: Can be tested by triggering events and observing the counter increment in the header.

**Acceptance Scenarios**:

1. **Given** the dashboard is running, **When** no events have been detected, **Then** the header shows "Detected Events: 0".
2. **Given** 3 events have been detected during the session, **When** the user views the header, **Then** it shows "Detected Events: 3".

---

### Edge Cases

- What happens when no data has been received yet? The spectrogram area shows an empty dark background, not an error or blank space.
- What happens when the browser tab is backgrounded for extended time? On return, the spectrogram rebuilds from ring buffer data without visual glitches.
- How does the spectrogram handle very short time windows (< 5 seconds)? It still renders using high overlap for temporal resolution, even if the image is sparse.
- What happens when the WebSocket disconnects and reconnects? The system backfills missing data from the server-side buffer, then resumes normal streaming. See FR-029, FR-030.
- What happens with a single sample rate channel (100 Hz) vs higher rate? FFT parameters auto-adjust to match the actual sample rate.
- What about mobile/tablet screens? Desktop-optimized layout; on smaller screens panels stack vertically with scroll. No special mobile layout is provided.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST render a real-time spectrogram panel below each active waveform channel in the WebUI.
- **FR-002**: Spectrogram MUST use FFT-based frequency analysis with NFFT set to the nearest power-of-2 to the channel's sample rate (e.g., 128 for 100 Hz), and zero-pad to NFFT*4 (e.g., 512 for 100 Hz) to interpolate the frequency output for smoother visual display. If insufficient data is available (fewer samples than NFFT), fall back to NFFT=8 with overlap=6.
- **FR-003**: Spectrogram MUST use the inferno colormap (purple-to-yellow gradient) consistent with rsudp's visual style.
- **FR-004**: Spectrogram MUST apply power scaling (magnitude^(1/10)) for better visual contrast of low-energy signals.
- **FR-028**: Spectrogram color scale (mapping from power values to colormap colors) MUST auto-adjust each frame to the min/max of the current power-scaled data, without fixed vmin/vmax. This matches rsudp's behavior where matplotlib's automatic color normalization adapts to the data range on every update.
- **FR-005**: Spectrogram and waveform panels MUST share the same time axis (in relative seconds from the start of the visible window) and scroll together as new data arrives. Note: rsudp uses different coordinate systems (waveform=UTC datetime, spectrogram=relative seconds) with manual alignment; the WebUI MUST use a unified relative-seconds axis for both panels to avoid this complexity.
- **FR-026**: The shared time axis MUST display elapsed time in seconds (e.g., 0, 10, 20, ..., 90) relative to the left edge of the visible window, with the X-axis label "Time (seconds)" shown only on the bottom-most panel.
- **FR-027**: When multiple channels are displayed, ALL waveform panels and ALL spectrogram panels MUST share the same time axis range so that scrolling and time alignment are consistent across all channel pairs.
- **FR-006**: System MUST sort active channels in display order: Z-ending first, then E-ending, then N-ending, then others alphabetically.
- **FR-007**: System MUST provide a toggle in the control panel to show/hide spectrogram panels.
- **FR-008**: System MUST support configurable frequency range limits (lower and upper bounds in Hz) for the spectrogram display.
- **FR-009**: System MUST support logarithmic Y-axis mode for the spectrogram with standard frequency tick marks (0.5, 1, 2, 5, 10, 20, 50 Hz).
- **FR-010**: System MUST display vertical event markers at STA/LTA trigger (blue dashed) and reset (red dashed) times on BOTH the waveform AND spectrogram panels, spanning the full height of each panel.
- **FR-011**: System MUST display a running event counter in the dashboard header showing the total detected events for the current session.
- **FR-024**: The plot area MUST display a centered title text at the top in the format "{STATION} Live Data - Detected Events: {N}" (e.g., "AM.R6E01 Live Data - Detected Events: 3"), using the foreground color on the dark background.
- **FR-025**: The plot area MUST display "rsudp-rust" as a branding text in the top-left corner (the position where rsudp displays the Raspberry Shake logo), independent of the title text.
- **FR-031**: System MUST display a seismic intensity class indicator in the top-right corner of the dashboard as a badge: a rounded-corner box with the intensity class number (e.g., "3", "5+", "6-") in large white text on a JMA color background. The indicator is NOT shown by default (hidden when no alert is active). It MUST appear only when an STA/LTA trigger fires, showing the maximum JMA seismic intensity class observed since the trigger start, and remain visible until 30 seconds after the trigger resets. After the 30-second post-reset period, the indicator MUST disappear. The badge background MUST use the same color scale as the Hue integration:
  - Intensity 1: #F2F2FF (very light blue)
  - Intensity 2: #00AAFF (light blue)
  - Intensity 3: #0041FF (blue)
  - Intensity 4: #FAE696 (light yellow)
  - Intensity 5-: #FFE600 (yellow)
  - Intensity 5+: #FF9900 (orange)
  - Intensity 6-: #FF2800 (red)
  - Intensity 6+: #A50021 (dark red)
  - Intensity 7: #B40068 (magenta)
- **FR-032**: During the visible period (from trigger to 30 seconds after reset), the indicator MUST update in real-time: if the maximum intensity class increases, the displayed class and color MUST update immediately.
- **FR-034**: If a new STA/LTA trigger fires while the previous indicator is still visible (within the 30-second post-reset window), the indicator MUST reset and begin tracking the new event's maximum intensity class from the new trigger time.
- **FR-035**: The display time window MUST be configurable between 5 seconds and 300 seconds maximum. This upper limit serves as an indirect memory constraint for the client-side spectrogram and waveform buffers.
- **FR-012**: Spectrogram rendering MUST maintain at least 15 FPS update rate without degrading waveform rendering performance.
- **FR-013**: Spectrogram MUST be rendered using the Canvas 2D API, consistent with the existing waveform rendering approach.
- **FR-014**: Spectrogram overlap MUST adapt to the display time window: 97.5% overlap (NFFT*0.975) for windows of 60 seconds or shorter (high time resolution), and 90% overlap (NFFT*0.9) for windows longer than 60 seconds (reduced computation).
- **FR-015**: System MUST apply a Hanning window function to FFT input data before computing the frequency spectrum.
- **FR-016**: The entire WebUI page MUST use a dark theme with background #202530, replacing the existing light theme. Waveform line color MUST be #c28285 (pinkish, linewidth ~0.45). All UI elements (header, control panel, cards, text) MUST be restyled to work on the dark background with light foreground text (rgba(204, 204, 204, 1.0)).
- **FR-017**: Spectrogram Y-axis MUST display frequency labels in Hz.
- **FR-018**: The waveform panel and spectrogram panel height ratio MUST be approximately 2:1 (waveform ~2/3, spectrogram ~1/3 of the channel pair's vertical space).
- **FR-019**: Each waveform panel MUST display the channel name (e.g., "EHZ") as a legend label in the upper-left corner of the panel.
- **FR-020**: Waveform Y-axis MUST display amplitude values with unit labels determined by the deconvolution setting, following the rsudp-compatible unit table:
  - `VEL` → "Velocity (m/s)"
  - `ACC` → "Acceleration (m/s²)"
  - `GRAV` → "Earth gravity (g)"
  - `DISP` → "Displacement (m)"
  - `CHAN` → channel-specific: EH-prefixed channels → "Velocity (m/s)", EN-prefixed channels → "Acceleration (m/s²)", all others → "Counts"
  - No deconvolution (default) → "Counts"
  Values MUST use engineering notation (SI prefixes: μ, m, k, M, etc.) when the amplitude range makes it appropriate, consistent with rsudp's EngFormatter behavior.
- **FR-021**: Waveform MUST be centered by subtracting the mean of all samples within the visible time window, so that Y=0 represents the window's average amplitude (DC offset removal). This ensures the waveform baseline stays at the vertical center of the panel regardless of sensor DC drift.
- **FR-022**: Waveform Y-axis range MUST auto-scale to the min/max of the mean-subtracted data with 10% vertical padding on both top and bottom.
- **FR-023**: Spectrogram FFT input data MUST also have the per-window mean subtracted before computing the frequency spectrum, consistent with the waveform's DC offset removal.
- **FR-029**: When the WebSocket connection is established (both initial page load and reconnection after disconnection), the client MUST send the timestamp of the last received sample to the server (or no timestamp on initial load). The server MUST respond with all buffered data from that timestamp to the current time (backfill). On initial load with no timestamp, the server MUST send all currently buffered data so the user immediately sees the full available time window. If the gap exceeds the server-side buffer capacity, the server MUST return as much data as available from the buffer start. The server-side buffer MUST retain data for the duration matching the configured display time window (e.g., 90 seconds at default settings, up to 300 seconds if the user increases the time window).
- **FR-030**: After backfill data is received, both the waveform and spectrogram MUST re-render seamlessly, incorporating the backfilled data without visual discontinuities or duplicate samples. On initial page load, the waveform and spectrogram MUST be populated and visible within 1 second of the WebSocket connection being established.

### Key Entities

- **SpectrogramState**: Per-channel state holding FFT computation results (frequency bins, time slices, magnitude matrix) needed to render the spectrogram image as a 2D heatmap.
- **ChannelPair**: A paired waveform+spectrogram visual unit for one seismic channel, including shared time axis reference and synchronized scrolling behavior.
- **EventMarker**: A trigger or reset event with timestamp, type (trigger/reset), and associated channel, used to render vertical indicator lines on the waveform.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users see a live spectrogram below each waveform channel within 3 seconds of the first data packet arriving.
- **SC-002**: The combined waveform+spectrogram layout visually matches rsudp's single-channel and multi-channel plot layouts as verified by side-by-side comparison with reference screenshots (`references/rsudp_single_channel.png`, `references/rsudp_4channel.png`).
- **SC-003**: Dashboard renders waveform+spectrogram at 15+ FPS for up to 4 channels simultaneously on a standard desktop browser.
- **SC-004**: Spectrogram toggle, frequency range, and log scale controls are accessible within 2 clicks from the main dashboard view.
- **SC-005**: Event trigger/reset markers appear on the waveform within 1 second of the backend broadcasting the alert event.
- **SC-006**: Users familiar with rsudp can identify the same visual patterns (event frequency signatures, noise floor characteristics) in the WebUI spectrogram as they would in the rsudp desktop plot.

## Clarifications

### Session 2026-02-10

- Q: FR-016のダークテーマはプロットエリアのみかページ全体か？ → A: ページ全体をダークテーマ (#202530) に変更。既存ライトテーマは廃止。
- Q: サーバー側バッファの保持期間は？ → A: 表示ウィンドウの設定値に連動（デフォルト90秒、ユーザーがウィンドウを広げれば追従）。
- Q: 震度階級インジケータの表示形式は？ → A: バッジ型。角丸ボックスに震度階級数字を大きめ白文字で表示、背景色はJMAカラー。
- Q: モバイル/レスポンシブ対応は？ → A: デスクトップ優先。モバイルはそのまま縦スクロールで表示、レスポンシブ対応は最低限。
- Q: スペクトログラムのメモリ上限は？ → A: 表示ウィンドウの最大値を300秒に制限し、間接的にメモリを管理。

## Assumptions

- The existing WebSocket streaming infrastructure (binary waveform packets at ~100 Hz) provides sufficient data for real-time spectrogram computation on the client side.
- FFT computation in the browser is fast enough for real-time spectrogram rendering at 100 Hz sample rate with 128-point FFT.
- The existing RingBuffer implementation can supply raw sample data for FFT input.
- The inferno colormap can be implemented as a 256-entry lookup table without external dependencies.
- 100 Hz is the standard Raspberry Shake sample rate; the system should handle this as the primary case.
- The existing dark theme can be adjusted to match rsudp's exact color values (#202530, #c28285) without breaking other UI elements.
