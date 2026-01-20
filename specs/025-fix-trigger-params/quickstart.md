# Quickstart: Verifying Trigger Parameter Fix

## Verification Steps

1. **Configure**:
   Update `settings.toml` with the user-provided block:
   ```toml
   [alert]
   enabled = true
   channel = "HZ"
   sta = 6.0
   lta = 30.0
   duration = 0.0
   threshold = 1.1
   reset = 0.5
   highpass = 0.1
   lowpass = 2.0
   deconvolve = false
   units = "VEL"
   on_plot = true
   ```

2. **Run**:
   ```bash
   # Enable debug logging for trigger logic
   RUST_LOG=rsudp_rust::trigger=debug ./target/release/rsudp-rust
   ```

3. **Check Logs**:
   Look for startup logs confirming filter design:
   ```text
   INFO rsudp_rust::trigger: TriggerManager initialized.
   INFO rsudp_rust::trigger: Filter: Butterworth Order=4, Band=[0.1, 2.0] Hz, Rate=100.0 Hz
   INFO rsudp_rust::trigger: Calculated Coefficients: ...
   ```

4. **Stream Data**:
   ```bash
   ./target/release/streamer --file references/mseed/fdsnws.mseed --addr 127.0.0.1:8888 --speed 1.0
   ```

5. **Compare**:
   The logs should show trigger times matching `rsudp` output.
