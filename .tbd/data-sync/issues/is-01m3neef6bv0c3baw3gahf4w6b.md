---
type: is
id: is-01m3neef6bv0c3baw3gahf4w6b
title: make check writes snapshots into the real user cache (~/.cache/fdu)
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-29T02:03:38.059Z
updated_at: 2026-09-30T02:45:12.626Z
closed_at: 2026-09-30T02:45:12.625Z
close_reason: "Fixed in b0e82ff9: run-golden.mjs gives each golden/parity run a temp XDG_CACHE_HOME and drops FDU_CACHE_DIR (sessions naming their own still win); crates/fdu-py/tests/conftest.py autouse fixture isolates every pytest test (outside tmp_path) and test_cache_isolation.py pins it via fdu.cache_directory(); public_smoke.py and smoke.py run main() under a temp cache home; the terminal test drops an inherited FDU_CACHE_DIR. Verified: stub-tryscript run shows XDG=/tmp/fdu-golden-cache-* and FDU_CACHE_DIR unset, dir removed afterwards; the conftest fixture exercised in a scratch pytest (isolation + a test's own setenv wins); ruff clean. Rust tests were not changed (their cache-writing tests already isolate). Needs the coordinator's make check for the pytest/smoke/golden runs against a built extension."
resolution: null
duplicate_of: null
---
crates/fdu-py/tests/public_smoke.py, pytest temp trees and a tryscript session write under the invoking user's cache; gates should set an isolated XDG_CACHE_HOME/FDU_CACHE_DIR. Found in the 0.2.1 stability pass.
