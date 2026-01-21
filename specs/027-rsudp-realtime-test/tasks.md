# Tasks: RSUDP Realtime Test Run

**Feature**: RSUDP Realtime Test Run
**Status**: Completed
**Implementation Strategy**: Update the existing test scripts to run at 1x speed and execute the long-running test.

## Phase 1: Configuration Update
Goal: Switch to realtime playback.

- [X] T001 Update `scripts/run_python_ref.sh` to use `--speed 1.0`
- [X] T002 Update `scripts/run_rust_target.sh` to use `--speed 1.0`

## Phase 2: Execution & Verification
Goal: Run the test and verify results.

- [X] T003 Execute `scripts/run_comparison.sh` (Long running task)
- [X] T004 Verify `logs/comparison_report.csv` contains matched events