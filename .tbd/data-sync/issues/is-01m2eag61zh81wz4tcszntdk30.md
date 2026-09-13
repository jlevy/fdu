---
type: is
id: is-01m2eag61zh81wz4tcszntdk30
title: "PR #51 review PLAN-1: plan records a warm serving path that errors and closes fdu-etfj for one surface of three"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2eafpfpe8k5c9z9dhrqvy2y
created_at: 2026-09-13T21:24:17.086Z
updated_at: 2026-09-13T21:24:17.086Z
---
PR #51 review PLAN-1 (Medium). plan-2026-08-25-fdu-opened-root-inventory-engine.md lines 1780-1783 say a watch-maintained cache still serves a plain report; 1807-1810 and 2185 mark fdu-etfj done. A plain report never reads the snapshot, --analyze errored (COMMIT-1), and Python's default roll-up still performed control-file I/O (COMMIT-3).

Fix in step with COMMIT-1 and COMMIT-3: state the acceptance per surface (CLI one-shot, Python one-shot, opened root) and describe --cache only as the one directional path.
