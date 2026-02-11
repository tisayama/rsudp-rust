# Feature Specification: WebUI Plot Polish

**Feature Branch**: `037-webui-plot-polish`
**Created**: 2025-02-10
**Status**: Draft
**Input**: User description: "Improve WebUI waveform and spectrogram display to match rsudp desktop application visuals. Fix Y-axis tick spacing, add white border frames, switch to absolute time labels, and remove Alert History feature."

**Reference**: See `references/R6E01-2025-11-25-090123.png` for the target visual appearance from rsudp desktop.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Absolute Time Axis Labels (Priority: P1)

A seismologist viewing the WebUI dashboard needs to see absolute timestamps (HH:MM:SS in UTC) on the X-axis instead of relative seconds starting from zero. This allows them to correlate seismic events with exact times, matching the rsudp desktop application behavior. Time labels should appear at regular intervals (every 10 seconds) and scroll with the waveform data.

**Why this priority**: Absolute time is essential for seismic event analysis. Relative time starting from 0 provides no actionable temporal reference and makes it impossible to correlate events across instruments or with external data sources.

**Independent Test**: Can be fully tested by opening the WebUI with live data and verifying that X-axis labels show HH:MM:SS times that match the current UTC clock, updating as data scrolls.

**Acceptance Scenarios**:

1. **Given** the WebUI is displaying live waveform data, **When** the user views the X-axis, **Then** time labels are shown in HH:MM:SS format (UTC) at 10-second intervals.
2. **Given** the waveform is scrolling with new data, **When** new data arrives, **Then** the time labels shift left with the data and new labels appear at the right edge maintaining 10-second spacing.
3. **Given** the WebUI loads with backfill data, **When** the initial display renders, **Then** time labels reflect the actual timestamps of the backfilled data, not starting from zero.
4. **Given** the spectrogram is displayed below the waveform, **When** the user views the X-axis, **Then** the spectrogram shares the same absolute time axis as the waveform above it.

---

### User Story 2 - White Border Frames Around Plots (Priority: P1)

A user viewing the dashboard should see clear white/light-colored border frames around both the waveform plot area and the spectrogram plot area, matching the rsudp desktop application. This provides visual separation between the plot content and the surrounding dark background, improving readability.

**Why this priority**: The border frames are a core visual element of the rsudp reference design and significantly improve plot readability by clearly delineating the data area boundaries.

**Independent Test**: Can be fully tested by opening the WebUI and visually confirming that both waveform and spectrogram have white rectangular border frames around their plot areas.

**Acceptance Scenarios**:

1. **Given** the WebUI is displaying a channel with waveform, **When** the user views the waveform plot, **Then** a white/light border rectangle frames the plot area (left margin to right edge, top to bottom of plot).
2. **Given** the WebUI is displaying spectrogram below waveform, **When** the user views the spectrogram, **Then** a white/light border rectangle frames the spectrogram area.
3. **Given** only waveform is displayed (no spectrogram), **When** the user views the plot, **Then** the waveform still has a white border frame.

---

### User Story 3 - Y-Axis Tick Spacing with Round Numbers (Priority: P1)

A seismologist viewing the waveform needs Y-axis tick marks at evenly-spaced round numbers (using a 1-2-5 series pattern), similar to the rsudp desktop application. Currently, the Y-axis ticks are irregularly spaced, making it difficult to read signal amplitudes at a glance.

**Why this priority**: Proper Y-axis scaling with round number ticks is fundamental for reading waveform amplitudes. The current irregular spacing reduces data readability.

**Independent Test**: Can be fully tested by opening the WebUI with live data and verifying that Y-axis tick labels use round numbers (e.g., -2, -1, 0, 1, 2 or -500, 0, 500) with even spacing.

**Acceptance Scenarios**:

1. **Given** the waveform is displaying data, **When** the user views the Y-axis, **Then** tick marks are placed at evenly-spaced round numbers following a 1-2-5 series (e.g., ..., 0.1, 0.2, 0.5, 1, 2, 5, 10, 20, 50, ...).
2. **Given** the waveform amplitude changes (e.g., during an earthquake), **When** the Y-axis auto-scales, **Then** the tick labels update to new round numbers that fit the new range while maintaining even spacing.
3. **Given** different channels with different amplitude ranges, **When** viewing multiple channels simultaneously, **Then** each channel's Y-axis independently shows appropriate round-number ticks for its data range.

---

### User Story 4 - Remove Alert History Feature (Priority: P2)

The Alert History page and its related functionality are no longer needed. The /history route, Alert History navigation, and associated code should be removed to simplify the application and reduce maintenance burden.

**Why this priority**: Removing unused features reduces code complexity. This is P2 because it doesn't affect the visual quality of the main dashboard, but should be done as part of this polish pass to keep the codebase clean.

**Independent Test**: Can be tested by verifying that navigating to /history returns a 404 or redirects, and that no Alert History links or buttons appear in the UI.

**Acceptance Scenarios**:

