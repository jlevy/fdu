---
type: is
id: is-01m0wqh3nwzjz4naa9rap02sq5
title: Clock cap-refused upserts that mutate existing index state
kind: bug
status: open
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5020603690
    at: 2026-08-25T15:10:51.574Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#issuecomment-5412701379
    at: 2026-08-25T15:25:12.692Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5021835489
    at: 2026-08-25T17:17:54.828Z
labels:
  - pr47-review
  - metabrowser
dependencies: []
parent_id: is-01m0vx6yw0f8bddcwggvk2ha0p
created_at: 2026-08-25T15:09:57.307Z
updated_at: 2026-08-25T17:17:54.829Z
---
At PR #47 exact head d58d9c5036818f33fe390c31453eb7548ba7abfa, cap-refused mutations now advance the clock, but AppliedDelta is inaccurate. apply_upsert returns true when ensure_dir_chain created ancestors or upsert_beneath removed a kind-changing row; apply_validated_with then appends the original observed Op::Upsert to effective. That leaf file was refused and is absent, while the actual directory insertions or removal are omitted. The public journal therefore claims an effective mutation that did not happen and loses the mutations that did. The new tests assert only that since().deltas is nonempty, so they accept this false delta and do not exercise WatchBatch. Return a structured apply result separating refusal from the exact effective operations (or an explicit dirty/invalidation carrier that is not documented as replayable ops), and preserve the actual created-directory/removal paths under the same clock. Assert the complete delta contents, absence of the refused upsert, exact terminal state clock, WatchBatch dirty/state, and tally conservation for first and later refusals.

## Notes

Answered at the branch head after the fourth review round.

Mutation and refusal are separate facts now, which is the second of the two
shapes the review offered. Preflighting was the wrong one: the cap is consulted
where a new *file* row would be allocated, and by then two things have happened
that must not be undone. ensure_dir_chain has created the ancestors of a deep
path -- directories are deliberately admitted whatever the cap says, so the tree
stays navigable to what is already there -- and upsert_beneath has removed a
kind-changing row at the path itself, because a directory replaced in place by a
file is one event and the old row cannot survive it. Refusing the pair would leave
the index describing a directory that is not there, which is worse than either
alternative.

What was wrong was the report. upsert_beneath returned false on refusal
regardless, so those rows reached the index with no delta naming them and no data
clock moving past them: a consumer resuming from its cursor was current on a tree
that had rows it had never been told about. ensure_dir_chain returns (EntryId,
bool), upsert_beneath tracks the removal it performed, and apply_upsert reports
either half as a change.

Tests, both cases and the second refusal as well as the first:
a_refused_upsert_reports_what_it_changed_on_the_way_to_refusing (a deep path under
a full cap, twice, asserting the clock moves, the directories are in the roll-up,
since() is non-empty, and the cap still holds at 2) and
a_kind_change_refused_by_the_cap_still_reports_the_row_it_removed (an *empty*
directory replaced by a file, deliberately empty because a directory with files in
it frees room under the cap and is a different case).

The second fixture needed care: a capped walk stops *discovering* at the cap, so a
fixture that starts full never reads the directory at all. It opens with one file
under a cap of two, then fills the cap by refresh.

Four mutations, all caught: refusing reports unchanged again, the kind-change
removal untracked, the chain's mutations dropped at the caller, and building an
ancestor not counted as a mutation.
