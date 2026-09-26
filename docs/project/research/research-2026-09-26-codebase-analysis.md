# Research: Accurate, Concise Codebase Analysis

**Date:** 2026-09-26

**Author:** Codex, with delegated empirical analysis

**Status:** Complete research; implementation proposed

## Overview

`fdu --analyze=code .` should answer: **How much code is here, which languages is it
written in, and where should I look next?** The report should explain its scope and
coverage in a few lines, then show a language breakdown and the files or directories
that dominate the codebase.
Its calculation cost should remain proportional to the files the caller asked to
analyze.

The current implementation provides much of the necessary foundation: versioned content
metrics, language classification, ignored-state selection, per-file records, rollups,
coverage reporting, and one engine shared by Rust, Python, and the command line.
However, direct comparisons expose parsing errors in valid source, and ignored-file
selection currently reduces reported totals without reducing content reads.

The recommended sequence is:

1. Fix demonstrated counting errors and preserve their minimal reproductions.
2. Use one ignored-population option; derive traversal and body reads from the requested
   measurements, pruning ignored subtrees when excluded.
3. Add a code-first overview with separate populations, coverage, and metric sorting.
4. Compose existing request controls for less common combinations of populations and
   measurements.
5. Evaluate optional structural complexity only after the counting layer is reliable.

This follows the earlier
[content-metrics research](research-2026-08-12-fast-file-content-metrics.md), which
already separates classification, measurement, and inclusion policy.
It supplies new evidence about the shipped implementation and a narrower next increment.

## Questions to Answer

1. Is the shipped line counter comparable in quality to an established counter?
2. What does GitHub actually measure, and which aspects should fdu match or improve?
3. What can Metabrowser teach us about a concise directory overview?
4. How should ignored, generated, vendored, and project source be distinguished?
5. Which additional measurements justify their implementation and runtime costs?

## Scope

The empirical baseline is released fdu 0.1.0 versus locally available Tokei 14.0.0.
Source inspection uses fdu revision `e06c3e084dca7eec60cd2f1216f942c43c4398f0` and
Metabrowser revision `091d4043`. The Metabrowser review examines its production overview
models and associated tests; it does not claim a live-browser usability study.

The study includes matched-file comparisons, hand-classified syntax examples, the GitHub
Linguist and crates.io counting pipelines, analysis cost, and proposed output.
It does not implement new defaults, change dependencies, or establish performance
benchmarks. Cache-location recommendations from the same discussion are recorded
separately below because they affect operation rather than the meaning of code metrics.

## Findings

### SLOC Quality Requires Adjudicating Disagreements

Both tools aim to count physical lines containing code, with separate comment and blank
counts.
Agreement is useful evidence, but neither implementation is a correctness oracle.
The study uses identical file contents and scope, with fdu caching disabled and Tokei
ignore rules disabled, so disagreements cannot be explained by one tool skipping tests
or vendor directories.

Across 30 deliberately adversarial fixtures, fdu matched 14 hand-classified expectations
and Tokei matched 21; eight cases failed in both.
These numbers describe the selected diagnostic corpus, not general accuracy rates.
The fixture sources, expected partitions, actual results, and real-file hashes are
preserved in [the comparison evidence](evidence/codebase-analysis-2026-09-26.json).

Every tuple below is **(code, comment, blank)**:

| Construct | Expected | fdu 0.1.0 | Tokei 14.0.0 |
| --- | --- | --- | --- |
| Rust ordinary multiline string | (3, 0, 0) | (2, 1, 0) | (3, 0, 0) |
| Rust lifetime before a block comment | (2, 1, 0) | (3, 0, 0) | (1, 2, 0) |
| Rust raw string | (3, 0, 0) | (3, 0, 0) | (3, 0, 0) |
| C++ raw string | (3, 0, 0) | (2, 1, 0) | (3, 0, 0) |
| Java text block | (4, 0, 0) | (3, 1, 0) | (4, 0, 0) |
| C# verbatim string | (3, 0, 0) | (2, 1, 0) | (3, 0, 0) |
| JavaScript regex containing `/*` | (2, 0, 0) | (1, 1, 0) | (1, 1, 0) |
| Go raw string | (4, 0, 0) | (4, 0, 0) | (3, 1, 0) |
| Shell heredoc | (4, 0, 0) | (3, 1, 0) | (3, 1, 0) |
| SQL ordinary multiline string | (3, 0, 0) | (2, 1, 0) | (3, 0, 0) |

Two minimal, valid fdu failures illustrate the mechanism:

