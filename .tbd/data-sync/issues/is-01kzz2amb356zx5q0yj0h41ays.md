---
type: is
id: is-01kzz2amb356zx5q0yj0h41ays
title: record.py silently picks an arbitrary comparison when a run has several
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01kzz29dspd7bsy6jk98mpb9z3
created_at: 2026-08-14T02:41:26.371Z
updated_at: 2026-08-14T03:00:06.090Z
closed_at: 2026-08-14T03:00:06.090Z
close_reason: "Split _selected_comparison out of _headline and made every ambiguous case an error surfaced through parser.error: a sweep run with several comparisons and no variant names, only one of the two variant flags, and a named pair the run does not hold. A job with no comparison at all stays a null change, which is what a baseline recording needs. Six regression tests."
---
benchmarks/realtree/record.py _headline documents the hazard exactly ('taking whichever one came first would report the wrong pair') and then falls through to a loop that does precisely that whenever --control-variant and --candidate-variant are not both supplied.

Three ways to record a wrong or empty number without an error: a sweep run with several comparisons and no explicit variants returns whichever pair iterates first; supplying only one of the two flags silently falls through to the same loop; and a named pair that does not exist returns None, recording change_pct: null as though the experiment were non-comparative.

PR #4's R12 fix rejects all three. Port it as a focused change to _headline, without the environment-matrix guardrail machinery that surrounded it on that branch.
