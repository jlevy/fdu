---
type: is
id: is-01m3gtxfx2qmeg2d77gjcq3h6w
title: Allow smoke-test environments outside the source checkout
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-27T07:05:21.047Z
updated_at: 2026-09-27T07:05:21.047Z
---
make python-smoke and python-sdist-smoke hardcode crates/fdu-py/.venv-smoke and .venv-sdist even when TMPDIR, CARGO_TARGET_DIR, UV_CACHE_DIR, and UV_PROJECT_ENVIRONMENT point to external task scratch. This puts disposable test environments on the internal source volume under the local macOS storage policy. Expose explicit environment-directory configuration and keep parity/typecheck consumers on the same paths. In the macOS benchmark-refresh task, both task-owned environments were relocated to external scratch after the gate, with checkout symlinks retained and imports verified. No Makefile change included in PR #132.
