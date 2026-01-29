# Research Findings: Daemonization and System Installation

**Feature**: 029-daemonize-rsudp
**Status**: Research Complete
**Date**: 2026-01-29

## 1. Decisions

### Decision 1: Configuration Loading
- **Decision**: Use the existing `--config` (`-C`) command-line argument to pass the path `/etc/rsudp/rsudp.toml` to the binary in the systemd service file.
- **Rationale**: The application already supports this argument via `clap`. This is standard practice for system services and avoids reliance on environment variables or hidden default paths that might differ between users.
- **Alternatives Considered**: 
  - Environment variables (less visible in `systemctl status`).
  - Hardcoded path (inflexible).

### Decision 2: User Privileges and Groups
- **Decision**: Create a system user `rsudp` with primary group `rsudp`. No supplementary groups like `dialout` or `input` are needed.
- **Rationale**: Code analysis confirms the application is purely network-based (UDP) and does not interact with serial/USB hardware directly.
- **Note**: If audio alert features (local playback) are added later, the `audio` group may be required, but it is not needed for the current scope.

### Decision 3: Systemd Security Hardening
- **Decision**: Enable the following directives in `rsudp.service`:
  - `NoNewPrivileges=true`
  - `PrivateTmp=true`
  - `ProtectSystem=full` (or `strict` with explicit `ReadWritePaths`)
  - `ProtectHome=true`
- **Rationale**: These are standard hardening measures for network services that do not require root access. They limit the blast radius in case of compromise.
- **Constraint**: The application writes data (plots, screenshots) to an output directory. `ReadWritePaths` must be configured for `/var/lib/rsudp` (or similar) to allow this.

### Decision 4: Logging Strategy
- **Decision**: Rely on `tracing-subscriber`'s default behavior (stdout/stderr) which is automatically captured by `journald`.
- **Rationale**: This is the modern standard for systemd services. It avoids complex log rotation configuration within the app itself. Logs can be viewed via `journalctl -u rsudp`.

## 2. Open Questions Resolved

- **Q**: Does the binary support `--config`? 
  - **A**: Yes, confirmed in `src/main.rs`.
- **Q**: Are hardware groups needed?
  - **A**: No, purely UDP-based.
- **Q**: Does existing logging conflict?
  - **A**: No, it uses `tracing` to stdout, which is perfect.

## 3. Recommended Install Layout

| Component | Source | Dest | Ownership |
| :--- | :--- | :--- | :--- |
| Binary | `target/release/rsudp-rust` | `/usr/local/bin/rsudp-rust` | root:root (755) |
| Config | `rsudp_settings.toml` (template) | `/etc/rsudp/rsudp.toml` | root:rsudp (640) |
| Service | `rsudp.service` | `/etc/systemd/system/rsudp.service` | root:root (644) |
| Data Dir | (mkdir) | `/var/lib/rsudp` | rsudp:rsudp (750) |

**Note**: The Data Dir `/var/lib/rsudp` is critical for saving plots/alerts. The service must be configured to run with `WorkingDirectory=/var/lib/rsudp` or configured to write there.
