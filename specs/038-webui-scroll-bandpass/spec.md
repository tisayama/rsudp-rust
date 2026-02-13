# Feature Specification: WebUI Scroll Sync & Bandpass Filter

**Feature Branch**: `038-webui-scroll-bandpass`
**Created**: 2026-02-10
**Status**: Draft
**Input**: User description: "WebUI further polish: right-align backfill data, sync channel scrolling, fix waveform/spectrogram alignment, implement bandpass filter & range display"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Right-Aligned Backfill Display (Priority: P1)

When the application starts or reconnects, the server sends backfill data that may be shorter than the configured Time Window. Currently, this data renders left-to-right starting from the left edge of the plot, leaving empty space on the right. Users expect the most recent data to appear at the right edge of the plot (right-aligned), with empty space on the left side, just as it would appear during normal live scrolling.

Once enough data accumulates to fill the Time Window, the display transitions seamlessly to the normal live scrolling behavior (data scrolling from right to left).

**Why this priority**: This is the most visually jarring issue during startup — the waveform appearing to "grow" from the left is confusing and inconsistent with standard seismograph display conventions.

**Independent Test**: Start the WebUI with a fresh connection. The initial partial data should appear right-aligned in the plot. After the Time Window fills, scrolling continues normally.

**Acceptance Scenarios**:

1. **Given** the server buffer has 30 seconds of data and Time Window is 90 seconds, **When** the WebUI first connects and receives backfill, **Then** the 30-second waveform appears right-aligned with 60 seconds of empty space on the left.
2. **Given** the backfill data is right-aligned, **When** new live data arrives and eventually fills the Time Window, **Then** the display transitions smoothly to normal left-scrolling behavior with no visual jump.
3. **Given** the Time Window is changed to a shorter value (e.g., 20 seconds) while only 30 seconds of data exist, **When** the display updates, **Then** the full 20-second window is filled with the most recent data.

---

### User Story 2 - Synchronized Channel Scrolling (Priority: P1)

When multiple channels are displayed, each channel's waveform currently scrolls independently based on when its data packets arrive. This creates a visually distracting effect where channels appear to scroll at slightly different times. All channels should scroll in unison using a single shared time reference.

**Why this priority**: The unsynchronized scrolling makes it difficult to visually compare signals across channels — a core use case for multi-channel seismograph displays.

**Independent Test**: Open the WebUI with 3+ channels visible. All channels should scroll together in perfect sync, with time axis labels aligned across all channels.

**Acceptance Scenarios**:

1. **Given** multiple channels are displayed, **When** new data packets arrive at slightly different times for each channel, **Then** all waveforms scroll together using a single shared timestamp reference.
2. **Given** channels are synchronized, **When** the user changes the Time Window setting, **Then** all channels update simultaneously to the new window size.

---

### User Story 3 - Waveform/Spectrogram Scroll Alignment (Priority: P1)

The waveform and spectrogram for the same channel sometimes display slightly offset time positions. This occurs because the waveform uses sample-count-based positioning while the spectrogram uses column-count-based positioning, and these two approaches can drift apart. Both plots for a given channel must use the same time reference so that features in the waveform align precisely with their corresponding frequency content in the spectrogram.

**Why this priority**: Misaligned waveform and spectrogram defeats the purpose of showing them together — users need to correlate time-domain and frequency-domain features visually.

**Independent Test**: Open the WebUI with spectrogram enabled. Alert trigger lines and data features should appear at exactly the same horizontal position in both the waveform and spectrogram plots.

**Acceptance Scenarios**:

1. **Given** a channel with both waveform and spectrogram visible, **When** an alert trigger marker is displayed, **Then** the marker appears at the same horizontal (time) position in both plots.
2. **Given** the spectrogram and waveform share the same time axis, **When** the display scrolls, **Then** both plots move together with no relative drift.

---

### User Story 4 - Bandpass Filter & Range Display (Priority: P2)

The rsudp reference application displays "Bandpass (X - Y Hz)" in the lower-left corner of the waveform plot and "Range (X - Y Hz)" in the lower-left corner of the spectrogram plot. These labels indicate the active frequency filtering applied to the waveform data and the frequency range displayed in the spectrogram. The WebUI should display equivalent labels showing the current bandpass filter and spectrogram range settings.

**Why this priority**: This is a visual polish item that provides important metadata to users. It is lower priority because it does not affect data correctness, but it improves the information density of the display to match the reference application.

**Independent Test**: Open the WebUI and verify that "Bandpass (X - Y Hz)" appears in the waveform plot lower-left and "Range (X - Y Hz)" appears in the spectrogram plot lower-left, with values matching the configured settings.

**Acceptance Scenarios**:

