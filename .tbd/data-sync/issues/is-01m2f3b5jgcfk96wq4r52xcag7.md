---
type: is
id: is-01m2f3b5jgcfk96wq4r52xcag7
title: "perf-floor: per-subject host_regime stays quiet after the document downgrades to uncontrolled"
kind: bug
status: closed
priority: 3
version: 3
labels:
  - stack-followup
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-14T04:38:27.152Z
updated_at: 2026-09-30T02:55:44.366Z
closed_at: 2026-09-30T02:55:44.365Z
close_reason: "Fixed in d92c115e with fdu-ayzg: when any measured trial breaches the gate, run() overwrites every subject's host_regime with uncontrolled (the document's claim) and rescores it; each subject keeps its own invalid_trials to show where the breach happened. Test: a two-subject downgrade where only the first breached leaves both subjects uncontrolled and undecided; make perf-test OK."
resolution: null
duplicate_of: null
---
PR #49 delta review PR49-DELTA-3 (https://github.com/jlevy/fdu/pull/49#pullrequestreview-5193948340). floor.py:757 vs :1041 at 6e018a0: when a later trial breaches the quiet gate, the document-level regime becomes uncontrolled, but each subject's host_regime in the JSON still says quiet. A reader of one subject's row sees a claim the document withdrew. Fix: derive the per-subject regime from its own trials' validity, or overwrite it on downgrade, and test it.
