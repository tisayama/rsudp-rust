# Quickstart Guide: Daemon Installation

This guide explains how to install and run `rsudp-rust` as a system service on Ubuntu/Debian.

## Prerequisites

- **OS**: Ubuntu 22.04+ or Debian 11+
- **Rust**: Installed via `rustup` (stable channel)
- **Dependencies**: `build-essential`

```bash
sudo apt update && sudo apt install -y build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

## Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/your-org/rsudp-rust.git
   cd rsudp-rust
   ```

2. **Build the binary**:
   ```bash
   make
   ```

3. **Install to system (requires sudo)**:
   This creates the `rsudp` user, installs the binary to `/usr/local/bin`, and sets up the systemd service.
   ```bash
   sudo make install
   ```

4. **Edit Configuration**:
   Edit the configuration file to match your station settings:
   ```bash
   sudo nano /etc/rsudp/rsudp.toml
   ```

## Service Management

- **Start the service**:
  ```bash
  sudo systemctl start rsudp
  ```

- **Enable auto-start on boot**:
  ```bash
  sudo systemctl enable rsudp
  ```

- **Check status**:
  ```bash
  sudo systemctl status rsudp
  ```

- **View logs**:
  ```bash
  sudo journalctl -u rsudp -f
  ```

## Uninstallation

To remove the binary and service file (configuration and data are preserved):
```bash
sudo make uninstall
```
