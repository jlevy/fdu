---
type: is
id: is-01m2vseyr1w2b7e4sbyxxckx2g
title: "H107: measure default gitignore-on vs no-controls"
kind: task
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m2vs96dg6kw3ka6k4ghyvf0f
created_at: 2026-09-19T02:55:53.088Z
updated_at: 2026-09-19T04:42:57.843Z
closed_at: 2026-09-19T04:42:57.843Z
close_reason: "exp-105 baseline recorded (uncontrolled; quiet cell failed). exp-106 rejected H107: wall +1.64% [-4.00%, +4.37%] on metabrowser."
resolution: null
duplicate_of: null
---
Pre-register: on a git-heavy deciding subject, default-tree with read_controls on vs the same binary with read_controls off / --no-gitignore. Metric: wall_ns. Accept: |median| >= 3% and 95% CI excludes zero, either direction — this is a characterization that chooses the honest control, not a speed win. Oracle: tallies/digest must differ by the ignored set when controls are on. Do not accept a files/sec README number from the no-controls arm.
