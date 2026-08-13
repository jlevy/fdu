# Research: Fast File-Type and Content Metrics

**Date:** 2026-08-12

**Author:** Joshua Levy, with Codex research assistance

**Status:** Complete

## Overview

fdu can add useful code and prose measurements without putting content I/O on its
metadata-only fast path.
The recommended design is a versioned, opt-in analyzer ladder:

1. Preserve exact byte and file-count rollups for every type without opening files.
2. Add a small fdu-owned streaming analyzer for physical lines, blank lines, nonblank
   lines, raw words, and an early binary probe.
3. Add language-aware code, comment, and blank-line counts through a separately gated
   analyzer, initially prototyped with Tokei’s per-buffer Rust API.
4. Add logical-word and markup-aware prose projections only for file families that need
   them.
5. Add bounded content heuristics for ambiguous language detection after path rules and
   shebangs, not on every file.

This separates three concerns that other tools often combine: identifying a file,
measuring its contents, and deciding how its measurements roll up.
GitHub Linguist is strong prior art for identification, but GitHub’s language
percentages are byte percentages, not comment-aware lines of code.
SCC and Tokei are strong prior art for code measurement.
FlexDoc and the logical-word-count proposal are strong prior art for prose measurement.

The exploratory spikes support the layered design.
On a warm 19.9 MB source corpus, complete Tokei and SCC runs finished in roughly 42-65
ms median. An in-memory pass that counted lines, blanks, raw words, and NUL bytes
sustained about 590 MB/s, while SCC’s deeper code/comment state machine sustained about
260 MB/s on the same synthetic code-like input.
A 10 KB NUL probe had no measurable cost on the ordinary corpus and prevented expensive
nonsense parsing on a deliberately mislabeled binary corpus.

The content tier should keep the cache design already established by the
[file-rollup research](research-2026-08-06-file-rollup-engine.md): results are keyed by
file fingerprint, analyzer identity, analyzer version, and type-rule version.
A warm scan rereads only changed files.
The metadata-only jobs and their performance contract remain unchanged.

## Questions to Answer

1. Which metrics are useful and additive enough to store in per-file and per-directory
   rollups?
2. How should fdu distinguish binary, code, prose, markup, data, and unknown files
   without paying deep-detection costs on every file?
3. What should “lines of code” mean across languages, comments, mixed lines, docstrings,
   embedded languages, and files without a final newline?
4. How should raw words, normalized words, and fixed-word page estimates work for plain
   text and markup?
5. Which parts should fdu implement, and which should come from GitHub Linguist, SCC,
   Tokei, or smaller Rust libraries?
6. What speed and memory costs appear at each depth of analysis?

## Scope

**Included:**

- Per-file metrics and additive rollups by directory, file type, content family, and
  language
- Path-based detection, binary probing, shebangs, bounded ambiguity heuristics, and a
  plan for deeper Linguist-style detection
- Physical, blank, nonblank, code, and comment line semantics
- Raw words, logical words, markup-aware prose, and fixed-word page equivalents
- Direct source review of GitHub Linguist, SCC, Tokei, loc, FlexDoc, and the
  logical-word proposal
- A survey of focused Rust crates for binary, MIME, Unicode-word, and Markdown handling
- Exploratory warm-cache and in-memory performance spikes

**Excluded:**

- Product implementation or a dependency change
- Parser-perfect semantic lines of code, executable-statement counts, AST complexity, or
  compiler integration
- Exact linguistic word segmentation and exact model-token counts
- Release-grade performance claims.
  The measurements in this brief establish an order of magnitude and choose the next
  experiments; they do not replace the
  [performance protocol](../guides/performance-loop.md).

## Findings

### Classification, Measurement, and Inclusion Are Separate Decisions

A file needs three independent labels:

- **Identity:** a stable type and optional language, such as Rust, Markdown, PDF, or an
  unknown `.foo` extension
- **Content family:** code, prose, markup, data, binary, or unknown
- **Inclusion policy:** whether a view includes first-party, generated, vendored,
  documentation, or ignored content

