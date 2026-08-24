---
type: is
id: is-01m0rm411csthct7rcayfhs1y0
title: Price the leaf-count field that already shipped on the ancestor-merge path
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0racd5dxjfx1g5e0dsfay8q
created_at: 2026-08-24T00:53:25.163Z
updated_at: 2026-08-24T15:23:03.474Z
---
fdu-5hip added `others` to InternedRollUp, so every ancestor merge now carries one more
u64 add and every unmerge one more saturating_sub. That shipped WITHOUT a measurement.

Why this is its own bead rather than part of fdu-n4gn: n4gn prices the reducer union --
planes, groups, composed provenance, leaf counts -- and is blocked until the planes exist
to be measured. Leaf counts are not blocked; they are already in the hot path. A shipped
unmeasured change should not wait on an unrelated dependency to be priced.

The contract's own amendment warns about exactly this ordering: "a cost acceptable for
each alone can be wrong in combination", on the ancestor-merge path exp-064 took from
43.73% to 14.07% and campaign 2 plans to delete rather than tune. Accreting members of
the union ahead of the measurement that is supposed to choose the representation is the
risk; this bead bounds the part already taken.

WHAT TO DO: `make perf-compare` on a real tree, interleaved and paired, against the
commit before fdu-5hip landed. Record with `make perf-record` including a null result --
the expectation is that one u64 per merge is unmeasurable, and an unmeasurable result is
worth recording precisely so the next person does not re-run it. Republish with
`make perf-ledger` and `make perf-report`.

NEEDS A QUIET HOST. Run on a shared CI runner it measures the runner, which is why no
timing gate is in `make check`. State platform, host (bare metal or virtualized) and
cache state with the number.

DECIDED 2026-08-24: keep `others`. Reverting was considered and rejected -- the
implemented provider contract explicitly requires roll-up leaf counts, so removing the
field churns a required surface to un-ship one u64 add per merge. The measurement stands
as the open obligation: run on ANY quiet host as soon as one is available, or fold into
fdu-n4gn's paired run when the planes exist -- whichever comes first; do not wait for
n4gn on principle.
