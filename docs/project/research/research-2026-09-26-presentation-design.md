# Research: Consistent Presentation and Composable Display Limits

**Date:** 2026-09-26

**Author:** Codex

**Status:** Complete research; implementation proposed

## Overview

fdu should show the important contents of a directory without making the reader decode
different conventions in each view.
Primary measurements should be easy to distinguish from supplementary detail, and
display limits should hide detail without changing totals.

The proposed default tree shows entries contributing at least 1% of the selected root’s
size, descending as far as depth 5. An optional breadth cap and total row limit compose
with those controls.
Cyan names, normal primary counts, and gray parenthetical annotations apply consistently
across human reports.

The performance footer should distinguish ignore files from parsed rules and report both
elapsed time and overall throughput, with precise definitions of work and time.

This is a research and design proposal, not implemented behavior.
The companion [codebase-analysis research](research-2026-09-26-codebase-analysis.md)
owns population selection and content metrics.
This document owns their presentation and display bounds.
The owner explicitly permits replacing alpha interfaces without compatibility aliases.

## Questions to Answer

1. Where do today’s views, colors, counts, and limit semantics disagree?
2. What visual roles and row grammar should every human report share?
3. How can a tree reveal significant deep contents while keeping its output manageable?
4. Which limits are independent, and how should they compose across views?
5. What precisely do ignore-rule counts and performance rates measure?

## Scope and Method

The audit uses the owner’s pasted examples, source at
`afe7eff1fcc0315a797e1a00efbb6a9b431dbcac`, and installed fdu 0.1.0. A controlled
eight-file fixture exercises 15 view/format combinations with color forced and disabled.
The [probe evidence](evidence/presentation-design-2026-09-26.json) records inputs and
outputs. A second controlled fixture checks seven existing depth, breadth, group, and
flat-list requests. The source review also covers watch framing, help, and diagnostics.

The pasted transcript contains no ANSI styling, so color findings come from the probe
and source. This verifies emitted spans, not contrast on every terminal theme.
Separate invocations have different timestamps; their structured-output timestamp
differences are not evidence of color changing data.
No speed benchmark is claimed.

## Current Behavior

### Presentation Inventory

The [report renderer](../../../crates/fdu-core/src/report_format.rs) defines cyan
directory names, green category labels and bars, bold cyan headings, and gray telemetry.
The [CLI](../../../crates/fdu/src/cli.rs) separately defines corresponding heading and
telemetry colors plus warning/error styles.
Comments describe shared heading ownership, but the constants are currently duplicated.

| Surface | Observed Behavior | Gap or Intentional Exception |
| --- | --- | --- |
| Tree | Cyan directory names, parenthesized file totals, plain ignored suffix | Primary totals look secondary; ignored annotations lack gray |
| Summary | Counts outside parentheses, plain ignored suffix | Summary helper has no color argument |
| Extensions | Green labels, primary file totals, plain ignored suffix | Annotation styling differs from telemetry |
| Types, families, languages, documents | Green labels; line/code/page parentheses in a plain suffix string | Parenthetical details are not styled as secondary |
| Largest, recent | Ranking measurement and unstyled path | Human paths lack the tree’s cyan convention |
| Bare files, `paths` | Path-oriented rows | Preserve clean path output rather than adding decorations |
| `long` | Plain size, age, and escaped path columns | Preserve its explicit flat-output contract |
| JSON, JSONL, YAML | Typed values, no ANSI even with forced color | Correct separation of data and terminal presentation |
| Help, notes, watch separators, footer | Existing semantic colors | Reuse the role catalog without altering lifecycle/streams |

All 12 human probe cases preserve their report body after ANSI removal, excluding the
changing performance footer.
Retain that invariant.
A whole output containing ANSI does not establish that its rows are styled: the footer
can be its only colored portion.

The source directly explains the examples: `render_text_tree` writes `name (N files)`;
`ignored_suffix` has no styling input; `render_text_metrics` appends plain strings;
summary and ranked-file helpers do not receive a color decision.

