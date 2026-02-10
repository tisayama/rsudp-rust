# Quickstart: RSAM Calculation and UDP Forwarding

**Feature**: 035-rsam-calculation
**Date**: 2026-02-10

## Configuration

Add or modify the `[rsam]` section in `rsudp.toml`:

```toml
[rsam]
enabled = true
quiet = false
fwaddr = "192.168.1.100"
fwport = 8887
fwformat = "LITE"
channel = "HZ"
interval = 10
deconvolve = false
units = "VEL"
```

### Configuration Fields

| Field | Description | Default |
|-------|-------------|---------|
| `enabled` | Enable RSAM module | `false` |
| `quiet` | Suppress periodic log output | `true` |
| `fwaddr` | UDP destination IP address | `"192.168.1.254"` |
| `fwport` | UDP destination port | `8887` |
| `fwformat` | Output format: `LITE`, `JSON`, or `CSV` | `"LITE"` |
| `channel` | Channel to monitor (suffix match) | `"HZ"` |
| `interval` | Calculation interval in seconds | `10` |
| `deconvolve` | Enable sensitivity conversion | `false` |
| `units` | Unit mode: `VEL`, `ACC`, `DISP`, `GRAV`, `CHAN` | `"VEL"` |

## Verification

### 1. Check Startup Log

When RSAM is enabled, the startup log should show:

```
RSAM: channel=HZ, interval=10s, format=LITE, destination=192.168.1.100:8887, deconvolve=false
```

### 2. Monitor RSAM Output (quiet=false)

With `quiet = false`, RSAM values are logged at each interval:

```
RSAM [EHZ]: mean=523.45, median=510.2, min=10.5, max=1050.3
```

### 3. Listen for UDP Packets

Use a simple listener to verify UDP output:

```bash
# Using netcat
nc -u -l 8887

# Expected LITE format output:
# stn:R6E01|ch:EHZ|mean:523.45|med:510.2|min:10.5|max:1050.3
```

### 4. Test with Streamer

```bash
# Terminal 1: Start UDP listener
python3 -c "
import socket
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.bind(('0.0.0.0', 8887))
while True:
    data, addr = sock.recvfrom(65535)
    print(data.decode())
"

# Terminal 2: Start rsudp-rust with RSAM enabled
cargo run --bin rsudp-rust -- -C rsudp.toml

# Terminal 3: Start streamer
cargo run --bin streamer -- --file references/mseed/ehz_fdsnws.mseed --speed 10
```

## Running Tests

```bash
# Run all RSAM tests
cargo test --test test_rsam

# Run specific test
cargo test --test test_rsam test_rsam_lite_format

# Run with output
cargo test --test test_rsam -- --nocapture
```

## Output Format Examples

### LITE (default)

```
stn:R6E01|ch:EHZ|mean:523.45|med:510.2|min:10.5|max:1050.3
```

### JSON

```json
{"station":"R6E01","channel":"EHZ","mean":523.45,"median":510.2,"min":10.5,"max":1050.3}
```

### CSV

```
R6E01,EHZ,523.45,510.2,10.5,1050.3
```
