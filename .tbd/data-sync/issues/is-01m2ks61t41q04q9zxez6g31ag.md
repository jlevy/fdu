---
type: is
id: is-01m2ks61t41q04q9zxez6g31ag
title: "PR #65 review F3: nothing asserts fdu.report forwards read_controls and the control limits"
kind: bug
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m2ks5fy0rg2stv9nf1vc7wzb
created_at: 2026-09-16T00:17:05.856Z
updated_at: 2026-09-16T00:17:05.856Z
---
PR #65. crates/fdu-py/src/lib.rs:1283,:1313,:1337; crates/fdu-py/python/fdu/_api.py:441; tests/parity/py/parity_cli.py scan_options(); crates/fdu/src/cli.rs:600-619. public_smoke.py:371 checks fdu.scan only, so the pending #63 two-limit split could silently drop a limit from fdu.report. Add the forwarding test, then plumb both limits at every site when #63 lands.