Counts are also inconsistent.
`human_count` already groups thousands and the footer uses it, but row renderers
interpolate many integers directly.
Thus `21,527` and `21527` appear in the same report.
Width helpers correctly pad outside ANSI spans, although `.chars().count()` measures
Unicode scalar values rather than terminal display columns.

### Current Limits and Defaults

[View defaults and tree expansion](../../../crates/fdu-core/src/query/query_report.rs)
and [request resolution](../../../crates/fdu-core/src/query/query_request.rs) establish:

| Current Output | Default Depth | Default Limit | Limit Meaning |
| --- | --- | --- | --- |
| Default list in tree format; tree view | 2 | 10 | Children per directory |
| Flat list; files | Unbounded | All | Rows in the result |
| Largest; recent | Unbounded | 20 | Rows in the result |
| Types, extensions, families, languages, documents | Not hierarchical | 10 | Groups in the section |
| Summary | Not hierarchical | One aggregate | No expandable row collection |

Tree rows currently represent directories only; direct files contribute to rollups but
never become leaves.
The root is depth 0. `--depth=0` shows the root only.
There is no minimum-share display setting.
`--limit` changes meaning between tree and flat output, and grouped views can hide
categories by default.

Tree expansion sorts children and truncates their list at each directory.
A truncated node is marked in the data, but human rendering prints an ellipsis only when
it also has retained children.
Depth boundaries therefore do not always expose hidden detail.
Flat/grouped bounds have a clearer `(10 of 18; --limit all ...)` note.

`--scan-depth` limits discovery and retained scope.
`--depth` limits the displayed tree.
The [design principles](../architecture/fdu-design-principles.md) explicitly distinguish
these, require shared defaults, and prohibit silently truncated answers.

### Existing Telemetry

[PerformanceSummary](../../../crates/fdu-core/src/execution.rs) carries walked regular
files and their apparent/allocated bytes, fresh analysis candidates, actual body bytes
read, analysis time, and cached content records.
Existing analysis rates use analysis elapsed time; overall file/byte rates are absent.

[ControlObservation](../../../crates/fdu-core/src/control.rs) records applied and
refused control-file counts, but no parsed-rule count.
The [ignore parser](../../../crates/fdu-core/src/control/gitignore.rs) already retains
the accepted pattern vector.
Applied files describe report coverage, including retained state; they are not
necessarily files read afresh this invocation.

The footer timer starts after request validation, before report preparation.
It is sampled after report output is flushed and pending snapshot saving has joined,
before footer emission.
This is total report elapsed time, including analysis, rendering and save waiting, but
excluding process startup, validation, and footer emission.

## Proposed Presentation Contract

### Semantic Roles and Parentheses

| Role | Color When Enabled | Placement |
| --- | --- | --- |
| Primary measurement | Normal foreground, ordinarily white on a dark terminal | Outside parentheses |
| File/directory name | Cyan | Name/path span only |
| Category label | Green | Existing type, extension, family, and language convention |
| Supplementary detail | Gray, existing bright-black role | Entire parenthetical span, including delimiters |
| Heading | Bold cyan | Existing section-heading convention |
| Telemetry/notes | Gray | Footer, denominator notes, bounds, watch separators |
| Warning/error | Existing yellow/red roles | Explicit diagnostic text, not buried in gray detail |

Use the normal readable foreground for “white”, rather than hard-coded white RGB that
can disappear on a light theme.
Use one gray role rather than mixing faint intensity, bright black, and arbitrary RGB
values.

The requested structural change is:

```text
attic 3508 files (43 MiB ignored)
```

The name is cyan, `3508 files` is normal foreground, and the entire ignored annotation
is gray.
With shared count formatting, this becomes `attic 3,508 files (43 MiB ignored)`.
Do not gray the primary total or name merely because the directory is ignored.

Apply the same secondary role to measurement breakdowns:

```text
477,298 lines (439,949 nonblank, 37,349 blank)
```

The total is primary and the breakdown is gray.
Code/comment/blank breakdowns and page estimates follow the same rule.
When code lines are the primary metric, put that total outside parentheses as in the
companion code-overview proposal.

