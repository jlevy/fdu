---
type: is
id: is-01kzsa17y59ny39nyjrz9wk8kc
title: "PR #6 provenance: three unaddressed defects found via the merge into PR #5"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies: []
created_at: 2026-08-11T21:00:40.772Z
updated_at: 2026-08-11T21:07:13.933Z
---
Cursor Bugbot reviewed PR #5 after PR #6 was merged into it and found three defects in the incoming provenance work. All three verified by reading the merged code; none are addressed on origin/fsevents-scoped-revalidation as of this writing. Filed rather than fixed in PR #5, because editing that branch's code from another branch would conflict with its author's in-flight work - these should land in PR #6.

R8 (High): Index::provenance documents that a directory reports the composed provenance of its whole subtree, so it is only as trustworthy as its least trustworthy descendant. provenance_of does not implement that: it reads only the directory's own Source, interval promotion, and freshness, never walking children. Confirming evidence beyond the reviewer's: Provenance::combine, the helper written for exactly this composition, has no caller anywhere in crates/fdu/src outside its own unit tests. A directory row can therefore read verified while cached children sit underneath, which is the specific dishonesty the provenance design exists to prevent.

R9 (Medium): observed_at maps Source::Revalidated to scanned_at_ns, the index construction time, while provenance_of's interval promotion returns verified_at from finish_reconcile, the time the sweep actually completed. One reconciliation pass therefore reports two different as-of times for equally verified paths, and the delta-stamped ones report the older, wrong one.

R10 (Medium): the EntryId::ROOT branch of the upsert path sets attrs and bumps the revision, then returns without touching source. The non-root path deliberately refreshes source on an unchanged upsert - its comment explains that this is what lets a consumer clear a stale-value indicator after verification. The root is excluded from that, so root metadata keeps a stale Scanned or Cached stamp after the same session verified it.

## Notes

Cross-referenced against the reviews already on PR #6 on 2026-08-11: R8 is fdu-b1ts (also in the senior code review's Existing Review Context, with a pre-merge requirement to either land composition or narrow the documented contract); R10 shares C1's root cause (root-special apply_upsert never copies applying_source on either path). Only R9 - inconsistent as-of times within one sweep - was missing from PR #6's record, plus the two-Provenance name collision (fdu-2kat) that only exists in the merged tree. Both posted on PR #6 as github.com/jlevy/fdu/pull/6#issuecomment-5258865270 with a suggested fix and test for R9.