GitHub Linguist combines these decisions for the purpose of a repository language bar.
It excludes binary, vendored, generated, documentation, data, and prose files, then
rolls up the remaining programming and markup files by byte count.
That policy is useful for GitHub but wrong as fdu’s storage model: fdu must still report
that documentation, data, binaries, and unknown files occupy space.
Inclusion is a query policy over stored facts, not part of identification.

The type result should carry provenance rather than pretending every result has equal
certainty:

```text
type_id              stable fdu type identifier
content_family       code | prose | markup | data | binary | unknown
language_id          optional stable language identifier
detection_source     exact_name | extension | shebang | heuristic | magic | override
detection_confidence exact | likely | fallback
type_rules_version   version of the rules that produced the result
```

Unknown extensions should remain visible as normalized groups such as `ext:.foo`.
Extensionless files should remain `no_extension` unless an exact-name, shebang, or
content rule identifies them.
This is more complete than SCC and Tokei’s default reports, which skip many unknown
files.

### The Detection Ladder Should Bound Content Work

The existing type-rule direction in `fdu-v4lc` is the right foundation.
Detection should stop at the first decisive layer:

1. **Explicit override.** User or embedding-supplied rules win.
2. **Exact filename.** `Dockerfile`, `Makefile`, `Cargo.lock`, and similar names.
3. **Compound and simple extension.** Match the longest known suffix first, then the
   final extension, case-insensitively where the rule says so.
4. **Known binary family.** Strong extensions such as common image, archive, font,
   audio, and video types can skip text analysis while retaining exact byte rollups.
5. **First-chunk inspection.** On one open file handle, read a bounded first chunk,
   recognize byte-order marks, check for NUL bytes, and continue from the same handle
   only when text analysis is needed.
6. **Shebang.** For extensionless or explicitly ambiguous text, inspect the first line.
7. **Bounded ambiguity heuristic.** Only when a path maps to multiple languages, scan at
   most 20 KB for cheap required literals and then language-specific regular
   expressions.
8. **Optional full detector.** Modelines, XML headers, manpage conventions, generated
   and vendored rules, and statistical classification belong in a deeper analyzer.

This ordering borrows the strongest ideas from Linguist and SCC without paying
Linguist’s complete Ruby, ICU, and libgit2 stack.
Linguist’s current sequence is modeline, filename, shebang, extension, XML header,
manpage, heuristics, then a naive Bayesian classifier.
SCC master uses exact names and compound extensions, reads at most 200 bytes for
shebangs, and inspects at most 20 KB for ambiguous-extension heuristics.
It first searches for required literals so most files never execute the regular
expressions.

Binary detection needs a documented confidence model.
A NUL check is fast but not a proof: UTF-16 and UTF-32 text normally contain NUL bytes,
while some binary formats may not.
A byte-order mark must therefore be recognized before NUL classification.
Version one can analyze UTF-8 and ASCII, record other recognized encodings as text with
content metrics unavailable, and add streaming decoding later.
Missing metrics must remain absent, with coverage counts, rather than silently becoming
zero.

The small `content_inspector` crate implements almost this exact BOM-plus-NUL heuristic
over the first 1,024 bytes.
Its logic is small enough to implement and test directly in fdu instead of adding a
dependency. The `infer` crate is useful later for friendly names for common binary magic
signatures. `tree_magic_mini` is broader but can depend on a host MIME database, may
embed GPL-licensed database material, and reports a 5-100 us classification range.
It is too costly and operationally variable for the default content gate.

### Metric Slots Need Explicit Semantics and Coverage

Every analyzer result should be a fixed, versioned group of additive integer slots.
The reducer registry owns addition, subtraction, persistence, and invalidation.
The analyzer owns the meaning of each slot.

The first generic content analyzer should expose:

| Slot | Meaning |
| --- | --- |
| `physical_lines` | Logical lines, including the final non-newline-terminated line |
| `blank_lines` | Lines containing no bytes other than configured line whitespace |
| `nonblank_lines` | `physical_lines - blank_lines` |
| `raw_words` | Whitespace-delimited words, for prose-family files only |
| `analyzed_bytes` | Bytes actually accepted as text by this analyzer |

