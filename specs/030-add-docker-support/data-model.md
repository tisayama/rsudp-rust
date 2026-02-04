# Data Model: Container Persistence

**Feature**: 030-add-docker-support

## Container Volumes

The Docker setup relies on file-system persistence for configuration and output data.

### 1. Configuration (`/etc/rsudp/rsudp.toml`)
- **Mount Type**: Bind Mount (Read-Only)
- **Source**: `./rsudp.toml` (Host)
- **Destination**: `/etc/rsudp/rsudp.toml` (Container)
- **Description**: TOML configuration file defining station parameters, alert thresholds, and processing logic.

### 2. Output Data (`/var/lib/rsudp`)
- **Mount Type**: Bind Mount (Read-Write)
- **Source**: `./output` (Host)
- **Destination**: `/var/lib/rsudp` (Container)
- **Ownership**: Owned by UID 65532 (nonroot) inside the container.
- **Contents**:
  - `plots/`: Generated waveform images (PNG).
  - `logs/`: Application log files (if configured to file logging).
  - `data/`: Raw MiniSEED archives (if enabled).

### 3. Docker Internal Volumes (Optional)
- **Name**: `nextjs_cache`
- **Description**: Persists `.next/cache` to speed up frontend rebuilds across restarts.