Inline ignored data always uses parentheses and gray.
A dedicated ignored-population row or column is primary data with an explicit label, so
its values remain normal.
Literal parentheses inside a filename remain part of the cyan name.
Build semantic spans; never recolor parentheses by searching the finished output string.

### Common Row and Numeric Conventions

Use this order where applicable:

```text
primary columns  label  primary totals (breakdown) (ignored contribution)
```

Use the existing grouped-count helper for every human file, line, rule, and coverage
count. Keep size units in B/KiB/MiB/GiB and state the apparent/allocated basis when it
matters. Machine output retains raw numeric values.

Use the same visible escaping of control characters in human names before applying
color; flat formats already have a dedicated path-escaping helper, while several text
renderers write display strings directly.
Preserve ordinary filename characters, and keep exact path identity in the structured
representation.

Keep the percentage denominator explicit.
In the sample language table, the existing “Percentage column: code lines” note
correctly distinguishes it from the adjacent byte column.
Positive shares below rounding precision should display `<0.1%` or `<1%` rather than an
apparent zero; an unavailable denominator is not a zero share.

Supplementary generated/vendor/documentation flags can share a gray parenthetical group,
but their overlapping counts must not look like a disjoint partition.
Coverage failures need an explicit completeness statement; do not demote an actionable
error into decorative detail or imply that unsupported files were counted as zero code.

Measure widths before ANSI is applied, reset styling at each span boundary, and verify
Unicode display widths with dedicated fixtures.
Removing ANSI must preserve wording, spacing, ordering, and bounds.
Color may reinforce meaning but cannot be its only carrier.

## Proposed Adaptive Tree and Limit Model

### Independent Controls

“Minimum share” names the threshold correctly: an entry below the threshold is hidden.
It is not a maximum percentage or a cumulative coverage target.

| Proposed Control | Meaning | Default Tree Value |
| --- | --- | --- |
| `--depth=N` or `all` | Maximum displayed entry depth; root is 0 | `5` |
| `--min-share=P%` | Minimum contribution to the selected root’s measure | `1%` |
| `--breadth=N` or `all` | Maximum displayed immediate children of each directory | `all` |
| `--limit=N` or `all` | Maximum data rows per report section | `all` |
| Existing `--scan-depth=N` | Maximum discovery/retention depth | Unbounded |

`--limit` has one meaning across outputs.
Move the tree’s per-directory cap to `--breadth`; no compatibility alias is needed.
Both are optional because they answer different requests.
Annotation lines and headers are not data rows.
The tree root is a data row, so `--limit=0` prints no tree rows and an explicit omission
note; `--depth=0` prints the root unless a separate row cap suppresses it.

Accept nonnegative integer maxima or `all`; accept finite decimal percentages from `0%`
through `100%`, with an explicit percent suffix.
Reject malformed, negative, nonfinite, or out-of-range values before scanning.

Depth and breadth apply only to a hierarchical view.
Reject explicit settings when no requested view can use them; for a mixed report, apply
them to its tree sections only.
Resolve defaults once in the engine and expose them in machine output and a concise
human scope note. Do not invent a separate “dynamic depth” mode: the share predicate
naturally decides which branches expand within the depth bound.

### Default Guarantees and the Breadth Conflict

The default asks: **Which contents contribute at least 1% of this root, down to depth
5?** Show every qualifying entry in that range, along with the root and necessary
ancestors.
A size-ordered tree descends into a directory while it has qualifying children
and the next depth is permitted.

A default breadth of 10 cannot promise every entry above 1%: eleven children of 2% each
are a counterexample.
Therefore default breadth and total row count are unbounded, with the share/depth pair
providing the bound.
An explicit breadth or row cap intentionally relaxes the completeness guarantee and must
say what it omitted.

For complete, nonnegative additive sizes and positive root size, at most 100 disjoint
entries at any one depth can each contribute at least 1%. Depths 1 through 5 therefore
admit at most 500 qualifying rows, plus the root.
Typical trees are much shorter.
This is a bound on data rows, not annotation lines, and assumes exact known sizes.
Changing the threshold to 0.5% doubles that worst-case bound; setting it to 0% removes
this bound and admits zero-size entries as well.

### Root-Relative Shares and Stable Totals

