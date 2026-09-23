---
type: is
id: is-01m2pmrmvr6x2kz662jksackef
title: "JSONL writer rewrites strings: collapse() edits bracketed paths"
kind: bug
status: closed
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - output
  - release
dependencies: []
parent_id: is-01m2pj0f459s8ad1efzyn2qmbq
created_at: 2026-09-17T02:57:35.607Z
updated_at: 2026-09-23T08:14:06.932Z
closed_at: 2026-09-23T08:14:06.932Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
report_format.rs collapse() joins pretty JSON lines and replaces "{ ", " }", "[ ", " ]" globally, so a path
`a [ b/f { g }.txt` is emitted as `a [b/f {g}.txt` (confirmed on the release build). Also: --watch --format yaml
emits JSON change records. Resolved by writers serializing the answer model directly.