```rust
const S: &str = "first
// text
last";
```

All three lines belong to a string declaration, but fdu counts its middle line as a
comment because ordinary quote state is reset at a newline.

```javascript
const re = /[/*]/;
const answer = 42;
```

Both lines contain code, but both counters interpret characters inside the regex as the
beginning of a block comment and misclassify the second line.

The real-file comparison used 107 identical source files from the fdu repository:

| Source | Files | fdu Code Lines | Tokei Code Lines | Files with Different Code Counts |
| --- | ---: | ---: | ---: | ---: |
| Rust | 80 | 72,870 | 73,877 | 28 |
| Python | 7 | 4,880 | 4,880 | 0 |
| JavaScript | 19 | 3,391 | 3,458 | 5 |
| Shell | 1 | 64 | 64 | 0 |
| Total | 107 | 81,205 | 82,279 | 33 |

The largest discrepancy is attributable to Tokei: for `scan.rs`, it reports 9,089 code
lines versus fdu’s 8,165. Replacing a single Rust double-quote character literal with an
ordinary character in a temporary copy drops Tokei to 8,164. This minimal reproduction
confirms the problem:

```rust
fn main() { let c = '"'; }
// comment

fn other() {}
```

Expected and fdu: `(2, 1, 1)`; Tokei: `(4, 0, 0)`. The remaining one-line difference in
the modified real file was not adjudicated.
Consequently, the aggregate discrepancy does not measure fdu’s error.
The defensible conclusion is that fdu has demonstrated correctness gaps and equal parser
quality is not established; replacing it with this comparator would also introduce
errors.

fdu’s current contract deliberately counts mixed code/comment lines as code, blank lines
inside block comments as comments, and multiline string contents as code.
Python docstrings count as code in `code-sloc-v1`. Differences caused by those choices
must be separated from failure to recognize a language’s string or comment syntax.
See [the counter](../../../crates/fdu-core/src/content/content_code_metrics.rs).

Existing coverage includes a 15-language
[golden fixture project](../../../tests/golden/fixtures/code-project) and
[CLI analysis sessions](../../../tests/golden/cli-content.tryscript.md).
The new findings identify missing syntax cases; they do not imply that language-level
fixtures were absent.

### GitHub Measures Language Share by Bytes