Let `R` be the full selected root measure before display bounds, and `m(e)` an entry’s
selected subtree measure.
Show `e` when `m(e) / R >= P / 100`. Equality qualifies.
Compare raw integer measures against the parsed decimal percentage using overflow-safe
rational arithmetic; never filter on rounded displayed percentages.

Use the selected apparent/allocated byte basis for a size tree.
Keep `R` fixed across levels.
A directory’s 100%-of-parent child can still be insignificant relative to the root,
which is why a parent-relative threshold does not meet this requirement.
Do not add percentages across ancestor and descendant rows: their totals overlap.

Display bounds never change the root, directory totals, content-analysis population,
cache identity for discovered facts, or another view’s measurements.
Obtain the required rollups first and prune the presentation over them.
A large directory cannot be proven small by skipping its unmeasured descendants.
`--scan-depth` and ignored-population exclusion remain distinct requests that can change
which facts exist.

For partial/unknown sizes, do not silently discard an entry based on an unproven share.
Mark coverage and preserve potentially qualifying branches under explicit depth/breadth/
row bounds. The 501-row guarantee does not apply to unknown totals.
A zero-size root has no percentage denominator: show its root and explain the undefined
share; `--min-share=0%` allows a bounded structural listing of zero-size contents.

### Expansion and Omission Algorithm

1. Resolve population and measurements using existing request semantics; obtain root and
   subtree totals and coverage.
2. For each visible directory below the depth bound, form eligible child rows and
   compute their shares against the same `R`.
3. Apply minimum share, sort with the requested ordering and stable path ties, then
   apply breadth. Recurse only into retained directory rows; file rows are leaves.
4. Render the resulting tree in deterministic depth-first order, enforcing any explicit
   section row cap. This cap is a display-prefix budget, not a claim to select globally
   largest descendants across levels.
5. Retain omission metadata by reason: share, breadth, depth, or total rows.
   Assign each omitted subtree at its first exclusion boundary so remainders are not
   double-counted.

For a complete additive size tree, a qualifying descendant’s ancestors must also
qualify. For other row filters that match a descendant but not its ancestor, preserve
required structural ancestors and label their role rather than flattening away its path.
Ordering changes neither shares nor totals; `--sort=name --breadth=10` explicitly means
the first ten eligible names, not the largest ten entries.

Show significant regular files as leaves as well as directory rollups.
Otherwise “show everything at least 1%” would miss a large archive directly under the
root. This is an intentional extension of today’s directory-only tree.
Directory rows carry file totals; file leaves carry their size/name without a redundant
`1 file` suffix. The existing `--kind=dir` row selection can retain a directory-only
presentation; its aggregate semantics must continue counting the selected descendants
represented by those rows.
Do not follow symlink targets solely to satisfy a display threshold.

Depth boundaries need explicit markers even when no children were printed.
For example, `(deeper entries hidden; --depth all)` is gray.
Breadth and row omissions should name the corresponding flag; share omissions should
state the threshold.
Show exact omitted row counts and disjoint remainder bytes when the model knows them,
otherwise state that the quantity is unavailable.
A single `truncated` boolean cannot express these distinctions; use typed omission
reasons in the engine and structured reports.

### Application to the Supplied Tree

At the final proposed 1% default, 554 MiB implies an approximate 5.54 MiB threshold:

- `crates`, `fdu-py`, `.git`, and significant children such as `objects` remain visible.
- Continue beneath `fdu-py` while a child contributes at least 1%, up to depth 5. Its
  deeper names and sizes are absent from the supplied output and cannot be inferred.
- `attic` remains; its 6.0 and 5.9 MiB children qualify, while its 5.1 MiB children do
  not.
- `explorations/benchmarks` and `docs/project` qualify; most tiny support directories do
  not.

These judgments use rounded pasted sizes and are illustrative; boundary decisions must
use exact bytes. At a configurable 0.5%, the threshold is approximately 2.77 MiB and
additional children, including the shown 3.6 MiB `fdu-core`, qualify.

Proposed examples:

```shell
fdu .                                      # depth 5, minimum 1%, no additional row caps
fdu . --min-share=0.5%                      # more detail, same maximum depth
fdu . --depth=8                             # follow significant branches deeper
fdu . --breadth=10 --limit=100              # explicit caps, with omission notices
fdu . --depth=all --min-share=0% --breadth=all --limit=all
```

### Consistency across Other Views

Keep `--depth` and `--breadth` structural, and `--limit` a section row budget
everywhere. Flat lists remain complete by default.
Group tables should also default to complete output, including all languages; an
arbitrary top ten hides useful small categories.
`largest` and `recent` may remain transparent presets with an explicit default of 20,
since they name a bounded ranking.
All defaults remain overridable.

A minimum-share filter can apply to other additive grouped or file views using the
view’s declared measure: bytes for disk listings, code lines for code-language shares,
and words for document shares.
Default it to 0% outside the size tree so small languages and categories remain visible.
Sorting by age or name does not redefine that measure.
Reject an explicit share request when a view has no meaningful additive denominator.
Show the measure in the header so `1%` never requires guessing what is being divided.

Multiple views keep their own populations, denominators, and omission metadata.
Bounds are part of the requested answer and must agree across Rust, Python, and all
formats; ANSI and human annotations are presentation.
Selecting JSON must not silently remove limits or change the chosen default hierarchy.
Record resolved bounds and typed omissions in the schema, with a version change when
that shape changes.

## Performance Footer Proposal

### Shape and Numerators

Keep the whole footer gray.
For the pasted metadata example:

```text
Performance: walked 21,527 files / 554 MiB; ignore rules 48 files / R rules; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total 103.5 ms (208k files/s, 5.61 GB/s allocated walked)
```

`R` is a placeholder: the pasted output does not contain the rule count.
Rates are approximate calculations from the rounded sample, not new measurements.
Calculate the real values from original integers and duration, never formatted strings.
Use `apparent walked` when that is the selected size basis.

| Value | Definition |
| --- | --- |
| Ignore files | Control-file locations encountered in the observed scope: applied plus refused |
| Ignore rules | Accepted parsed pattern entries in applied files, including negations and repeated/shadowed rules; excluding comments, blanks, and rejected lines |
| Overall files/sec | Existing scan report’s successfully observed regular-file count divided by total report seconds |
| Overall GB/sec | Represented walked bytes in the answer’s size basis divided by total report seconds and 1,000,000,000 |
| Content read rate | Actual fresh body-read bytes divided by analysis elapsed seconds |
| Fresh analysis rate | Fresh candidates processed divided by analysis elapsed seconds |

Overall GB/sec is represented-size throughput, not storage bandwidth.
Metadata-only scans do not read all those file bodies.
Preserve the separate content-read rate, for example
`content read 208 MiB (215 MiB/s analysis)`, and the fresh-candidate rate.
Use decimal GB consistently; a GiB divisor must not be labelled GB. Phase rates and
whole-report rates have different denominators and must not be added.

Use the same elapsed sample for both overall rates and the printed total.
Preserve the current total-report timing boundary and document it.
Zero/unavailable duration produces an unavailable rate, never infinity.
Positive duration and zero walked work produce zero walk rates; a cache-only answer must
not turn cached inventory into walked work.
Use sufficient significant digits to avoid rounding nonzero rates silently to zero.

The scan report removes replaced-wave counts; progress can count repeated work.
Preserve the scan-report basis rather than mixing retry-inclusive progress with report
counters. Display caps do not change the work numerator.

### Rule Counts and Coverage

Count rules once per governing file location, even when identical source text is
interned and parsed once.
Count negations and shadowed duplicates because they are still parsed rules, not just
rules that eventually match a path.
Expose a count from the existing matcher and aggregate in the engine; never
reopen/reparse controls in the CLI to fill out the footer.

Examples with illustrative counts:

```text
ignore rules 0 files / 0 rules
ignore rules disabled
ignore rules 50 files / 312 known rules (2 files refused; total rules unknown)
ignore rules 48 files / 312 rules (cached)
```

Observed zero, observation disabled, partial/refused coverage, and retained coverage are
different states. Counts are scoped to discovered controls; pruned/unvisited subtrees
cannot contribute an exact whole-repository total.
Warm runs may reuse unchanged controls, so label coverage as ignore files/rules rather
than files freshly read.

