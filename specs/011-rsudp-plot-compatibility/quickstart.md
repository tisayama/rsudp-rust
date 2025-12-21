# Quickstart: rsudp Plot Compatibility

## Overview
This feature enhances the alert images to look like standard `rsudp` plots, including a spectrogram.

## Verification Steps

### 1. Trigger an Alert
1. Start `rsudp-rust`: `cargo run --bin rsudp-rust`
2. Run the streamer: `cargo run --bin streamer -- --file ../references/mseed/fdsnws.mseed --speed 10`
3. Wait for the alert to trigger and reset (watch the logs for "Reset" and "Snapshot").

### 2. Check the Image
1. Open the generated image in `rsudp-rust/alerts/`.
2. **Visual Inspection Criteria**:
   - **Layout**: Top panel is Waveform, Bottom panel is Spectrogram.
   - **Waveform**: Blue line on white/grey grid. X-axis is time (UTC or relative).
   - **Spectrogram**: Heatmap (Viridis/Magma style). Y-axis is Frequency (Hz), X-axis aligned with Waveform.
   - **Metadata**: Title includes station name, channel, and start time.

### 3. Verify Spectrogram Accuracy (Optional)
1. Use a synthetic sine wave input (if test mode available) to verify the spectrogram shows a horizontal line at the correct frequency.
