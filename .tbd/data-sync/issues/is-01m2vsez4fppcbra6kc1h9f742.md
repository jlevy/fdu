---
type: is
id: is-01m2vsez4fppcbra6kc1h9f742
title: "H109: control reclassification Path comparison"
kind: task
status: closed
priority: 2
version: 5
delegate: unknown@spud10
labels: []
dependencies: []
parent_id: is-01m2vs96dg6kw3ka6k4ghyvf0f
hold: null
hold_until: null
created_at: 2026-09-19T02:55:53.486Z
updated_at: 2026-09-19T05:54:16.673Z
started_at: 2026-09-19T05:39:15.487Z
closed_at: 2026-09-19T05:54:16.672Z
close_reason: "exp-108: deciding-scale content-cache-hit profile on metabrowser-clone; install_controls 7.2% of profile / 7.5% of engine; Path rewrite not justified; next-up H83"
resolution: null
duplicate_of: null
---
fdu-hzyb. Only after a profile on a deciding controls-bearing subject. Predicted: content-cache-hit wall down >= 3% if the change is a memory-access rewrite like H102, not an instruction trim (exp-104). Gate with control goldens and assert-same-image. Skip if the profile share collapses at deciding scale.

## Notes

exp-107 metadata CLI profile is not H109s instrument. system-private-frameworks one-shot: 0 control reads, 0 same-parent path comparisons, detached walk about 96% of wall. A 667-file --analyze=code pair with no gitignore also showed 0 path comparisons (warm revalidation reused the sidecar).

H109 exp-108 PRE-REGISTER (profile, not a trim):

This experiment is a deciding-scale content-cache-hit PROFILE. It does not test an instruction-only Path rewrite (exp-104 is a dead end).

Metric (named before measuring):
- Attribution: where time goes on the content-cache-hit job at deciding scale, especially install_controls -> reclassify_controlled_subtrees / Path compare_components share vs the exp-104 screening figure (19.43% profile / 25.5% engine on 3k cargo-registry).
- Attachment: wall_ns and peak_rss_bytes from a real 12-pair self-comparison of the same release probe so the profile is not orphaned.

Accept for this profile (not a code-change verdict): tree fingerprint unchanged; sidecar filled once then hit runs measured; content-cache-hit is actually a hit (cache-only open, no analysis reads); Path rewrite is justified only if the deciding-scale hit-path share remains large enough that a memory-access rewrite like H102 could move wall >= 3%. Skip the rewrite if the share collapses.

Subject: nominated metabrowser-clone (live ~/wrk/github/metabrowser). Job: harness content-cache-hit (basic content, CachePolicy::Only) after one content-seed. Isolated scratch snapshot. Sequential processes; harness interleaves the two same-binary arms. FDU_COUNTERS unset on the claim-grade pair; FDU_COUNTERS=1 on a later instrumented hit; sampling profile with counters/oracle disabled for clean attribution. Attempt PERF_HOST_REGIME=quiet; if the cell fails, label uncontrolled. No RAM disk. No engine patch unless the profile names one smallest falsifiable memory-access change and there is time to pair-measure it.

exp-104 dead end stands. README 200K/4M not in scope.
