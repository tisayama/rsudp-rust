# Tasks: RSUDP Test Run and Comparison

**Feature**: RSUDP Test Run and Comparison
**Status**: Completed
**Implementation Strategy**: Set up a Python venv for the reference implementation, create a shell script to orchestrate sequential runs of both systems, and a Python script to parse and compare the logs.

## Phase 1: Setup
Goal: Prepare the environment for reference execution.

- [X] T001 Create `scripts/` directory if not exists
- [X] T002 Implement `scripts/setup_python_env.sh` to create `rsudp-venv` and install dependencies from `references/rsudp/docsrc/requirements.txt`

## Phase 2: Foundational
Goal: Create configuration and parsing tools.

- [X] T003 Create `scripts/rsudp_settings.json` with the user-provided alert parameters
- [X] T004 Create `scripts/rsudp_settings.toml` with the equivalent parameters for Rust
- [X] T005 Implement `scripts/compare_logs.py` to parse logs and generate `comparison_report.csv`

## Phase 3: [US1] Run Python Reference (Priority: P1)
Goal: Generate the baseline log.

- [X] T006 [US1] Implement `scripts/run_python_ref.sh` to start `rsudp` client and `streamer` (target 127.0.0.1:8888)

## Phase 4: [US2] Run Rust Target (Priority: P2)
Goal: Generate the target log.

- [X] T007 [US2] Implement `scripts/run_rust_target.sh` to start `rsudp-rust` and `streamer` (target 127.0.0.1:8888)

## Phase 5: [US3] Comparative Analysis (Priority: P3)
Goal: Compare results.

- [X] T008 [US3] Implement `scripts/run_comparison.sh` driver script to execute setup, runs, and comparison sequentially
- [X] T009 [US3] Execute `scripts/run_comparison.sh` and verify output

## Dependencies

- Phase 3/4 depend on Phase 1/2.
- Phase 5 depends on Phase 3/4 logs.

## Parallel Execution Examples

- T003/T004 (Configs) and T005 (Parser) can be implemented in parallel.

## Implementation Strategy

1. **Scripts**: Build the harness first.
2. **Execute**: Run the harness to generate artifacts.
3. **Analyze**: Check the CSV report.