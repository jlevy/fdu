---
type: is
id: is-01m0g41kjm8d3zdsyvpzx69tzx
title: "Salvage the still-useful parts of PR #27 before closing it"
kind: task
status: in_progress
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-08-20T17:38:33.170Z
updated_at: 2026-08-20T20:51:33.655Z
---
PR #27 (codex/performance-research-white-paper) is superseded as a *report* by PR #36, but it is not fully subsumed. Reviewed 2026-08-20; these carry real value and are not in main:

1. revision_series.py plus 'make perf-replay-revisions' (354 lines). Archives each commit, builds the same release probe with one toolchain, and measures every revision in one interleaved run. This is the principled way to build an absolute history: PR #36's absolute figure only exists because someone happened to run four cumulative checkpoints by hand, and cannot be extended or re-derived. Highest-value item.

2. perf-compare measuring five jobs instead of two. main measures only cold-scan-index and warm-revalidate, so a comparison run today cannot reproduce the five-row absolute figure the historical runs produced. One-line Makefile change.

3. '--limit all' in scripts/content-selfcheck.mjs. The script asserts seven tracked type ids are present in the types view; without the flag the default limit of 10 truncates. On the current tree it passes by exactly one row - 'toml' is the 10th of 32 - so one new file type larger than the TOML makes the gate fail with a misleading 'missing tracked file type' error.

4. The Checkpoint contract (kept_variant, profile, source_revision) plus source_revision on Variant in measure.py. PR #36 derives kept-variant from the verdict instead, which is right for the report, but the recorded source revision is what makes item 1 auditable rather than a re-run.

5. softschema_table.py (221 lines): a generic soft-schema directory to table projection, reusable beyond this report.

6. README rename of report-2026-08-12 from 'the white paper' to 'the performance architecture', which is what it actually is.

Item 3 already applied in PR #36's follow-up? No - track separately. Close PR #27 only after these are landed or explicitly dropped.
