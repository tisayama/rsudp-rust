# Quickstart: Intensity Display on Alert Plots

## Overview
This feature adds a large, colored "Shindo" (Seismic Intensity) box to the top-right of alert images, providing instant scale recognition using the official JMA color scheme.

## Verification

### 1. Manual Generation Test
Run the comparison test script to see the new annotation in action:
```bash
cd rsudp-rust
cargo test --test test_plot_comparison -- --nocapture
```
Open `rust_comparison_300s.png` and verify the top-right corner has a colored box.

### 2. Live Simulation
1. Start the server: `cargo run --bin rsudp-rust`
2. Run the streamer with a high-intensity file: 
   `cargo run --bin streamer -- --file ../references/mseed/20251208_tsukuba_fdsnws.mseed --speed 10`
3. Wait for the alert reset and open the image in the WebUI history or `alerts/` folder.

## Visual Acceptance Criteria
- **Font**: Must be Noto Sans Japanese (Bold/Heavy recommended for visibility).
- **Text**: "震度 X" or "震度 X弱/強".
- **Color**:
  - Shindo 3 -> Green
  - Shindo 4 -> Yellow
  - Shindo 5- -> Orange
  - ...etc.
- **Contrast**: Text must be white and perfectly legible against the background color.
