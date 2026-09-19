---
type: is
id: is-01m2wa4fzzg9afbtvd6ne8jxwr
title: "H117: opened-root retained read vs metadata one-shot on Darwin"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01m2wa49kmg1xct59bpbdxvs77
created_at: 2026-09-19T07:47:16.094Z
updated_at: 2026-09-19T07:47:16.094Z
---
Determination, not a CLI snapshot-load patch (H108). Opened-root second report at least 3% faster than one-shot default-tree / fdu PATH on an unchanged Darwin tree. Instrument: fdu-core probe mode or Python Index that opens once and reports twice. opened-discovery and warm-revalidate are different jobs. One-shot footer stays cold scan.