Byte size and file count already come from metadata and apply to every file, including
binaries and unsupported encodings.
The analyzer should also roll up coverage:

```text
eligible_files, analyzed_files, eligible_bytes, analyzed_bytes
```

Coverage distinguishes “the tree has no code” from “the code analyzer did not run.”
It also makes partial or stale content results honest during an incremental scan.

The initial implementation should call `nonblank_lines` exactly that.
It is not yet lines of code because it includes comment-only lines.
Once the language-aware analyzer runs, its standard code slots should be:

| Slot | Meaning |
| --- | --- |
| `code_lines` | Lines containing code; this is the standard SLOC/LOC rollup |
| `comment_lines` | Comment-only lines, including blank physical lines inside a multiline comment when the dialect specifies that behavior |
| `blank_lines` | Whitespace-only lines outside comments |
| `physical_lines` | `code_lines + comment_lines + blank_lines` |

A line containing both code and a comment counts as code.
This is the convention used by SCC and cloc and prevents one physical line from
contributing twice. Strings hide comment delimiters.
Nested comments are language-specific.
Docstring treatment, generated-file treatment, and embedded-language attribution must be
analyzer settings or versioned dialect rules, never implicit changes.

Per-byte code/comment/string classification should remain a separate opt-in result.
SCC allocates one classification byte per source byte, and the local spike measured a
roughly 15% parser-time increase on a code-like buffer.
Most fdu rollups need line counts, not a byte mask.

Unique lines of code are not a normal additive slot.
A correct union needs the line multiset, a reference-counted hash index, or an
explicitly approximate sketch; simple per-file ULOC sums overcount duplicates and cannot
be cleanly subtracted after a file change.
Defer ULOC until a separate reducer justifies its memory and persistence cost.

### Prose Needs Both a Cheap Raw Measure and a Reader-Visible Measure

Plain English text should not be summarized only by source lines.
The useful base metrics are physical lines, nonblank lines, raw words, and a derived
fixed-word page equivalent.
Store words, not rounded pages.
A query can display `logical_words / words_per_page` with an explicit default such as
250 words per page.
Deriving pages after aggregation avoids the error from rounding every
file separately and lets a caller choose another fixed convention without invalidating
cached content.

Raw whitespace words are intuitive for ordinary English and cheap to count in the same
pass as lines. They fail on unspaced CJK text, long URLs and identifiers, minified code,
and punctuation-dense data.
FlexDoc’s implemented logical-word measure provides a useful additional *volume* metric:

1. Non-whitespace Unicode characters with East Asian Width `W` or `F` contribute 0.5
   word each.
2. Remaining text is split on whitespace.
3. Its word count is clamped to an average of 3-6 non-whitespace characters per word.
4. The combined non-negative value is rounded half-up.

The measure equals raw word count for ordinary spaced prose in its normal character
range. It prevents one URL or identifier from collapsing to one word and gives unspaced
CJK a useful magnitude.
It is not linguistic segmentation.
Thai, Lao, Khmer, and Myanmar need separate treatment if they become important, and
punctuation-dense machine formats remain a poor basis for human page counts.
fdu should therefore expose both `raw_words` and `logical_words`, label logical words as
a normalized volume unit, and apply prose page estimates only to prose or reader-visible
markup projections.

The published logical-word implementation builds a replacement string and word list.
A Rust analyzer can compute the same sufficient statistics in one streaming UTF-8 pass:
non-whitespace wide-code-point count, non-wide token count, and non-wide token character
count. The aggregate must be computed before rounding.
A pinned Unicode property table or a documented range approximation becomes part of the
analyzer version.

Raw Markdown is not reader-visible prose.
Counting it includes destinations, reference definitions, badges, HTML attributes, and
code samples. FlexDoc’s `prose_text()` gives a good contract:

- keep paragraphs and headings, plus table cells only when requested
- reduce links and images to label or alt text and exclude destinations
- remove inline code, footnote references, reference definitions, code blocks,
  frontmatter, and block markers
- remove inline HTML tags while retaining text they wrap
- preserve source line wrapping

