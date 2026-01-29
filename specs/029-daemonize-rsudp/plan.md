# Implementation Plan - Daemonization and System Installation Support

**Feature**: Daemonization and System Installation Support
**Status**: Planning
**Link**: [Feature Spec](spec.md)

## 1. Technical Context

### 1.1 Architecture
The architecture remains largely unchanged, focusing on the deployment and operational aspect of the existing binary.
- **Service Layer**: A new `systemd` service file (`rsudp.service`) will be introduced to manage the process lifecycle.
- **Installation Layer**: A `Makefile` will be created to automate the compilation, binary placement, configuration deployment, and service registration.
- **Permissions**: A dedicated `rsudp` user will isolate the service from root privileges.

### 1.2 Dependencies
- **Systemd**: Required for service management. Standard on target OS (Ubuntu/Debian).
- **Make**: Required for build orchestration.
- **useradd/groupadd**: Standard system utilities for user management.

### 1.3 Unknowns & Risks
- **Risk**: User permissions on USB devices or serial ports (if used for hardware connection) might require `dialout` group membership.
  - *Mitigation*: Research if `rsudp` typically needs specific hardware groups and add them to the user creation logic.
- **Risk**: Log directory permissions.
  - *Mitigation*: Ensure `make install` creates `/var/log/rsudp` (or similar) with correct ownership if file logging is used, though spec mentions `journald` (stdout).

## 2. Constitution Check

| Principle | Check | Result |
| :--- | :--- | :--- |
| **I. Stability** | Does auto-restart loop prevention align with stability? | **PASS**. Prevents resource exhaustion. |
| **II. Testing** | Can installation be tested? | **PASS**. E2E tests for install/uninstall are feasible in CI or VMs. |
| **III. Performance** | Does daemonization affect performance? | **PASS**. Negligible overhead. |
| **IV. Maintainability** | Is the Makefile clear? | **PASS**. Using standard targets (`all`, `install`, `uninstall`). |
| **V. Japanese Spec** | Is the spec in Japanese/English bilingual? | **PASS**. (Spec is English, plan context bilingual capability). |
| **VI. Standard Stack** | N/A | N/A |
| **VII. Self-Verification** | Can I verify `make install` locally? | **PASS**. Requires sudo but verifiable. |
| **VIII. Branching** | Using a feature branch? | **PASS**. `029-daemonize-rsudp`. |

## 3. Phase 0: Research & Design

### 3.1 Research Topics
- **Config Path**: Best practice for passing config path to Rust binary (clap args vs env var). Spec clarified CLI args.
- **Service Hardening**: What systemd security options (e.g., `ProtectSystem=full`) should be enabled?
- **User Groups**: Does `rsudp` typically need `dialout` or `input` groups for Shake/USB access?

### 3.2 Design Decisions
- **Makefile Path**: Root of repo.
- **Install Paths**:
  - Binary: `/usr/local/bin/rsudp-rust`
  - Config: `/etc/rsudp/rsudp.toml`
  - Service: `/etc/systemd/system/rsudp.service`
- **User**: `rsudp` (system user, no home, nologin).

## 4. Phase 1: Implementation

### 4.1 Artifacts
- `Makefile`
- `rsudp.service` (template)
- `README.md` (updates)

### 4.2 Steps
1. Create `rsudp.service` file with `ExecStart`, `User=rsudp`, `Restart=on-failure`.
2. Create `Makefile` with targets:
   - `build`: `cargo build --release`
   - `install`: Copy binary, create user, copy service, reload daemon.
   - `uninstall`: Stop service, remove files, (optionally keep user/config).
3. Update `README.md` with detailed installation instructions.

## 5. Phase 2: Verification

### 5.1 Manual Verification Scenarios
- [ ] Run `make` -> succeeds.
- [ ] Run `sudo make install` -> user created, binary in `/usr/local/bin`, service active.
- [ ] `systemctl status rsudp` -> shows running.
- [ ] `kill -9 <pid>` -> service restarts automatically.
- [ ] `sudo make uninstall` -> service gone, binary gone.

### 5.2 Automated Tests
- Not applicable for `systemd` integration within standard `cargo test`. Manual verification or specialized VM-based CI required.

## 6. Sign-off
- [ ] Design approved
- [ ] Implementation completed
- [ ] Verified manually on target OS