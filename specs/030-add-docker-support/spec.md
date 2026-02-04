# Specification: Docker and Docker Compose Support

**Feature**: Docker and Docker Compose Support
**Status**: DRAFT
**Feature Branch**: `030-add-docker-support`

## 1. Executive Summary



This feature introduces a production-ready containerization setup for the application using Docker and Docker Compose. It separates the Rust backend (`rsudp-rust`) and Next.js frontend (`webui`) into distinct services, orchestrated by Docker Compose. This ensures consistent environments for development and deployment, simplifies the installation process, and follows containerization best practices.



## 2. Clarifications







### Session 2026-01-29



- Q: Where should the Dockerfiles be located? → A: Co-located in `rsudp-rust/` and `webui/` directories.



- Q: How should services communicate? → A: Via Docker service names (DNS), e.g., `http://rsudp-rust:8081`.



- Q: Which base image should be used for the final runtime? → A: Distroless (`gcr.io/distroless/cc-debian12`).



- Q: Where should the `docker-compose.yml` file be located? → A: At the repository root.



- Q: What should be the restart policy for the services? → A: Automatically restart on failure (`restart: on-failure`).







## 3. User Scenarios







### 3.1. Developer Setup



**Actor**: Developer



**Scenario**: A developer wants to contribute to the project without installing Rust or Node.js toolchains locally.



**Flow**:



1.  Developer clones the repository.



2.  Developer runs `docker compose up --build`.



3.  System builds both backend and frontend images.



4.  Developer accesses the WebUI at `http://localhost:3000`.



5.  Developer sends UDP data to `localhost:8888`.







### 3.2. Server Deployment



**Actor**: System Administrator



**Scenario**: Deploying the application to a production server.



**Flow**:



1.  Admin copies the project (or fetches git repo) to the server.



2.  Admin configures settings via mapped configuration files or environment variables.



3.  Admin starts services with `docker compose up -d`.



4.  The application runs reliably in the background, restarting automatically if configured.



5.  Data and logs are persisted to host directories via volumes.







## 4. Functional Requirements







### 4.1. Containerization



1.  **Backend Image**: Create a dedicated Dockerfile co-located in `rsudp-rust/Dockerfile`.



    *   Must compile the Rust binary in a builder stage (Multi-stage build).



    *   **Runtime Image**: Use Distroless (`gcr.io/distroless/cc-debian12`) for the final stage.



    *   Must run as a non-root user where possible (security).



    *   Must expose the UDP port for seismic data.



2.  **Frontend Image**: Create a dedicated Dockerfile co-located in `webui/Dockerfile`.



    *   Must build the Next.js application.



    *   Must serve the application (e.g., via `npm start` or a lightweight server).



    *   Must expose the HTTP port.







### 4.2. Orchestration (Docker Compose)



1.  **Service Definition**: Define `rsudp-rust` and `webui` as separate services in a root-level `docker-compose.yml`.



    *   **Reliability**: Both services must be configured to automatically restart on failure (`restart: on-failure`).



2.  **Networking**:



    *   Services must share a bridge network to communicate.



    *   **Frontend-to-Backend**: The frontend must address the backend using the Docker service name (e.g., `http://rsudp-rust:8081`) instead of `localhost`.



    *   Expose UDP port (default 8888) to the host.



    *   Expose WebUI HTTP port (default 3000) to the host.



3.  **Volumes & Persistence**:



    *   Mount configuration file (`rsudp.toml`) from host to container.



    *   Mount output directory (for plots/screenshots) to host to ensure data persistence.



    *   Mount logs directory.















### 4.3. Configuration



1.  The setup must allow users to provide their own `rsudp.toml` without rebuilding images.



2.  Environment variables in `docker-compose.yml` should allow overriding key settings (like ports) if applicable.







## 5. Success Criteria







1.  **Build Success**: `docker compose build` completes without errors for both services.



2.  **Startup Success**: `docker compose up` starts all containers, and they remain in "Up" state (healthy).



3.  **Connectivity**:



    *   WebUI is accessible at `http://localhost:3000`.



    *   Backend API is reachable by Frontend.



    *   Backend receives UDP packets sent to host port 8888.



4.  **Persistence**: Files created in the output directory inside the container appear on the host system.







## 6. Assumptions & Constraints







*   **Architecture**: The user is running on x86_64 architecture (standard Intel/AMD). ARM (Raspberry Pi) support is desirable but might require multi-arch builds (out of immediate scope unless specified, but Alpine usually handles this well).



*   **Permissions**: The user has Docker and Docker Compose installed and has permission to run them.



*   **Ports**: Ports 8888 (UDP) and 3000 (TCP) are free on the host.







## 7. Security Considerations







*   Containers should not run as root if possible (especially the frontend).



*   **Base Image**: Distroless (`gcr.io/distroless/cc-debian12`) is used to minimize attack surface.



*   Secrets (if any) should be handled via environment variables, not hardcoded.




