# Feature: Fast File Content Metrics

**Date:** 2026-08-12

**Author:** fdu project

**Status:** Approved for phased implementation

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

### SCC Parity and Differentiation

“Beat SCC” is not one claim.
The plan separates surfaces where fdu should lead from surfaces where parity is
expensive:

| Surface | Expected difficulty | Target |
| --- | --- | --- |
| Physical, blank, nonblank, and raw-word streaming | Easy | Match the obvious semantics and plausibly exceed SCC when less syntax work is requested |
| Early known-binary and NUL rejection | Easy | Avoid full reads that SCC may perform on mislabeled binaries |
| Warm unchanged trees and one-percent churn | Moderate | Lead through per-analyzer caching and incremental directory rollups, which SCC does not provide |
| Common-language code/comment/blank counts | Moderate | Match SCC-style standard SLOC for the explicitly supported dialect set |
| Hundreds of languages and every ambiguity rule | Hard | Grow only from measured demand; do not claim SCC’s breadth initially |
| Complexity, ULOC, minified/generated heuristics, and embedded languages | Hard | Keep outside the first standard rollup or add later as independently versioned analyzers |
| Raw, logical, and reader-visible document words, paragraphs, and pages | Moderate | Go beyond SCC with first-class textual volume inspired by FlexDoc |

The likely first wins are basic analysis, binary-heavy inputs, cache hits, and small
incremental changes.
Cold `code-sloc-v1` parity is a measured goal, not a promise to outperform a mature code
counter before profiling.
Feature claims must name the analyzer dialect and supported-language set; speed claims
must name corpus, cache state, and semantic oracle.

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
| `text-logical-v1` | `logical_word_stats` | Additive integer statistics from which normalized prose-volume units are derived |
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

Logical words follow the same rule.
A rounded per-file logical-word value is not additive, so `text-logical-v1` stores three
integer sufficient statistics:

```text
wide_nonspace_chars
nonwide_whitespace_tokens
nonwide_nonspace_chars
```

At query time, the reducer clamps `nonwide_whitespace_tokens` between
`nonwide_nonspace_chars / 6` and `nonwide_nonspace_chars / 3`, adds one half per wide or
fullwidth non-whitespace character, and rounds half-up once after aggregation.
The implementation uses checked integer arithmetic rather than floating point for the
pinned `3`, `6`, and `1/2` constants.
This matches FlexDoc’s logical-word semantics while making directory, type, and filtered
totals independent of file boundaries.

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

`OpenConfig` accepts an `AnalysisRequest` whose default enables no analyzers.
`ScanConfig` remains metadata-only: the content coordinator consumes regular-file
candidates already retained in the index, so neither scan producer nor walker owns
content policy. `Query` gains a metric projection and a query-only words-per-page
convention. Report rows expose requested metric maps and coverage without removing their
existing byte, count, timestamp, and provenance fields.

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
`report_format.rs` keeps `fdu.report/1` for unchanged metadata-only requests during the
compatibility window and emits `fdu.report/2` when a stable type/family or content field
is requested. A content request always emits the content-capable schema.

Python exposes the same `AnalysisRequest`, metric identifiers, coverage, and report
values as Rust. It does not implement analyzers or aggregation in Python.
Content work releases the GIL through the same Rust execution boundary as scans.

## Implementation Plan

### Phase 1: Classification and Zero-I/O Type Rollups

- [x] Define stable file-type IDs, content families, detection sources,
  unknown-extension preservation, and a versioned checked-in rule manifest
- [x] Reuse the compiled native type-rule work owned by `fdu-v4lc`; do not add a second
  classifier or runtime manifest parser
- [x] Resolve exact filenames, compound extensions, and ordinary extensions without file
  reads
- [ ] Make `types` report stable type IDs, add `families`, and preserve the current raw
  grouping and row semantics as `extensions`
- [ ] Implement the generic metric-summary projection for extension, type, and family
  groups, initially supporting file and byte metrics and exact share denominators
- [x] Define analyzer, metric-slot, coverage, options-fingerprint, and
  content-provenance types without enabling an analyzer
- [x] Add the optional sparse `ContentIndex` boundary and prove it allocates nothing per
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

- [x] Implement `content-basic-v1` as one allocation-light streaming pass over each
  eligible file
- [x] Share one 16 KiB prefix among byte-order-mark recognition, NUL detection, UTF-8
  admission, and the continuing line/word scan
