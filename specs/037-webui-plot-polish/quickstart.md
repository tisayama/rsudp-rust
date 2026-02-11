# Quickstart: WebUI Plot Polish Verification

## Prerequisites

- Docker + Docker Compose running
- Backend streaming waveform data via WebSocket
- Browser accessing WebUI at http://localhost:3000

## Verification Scenarios

### Scenario 1: Absolute Time Axis

1. Open the WebUI dashboard
2. Wait for data to appear (backfill + live streaming)
3. **Verify**: X-axis shows HH:MM:SS labels (UTC) at 10-second intervals
4. **Verify**: Labels scroll left as new data arrives
5. **Verify**: Labels align to clock boundaries (e.g., 09:01:00, 09:01:10, 09:01:20)
6. **Verify**: Time labels appear in the gap **between** waveform and spectrogram (not below spectrogram)
7. **Verify**: "Time (UTC)" axis title appears below the spectrogram
8. **Verify**: Time labels appear on every channel pair, not just the bottom one
9. **Compare**: Side-by-side with reference screenshot `references/R6E01-2025-11-25-090123.png`

### Scenario 2: White Border Frames

1. Open the WebUI dashboard with live data
2. **Verify**: White/light rectangular border frames the waveform plot area (not the full canvas with margins)
3. **Verify**: White/light rectangular border frames the spectrogram plot area
4. **Verify**: There is a visible gap between waveform border and spectrogram border (for time labels)
5. **Verify**: Borders are visible but not overpowering
6. **Compare**: Border style matches rsudp desktop reference screenshot

### Scenario 3: Y-Axis Round Number Ticks & Grid Lines

1. Open the WebUI dashboard with live data
2. **Verify**: Y-axis labels show round numbers (e.g., -2, -1, 0, 1, 2 or -500, 0, 500)
3. **Verify**: Ticks are evenly spaced (equal pixel distance between each)
4. **Verify**: Number of ticks is between 3 and 7
5. **Verify**: Faint horizontal grid lines are visible at each Y-axis tick position
6. **Verify**: Y-axis range is NOT forced symmetric — reflects actual data min/max
7. Trigger a seismic event (or inject large amplitude data)
8. **Verify**: Y-axis rescales to new round numbers appropriate for the larger range
9. **Compare**: Tick style matches rsudp desktop reference screenshot

### Scenario 4: Spectrogram Axis Label

1. Open the WebUI dashboard with spectrogram enabled
2. **Verify**: Frequency axis label reads "Frequency (Hz)" (not just "Hz")

### Scenario 5: Alert History Removed

1. Open the WebUI dashboard
2. **Verify**: No "View Alert History" button in the sidebar
3. Navigate directly to http://localhost:3000/history
4. **Verify**: 404 page is shown (Next.js default)
5. **Verify**: Audio alerts still work when an event triggers
6. **Verify**: Alert markers still appear on waveform when triggered

### Scenario 6: Overall Visual Comparison

1. Open the WebUI dashboard with live data streaming
2. Open the reference screenshot `references/R6E01-2025-11-25-090123.png` side-by-side
3. **Verify**: The WebUI closely matches the rsudp desktop in:
   - Time axis formatting (HH:MM:SS) and position (between plots)
   - Y-axis tick spacing and round numbers
   - Horizontal grid lines at tick positions
   - Plot border frames around both waveform and spectrogram
   - "Time (UTC)" label below spectrogram
   - "Frequency (Hz)" label on spectrogram Y-axis
4. **Verify**: Spectrogram still renders correctly with inferno colormap
5. **Verify**: Intensity badge still appears during alerts
6. **Verify**: No visual regressions in other dashboard elements
