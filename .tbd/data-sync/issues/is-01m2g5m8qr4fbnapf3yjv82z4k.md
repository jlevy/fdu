---
type: is
id: is-01m2g5m8qr4fbnapf3yjv82z4k
title: make python-smoke picks a free-threaded Python when the host has one, and cannot install the abi3 wheel
kind: bug
status: in_progress
priority: 3
version: 3
labels:
  - stack-followup
  - release
dependencies: []
created_at: 2026-09-14T14:37:36.888Z
updated_at: 2026-09-15T05:36:44.591Z
---
Found by the pre-merge make check on 2026-09-14. The python-smoke recipe in the Makefile runs 'uv venv --clear .venv-smoke' without --python. On a host where uv manages a free-threaded CPython 3.14 (uv 0.12.8 chose 3.14.7+freethreaded), the venv is cp314t, and installing the abi3 wheel fails with 'fdu==0.1.0 has no wheels with a free-threading compatible ABI tag'. CI is unaffected because setup-uv always pins the Python version. Locally, make check fails at python-smoke until UV_PYTHON=3.12 is set. Fix: pass an explicit, non-free-threaded --python to uv venv in python-smoke (for example the CI job's version), or skip the wheel smoke with a clear message when the interpreter is free-threaded. If a free-threaded wheel is intended, build one.