- [ ] Pin empty-file, final-boundary, LF, CRLF, lone-CR, mixed-ending,
  boundary-across-chunk, whitespace-only, invalid-UTF-8, BOM, long-line, and
  adversarial-binary semantics in golden fixtures
- [x] Add conditional analysis observations, stale-read rejection, derived delta
  application, and subtract/add directory and type rollups
- [x] Add per-analyzer atomic persistence keyed by strong file fingerprint, type rules,
  analyzer version, and options
- [ ] Expose physical, blank, and nonblank lines for accepted text; expose raw words and
  query-derived pages only for prose-family files
- [x] Expose analyzer coverage and make requested-analysis failures participate in the
  existing partial-result contract
- [ ] Add `basic` analysis selection and metric projection to Rust, CLI, Python, and the
  versioned machine schema
- [ ] Extend the metric-summary projection with physical, blank, nonblank, and raw-word
  slots; add the `documents` preset and query-derived pages
- [ ] Lock the complete Rust/CLI/Python contract with the multilingual tryscript fixture
  and the repository self-host sanity check before changing the implementation for speed
- [ ] Then benchmark cold reads, warm filesystem cache, content-cache hits, 1% churn,
  large files, many tiny files, and the adversarial mislabeled-binary corpus

**Exit criteria:** line identities hold for every fixture; unchanged warm runs perform
no content opens; a changed file updates ancestors by subtraction and addition; binaries
produce coverage rather than text counts; end-to-end goldens and the self-host check are
green before performance work begins; and the basic analyzer stays within its
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
- [ ] Extend the multilingual tryscript and repository self-host checks, then freeze
  their semantic outputs before any code-parser performance iteration

**Exit criteria:** required common-language fixtures are versioned and green; every
physical line belongs to exactly one code, comment, or blank category; no unsupported
file is silently reported as zero code; and the selected parser passes dependency,
performance, and semantic gates.

### Phase 4: Logical and Markup-Aware Prose

- [ ] Implement `text-logical-v1` as streaming sufficient statistics under the pinned
  3-to-6 non-whitespace-character clamp and half-weight wide-character rule
- [ ] Store `LogicalWordStats` additively and derive `logical_words` only after the
  selected files have been aggregated; never sum rounded per-file logical words
- [ ] Keep raw, logical, and visible words together so callers can choose literal,
  normalized, or reader-visible volume
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
- [ ] Extend the multilingual tryscript and repository self-host checks with document
  rows, raw/logical/visible word relationships, paragraphs, and page denominators before
  prose performance iterations
- [ ] Decide whether a sentence slot has a stable enough contract to create
  `text-sentences-v1`; absence remains preferable to a misleading count

**Exit criteria:** page estimates use aggregated words and an explicit convention;
logical-word fixtures match the referenced proposal; Markdown measurements reflect
reader-visible prose; raw, logical, and visible metrics remain distinct slots; and fdu
offers first-class document volume that SCC’s code-focused summary does not.

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

## File and Function Implementation Map

The implementation stays inside the existing `fdu` and `fdu-py` crates.
It adds modules, not another crate, and it does not modify the metadata walker to
perform content I/O. Names below are the intended ownership boundaries; small private
helpers may move during implementation when tests show a clearer boundary, but public
types and mutation paths must retain these responsibilities.

### Core Classification and Metric Contracts

| File | Types and functions | Responsibility |
| --- | --- | --- |
| `crates/fdu/src/classify.rs` | `FileTypeId`, `ContentFamily`, `DetectionSource`, `Classification`, `classify_path()` | Consume the compiled rules from `fdu-v4lc`; resolve exact names and extensions without content I/O; preserve unknown extensions as stable IDs |
| `crates/fdu/src/content/mod.rs` | module exports, `AnalysisRequest`, `AnalysisProfile`, `resolve_profile()`, `analyze_index()` | Public content-analysis facade; expand profiles into ordered analyzer IDs; own one-shot orchestration over index candidates |
| `crates/fdu/src/content/types.rs` | `AnalyzerId`, `AnalyzerVersion`, `MetricSlotId`, `MetricValues`, `LogicalWordStats`, `AnalysisCoverage`, `CoverageReason`, `ContentProvenance`, `AnalysisCandidate`, `AnalysisObservation`, `AppliedAnalysisDelta` | Define the versioned, additive contract shared by workers, index, cache, query, CLI, and Python |
| `crates/fdu/src/content/index.rs` | `ContentIndex`, `FileAnalysis`, `ContentRollUp`, `ContentIndex::commit()`, `ContentIndex::invalidate()`, `ContentIndex::rollup()` | Hold sparse per-file results and precomputed directory/type/family rollups; subtract before add; allocate only after analysis is enabled |
| `crates/fdu/src/index.rs` | add `content: Option<Box<ContentIndex>>`; `analysis_candidates()`, `apply_analysis()`, `content_rollup_of()`; invalidate from `apply_upsert()`, `apply_remove()`, and `remove_entry()` | Keep generation, revision, fingerprint, and ancestor ownership inside the sole mutation authority; reject stale worker observations and emit typed derived deltas |
| `crates/fdu/src/lib.rs` | export content types; add `OpenConfig::analysis`; extend `OpenReport`; call `analyze_index()` from `open_with_pending_save()` | Preserve the disabled default; run analysis after metadata is complete for one-shot opens; expose content completeness separately from metadata completeness |

