---
type: is
id: is-01m2esgtxk7ed7k7mh039e51m7
title: "Release note: the type_rules_fingerprint change cold-rescans every cached tree on upgrade"
kind: task
status: closed
priority: 3
version: 7
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
delegate: codex@spud10
labels:
  - stack-followup
  - release
dependencies: []
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
hold: null
hold_until: null
created_at: 2026-09-14T01:46:47.090Z
updated_at: 2026-09-16T18:16:55.943Z
started_at: 2026-09-16T18:06:12.439Z
closed_at: 2026-09-16T18:16:55.943Z
close_reason: "PR #64 merged at a74dade1f90d1110e9a5bc3576d66f1ece10d90a with the final 0.1.0 release notes and upgrade guidance; all 19 CI jobs passed."
resolution: null
duplicate_of: null
---
Release-note follow-up from PR #48's description ("The command line is unmoved"): "One consequence deserves a release note."

**What.** The opened-root rewrite changes the type registry, so `type_rules_fingerprint` moves. It is invisible in human output (one `cli-content.tryscript.md` golden records the new value), but on-disk caches are keyed on it. The first run of the new binary on every previously cached tree finds its snapshot mismatched and cold-scans. That is correct, since the fingerprint exists to move when the rules move, but a user sees a silent slow first run after upgrading, and the old snapshots stay on disk.

**Also check before writing it.** Later PRs in the same stack change what a cached snapshot is: #51 and #52 make one-shot reports controls-off, and #52's description notes the historical v2 snapshot format differs from v3. Confirm on the release candidate exactly which previously written caches are still served, so the note is accurate.

**To do.** When the stack reaches a release, write one note saying:
- the first run on each cached tree after upgrading re-scans cold;
- why;
- how to reclaim the stale snapshots, with whatever cache-clearing mechanism exists at that point (fdu-558j tracks that none prunes them today).

Put it wherever release notes live for that release, and close this bead with a link.

## Notes

2026-09-15 DRAFT EXISTS: the upgrade note is drafted in draft PR https://github.com/jlevy/fdu/pull/64. It appears in CHANGELOG.md, under [0.1.0] 'Upgrading from a pre-release build', and in docs/project/release-notes/0.1.0.md, under 'Upgrading from a Pre-Release Build'.
What it says:
(1) The first run on each previously cached tree scans cold and replaces the snapshot if that run saves one, because the snapshot format (PR A: version 4) and the type-rule and ignore-rule fingerprints changed.
(2) fdu does not remove old snapshots. --cache-clear and --cache-clear=all delete only recognized snapshots (cache.rs clear_cache/clear_all_caches); snapshot.rs read_header rejects any other FORMAT_VERSION or engine fingerprint. An old-format file stays until a run on the same root replaces it. 'fdu --cache-status=all --format json' lists it as "recognized": false; text status prints 'No cached snapshots.' when every file is unrecognized. Delete by hand from the cache directory (fdu-m6lr would add --cache-clear=unreadable).
Still owed before closing:
- On the release candidate, run over a tree cached by a build of today's main. Confirm the snapshot is refused (cold scan), not served, and that --cache-clear leaves it. Before PR A's format bump, a dev-build 0.1.0 snapshot is still recognized, since the engine fingerprint mixes only CARGO_PKG_VERSION, FORMAT_VERSION and CLASSIFICATION_VERSION.
Close this bead with the note's location when the final version of PR #64 merges.
2026-09-16 REWRITTEN AND RESOLVED IN TEXT, on PR #64 commit 1f5586d (merge of main 16efcd0 into claude/release-notes-0.1.0).
The note's old second half is obsolete. PR #67 makes --cache-clear and --cache-clear=all remove stale snapshots (another fdu version, another format, or an unreadable header) as well as current ones, so the "delete them from the cache directory by hand" instruction is gone from both documents.
What the note now says, in CHANGELOG.md under [0.1.0] 'Upgrading from a pre-release build' and in docs/project/release-notes/0.1.0.md under 'Upgrading from a Pre-Release Build':
(1) The first run on each previously cached tree scans cold. A snapshot is keyed on an engine fingerprint mixing CARGO_PKG_VERSION, FORMAT_VERSION and CLASSIFICATION_VERSION (crates/fdu-core/src/snapshot.rs:201-211, FORMAT_VERSION now 4 at :64) plus the type-rule and ignore-rule fingerprints that scope it, so the crate version alone moves it at every release. That one cold run per tree is what every upgrade costs, not a one-off.
(2) fdu --cache-clear=all reclaims what that strands, and fdu --cache-status=all names each file as stale with its reason first. Files that are not fdu's are never removed.
The release-candidate check this bead owed is now pinned by a golden rather than owed to a manual run: tests/golden/cli-lifecycle.tryscript.md plants an older-format and an other-engine snapshot through tests/golden/bin/cache-plant.mjs ("exactly the files a release upgrade leaves behind"), goldens --cache-status=all reporting them stale, and goldens --cache-clear=all removing them while leaving unrecognized files in place. crates/fdu-core/src/snapshot.rs::engine_fingerprint_mismatch_discards_the_snapshot covers the refusal itself.
Close this bead when the final version of PR #64 merges.
2026-09-16 NARROWED after review 5225288341 (RN64-5), PR #64 commit 79e8241. Point (1) above overstated it: CARGO_PKG_VERSION was already 0.1.0 in development builds (the -dev+g suffix is only build.rs's --version string), so the crate version does not move between a development build and 0.1.0. What decides is the format and the scope. Both documents now say: a snapshot written before format 4, or under different type rules or .gitignore settings, is not served and that tree scans cold once; a snapshot a development build wrote in format 4 under the same settings can be served warm; the version moves at every release, so each later upgrade costs one cold run per cached tree. Point (2) is unchanged. The rename list under the same heading was removed (RN64-4, 5eb497b). Still close on merge.
