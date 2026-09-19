---
type: is
id: is-01m2wa4fzzg9afbtvd6ne8jxwr
title: "H117: opened-root retained read vs metadata one-shot on Darwin"
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
parent_id: is-01m2wa49kmg1xct59bpbdxvs77
created_at: 2026-09-19T07:47:16.094Z
updated_at: 2026-09-19T09:01:06.183Z
closed_at: 2026-09-19T09:01:06.182Z
close_reason: |
  exp-116: confirmed. opened-second-report 1.6ms vs default-tree 2612ms (~1630x) on system-private-frameworks. Probe mode kept. Not a snapshot load on fdu PATH. Next: H120.
resolution: null
duplicate_of: null
---
Determination, not a CLI snapshot-load patch (H108). Opened-root second report at least 3% faster than one-shot default-tree / fdu PATH on an unchanged Darwin tree. Instrument: fdu-core probe mode or Python Index that opens once and reports twice. opened-discovery and warm-revalidate are different jobs. One-shot footer stays cold scan.

## Notes

H117 / exp-116 pre-register (2026-09-19).

Claim: a metadata one-shot throws the index away (H108). An opened root retains it.
A second opened-root tree report on an unchanged Darwin tree is at least 3% faster
than a one-shot of the same request (expected several-fold).

Instrument: probe mode opened-second-report (engine public API; not a CLI flag).
Job pair: default-tree (one-shot) vs opened-second-report (component = second read).
Subject: system-private-frameworks (H108 tree) or metabrowser-clone.
Accept: opened second-report component below default-tree wall by at least 3%;
one-shot stays a cold scan. Determination, not a snapshot load on fdu PATH.

Same binary both variants. Quiet first; uncontrolled OK. No RAM disk.