1. **Given** the bandpass filter is configured with low=0.7 Hz and high=9.0 Hz, **When** the waveform plot renders, **Then** the label "Bandpass (0.7 - 9.0 Hz)" appears in the lower-left corner of the waveform data area.
2. **Given** the spectrogram frequency range is 0-50 Hz, **When** the spectrogram renders, **Then** the label "Range (0 - 50 Hz)" appears in the lower-left corner of the spectrogram data area.
3. **Given** the user changes the spectrogram frequency range in the control panel, **When** the spectrogram re-renders, **Then** the "Range" label updates to reflect the new values.

---

### Edge Cases

- What happens when a channel receives no data at all during backfill? The plot area should remain empty (background only) with no waveform drawn.
- What happens when one channel's data stream is delayed or temporarily interrupted while others continue? The delayed channel should show a gap or frozen last position while other channels continue scrolling (they still share the same time reference).
- What happens when the bandpass filter is not configured (disabled)? The "Bandpass" label is hidden entirely (not rendered).
- What happens when all channels start with different amounts of backfill data? Each channel renders only the data it has, but all are right-aligned to the same shared time reference, so the right edges are aligned even if data lengths differ.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST display waveform data right-aligned when the available data is shorter than the Time Window, with empty space on the left side of the plot.
- **FR-002**: System MUST transition seamlessly from right-aligned partial display to normal scrolling once the data fills the Time Window.
- **FR-003**: System MUST use a single shared time reference across all displayed channels so that all waveforms scroll in unison.
- **FR-004**: The shared time reference MUST be the maximum (most recent) timestamp across all channels' latest received data.
- **FR-005**: System MUST use the same time reference for both waveform and spectrogram rendering within each channel, ensuring both plots are horizontally aligned.
- **FR-006**: System MUST display a "Bandpass (X - Y Hz)" label in the lower-left corner of each waveform plot area, showing the `[plot]` section's `filter_highpass` and `filter_lowpass` values (the waveform display filter, not the alert trigger filter).
- **FR-007**: System MUST display a "Range (X - Y Hz)" label in the lower-left corner of each spectrogram plot area, showing the current spectrogram frequency range.
- **FR-008**: Bandpass and Range label values MUST update dynamically when settings change.
- **FR-009**: System MUST render waveform data starting from the correct x-position based on the timestamp of each sample relative to the shared time axis.
- **FR-010**: System MUST render spectrogram columns at x-positions computed from the shared time axis (not from column count or array index).
- **FR-011**: When bandpass filtering is not configured (disabled), the "Bandpass" label MUST be hidden entirely (not rendered).
- **FR-012**: The shared time reference and right-alignment logic MUST work correctly for any Time Window duration (10s to 300s).

### Key Entities

- **Shared Time Reference**: A single timestamp (the most recent across all channels) used as the right edge of the time axis for all plots. Maintained at the page level and passed to all channel components.
- **Bandpass Filter Settings**: Low and high cutoff frequencies for the waveform bandpass filter. Sourced from the backend `[plot]` configuration section (`filter_highpass`, `filter_lowpass`), not the `[alert]` section which has different values for STA/LTA trigger detection.
- **Spectrogram Range Settings**: Minimum and maximum frequency values for the spectrogram display range. Already exist as `spectrogram_freq_min` and `spectrogram_freq_max` in PlotSettings.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: During backfill, waveform data appears at the right edge of the plot within 1 render frame of data arrival, with empty space visible on the left.
- **SC-002**: All visible channels scroll in perfect visual sync — no observable horizontal offset between channels' time positions.
- **SC-003**: Waveform alert markers and spectrogram alert markers for the same event appear at the same horizontal pixel position (within 2px tolerance).
- **SC-004**: "Bandpass" and "Range" labels are visible and display correct frequency values matching the active configuration.
- **SC-005**: Transition from partial backfill display to full-window scrolling occurs without any visible jump, flicker, or discontinuity.

## Clarifications

### Session 2026-02-10

- Q: Which bandpass filter values should the "Bandpass" label display — `[plot]` filter (0.7-2.0 Hz) or `[alert]` filter (0.1-2.0 Hz)? → A: Show `[plot]` section filter values (waveform display filter).
- Q: What should the "Bandpass" label show when filtering is disabled? → A: Hide the label entirely (not rendered).

## Assumptions

- The bandpass filter frequency range is available from the backend configuration (or will be added as new fields in the settings API).
- The spectrogram frequency range is already available in PlotSettings (`spectrogram_freq_min`, `spectrogram_freq_max`).
- The current rendering approach (30 FPS interval-based canvas drawing) will be maintained; no change to the rendering architecture.
- "Right-aligned" means the most recent sample aligns with the right edge of the plot, matching the behavior once live data fills the window.

## Out of Scope

- Actual signal processing (bandpass filtering of waveform data) — only the display labels are in scope. If backend already applies filtering, we display the parameters; we do not implement a new filter.
- Changes to the spectrogram computation algorithm.
- Changes to the backend data streaming protocol or WebSocket message format.
- Persistent storage of scroll synchronization state.
