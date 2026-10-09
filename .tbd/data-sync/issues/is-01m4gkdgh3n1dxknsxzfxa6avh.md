---
type: is
id: is-01m4gkdgh3n1dxknsxzfxa6avh
title: Format machine-output instants into a fixed buffer instead of a String per row
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T15:09:59.202Z
updated_at: 2026-10-09T15:09:59.202Z
---
Review C7 on #191 (https://github.com/jlevy/fdu/pull/191#issuecomment-6082950211), measured in exp-212 (macOS, rustup, uncontrolled host, 12 pairs, control 148ef78e vs ae90aef4): render-json component 128.8 -> 163.2 ms, +26.60% [+26.47%, +27.11%], wall +4.27% [+3.36%, +6.47%]; render-yaml component 118.4 -> 156.4 ms, +32.37% [+31.15%, +33.01%], wall +5.98% [+4.10%, +7.62%]. About 151k rows (an unbounded tree plus a file list), so about 0.23 us a row in JSON and 0.25 us in YAML, from the new per-row fields, chiefly modified_at. Where: report_format.rs emit_instant (list rows and tree nodes) and query_values.rs format_rfc3339_nanos (civil_from_days, then a seven-field zero-padded format! into a new String). Fix per review C: write the 30 RFC 3339 bytes into a fixed stack buffer handed to the sink (no String, no format!), then re-run render-json, render-yaml, and render-jsonl paired against ae90aef4; also measure a 100k-row Python list report to decide whether Python's eager _instant datetime derivation (_models.py) should be lazy. Decide before 0.5.0 whether the residual cost is acceptable.
