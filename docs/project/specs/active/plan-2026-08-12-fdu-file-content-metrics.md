# Feature: Fast File Content Metrics

**Date:** 2026-08-12

**Author:** fdu project

**Status:** Draft

## Overview

Add fast, incremental measurements for file types, source code, prose, and markup
without slowing fdu’s metadata-only default path.
The feature begins with byte and file-count rollups that require no file reads, then
adds opt-in content analyzers in increasing order of semantic and implementation
complexity:

1. stable file-type and content-family rollups
2. physical, blank, and nonblank lines plus basic prose words
3. language-aware code, comment, and blank source lines
4. normalized and markup-aware prose measurements
5. bounded deeper detection and specialized formats

Every phase is useful by itself.
Each analyzer has a versioned metric dialect, explicit coverage, independent cache
identity, and additive per-directory rollups.
A user can tell the difference between zero code and code that was not analyzed, and an
analyzer change invalidates only that analyzer’s results rather than the metadata index.
Grouped metric summaries make the same data useful as a GitHub-style language report or
a document-volume report without introducing separate walkers or reducers.

The evidence and prior-art review behind this plan are in
[the fast file-content metrics research](../../research/research-2026-08-12-fast-file-content-metrics.md).
That work compares GitHub Linguist, SCC, Tokei, FlexDoc, and focused Rust libraries, and
includes exploratory whole-tree, in-memory, and binary-gating measurements.

## Goals

- Roll up file counts, apparent bytes, and allocated bytes by stable file type and broad
  content family without opening files
- Keep content analysis explicit and opt-in, with no content reads, analyzer
  allocations, or analyzer worker pool when it is disabled
- Count physical, blank, and nonblank lines accurately for ordinary text while rejecting
  binaries and reporting unsupported encodings as uncovered rather than zero
- Provide a standard code-line rollup that distinguishes code, comments, and blanks for
  common languages and names its parser dialect
- Provide language and document summaries with file counts, applicable metric columns,
  coverage, and percentages over an explicitly named denominator
- Measure prose using raw words, normalized logical words, paragraphs, and fixed-word
  page equivalents without storing rounded page counts
- Exclude Markdown destinations, code, and hidden syntax from reader-visible prose
  measurements
- Resolve most file types from paths and inspect content only for binary safety,
  extensionless files, or genuinely ambiguous candidates
- Cache per-file analyzer results by the existing strong file fingerprint and update
  directory and type rollups by subtraction and addition after changes
- Expose analyzer identity, semantic version, detection source, coverage, and provenance
  through Rust, Python, and versioned machine output
- Measure cold, warm, cache-hit, binary-heavy, and incremental-churn behavior before
  enabling each deeper analysis profile

## Non-Goals

- Make content analysis part of the default `fdu PATH` scan
- Call nonblank physical lines “lines of code”; comments require a language-aware parser
- Embed GitHub Linguist, invoke SCC or Tokei as a subprocess, or let another tool walk
  the tree a second time
- Parse abstract syntax trees, count statements, calculate cyclomatic complexity, or
  provide exact model-token counts
- Identify every format in Linguist’s catalog before shipping useful common-format
  coverage
- Treat a MIME label or binary magic signature as a programming-language detector
- Count minified data, URLs, code fences, or markup syntax as human-readable pages
- Copy GitHub’s byte-based language percentages without labeling the denominator; fdu
  supports byte share but uses code-line share for its default language summary
- Add sentence counts until a versioned boundary rule survives representative prose,
  abbreviation, Unicode, and markup-projection fixtures
- Persist per-byte code/comment classifications; ordinary rollups need counts, not a
  byte-sized label for every source byte

## Compatibility

This is a pre-1.0 feature, but existing metadata users must not opt into content work by
accident.

| Surface | Requirement |
| --- | --- |
| Rust and Python APIs | Keep existing metadata-only calls and defaults; add analysis through typed options |
| Default CLI | Keep `fdu PATH` metadata-only and preserve its tree output |
| Type views | Move the current raw-extension rows to `extensions`; make `types` mean stable detected types; add `families` for broad content families |
| Machine output | Preserve the current schema for unchanged metadata views; use a new schema for stable type/family or content fields |
| Metadata snapshots | Continue reading and writing snapshot v2 without content; do not force a metadata rescan for an analyzer change |
| Content cache | Add independently versioned sidecars; treat absent, corrupt, or unknown sidecars as an analyzer cache miss |
| Database schemas | N/A |

