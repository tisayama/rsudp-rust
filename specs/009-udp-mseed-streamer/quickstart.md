# Quickstart: UDP MiniSEED Streamer

## Overview
This tool simulates a Raspberry Shake sensor by streaming MiniSEED records over UDP in a time-aware manner. It decodes MiniSEED data and sends it in the string-based format expected by `rsudp` and `rsudp-rust`.

## Setup
Ensure you have the Rust toolchain installed.

```bash
cd rsudp-rust
# The streamer is part of the rsudp-rust workspace or a standalone binary
# Implementation decision: Add as a sub-command to the main binary or a separate bin.
```

## Basic Usage

### 1. Real-time Streaming
Stream a file to the default receiver port (12345) at 1x speed.
```bash
cargo run --bin streamer -- --file data.mseed --addr 127.0.0.1:12345
```

### 2. Fast Playback
Playback at 10x speed for quick verification of intensity calculations.
```bash
cargo run --bin streamer -- --file data.mseed --speed 10
```

### 3. Looping Simulation
Keep the simulation running indefinitely for stability testing.
```bash
cargo run --bin streamer -- --file data.mseed --loop
```

## Verifying Results
Point the streamer at your `rsudp-rust` instance. You should see:
1. `INFO` logs in the receiver indicating packet arrival.
2. Waveforms appearing in the WebUI.
3. Accurate timestamps in the WebUI matching the original file.
