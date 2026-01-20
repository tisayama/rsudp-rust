# Quickstart: Verifying STA/LTA Fix

## Test with Streamer

1. **Build**:
   ```bash
   cargo build --release
   ```

2. **Run Receiver** (with high debug log for trigger):
   ```bash
   RUST_LOG=rsudp_rust::trigger=debug ./target/release/rsudp-rust --udp-port 8888 --station R6E01 --network AM
   ```

3. **Run Streamer** (100x speed for quick verification):
   ```bash
   ./target/release/streamer --file references/mseed/fdsnws.mseed --addr 127.0.0.1:8888 --speed 100.0
   ```

## Expected Behavior
- **Initial 30s (LTA)**: No logs about ratios, or "Warmup..." logs.
- **After 30s**: "Warmup complete".
- **Event**: Trigger should occur at roughly the same timestamp relative to the file start as `rsudp`.
- **False Positives**: Should be zero.
