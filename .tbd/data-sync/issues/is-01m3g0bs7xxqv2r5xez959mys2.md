---
type: is
id: is-01m3g0bs7xxqv2r5xez959mys2
title: Define the freshness contract for FSEvents journal-scoped answers
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md
labels: []
dependencies: []
created_at: 2026-09-26T23:21:17.806Z
updated_at: 2026-09-27T00:45:38.785Z
closed_at: 2026-09-27T00:45:38.785Z
close_reason: "Completed in PR #131 (d607d489): reviewed reproducible cross-process probe, explicit opt-in journal freshness contract, reuse of watch build feature, and additional multi-root hour/day disk-history proposal. Full local make check, cross-lint, probe tests, and all CI checks passed. Production replay, long-gap acceptance, and end-to-end large-tree latency remain separately tracked."
resolution: null
duplicate_of: null
---
Before history replay can be enabled automatically, reconcile the active plan's risk-bounded Source::JournalScoped route with the first-principle contract that cache and delivery change cost, not answers. Decide whether journal-scoped output requires explicit opt-in, is a provisional result followed by verified convergence, or changes the documented default contract. Ensure text as well as machine output makes the weaker trust visible; age bounds and periodic sweeps do not make an individual answer exact.
