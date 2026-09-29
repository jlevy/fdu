---
type: is
id: is-01m3neef6bv0c3baw3gahf4w6b
title: make check writes snapshots into the real user cache (~/.cache/fdu)
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-29T02:03:38.059Z
updated_at: 2026-09-29T02:03:38.059Z
---
crates/fdu-py/tests/public_smoke.py, pytest temp trees and a tryscript session write under the invoking user's cache; gates should set an isolated XDG_CACHE_HOME/FDU_CACHE_DIR. Found in the 0.2.1 stability pass.
