---
type: is
id: is-01m138x5kdsqfcebcdp01y6rr7
title: Portable omission examples can empty out while the count stays high
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m0y1shykye8sc7h7e9rkk6kh
created_at: 2026-08-28T04:09:04.876Z
updated_at: 2026-08-28T04:09:04.876Z
---
ServingIndexes maintains portable_omitted as an exact count and portable_examples as a bounded sample (MAX_PORTABLE_PATH_EXAMPLES). The two drift apart under churn, and a consumer can receive an issue reporting a large omission count beside an empty example list, which reads as a contradiction.

Mechanism: insert_serving_entry pushes an example only while the list is below its cap, and remove_serving_entry retains-out the removed id unconditionally. Nothing refills a freed slot, because the engine keeps no list of omitted ids to refill from. So inserting twenty non-portable entries fills the sample with the first eight and sets the count to twenty; removing those same eight leaves count twelve and examples empty, permanently.

The exact count is the contract and it stays exact, so this is a quality-of-diagnostic issue rather than a correctness one. But 'omitted: 12, examples: []' looks like a bug in the example machinery, and the next reader will go looking for one.

Decide and record which of these is intended:

1. Document that examples are a bounded sample of omissions observed at insert time and may be empty while the count is positive, then state it in PortablePathIssue's own documentation so a consumer reads it there rather than inferring it.
2. Refill opportunistically by retaining candidate ids beyond the cap - rejected unless measured, since it adds unbounded retention to solve a presentation problem.
3. Stop removing examples on removal and let the rendering filter dangling ids, which portable_examples already does through path_of. Same observable outcome, less mutation.

Option 1 is the cheapest and most honest. Note that fdu-zgva, if adopted, deletes this problem outright along with the example machinery, so do not invest heavily here before that decision.