The `types` view change is the one intentional pre-1.0 CLI and schema correction.
Today it is named “types” but reports extensions.
After Phase 1, `--view extensions` preserves that exact grouping, while `--view types`
groups aliases such as `.jpg` and `.jpeg` under one stable type and `--view families`
groups types as code, prose, markup, data, binary, or unknown.
Human and machine migration notes must ship in the same phase.

## Background

The index already maintains invertible per-extension file and byte tallies in every
directory rollup. That answers the cheapest form of “size by type” from metadata alone,
but it does not recognize exact filenames such as `Dockerfile`, group extensions into
stable language or content families, or inspect content.

Content work has different cost and correctness constraints from metadata:

- a metadata scan already knows file size and extension
- even a simple line count must open and read every eligible file
- a mislabeled binary can waste reads and produce millions of meaningless lines
- code/comment separation depends on language strings, comments, nesting, and docstring
  rules
- raw Markdown words include destinations and code that a reader does not experience as
  prose
- unchanged content should not be reread after a warm open

The research spike found that a fused in-memory pass over a Go-like buffer counted
lines, blanks, raw words, and a NUL prefix at approximately 590 MB/s on the reference
host. An SCC-style code/comment state machine ran at approximately 260 MB/s. On a 758 MB
logical corpus of executables mislabeled as `.go`, early NUL gating reduced wall time
from 320.8 ms to 87.9 ms and prevented 3,011,968 invalid source lines.
These are exploratory envelopes rather than portable product claims, but they justify
the order of the phases: fuse cheap counters, reject binary input early, and add syntax
only where its extra work changes the answer.

## Design Principles

### Metadata and Content Remain Separate Tiers

Metadata answers remain available from the current index and snapshot without content
I/O. Content analysis is a derived tier selected by the caller.
Its scheduling, cache, and failures cannot delay or invalidate a metadata-only answer.

The default disabled path must satisfy all of these mechanically:

- no regular file is opened for content
- no analyzer thread pool is created
- no per-entry content allocation is made
- existing metadata-only tree, files, and summary output is byte-for-byte unchanged
- existing snapshot compatibility is unchanged

The index owns an optional boxed `ContentIndex` with sparse per-file results and
per-directory rollups only when at least one analyzer is enabled.
It is not embedded as a new allocation in every metadata entry.

### Classification, Eligibility, and Measurement Are Separate

A file has three independent facts:

1. **Type:** a stable file kind such as `rust`, `markdown`, `jpeg`, or `unknown:.abc`
2. **Family:** `code`, `prose`, `markup`, `data`, `binary`, or `unknown`
3. **Analyzer coverage:** which requested analyzer accepted or skipped the file and why

Type detection does not imply that an analyzer succeeded.
A `.rs` file containing an early NUL remains type `rust`, family `code`, and uncovered
by the text and code analyzers with reason `binary`. An unknown extension remains
visible in byte rollups rather than disappearing into one opaque “other” bucket.

The detection cascade is ordered by cost:

1. exact filename
2. compound extension
3. ordinary extension
4. known binary or text family
5. byte-order mark and early NUL probe
6. shebang for unresolved text
7. bounded ambiguity heuristic for extensions with more than one candidate
8. optional modeline, XML, manpage, generated-file, and statistical rules

Phases 1 and 2 stop after step 5. Later phases add the remaining steps only for
unresolved or ambiguous candidates.

### Metric Slots Are Versioned Contracts

Every stored value is identified by `(analyzer_id, analyzer_version, slot_id)`. The
analyzer owns the slot’s meaning.
Changing how a docstring, nested comment, Markdown code fence, or Unicode character is
counted requires a new analyzer version and new golden expectations.

The initial slots are:

| Analyzer | Slot | Meaning |
| --- | --- | --- |
| metadata | `files` | Regular files in the rollup |
| metadata | `apparent_bytes` | Logical file bytes from stat metadata |
| metadata | `allocated_bytes` | Allocated file bytes where the platform reports them |
| `content-basic-v1` | `physical_lines` | Logical lines, including a final line without `\n` |
| `content-basic-v1` | `blank_lines` | Lines containing only configured line whitespace |
| `content-basic-v1` | `nonblank_lines` | `physical_lines - blank_lines` |
| `content-basic-v1` | `raw_words` | Whitespace-delimited words in eligible prose files |
| `code-sloc-v1` | `code_lines` | Physical lines containing code, including mixed code/comment lines |
| `code-sloc-v1` | `comment_lines` | Comment-only lines under the named language dialect |
| `code-sloc-v1` | `blank_lines` | Whitespace-only lines outside comments |
| `code-sloc-v1` | `physical_lines` | `code_lines + comment_lines + blank_lines` |
| `text-logical-v1` | `logical_words` | Normalized prose-volume units under the pinned logical-word rules |
| `text-structure-v1` | `paragraphs` | Plain-text paragraph runs or projected markup blocks |
| `markdown-prose-v1` | `visible_raw_words` | Raw words after reader-visible Markdown projection |
| `markdown-prose-v1` | `visible_logical_words` | Logical words after reader-visible Markdown projection |

`content-basic-v1` recognizes LF, CRLF, and lone CR as line boundaries, including mixed
conventions in one file.
CRLF is one boundary even when its two bytes straddle input chunks, and a terminal line
boundary does not invent another line.
An empty file has zero physical lines.
A blank line contains only Unicode White_Space code points under the analyzer’s pinned
Unicode table after removing its line boundary.
The implementation keeps an ASCII fast path and decodes only when a line contains
non-ASCII bytes. Raw words are maximal runs separated by Unicode whitespace in validated
UTF-8 prose.

A line containing both code and a trailing comment counts once as code.
Strings hide comment delimiters.
Docstrings, nested comments, generated files, and embedded languages are explicit
dialect rules rather than undocumented exceptions.
Every language analyzer uses the shared physical-line boundaries and blank-line
classification. An adapter that cannot parse lone-CR input directly normalizes only that
uncommon path to LF before parsing; LF and CRLF stay allocation-free.

Pages are a query-time derived value:

```text
page_equivalents = words / words_per_page
```

The default is 250 words per page and is always printed with the result.
Words are summed before division, so directory totals do not accumulate per-file
rounding error and a caller can choose another convention without invalidating cached
analysis. Machine output carries the word numerator and words-per-page denominator as
integers; a rendered decimal is presentation rather than stored state.

### Coverage Is Data, Not a Warning String

Each analyzer rollup carries additive coverage counters:

```text
eligible_files, eligible_bytes
analyzed_files, analyzed_bytes
skipped_binary_files, skipped_binary_bytes
skipped_encoding_files, skipped_encoding_bytes
skipped_too_large_files, skipped_too_large_bytes
failed_files, failed_bytes
```

Zero metrics with `eligible_files == analyzed_files` mean that the analyzer found zero.
Zero metrics with `analyzed_files == 0` mean that it did not measure the eligible files.
Human output summarizes incomplete coverage; JSON and YAML retain every counter and the
bounded per-path errors already used by reports.

When a requested analyzer is incomplete, the report’s overall `complete` value is false
and the CLI follows the existing partial-result exit contract.
Metadata completeness remains independently visible.

### Derived Updates Obey the Delta Contract

Content workers never mutate the index directly.
They produce a conditional analysis observation containing:

- relative path and generation-safe entry expectation
- pre-read metadata fingerprint
- post-read file fingerprint
- resolved type and detection source
- analyzer identities and results
- coverage outcome

The worker opens a file once, takes the pre-read fingerprint from that open handle,
verifies that it matches the accepted metadata entry, reads it, and takes the post-read
fingerprint from the same handle after EOF. The index accepts the result only if the
entry identity, metadata revision, and both fingerprints still match.
Otherwise it discards and requeues the analysis without exposing mixed-version content.

Committing an analysis result subtracts the old per-file contribution from every
ancestor and type rollup, then adds the new contribution.
A metadata update or removal first subtracts and invalidates affected content results.
The committed content change receives the same logical ordering guarantees as other
index changes and is visible to sessions and watchers as a typed derived delta.

