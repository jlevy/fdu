---
type: is
id: is-01m2f3b5jgcfk96wq4r52xcag7
title: "perf-floor: per-subject host_regime stays quiet after the document downgrades to uncontrolled"
kind: bug
status: open
priority: 3
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T04:38:27.152Z
updated_at: 2026-09-14T04:38:27.152Z
---
PR #49 delta review PR49-DELTA-3 (https://github.com/jlevy/fdu/pull/49#pullrequestreview-5193948340). floor.py:757 vs :1041 at 6e018a0: when a later trial breaches the quiet gate, the document-level regime becomes uncontrolled, but each subject's host_regime in the JSON still says quiet. A reader of one subject's row sees a claim the document withdrew. Fix: derive the per-subject regime from its own trials' validity, or overwrite it on downgrade, and test it.
