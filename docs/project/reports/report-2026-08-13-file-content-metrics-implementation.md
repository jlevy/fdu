# File-Content Metrics Implementation Report

**Date:** 2026-08-13\
**Status:** implemented and locally validated

## Outcome

fdu now supplies one opt-in capability ladder over its existing index, query, cache,
CLI, Rust, and Python surfaces:

1. Metadata-only scans classify exact names and extensions and roll up apparent and
   allocated bytes without opening file content.
2. `content-basic-v1` streams UTF-8 once for physical, blank, and nonblank lines plus
   raw prose words, while reporting binaries, invalid encodings, oversized inputs, I/O
   failures, races, and unsupported analyzers as coverage rather than zeroes.
3. `code-sloc-v1` partitions every physical line into code, comment, or code blank for
   Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin,
   shell, and SQL.
4. `text-logical-v1` supplies FlexDoc-style normalized volume, paragraph runs, and
   aggregate-derived pages; `markdown-prose-v1` measures reader-visible CommonMark while
   excluding destinations, code, metadata, footnote markers, and hidden markup.
5. Bounded deep detection handles unresolved and ambiguous inputs with shebangs,
   modelines, required C++ literals, XML and manpage markers, binary signatures, and
   generated/vendor/documentation flags.

The default remains metadata only.
Content reads are bounded by file size and worker count, use the existing file handle,
commit only after fingerprint revalidation, and persist in a separately versioned
sidecar. Metric summaries use `fdu.report/2` and expose exact shares, analyzer
identities, coverage, detection source and confidence, and origin flags in human, JSON,
JSONL, YAML, and Python output.

## SCC Parity and Deliberate Differences

fdu matches the part of SCC that is straightforward to make an additive, incremental
contract: physical/code/comment/blank partitions, mixed code-and-comment lines counted
once as code, language summaries and percentages, common newline conventions, binary
gating, and a bounded ambiguity path.
The exact dialect is versioned rather than presented as parser-perfect source semantics.

fdu goes beyond SCC for this product by retaining per-directory and per-type rollups,
unknown and binary byte coverage, warm per-file content reuse, report provenance, and
text/document volume with markup-aware word and page estimates.

It does not yet match SCC’s long-tail breadth: the reviewed SCC master had 365 language
definitions, while `code-sloc-v1` intentionally supports 15 common languages.
Complexity, COCOMO, unique-line sets, per-byte classification, embedded-language
attribution, AST metrics, statistical language detection, and specialized markup beyond
Markdown remain separate future analyzers.
Those features are harder because they add grammar breadth, non-additive state, linear
memory, or semantic ambiguity rather than another cheap tally in the existing pass.

## Performance Evidence

On the local M1/APFS host, complete CLI comparisons over an immutable 233-file, 3.18 MB
self-host archive measured fdu at 11.9 ± 0.4 ms, SCC 3.7.0 at 9.7 ± 0.5 ms, and Tokei 14
at 13.3 ± 0.9 ms. On a generated 7,500-file common-language tree, the corresponding
figures were 108.9 ± 4.2 ms, 90.9 ± 1.6 ms, and 111.2 ± 17.5 ms.
fdu was close to Tokei and about 20–23 percent behind SCC while also building its
reusable metadata/content index.
These are host-specific product-scale checkpoints, not universal speed claims.

Plain-text, Markdown, and self-host document jobs measured initial passes near 85.0 ms,
150.9 ms, and 18.0 ms, with compatible sidecar loads near 19.3 ms, 20.9 ms, and 6.6 ms.
The accepted UTF-8 chunk iteration improved constrained Markdown wall time by 12.04
percent and analyzer component time by 13.67 percent while preserving every semantic
digest.

The isolated classifier performed 100,000 resolved decisions in a 37.085 ms median and
ambiguity-maximizing decisions in 42.579 ms, or about 371 and 426 ns per decision.
The roughly 55 ns deeper-tier increment remains off the ordinary resolved path.

## Compatibility and Validation

The repository-wide `make check` gate passed after implementation.
It included supply-chain policy, formatting, clippy, all Rust feature combinations, 326
all-feature library tests, CLI and watch integration tests, 92 exact tryscript
scenarios, 63 performance contract tests, documentation, audits, public hygiene, MSRV,
and installed-wheel smoke.
The self-host check analyzed 278 tracked files and reported 80,487 text lines, 33,086
standard LOC, and 10 types while satisfying every rollup and coverage invariant.
Cache-only and refreshed Python indexes also preserve partial content status,
diagnostics, and coverage when an invalid source is served entirely from its sidecar.

Python 3.12 remains the minimum and the stable native ABI floor.
The gate built `fdu-0.0.1-cp312-abi3-macosx_11_0_arm64.whl` with CPython 3.12, then
installed and exercised that wheel under CPython 3.14.6. No Python 3.13 scope increase
was needed.

The detailed evidence remains split into the
[research brief](../research/research-2026-08-12-fast-file-content-metrics.md),
[implementation plan](../specs/done/plan-2026-08-12-fdu-file-content-metrics.md),
[SLOC engine decision](report-2026-08-13-code-sloc-engine-decision.md),
[SLOC performance checkpoint](report-2026-08-13-code-sloc-performance.md),
[Markdown parser decision](report-2026-08-13-markdown-parser-decision.md),
[document performance report](report-2026-08-13-document-metrics-performance.md), and
[deep-detection report](report-2026-08-13-bounded-deep-detection.md).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