For fdu, a later `markdown-prose-v1` analyzer can implement this contract with
`pulldown-cmark`’s streaming event API. A pull parser avoids an owned AST when the only
output is additive metrics.
Fenced code can be excluded from prose and optionally sent to the code analyzer under
its declared language.
HTML, reStructuredText, AsciiDoc, and notebook projections should be distinct analyzers;
a generic regular expression is not accurate enough to strip scripts, styles,
attributes, URLs, and nested markup.

### GitHub Linguist Is a Detector and Byte Rollup, Not the LOC Engine

At the pinned 2026-08-12 commit, Linguist defines 828 languages or formats: 557
programming, 71 markup, 182 data, and 18 prose.
It has the broadest classification taxonomy in this survey and mature rules for
ambiguous extensions, generated content, vendored paths, documentation, and overrides.

GitHub’s repository languages API returns bytes of code per language.
Linguist’s own single-file `loc` is a physical-line count and `sloc` is a nonblank-line
count; it does not remove comment-only lines.
GitHub therefore should not be used as the semantic reference for standard comment-aware
LOC. It remains the strongest reference for type identity and detection precedence.

Directly embedding Linguist would add Ruby, ICU-backed encoding detection, and libgit2
bindings to a small Rust engine.
A generated, pinned subset of Linguist’s path metadata could be useful later, but
importing all rules creates a synchronization product of its own.
The first release should extend fdu’s existing declarative type rules and use Linguist
fixtures as cross-checks.

### Tokei Is the Best Rust Reuse Candidate; SCC Is the Best Design Reference

Tokei 14 is a Rust library as well as a CLI. The reviewed source defines 332 language
types and exposes `LanguageType::parse_from_slice`, so fdu can retain ownership of
walking, I/O, caching, and rollups while reusing only the per-buffer code parser.
Tokei uses generated syntax tables, a state machine that distinguishes strings and
comments, and support for nested comments and embedded languages.
It also brings a sizable dependency and concurrency surface, including Aho-Corasick,
`encoding_rs_io`, `grep-searcher`, `ignore`, Rayon, regular expressions, and
serialization support.

SCC master is a Go program and library with 365 language definitions.
It has an allocation-conscious parallel walker, reusable read buffers, a byte-oriented
token trie and state machine, generated language constants, binary probing, optional
generated/minified detection, ULOC, and optional per-byte classification.
Its current ambiguity heuristics are explicitly inspired by Linguist.
Reusing SCC in-process would require Go FFI or a subprocess, neither of which fits fdu’s
library and Python embedding contract.
Porting its state machine and language data is possible under MIT, but it would make fdu
responsible for a large language dialect and its ongoing synchronization.

The practical split is:

- use SCC as an algorithm reference, correctness oracle, and benchmark comparator
- prototype Tokei behind an optional Rust feature using `parse_from_slice`
- keep traversal, file opening, binary gating, analyzer versioning, caching, and rollups
  inside fdu
- implement the generic basic analyzer directly because it is small and on the hot path

The older Rust `loc` project is not a current candidate.
Its last reviewed commit is from 2022, and its README directs users to SCC for speed and
accuracy or Tokei for Rust reuse.

### Tool Dialects Differ Enough to Require Versioned Golden Fixtures

On the same SCC source tree with ignore rules disabled, SCC master reported 1,827 files,
591,234 physical lines, 437,910 code lines, and 88,538 comment lines.
Tokei reported 1,683 files, 585,242 physical lines, 463,759 code lines, and 57,666
comment lines. Neither result is inherently the universal truth.
They recognize different files and languages and make different choices for assembly,
GraphQL, Markdown, docstrings, and embedded content.

fdu should publish a named metric dialect and keep small adversarial fixtures per common
language. Fixtures need strings containing comment markers, code plus trailing comments,
nested multiline comments, blank lines inside comments, raw strings, docstrings,
unterminated constructs, CRLF, and a final line without `\n`. Changes to expected
results require an analyzer-version change and cache invalidation.

The first target need not be every format in Linguist.
“Common and accurate” is more valuable than “hundreds and silently approximate.”
A coverage report can make the long tail visible while the supported set grows.