`AnalysisCandidate` is an owned snapshot containing `EntryId`, relative and absolute
path, entry revision, `Attrs::fingerprint()`, classification, and requested analyzers.
Workers must not retain an index lock.
`AnalysisObservation` carries the same expectation, the pre-read and post-read handle
fingerprints, results, and coverage outcome.
`Index::apply_analysis()` is the only content mutation entry point: it rechecks the
generation, revision, kind, classification, and all fingerprints before committing.
A mismatch returns a stale outcome that the coordinator may requeue; it never publishes
partial metrics.

### Streaming, Detection, and Analyzer Ownership

| File | Types and functions | Responsibility |
| --- | --- | --- |
| `crates/fdu/src/content/basic.rs` | `BasicAccumulator`, `BasicAccumulator::push()`, `BasicAccumulator::finish()`, `TextAdmission` | Fuse NUL detection, UTF-8 validation, physical/blank/nonblank lines, raw prose words, paragraph runs, and logical-word sufficient statistics in a reusable-buffer streaming pass |
| `crates/fdu/src/content/worker.rs` | `AnalysisCoordinator`, `AnalysisCoordinator::run()`, `analyze_candidate()`, `read_bounded()` | Own the bounded queue and worker pool; open once; `fstat` before and after EOF; share the first 16 KiB; enforce cancellation and per-file limits |
| `crates/fdu/src/content/code.rs` | `CodeAnalyzer` trait, `analyze_code()`, optional `tokei_adapter`, optional native state machine | Consume an fdu-owned checked buffer and return the `code-sloc-v1` partition without walking, reopening, or spawning unbounded work |
| `crates/fdu/src/content/text.rs` | `LogicalWordAccumulator`, `logical_words()`, `PlainTextAccumulator` | Implement the integer FlexDoc-compatible `3..6` clamp, half-weight wide characters, raw/logical words, and plain-text paragraph runs |
| `crates/fdu/src/content/markdown.rs` | `MarkdownAccumulator`, `analyze_markdown()` | Fold parser events directly into visible-word and paragraph statistics; never materialize a second projected document |
| `crates/fdu/src/content/detect.rs` | `probe_prefix()`, `detect_shebang()`, `resolve_ambiguity()` | Add only bounded content-dependent detection; keep known extension and exact-name paths free of regex or statistical work |

`BasicAccumulator` retains whether any byte has been seen, whether the current logical
line contains a non-whitespace code point, whether the previous chunk ended in CR,
whether a raw word is open, incremental UTF-8 decoder state, and paragraph-run state.
`finish()` alone accounts for a final unterminated line or word.
Tests invoke `push()` at every byte boundary so CRLF, multibyte whitespace, BOM, and
token boundaries cannot accidentally depend on buffer size.

Phase 2 may put logical-word sufficient statistics into the fused pass while exposing
only raw words; this is allowed only when disabled slots cost no extra allocation and
the measured CPU delta is negligible.
Otherwise `LogicalWordAccumulator` is enabled by the prose profile in Phase 4. Known
binary types skip opening.
Unknown and text/code candidates share one prefix and one handle; a NUL discovered after
the prefix discards all provisional text results and records `binary` coverage.

### Cache and Lifecycle Integration

