# Quickstart: Verifying Smooth Streaming

## Prerequisites
- `rsudp` (Python) running.
- `rsudp-rust` built.

## Run Verification

1. **Start rsudp**:
   ```bash
   rs-client -s
   ```

2. **Start Streamer**:
   ```bash
   # Use default 25 samples/packet
   ./target/debug/streamer --file ../references/mseed/fdsnws.mseed --addr 127.0.0.1:8888
   ```

3. **Check Custom Granularity**:
   ```bash
   # Test with 50 samples/packet (0.5s updates)
   ./target/debug/streamer --file ../references/mseed/fdsnws.mseed --addr 127.0.0.1:8888 --samples-per-packet 50
   ```

## Expected Behavior
- **Default**: The plot in `rsudp` should update approx. 4 times per second. Movement should be smooth.
- **Large Packets**: If you see jerky movement (updating once every few seconds), the fix is not working.
- **Banding**: There should be no vertical "stripes" or gaps in the spectrogram/waveform.
