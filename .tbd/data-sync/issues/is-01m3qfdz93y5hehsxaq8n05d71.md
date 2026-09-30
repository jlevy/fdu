---
type: is
id: is-01m3qfdz93y5hehsxaq8n05d71
title: "SLOC comparison: commit the per-file join and attribution script beside its evidence"
kind: task
status: open
priority: 4
version: 1
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3qdjym66ky971ft6yjdq7s8
created_at: 2026-09-29T20:59:19.203Z
updated_at: 2026-09-29T20:59:19.203Z
---
Review suggestion on #162: the 'first matching construct' attribution that assigned the 187 disagreeing kernel files to scc and tokei defects exists only as prose in the validation JSON's attribution_method. Commit a script next to the evidence in docs/project/research/ so the pass can be rerun when tokei 16 or scc 4.2 ships.
