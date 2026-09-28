---
type: is
id: is-01m3jhrr65wen7ak684hvm351w
title: Reject unrepresentable aggregate totals before index mutation
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-27T23:03:57.376Z
updated_at: 2026-09-27T23:03:57.376Z
---
Public Index::apply accepts arbitrary u64 Attrs.size/allocated. Two file upserts with sizes u64::MAX and 1 reach InternedRollUp::merge (crates/fdu-core/src/index.rs:301-321): debug builds panic on `+=`, release builds wrap apparent/allocated and extension totals. Query subtree projection also uses unchecked additions (crates/fdu-core/src/query/query_subtrees.rs:100-106, 151-161). The result may claim an exact, small total and subtraction cannot restore it.

Fix with a typed, fault-atomic rejection before any index mutation when the projected accepted batch would produce an unrepresentable count/byte total. Preserve legitimate replacement/removal batches whose final totals fit; do not silently saturate or wrap. Preflight must honor ordered duplicate paths, conditional stale operations, kind replacement and subtree removal, and scanner/baseline/reconcile lanes. Add a minimal u64::MAX+1 regression, an exact-fit replacement/removal regression, and a failed-batch unchanged-index assertion, plus selected-query overflow coverage. Ordinary filesystem inputs would need a >16 EiB aggregate to trigger this; the public synthetic Index API reaches it directly.
