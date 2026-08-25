---
type: is
id: is-01m0vx6yw0f8bddcwggvk2ha0p
title: "A native walk budget: stop discovery at the cap, and say so"
kind: task
status: closed
priority: 1
version: 10
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47
    at: 2026-08-25T09:54:14.654Z
  - kind: pr
    url: https://github.com/jlevy/metabrowser/pull/74
    at: 2026-08-25T09:54:14.655Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5017522830
    at: 2026-08-25T09:57:37.582Z
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-25T07:30:01.728Z
updated_at: 2026-08-25T13:04:01.063Z
closed_at: 2026-08-25T13:04:01.063Z
close_reason: null
resolution: null
duplicate_of: null
---
The interactive-client contract declares a file budget as *scope* -- validated as positive,
a component of the scope fingerprint, and enforced by the reference provider, which stops
discovery at it and reports partial coverage with reason `budget` plus a typed
resource-budget issue.

fdu had none. `CoverageReason::Budget` was declared and documented as unreachable, and
`ScanScope` carried depth, symlink, filesystem and hidden-admission facts with no entry cap.
Two providers given the same scope fingerprint therefore returned different inventories:
complete where the other was truncated.

DECIDED (MB74-D2, answered on PR #47 at head `eaae030`): fdu implements the native budget.
The cap stays fingerprinted semantic scope on the consumer side because the shipped
resource stop, partial coverage and typed issue must be preserved, and projection limits
are explicitly not a substitute.

The bound has to stop the *walk*, not truncate an *answer*. A budget that only bounds a
projection leaves the tree read anyway and saves nothing -- the cost it exists to avoid is
the discovery, not the serialization.

What this touches:
- `ScanConfig` gains the cap; `ScanScope` reports it and folds it into the scope
  fingerprint, so a snapshot taken under one cap cannot be reused under another.
- Snapshot `FORMAT_VERSION` and `SERIALIZED_SCOPE_BYTES` move for the new scope field.
- Discovery stops at the cap and marks coverage `Partial(Budget)` from the stopping point
  upward, which is what finally makes that variant reachable.
- A typed `IssueKind::ResourceStop` rides on the run facts, so a consumer sees why rather
  than inferring it from a count.
- `--max-files` on the SCOPE axis and `ScanOptions.max_files`, both surfaces, one judge.

Acceptance: a tree larger than the cap stops reading at it -- proven by counters, not by
the row count, since a truncating projection would pass a row-count assertion; coverage is
partial with reason `budget`; the issue is typed; the fingerprint differs between two caps
over the same tree; and a `--cache only` open under a different cap refuses the snapshot.

## Notes

The remainder the reviewer kept this open for -- capped refresh and watch
semantics -- is closed at ce8d78b, together with fdu-7sou, because one mechanism
answers both.

The strict discovery cap (1b76062) stays and keeps its own job: scan::Budget
stops the walk *reading* at the cap, which is what makes a capped scan cheap.
What it could not do is survive anything after the scan. Reconciliation walks
from the index and never consulted it, so one refresh turned a bounded inventory
into an unbounded one while the scan identity went on claiming a cap; and a watch
was refused outright, which left the cap a scan-only bound.

The index now keeps the cap itself. upsert_beneath refuses a new file row once
the root roll-up reaches max_files -- the one place a new row is allocated, and
the one place the previous state of the path is already in hand. Walk, refresh
and watch are all bounded by one rule, and validate_for_watch_scope has nothing
left to refuse.

Directories are not counted: a directory carries no bytes of its own, and
admitting one keeps the tree navigable to what is there. The refusal and the
coverage loss ride in one commit (AppliedDelta::of_both) rather than two clocks,
because a cursor between them would name a moment at which the index had dropped
an entry and still claimed to cover everything.

Recorded honestly, because it is a property of the axis rather than of this
implementation: which files a long-lived capped index holds depends on the order
events arrived, as which files a capped walk holds depends on the order it
reached them. No rule bounds the retained set and is history-independent at once.
Coverage says the set is short, which is the fact a consumer can act on.

FDU47-C2 asked for one observable rule pinned by a boundary-case fixture. This is
the rule; the fixture is fdu-kl7r's, and the divergence it has to pin is that the
consumer's reference walker gives each subtree rewalk a fresh budget, so its
retained set is bounded per walk rather than in total. One side has to move, and
this is the side that keeps the bound a bound.

Tests: a_refresh_does_not_grow_a_capped_index_past_its_cap and its uncapped
control in walk_budget.rs; a_capped_index_refuses_a_watched_file_past_its_cap,
a_deletion_frees_a_slot_the_next_arrival_can_take, and the live
a_capped_watch_holds_its_cap_against_live_events. Four mutations checked: the
refusal removed, off by one, directories counted, and the refusal not marking
coverage.