### Analysis Is Scheduled Behind Metadata

The metadata producer remains responsible for traversal and stat work.
Accepted regular-file observations enqueue bounded content jobs only when an analyzer is
enabled and no matching cached result exists.
A separate bounded worker pool performs content reads, so slow files cannot stop partial
metadata rollups from becoming available.

The one-shot CLI waits for every requested analyzer before rendering a complete result.
Streaming consumers may read partial content rollups with explicit coverage while jobs
are in flight. Backpressure bounds queued paths and bytes.
Each worker reuses its read buffer and releases buffers above a measured retention
limit.

The basic analyzer is streaming and has constant memory with file size.
Analyzers that require a complete buffer, including the Tokei and Markdown prototypes,
have a configurable per-file buffer limit and report `too_large` when they skip a file.
They never allocate directly from an untrusted file-size declaration.

### Binary and Text Admission Are Conservative

The first 16 KiB chunk is shared by detection and the basic analyzer:

- recognize UTF-8 and UTF-16 byte-order marks
- treat an early NUL as binary unless a supported text encoding explains it
- validate UTF-8 before reporting word metrics
- continue from the same file handle for accepted text
- stop immediately for rejected binary content

Phase 2 counts line boundaries bytewise for accepted UTF-8 and ASCII text.
Every later chunk also checks for NUL; finding one discards accumulated text metrics and
records binary coverage, while the prefix check lets ordinary binaries stop early.
UTF-16 and other encodings remain explicitly uncovered until a measured use case
justifies decoding support.
Known binary extensions can be skipped before opening; the content probe remains the
correctness gate for mislabeled or unknown files.

### Caches Invalidate Per Analyzer

Content cache identity is:

```text
(path, file_fingerprint, type_rules_fingerprint,
 analyzer_id, analyzer_version, analyzer_options_fingerprint)
```

The metadata snapshot remains reusable when an analyzer is added, removed, or upgraded.
Each analyzer identity has one independently checksummed, atomically replaced sidecar
next to the metadata cache.
A reader ignores only an unrecognized or corrupt analyzer sidecar.
It does not discard valid metadata or results from other analyzers.

After metadata revalidation, unchanged fingerprints reuse matching content records
without opening files.
A cache-only report labels content `Cached`; a revalidated match labels it
`Revalidated`; a freshly read result labels it `Scanned`. Adding a query-time page size
never changes cache identity because pages are derived from stored words.

Cache inspection and removal commands treat metadata and content artifacts as one user
cache while reporting their byte sizes separately.

## API and Output Changes

### Grouped Metric Summaries

One `MetricSummarySpec` powers all grouped content reports:

- group by raw extension, stable type, or broad content family
- optionally admit only selected content families
- project any requested additive metric slots
- choose one projected metric as the percentage denominator
- carry analyzer coverage for every row and for the complete report

Each machine row carries the exact share numerator and denominator as integers.
Human renderers calculate the percentage from those integers and may round for display;
coverage excluded from the denominator is printed separately, so analyzed rows cannot
appear to represent an unsupported portion of the tree.
A zero denominator produces an unavailable percentage, not zero percent.

Three named views lower to this generic projection rather than implementing their own
aggregation:

| View | Group and family | Default metric columns | Default percentage |
| --- | --- | --- | --- |
| `languages` | stable type; code | files, apparent bytes, code, comment, blank, physical lines | code lines |
| `documents` | stable type; prose and markup | files, apparent bytes, physical, blank, nonblank lines, raw words, pages | raw words |
| `metrics` | caller-selected group and families | caller-selected | none unless requested |

`--percent-of apparent-bytes` gives the same broad kind of byte share as GitHub’s
language bar; the default `languages` report instead answers what proportion of measured
source code belongs to each language.
When Phase 4 lands, callers can add logical or visible-prose metrics to `documents`
without changing its Phase 2 defaults.

The existing `types`, new `extensions`, and new `families` views use the same summary
engine for metadata fields.
Named summary views default to descending percentage, with stable type ID as the tie
breaker; callers can use the existing sort axis to select a name or metric-qualified
ordering. Tests assert that each named view equals its explicit `metrics` composition,
preventing the presets from becoming alternate semantics.

