---
type: is
id: is-01m2yt3kzcakn6nka88x1z48ja
title: Older reconciliation can publish stale failures after newer verification
kind: bug
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-20T07:04:53.482Z
updated_at: 2026-09-23T08:14:06.795Z
closed_at: 2026-09-23T08:14:06.795Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Overlapping reconciliations can finish out of order. Retained issues now carry the originating pass epoch, but an older pass that closes after a newer clean pass can still add its stale errors and partial freshness because completed verification ownership is not retained by epoch. Conditional fact arbitration does not necessarily reject unchanged newer verification. Track latest completed coverage by scope/path or otherwise suppress older closure evidence.

## Notes

Confirmed during fdu-8jn2 review. The omission fix tags pass-produced details with the originating epoch, but no completed-scope epoch currently lets an older closer know a newer clean verification superseded its errors. This broader fact/freshness ordering defect remains open for a dedicated change; it is distinct from scalar omitted-count erasure and skipped-subtree cleanup.
