# Quickstart: Docker Deployment

This guide explains how to run the `rsudp-rust` system using Docker and Docker Compose.

## Prerequisites

- **Docker Engine** (24.0+)
- **Docker Compose** (2.0+)
- **Git**

## Installation & Running

1.  **Clone the Repository**
    ```bash
    git clone https://github.com/your-org/rsudp-rust.git
    cd rsudp-rust
    ```

2.  **Prepare Configuration**
    Create a configuration file from the template or use the default.
    ```bash
    # Create a minimal config if you don't have one
    touch rsudp.toml
    # OR copy an existing one
    cp config.example.toml rsudp.toml
    ```

3.  **Prepare Output Directory**
    Create a directory for persistent data (plots, logs).
    ```bash
    mkdir output
    # Ensure the container (UID 65532) can write to it
    sudo chown 65532:65532 output
    # OR broadly allow write access (easier for dev)
    chmod 777 output
    ```

4.  **Start Services**
    ```bash
    docker compose up -d
    ```

5.  **Verify**
    - **Web Interface**: Open [http://localhost:3000](http://localhost:3000).
    - **Backend Logs**:
      ```bash
      docker compose logs -f rsudp-rust
      ```

## Usage

### Sending Test Data
You can send UDP packets to `localhost:8888`.
```bash
# Example using netcat (if supported by your test data format)
cat data.mseed | nc -u localhost 8888
```

### Stopping
```bash
docker compose down
```

## Troubleshooting

### Permission Denied (Output)
If you see errors about writing to `/var/lib/rsudp` or `output/`, ensure the host directory permissions are correct (Step 3). The backend runs as a non-root user (UID 65532).

### Port Conflicts
If port 8888 or 3000 is in use, modify `docker-compose.yml` or set environment variables:
```bash
RSUDP_PORT=9999 WEB_PORT=3001 docker compose up -d
```
