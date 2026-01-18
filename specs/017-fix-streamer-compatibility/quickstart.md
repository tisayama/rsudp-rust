# Quickstart: Testing Streamer Compatibility

## Prerequisites
- `rsudp` (Python) installed and configured to listen on port 8888.
- `rsudp-rust` built (`cargo build`).

## Test Procedure

1. **Start rsudp (Receiver)**
   ```bash
   # In a separate terminal or Python environment where rsudp is installed
   rs-client -s
   # Ensure it is listening on UDP port (default 8888 or configure as needed)
   ```

2. **Run Streamer (Sender)**
   ```bash
   # From rustrsudp_speckit/rsudp-rust directory
   ./target/debug/streamer --file ../references/mseed/fdsnws.mseed --addr 127.0.0.1:8888
   ```

3. **Verification**
   - Check `rsudp` logs. It should **NOT** show `ValueError: invalid literal for int()`.
   - `rsudp` should plot the waveform data.

## Expected Packet String
Debug output from `streamer` should look like:
`Sending packet: {'EHZ', 1678886400.1234, 100, 102, ...}`
