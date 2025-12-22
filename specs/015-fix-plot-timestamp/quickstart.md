# Quickstart: rsudp Plot Timestamp & Grid Fix

## Overview
This feature improves the aesthetic and functional compatibility of the alert plots by introducing UTC timestamps on the X-axis and removing unnecessary grid lines.

## Verification Steps

### 1. Manual Generation Test
Run the comparison test script to see the new formatting:
```bash
cd rsudp-rust
cargo test --test test_plot_comparison -- --nocapture
```
Open `rust_comparison_badge.png` (or the specific verification file) and check:
- **X-axis**: Does it show `HH:MM:SS` (e.g., `09:01:30`) instead of `10`, `20`, `30`?
- **Waveform Grid**: Is the background of the waveform plot solid dark without grey lines?
- **Alignment**: Does the time on the spectrogram align with the waveform peaks above it?

### 2. Layout Check
- Verify that the X-axis labels only appear at the very bottom of the image.
- Verify that the top waveform plots do not have redundant X-axis labels.
