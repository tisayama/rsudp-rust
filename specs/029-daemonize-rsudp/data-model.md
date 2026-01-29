# Data Model

**Feature**: Daemonization and System Installation Support
**Status**: Draft

## 1. File Artifacts

The "data model" for this infrastructure feature consists of the configuration files and directory structures created on the host system.

### 1.1 System Configuration

| Entity | Path | Permissions | Description |
| :--- | :--- | :--- | :--- |
| **Service User** | `rsudp` (uid/gid) | N/A | Dedicated system user for running the daemon. |
| **Service File** | `/etc/systemd/system/rsudp.service` | 644 root:root | Systemd unit definition. |
| **Config File** | `/etc/rsudp/rsudp.toml` | 640 root:rsudp | Main application configuration. |
| **Executable** | `/usr/local/bin/rsudp-rust` | 755 root:root | Compiled binary. |
| **Data Directory** | `/var/lib/rsudp` | 750 rsudp:rsudp | Working directory for logs, plots, and state. |

## 2. Service Definition (rsudp.service)

This defines the data structure of the systemd unit file to be generated.

```ini
[Unit]
Description=rsudp-rust Seismic Monitoring Service
After=network.target

[Service]
Type=simple
User=rsudp
Group=rsudp
# Application Data
WorkingDirectory=/var/lib/rsudp
ExecStart=/usr/local/bin/rsudp-rust --config /etc/rsudp/rsudp.toml

# Reliability
Restart=on-failure
StartLimitIntervalSec=300
StartLimitBurst=5

# Hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=full
ProtectHome=true
ReadWritePaths=/var/lib/rsudp

[Install]
WantedBy=multi-user.target
```

## 3. Makefile Targets

The `Makefile` serves as the procedural model for installation.

- **build**: Compiles the Rust binary.
- **install**:
  1. Checks for/creates `rsudp` user.
  2. Copies binary to `/usr/local/bin`.
  3. Creates config dir `/etc/rsudp` and installs default config (if missing).
  4. Creates data dir `/var/lib/rsudp` with permissions.
  5. Installs `rsudp.service`.
  6. Reloads systemd daemon.
- **uninstall**:
  1. Stops/disables service.
  2. Removes binary and service file.
  3. (Optional) Leaves data/config to preserve user data.
- **clean**: Runs `cargo clean`.
