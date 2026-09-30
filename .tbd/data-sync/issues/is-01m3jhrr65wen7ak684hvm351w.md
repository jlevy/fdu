---
type: is
id: is-01m3jhrr65wen7ak684hvm351w
title: Reject unrepresentable aggregate totals before index mutation
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-27T23:03:57.376Z
updated_at: 2026-09-30T03:48:52.134Z
closed_at: 2026-09-30T03:48:52.134Z
close_reason: "Fixed in 442af4d5: Index::preflight_totals refuses, before any mutation, a batch whose whole-tree files/dirs/bytes/allocated total would leave u64 after any accepted operation in order (root all roll-up bounds every other), with the new typed Error::UnrepresentableTotal { path, counter }; a cheap additive bound settles ordinary batches and only a batch failing it is replayed exactly (TotalsOverlay: replacements, kind changes, subtree removal, ordered duplicate paths, stale conditional ops, opened file budget). General, baseline, scanner, and reconcile lanes all pass through reduce_prepared. The snapshot loader fails closed on sizes summing past u64. Tests: a_batch_whose_byte_total_would_exceed_u64_is_refused_before_any_mutation, exact_fit_replacement_removal_and_kind_change_batches_are_accepted, a_stale_conditional_upsert_does_not_count_toward_the_projected_total, the_baseline_and_scanner_lanes_refuse_unrepresentable_totals, a_snapshot_whose_sizes_sum_past_u64_fails_closed, a_selection_over_an_exactly_full_tree_sums_to_u64_max; four of them panicked before the fix. Callgrind on the scanner lane (drivers/net, 6,653 entries, 3 rounds each): +0.004% instructions, inside round-to-round variance. The detached one-shot builder only reads stat sizes and is unchanged."
resolution: null
duplicate_of: null
---
Public Index::apply accepts arbitrary u64 Attrs.size/allocated. Two file upserts with sizes u64::MAX and 1 reach InternedRollUp::merge (crates/fdu-core/src/index.rs:301-321): debug builds panic on `+=`, release builds wrap apparent/allocated and extension totals. Query subtree projection also uses unchecked additions (crates/fdu-core/src/query/query_subtrees.rs:100-106, 151-161). The result may claim an exact, small total and subtraction cannot restore it.

Fix with a typed, fault-atomic rejection before any index mutation when the projected accepted batch would produce an unrepresentable count/byte total. Preserve legitimate replacement/removal batches whose final totals fit; do not silently saturate or wrap. Preflight must honor ordered duplicate paths, conditional stale operations, kind replacement and subtree removal, and scanner/baseline/reconcile lanes. Add a minimal u64::MAX+1 regression, an exact-fit replacement/removal regression, and a failed-batch unchanged-index assertion, plus selected-query overflow coverage. Ordinary filesystem inputs would need a >16 EiB aggregate to trigger this; the public synthetic Index API reaches it directly.
