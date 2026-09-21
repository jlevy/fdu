---
type: is
id: is-01m32dphqr330wwjakmf2jrwm8
title: Adopt cargo-mutants in CI rather than a hand-maintained mutant registry
kind: feature
status: open
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T16:45:00.023Z
updated_at: 2026-09-21T17:06:20.331Z
---
Today's H138 guard was written specifically to catch a regression and was algebraically incapable of failing: `both < types * 2` expands to `F + 2W + 2A < 2F + 2W + 2A`, true whenever the fixed per-report overhead F is positive, however many walks ran. A never-share build measured 10,265 against that bound of 10,268 and passed. CI was green and proved nothing.

It was only caught by disabling the optimization and re-running. Nothing in the repository requires that step, so the next such guard can ship inert the same way.

Proposal: a small named-mutant harness, not full mutation coverage. Each entry names a mutation (a one-line source edit disabling a specific optimization) and the test that must fail under it. The harness applies each mutant, runs the named tests, and fails if any of them still passes. Roughly the shape of the path-independence registry: deny-by-default, and an entry whose test no longer fails under its mutant is itself a failure, so a guard cannot quietly go inert.

Seed it with the two that exist today:
- `row_consumers > 1` -> never share, must fail `unfiltered_metric_views_share_one_every_entry_walk`.
- `row_consumers > 1` -> always share, must fail `bounded_single_file_view_does_not_clone_every_materialized_path`.

Related rule worth stating alongside it: an allocation guard must compare against an independently measured quantity, never a multiple of one of its own measurements. The fix for H138 measures `[Families]` as a third observation and asserts `types + families - both >= FILES`; the sharing saving is 2,054 and a never-share build saves 3.

## Notes

SUPERSEDED BY REVIEW, 2026-09-21. The named-mutant registry proposed here was rejected on three grounds, all of which I accept:

1. cargo-mutants would have found the inert H138 guard unaided. It substitutes `>` with `<`, `==`, `>=` and `&&` with `||`, so `row_consumers > 1` becomes `row_consumers < 1`, which is never-share. The old guard passing would have been reported MISSED without anyone having to suspect it. A hand registry only ever checks guards somebody already distrusts — which is precisely the guard that does not need checking.
2. Rot vectors the proposal omitted: a mutant is a textual edit, so a rename makes it silently not apply. "Did not apply" and "did not compile" must both be failures; one mutant in the review did not compile (`walked.is_none()` alone trips the unused-variable deny). Entries would also need a base-commit and file:line anchor.
3. Measured cost on this host, hot cache: one-line core mutant rebuilds the test binary in 2.6-4.1s, the command line incrementally in 2.3-2.8s, cold build 34-43s.

Adopt instead: `cargo mutants --in-diff` on pull requests plus a scheduled full run over `fdu-core`, scoped with `--re` / `-- --test`. CI only, never `make check` — a timing-shaped gate on a shared runner measures the runner, and the rebuild cost per mutant is real.

Pin the tool under the 14-day cool-off as SUPPLY-CHAIN-SECURITY.md requires; it would be a new bootstrap entry.

The companion rule stands and is now implemented on PR #104: an allocation guard compares against an independently MEASURED quantity, never a multiple of one of its own measurements. The H138 guard now measures the fixed per-report overhead with a zero-view report and subtracts it, rather than assuming it negligible.
