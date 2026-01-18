# Feature Specification: Cleanup Project Files and Update README

**Feature Branch**: `021-cleanup-project-files`
**Created**: 2026-01-18
**Status**: Draft
**Input**: User description: "rsudp-rustディレクトリーにある、コミットする必要のなかったゴミファイルを削除して、README.md の内容をアップデートして"

## User Scenarios & Testing *(mandatory)*

## Clarifications

### Session 2026-01-18
- Q: .gitignore の更新 → A: はい。削除対象のファイルパターンを .gitignore に追記し、再発を防止する。
- Q: README の言語 → A: 英語と日本語の併記。

### User Story 1 - Project Directory Cleanup (Priority: P1)

As a maintainer, I want to remove untracked or incorrectly committed files from the `rsudp-rust` directory to ensure the repository remains clean and professional.

**Why this priority**: Accumulated artifacts like PID files, test outputs, and nested target directories clutter the codebase and can lead to confusion or deployment issues.

**Independent Test**:
1. Check `rsudp-rust` directory structure.
2. Confirm removal of `rsudp.pid`, `rsudp-rust/rsudp-rust/` nested directory, and generated PNG files in `rsudp-rust/alerts/`.
3. Verify only source code, essential configuration, and documentation remain.

**Acceptance Scenarios**:

1. **Given** artifacts from previous runs/tests, **When** the cleanup is performed, **Then** `rsudp.pid` MUST be deleted.
2. **Given** a nested `rsudp-rust/rsudp-rust/` directory, **When** the cleanup is performed, **Then** that directory MUST be deleted.
3. **Given** generated alert images in `rsudp-rust/alerts/`, **When** the cleanup is performed, **Then** all `.png` files MUST be removed while preserving `.gitkeep`.

---

### User Story 2 - README Update (Priority: P2)

As a developer, I want the `README.md` to provide comprehensive information about the project, including its purpose, features, build instructions, and usage, so that new users can easily get started.

**Why this priority**: A good README is essential for project visibility and ease of use. The current README is too minimal and doesn't reflect the significant functionality implemented so far.

**Independent Test**:
1. Open `rsudp-rust/README.md`.
2. Verify it contains:
   - Project overview
   - Key features (UDP receiver, STA/LTA alert, Seismic Intensity, WebUI)
   - Setup and Build instructions
   - How to run (Live mode, Simulation mode)
   - How to run tests (Unit, Integration, E2E)

**Acceptance Scenarios**:

1. **Given** an outdated README, **When** the update is performed, **Then** it MUST describe the project as a high-performance Rust implementation of `rsudp`.
2. **Given** the lack of usage instructions, **When** the update is performed, **Then** it MUST include examples of running the application and the streamer.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST identify and remove the following files/directories in `rsudp-rust/`:
  - `rsudp.pid`
  - `rsudp-rust/rsudp-rust/` (nested project artifact)
  - `alerts/*.png`
- **FR-002**: The system MUST update `rsudp-rust/.gitignore` to include patterns for the removed files to prevent re-committing them.
- **FR-003**: The system MUST update `rsudp-rust/README.md` in both English and Japanese with:
  - Project description.
  - Feature list.
  - Detailed build and run instructions.
  - Testing guide (including `e2e_alert`).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The `rsudp-rust` directory is free of build artifacts and temporary files (specifically `rsudp.pid` and nested `rsudp-rust` dir).
- **SC-002**: All generated image files are removed from the repository.
- **SC-003**: `README.md` is updated to at least 20 lines of content covering all implementation milestones.