### Rust API

Add library-owned, non-stringly-typed concepts for:

- `FileTypeId`, `ContentFamily`, and `DetectionSource`
- `AnalyzerId`, `AnalyzerVersion`, and `MetricSlotId`
- `AnalysisRequest` containing an ordered set of analyzers and bounded analyzer options
- `MetricValues` and `AnalysisCoverage`
- `MetricSummarySpec`, `MetricGroup`, and exact metric-share numerator and denominator
- per-file, per-type, per-directory, and summary content rollups
- conditional analysis observations and committed derived deltas

`ScanOptions` and `OpenOptions` accept an `AnalysisRequest` whose default enables no
analyzers. `Query` gains a metric projection and a query-only words-per-page convention.
Report rows expose requested metric maps and coverage without removing their existing
byte, count, timestamp, and provenance fields.

The analyzer boundary receives a resolved type, a bounded byte stream or checked buffer,
and cancellation state.
It cannot walk paths, open a second file handle, spawn an unbounded pool, mutate the
index, or define cache lifecycle.

### CLI

Add repeatable analysis selection under the scan-scope axis:

```text
--analyze basic
--analyze code
--analyze prose
--analyze deep
```

Profiles compose and imply prerequisites: `code` and `prose` include the basic binary
and text-admission pass; `deep` adds bounded detection rather than rerunning the file.
No flag means metadata only.
Within the existing five-axis CLI, analysis belongs to scan scope because it changes
what is read and cached; `languages`, `documents`, and `metrics` belong to the view axis
because they are pure projections over retained results.
A view never silently enables an analyzer.

Add repeatable metric projection for views:

```text
--metric nonblank-lines
--metric code-lines
--metric comment-lines
--metric raw-words
--metric logical-words
--metric pages --words-per-page 250
```

Add `extensions` and `families` to the view grammar when Phase 1 gives `types` its
stable detected-type meaning.
Add the grouped summary views and their generic form:

```text
fdu PATH --analyze code --view languages
fdu PATH --analyze basic --view documents --words-per-page 250
fdu PATH --analyze code --view metrics --group-by type \
  --content-family code --metric physical-lines --metric blank-lines \
  --metric nonblank-lines --percent-of nonblank-lines
```

`physical-lines` includes blank lines, `nonblank-lines` excludes them, and `code-lines`
excludes both blank and comment-only lines.
This metric choice is the whitespace-inclusion option; analysis always retains
`blank-lines` separately, so changing the displayed total or percentage never rereads a
file or invalidates a cache.
The default metric remains the selected apparent or allocated byte size, so existing
tree, files, and summary commands do not change.
A requested metric whose analyzer is not enabled is an explicit usage error rather than
a column of zeros. Human views show one primary metric at a time; machine formats may
request several and include analyzer identity and coverage beside them.
Metric sorting uses the existing sort axis with a metric-qualified value rather than a
one-off command.

### Machine Schemas and Python

Content fields require a new JSON/JSONL/YAML schema version.
The old schema remains stable for metadata-only requests during the compatibility
window; a content request always emits the content-capable schema.

Python exposes the same `AnalysisRequest`, metric identifiers, coverage, and report
values as Rust. It does not implement analyzers or aggregation in Python.
Content work releases the GIL through the same Rust execution boundary as scans.

## Implementation Plan

### Phase 1: Classification and Zero-I/O Type Rollups

- [ ] Define stable file-type IDs, content families, detection sources,
  unknown-extension preservation, and a versioned checked-in rule manifest
- [ ] Reuse the compiled native type-rule work owned by `fdu-v4lc`; do not add a second
  classifier or runtime manifest parser
- [ ] Resolve exact filenames, compound extensions, and ordinary extensions without file
  reads
- [ ] Make `types` report stable type IDs, add `families`, and preserve the current raw
  grouping and row semantics as `extensions`
- [ ] Implement the generic metric-summary projection for extension, type, and family
  groups, initially supporting file and byte metrics and exact share denominators
