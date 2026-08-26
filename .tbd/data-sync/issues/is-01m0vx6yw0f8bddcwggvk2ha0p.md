---
type: is
id: is-01m0vx6yw0f8bddcwggvk2ha0p
title: "A native walk budget: stop discovery at the cap, and say so"
kind: task
status: open
priority: 1
version: 19
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
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
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5019372007
    at: 2026-08-25T13:21:14.058Z
  - kind: other
    url: https://github.com/jlevy/fdu/actions/runs/32851927452/job/97814746382
    at: 2026-08-25T13:22:46.188Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#issuecomment-5411017628
    at: 2026-08-25T13:22:59.790Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5020603690
    at: 2026-08-25T15:10:54.510Z
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
child_order_hints:
  - is-01m0wqh3nwzjz4naa9rap02sq5
created_at: 2026-08-25T07:30:01.728Z
updated_at: 2026-08-26T07:01:50.826Z
closed_at: null
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

FDU47-E2 addressed at 353d48f: a cap refusal is now one event with three faces
that agree at one clock.

The gap was real and was mine. Index::apply marked Partial(Budget) atomically
but merge_apply_stats dropped `refused` and ReconcileReport::is_complete ignored
it -- so a refresh reported complete while its own terminal index said partial
(budget), two answers about one call from the same operation. And the coverage
loss carried no typed issue, so a consumer mapping resource exhaustion to its own
RESOURCE_BUDGET state had nothing to map.

Now: the count reaches the merged report, is_complete counts it for the same
reason it counts `stale` (the sweep finished and the index does not hold what it
found), coverage() names the cap ahead of whatever the walk itself reported
(because it is the reason the *index* is short rather than a reason the walk
was), and the refusal commits the coverage transition and the RunFacts
transition in one delta.

Reported once per scope rather than once per refusal -- a long watch over a full
tree refuses on every arrival -- and that follows from the guard rather than a
second check inside it. An `already_reported` branch there would be unreachable
given the guard, and unreachable defensive code reads as a hazard that does not
exist.

Tests: a_refusal_reports_a_count_a_coverage_and_a_typed_issue_together and
a_repeated_refusal_reports_one_issue_rather_than_one_each. Mutations checked:
`refused` dropped from the merge, from is_complete, and from coverage; the typed
issue and completeness loss withheld; and the exactly-once guard removed. Each
fails a named test.

Also fixed here: the Windows CI red on the new max-files golden was fixture
setup, not cap enforcement -- `sh -c 'printf x > a; printf x > b; printf x > c'`
produced one 3-byte file there, so the uncapped control saw one file too.
Replaced with the repository's standard node writer.
