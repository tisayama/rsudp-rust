# Research Findings: Docker Support

**Feature**: 030-add-docker-support
**Date**: 2026-01-29

## 1. Rust Backend Containerization
- **Goal**: Small, secure image for `rsudp-rust`.
- **Approach**: Multi-stage build.
  - **Builder**: `rust:1.82-bookworm` (Debian 12 based). Compiles the release binary.
  - **Runtime**: `gcr.io/distroless/cc-debian12`. Contains minimal glibc runtime required for Rust binaries.
- **Security**: Create a non-root user in the builder stage (or assume distroless `nonroot` user). Distroless `cc-debian12` defines a `nonroot` user (uid 65532).
- **Caveat**: Distroless has no shell. Entrypoint must be the binary itself. Troubleshooting requires debug images or `docker run --entrypoint`.

## 2. Next.js Frontend Containerization
- **Goal**: Production build of `webui`.
- **Approach**: Multi-stage build.
  - **Builder**: `node:22-alpine`. Runs `npm run build`.
  - **Config**: Enable `output: 'standalone'` in `next.config.mjs` to reduce image size (only copies necessary files).
  - **Runtime**: `node:22-alpine`. Copies `.next/standalone`, `.next/static`, and `public`.
- **Reasoning**: Alpine is much smaller than Debian. Distroless for Node is possible but `node:alpine` is a standard, well-supported balance for Next.js apps needing basic runtime tools.

## 3. Orchestration & Networking
- **Compose**: Root `docker-compose.yml`.
- **Services**:
  - `rsudp-rust`: 
    - Ports: `8888:8888/udp` (Data In), `8081` (API/WS - internal or mapped).
    - Volumes: `./rsudp.toml:/etc/rsudp/rsudp.toml:ro`, `./output:/var/lib/rsudp`.
    - User: 65532 (distroless nonroot).
  - `webui`:
    - Ports: `3000:3000`.
    - Environment: `BACKEND_URL=http://rsudp-rust:8081`.
- **Permission Issue**: The `rsudp-rust` container running as uid 65532 will try to write to `./output` mounted from host.
  - **Mitigation**: The host directory `./output` must be writable by uid 65532, or `docker-compose` runs with user remapping.
  - **Plan**: Document that users should `mkdir output && chmod 777 output` (simple) or `chown 65532:65532 output` for strict security.

## 4. Healthchecks
- **Backend**: Rust app doesn't have `curl` in distroless.
  - **Option A**: Use `grpc_health_probe` (if grpc).
  - **Option B**: Copy a statically compiled healthcheck binary.
  - **Option C**: Rely on Docker's process monitoring (if app crashes, it restarts).
  - **Decision**: For now, relying on `restart: on-failure` and process exit is sufficient. Distroless makes internal healthchecks harder without adding tools.
- **Frontend**: `wget` is available in Alpine.
  - Command: `wget --no-verbose --tries=1 --spider http://localhost:3000 || exit 1`.

## 5. Configuration Strategy
- **File**: `rsudp.toml` mounted to container.
- **Env Vars**: Ports defined in `.env` or defaults in `docker-compose.yml`.

