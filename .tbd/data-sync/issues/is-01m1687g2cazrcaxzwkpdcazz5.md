---
type: is
id: is-01m1687g2cazrcaxzwkpdcazz5
title: MetaBrowser tree page assembly does not enforce the request work budget
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-29T07:54:57.995Z
updated_at: 2026-08-29T07:54:57.995Z
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
