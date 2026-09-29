---
type: is
id: is-01m3p8164h8qbnqeyn3mpdqqva
title: "H180: summary-route walker trims (control spelling from name bytes, no per-file path clone into Op::Upsert)"
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
hold: null
hold_until: null
created_at: 2026-09-29T09:30:45.777Z
updated_at: 2026-09-29T11:34:54.232Z
started_at: 2026-09-29T10:40:08.475Z
closed_at: 2026-09-29T11:34:54.232Z
close_reason: "Accepted and merged (c2a75fe4): exp-183 node-modules-dense aggregate-summary -8.68%, exp-184 linux-v6.12 -5.63% (blind -12.76%); default-tree placebos include zero; walker instructions -28 to -33%."
resolution: null
duplicate_of: null
---
From the 2026-09-29 side-by-side profile: the summary route's walkers cost ~2x the default route's (2,602-3,087 vs 1,237-1,483 instructions per entry on linux-v6.12 / node-modules-dense). path_control_spelling (control.rs:834) parses Path::file_name() for every entry (~270 instructions per entry even with no .gitignore); every entry builds a PathBuf (scan.rs:3717) and clones it into Op::Upsert (scan.rs:3797) though files drop the original. Check the control spelling on the name bytes the walker already has, and move the path into the op when the entry does not descend. The clone piece overlaps H51 (refuted on macOS); the Linux mechanism differs: the summary route is CPU-bound (3.4 cores) and glibc frees cross threads. Deciding: aggregate-summary --no-controls and aggregate-summary on node-modules-dense and linux-v6.12, 20 pairs; placebo default-tree; prediction walker instructions -15-25% on the summary route. Opus implements and reviews (small item).
