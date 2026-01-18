# Quickstart: Verifying Alerts

## Prerequisites
- `rsudp-rust` built.
- `streamer` built.

## Verification Steps

1. **Start rsudp-rust**:
   ```bash
   ./target/debug/rsudp-rust --udp-port 8888 --station R6E01
   ```

2. **Start Streamer**:
   ```bash
   ./target/debug/streamer --file ../references/mseed/fdsnws.mseed --addr 127.0.0.1:8888 --speed 1.0
   ```

3. **Expected Output**:
   - `rsudp-rust` logs should show:
     ```text
     Trigger threshold X exceeded (ratio: Y). ALARM!
     ```
   - Alerts should be generated in `alerts/`.

4. **Failure Condition**:
   - Stream finishes without any "Trigger threshold exceeded" message.
