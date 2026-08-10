---
type: is
id: is-01kzn1jf9f4k7amp6z8t5g07zx
title: Elide redundant index path walks and no-op applies during reconciliation
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies: []
parent_id: is-01kzg4ak7v8z2a7s41rsms8jcb
created_at: 2026-08-10T05:15:50.446Z
updated_at: 2026-08-10T05:21:28.838Z
closed_at: 2026-08-10T05:21:28.837Z
close_reason: "Implemented coherent EntryId-based present-child expectations and exclusive exact-no-op elision while retaining conditional ABA arbitration for IndexHandle. Three focused regressions plus the complete make check pass (153 all-feature Rust tests, all feature matrices, 26 golden blocks, 56 performance tests, docs/audits/Python/wheel/uvx). Nine alternating exact-oracle balanced-100k pairs all improved: 714.231 ms to 575.499 ms median, -18.15% paired median. Evidence: docs/project/research/research-2026-08-09-reconciliation-index-fast-path.md."
---
The first exact cost curve reaches 8.186 s at 500k and 62.906 s at 1M. Each known child expectation currently joins a path and performs repeated root-to-leaf lookups even though child iteration already has its EntryId; unchanged exclusive reconciliation then conditionally arbitrates and applies a guaranteed no-op. Construct present-child expectations directly from coherent entry identity/state, and on the &mut Index path count exact metadata matches without building/applying an observation. Preserve conditional ABA arbitration for IndexHandle, Delta as the only mutation path, accurate ApplyStats, removals/kind changes/partial errors, and exact oracle equality. Accept only with focused equivalence tests, paired same-corpus release evidence, and full make check.