- [ ] Define analyzer, metric-slot, coverage, options-fingerprint, and
  content-provenance types without enabling an analyzer
- [ ] Add the optional sparse `ContentIndex` boundary and prove it allocates nothing per
  entry while disabled
- [ ] Pin metadata-only Rust, Python, human, and machine output compatibility
- [ ] Publish the pre-1.0 `types`-to-`extensions` migration in help, schemas, and
  release notes
- [ ] Record default-path wall, CPU, RSS, snapshot, and per-entry memory evidence before
  advancing

**Exit criteria:** common exact names and extensions produce deterministic type, family,
file, and byte rollups; unknown and no-extension files remain visible; no regular file
is opened; and the disabled-path performance verdict is “no measurable regression.”

### Phase 2: Basic Streaming Lines, Binary Gating, and Raw Text

- [ ] Implement `content-basic-v1` as one allocation-light streaming pass over each
  eligible file
- [ ] Share one 16 KiB prefix among byte-order-mark recognition, NUL detection, UTF-8
  admission, and the continuing line/word scan
- [ ] Pin empty-file, final-boundary, LF, CRLF, lone-CR, mixed-ending,
  boundary-across-chunk, whitespace-only, invalid-UTF-8, BOM, long-line, and
  adversarial-binary semantics in golden fixtures
- [ ] Add conditional analysis observations, stale-read rejection, derived delta
  application, and subtract/add directory and type rollups
- [ ] Add per-analyzer atomic persistence keyed by strong file fingerprint, type rules,
  analyzer version, and options
- [ ] Expose physical, blank, and nonblank lines for accepted text; expose raw words and
  query-derived pages only for prose-family files
- [ ] Expose analyzer coverage and make requested-analysis failures participate in the
  existing partial-result contract
- [ ] Add `basic` analysis selection and metric projection to Rust, CLI, Python, and the
  versioned machine schema
- [ ] Extend the metric-summary projection with physical, blank, nonblank, and raw-word
  slots; add the `documents` preset and query-derived pages
- [ ] Benchmark cold reads, warm filesystem cache, content-cache hits, 1% churn, large
  files, many tiny files, and the adversarial mislabeled-binary corpus

**Exit criteria:** line identities hold for every fixture; unchanged warm runs perform
no content opens; a changed file updates ancestors by subtraction and addition; binaries
produce coverage rather than text counts; and the basic analyzer stays within its
preregistered CPU, memory, and I/O budgets.

### Phase 3: Common-Language Standard SLOC

- [ ] Build a feature-gated Tokei per-buffer adapter that consumes fdu-owned buffers and
  never invokes Tokei’s walker and disables or bounds nested parallelism
- [ ] Compare binary size, compile time, RSS, large-file behavior, cancellation,
  per-file latency distribution, cache reuse, and 1% churn with a narrow native
  SCC/Tokei-style state-machine prototype
- [ ] Select Tokei or the native implementation through a recorded decision gate before
  adding a production dependency
- [ ] Define `code-sloc-v1` for code, comment, blank, physical, mixed-line, docstring,
  nested-comment, generated-file, and embedded-language behavior
- [ ] Add adversarial golden fixtures for Rust, Python, JavaScript, TypeScript, Go,
  Java, C, C++, C#, Ruby, PHP, Swift, Kotlin, shell, and SQL
- [ ] Compare those fixtures and representative repositories with pinned SCC and Tokei
  revisions, recording intentional dialect differences
- [ ] Expose standard `code_lines` as the ordinary LOC/SLOC rollup and keep
  `nonblank_lines` separately named
- [ ] Add the `languages` preset with exact code-line shares, optional byte shares, and
  explicit unsupported-language coverage
- [ ] Report unsupported and ambiguous language coverage rather than falling back to
  nonblank lines

**Exit criteria:** required common-language fixtures are versioned and green; every
physical line belongs to exactly one code, comment, or blank category; no unsupported
file is silently reported as zero code; and the selected parser passes dependency,
performance, and semantic gates.

### Phase 4: Logical and Markup-Aware Prose

- [ ] Implement `text-logical-v1` as streaming sufficient statistics under the pinned
  3-to-6 non-whitespace-character clamp and half-weight wide-character rule