| File | Types and functions | Responsibility |
| --- | --- | --- |
| `crates/fdu/src/cache_file.rs` | `write_atomically()`, bounded header/read helpers, temporary-file cleanup | Extract the existing sibling-temp, sync, rename, and stale-temp lifecycle from `snapshot.rs` for reuse without changing metadata snapshot v2 |
| `crates/fdu/src/snapshot.rs` | call `cache_file::write_atomically()`; keep `FORMAT_VERSION = 2` and content-free records | Preserve metadata snapshot compatibility and engine-fingerprint behavior |
| `crates/fdu/src/content/cache.rs` | `sidecar_path()`, `load_sidecar()`, `save_sidecar()`, `AnalyzerCacheHeader` | Read and write one bounded, checksummed sidecar per analyzer identity; validate file fingerprint, rule fingerprint, analyzer version, and options fingerprint before allocation |
| `crates/fdu/src/cache.rs` | extend `CacheStatus`; update `cache_status()`, `list_caches()`, `clear_cache()`, `clear_all_caches()` | Present metadata and recognized content sidecars as one cache lifecycle while refusing to delete foreign files |
| `crates/fdu/src/lib.rs` | change private `PendingSave` storage from one optional handle to a vector of named save handles; retain `PendingSave::join()` | Overlap metadata and analyzer-sidecar writes with rendering, join every started writer, and preserve the current public lifecycle |
| `crates/fdu/src/session.rs` and `crates/fdu/src/watch.rs` | enqueue candidates from committed metadata deltas; publish `AppliedAnalysisDelta` | Invalidate immediately on a metadata change, analyze asynchronously, and preserve the same stale-result checks during watch sessions |

Sidecar corruption is analyzer-local.
A missing or invalid `code-sloc-v1` file cannot invalidate metadata or a valid
`content-basic-v1` sidecar.
A `CachePolicy::Only` request may report content only from matching sidecars and must
fail explicitly when a requested analyzer is absent; it cannot read source files.
A normal warm revalidation first reconciles metadata, then admits sidecar records only
for unchanged fingerprints.

### Query, CLI, Rendering, and Python

| File | Types and functions | Responsibility |
| --- | --- | --- |
| `crates/fdu/src/query/report.rs` | extend `ViewSpec`; add `MetricSummarySpec`, `MetricGroup`, `MetricRow`, `MetricShare`, `metric_rows()`; extend `Walked`, `build_section()`, `merge_summary()` | Power `extensions`, `types`, `families`, `languages`, `documents`, and `metrics` from one projection; aggregate sufficient statistics before deriving logical words or pages |
| `crates/fdu/src/query/selection.rs` | add metric-qualified sort key and validation | Reuse the sort axis for projected metrics without introducing report-specific ordering flags |
| `crates/fdu/src/query/mod.rs` | export the new query and row types | Keep the library surface typed and discoverable |
| `crates/fdu/src/cli.rs` | add scope `--analyze`; view/metric/group/percentage/page options; update `Cli::run()`, `parse_view()`, and new parsers | Parse the complete request before I/O, reject a metric whose analyzer is absent, and keep every option on one of the five documented axes |
| `crates/fdu/src/report_format.rs` | add content sections to `render_text()`, JSON, JSONL, and YAML helpers; introduce `fdu.report/2` only for content-capable output | Render stable integer numerators, denominators, analyzer identities, provenance, and coverage; retain byte-identical `fdu.report/1` metadata output during the compatibility window |
| `crates/fdu-py/src/lib.rs` | extend `open()`, `scan()`, `PyIndex::report()`, `report_dict()`, and parsers | Mirror Rust profiles, metric IDs, grouped rows, coverage, and query-derived pages; keep analysis in Rust with the GIL released |
| `crates/fdu/src/skills/SKILL.md` and user docs | document profiles, metrics, coverage, and examples | Keep agent-facing help and human CLI help synchronized with the shipped surface |

The `languages` and `documents` presets call the same `metric_rows()` path as an
explicit `metrics` request.
Stable report tests compare the named and expanded forms.
The filtered tier extends the existing single `walk()` pass to collect content
contributions; the unfiltered tier reads `ContentIndex` rollups.
No report reads a file.

### Tests, Self-Hosting, and Performance Harness

