# Tasks: Docker and Docker Compose Support

**Feature**: Docker and Docker Compose Support
**Branch**: `030-add-docker-support`
**Status**: DRAFT

## Phase 1: Setup

*Goal: Initialize Docker configuration files and prepare the environment.*

**Independent Test Criteria**: Dockerfiles exist in correct locations.

- [x] T001 Create `rsudp-rust/Dockerfile` with placeholder content in `rsudp-rust/Dockerfile`
- [x] T002 Create `webui/Dockerfile` with placeholder content in `webui/Dockerfile`
- [x] T003 Create `docker-compose.yml` with version definition in `docker-compose.yml`
- [x] T004 Create `.dockerignore` file in repository root in `.dockerignore`

## Phase 2: Foundational Tasks

*Goal: Implement optimized Dockerfiles for both services.*

**Independent Test Criteria**: Both images build successfully individually.

- [x] T005 [P] Implement multi-stage build for backend in `rsudp-rust/Dockerfile`
- [x] T006 [P] Implement multi-stage build for frontend in `webui/Dockerfile`
- [x] T007 Configure `next.config.mjs` for standalone output in `webui/next.config.mjs`
- [x] T008 Add `rsudp` non-root user setup (or use distroless default) in `rsudp-rust/Dockerfile`

## Phase 3: Developer Setup (User Story 1)

*Goal: Orchestrate services for local development usage.*

**Independent Test Criteria**: `docker compose up` starts both services, WebUI is accessible at localhost:3000, backend receives UDP.

- [x] T009 [US1] Define `rsudp-rust` service in `docker-compose.yml`
- [x] T010 [US1] Define `webui` service in `docker-compose.yml`
- [x] T011 [US1] Configure bridge network and internal DNS aliases in `docker-compose.yml`
- [x] T012 [P] [US1] Map host ports 8888 (UDP) and 3000 (TCP) in `docker-compose.yml`
- [x] T013 [US1] Configure environment variables for service discovery in `docker-compose.yml`

## Phase 4: Server Deployment (User Story 2)

*Goal: Ensure persistence, reliability, and production readiness.*

**Independent Test Criteria**: Data persists across container restarts; services auto-restart on failure.

- [x] T014 [US2] Add volume mount for `rsudp.toml` config in `docker-compose.yml`
- [x] T015 [US2] Add volume mount for output directory persistence in `docker-compose.yml`
- [x] T016 [US2] Configure `restart: on-failure` policy for both services in `docker-compose.yml`
- [x] T017 [P] [US2] Add `.next/cache` volume for frontend build performance in `docker-compose.yml`

## Phase 5: Polish & Cross-Cutting Concerns

*Goal: Finalize documentation and clean up.*

**Independent Test Criteria**: Documentation matches implementation.

- [x] T018 Update README with Docker instructions in `README.md`
- [x] T019 Clean up any temporary files or unused scripts in `rsudp-rust/Dockerfile`

## Dependencies

1. **Phase 1 (Setup)**: Blocks everything.
2. **Phase 2 (Foundational)**: Blocks Phase 3 & 4.
3. **Phase 3 (Developer Setup)**: Blocks manual verification of the stack.
4. **Phase 4 (Server Deployment)**: Can be done in parallel with Phase 3 testing but logically extends it.

## Parallel Execution Examples

- **Backend & Frontend Dockerfiles**: T005 and T006/T007 can be implemented simultaneously by different developers.
- **Service Definitions**: T009 and T010 can be drafted in parallel, then merged.

## Implementation Strategy

1. **MVP**: Build both images manually and verify they work.
2. **Orchestration**: Connect them via Compose for the Developer Workflow.
3. **Production**: Add volumes and restart policies for Server Deployment.
