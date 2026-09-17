---
type: is
id: is-01m2pydzqqs5qf4n3bxtaf3twt
title: "P1.1.1: Move the fixture, comparator, and matrix into tests/path_independence"
kind: task
status: in_progress
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: claude-code@spud10
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye0422qnb6171mmxabt4x
parent_id: is-01m2pmr9n3mq4nb2r1pc328qpz
hold: null
hold_until: null
created_at: 2026-09-17T05:46:32.045Z
updated_at: 2026-09-17T06:17:15.700Z
started_at: 2026-09-17T06:17:15.699Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 1: The Path-Independence Harness", commit 1. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `tests/path_independence/fixture.py`: `build_fixture(root) -> FixtureFacts`, ported from `explorations/path-independence/make_fixture.sh`: seeded bytes instead of `/dev/urandom`, `os.utime` instead of `touch -t`, symlinks skipped and recorded where `os.symlink` fails.
- `tests/path_independence/matrix.py`: `REQUESTS`, `WARMERS`, `MUTATIONS`, `ROUTES`, `SUBSET`, moved from the exploration's `R`, `W`, `mutate`, and `MUTATIONS`; routes `cli-report`, `py-report`, `py-open`, `py-scan`.
- `tests/path_independence/runner.py`: `run_cli`, `run_py`, `normalize`, `compare -> Verdict`, `case_key`. `normalize` drops only `source`, `freshness`, `scan_started_at`, and `generated_at`; verdicts are `same`, `differs`, or `outcome_class`.
- `tests/path_independence/pyrun.py`: `main`, moved; refuses to run when `fdu` imports from `crates/fdu-py/python` (the parity safety property).
- `FDU_BIN` is an absolute path defaulting to `target/debug/fdu`, never resolved through `PATH`; each invocation gets its own `XDG_CACHE_HOME`, which `user_cache_dir` honors on every platform (`lib.rs:794-799`).
- `Makefile`: add the directory to `PYTHON_LINT_PATHS` (`:345`).
- Python `unittest`, standard library only, run through uv's Python 3.12 as `release-test` is (`Makefile:380-381`).

**Tests**

- `tests/path_independence/test_harness.py`: comparator unit tests that need no fdu build.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