Keep counts in the shared observation model and maintain them on replacement, removal,
and snapshot reconstruction.
Deriving a count from retained parsed sources need not add a redundant snapshot field.
Rule coverage should agree across Rust, Python, and machine reports.
Transient execution rates stay outside the stable report data, following the existing
`PerformanceSummary` boundary.

## Implementation Approach and Verification

Use a small shared semantic role catalog plus helpers for counts, labels, secondary
parentheticals, and aligned spans.
Retain the current renderers.
Per-renderer patches would duplicate policy, while a general terminal UI framework adds
unnecessary scope. Put reusable presentation definitions where the CLI can consume them
without core importing CLI code.
Do not post-process whole strings to find annotations.

Keep watch framing and path-oriented `paths`/`long` contracts intact.
Structured formats contain typed fields and numeric values, never ANSI or strings such
as `3,508 files`. Honor existing color controls and test actual precedence: explicit
`--color=always` currently overrides `NO_COLOR`, while machine formats remain uncolored.

Implementation should be divided into:

1. Shared roles and row grammar, including primary file totals, cyan names, gray ignored
   and metric parentheses, and grouped integer counts.
2. Explicit engine display bounds and omission metadata; adaptive expansion and large
   file leaves; shared default resolution and schema changes.
3. Rule-count observation and footer rates with precise coverage and timing semantics.

Acceptance criteria:

- ANSI-span assertions on tree, summary, grouped, and ranked text; plain/colored visible
  equality alone cannot verify that the correct substring is gray.
  Include literal parentheses and Unicode filenames, count boundaries, and reset
  behavior.
- Complete and partial metric coverage; unknown and zero denominators; overlapping
  classification flags; multi-view headings, redirects, watch framing, and flat formats.
- A large chain through depth 5, a large child at depth 6 with a visible depth marker,
  eleven 2% siblings, exact 1% boundaries, large direct files, many small siblings,
  empty trees, and unknown subtree sizes.
- Root-relative threshold invariance across depth, sorting, explicit breadth/row caps,
  filters, and apparent/allocated size.
  A small branch’s large parent-relative share must not bypass the root threshold.
- Exact additive worst-case bound on a complete synthetic tree; correct omission reasons
  and disjoint remainders; complete output when all bounds are lifted.
- Unchanged root/ancestor totals, analysis work, and cache answers under display
  changes; identical resolved scope and omission fields across Rust, Python, and
  formats.
- Rule fixtures with comments, blanks, escaped markers, negations, duplicates, shared
  source text, refused files, and mutations, on cold/warm/cache-only routes.
- Fixed-number rate tests for decimal conversion, size basis, cache hits, and zero time;
  no wall-clock performance assertions in formatting tests.

## Next Steps

Research is tracked in `fdu-l5y5`. Presentation roles are tracked in `fdu-uj14`,
adaptive tree and limit semantics in `fdu-3y5z`, and rule counts/throughput in
`fdu-xlw4`. Update all affected goldens deliberately.
Run the normal repository handoff gate before shipping.
No backward-compatibility scaffolding is required; revise the request models, help,
documentation, schemas, and Python surface together.

## References

- [Report renderer](../../../crates/fdu-core/src/report_format.rs)
- [View defaults and tree expansion](../../../crates/fdu-core/src/query/query_report.rs)
- [Request resolver](../../../crates/fdu-core/src/query/query_request.rs)
- [CLI styles, limits, and footer](../../../crates/fdu/src/cli.rs)
- [Performance summary](../../../crates/fdu-core/src/execution.rs)
- [Control observation](../../../crates/fdu-core/src/control.rs)
- [Ignore matcher](../../../crates/fdu-core/src/control/gitignore.rs)
- [Scan counters](../../../crates/fdu-core/src/scan.rs)
- [Design principles](../architecture/fdu-design-principles.md)
- [Surface architecture](../architecture/fdu-surface-architecture.md)
- [Presentation evidence](evidence/presentation-design-2026-09-26.json)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
