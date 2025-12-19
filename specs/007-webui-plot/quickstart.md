# Quickstart: WebUI Plot System

## Backend (Rust)

The backend integrates with the seismic pipeline and broadcasts data via WebSockets.

1.  **Run with all dependencies**:
    ```bash
    cd rsudp-rust
    cargo run
    ```
    - The server will listen on `http://0.0.0.0:8080`.
    - WebSocket endpoint: `ws://localhost:8080/ws`.
    - API endpoints: `/api/settings`, `/api/channels`.

## Frontend (Next.js)

The frontend provides a real-time scrolling dashboard using Canvas.

1.  **Install & Start**:
    ```bash
    cd webui
    npm install
    npm run dev
    ```
    - Access the dashboard at `http://localhost:3000`.

## Features
- **Real-time Scrolling**: Waveforms are rendered at 60 FPS using HTML5 Canvas.
- **Binary Protocol**: Seismic samples are streamed as binary data to minimize latency.
- **Dynamic Configuration**: Toggle channels and adjust time windows via the sidebar.
- **Alert Indicators**: Seismic triggers (STA/LTA) are highlighted with vertical markers.
- **Auto-recovery**: Frontend automatically reconnects to the backend with exponential backoff.