GitHub’s repository language bar uses Linguist to identify languages, exclude categories
such as generated and vendored content, and compute shares by bytes.
It analyzes the default branch, whereas fdu examines a local filesystem scope that can
include untracked and ignored files.
Matching GitHub percentages therefore requires matching both the population and the
measurement. A code-line percentage should be labelled as such.
[Linguist’s explanation](https://github.com/github-linguist/linguist/blob/main/docs/how-linguist-works.md)
and
[override rules](https://github.com/github-linguist/linguist/blob/main/docs/overrides.md)
describe the implementation.

For this work, “at least as good as GitHub” means an immediately legible language
breakdown, useful handling of generated/vendor material, explainable classification, and
a way to override it, enriched with actual code/comment/blank counts and local directory
navigation. It is not a claim that fdu already matches Linguist’s language breadth or
detection quality. Ambiguous extensions, embedded languages, notebooks, and
`.gitattributes` overrides need a separate compatibility corpus.

### crates.io Applies a Package-Specific Definition

crates.io runs Tokei over a published package archive, filters several test/example/
benchmark paths and non-code languages, and treats docstrings as comments.
Inline Rust unit tests inside accepted source files still count.
Its current source pins Tokei 15.0.0; the empirical comparison here uses installed
14.0.0, and neither fact proves which version generated an existing registry count.
See the registry’s
[counter](https://github.com/rust-lang/crates.io/blob/main/crates/crates_io_linecount/src/lib.rs),
[path filters](https://github.com/rust-lang/crates.io/blob/main/crates/crates_io_linecount/src/paths.rs),
and
[dependency](https://github.com/rust-lang/crates.io/blob/main/crates/crates_io_linecount/Cargo.toml).

An exact crates.io comparison profile could be useful, but it should be explicit.
Tests and examples are maintained code and should remain visible in a general codebase
overview. No numerical crates.io release-count comparison was completed in this study.

### Metabrowser Separates Populations Before Showing the Breakdown

Metabrowser’s File Overview combines totals and type distribution under one section.
It offers a Files/Bytes selector beside Show ignored and persists that preference.
The current default includes ignored files.
The totals retain separate non-ignored, ignored, and combined populations; changing Show
ignored changes the detailed distribution without moving those fixed populations.
The distribution’s denominator changes with its selected population.

Rows are ordered by the selected measure, with deterministic tie-breaks.
Small contributions and long tails are folded into an explicit remaining group.
The examined implementation uses ten rows per subsection and a one-percent visibility
threshold; fdu should derive its own bound from the CLI’s available space and preserve
the
[requirement to expose every truncation](../architecture/fdu-design-principles.md#truncate-freely-never-truncate-silently).

The useful lessons are one scope control, stable population totals, a clear denominator,
and summary before detail.
The overview’s primary measurements are file counts and bytes; it does not provide
evidence of an existing SLOC or complexity analyzer.
Its display toggle also does not establish that ignored directories were never scanned.

Production source:
[panel composition](https://github.com/jlevy/metabrowser/blob/091d4043/src/metabrowser/builtin_plugins/folder/file-overview-panel.js),
[controls](https://github.com/jlevy/metabrowser/blob/091d4043/src/metabrowser/builtin_plugins/folder/rollup-controls.js),
[population totals](https://github.com/jlevy/metabrowser/blob/091d4043/src/metabrowser/builtin_plugins/folder/folder-totals.js),
[breakdown model](https://github.com/jlevy/metabrowser/blob/091d4043/src/metabrowser/builtin_plugins/folder/file-type-summary-model.js).

### Excluding Ignored Files Currently Saves Presentation, Not Content Reads

The existing commands already select non-ignored or ignored report populations:

```shell
fdu --analyze=code . --exclude-ignored
fdu --analyze=code . --only-ignored
```

A cold fixture containing `.gitignore`, one one-line `main.rs`, and an ignored Rust file
with 100 code lines produced these results with fdu 0.1.0:

| Request | Reported Code Lines | Fresh Files Analyzed | Content Read |
| --- | ---: | ---: | ---: |
| Include everything | 101 | 3 | 1.7 KiB |
| `--exclude-ignored` | 1 | 3 | 1.7 KiB |

The file contents were `.gitignore` = `ignored/\n`, `main.rs` = `fn main() {}\n`, and
`ignored/vendor.rs` = `const X: u32 = 1;\n` repeated 100 times.
Both invocations used `--cache=off`; the shipped performance summary supplied the
fresh-file and read-volume observations.
These are work measurements, not timing claims.

Source inspection agrees: `AnalysisRequest` contains an analyzer set and worker count,
and candidate construction visits indexed regular files before query selection.
See [analysis settings](../../../crates/fdu-core/src/content/content_model.rs),
[candidate construction](../../../crates/fdu-core/src/index.rs), and
[analysis execution](../../../crates/fdu-core/src/content/content_analysis.rs).

Two distinct improvements follow:

- **Skip ignored content reads:** retain metadata totals where needed, but do not open
  ignored bodies for a request that only needs non-ignored content.
- **Prune ignored traversal:** avoid visiting ignored subtrees at all, accepting that
  exact ignored file/byte totals are then unavailable.
  The execution plan must distinguish these costs, but they do not require separate
  ignored-file switches.

If ignored files were not analyzed, their code-line total must say “not analyzed”.
If their directories were not traversed, their file/byte totals must say “not scanned”.
Neither is zero. A display filter alone should never promise either saving.

### Ignored Is Not the Same as First-Party or Tracked

Non-ignored files may be untracked, generated, or vendored; ignored files may contain
valuable handwritten code.
The overview should use the precise labels “non-ignored” and “ignored”, rather than
treating either category as an authorship judgment.

fdu already has bounded generated-content probes and path-based vendor/documentation
flags in
[file-type detection](../../../crates/fdu-core/src/classify/file_type_detection.rs).
These are useful inputs, but they are heuristics and can overlap.
For example, vendored documentation may also be generated.
They must not be displayed as disjoint slices unless an explicit partition rule assigns
each file to one slice.
Nor should existing flags be confused with implemented Linguist `.gitattributes`
compatibility.

## Comparison Matrix

| Capability | fdu Today | Useful Next Increment | Cost Judgment |
| --- | --- | --- | --- |
| Code/comment/blank counts | Versioned counter; confirmed syntax gaps | Correct demonstrated lexical cases | Medium engineering; same read pass |
| Language overview | Shares already use code lines under `--analyze=code` | Code-first columns, totals, explicit scope and coverage | Low to medium |
| Ignored populations | Report selection exists; content still read | Separate totals; selected analysis avoids reads | Medium; scope/cache work required |
| Generated/vendor labels | Some flags already exist | Expose flags and explain rules before broad exclusions | Low to medium |
| Largest source files | Per-file metrics exist | Rank by code lines with paths and bounded remainder | Low to medium; no extra body reads |
| Directory concentration | Hierarchical index exists | Code-line shares for dominant directories | Low to medium; reducer/query work |
| Size distribution | Per-file counts available | Maximum and top contributors first; quantiles later | Low for top-k; quantiles need a policy |
| Branching estimate | Absent | Optional language-specific lexical estimate | Medium to high; measure overhead |
| Exact function complexity | Absent | Defer pending demand for parser integration | High |

Costs are relative implementation judgments, not measured estimates or commitments.

## Options Considered

### Improve the Existing Streaming Counter

This retains a single read pass, current metric contracts, and existing cache
integration. Minimal syntax regressions are inexpensive to add, and several missing
states are localized.
However, supporting more languages creates ongoing lexer maintenance.
Fixing every new disagreement independently can grow into maintaining a language parser
collection without an explicit decision to do so.

### Adopt a Mature Counter Behind the Engine Boundary

A library such as Tokei offers broader syntax handling and a larger upstream corpus.
It also introduces dependency, API, memory, and semantic integration work, and the
observed Tokei defects show that adoption cannot replace independent expectations.
The earlier [content-metrics research](research-2026-08-12-fast-file-content-metrics.md)
considered this approach.
Revisit it with the newly established fixture corpus if the native fixes become
expensive. Compare a current reviewed release before deciding.

### Add Complexity Through a Small Lexical Estimate

SCC illustrates a modest approach: count selected branch/loop tokens while already
scanning code. Its documentation explicitly limits comparison to similar-language code
and distinguishes the estimate from exact cyclomatic complexity.
See [SCC’s explanation](https://github.com/boyter/scc#complexity-estimates).

For fdu, an optional `branch_points`-style metric could help rank files within a
language once strings and comments are correctly recognized.
The spelling and option remain proposals.
Publish the token rules and coverage; avoid presenting a cross-language sum as a
universal quality score.
Large files by SLOC provide a useful first step without inventing a complexity metric.

### Defer Full Parsing, History, and Cost Estimates

AST parsing, function-level cyclomatic/cognitive complexity, duplication detection,
repository-history churn, and maintainability scores have substantially different
storage or runtime costs.
They need separate explicit analyzers and evidence of demand.
Do not put them into the default code overview during this increment.
Likewise, estimated engineering effort or dollar cost is not a reliable consequence of a
line count and does not help the immediate orientation task.

## Proposed Design

### Design for the Alpha Product Directly

The owner has confirmed that backward compatibility does not constrain this design.
Replace redundant flags and public fields together across Rust, Python, the CLI, and
their documentation.
Do not retain deprecated aliases, duplicate request models, or old cache readers merely
to preserve the alpha interface.
Continue rejecting incompatible cache data and rebuilding it; alpha status does not
justify serving results calculated under different rules.
Change the report schema identifier when its shape or meaning changes, and update all
public models and goldens together rather than retaining adapters for the old alpha
shape.

The proposal below is the recommended target design, not implemented behavior.
The common interface should express the answer wanted; the engine derives the work
needed to produce it.

### Separate Population, Measurement, and Presentation

Start with the facts a caller wants, rather than the stages the implementation runs.

| Facet | Question | Values or Examples |
| --- | --- | --- |
| Population | Which entries contribute to this answer? | `--ignored` and existing path/kind predicates |
| Measurement | Which facts are needed about those entries? | Metadata by default; `--analyze` requests content metrics |
| Presentation | How are measured facts exposed? | `--view`, format, sorting, and paging |

Ignored state is a classification with two known values, plus an unavailable state when
rules could not be observed.
The three population choices are subsets of those two known classes.
“Scan” and “report” are execution stages, and “separate” is a presentation choice; none
is another value of ignored state.

For each population, the caller can request no contribution, metadata, or metadata plus
content metrics. The resulting work follows:

- No contribution permits pruning when rule semantics prove the subtree cannot contain
  an eligible entry.
- Metadata requires discovery and stat work, but no body reads for content analysis.
- Content metrics require reading eligible bodies for the requested analyzers.

Discovery can still need ancestor directory enumeration and ignore-control reads.
These are prerequisites, not an obligation to include those ancestors in the reported
population. A promise to exclude ignored content must distinguish this control work from
opening ignored bodies for analysis.

### One Everyday Option Selects the Population and Its Work

Replace the ignored booleans with:

```text
--ignored=include|exclude|only
```

| Value | Answer | Required Work |
| --- | --- | --- |
| `include` | Both populations, with separate contributions and an all-files total | Discover both; requested content analysis covers both by default |
| `exclude` | Non-ignored files | Prune effectively ignored subtrees; do not analyze ignored bodies |
| `only` | Ignored files | Discover ignored matches through non-ignored ancestors; do not analyze non-ignored bodies |

The two common modes are `include` for a full overview and `exclude` for a cheaper
non-ignored overview.
`only` has a concrete diagnostic use: inspect dependency trees, build output, or
handwritten code hidden by ignore rules.
It cannot generally prune non-ignored directories, because they may contain ignored
children. The same selection vocabulary therefore does not promise symmetric costs.

There is no additional `--scan-ignored` option.
An exclusion changes the answer’s population and enables the engine to avoid work for
that population. Display limits remain presentation: requesting the ten largest files by
code lines still requires counts for every eligible file.
A content-dependent predicate likewise requires obtaining the metrics it tests.

Use the same effective ignore rules for admission and reporting, including ancestor
exclusions and supported negations.
Read necessary controls before deciding a subtree is safely prunable.
Do not invent Git-index semantics: `.gitignore` classification does not mean tracked
versus untracked, and `.git` itself needs an explicit scope policy if excluded.
If rules cannot be read or are refused under a budget, preserve an explicit
unknown/partial outcome; do not infer that unknown entries are non-ignored or silently
prune them as ignored.

### Reuse the Existing Measurement and View Controls

`--analyze` already selects content measurements such as code, lines, and words.
`--view` selects how the available measurements are presented.
Both operate on the population selected for that request.
A view must not silently enable an analyzer or change the population.
There is no additional measurement-scope flag.

These controls express the six uniform cases directly:

| Non-Ignored | Ignored | Request |
| --- | --- | --- |
| Metadata | Omitted | `fdu . --ignored=exclude` |
| Content | Omitted | `fdu . --ignored=exclude --analyze=code` |
| Omitted | Metadata | `fdu . --ignored=only` |
| Omitted | Content | `fdu . --ignored=only --analyze=code` |
| Metadata | Metadata | `fdu . --ignored=include` |
| Content | Content | `fdu . --ignored=include --analyze=code` |

“Content” includes metadata and the explicitly requested analyzer set.
Views can vary independently in every row, subject to their metric requirements.
All syntax using `--ignored` is proposed.

There are two remaining combinations: metadata for both populations with content metrics
for just one.
For example, someone may want whole-tree disk usage and code counts limited
to non-ignored files.
Express those as two ordinary requests:

```shell
fdu . --ignored=include
fdu . --ignored=exclude --analyze=code
```

To analyze only ignored output alongside whole-tree disk usage, use `only` in the second
request. Different analyzers for different populations likewise use ordinary requests
with the respective `--ignored`, `--analyze`, and `--view` values.
Each answer keeps one population and an unambiguous denominator.
This covers all eight nonempty combinations of omitted, metadata, and content across the
two populations without adding another scope control.

The initial CLI produces separate reports for these compositions.
A library application can compose their results and reuse compatible retained metadata
where the lifecycle supports the requested analysis.
If a single combined CLI report becomes useful, it should compose these same request
objects; it should not introduce a second interpretation of `--ignored`. Cache reuse is
an optimization and must preserve each request’s cold answer.

### When Is Scanning without Reporting Useful?

A one-shot request should not deliberately measure facts that contribute to no requested
row, aggregate, denominator, or coverage statement.
Showing a summary while hiding individual rows is still reporting those files.
Sorting or selecting by a content metric can also require reading a file that ultimately
has no displayed row.
Neither case needs a second ignored-file policy.

Preparing a retained index for later interactive queries is a genuine separate use case.
A browser can discover both populations once and query either as its user changes the
view. The retained-root API already distinguishes discovery from queries; preserve that
capability, and state what additional discovery is needed when a query asks beyond the
retained coverage. Cache warming belongs to an explicit preparation lifecycle with
declared scope. It should not make an ordinary excluded-population report silently scan
everything.

Grouping and row focus remain query/presentation operations over the available facts.
An application may show only ignored rows while preserving a whole-tree summary.
It must name each aggregate’s population; applying a row filter must not silently
redefine its denominator.
No extra CLI control is needed merely to keep this expressible through engine queries.

### Use Stable Defaults and Explicit Unavailable States

Default `--ignored` to `include` for metadata and content requests alike: the question
is “what is here, and how much belongs to each population?”
The code overview separates the populations so ignored dependencies cannot obscure the
non-ignored code contribution.
Content reads remain opt-in through `--analyze`; `--ignored=exclude` is the explicit
choice for the cheapest non-ignored overview.
Views, formats, sorting, workers, cache policy, and enabling an analyzer do not change
the selected population.
Rust, Python, and the CLI use the same typed resolver.

`--no-gitignore` disables classification and is compatible with `include` only.
The report must label the population unclassified, rather than labelling every entry
non-ignored.
Refuse `exclude` and `only` when the classification needed to interpret them
is disabled.

Machine output records the resolved population, requested analyzers, and coverage; human
summaries state them concisely.
Pruned populations have “not scanned” metadata, not zero counts.
Metrics whose analyzers were not requested are absent, not zero.
Read failures remain incomplete measurements within the requested scope.
Partial ignore classification preserves its own unknown contribution.
These states must remain distinguishable across formats and warm/cold execution.

### A Code Overview Is a View over the Same Metrics

Add a compact `code` view and make it the default view for `--analyze=code` when no view
was explicitly supplied.
Keep `languages` as the narrower language table.
Selecting a view never enables analysis; requesting the code view without the code
analyzer produces a direct error explaining the missing requirement.

The code overview contains:

1. The selected population, code-line total, analyzed source-file count, and language
   count, followed by comment and blank-line totals.
2. A language table ordered by code lines descending, with a stable name tie-break.
3. Separate ignored/non-ignored contributions and a concise coverage statement when any
   source files were unsupported, unreadable, or changed during analysis.

For example, using illustrative numbers for `--ignored=include`:

```text
Code overview: all files; tests included
110,400 code lines | 2,030 analyzed source files | 4 analyzed languages
Non-ignored: 18,400 code lines / 210 source files
Ignored:     92,000 code lines / 1,820 source files

Language       Non-ignored    Ignored        All     Share
JavaScript           2,000     74,000     76,000     68.8%
Python               4,000     12,500     16,500     14.9%
Rust                12,000      4,500     16,500     14.9%
Shell                  400      1,000      1,400      1.3%

Share: measured code lines in both populations; rounded
Comments: 12,100 lines | Blank: 17,600 lines
Coverage: 2,030 source files counted; 3 unsupported; 1 read error
```

The file and language counts on the first line count analyzed source; missing source
coverage is separate.
Documentation, configuration, and binary inventories remain accessible through their
existing views and are not relabelled as source code.
Tests and examples remain included; maintained code is not limited to production files.

With `include`, the language table exposes both contributions directly; a combined total
alone can hide the working codebase inside a large dependency tree.
All = non-ignored + ignored for each fully measured metric.
With `exclude` or `only`, collapse the table to the requested population.
When an application combines reports with different populations, label each report
explicitly. Do not show an all-files code total when only one population was analyzed.
Generated and vendored flags can overlap, so their totals must not be presented as
another disjoint partition.
With pruning, print “ignored contents not scanned”, with no invented file or byte total.
Boundary directories may be counted only if labelled as boundaries, never as an estimate
of their unseen descendants.

All formats expose the same population, measurement denominator, and unavailable states.
A scope exclusion is an intentional omission; a read failure is incomplete measurement
within the selected scope.
Neither is a zero count.
Report languages completely at first, since this list is normally short.
If a user requests a bound, show an exact remainder and the way to lift it.

### Reuse Metrics for Useful Drill-Downs

Extend sorting to accept registered numeric metric IDs, starting with `code_lines`. This
gives file and directory reports the same capability without another special “largest
code files” command or analyzer:

```shell
fdu . --analyze=code --view=files --sort=code_lines --limit=10
```

Require the owning analyzer, order descending by default, use path order for ties, and
put unavailable values after measured values rather than treating them as zero.
Directory metrics are sums over the same selected descendants.
Keep the default overview short; these drill-downs are explicit additional views.
Defer medians, percentiles, and top-file summaries embedded in every default report
until a concrete use case justifies their extra aggregation and output.

Expose existing generated/vendor/documentation flags with their heuristic provenance
before adding automatic exclusions.
Evaluate `.gitattributes` overrides and ambiguous language detection as a subsequent
classification increment with independent fixtures.
Do not claim exact Linguist parity from a handful of matching language names.

### Keep the Engine Model Small and Explicit

The engine owns the requested population, analyzer set, and report presentation.
Every requested metric in one report uses that report’s population.
Resolve these once and derive an execution plan that obtains the required facts.
Reuse the population predicates across discovery, analysis candidates, and reports; do
not copy CLI checks into workers, reducers, or Python wrappers.
Display limits, pages, and tree display depth do not narrow measurement scope.

Keep logical requests distinct from retained-state coverage.
A retained index can contain more metadata than one query needs; a one-shot request can
prune entries that cannot contribute to its answer.
Use exact scope identity initially, with broader-cache reuse allowed only through a
projection proven equal to a cold answer.
An all-files cache must not expose ignored totals in an excluded-population request.
A pruned index cannot answer an all-files request by relabelling itself complete.

Metadata and content coverage remain distinct, and content identity includes both the
analyzer set and requested population.
An application can compose ordinary requests without requiring a special cache format
for every combination; broader reuse remains an optimization, not a prerequisite for
correctness.

Include relevant ignore-rule state, analyzer versions, and existing entry/type
identities in admission and invalidation decisions.
Rule changes must trigger appropriate discovery and invalidate affected metrics,
including discovering a previously pruned subtree that becomes eligible.
Required body reads happen outside index mutation and their results commit
conditionally. The [engine architecture](../architecture/fdu-engine-architecture.md) and
[surface architecture](../architecture/fdu-surface-architecture.md) own these
boundaries. Do not make this increment depend on adding content analysis to a serving
lifecycle that does not support it yet; any later lifecycle support must use the same
model.

### Fix Accuracy Before Adding a Complexity Number

Fix the demonstrated Rust states, common multiline literals, and JavaScript regex
handling against hand-classified fixtures.
Update analyzer version/fingerprint inputs whenever results change.
Keep differential tests as evidence, including known comparator defects, rather than
adopting another tool’s output as the expected answer.

Largest files and directory code concentration are the first structural aids.
A later optional `branch_points` metric can count explicitly listed per-language tokens
outside strings/comments, in the same streaming pass.
It needs its own analyzer contract, coverage, and measured overhead, and must not
silently enable under `--analyze=code`. Its analyzer option spelling should be settled
only when the prototype is retained.
Defer AST complexity, duplication analysis, Git history, and composite quality scores.

### Acceptance Criteria

- Equivalent explicit requests return the same content across Rust, Python, the CLI,
  formats, worker counts, and cold/warm paths.
  Test default resolution separately.
- Exercise the six direct population/measurement combinations and the two composed cases
  above. Where classification and measurement are complete, include totals equal the sum
  of exclude and only totals for the same analyzer set.
  Test unknown classification, unsupported files, read failures, no recognized
  languages, and zero code lines.
- A fresh excluded body is never opened for content analysis; control-file reads are
  separately accounted for.
  Prove this with counters, not timing alone.
- `--ignored=exclude` prunes safely excluded subtrees and reports their totals
  unavailable. Test ancestor rules, negation, nested repositories, refused controls, and
  rules changing between runs.
  `only` must still find ignored descendants beneath non-ignored ancestors.
- Composed requests retain their own populations and denominators.
  A preceding all-files metadata report must not make an excluded-population code report
  read ignored bodies or expose ignored totals.
  Presentation-only changes preserve measured totals; explicit retained preparation
  supports later narrow queries.
- Exercise all-to-selected and selected-to-all cache transitions, including edits to
  `.gitignore`; unavailable data cannot become available just because a warm cache
  happens to contain it.
- Minimal lexer failures pass across chunk splits, CRLF, and missing final newlines.
  Preserve the existing surface corpus and read every changed golden.
- Measure fresh-read volume, allocations, peak memory, and cold/warm time on mixed
  repositories under the [performance protocol](../guides/performance-loop.md).
  The existing single-line buffer and large generated files belong in that corpus.

## Next Steps

| Order | Work | Completion Evidence |
| --- | --- | --- |
| 1 | Fix Rust states and the common multiline forms; isolate the JS regex fix | Hand-established fixture results, chunk-boundary checks, analyzer invalidation, and surface parity |
| 2 | Unify population selection under `--ignored`; derive traversal and content work, including pruning for `exclude` | Parser/API tests; no excluded-body reads or safely prunable descendant enumeration; correct cache transitions |
| 3 | Add the code overview and metric-based file/directory sorting | Scope-labelled goldens, complete denominator accounting, structured-output parity |
| 4 | Improve language/classification coverage and evaluate a reviewed counter library if native repairs expand | Matched syntax and classification corpora, documented overrides, dependency and cost evidence |
| 5 | Prototype optional branching estimates only after the overview is useful and accurate | Defined per-language rules, measured overhead, explicit optional analyzer |

Research is tracked in `fdu-rq93`; the empirical investigation is `fdu-4il8`. Confirmed
parser follow-ups are `fdu-ov8o` (Rust), `fdu-lr38` (JavaScript regex), and `fdu-f1m3`
(remaining multiline forms).
The overview increment is tracked in `fdu-zdbq`, and unified ignored-population work in
`fdu-gdg0`; its traversal-pruning acceptance work is tracked in `fdu-a0kr`. These are
implementation boundaries within one public option, not separate user modes.
Break these increments into implementation tasks before changing the engine.
The alpha compatibility decision permits replacing interfaces; this research change
itself does not implement those replacements.

## Related Operational Decision: Cache Location

Current fdu cache defaults are `~/.cache/fdu` on Linux, `~/Library/Caches/fdu` on macOS,
and `%LOCALAPPDATA%\\fdu` on Windows.
`XDG_CACHE_HOME` overrides the base directory on all platforms.
There is currently no app-specific `FDU_CACHE_DIR` override.
See [cache path resolution](../../../crates/fdu-core/src/lib.rs).

uv uses XDG-style `~/.cache/uv` on both Linux and macOS, and `%LOCALAPPDATA%\\uv\\cache`
on Windows, with an explicit cache-directory override.
Rust tools have mixed conventions: sccache uses native cache directories, while Cargo’s
`~/.cargo` is a broader tool home containing configuration and installed binaries as
well as downloads. [uv storage](https://docs.astral.sh/uv/reference/storage/),
[uv cache overrides](https://docs.astral.sh/uv/concepts/cache/#cache-directory),
[sccache directories](https://github.com/mozilla/sccache/blob/main/docs/Local.md),
[Cargo home](https://doc.rust-lang.org/cargo/guide/cargo-home.html).

If simplifying fdu’s paths, prefer `~/.cache/fdu` across Unix with an app-specific
override that can select `~/.fdu/cache`. A general `~/.fdu` home becomes more compelling
if durable configuration or data is introduced.
Cache relocation must explicitly handle old files; rebuilding disposable snapshots is
sufficient, but silently leaving the old cache consumes disk indefinitely.
Automatic retention/size limits remain a separate open task, `fdu-558j`. No cache move
is part of this research change.

## Methodology and Limits

The delegated study compared isolated copies of the same source files, with a
hand-classified syntax corpus and a repository corpus.
The main agent independently validated the minimal Rust multiline-string and lifetime
examples with `rustc`, and the JavaScript regex example with `node --check`; they are
valid source. The fixtures intentionally target weaknesses and cannot estimate
population-wide accuracy.
Tool versions and source revisions are recorded with the evidence.

The ignored-file work probe was a separate cold invocation pair; it establishes that
selection did not prevent body reads in the released CLI. Metabrowser conclusions come
from its inspected source and test structure.
GitHub, crates.io, uv, and SCC comparisons use their primary documentation/source as
accessed on 2026-09-26. Proposed costs and output shapes are engineering judgments.
No current Tokei 15 benchmark, cross-project accuracy percentage, or universal
complexity comparison is claimed.
The released fdu binary corresponds to tag commit
`7cf7f1b4b39930ebcf54df7ebd2645d995e4e458`; the source corpus revision is listed above.

To reproduce a fixture, write its `source` field from the evidence JSON into an
otherwise empty directory, using the fixture key as its filename, then run:

```shell
fdu CASE_DIRECTORY --analyze=code --format=json --cache=off --limit=all
tokei --no-ignore --hidden --output=json CASE_FILE
```

Read fdu’s `reports[0].metrics.total.metrics` fields `code_lines`, `comment_lines`, and
`code_blank_lines`; compare Tokei’s `Total.code`, `Total.comments`, and `Total.blanks`.
The evidence also retains primary-file Tokei values because `Total` can include embedded
language blobs. To reproduce the real corpus, check out the recorded source revision,
copy each evidence `real` path individually to an otherwise empty directory, verify its
stored hash, and use the same commands.
No new dependency is needed when those tool versions are already installed.

## References

- [Fast file-type and content metrics](research-2026-08-12-fast-file-content-metrics.md)
- [Interactive browser use case](research-2026-08-11-interactive-browser-use-case.md)
- [fdu usage: content analysis](../../usage.md#analyze-file-contents)
- [fdu design principles](../architecture/fdu-design-principles.md)
- Primary source links beside each external finding above

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
