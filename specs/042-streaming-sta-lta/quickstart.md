# Quickstart: Streaming STA/LTA Trigger Calculation

**Feature**: 042-streaming-sta-lta | **Date**: 2026-03-02

## Prerequisites

- Rust 1.7x (stable)
- Python 3.x + `obspy` (for verification only)
- Test data: `references/mseed/shindo0.mseed`

## Build & Test

```bash
# Build
cargo build --release --manifest-path rsudp-rust/Cargo.toml

# Run unit/integration tests
cargo test --manifest-path rsudp-rust/Cargo.toml

# Verify against Python reference
python3 verify_stalta.py
```

## Verification

### Python Reference (expected baseline)
```bash
python3 verify_stalta.py
# Expected: ALARM→RESET = 72 seconds, max ratio ≈ 4.5
```

### Rust Streaming Verification
```bash
python3 verify_rust_stalta.py
# Before fix: ALARM→RESET = 159.5 seconds, max ratio ≈ 3.9
# After fix:  ALARM→RESET ≈ 72 seconds (±5s), max ratio ≈ 4.5
```

### E2E Test (streamer + rsudp-rust)
```bash
# Terminal 1: Start rsudp-rust
rsudp-rust/target/release/rsudp-rust --config test_capture.toml

# Terminal 2: Stream test data
rsudp-rust/target/release/streamer --file references/mseed/shindo0.mseed --port 10000

# Verify ALARM→RESET timing in rsudp-rust output
```

## Key Files

| File | Role |
|------|------|
| `rsudp-rust/src/trigger.rs` | Primary change: streaming STA/LTA |
| `rsudp-rust/tests/integration_alert.rs` | Integration test update |
| `verify_stalta.py` | Python reference (ObsPy recursive_sta_lta) |
| `verify_rust_stalta.py` | Rust behavior simulation (pre/post fix) |
| `references/mseed/shindo0.mseed` | Test data (JMA shindo 0 earthquake) |
