# Tasks: Philips Hue V2 Integration

**Feature**: Philips Hue V2 Integration
**Branch**: `031-hue-v2-integration`
**Status**: DRAFT

## Phase 1: Setup

*Goal: Initialize project structure and dependencies.*

**Independent Test Criteria**: New module structure exists and project compiles with new dependencies.

- [x] T001 Add `reqwest` (with rustls), `mdns-sd`, and `serde` dependencies to `rsudp-rust/Cargo.toml`
- [x] T002 Create `rsudp-rust/src/hue` directory structure with mod.rs, client.rs, discovery.rs, and config.rs
- [x] T003 Create `rsudp-rust/src/bin/rsudp-hue.rs` with placeholder main function
- [x] T004 Define `HueConfig` struct in `rsudp-rust/src/hue/config.rs` per data model
- [x] T005 [P] Integrate `HueConfig` into main `Settings` struct in `rsudp-rust/src/settings.rs`

## Phase 2: Foundational Tasks

*Goal: Implement core Hue API client and discovery logic.*

**Independent Test Criteria**: Unit tests for config parsing and mocked API client pass.

- [x] T006 Implement `Discovery` module using `mdns-sd` to find bridge IP in `rsudp-rust/src/hue/discovery.rs`
- [x] T007 Implement `HueClient` struct with `reqwest` and self-signed cert support in `rsudp-rust/src/hue/client.rs`
- [x] T008 [P] Implement `register_app` (Link Button) method in `rsudp-rust/src/hue/client.rs`
- [x] T009 [P] Implement `get_lights` and `get_rooms` methods in `rsudp-rust/src/hue/client.rs`
- [x] T010 Implement `set_light_state` method (on/off, color, alert) in `rsudp-rust/src/hue/client.rs`
- [x] T011 [P] Implement `rgb_to_xy` color conversion helper in `rsudp-rust/src/hue/mod.rs`

## Phase 3: Setup & Configuration CLI (User Story 1)

*Goal: Enable users to discover bridges, pair, and list devices via CLI.*

**Independent Test Criteria**: `rsudp-hue setup` outputs app key; `rsudp-hue list` shows devices.

- [x] T012 [US1] Implement `setup` subcommand logic (discovery + pairing loop) in `rsudp-rust/src/bin/rsudp-hue.rs`
- [x] T013 [US1] Implement `list` subcommand logic (fetch lights/zones) in `rsudp-rust/src/bin/rsudp-hue.rs`
- [x] T014 [US1] Add CLI argument parsing with `clap` in `rsudp-rust/src/bin/rsudp-hue.rs`

## Phase 4: Alert Visual Signaling (User Story 2)

*Goal: Integrate Hue control into the main alert pipeline.*

**Independent Test Criteria**: Triggering an alert causes mocked Hue client to receive correct Pulse commands.

- [x] T015 [US2] Implement `HueIntegration` struct to manage runtime state and background discovery in `rsudp-rust/src/hue/mod.rs`
- [x] T016 [US2] Implement "Yellow Pulse" trigger logic in `rsudp-rust/src/hue/mod.rs`
- [x] T017 [US2] Implement JMA Color mapping logic for Reset pulse in `rsudp-rust/src/hue/mod.rs`
- [x] T018 [US2] Implement state restoration logic (store pre-alert state) in `rsudp-rust/src/hue/mod.rs`
- [x] T019 [US2] Integrate `HueIntegration` into `rsudp-rust/src/main.rs` (alert pipeline)
- [x] T020 [US2] Handle alert concurrency (preemption) logic in `rsudp-rust/src/hue/mod.rs`

## Phase 5: Polish & Cross-Cutting Concerns

*Goal: Finalize documentation and robustness.*

**Independent Test Criteria**: Documentation matches implementation; IP tracking works.

- [x] T021 Implement background task for periodic IP verification in `rsudp-rust/src/hue/discovery.rs`
- [x] T022 Update `rsudp.toml` template/example with [HUE] section
- [x] T023 Update `README.md` with Hue integration usage instructions

## Dependencies

1. **Phase 1**: Blocks everything.
2. **Phase 2**: Blocks Phase 3 & 4.
3. **Phase 3**: Can be tested independently of Phase 4.
4. **Phase 4**: Depends on core client (Phase 2) and config (Phase 1).

## Parallel Execution Examples

- **Client Methods**: T008, T009, and T010 can be implemented in parallel.
- **CLI Commands**: T012 (Setup) and T013 (List) are independent logic paths.

## Implementation Strategy

1. **Core**: Build the `HueClient` and `Discovery` modules first to ensure we can talk to the hardware.
2. **CLI**: Build the CLI tool to facilitate manual testing and configuration for the user.
3. **Integration**: Connect the logic to the main `rsudp` pipeline for real-time alerts.
