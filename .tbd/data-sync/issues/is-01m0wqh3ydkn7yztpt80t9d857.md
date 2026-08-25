---
type: is
id: is-01m0wqh3ydkn7yztpt80t9d857
title: Make live one-filesystem admission fail open portably
kind: bug
status: closed
priority: 2
version: 8
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5020603690
    at: 2026-08-25T15:10:52.875Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#issuecomment-5412701379
    at: 2026-08-25T15:25:13.078Z
  - kind: other
    url: https://github.com/jlevy/fdu/actions/runs/32865588065/job/97860038258
    at: 2026-08-25T16:06:42.801Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#issuecomment-5413222445
    at: 2026-08-25T16:06:42.802Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5021788526
    at: 2026-08-25T17:13:14.659Z
labels:
  - pr47-review
dependencies: []
parent_id: is-01kzqn502680awzhvddzntq32d
created_at: 2026-08-25T15:09:57.581Z
updated_at: 2026-08-25T17:13:46.035Z
closed_at: 2026-08-25T17:12:18.238Z
close_reason: "Verified complete at PR #47 exact head e9af881a31243c5c763eff09b2e21ece3a7f5aab. watch::admitted preserves an unavailable root device as None; on_root_filesystem fails open only for that absence or a failed parent stat; the production regression separates missing-root identity from a real parent; the device-comparison test is Unix-gated; and all 19 exact-head CI checks, including Windows, are green."
resolution: null
duplicate_of: null
---
At PR 47 exact head 5eb25743f9b7b1a8626bfd1998a5f5ae5bea0e10, watch::admitted maps an unavailable root device to 0, then on_root_filesystem rejects any successfully statted nonzero-device parent. This contradicts the documented fail-open behavior on transient root stat failure. Preserve absence as Option and bypass only this axis when root identity is unavailable. Exact-head CI has 18 green checks and one Windows failure at https://github.com/jlevy/fdu/actions/runs/32865588065/job/97860038258: scan::tests::the_filesystem_boundary_is_asked_of_the_parent_not_the_entry asserts a real-device distinction on Windows, where device is always 0 and one_filesystem is unsupported. Platform-gate the real-filesystem test or inject a portable device probe, while retaining a separate production test for missing-root-device fail-open behavior.

## Notes

Answered at the branch head after the fourth review round.

on_root_filesystem takes Option<u64>. The value was a u64 with a failed root
stat flattened to 0, whose comment said it fails open and whose behaviour was the
opposite: no real device is zero, so every successfully stat'ed parent failed the
comparison -- and an out-of-scope upsert becomes a *removal*, so a root that
momentarily could not be read would have emptied the index one event at a time.
None admits, and the type is what says so rather than a sentinel.

Also removed rather than kept: the clause admitting a parent whose own device
read zero. It can only arise where the platform has no device identity, and
ScanConfig::validate refuses one_filesystem there before any of this runs.
Restoring it changes no answer any test or platform can produce, which is the
argument for not carrying it -- recorded in the doc comment so it is not re-added.

The real-device test is now #[cfg(all(feature = "watch", unix))], which is the
Windows CI failure. one_filesystem is refused at validation on a platform with no
device identity, and every entry there reports device zero, so "whose device is
consulted" is a question that platform cannot answer: the assertions were passing
or failing on the arithmetic of two zeroes rather than on the rule.

Tests: the_filesystem_boundary_is_asked_of_the_parent_not_the_entry gains the
None and Some(0) cases, and watch::tests::an_unreadable_root_admits_rather_than_
removing_everything covers the production path. Isolating the root's absence from
a parent's took care and the test says how: both admit, so a nonexistent root with
a relative path passes whatever the missing device is flattened to. The op carries
an absolute path, which Path::join resolves to itself, so the parent is a real
directory with a real device while the root is unreadable.

Four mutations, all caught -- flattening to Some(0) at the watcher, absence
refusing instead of admitting, restoring the zero-parent clause (a no-op by
construction, recorded above), and asking the entry's own device.