| File | Tests or functions | Responsibility |
| --- | --- | --- |
| `crates/fdu/src/content/*` test modules | accumulator, parser, cache-bound, and detection tables plus chunk-boundary/property tests | Pin each versioned analyzer dialect at its narrowest unit |
| `crates/fdu/tests/content_incremental.rs` | full scan versus insert/update/rename/type-change/remove sequences; race and ABA cases | Prove subtract/add maintenance, stale rejection, and full-recompute equivalence |
| `crates/fdu/tests/content_cache.rs` | cold, revalidated, cache-only, corrupt, mismatched, and partial cases | Prove analyzer-local invalidation and no-open cache hits |
| `tests/golden/fixtures/content-project/` | small multilingual Rust, Python, JavaScript/TypeScript, Go, shell, SQL, Markdown, plain text, mixed endings, and binary files | Provide one auditable exact-output fixture rather than depending on the changing repository for golden totals |
| `tests/golden/cli-content.tryscript.md` | human, JSON, JSONL, YAML, aliases, errors, coverage, and equivalent-preset transcripts | Lock the end-to-end CLI and schema contract before optimization begins |
| `scripts/content-selfcheck.mjs` and `make content-selfcheck` | materialize tracked `HEAD` files with `git archive`, run the release binary with cache off, and validate the machine report | Exercise fdu on its own multilingual Rust, Python, JavaScript, Markdown, TOML, YAML, and shell sources without scanning `.git`, build output, caches, or other worktrees |
| `crates/fdu/examples/perf_probe.rs` | add `content-basic`, `code-sloc`, `text-prose`, `markdown-prose`, and `binary-gate` modes | Measure components through supported public APIs and emit semantic digests outside the component timer |
| `benchmarks/corpora.json`, `benchmarks/scenarios.json`, and `benchmarks/schema/` | add content recipes, named jobs, transitions, and result fields | Reuse the existing generated-corpus, validation, and paired-measurement contracts |
| `benchmarks/realtree/` and the experiment ledger | add immutable self-host tree fingerprints and SCC/Tokei comparators | Measure real multilingual behavior and record accepted and rejected optimization hypotheses |

The self-host check intentionally asserts identities and coverage, not a permanently
pinned total for the moving repository.
It requires all expected families and common languages to appear, every grouped total to
equal the summary, every code partition to satisfy `physical = code + comment + blank`,
every basic partition to satisfy `physical = nonblank + blank`, unsupported files to
retain file and byte coverage, and binary files to expose no text metrics.
Exact values live in the small tryscript fixture.
Performance runs use an immutable `git archive` of a recorded commit outside the
measured root and record its fingerprint, so candidate and control see identical bytes.

### Merge Slices and Required Gates

Each row below is independently reviewable and keeps metadata-only behavior intact:

1. Land compiled classification, stable type/family summaries, and the sparse disabled
   content boundary after `fdu-v4lc`.
2. Land the basic accumulator and its unit/property tests without exposing CLI output.
3. Land orchestration, conditional derived deltas, sidecars, coverage, and cache tests.
4. Land Rust/CLI/Python projections, `fdu.report/2`, the multilingual tryscript fixture,
   and `content-selfcheck`; this is the Phase 2 semantic lock.
5. Only after slice 4 passes `make test-golden`, `make content-selfcheck`, and
   `make check`, run preregistered basic-analysis performance iterations and accept or
   reject each change through the existing paired protocol.
6. Repeat the same semantic-lock-then-performance sequence for common-language SLOC.
7. Repeat it for logical prose and Markdown-visible prose.
8. Add deep detection and specialized formats only after ordinary resolved paths are
   profiled and the prior analyzer dialects remain unchanged.

No performance patch may update a golden merely to make an unexpected semantic change
pass. First explain and version the semantic change, update the analyzer ID or version
when required, review the golden diff, and only then resume performance measurement.

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

### End-to-End Semantic Lock

Before the first performance iteration in each phase:

1. Add or update unit and property tests for the analyzer dialect.
2. Add the user-visible request to `tests/golden/cli-content.tryscript.md` and review
   the full human and machine output diff.
3. Run `make test-golden`, then `make content-selfcheck` against a tracked-file archive
   of this multilingual repository, then the repository-wide `make check` gate.
4. Record the fixture digest and self-host invariants in the performance scenario’s
   validation contract.
5. Only then preregister and measure an optimization hypothesis.

After each candidate optimization, the same tests and digests run before its timing
sample is eligible. The changing repository is a broad sanity corpus, not an exact
golden; the committed multilingual fixture remains the exact, reviewable contract.

### Performance Tests

Add these named jobs to the existing performance harness and experiment ledger:

