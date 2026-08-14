---
type: is
id: is-01m00hb67nwmzhvy8z8eak0bbp
title: Eleven reallocations per index entry, source unattributed
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T16:23:07.764Z
updated_at: 2026-08-14T16:23:07.764Z
---
The new per-layer counters (exp-052) show a 450k-entry cold scan performing 4.94 million reallocations, about 11.0 per entry, alongside 15.4 allocations per entry and 2,456 bytes allocated per entry. Reallocation is Vec growth, and 11 per entry is far more than the walk's obvious per-entry allocations account for. The count tracks roll-up merges almost one to one (11.9 per entry), which suggests InternedRollUp::merge, but by_ext is a BTreeMap and those allocate fixed-size nodes rather than reallocating - so the correlation is probably coincidence, both being proportional to tree depth. Counters localize this to a layer without attributing it to a call site, which is the honest limit of instrumentation that does not sample stacks. Next step is a callgrind run reading the caller tree for realloc, not the flat profile. Worth doing because 4.94 million reallocations on a 1.85 second scan is a large enough line to matter if it has a cheap cause, and the allocator was already about 35 percent of engine work in the exp-051 profile.
