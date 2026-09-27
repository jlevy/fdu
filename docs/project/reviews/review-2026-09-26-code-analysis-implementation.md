# Code Analysis Implementation Review

## Scope and Verdict

Review of the uncommitted implementation against research-plan revision
`4016fd99827a50af96d977798f5d590dc8525fb6`, covering the Rust engine and command line,
Python bindings and value models, shared test harnesses, and documentation.
The review followed the tbd precommit and code-review shortcuts.

**Verdict: implementation findings addressed; integration validation passed.** The
findings below were discovered and corrected during review, before release.
The [testing architecture review](review-2026-09-26-code-analysis-testing.md) records
which evidence each test layer owns and the remaining verification work.

## Findings

### R1 — High: Unknown Ignore Classification Was Treated as Non-Ignored

`crates/fdu-core/src/index.rs:3879`. An unreadable or refused control source can hide
both ignore rules and negations.
Treating the affected entries as known non-ignored would misstate population totals and
could analyze content an excluded-population request did not authorize as a member.

**Fix:** Retain the unreadable-control boundary across refresh, expose unknown
classification, withhold narrow-population membership until rules are known, and clear
that boundary only on a successful owning observation.
**Status:** Addressed, with recovery and renewed-pruning checks plus unknown report
annotations.

### R2 — High: Custom Metadata Paths Could Share an Analysis File

`crates/fdu-core/src/cache.rs:59`. Removing arbitrary filename extensions to derive
sidecars caused distinct metadata paths to collide.

**Fix:** Use the conventional `.metadata.bin` / `.analysis.bin` pair for conventional
names and append `.derived.bin` to the full spelling of arbitrary metadata paths.
Keep the two sidecar suffix namespaces disjoint.
**Status:** Addressed, with explicit collision and cleanup checks.

### R3 — Medium: Ignored-Only Analysis Reported False Incompleteness

`crates/fdu-core/src/index.rs:4028`. A narrowed index retains control files for
reconciliation even when they are outside the requested analysis population.
Counting every retained file as an analysis candidate made a complete ignored-only
report exit with partial status.

**Fix:** Derive pending analysis from the same eligibility rule as candidate
enumeration. **Status:** Addressed; core and command-line checks cover cache off, auto,
and only.

### R4 — Medium: Share Filtering Hid Rows Without an Omission Notice

`crates/fdu-core/src/query/query_report.rs:936`. Grouped metric, language, and extension
views applied minimum-share filtering before recording row-limit totals, leaving the
filtered rows unreported.

**Fix:** Record `share_omitted` separately from row-cap bounds and name the correct
remedy for each in human output; expose the count on every public surface.
**Status:** Addressed with composed share/row bounds and shared golden output.

### R5 — Medium: All-Cache Python Operations Depended on a Scan Root

`crates/fdu-py/src/lib.rs:1185`. The parity adapter resolved the cache directory through
a root-specific cache filename, so a missing root prevented an operation whose scope was
all caches.

**Fix:** Expose the engine’s root-independent directory resolver through Python and use
it for all-cache operations.
**Status:** Addressed, with shared lifecycle scenarios and a public API test.

### R6 — Medium: New Cache Resolution Preceded Python Request Validation

`crates/fdu-py/src/lib.rs:974`. Propagating canonical-root errors from cache-path
resolution caused an invalid request against a missing root to fail with a filesystem
error before its argument error.

**Fix:** Validate the complete logical request before constructing delivery settings
that resolve filesystem paths.
**Status:** Addressed with a missing-root/invalid-query regression check.

### R7 — Medium: Watch Persistence Tests Kept the Old Filename Assumption

`crates/fdu/tests/watch_persistence.rs:36`. The fingerprint helper looked for `.fdu`
files, so it could not observe a newly named snapshot.
A failed warm-up assertion could also leave a watcher running.

**Fix:** Recognize committed metadata snapshot names and give spawned watchers scoped
kill-and-wait cleanup.
**Status:** Addressed; all three persistence tests pass.

## Design Assessment

The engine owns population admission, measurement, projection, cache paths, and report
models. The command line and Python translate the same request and consume the same
report. Population selection affects retained scope; display limits do not.
This keeps pruning, cache identity, and user-visible totals aligned.

The defaults use root-relative shares and a depth bound without an arbitrary breadth
cap. Explicit depth, breadth, share, and row limits remain independently composable.
Unknown ignore classification and missing analysis stay explicit instead of becoming
zero-valued measurements.

A separate scan-ignored flag would duplicate the population decision.
A full parser or replacement counting library would add cost without a demonstrated
accuracy gain on the adjudicated corpus.
The
[conditional evaluation](../research/evidence/codebase-analysis-conditional-2026-09-26.json)
records why classification expansion, a counter-library replacement, and branch-point
metrics are deferred.

## Documentation

The implementation updates the plan, usage reference, machine-output schemas, cache
layout guide, architecture notes, Python README, embedded CLI skill, and changelog.
The cache documentation describes the current destination and file roles; no migration
from the previous alpha layout is promised.

## Confirmed Benign and Remaining Limits

- Focused lexer/chunk-boundary, allocation, concurrency, and cache-failure tests are
  retained because a shorter public golden would not prove the same contracts.
- Custom metadata filenames use a distinct sidecar suffix intentionally; application
  cache enumeration recognizes only its own conventional names.
- The performance comparison measures cold and seeded-cache delivery of the same
  request. It does not establish a before/after metadata-performance improvement.
- macOS runtime checks do not establish Linux or Windows runtime behavior.
  Cross-compilation and CI provide separate evidence.
- The Python parity artifact must be recorded on Linux; it is not regenerated locally.

## Verification Results

The complete local `make check` passed in 870.17 seconds on the integrated revision.
Apple and Windows cross-lint passed; all 19 jobs in
[CI run 36291040472](https://github.com/jlevy/fdu/actions/runs/36291040472) passed.
The reviewed Linux parity artifact matches the local replay, and the full CLI/Python
path-independence matrix passed 16,787 cases without unregistered differences.
[PR #133](https://github.com/jlevy/fdu/pull/133) contains the implementation above the
research-only PR #130. No release or tag was created.

## Verification Checklist

- [x] Review implementation and fix actionable findings.
- [x] Add the testing-review phase and dependency-linked beads.
- [x] Complete the final local handoff gate and platform checks.
- [x] Review Linux-recorded parity evidence and finish CI.
- [x] Update plan completion, commit, publish the implementation PR, and sync beads.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
