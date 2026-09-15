---
type: is
id: is-01m2h7jd6sqqr8n05g4375s6yt
title: Write the 0.1.0 CHANGELOG section and GitHub release notes
kind: task
status: open
priority: 1
version: 2
labels:
  - release
dependencies: []
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-15T00:30:47.512Z
updated_at: 2026-09-15T20:14:06.289Z
---
Found by the 2026-09-14 release-readiness audit (scratchpad reviews/release-readiness.md, section 5 has the outline). CHANGELOG.md has only [Unreleased] on main (102 lines; 168 at #57's head, 177 at #60's head) and it is a development log: 'Initial scaffold', and several 'Breaking:' entries (journal_capacity -> journal_capacity_bytes, Result-returning ignore accessors, per-projection refusals, portable selection identity, removed gitignore build feature) that describe changes to unreleased contracts. Per tbd guidelines release-notes-guidelines and release-engineering-rules: a first release has no previous release, so nothing is a Fix or Breaking; fold those entries into descriptions of the contract as it stands, and the pre-release gate requires the notes before tagging. Deliverables: (1) '## [0.1.0] - <date>' in CHANGELOG.md, Keep a Changelog form, almost entirely Added, keeping the Known limitations list updated (control bounds, one-shot content analysis, memory vs dust fdu-syyl, no cache pruning beyond --cache-clear); (2) GitHub release notes draft: what fdu is, install commands, features by surface (CLI six axes and the view vocabulary summary/tree/types/extensions/families/languages/documents/largest/recent/files/full which the CHANGELOG never recorded; fdu.report/4 and /5 and fdu.stream/1 and what a consumer pins; exit codes; Python fdu and fdu.opened; Rust fdu-core with watch as the one build feature and read_controls as the per-request switch; .gitignore semantics honoured and not), defaults worth knowing (CLI does not read .gitignore in 0.1.0 while fdu.open does, and read_controls=False), the 'upgrading from a development build' note from fdu-apbl (type-rules and ignore-rules fingerprints moved; first run per cached tree cold-scans; --cache-clear and --cache-status=all; confirm on the release candidate that a pre-#57 v3 snapshot is refused), platform notes (wheel matrix, Linux arm64 structurally validated only, abi3 3.12+, no free-threaded wheel, MSRV 1.85, macOS getattrlistbulk, Linux evidence virtualized, Windows no perf claims, 0600 snapshots), performance claims only as the evidence report supports and dated, SECURITY.md pointer, compare link. Close fdu-apbl with the note's location when done. Acceptance: CHANGELOG has a [0.1.0] section; release notes reviewed by the user before tagging; every item describes the product as released, none the path to it.

## Notes

2026-09-15 DRAFT EXISTS: draft PR https://github.com/jlevy/fdu/pull/64 (branch claude/release-notes-0.1.0, commits 4abb1aa and f848280 on main f047dab). Kept as a draft until PR A (control bounds degrade) and PR B (.gitignore on by default) merge.
- CHANGELOG.md: [Unreleased] consolidated into '## [0.1.0] - YYYY-MM-DD' with Added (by surface), 'Upgrading from a pre-release build', and 'Known limitations'. The dev-log entries are folded into the shipped contract.
- docs/project/release-notes/0.1.0.md: the GitHub release body (summary, highlights, install, platforms, compatibility, limitations, upgrade notes, security, links). docs/project/guides/release-process.md's announce step now passes that file from the tagged clone instead of a hand-copied CHANGELOG section.
- Placeholders are HTML comments: <!-- PR A ... -->, <!-- PR B ... -->, schema-version names, the release date, and <!-- fdu-y5xr --> for the headline performance figure. The PR body lists each one with the bead that decides it, plus a to-confirm list.
- Every factual claim was checked against main f047dab; the PR body lists the sources. Corrections to the readiness outline: MIN_JOURNAL_CAPACITY_BYTES is 64 KiB, not 512 B; --cache also takes read-only; --cache-clear and --cache-clear=all never delete unrecognized snapshots, so they do not reclaim old-format ones.
To finish: resolve every placeholder after PR A and PR B land, fill in the fdu-y5xr figure and the date, have the user review, then mark the PR ready. Close this bead when the final version merges.