### Content Metrics Belong Outside the Metadata Critical Path

The architecture should treat content as derived data:

```text
metadata scan
    -> exact bytes and file/type counts
    -> optional bounded content queue
        -> open once and inspect first chunk
        -> binary or unsupported encoding: stop
        -> generic text pass
        -> optional code or markup analyzer
        -> cache result by fingerprint and analyzer versions
        -> emit additive reducer delta
```

The queue must be bounded by bytes as well as file count.
Large files, minified files, and generated files need explicit policy rather than hidden
tool defaults. Content workers should not inherit the metadata walker’s concurrency
blindly, and a Tokei prototype must prevent nested Rayon work from oversubscribing fdu’s
own pool.

The cache key should include at least:

```text
(file fingerprint, type_id, type_rules_version, analyzer_id, analyzer_version, options)
```

The value contains slots, coverage, and detection provenance.
Per-directory rollups are persisted by analyzer.
A changed file subtracts its old slots and adds its new slots.
Changing type rules or analyzer semantics invalidates only the affected derived tier,
not the metadata snapshot.

### Exploratory Performance Results

The source-corpus benchmark used SCC’s current checkout including vendored Go modules,
with `.git` excluded and ignore-file behavior disabled.
The corpus had 1,920 regular files and 19,876,258 bytes.
SCC recognized 1,827 files and 19,583,229 bytes; Tokei recognized 1,683 files.
All runs used a warm operating-system cache on an Apple M1 Pro.
Each tool received 10 warmups and 100 measured runs.
Medians are more useful than means here because a few scheduler outliers were large.

| Complete tool | Median wall | Work reported | Interpretation |
| --- | ---: | --- | --- |
| Tokei 14.0.0 | 42.3 ms | 1,683 files; 585,242 lines | Fastest here, but narrower recognized set |
| SCC 3.7.0 | 49.6 ms | 1,819 files; 591,062 lines | Current stable SCC speed envelope |
| SCC master, 4.0 beta | 65.4 ms | 1,827 files; 591,234 lines | More languages and new ambiguity logic; development snapshot |

This is not a fair accuracy race because the inputs accepted and the resulting dialects
differ. It does show that exhaustive code/comment/blank counting over roughly 20 MB is a
tens-of-milliseconds job once data is cached.
Cold storage, a million tiny files, and incremental churn have different physics.

The in-memory SCC-package spike used a repeated 9,000-byte Go-like buffer and ten 750 ms
benchmark samples:

| Analysis depth | Approximate median | Approximate throughput | Allocation behavior |
| --- | ---: | ---: | --- |
| SIMD newline count | 0.19 us | 47 GB/s | None |
| Lines, blanks, raw words, and 10 KB NUL check | 15.3 us | 590 MB/s | None |
| SCC code/comment/blank state machine | 34.3 us | 260 MB/s | Small parser overhead |
| SCC state machine plus per-byte classification | 39.6 us | 225 MB/s | One extra byte per input byte |

The SIMD number is an L1-cache microbenchmark, not attainable end-to-end filesystem
throughput. The useful comparison is the relative ladder: several generic tallies fit in
one allocation-free pass, language-aware comments roughly doubled CPU on this input, and
byte classification added about 15% plus linear memory.

On the ordinary source corpus, SCC medians with its 10 KB binary check enabled and
disabled differed by about 1 ms in the direction opposite an expected overhead.
The probe cost was below run-to-run noise.
A deliberately adversarial corpus then named 32 hard links to a 23.7 MB executable as
`.go`, for 758 MB of logical input.
With three warmups and 20 runs, the NUL-enabled median was 87.9 ms; disabling the check
and parsing the binaries as Go took 320.8 ms and invented 3,011,968 source lines.
The probe was 3.65 times faster and prevented invalid metrics.

SCC reads each complete file before its NUL check.
fdu can do better for real binaries: open once, inspect the first buffered chunk, stop
immediately for binary content, or continue reading from the same handle for text.
The text path pays no second open and the binary path avoids the full read.

## Key Insights

