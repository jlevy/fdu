---
type: is
id: is-01kzrwynajzz2k9m8y8a0cgnya
title: "Confidence per value: paint approximate, converge to verified"
kind: feature
status: closed
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies: []
created_at: 2026-08-11T17:12:04.689Z
updated_at: 2026-08-11T19:34:04.565Z
closed_at: 2026-08-11T19:34:04.564Z
close_reason: Superseded by the finer decomposition under epic fdu-wpa0 (fdu-ywa4 types, fdu-fka6 roll-up composition, fdu-c817 Cached-on-load, plus surfacing beads). The design it carried is now in the plan's provenance section including the storage decision.
---
The browser requirement the CLI never had: a slightly stale number beats no number. Reopening a large folder should paint from cache immediately, mark every value approximate, and clear marks as verification confirms them. This is NOT a violation of 'the cache may never silently lie' - the word is SILENTLY, and a labelled cached value is the honest version (the original research already staked this out: fast-but-wrong is a non-goal, fast-and-labelled is a feature). GAP TODAY: an index loaded from a snapshot reports Freshness::Fresh, because the snapshot was complete when written - for a browser painting on load that is exactly backwards, since nothing has been checked since the file was read. DESIGN: Confidence { Verified (stat-checked this session) | JournalConfirmed{as_of} (journal reported nothing touching this subtree; weaker because the Phase 0 spike showed FSEvents omits history without a flag, which G12's periodic sweep bounds) | Cached{as_of} (believed, unchecked, may be too high OR too low) | Partial (walk in progress; strictly a lower bound, can only grow) }. Partial vs Cached is two different UI affordances - monotone 'at least 3.2 GB, counting' vs point-in-time 'about 3.2 GB, as of 2 min ago' - and collapsing them makes a shrinking number look like a bug. Confidence rolls up by MINIMUM through the existing merge_upward path, so a directory is only as trustworthy as its least trustworthy descendant, and it costs an ordered enum in the reducer set. Convergence must be observable, not polled: emit confidence transitions per path, reporting confirmations as well as corrections, since a consumer that only hears about corrections cannot distinguish 'still checking' from 'checked and fine'. Add session.prioritize(path) so verification follows the user's attention rather than sweeping uniformly over rows nobody is reading. WHY THIS MAKES THE JOURNAL WORTH MORE: without it every cached row is equally suspect and clearing indicators means verifying all of them (minutes at home-folder scale); with it a ~200 ms replay names the few directories that could have changed, so almost every row moves from Cached to JournalConfirmed at once and only a handful keep their marks.

## Notes

Renamed and restructured per operator direction: the model is PROVENANCE, not confidence - three orthogonal facts per value rather than one enum. Provenance { source: Scanned|Revalidated|JournalConfirmed|Cached, observed_at, complete }. Splitting complete out of source keeps the two UI affordances distinct (monotone 'counting' vs point-in-time 'as of'). Rolls up by weakest/oldest/AND through merge_upward. Surfaces everywhere: Report rows, all four formats, Python, CLI ('as of' header + per-row markers, quiet when everything is freshly observed). Principles updated across AGENTS.md, the research goal 7, and the composable-CLI plan (now 12 principles): trade speed for certainty in the open never in secret; benefit from the OS never depend on it; platform APIs bound uncertainty rather than compute answers.
