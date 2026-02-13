# Quickstart: WebUI Scroll Sync & Bandpass Filter

**Feature**: 038-webui-scroll-bandpass
**Date**: 2026-02-10

## Prerequisites

- Docker and Docker Compose installed
- `rsudp.toml` configured with valid Raspberry Shake station
- At least 3 active channels (e.g., EHZ, ENZ, ENE, ENN)

## Test Scenarios

### Scenario 1: Right-Aligned Backfill Display

**Setup**: Restart the application to trigger fresh backfill.

```bash
docker compose down && docker compose up -d
```

**Steps**:
1. Open WebUI immediately after startup (within 5 seconds)
2. Observe the waveform plots during backfill loading

**Expected**: Waveform data appears at the RIGHT edge of each plot. The left side of the plot is empty (shows background color). As more data arrives, the waveform extends leftward until the full Time Window is covered.

**Verification**: Take a screenshot during the first 10 seconds — data should be right-aligned, not left-aligned.

---

### Scenario 2: Synchronized Channel Scrolling

**Setup**: Application running with 3+ channels receiving data.

**Steps**:
1. Wait until all channels have data filling the Time Window
2. Observe the HH:MM:SS time labels across all channels
3. Watch the scrolling behavior for 30+ seconds

**Expected**: All channels show identical time labels at the same horizontal positions. The scrolling motion is synchronized — no channel appears to scroll ahead or behind others.

**Verification**: The rightmost time label on all channels should be the same time (within 1 second).

---

### Scenario 3: Waveform/Spectrogram Alignment

**Setup**: Application running with spectrogram enabled (default).

**Steps**:
1. Wait for an alert trigger event (or trigger one manually if possible)
2. Observe the dashed alert marker line on both waveform and spectrogram

**Expected**: The alert trigger line appears at the exact same horizontal position in both the waveform and spectrogram for the same channel. No visible horizontal offset.

**Verification**: Take a screenshot during an alert — the vertical marker lines should be pixel-aligned between waveform and spectrogram.

---

### Scenario 4: Bandpass Label Display

**Setup**: Ensure `rsudp.toml` has `filter_waveform = true` with `filter_highpass = 0.7` and `filter_lowpass = 2.0`.

**Steps**:
1. Open WebUI and observe the waveform plot
2. Look at the lower-left corner of the waveform data area

**Expected**: A label "Bandpass (0.7 - 2.0 Hz)" appears in the lower-left corner of each waveform plot.

**Verification**: Label text matches the configured values in `rsudp.toml`.

---

### Scenario 5: Range Label Display

**Setup**: Application running with spectrogram enabled.

**Steps**:
1. Open WebUI and observe the spectrogram plot
2. Look at the lower-left corner of the spectrogram data area
3. Change spectrogram frequency range in Control Panel (if available)

**Expected**: A label "Range (0 - 50 Hz)" (or matching configured values) appears in the lower-left corner of each spectrogram plot. If range is changed, label updates.

**Verification**: Label values match `spectrogram_freq_min` and `spectrogram_freq_max` settings.

---

### Scenario 6: Bandpass Label Hidden When Disabled

**Setup**: Set `filter_waveform = false` in `rsudp.toml` and restart.

**Steps**:
1. Open WebUI and observe the waveform plot lower-left corner

**Expected**: No "Bandpass" label is visible.

**Verification**: The lower-left corner of the waveform shows no filter label.

---

### Scenario 7: Backfill-to-Live Transition

**Setup**: Restart the application.

**Steps**:
1. Open WebUI immediately after startup
2. Watch continuously for 2+ minutes (until data exceeds Time Window)

**Expected**: The initial right-aligned partial waveform smoothly transitions to full-window scrolling. No visual jump, flicker, or discontinuity at the transition point.

**Verification**: Record screen or watch carefully — the transition should be imperceptible.

---

## Quick Verification Checklist

- [ ] S1: Waveform right-aligned during backfill
- [ ] S2: All channels scroll in sync
- [ ] S3: Waveform/spectrogram markers aligned
- [ ] S4: "Bandpass (X - Y Hz)" label visible when filter enabled
- [ ] S5: "Range (X - Y Hz)" label visible on spectrogram
- [ ] S6: Bandpass label hidden when filter disabled
- [ ] S7: Smooth backfill-to-live transition
