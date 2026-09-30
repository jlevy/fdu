---
type: is
id: is-01m1687g2cazrcaxzwkpdcazz5
title: MetaBrowser tree page assembly does not enforce the request work budget
kind: bug
status: open
priority: 2
version: 4
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-08-29T07:54:57.995Z
updated_at: 2026-09-30T03:58:33.194Z
---
The plan spec's implementation table requires `assemble_tree_pages` / `TreePageAssembly`
to "Enforce stable provider version, positive row bound, unique advancing opaque
continuation, maximum pages, maximum rows, and request work budget."

Verified on MetaBrowser `codex/inventory-contract-alignment`,
`src/metabrowser/inventory_engine/tree_page_assembly.py`: it enforces the unique
`query_id`, first-page start, positive bounds, `max_pages`, the projection type, the
per-page row bound, filtered-total stability, engine version pinning, duplicate paths via
`seen_paths`, and `max_assembled_rows`. It never reads reported work. The word does not
appear in the file.

So one clause of the specified rule is unimplemented. That is not currently a failure, and
implementing it today would create one: fdu's tree projection can report `rows_visited`
above the requested `max_work` while its level-advance search runs (`fdu-pokc`), so a
literal implementation would reject valid pages as `InventoryConsistencyError` on exactly
the trees where the overrun happens.

Order matters: land `fdu-pokc` first so the engine honours the bound, then enforce it here.
Enforcing first would make the assembly reject correct pages.

Worth deciding explicitly whether the rule should be an assertion (reject) or an
observation (record and surface), since a provider that overruns its budget is a
performance fault rather than a correctness one, and the duplicate-path check already
catches the correctness failure this would otherwise stand in for.

## Notes

## Decided: observation, not assertion

`max_work` is a soft budget for the tree projection. It says where a page should stop, not
how much work to refuse, and a page reports what it actually spent. Overrunning slightly
is the normal case rather than a fault, because the page stops at the first position its
cursor can name and that position may be a little past the budget.

So this should record and surface the reported work, not reject the page. Rejecting
discards correct rows over a performance signal, and the duplicate-path check via
`seen_paths` already catches the correctness failure an assertion would otherwise stand in
for.

What the bound actually has to guarantee is termination and forward progress, and both are
now enforced in the engine rather than by the consumer: the level advance walks a finite
index strictly forward, and every page emits at least one row or ends the traversal. See
`opened::tests::a_page_moves_even_when_the_path_walk_spends_the_budget`.

This no longer needs to wait on a strict bound in `fdu-pokc`. It needs the reported work
surfaced somewhere a person will see it.

2026-09-30 stability pass (claude/stability-fixes): the code this bead names, MetaBrowser's src/metabrowser/inventory_engine/tree_page_assembly.py, is not in the fdu repository, so it cannot be changed here. fdu's side of the decision recorded above is in place at b1376507: a tree page always makes forward progress and terminates (opened::tests::a_page_moves_even_when_the_path_walk_spends_the_budget), and each page reports the work it spent, so the consumer can record and surface an overrun as the observation the notes decided on. Left open for a MetaBrowser session.
