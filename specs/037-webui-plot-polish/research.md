# Research: WebUI Plot Polish

## R1: 1-2-5 Series Tick Algorithm

**Decision**: Use the standard "nice number" algorithm (Heckbert, 1990) to compute axis tick intervals.

**Rationale**: This is the de facto standard for scientific plot axis labeling. The algorithm produces human-readable tick values (multiples of 1, 2, or 5 at appropriate magnitudes) and is used by matplotlib, d3.js, plotters, and virtually all plotting libraries.

**Algorithm**:
```
function niceStep(range, targetTicks):
  roughStep = range / targetTicks
  magnitude = 10^floor(log10(roughStep))
  fraction = roughStep / magnitude
  if fraction <= 1.5: niceStep = 1 * magnitude
  else if fraction <= 3.5: niceStep = 2 * magnitude
  else if fraction <= 7.5: niceStep = 5 * magnitude
  else: niceStep = 10 * magnitude
  return niceStep
```

**Alternatives considered**:
- Linear interpolation (current approach): Produces non-round values like -1100, -550. Rejected.
- d3-scale library: Would add a dependency for just one function. Rejected.
- Fixed tick count with formatting: Still produces ugly numbers. Rejected.

## R2: Timestamp Propagation to Canvas

**Decision**: Pass `latestTimestamp: Date | null` as a prop to ChannelPairCanvas, computed from the last received WaveformPacket's timestamp + sample count offset.

**Rationale**: The right edge of the waveform represents the most recent sample. By computing this as `packet.timestamp + (packet.samples.length / packet.sample_rate) * 1000`, we get the precise time of the right edge. This avoids modifying RingBuffer (which is sample-only) and keeps the timestamp propagation simple.

**Alternatives considered**:
- Modify RingBuffer to store timestamps: Would require significant refactoring of a core data structure. Rejected.
- Use `new Date()` as "now": Would be inaccurate when backfill data is displayed (right edge would show future time). Rejected.
- Pass full timestamp array: Unnecessary overhead; only the latest timestamp is needed since samples are evenly spaced. Rejected.

## R3: Time Label Alignment

**Decision**: Align time labels to 10-second boundaries (seconds divisible by 10), not to relative offsets.

**Rationale**: The rsudp desktop application shows labels at clock-aligned positions (e.g., 09:01:00, 09:01:10, 09:01:20). This is more natural for reading and matches the reference screenshot.

**Algorithm**:
```
rightEdge = latestTimestamp
leftEdge = rightEdge - windowSeconds * 1000
firstTick = ceil(leftEdge / 10000) * 10000  // Next 10-second boundary after left edge
for t = firstTick; t <= rightEdge; t += 10000:
  x = LEFT_MARGIN + ((t - leftEdge) / (windowSeconds * 1000)) * plotWidth
  label = formatHHMMSS(t)
  drawLabel(x, label)
```

**Alternatives considered**:
- Labels at fixed pixel intervals: Would not align to clock boundaries. Rejected.
- Labels at 20-second intervals: Reference screenshot appears to use 20s intervals for wider windows, but user explicitly requested 10s. Using 10s.

## R4: Border Color and Style

**Decision**: Use `rgba(255, 255, 255, 0.6)` with 1px line width for plot area borders.

**Rationale**: The rsudp desktop reference screenshot shows a light gray/white border that is visible but not dominant. A semi-transparent white at 60% opacity provides the right contrast against the dark background (#202530) without being harsh.

**Alternatives considered**:
- Solid white (#FFFFFF): Too bright/harsh against dark background. Rejected.
- Gray (#CCCCCC): Less visible, especially on the spectrogram. Rejected.
- CSS border on canvas element: Would include margins/axis area inside the border. Rejected — must use canvas `strokeRect` to frame only the data area.

## R5: Time Label Placement (from reference screenshot analysis)

**Decision**: Draw time labels in a gap area between the waveform and spectrogram canvases (below the waveform border, above the spectrogram border). Draw "Time (UTC)" axis title below the spectrogram.

**Rationale**: The reference screenshot clearly shows time labels (09:01:00, 09:01:10, ...) positioned in a gap between the two plots, not below the spectrogram. This matches matplotlib's standard subplot layout where shared x-axis labels go between plots.

**Implementation approach**:
- Move the TIME_AXIS_HEIGHT area from below the spectrogram to below the waveform canvas
- The waveform canvas gets extra height for time labels below its border
- The spectrogram canvas gets extra height for "Time (UTC)" label below its border
- When spectrogram is hidden, time labels + "Time (UTC)" both go below the waveform

**Alternatives considered**:
- Time labels below spectrogram only (current implementation): Does not match reference. Rejected.
- Duplicate time labels on both plots: Wastes space, not in reference. Rejected.

## R6: Horizontal Grid Lines

**Decision**: Add faint horizontal grid lines at Y-axis tick positions inside the waveform plot area.

**Rationale**: The reference screenshot shows faint grey horizontal lines at each Y-axis tick position. These aid amplitude reading at a glance without cluttering the display.

**Style**: `rgba(255, 255, 255, 0.15)`, 1px line width, spanning from LEFT_MARGIN to LEFT_MARGIN + plotWidth.

**Alternatives considered**:
- No grid lines (current): Harder to read amplitudes. Reference has them. Rejected.
- Full grid (horizontal + vertical): Too cluttered, reference only shows horizontal. Rejected.

## R7: Y-Axis Range (non-symmetric)

**Decision**: Use actual data min/max for Y-axis range instead of forcing symmetric around zero.

**Rationale**: The reference screenshot shows a Y-axis range of -2 to +3 mm/s, which is NOT symmetric around zero. The current code forces symmetry (`yMin = -maxAbs * 1.1, yMax = maxAbs * 1.1`). Changing to use actual data bounds with nice-number rounding will better represent asymmetric signals.

**Implementation**: After computing DC-offset-removed min/max from the sample buffer, expand to the nearest nice-number boundaries using the 1-2-5 step. E.g., if data ranges from -1.8 to +2.7, with step=1, the ticks become -2, -1, 0, 1, 2, 3.

**Alternatives considered**:
- Keep symmetric (current): Does not match reference behavior. Rejected for this feature.

## R8: Alert History Removal Scope

**Decision**: Remove the history page, navigation link, and the `AlertEvent` type. Keep `useAlerts.ts` hook and `VisualAlertMarker` type.

**Rationale**:
- `useAlerts.ts` provides audio playback on alerts — this is a live monitoring feature, not related to history viewing
- `VisualAlertMarker` is used by ChannelPairCanvas to draw trigger/reset markers on the waveform — still needed
- `AlertEvent` is only imported by the history page — safe to remove
- No backend changes needed (the `/api/alerts` endpoint can remain; removing it is out of scope)

**Alternatives considered**:
- Remove all alert-related code: Would break audio playback and waveform alert markers. Rejected.
- Remove backend `/api/alerts` endpoint: Out of scope per spec. Rejected.
