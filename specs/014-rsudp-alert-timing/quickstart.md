# Quickstart: rsudp-style Alert Post Timing

## Overview
This feature changes the timing of alert notifications and image generation. Instead of waiting for the earthquake to end (`RESET`), the system now generates the snapshot and sends the summary after a fixed percentage of the display window has passed since the initial `TRIGGER`.

## Verification Steps

### 1. Timing Verification
1. Start `rsudp-rust`.
2. Run the `streamer` with a known earthquake file.
3. Observe the logs.
4. **Expectation**: 
   - `ALARM` log appears immediately on detection.
   - `Snapshot` and `RESET报` (Email/UI) appear after exactly `duration * 0.7` seconds (e.g., 63 seconds if duration is 90s).
   - This should happen *before* the `RESET` log if the earthquake lasts longer than 63 seconds.

### 2. Visual Alignment Verification
1. Open the generated image.
2. **Expectation**: The trigger point (start of the earthquake) should be located at approximately 70% from the right (or 30% from the left, depending on implementation) of the time axis.

## Troubleshooting
- If snapshots are still appearing only at `RESET`, ensure the `pipeline.rs` has been refactored to use the timer-based trigger.
