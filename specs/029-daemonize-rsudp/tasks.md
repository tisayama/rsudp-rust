# Tasks: Daemonization and System Installation Support

**Feature**: Daemonization and System Installation Support
**Status**: Ready for Implementation

## Implementation Strategy
- **Infrastructure First**: Implement the `Makefile` and `rsudp.service` file independently.
- **Incremental Verification**: Verify the `make` build process, then `make install` in a VM or local environment (if safe), and finally systemd integration.
- **Documentation**: Ensure `README.md` is updated with clear steps for the end-user.

## Dependencies
- **Phase 1 (Setup)**: Must be completed first to establish file locations.
- **Phase 2 (Foundational)**: `Makefile` and `rsudp.service` are the core deliverables.
- **Phase 3+ (User Stories)**: Validate scenarios based on the artifacts created in Phase 2.

## Phase 1: Setup
**Goal**: Initialize necessary templates and file structures.

- [x] T001 Create `Makefile` in project root with placeholder targets.
- [x] T002 Create `rsudp.service` in `rsudp-rust/systemd/` (new directory) based on data model.

## Phase 2: Foundational (Blocking Prerequisites)
**Goal**: Implement the core installation logic and service definition.

- [x] T003 Implement `build` target in `Makefile` to run `cargo build --release`.
- [x] T004 Implement `install` target in `Makefile` to copy binary, config, service file, and create user `rsudp`.
- [x] T005 Implement `uninstall` target in `Makefile` to remove binary, service file, and clean up.
- [x] T006 [P] Update `rsudp.service` with security hardening (`NoNewPrivileges`, `PrivateTmp`, etc.) and `Restart=on-failure`.

## Phase 3: User Story 1 - Standard Installation
**Goal**: Users can build and install the application using standard `make` commands.
**Story**: [US1] Standard Installation

- [ ] T007 [US1] Verify `make install` places binary in `/usr/local/bin/rsudp-rust`.
- [ ] T008 [US1] Verify `make install` places config in `/etc/rsudp/rsudp.toml`.
- [ ] T009 [US1] Verify `make install` creates `rsudp` system user.
- [ ] T010 [US1] Verify `make install` enables and starts `rsudp.service`.

## Phase 4: User Story 2 - Auto-Restart on Crash
**Goal**: Service automatically restarts on failure.
**Story**: [US2] Auto-Restart on Crash

- [ ] T011 [US2] Verify `Restart=on-failure` directive in `rsudp.service`.
- [ ] T012 [US2] Verify `StartLimitIntervalSec` and `StartLimitBurst` settings in `rsudp.service`.

## Phase 5: Documentation & Polish
**Goal**: Ensure users know how to use the new installation method.

- [x] T013 Update `rsudp-rust/README.md` with prerequisites (systemd, make, rust).
- [x] T014 Update `rsudp-rust/README.md` with `make install` and `systemctl` usage instructions.
- [x] T015 Verify `make uninstall` cleans up correctly (idempotency check).

## Phase 6: Final Review
**Goal**: Cross-cutting concerns and final verification.

- [ ] T016 Manual verification of `systemctl status rsudp` after install.
- [ ] T017 Manual verification of logging via `journalctl -u rsudp`.
