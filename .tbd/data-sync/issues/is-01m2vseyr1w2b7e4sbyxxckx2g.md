---
type: is
id: is-01m2vseyr1w2b7e4sbyxxckx2g
title: "H107: measure default gitignore-on vs no-controls"
kind: task
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01m2x7xbt1c4we0wffeth7jrd6
created_at: 2026-09-19T02:55:53.088Z
updated_at: 2026-09-19T16:28:25.524Z
closed_at: null
close_reason: null
resolution: null
duplicate_of: null
---
Pre-register: on a git-heavy deciding subject, default-tree with read_controls on vs the same binary with read_controls off / --no-gitignore. Metric: wall_ns. Accept: |median| >= 3% and 95% CI excludes zero, either direction — this is a characterization that chooses the honest control, not a speed win. Oracle: tallies/digest must differ by the ignored set when controls are on. Do not accept a files/sec README number from the no-controls arm.

## Notes

2026-09-19 morning: reopened. Open only where ignore is the walk. Refuted on metabrowser (exp-106): wall +1.64% [-4.00%, +4.37%]. Do not retry metabrowser. Next-up after H122 may name a subject. Parent remaining-queue epic: fdu-8ya1.
