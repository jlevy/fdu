---
type: is
id: is-01m2ks61t41q04q9zxez6g31ag
title: "PR #65 review F3: nothing asserts fdu.report forwards read_controls and the control limits"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2ks5fy0rg2stv9nf1vc7wzb
created_at: 2026-09-16T00:17:05.856Z
updated_at: 2026-09-16T05:51:33.840Z
closed_at: 2026-09-16T05:51:33.839Z
close_reason: "302ecf0 (PR #65): merged #63 at 9105768 and plumbed both limits through report_once, _api.report, parity_cli's single scan_options, and the CLI's scan_config. public_smoke.py::check_a_one_shot_report_forwards_every_control_knob pins each knob by an answer only it produces: default limits apply the rule, a lowered control_line_limit refuses with line_limit, a lowered control_budget refuses with budget, read_controls=False reports no rules."
resolution: null
duplicate_of: null
---
PR #65. crates/fdu-py/src/lib.rs:1283,:1313,:1337; crates/fdu-py/python/fdu/_api.py:441; tests/parity/py/parity_cli.py scan_options(); crates/fdu/src/cli.rs:600-619. public_smoke.py:371 checks fdu.scan only, so the pending #63 two-limit split could silently drop a limit from fdu.report. Add the forwarding test, then plumb both limits at every site when #63 lands.
