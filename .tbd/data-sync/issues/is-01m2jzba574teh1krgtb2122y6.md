---
type: is
id: is-01m2jzba574teh1krgtb2122y6
title: "PR #61 review PR61-PY-1: UV_PYTHON=3.14t re-creates the fdu-pd1b failure with uv's misleading message"
kind: bug
status: closed
priority: 3
version: 3
labels:
  - release
dependencies: []
parent_id: is-01m2jzadk7w8m1xcsewzwg5wj1
created_at: 2026-09-15T16:45:35.266Z
updated_at: 2026-09-15T17:10:19.210Z
closed_at: 2026-09-15T17:10:19.209Z
close_reason: "f2fedf8: wheel-python prerequisite refuses a free-threaded WHEEL_PYTHON with a clear message; Node test via make -n"
resolution: null
duplicate_of: null
---
PR #61 at eb89150, Makefile:211. WHEEL_PYTHON ?= $(or $(UV_PYTHON),3.12) passes a free-threaded request (3.14t) straight to uv venv, which then fails with 'no wheels with a free-threading compatible ABI tag', the exact fdu-pd1b message. Guard the wheel-installing targets so a free-threaded WHEEL_PYTHON is refused with a clear message, tested through make.
