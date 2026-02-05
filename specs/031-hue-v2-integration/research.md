# Research Findings: Philips Hue V2 Integration

**Feature**: 031-hue-v2-integration
**Date**: 2026-02-04

## 1. Hue API v2 (CLIP v2)
- **Protocol**: HTTPS over local network.
- **Auth**: `hue-application-key` header.
- **Certificate**: Self-signed certificate issued by the Bridge.
  - **Decision**: Rust `reqwest` client must be configured to disable certificate verification (`danger_accept_invalid_certs(true)`) for local discovery, as distributing the CA cert is complex for end-users.
- **Resource ID**: API v2 uses UUIDv4 resource IDs for Lights, Rooms, and Zones.
  - **Impact**: Configuration must store these UUIDs, not integer IDs (API v1).

## 2. mDNS Discovery in Rust
- **Library**: `mdns-sd` is a pure Rust implementation.
- **Service Type**: `_hue._tcp.local`.
- **Strategy**: 
  - **Continuous**: Run a background task that listens for mDNS announcements to update the IP cache.
  - **One-shot**: For CLI, run a discovery for X seconds and list results.

## 3. Alert Logic Implementation
- **Trigger (Yellow Pulse)**:
  - Endpoint: `PUT /clip/v2/resource/light/{id}`
  - Payload: `{"signaling": {"signal": "breathe", "duration": 1000, "color": [{"xy": {"x": 0.43, "y": 0.50}}]}}` (Approx Yellow) - *Note: API v2 `alert` effect might be simpler ("breathe" is standard).*
  - **Correction**: API v2 `alert` object supports `{ "action": "breathe" }`. Color must be set separately or part of the same `PUT` if supported.
  - **Action**: Check if light is `on`. If so, update color and trigger alert.
- **Reset (Color Pulse)**:
  - Color mapping: Convert RGB (JMA) to XY color space.
  - **Algorithm**: Simple RGB to XY conversion formula is needed.
  - **Duration**: Repeat the pulse or set a long duration if API supports it. Standard `breathe` is one cycle. `identify` is also an option but usually white.
  - **Alternative**: Use `dynamics` or manually loop the pulse commands for 20 seconds.
  - **Decision**: Send color command + `breathe` action loop for 20s.

## 4. CLI Tool Structure
- **Crate**: `clap` with subcommands.
- **Commands**:
  - `discover`: List bridges.
  - `pair`: Trigger link button flow.
  - `list`: Show lights/rooms with IDs.

## 5. Security & Persistence
- **Config**: Plain text `rsudp.toml` is acceptable per spec.
- **Keys**: `app_key` is sensitive but local-only.

## 6. RGB to XY Conversion
- Hue lights use the CIE 1931 color space.
- **Formula**: Standard Gamma correction -> RGB to XYZ -> XYZ to xy.
- **Rust Crate**: `palette` or custom helper function.
- **Decision**: Implement a lightweight helper function to avoid heavy dependencies.
