# Feature Specification: Daemonization and System Installation Support

**Feature**: Daemonization and System Installation Support
**Status**: Draft
**Last Updated**: 2026-01-29

## Clarifications
### Session 2026-01-29
- Q: Service User Context → A: Use a dedicated `rsudp` system user.
- Q: Config Path Handling → A: Use CLI argument (--config path) passed in ExecStart.
- Q: Restart Policy Details → A: Standard burst limits (e.g., 5 failures in 5 min).

## 1. Context & Goals

### 1.1 Problem Statement
Currently, the `rsudp-rust` application runs as a foreground process. Users on Linux (Ubuntu/Debian) need a way to run it as a background service (daemon) that starts automatically on boot and restarts if it crashes. Additionally, the installation process involves manual build steps and copying files, which lacks a standardized method like `make install`.

### 1.2 Goals
- Enable `rsudp-rust` to run as a systemd service on Ubuntu/Debian.
- Provide a standard `Makefile` to simplify building and installing the binary and service files.
- Ensure the service automatically restarts upon failure.
- Document all necessary prerequisites and installation steps clearly in `README.md`.

### 1.3 Out of Scope
- Support for non-systemd init systems (e.g., SysVinit, OpenRC).
- Packaging as `.deb` or `.rpm` (standard `make install` is sufficient for this scope).
- Windows or macOS service support.

## 2. User Scenarios

### 2.1 Standard Installation
**Actor**: System Administrator / End User
**Flow**:
1. User clones the repository on a fresh Ubuntu/Debian server.
2. User reads `README.md` to install dependencies (e.g., build-essential, cargo).
3. User runs `make` to build the release binary.
4. User runs `sudo make install` to install the binary, service file, and creates the dedicated system user.
5. User enables and starts the service via `systemctl`.
**Outcome**: The application runs in the background as the `rsudp` user and starts on boot.

### 2.2 Auto-Restart on Crash
**Actor**: System Administrator
**Flow**:
1. The running `rsudp-rust` service encounters a critical error and panics.
2. Systemd detects the process exit.
3. Systemd automatically restarts the service after a short delay (e.g., 5s).
4. If the service fails more than 5 times within 5 minutes, systemd stops trying and marks the service as failed.
**Outcome**: Downtime is minimized for transient errors, while persistent failures are caught.

## 3. Functional Requirements

### 3.1 Build and Install Automation
- **FR1**: Provide a `Makefile` in the project root.
- **FR2**: `make` (or `make build`) must compile the Rust project in release mode.
- **FR3**: `make install` must:
    - Create a dedicated `rsudp` system user if it does not exist (or instruct user to do so).
    - Copy the compiled binary to a standard system path (e.g., `/usr/local/bin`).
    - Copy the systemd unit file to the system unit directory (e.g., `/etc/systemd/system`).
    - Copy default configuration files (if any) to a standard config location (e.g., `/etc/rsudp/`) and ensure `rsudp` user ownership.
- **FR4**: `make uninstall` must remove installed binaries and service files.

### 3.2 Systemd Integration
- **FR5**: Provide a generic `rsudp.service` unit file.
- **FR5.1**: Service must run as User `rsudp` and Group `rsudp` (or equivalent).
- **FR5.2**: Service command (`ExecStart`) must explicitly pass the configuration path via `--config` CLI argument.
- **FR6**: The service must be configured to restart automatically on failure (`Restart=on-failure`).
- **FR6.1**: Implement burst limit prevention using `StartLimitIntervalSec=300` and `StartLimitBurst=5` (or equivalent systemd defaults).
- **FR7**: The service must handle logging via standard output/error (journald).

### 3.3 Documentation
- **FR8**: `README.md` must list all system package dependencies required for building (e.g., Rust toolchain, C compiler).
- **FR9**: `README.md` must provide step-by-step commands for `make install` and `systemctl` operations.

## 4. Technical Assumptions
- Target OS: Ubuntu 22.04 LTS or Debian 11/12 (or newer).
- User has `sudo` privileges.
- Rust toolchain is installed via `rustup` or system packages.

## 5. Success Criteria

- **SC1**: A user on a clean Ubuntu/Debian environment can build and install the application using only `make` and `sudo make install`.
- **SC2**: The application successfully starts as a systemd service via `systemctl start rsudp`.
- **SC3**: Sending `kill -9` to the main process results in the service restarting automatically within 10 seconds.
- **SC4**: `make uninstall` cleanly removes the installed artifacts.