1. **Given** the WebUI is loaded, **When** the user views the navigation/header area, **Then** there is no link or button for "Alert History" or "/history".
2. **Given** a user navigates directly to /history, **When** the page loads, **Then** they see a 404 page or are redirected to the main dashboard.
3. **Given** the codebase after this change, **When** reviewing the code, **Then** there are no unused Alert History components, hooks, or page files remaining.

---

### Edge Cases

- What happens when the time window crosses a minute or hour boundary? Time labels should correctly display the transition (e.g., 09:01:50 → 09:02:00 → 09:02:10).
- What happens when the data has very small amplitude (near zero)? Y-axis ticks should still show round numbers with appropriate decimal precision (e.g., -0.001, 0, 0.001).
- What happens when the data has very large amplitude range? Y-axis should scale to appropriate round numbers (e.g., -1000, -500, 0, 500, 1000).
- How do borders render when the browser window is very narrow? The border frame should scale with the canvas, maintaining visibility.
- What happens during the backfill-to-live transition? Time labels should remain continuous with no gap or jump.

## Requirements *(mandatory)*

### Functional Requirements

**Time Axis (X-Axis)**:
- **FR-001**: The X-axis MUST display absolute timestamps in HH:MM:SS format (UTC).
- **FR-002**: Time labels MUST be placed at 10-second intervals along the X-axis, aligned to clock boundaries (seconds divisible by 10).
- **FR-003**: Time labels MUST scroll with the waveform data as new samples arrive.
- **FR-004**: The spectrogram X-axis MUST align with the waveform X-axis above it (same time range and labels).
- **FR-005**: Backfilled data MUST show correct historical timestamps, not relative offsets.
- **FR-006**: When spectrogram is displayed, time labels MUST be drawn between the waveform and spectrogram (in a gap area below the waveform frame).
- **FR-007**: A "Time (UTC)" axis title MUST be displayed below the spectrogram (or below the waveform if spectrogram is hidden).

**Plot Borders**:
- **FR-008**: A white/light-colored rectangular border MUST be drawn around the waveform plot area for each channel.
- **FR-009**: A white/light-colored rectangular border MUST be drawn around the spectrogram plot area for each channel (when spectrogram is displayed).
- **FR-010**: Borders MUST frame the data area only (inside the axis margins), matching the rsudp desktop style.

**Y-Axis Ticks & Grid Lines**:
- **FR-011**: Y-axis tick marks MUST be placed at evenly-spaced round numbers using a 1-2-5 series algorithm.
- **FR-012**: Y-axis tick labels MUST update when the waveform auto-scales to a new amplitude range.
- **FR-013**: The number of Y-axis ticks MUST be between 3 and 7 (inclusive) to avoid clutter while maintaining readability.
- **FR-014**: Y-axis tick values MUST be formatted appropriately (no excessive decimal places; use scientific notation or SI prefixes only if the range demands it).
- **FR-015**: Faint horizontal grid lines MUST be drawn at each Y-axis tick position across the full plot width.
- **FR-016**: The Y-axis range MUST use the actual data range (not forced symmetric around zero).

**Spectrogram Axis Label**:
- **FR-017**: The spectrogram frequency axis label MUST read "Frequency (Hz)" (not just "Hz").

**Alert History Removal**:
- **FR-018**: The /history page MUST be removed from the application.
- **FR-019**: Any navigation links or buttons referencing Alert History MUST be removed.
- **FR-020**: Unused Alert History components, hooks, and utility code MUST be removed.
- **FR-021**: Removing Alert History MUST NOT affect other dashboard functionality (waveform, spectrogram, intensity badge, connection status).

## Assumptions

- The backend WebSocket already provides timestamps with each waveform packet (confirmed: `tsMicros` field exists in binary waveform packets).
- The existing canvas-based rendering approach will be maintained (no switch to a charting library).
- "White border" refers to a light-colored stroke (e.g., `#CCCCCC` or `rgba(255,255,255,0.5)`) consistent with the rsudp desktop appearance in the reference screenshot.
- The 10-second interval for time labels is the default; this matches the rsudp desktop behavior visible in the reference screenshot.
- UTC timezone is used for all time displays, consistent with seismological convention.

## Out of Scope

- Changing the spectrogram colormap or rendering algorithm.
- Adding zoom or pan interactions to the plots.
- Changing the overall page layout or theme colors beyond the specified border additions.
- Modifying the intensity badge or connection status indicator styling.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can read the exact UTC time of any point on the waveform within 1-second accuracy by referencing the X-axis labels.
- **SC-002**: Plot borders are visually consistent with the rsudp desktop reference screenshot when compared side-by-side.
- **SC-003**: Y-axis tick values are always round numbers (from 1-2-5 series) with even spacing, regardless of signal amplitude.
- **SC-004**: No traces of Alert History remain in the UI or codebase after removal (no dead code, no broken links).
- **SC-005**: All existing dashboard functionality (live waveform, spectrogram, intensity badge, backfill) continues to work correctly after changes.
- **SC-006**: The visual appearance of the WebUI waveform and spectrogram closely matches the rsudp desktop reference screenshot in terms of axis labeling, borders, and tick spacing.
