# Tasks: UDP Packet Reception & Ingestion

**Input**: Design documents from `/specs/002-udp-receiver/`
**Prerequisites**: plan.md, spec.md, data-model.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Dependencies & Structure)

**Purpose**: Add required dependencies and establish module structure.

- [X] T001 Add `tokio` (with full features), `clap` (with derive), `tracing`, and `tracing-subscriber` to `rsudp-rust/Cargo.toml`.
- [X] T002 [P] Create `rsudp-rust/src/settings.rs` and define the `Settings` struct using `clap` to accept a `--port` argument (default 8888).
- [X] T003 [P] Create `rsudp-rust/src/receiver.rs` and add `pub mod receiver;` to `rsudp-rust/src/main.rs` to expose the module.

---

## Phase 2: Foundational (Data Model & Logging)

**Purpose**: Define core data structures and initialize observability.

- [X] T004 [P] Define the `Packet` struct in `rsudp-rust/src/receiver.rs` with fields: `source`, `data`, and `received_at`, deriving `Debug` and `Clone`.
- [X] T005 [P] Initialize `tracing_subscriber` in `rsudp-rust/src/main.rs` to enable formatted logging output to stdout.
- [X] T006 Update `rsudp-rust/src/main.rs` to parse command-line arguments using the `Settings` struct from T002.

---

## Phase 3: User Story 1 - Receive UDP Packets (Priority: P1)

**Goal**: The application binds to a UDP port, receives packets, and logs them/pushes them to a channel.

**Independent Test**: Running `cargo run` and sending a packet via `nc -u 127.0.0.1 8888` results in a log message showing the receipt.

### Tests for User Story 1

- [X] T007 [US1] Create an integration test in `rsudp-rust/tests/integration_receiver.rs` that spawns the receiver and sends a UDP packet to localhost, asserting that a packet is received (mocking the channel or checking logs if possible, but primarily testing binding and basic IO).

### Implementation for User Story 1

- [X] T008 [US1] Implement a function `start_receiver` in `rsudp-rust/src/receiver.rs` that takes a port number and a `mpsc::Sender<Packet>`.
- [X] T009 [US1] Inside `start_receiver`, implement the logic to bind a `tokio::net::UdpSocket` to the specified port.
- [X] T010 [US1] Implement the reception loop in `start_receiver`: await `socket.recv_from`, create a `Packet` instance, and log the event using `tracing::info!`.
- [X] T011 [US1] Send the created `Packet` through the provided `mpsc::Sender`.
- [X] T012 [US1] Update `rsudp-rust/src/main.rs` to create an `mpsc::channel`, spawn the `start_receiver` task, and implemented a simple consumer loop to log received packets (proving ingestion).

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Improve robustness and documentation.

- [X] T013 Improve error handling in `rsudp-rust/src/receiver.rs` to gracefully handle socket errors without crashing the loop (logging warnings instead).
- [X] T014 Run `cargo clippy` and fix any linting warnings in the new code.
- [X] T015 Verify the `quickstart.md` instructions work as expected with the final implementation.

---

## Dependencies & Execution Order

- **Phase 1 (Setup)**: Can start immediately.
- **Phase 2 (Foundational)**: Depends on Phase 1.
- **Phase 3 (User Story 1)**: Depends on Phase 2.
- **Phase 4 (Polish)**: Depends on Phase 3.

### Parallel Opportunities

- T002 and T003 in Phase 1 can be done in parallel.
- T004 and T005 in Phase 2 can be done in parallel.
- T007 (Test) can be written before T008-T012 (Implementation).

## Implementation Strategy

### MVP Delivery

1.  Complete Phase 1 & 2 to set up the environment and data structures.
2.  Implement the receiver logic in Phase 3.
3.  Verify with `nc` manually as per User Story 1 acceptance criteria.
4.  Run the integration test to ensure regression safety.