- [ ] Keep `raw_words` and `logical_words` together so callers can choose literal or
  normalized volume
- [ ] Add plain-text paragraph runs and derive fixed-word page equivalents only after
  aggregation
- [ ] Prototype `markdown-prose-v1` with `pulldown-cmark`, subject to the same
  dependency and buffer-size gates as the code analyzer
- [ ] Retain headings, paragraphs, link labels, image alt text, and opted-in table
  cells; exclude URLs, reference definitions, inline code, code blocks, frontmatter,
  footnote markers, and hidden HTML syntax
- [ ] Compute word statistics directly from parser events instead of materializing a
  second projected document
- [ ] Validate ordinary English, punctuation-heavy prose, symbolic text, multilingual
  spaced scripts, CJK, Markdown links, code fences, tables, HTML, and malformed input
- [ ] Decide whether a sentence slot has a stable enough contract to create
  `text-sentences-v1`; absence remains preferable to a misleading count

**Exit criteria:** page estimates use aggregated words and an explicit convention;
logical-word fixtures match the referenced proposal; Markdown measurements reflect
reader-visible prose; and raw, logical, and visible metrics remain distinct slots.

### Phase 5: Bounded Deep Detection and Specialized Formats

- [ ] Add shebang detection for unresolved text using at most the first 200 bytes
- [ ] Add required-literal prefilters and language heuristics only for ambiguous
  extensions, bounded to the first 20 KiB
- [ ] Add modelines, XML/manpage rules, generated/vendor/documentation flags, and binary
  magic names only when each has a named consumer and fixture corpus
- [ ] Add HTML, notebook, reStructuredText, and other mixed-format projections as
  independent analyzers rather than expanding Markdown semantics implicitly
- [ ] Record detection provenance and confidence on every affected result
- [ ] Benchmark the resolved fast path separately from ambiguity and specialized-format
  paths, including a corpus designed to maximize ambiguous candidates
- [ ] Keep AST metrics, per-byte classifications, and exact tokenizer integrations
  behind separate future analyzer IDs

**Exit criteria:** ordinary recognized files pay no regex or statistical-classifier
cost; every content probe has a byte bound; ambiguous decisions are explainable from
detection provenance; and deeper coverage improves without changing earlier metric
dialects.

## Testing Strategy

### Semantic Tests

- Table-test type rules, exact names, case normalization, compound extensions, unknown
  extensions, dotfiles, extensionless files, and conflicting candidates
- Golden every analyzer with empty input, no final line boundary, long lines, ASCII and
  Unicode whitespace variants, byte-order marks, invalid encodings, and early and late
  NUL bytes
- Generate LF, CRLF, lone-CR, and mixed-ending forms of the same fixtures, including
  CRLF split across every possible streaming chunk boundary, and require identical
  logical counts
- Assert `physical_lines == blank_lines + nonblank_lines` for the basic analyzer
- Assert `physical_lines == code_lines + comment_lines + blank_lines` for the code
  analyzer
- Golden strings containing comment markers, raw strings, multiline and nested comments,
  code plus trailing comments, docstrings, and unterminated constructs
- Golden raw source and reader-visible projections separately for Markdown and later
  markup analyzers
- Use property tests for additive rollup identities and logical-word
  sufficient-statistic composition across arbitrary chunk boundaries

### Incremental and Concurrency Tests

- Compare a fresh full analysis with every sequence of insert, update, rename, type
  change, analyzer upgrade, and removal on the same fixture tree
- Prove a metadata mutation subtracts stale derived contributions before any replacement
  result becomes visible
- Force content changes before open, during read, after EOF, and before commit; accept
  only fingerprint-stable results
- Recycle arena slots to prove generation and revision guards reject ABA results
- Cancel with an empty queue, a full queue, an active large read, and an active buffered
  parser without leaking workers or publishing partial file metrics
- Verify filtered and precomputed unfiltered reports agree when the filter admits every
  entry

### Cache and Schema Tests

- Round-trip each analyzer section independently and corrupt, truncate, or
  version-mismatch one section without losing metadata or unrelated analyzers