1. **Metadata bytes by type are nearly free; content metrics are not.** File size comes
   from the existing stat record.
   No content analyzer should be required to answer “which file types occupy space?”
2. **Binary detection is primarily a correctness gate.** Its ordinary cost is beneath
   end-to-end noise, while failing to gate a mislabeled binary can create expensive and
   meaningless LOC.
3. **A metric name is a versioned contract.** `nonblank_lines`, code SLOC, GitHub bytes,
   and parser-level statements are different measurements.
   Calling all of them LOC would make the cache untrustworthy.
4. **Deep detection should be conditional.** Most files resolve from exact names or
   extensions. Content heuristics belong only on ambiguous candidates.
5. **The first content pass should fuse independent cheap tallies.** Once bytes are in
   cache, counting lines, blanks, words, and a NUL prefix in one pass is cheaper than
   rereading for separate features.
6. **Reader-visible prose is a projection, not raw source.** Logical words solve volume
   normalization; they do not remove Markdown URLs, code, and hidden markup.
   Projection and word normalization are separate layers.
7. **Reuse a parser without surrendering the engine.** Tokei’s per-buffer API is the
   strongest Rust candidate, while fdu must retain I/O, concurrency, caching, and rollup
   control.

## Comparison Matrix

| Option | Primary value | Breadth at reviewed revision | Per-buffer library API | Fit for fdu | Main concern |
| --- | --- | ---: | --- | --- | --- |
| GitHub Linguist | Detection and repository policy | 828 types | Ruby blob API | Reference and fixture source | Heavy runtime; byte rollup, not comment-aware LOC |
| SCC 3.7/master | Fast code metrics and algorithm design | 365 definitions on master | Go package | Oracle, comparator, and source of techniques | Go integration or a maintained Rust port |
| Tokei 14 | Rust code/comment/blank parser | 332 definitions | Yes, `parse_from_slice` | Best reuse prototype | Dependency weight, nested concurrency, narrower set in the spike |
| loc 0.5 | Older Rust code counter | About 100 listed types | CLI-oriented | Eliminated | Stale; project recommends SCC or Tokei |
| fdu basic analyzer | Lines, blanks, words, binary gate | Family-based | Native | Ship first | Does not provide comment-aware LOC |
| `content_inspector` | BOM and NUL text/binary guess | Encoding families | Yes | Algorithm reference | Heuristic is too small to justify a dependency by itself |
| `infer` | Binary magic signatures | Common binary formats | Yes | Optional later layer | Not a programming-language detector |
| `tree_magic_mini` | MIME tree and host database | Broad MIME set | Yes | Eliminated from default path | Runtime database, licensing mode, and per-file cost |
| `unicode-segmentation` | UAX #29 boundaries | Unicode words | Yes | Optional exact-boundary tool | Not the normalized logical-word measure; dictionary scripts still need tailoring |
| `pulldown-cmark` | Streaming Markdown events | CommonMark and options | Yes | Best markup-projection prototype | Markdown only; projection contract still belongs to fdu |

## Options Considered

### Option A: Build Every Analyzer in fdu

**Description:** Implement basic metrics, language detection, comment parsers, markup
projection, encoding, and all language tables directly.

**Pros:**

- Full control over allocations, concurrency, binary size, versioning, and cache shape
- No dependency behavior hidden below the analyzer boundary
- Can optimize exactly for incremental per-file operation

**Cons:**

- Recreates years of language edge cases and fixtures
- Requires ongoing synchronization with hundreds of evolving language definitions
- High risk of fast but subtly wrong LOC

### Option B: Use Tokei for the Entire Content Tier

**Description:** Let Tokei walk and count trees, then adapt its aggregate report.

**Pros:**

- Mature Rust implementation and broad language support
- Fast in the local end-to-end spike

**Cons:**

- Duplicates fdu’s walker, ignore handling, I/O, concurrency, cache, and rollups
- Does not naturally support per-file fingerprint caching and changed-file deltas
- Makes coverage and file-type inclusion inherit Tokei policy

### Option C: fdu Pipeline With Reusable Per-Buffer Analyzers

