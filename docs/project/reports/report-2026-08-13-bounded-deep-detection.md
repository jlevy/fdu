# Bounded Deep-Detection Report

**Date:** 2026-08-13\
**Status:** implemented and validated\
**Scope:** file-type cascade, classification evidence, cache persistence, and isolated
classification cost

## Outcome

fdu now has a bounded second classification tier without putting regexes, parsers, or
statistical models on the ordinary resolved path.
Exact filenames and ordinary extensions remain path-only decisions.
Only an unresolved file or the explicitly ambiguous `.h` extension can invoke deeper
type detection, and every rule reports its source and confidence.

The shipped bounded rules are:

- 200 bytes for shebangs;
- 1 KiB for Emacs and Vim modelines;
- 16 KiB for C/C++ header literals, below the plan’s 20 KiB ceiling;
- first-byte signatures for XML, manpages, PDF, PNG, gzip, and ZIP; and
- 2 KiB for conventional generated-file markers.

Path-only vendor and documentation flags use component and basename comparisons.
The report projection aggregates detection-source, confidence, generated, vendored, and
documentation counts for every type or family row.
The independently versioned content sidecar persists the same fields at format version
4, so cache-only output retains the evidence.

## Named Consumers

The rules are not free-floating guesses.
The C/C++ decision selects the matching `code-sloc-v1` state machine; a modeline can
route an otherwise unknown source file to an existing code or document analyzer; and a
named binary signature terminates text analysis as soon as the prefix is recognized.
Grouped JSON, JSONL, and YAML output consumes every provenance and flag field.

HTML, notebooks, reStructuredText, and other mixed formats deliberately do not inherit
Markdown projection rules.
They retain basic text metrics where applicable until a separate analyzer ID, version,
fixtures, and performance evidence define each projection.
AST metrics, exact tokenizers, embedded-language metrics, and per-byte classifications
remain separate future analyzers for the same reason.

## Classification Cost

Two committed `fdu-perf-probe` jobs isolate path-only resolution from a corpus designed
to maximize bounded probes.
Both jobs perform 100,000 release-build classifications after scan setup and publish the
component duration under the existing validated performance schema.

Ten direct warm repetitions on the local M1/APFS host produced:

| Job | Median for 100,000 | Median per decision | Observed range |
| --- | ---: | ---: | ---: |
| `detect-resolved` | 37.085 ms | 370.8 ns | 36.822–38.438 ms |
| `detect-ambiguous` | 42.579 ms | 425.8 ns | 42.124–45.692 ms |

The ambiguity-maximizing mix was 14.8% slower, or about 55 ns per decision, than the
resolved mix. This is a rough component microbenchmark rather than an end-to-end
throughput claim; filesystem reads dominate content-analysis runs, and host contention
can distort whole process wall time.
The important result is that the measured penalty is isolated to candidates that need
the deeper tier.

## Validation

Unit fixtures cover positive, negative, and beyond-bound ambiguity cases; modelines;
signatures; generated/vendor/documentation flags; named analyzer routing; early binary
coverage; report aggregation; and sidecar round trips.
The end-to-end tryscript fixture pins the exact detection maps and flag counts across a
multiformat tree. All pre-existing content goldens continue to pin line, word, page,
SLOC, coverage, and cache behavior.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