| Job | Corpus and cache state | Question |
| --- | --- | --- |
| `content-disabled` | standard metadata corpora; analysis absent | Did classification or optional storage regress the default path? |
| `content-basic-cold` | source, prose, long-line, and many-tiny-file corpora; cache absent | What does one fused read cost by file shape? |
| `content-basic-warm-fs` | same bytes with filesystem pages warm; content cache absent | What is analyzer CPU throughput without cold-storage noise? |
| `content-basic-cache-hit` | compatible metadata and analyzer sidecars | Do unchanged files avoid content opens and approach metadata-only cost? |
| `content-basic-churn-1pct` | stable paths with one percent modified | Are cache lookup, invalidation, reread, and subtract/add proportional to change? |
| `binary-gate` | known binary, early-NUL mislabeled, late-NUL mislabeled, and random data | Does rejection save I/O and prevent invented metrics without hurting text? |
| `code-sloc-cold` | representative common-language source | What is end-to-end and parser-only throughput relative to SCC and Tokei? |
| `code-sloc-cache-hit` | compatible `code-sloc-v1` sidecar | Does the deeper analyzer preserve warm incremental behavior? |
| `text-prose` | English, punctuation-heavy, long-token, spaced multilingual, and CJK prose | What do raw plus logical statistics cost relative to basic lines? |
| `markdown-prose` | links, images, code fences, tables, HTML, and malformed Markdown | What does visible projection cost and how does it scale with input size? |
| `detect-ambiguous` | resolved fast path versus ambiguity-maximizing candidates | Do deeper rules remain bounded and off the common path? |
| `selfhost-content` | immutable archive of a recorded fdu commit | Do synthetic conclusions hold on a real multilingual project? |

Performance verdicts use the existing paired, interleaved protocol.
Each optimization is a separately preregistered experiment, preferably changing one
mechanism at a time.
Candidate and control each run at least 12 paired trials; acceptance requires a median
improvement of at least 3 percent, a 95 percent paired interval wholly below zero, valid
semantic oracles, and no meaningful RSS, artifact-size, compile-time, or tail-latency
regression outside the preregistered budget.
Accepted and rejected hypotheses both go in the experiment ledger.
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
| `fdu-w09m` | Complete | Refine and validate this implementation-ready plan spec |
| `fdu-v4lc` | Complete | Native compiled file-type rules consumed by Phase 1 |
| `fdu-j5ny` | In progress | Implement this approved spec in an isolated feature worktree |
| `fdu-m7n5` | Complete | Stable classification, metric contracts, and disabled sparse boundary |
| `fdu-ciq7` | Complete | Fused basic streaming analyzer and boundary/property tests |
| `fdu-96l2` | Complete | Workers, conditional deltas, incremental rollups, and sidecars |
| `fdu-8kd8` | Open; blocked by `fdu-96l2` | Basic Rust, CLI, report-schema, view, and Python surface |
| `fdu-occl` | Open; blocked by `fdu-8kd8` | Basic tryscript golden and multilingual self-host semantic lock |
| `fdu-tq3k` | Open; blocked by `fdu-occl` | Basic and binary-gate performance iterations |
| `fdu-jmrs` | Open; blocked by `fdu-tq3k` | Tokei-versus-native SLOC decision spike |
| `fdu-q3sx` | Open; blocked by `fdu-jmrs` | Production `code-sloc-v1` and language summary |
| `fdu-d82z` | Open; blocked by `fdu-q3sx` | SLOC goldens and self-host semantic lock |
| `fdu-zfjk` | Open; blocked by `fdu-d82z` | SLOC and comparator performance iterations |
| `fdu-cq7i` | Open; blocked by `fdu-zfjk` | Additive logical words, paragraphs, and pages |
| `fdu-6sas` | Open; blocked by `fdu-cq7i` | Reader-visible Markdown prose metrics |
| `fdu-1ysa` | Open; blocked by `fdu-6sas` | Document goldens and self-host semantic lock |
| `fdu-3b5a` | Open; blocked by `fdu-1ysa` | Text and Markdown performance iterations |
| `fdu-kgml` | Open; blocked by `fdu-3b5a` | Bounded deep detection and specialized formats |
| `fdu-eu80` | Open; blocked by `fdu-kgml` | Final compatibility, documentation, and release validation |

The epic’s child-order hints match this table, and every child was created with its
blocker edge already attached.
Implementation starts from `fdu-m7n5` only after `fdu-v4lc`; it does not reuse the
primary repository checkout.

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