**Description:** fdu owns detection, opening, binary gating, scheduling, versioning,
caching, coverage, and reducer deltas.
A basic analyzer is native.
Optional code and markup analyzers receive an already-read buffer and a resolved type.

**Pros:**

- Preserves the metadata fast path and incremental architecture
- Reuses Tokei or another parser where correctness work is largest
- Lets basic text and binary handling stay allocation-light
- Makes analyzer replacement and side-by-side validation possible

**Cons:**

- Requires a clean adapter contract and analyzer-version discipline
- Per-buffer libraries may still bring large dependency or thread-pool surfaces
- Needs cross-tool fixtures to detect semantic drift

### Eliminated Options

- **Run SCC, Tokei, or Linguist as a subprocess:** startup, serialization, deployment,
  cancellation, and Python-embedding behavior conflict with fdu’s library contract.
- **Embed GitHub Linguist:** its Ruby and native dependency stack is disproportionate,
  and its primary rollup is bytes rather than comment-aware LOC.
- **Adopt `tree_magic_mini` on the default path:** MIME-level depth is unnecessary for
  deciding whether code/text metrics are safe, and its database and licensing modes
  complicate portable embedding.
- **Call nonblank lines “LOC”:** this would be fast but semantically misleading.
- **Count raw markup as prose pages:** destinations, code, and hidden syntax inflate the
  human-reading measure.

## Recommendations

Adopt Option C and implement it as a capability ladder.

### Stage 0: Metadata Type Rollups

- Roll up existing logical/apparent and allocated byte metrics by stable type ID and
  content family.
- Resolve exact names and extensions without opening files.
- Preserve unknown extensions and no-extension groups.
- Keep this available in every default scan.

### Stage 1: `content-basic-v1`

- Make content analysis explicit and opt-in.
- Open once, inspect an 8-16 KB first chunk, recognize byte-order marks, and check NUL.
- Analyze UTF-8 and ASCII text; report coverage for skipped encodings and binaries.
- In one streaming pass, count physical, blank, and nonblank lines.
- Count raw words only for prose-family files.
- Derive fixed-word pages from aggregate words at query time, with the words-per-page
  constant shown in output.
- Cache by fingerprint, type rules, analyzer ID, analyzer version, and options.

### Stage 2: `code-sloc-v1` Prototype

- Add a feature-gated Tokei 14 prototype that calls `LanguageType::parse_from_slice` on
  fdu-owned buffers; do not use Tokei’s walker.
- Benchmark dependency size, compile time, per-file latency distribution, large-file
  nested parallelism, RSS, cold reads, cached reruns, and 1% churn.
- Compare common-language fixtures and real repositories against SCC 3.7, SCC master,
  Tokei, and the named fdu dialect.
- Ship Tokei only if the adapter meets fdu’s binary-size, concurrency, supply-chain, and
  semantic-version requirements.
  Otherwise port a narrow SCC/Tokei-style state machine and grow its language table from
  measured demand.

### Stage 3: `text-logical-v1` and `markdown-prose-v1`

- Implement streaming logical-word sufficient statistics with a pinned Unicode rule
  version.
- Retain raw words next to logical words.
- Add a `pulldown-cmark` spike for FlexDoc-compatible visible-prose projection.
- Exclude URLs, reference definitions, inline code, code blocks, frontmatter, and hidden
  HTML syntax; include link labels and image alt text.
- Store words and coverage; derive pages after rollup.

### Stage 4: Deeper Detection and Specialized Analyzers

- Add bounded ambiguity heuristics on at most the first 20 KB.
- Add modelines, XML/manpage rules, generated/vendor/documentation flags, and optional
  binary magic names only when their use cases justify them.
- Add HTML, notebook, reStructuredText, and other projections as separate analyzers.
- Keep per-byte classification, ULOC, tokenizer estimates, and AST-derived metrics
  opt-in and separately versioned.

## Next Steps

- [x] Convert this research into
  [a content-metrics specification](../specs/active/plan-2026-08-12-fdu-file-content-metrics.md)
  linked to `fdu-3n8c`.
- [ ] Define stable type, family, detection-provenance, analyzer, slot, and coverage
  schemas.
