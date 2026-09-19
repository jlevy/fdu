---
type: is
id: is-01m2vseyr1w2b7e4sbyxxckx2g
title: "H107: measure default gitignore-on vs no-controls"
kind: task
status: in_progress
priority: 1
version: 9
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
delegate: unknown@spud10
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01m2x7xbt1c4we0wffeth7jrd6
hold: null
hold_until: null
created_at: 2026-09-19T02:55:53.088Z
updated_at: 2026-09-19T19:59:55.397Z
started_at: 2026-09-19T17:59:25.895Z
closed_at: null
close_reason: null
resolution: null
duplicate_of: null
---
Pre-register: on a git-heavy deciding subject, default-tree with read_controls on vs the same binary with read_controls off / --no-gitignore. Metric: wall_ns. Accept: |median| >= 3% and 95% CI excludes zero, either direction — this is a characterization that chooses the honest control, not a speed win. Oracle: tallies/digest must differ by the ignored set when controls are on. Do not accept a files/sec README number from the no-controls arm.

## Notes

Skipped. Hunt recorded in exp-122: rustup and frameworks have 0 .gitignore; metabrowser is exp-106; cargo-registry screens only; tbd (145 gi / 34008 / opens 4876=4876) and urollup (80 gi / 97155 / opens 8599=8599) keep the same dir_opens with controls on and off. should_descend does not consult ignore, so a large ignored subtree cannot be the walk. Do not retry metabrowser. Do not invent a subject.
