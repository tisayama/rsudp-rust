# rsudp-speckit

This repository contains the Rust implementation of the `rsudp` seismic monitoring system and its associated Web UI.

## Project Structure

- `rsudp-rust/`: The core seismic data processing backend written in Rust.
- `webui/`: A Next.js-based web interface for real-time monitoring and alert visualization.
- `scripts/`: Utility scripts for testing and environment setup.
- `specs/`: Engineering specifications and implementation plans.

## Docker Usage (Recommended)

The easiest way to run the entire stack is using Docker Compose.

### Prerequisites
- Docker Engine (24.0+)
- Docker Compose (2.0+)

### Quick Start
1. **Prepare Output Directory**:
   ```bash
   mkdir output
   chmod 777 output # Or set strict permissions for uid 65532 (backend user)
   ```
2. **Configuration**:
   Ensure `rsudp.toml` exists in the root directory. You can use a template or create your own.
3. **Run**:
   ```bash
   docker compose up -d
   ```
4. **Access**:
   - WebUI: [http://localhost:3000](http://localhost:3000)
   - UDP Data Input: `localhost:8888`

### Troubleshooting
If the backend fails to write plots or logs, ensure the `output/` directory on the host is writable by UID `65532` (the non-root user in the distroless container).

## Development

For manual installation and build instructions, please refer to the README files in the respective sub-directories:
- [rsudp-rust/README.md](rsudp-rust/README.md)
- [webui/README.md](webui/README.md)

## License
Refer to the individual components for licensing information.