- [ ] Implement and benchmark `content-basic-v1` behind an opt-in feature or command.
- [ ] Build the Tokei per-buffer adapter spike without changing fdu’s walker.
- [ ] Add a common-language adversarial golden corpus and cross-tool comparison report.
- [ ] Add named cold, warm, cached, and 1%-churn content jobs to the performance ledger.
- [ ] Prototype the Markdown prose projection and logical-word streaming accumulator.

## Methodology

The source review pinned these revisions:

- fdu worktree base `11bdcde228e877a9f28c92f253a2b6735dc6228d`
- SCC master `50ea91a853f94fa581e6d505b85b0aef944bd7b5` and release 3.7.0
- GitHub Linguist `2e859d2fff68b2646e2f45eb230aba2cde431535`
- Tokei `fa44e5194060305576514d59b850353643afbfc8`
- loc `1e0c7f434ddfd51439e1d4eb126f31b7a04229d9`
- FlexDoc `b2043fd5ce45fcc0662b6026bfd18ac1629ad737`
- logical-word gist revision `0d730285ae7ae3046ba535a3d325a745e781273b`

Third-party source was checked out under ignored `attic/` paths.
SCC 3.7.0’s downloaded archive matched its published SHA-256 checksum.
SCC master was built from its vendored module tree with network access disabled.
The host was macOS 26.5.2 on Apple M1 Pro, with Rust 1.97.1 and Go 1.26.5. Hyperfine
drove the exploratory complete-tool runs.
Go’s benchmark harness drove the in-memory analysis-depth spike.

The binary experiment used hard links, so its 758 MB logical corpus consumed little
additional physical storage.
It was moved to the macOS Trash after measurement.

The linked [metabrowser PR #24](https://github.com/jlevy/metabrowser/pull/24) was also
audited. At the time of review it was a Git graph feature and contained no
language-detection or LOC research in its body, files, or comments.
This brief does not silently attribute findings to that PR; it uses the earlier fdu
research and the pinned primary sources instead.

## References

- [GitHub Linguist: how detection works](https://github.com/github-linguist/linguist/blob/2e859d2fff68b2646e2f45eb230aba2cde431535/docs/how-linguist-works.md)
- [GitHub Linguist blob metrics and inclusion policy](https://github.com/github-linguist/linguist/blob/2e859d2fff68b2646e2f45eb230aba2cde431535/lib/linguist/blob_helper.rb)
- [GitHub REST repository languages endpoint](https://docs.github.com/en/rest/repos/repos#list-repository-languages)
- [SCC source at the reviewed master revision](https://github.com/boyter/scc/tree/50ea91a853f94fa581e6d505b85b0aef944bd7b5)
- [SCC 3.7.0 release](https://github.com/boyter/scc/releases/tag/v3.7.0)
- [Tokei source at the reviewed revision](https://github.com/XAMPPRocky/tokei/tree/fa44e5194060305576514d59b850353643afbfc8)
- [loc source at the reviewed revision](https://github.com/cgag/loc/tree/1e0c7f434ddfd51439e1d4eb126f31b7a04229d9)
- [Rust `content_inspector` documentation](https://docs.rs/content_inspector/0.2.4/content_inspector/)
- [Rust `infer` documentation](https://docs.rs/infer/0.22.0/infer/)
- [Rust `tree_magic_mini` documentation](https://docs.rs/tree_magic_mini/3.2.2/tree_magic_mini/)
- [Rust `unicode-segmentation` source and documentation](https://github.com/unicode-rs/unicode-segmentation)
- [Rust `pulldown-cmark` documentation](https://docs.rs/pulldown-cmark/0.13.4/pulldown_cmark/)
- [FlexDoc](https://github.com/jlevy/flexdoc/tree/b2043fd5ce45fcc0662b6026bfd18ac1629ad737)
- [Logical Word Count proposal and validation](https://gist.github.com/jlevy/0d6d87885f6d85f31440e58b8cfce663)
- [Original fdu file-rollup research](research-2026-08-06-file-rollup-engine.md)
- [fdu performance-frontier research](research-2026-08-10-performance-frontier.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
