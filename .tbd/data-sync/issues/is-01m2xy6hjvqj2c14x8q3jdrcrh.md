---
type: is
id: is-01m2xy6hjvqj2c14x8q3jdrcrh
title: "H136: first-run default-tree leftover after H128"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01m2x7xbt1c4we0wffeth7jrd6
created_at: 2026-09-19T22:57:09.209Z
updated_at: 2026-09-19T23:08:09.179Z
closed_at: 2026-09-19T23:08:09.178Z
close_reason: "exp-135: confirmed leftover. first-run default-tree-first walk 83-88%. isolated snapshot-save 45.3ms (~11-16%), not skippable. no engine patch. quiet 75.4%. pair uncontrolled."
resolution: null
duplicate_of: null
---
After H128 (second-run default-tree walk 92.9%, snapshot_written false), leftover on deciding-scale default-tree-first: whether snapshot encode/write/render is >=3% of first-run wall and not already rejected (H100, H101, H78/H92, fdu-n75m).

Not content-cache-hit. Not a snapshot load on fdu PATH. Not H128. Quiet once; if fail, uncontrolled. Do not compile a cut unless inventory names a skippable >=3% mechanism.

exp-135. Frozen metabrowser-clone.
