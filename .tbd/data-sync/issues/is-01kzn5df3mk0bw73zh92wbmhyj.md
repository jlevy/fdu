---
type: is
id: is-01kzn5df3mk0bw73zh92wbmhyj
title: Raise supported Python floor to 3.12 and align uv packaging
kind: task
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzky8gxazfdstfgbv3m9fa58
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-10T06:23:00.723Z
updated_at: 2026-08-13T08:07:25.977Z
closed_at: 2026-08-13T08:07:25.976Z
close_reason: Raised the wheel and repository Python floor to 3.12, switched PyO3 to abi3-py312, regenerated both uv locks, removed obsolete pre-3.11 compatibility code, documented the floor, and added Python 3.12/3.14 CI coverage across all supported OSes. The Python 3.12 binding tests and 63 benchmark tests pass; a cp312-abi3 wheel builds and runs on local Python 3.14.
---
The maintainer explicitly authorizes dropping Python 3.9-3.11 support. Set the published fdu wheel and repository-owned Python tooling minimum to Python 3.12, align PyO3 abi3 and pyproject/uv lock/CI/docs, and test the minimum plus current interpreter. Selectively apply current jlevy/simple-modern-uv conventions while preserving fdu's Cargo workspace and maturin/PyO3 build, because the template's wholesale migration workflow excludes monorepos and native-extension projects. simple-modern-uv is first-party and exempt from the 14-day cool-off; do not add unnecessary Python runtime dependencies.

## Notes

Maintainer selected Python 3.12+ and explicitly dropped 3.9-3.11 support. Inspected first-party simple-modern-uv v0.4.0 at d05a34cf8c73d184a3f333ea478a3c2bd573d74e. Apply uv/version/CI conventions selectively; its adoption guide excludes monorepos and native extensions, so preserve Cargo workspace plus maturin/PyO3. Pending: pyproject requires-python, abi3-py312, uv lock, minimum/current CI, docs, installed wheel and uvx validation.
