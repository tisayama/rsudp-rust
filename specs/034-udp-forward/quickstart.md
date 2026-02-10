# Quickstart: UDP Data Forwarding (034-udp-forward)

**Date**: 2026-02-10

## Prerequisites

- Rust toolchain (1.7x stable)
- Existing `rsudp-rust` project builds successfully (`cargo build`)
- `rsudp.toml` configuration file

## Configuration

Add or update the `[forward]` section in `rsudp.toml`:

```toml
[forward]
enabled = true
address = ["192.168.1.100", "192.168.1.101"]
port = [8888, 9999]
channels = ["all"]       # or ["EHZ", "HZ"] for specific channels
fwd_data = true          # forward raw seismic data packets
fwd_alarms = false       # forward ALARM/RESET messages
```

**Rules**:
- `address` and `port` must have the same number of entries
- `channels = ["all"]` forwards all channels; specific entries use case-insensitive suffix matching (e.g., "HZ" matches "EHZ", "SHZ")
- Set `enabled = false` to disable forwarding entirely

## Verification

1. Start the application:
   ```bash
   cargo run --bin rsudp-rust -- --config rsudp.toml
   ```

2. Check startup logs for forwarding confirmation:
   ```
   INFO Forward: 2 destinations configured [192.168.1.100:8888, 192.168.1.101:9999]
   INFO Forward: channels=all, fwd_data=true, fwd_alarms=false
   ```

3. After data starts flowing, check periodic stats (every 60 seconds):
   ```
   INFO Forward #0 (192.168.1.100:8888): sent=1523, dropped=0, errors=0
   INFO Forward #1 (192.168.1.101:9999): sent=1523, dropped=0, errors=0
   ```

4. If a destination is unreachable, logs will show:
   ```
   WARN Forward #1 (192.168.1.101:9999): sent=0, dropped=0, errors=47
   ```

## Running Tests

```bash
# Unit tests for forward module
cargo test forward

# Integration test (starts local UDP listener)
cargo test test_forward_e2e
```

## Files Changed

| File                  | Change                                    |
| --------------------- | ----------------------------------------- |
| `src/forward.rs`      | New: ForwardManager, forwarding tasks     |
| `src/pipeline.rs`     | Modified: integrate forward_data/alarm calls |
| `src/main.rs`         | Modified: initialize ForwardManager       |
| `src/lib.rs`          | Modified: add `pub mod forward;`          |
| `tests/test_forward.rs` | New: unit + integration tests          |