- Prove a changed analyzer version misses only its own records
- Prove a changed page convention causes no content-cache miss
- Prove cache-only, revalidated, scanned, partial, and absent analyzer provenance in
  every output format
- Prove each named grouped view equals its explicit generic composition and percentages
  use the named metric, analyzed coverage, exact numerator, and exact denominator
- Golden native CLI, installed wheel, and Rust/Python parity for metadata-only and
  content requests
- Bound record counts, path lengths, metric slot counts, and declared allocation sizes
  before allocating from cache input

### Performance Tests

Add named jobs to the existing performance harness and experiment ledger:

- metadata-only classification and report, proving the disabled-path invariant
- basic analyzer over source, prose, large-line, many-tiny-file, and binary-heavy
  corpora
- content-cache hit and metadata-revalidated content reuse
- 1% content churn with stable path count
- code parser against pinned SCC and Tokei comparators
- Markdown projection and logical-word analysis
- ordinary resolved detection versus deliberately ambiguous detection

Performance verdicts use the existing paired, interleaved protocol.
CI proves benchmark contracts and semantic digests, not timing thresholds.
No README or release claim is made from the exploratory figures in the research brief.

## Rollout Plan

- Land each phase as a separately reviewable change whose default behavior remains
  metadata only
- Keep the research and this plan ahead of implementation in the stacked review chain
- Require a fresh analyzer ID or version for every semantic change after fixtures are
  published
- Keep optional parser dependencies feature-gated until their decision gates pass and
  their cool-off, license, advisory, and locked-build checks are recorded
- Add a profile to the default build only after measuring artifact size, compile time,
  runtime memory, and supported-platform behavior
- Preserve old metadata cache usefulness across every analyzer rollout
- Document coverage before broad language-count claims; unsupported inputs remain
  visible in byte and file rollups
- Move this spec to implemented only after every phase selected for the first public
  content release is complete and its beads, schemas, and validation evidence agree

## Resolved Decisions and Open Questions

Resolved for all phases:

- Content analysis is opt-in and never required for byte-by-type rollups
- `nonblank_lines` is not called LOC
- Standard LOC means `code_lines` in the named `code-sloc-v1` dialect
- Mixed code/comment lines count once as code
- Binary and unsupported input contributes metadata but no invented text metric
- Pages are derived from aggregate words with a visible 250-word default
- fdu owns walking, opening, scheduling, caching, rollups, and coverage
- External parsers receive fdu-owned buffers and never run a second tree walk

Phase-gated questions:

- Does the Tokei adapter meet the artifact, concurrency, semantic, and performance
  gates, or should fdu ship a narrower native state machine?
- What complete-buffer limit gives code and markup analyzers a safe default without
  excluding common large source and documentation files?
- Do Unicode sentence boundaries plus abbreviation and markup fixtures yield a metric
  worth naming, or should sentence count remain out of scope?
- Which specialized markup or data formats have enough measured demand to follow
  Markdown?

None of these questions blocks Phases 1 or 2.

## Beads

| Bead | Status | Role |
| --- | --- | --- |
| `fdu-3n8c` | Open | Governing content-tier feature; link this spec here |
| `fdu-0i15` | Complete | Research, prior-art survey, and exploratory benchmarks |
| `fdu-w09m` | In progress | Create and validate this phased plan spec |
| `fdu-v4lc` | Open prerequisite | Native compiled file-type rules consumed by Phase 1 |

Create implementation beads from this spec only after the plan is reviewed.
Use one bead per independently mergeable phase or decision spike, with dependency edges
matching the phase order.

## References

- [Fast file-content metrics research](../../research/research-2026-08-12-fast-file-content-metrics.md)
- [fdu design principles](../../architecture/fdu-design-principles.md)
- [Composable CLI and query surface plan](plan-2026-08-10-fdu-composable-cli-surface.md)
- [Progressive-results plan](plan-2026-08-11-fdu-progressive-results.md)
- [Performance testing plan](plan-2026-08-09-fdu-end-to-end-performance-testing.md)
- [Performance loop](../../guides/performance-loop.md)
- [Cache design](../../guides/cache-design.md)
- [Post-Phase-1 roadmap](../future/plan-2026-08-09-fdu-post-phase-1-roadmap.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
