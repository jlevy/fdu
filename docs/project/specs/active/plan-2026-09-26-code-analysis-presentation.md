# Feature: Codebase Analysis and Consistent Presentation

**Date:** 2026-09-26

**Author:** Codex

**Status:** Complete

**Tracking:** Epic `fdu-ccf7`; plan publication `fdu-r55w`;
[senior review follow-up](https://github.com/jlevy/fdu/pull/133#issuecomment-5853335891)
`fdu-khdv`.

## Overview

Make `fdu --analyze=code .` a concise, trustworthy overview of a local codebase, and
make every report use consistent measurements, visual roles, and display limits.
Correct the demonstrated counting errors before building new conclusions on those
counts. Derive traversal and content reads from the requested population and analyzers.

The metadata tree should answer “which contents account for at least 1% of this root,
down to depth 5?” Independent breadth and row limits let callers request less output.
The same engine models determine answers in Rust, Python, and the command line.

This plan implements the recommendations in the
[codebase-analysis research](../../research/research-2026-09-26-codebase-analysis.md)
and [presentation research](../../research/research-2026-09-26-presentation-design.md).
Those documents retain comparative evidence and alternatives; this plan selects the
contracts, implementation boundaries, and completion gates.
The owner has waived alpha backward-compatibility constraints and explicitly selected
`~/.cache/fdu` as the macOS cache default.

## Goals

- Correct demonstrated source-line classification errors with independent expectations.
- Make excluded ignored populations cheaper by avoiding unnecessary traversal and body
  reads, while preserving honest coverage and cold/warm equivalence.
- Show code totals, languages, population contributions, coverage, and useful file and
  directory rankings without enabling extra analyzers implicitly.
- Apply shared colors, number formatting, annotations, and omission semantics to every
  report where its format supports them.
- Provide composable depth, minimum-share, breadth, and section-row controls.
- Report ignore files and accepted rules, elapsed time, and clearly labelled throughput.
- Use consistent Unix cache defaults, explicit overrides, and descriptive cache
  filenames.

## Non-Goals

- Exact GitHub Linguist or crates.io count parity; their populations and definitions
  differ from a local codebase inventory.
- AST-based complexity, history/churn, duplication detection, effort/cost estimates, or
  a universal code-quality score.
- Automatic cache retention or a general `~/.fdu` application home.
  Retention remains tracked in `fdu-558j`.
- Adding live content analysis to serving lifecycles that currently reject it, automatic
  release publication, or new compatibility aliases for replaced alpha interfaces.

## Background and Governing Contracts

The research preserves 30 syntax fixtures, a matched 107-file comparison, 15
presentation probe cases, and seven current-limit probes.
Comparator agreement is evidence, not an oracle: some valid constructs fail in both fdu
and Tokei, and some fail only in Tokei.
Use the hand-established partitions and language syntax when resolving disagreements.

The [design principles](../../architecture/fdu-design-principles.md),
[engine architecture](../../architecture/fdu-engine-architecture.md), and
[surface architecture](../../architecture/fdu-surface-architecture.md) remain governing
contracts: one typed model owns each semantic decision, cached history cannot change an
answer, and every surface must agree.

This plan intentionally revises the current depth-two, ten-child tree default and its
directory-only rows.
Update the default descriptions in the design principles and surface architecture when
implementing that change.
The [directory-query plan](plan-2026-09-20-directory-query-formats.md) preserves the
filter and subtree-total contracts; its requirement to retain the earlier presentation
is superseded for this increment.
Historical implementation evidence stays intact.

## Design

### Population, Measurement, and Presentation

| Facet | Public Control | Contract |
| --- | --- | --- |
| Population | `--ignored=include\|exclude\|only`, existing entry predicates | Which entries contribute |
| Measurement | Existing `--analyze` | Which content facts are requested; metadata is available without analysis |
| Presentation | `--view`, format, sorting, display limits | Which projection of those facts is shown |
| Discovery scope | Existing scan depth and filesystem boundaries | Which facts may be discovered and retained |
| Delivery | Cache policy and workers | How the same requested answer is obtained |

Resolve defaults once in core.
Default ignored population is `include` for every analyzer set and format.
Replace the old ignored booleans together across the CLI, Rust, and Python; retain no
deprecated aliases.

- `include` discovers both known populations and presents their separate contributions.
- `exclude` prunes safely ignored subtrees and never analyzes ignored bodies.
- `only` discovers ignored matches through non-ignored ancestors and never analyzes
  non-ignored bodies. Non-ignored ancestors can still require enumeration/control reads.

Use the same effective ignore classification for discovery, candidate admission, and
reports. Necessary ignore-control reads are separate from content-analysis reads.
Unknown classification cannot justify pruning or count as known non-ignored material.
Under a partial-result policy, preserve unknown coverage separately; otherwise return an
explicit inability to produce the requested complete answer.

`--no-gitignore` is compatible only with `include`, labelled unclassified.
Reject `exclude` and `only` when classification is disabled before scanning.
A view or sort requiring content metrics must name the missing analyzer in its error; it
never enables that analyzer.

Each report has one population and one analyzer set.
Whole-tree metadata with non-ignored code metrics is two ordinary requests:

```shell
fdu . --ignored=include
fdu . --ignored=exclude --analyze=code
```

This preserves all six direct population/measurement combinations and both mixed
metadata/content combinations described in the research.
Do not add `--scan-ignored`, `--analyze-scope`, or a second interpretation of ignored
state.

### Counting Correctness and Identity

Repair the current streaming counter in bounded groups: Rust strings/lifetimes,
multiline literal and heredoc families, then JavaScript/TypeScript regex-versus-division
handling. Preserve the established definitions for mixed code/comment lines, blank lines
inside comments, multiline string contents, and Python docstrings unless an
independently reviewed metric contract changes them.

Turn every adjudicated research failure into a minimal fixture, including
counterexamples where the comparator is wrong.
Check delimiter state across read chunks, CRLF, escapes, and missing final newlines.
Name any unsupported construct explicitly rather than asserting full language parsing.

Change analyzer version/fingerprint inputs whenever results change.
Old derived records must be rejected and recomputed.
Preserve one streaming read pass for the enabled metrics and measure long-line and
generated-file memory behavior.

Population belongs to the shared logical request; pruning also shapes retained scope.
Use exact scope/identity matching initially.
Broader metadata or content reuse requires a separately proven projection equal to the
cold answer. Display bounds never enter identity for discovered facts or change
measurement work.

Ignore-rule edits can make previously pruned trees eligible.
Reconciliation must discover those trees and invalidate affected classification and
metrics. Keep filesystem reads outside index mutation and conditionally commit against
current entry/control identities.
Existing opened-root preparation may retain broader metadata for later queries, but
cannot claim unsupported live content capability or relabel missing scope as complete.

### Code Overview and Drill-Downs

Add the `code` view as a projection of existing code metrics.
With code analysis enabled and no explicit view, show this overview.
For a combined analyzer request, retain the other analyzer-driven sections needed to
show its requested measurements and avoid a second identical language table.
An explicit view always wins.

The code overview includes:

1. Selected population and code-line total, analyzed source-file count, and analyzed
   language count, with comment/blank totals.
2. All languages, ordered by code lines descending and stable language-name ties.
3. With `include`, combined totals and non-ignored/ignored contributions for the
   overview and each language.
   Human output keeps the combined total primary and the breakdown in supplementary
   parentheses; machine output carries separate fields.
   With `exclude` or `only`, show the selected population without a redundant breakdown.
4. An explicit share denominator and coverage for unsupported, unreadable, changed, or
   unclassified files.

Tests and examples remain included.
Documentation/configuration inventories retain existing views.
For fully measured, classified populations, combined values equal the sum of ignored and
non-ignored values. If classification is partial, expose its unknown contribution and do
not claim that two known columns exhaust the total.
Unrequested, not scanned, not analyzed, failed, and measured zero remain distinct.

Extend numeric sorting through the existing metric registry, starting with `code_lines`.
Require the owning analyzer, default numeric ranks to descending, use path ties, and
place unavailable values after measured ones.
The metadata-only `extensions` view rejects content-metric sorting and names supported
metadata sorts or a metric-capable view.
Directory metrics aggregate selected contents; overlapping displayed directories never
double-count a summary.

```shell
fdu . --analyze=code --view=files --sort=code_lines --limit=10
fdu . --analyze=code --view=list --kind=dir --sort=code_lines
```

Tree share eligibility uses the declared view measure, independent of sorting: a size
tree sorted by code lines still filters by size.
Language code shares use code lines; document word shares use words.
State the measure in the report.
A dedicated code-share tree can be evaluated later without overloading sort semantics.

Expose existing generated/vendor/documentation flags and heuristic provenance, using the
existing type-registry override capability where applicable.
Flags may overlap and are not an additional partition or automatic exclusion policy.
Preserve exact filenames and native path identity in machine data.

### Shared Presentation Roles

Use a small core-owned role catalog and span helpers consumed by all human renderers and
CLI framing. Retain existing renderer structure; avoid string post-processing that looks
for parentheses or ANSI sequences.

| Role | Human Presentation |
| --- | --- |
| Primary numbers and file totals | Normal readable foreground, outside parentheses |
| Names and paths | Cyan, scoped to the name span |
| Category labels | Existing green role |
| Supplementary details | Entire parenthetical span gray, including delimiters |
| Headings | Existing bold cyan role |
| Telemetry and omission notes | Gray |
| Warnings/errors | Explicit existing warning/error styles |

For example, `attic 3,508 files (43 MiB ignored)` has a cyan name, a normal file count,
and a gray ignored annotation.
Likewise, `477,298 lines (439,949 nonblank, 37,349 blank)` keeps its total primary and
its breakdown gray. Dedicated ignored columns/rows remain primary data; a filename
containing parentheses remains one name span.

Use shared integer grouping and B/KiB/MiB/GiB size conventions.
Keep machine numbers numeric.
Show small positive shares without rounding them to an apparent zero and use an
unavailable marker for undefined shares.
Escape control characters before styling; measure terminal display width before ANSI,
with reset boundaries that cannot leak.

Audit tree, summary, all grouped views, ranked rows, code overview, multiview headings,
watch framing, help, and diagnostics.
Preserve intentional `paths`/`long` contracts and machine formats without ANSI. Honor
existing color precedence, including forced color and `NO_COLOR`, and verify actual
spans rather than merely detecting any ANSI output.

### Composable Display Limits

| Control | Meaning | Tree Default | Other Views |
| --- | --- | --- | --- |
| `--depth=N\|all` | Maximum displayed entry depth; root is 0 | `5` | Hierarchy only |
| `--min-share=P%` | Minimum contribution to the full selected root measure | `1%` | `0%` for additive non-tree views |
| `--breadth=N\|all` | Maximum immediate child rows per directory | `all` | Hierarchy only |
| `--limit=N\|all` | Maximum data rows per report section | `all` | Complete lists/groups; 20 for largest/recent presets |
| `--scan-depth=N` | Discovery/retention bound | Existing unbounded default | Independent of display bounds |

Accept nonnegative integer maxima or `all`, and finite explicit percentages from `0%`
through `100%`. Validate before work.
Reject explicit hierarchy-only controls when no requested section can use them; for
mixed views, apply them to the hierarchical sections.
Reject explicit share filtering for views without an additive denominator.

The root counts toward the row limit; headers and omission annotations do not.
`--limit=0` produces no data rows and an omission note.
`--depth=0` produces the root unless the row limit suppresses it.
`0%` disables share pruning and admits zero-valued entries.
All bounds can be lifted independently.

Compute shares from exact values against one fixed selected root total before display
bounds, with overflow-safe decimal/rational comparison and inclusive equality.
At each directory, apply share eligibility, sort with deterministic ties, apply breadth,
and descend within depth.
Apply a section row cap to the resulting depth-first prefix.
A capped prefix is not a globally optimal selection of large descendants.

Include significant regular files as leaves and directories as rollups; files need no
redundant `1 file` annotation.
`--kind=dir` permits directory-only rows while preserving their eligible subtree
measurements. Preserve necessary structural ancestors for other row filters.
Display changes never alter aggregate totals or eligible content reads.
Do not follow new symlink targets to satisfy a display threshold.

Default breadth must be unlimited: eleven siblings of 2% each cannot all fit a cap of
ten.
Complete nonnegative additive sizes bound the 1%/depth-5 default to at most 501 data
rows including the root.
Unknown/partial measures do not satisfy that proof: preserve potentially significant
branches within explicit bounds and report incompleteness.
Apply that exception per subtree: a failed listing must not disable share pruning for
verified files or complete sibling subtrees.
Use the observed selected-root total as the lower-bound denominator and retain
incomplete branches whose unseen contents could cross the threshold.
A shared engine/command-line golden covers this interaction, alongside a portable
bounded-discovery tryscript case (fdu-k46n).

A zero root has no share denominator; show the root and explain it, with `0%` available
for a structural listing.

Store typed omission reasons for share, breadth, depth, and section rows.
Attribute each omitted subtree to its first exclusion boundary, with exact disjoint
remainder measures where known.
Do not add ancestor and descendant totals.
Mark a depth boundary even if no child rows were retained.
Every omission names the bound and how to lift it.

Resolve projection and limits before rendering.
JSON, JSONL, YAML, text, Rust, and Python must expose the same selected answer and
omissions; serialization cannot recover a discarded row or silently remove a default
bound. Preserve explicit flat projection choices.
Update the schema identifier when report shape or semantics changes.

### Ignore Accounting and Performance

Expose accepted rule counts through the existing control parser/observation model.
Count per governing file location, including negations, duplicates, and shadowed
patterns; exclude blanks, comments, and rejected lines.
Identical interned source text at two locations counts for each location.
Never reread rules from the CLI for telemetry.

Ignore-file totals include applied and refused locations within observed scope.
Show known rule count plus incomplete coverage when rules were refused, and distinguish
classification disabled, observed zero, retained coverage, and unvisited controls.
Maintain counts through rule replacement, removal, and snapshot reconstruction.

Add these rates to the existing gray footer using one total elapsed sample:

- Successfully observed regular files divided by total report seconds.
- Represented walked bytes in the selected size basis divided by total seconds and
  1,000,000,000, labelled `GB/s apparent walked` or `GB/s allocated walked`.

Represented-size throughput is not storage bandwidth.
Preserve actual content-read and fresh-analysis rates with their analysis-phase
denominator. Retain the existing total boundary: after validation, before preparation,
through rendering/flush and pending save join, before footer emission.
Use original integers and duration, not formatted values.
Zero/unavailable time produces an unavailable rate; zero work at positive time produces
zero. Cached inventory is not walked work.
Keep scan-report counts distinct from retry-inclusive progress.
Execution rates stay outside stable report content.

### Cache Location and File Names

The approved macOS default is `~/.cache/fdu`, matching Linux.
Resolve a single effective application cache directory for metadata snapshots, content
sidecars, status, and clear operations, in this order:

1. Explicit typed core destination (`--cache-dir=PATH` in the CLI and its Python
   equivalent).
2. `FDU_CACHE_DIR`, naming the exact application directory.
3. `XDG_CACHE_HOME/fdu` when that base is set.
4. `~/.cache/fdu` on Linux/macOS; the existing `%LOCALAPPDATA%/fdu` resolution and
   documented fallbacks on Windows.

Reject invalid or empty explicit overrides; resolve paths consistently before I/O.
Document that shell expansion supplies `$HOME`/`~` rather than inventing a separate
expansion grammar. Cache-off execution must not create a cache directory.

Use descriptive sibling names in the application cache directory:

```text
~/.cache/fdu/
  0123456789abcdef.metadata.bin
  0123456789abcdef.analysis.bin
```

The existing `.fdu` file is the filesystem snapshot: entries, metadata, classification,
and observed ignore-control state.
The `.fdu.content` file contains derived analyzer results, not copies of source files.
The new names expose those roles directly; `.bin` correctly identifies the binary
format. Keep the flat layout and existing root key of 16 lowercase hexadecimal digits,
derived from the canonical native root path.
Do not add a directory layer, root-basename sanitization, or a separate path-to-key
manifest merely for naming.
The key is a lookup aid, not a content hash or proof of identity; header identities and
validation still decide whether a file can answer a request.

`--cache-status` should show the root when its header is readable, the resolved
directory, metadata/analysis paths and bytes, analyzer coverage, and
current/stale/leftover status.
An unreadable header must not invent a root mapping.
Document that analysis files are written only when analysis is requested, all files are
disposable, and the authoritative facts remain in the scanned filesystem.

One core naming helper owns both paths and their pairing.
Update writers, loaders, status, clear, orphan detection, and staging recognition
together. Temporary files use `.<target-name>.tmp.<unique-suffix>` and remain beside
their destination for atomic rename.
Recognition still requires the expected file kind and magic, not just `.bin`. Preserve
symlink/unknown-file protection, revalidation before deletion, and protection for active
staging files. Keep the metadata/analysis identity and invalidation rules; renaming a
file is not permission to reuse incompatible data.

Document only the selected location, override precedence, new file roles, and current
status/clear commands.
The owner explicitly excludes old-location documentation, legacy-location discovery,
automatic migration, and legacy naming compatibility from this increment.
Existing caches may be rebuilt under the new names.
Automatic retention remains the separate task `fdu-558j`.

### Implementation Ownership

| Area | Existing Implementation Boundary | Required Change |
| --- | --- | --- |
| Request/defaults | `query/query_request.rs`, selection and view models | Population, code view, typed limits, validation and metric sort |
| Discovery/analysis | `scan.rs`, `index.rs`, `content/content_analysis.rs` | Shared eligibility, pruning, candidate selection and counters |
| Code metrics | `content/content_code_metrics.rs`, content identities | Lexical repairs and versioned results |
| Reports | `query/query_report.rs`, report/schema models | Population tallies, code projection, adaptive hierarchy and omissions |
| Human output | `report_format.rs`, CLI framing | Shared roles, number/width helpers and reviewed output |
| Controls/telemetry | `control.rs`, `control/gitignore.rs`, `execution.rs` | Rule coverage and defined rates |
| Cache destination | Core cache resolution and delivery settings | Shared destination and naming resolution; status/clear consistency |
| Public surfaces | CLI options, Python bindings/models, Rust API | Forward the same typed contracts and expose the same fields |

Paths in the table are relative to `crates/fdu-core/src` except CLI/Python boundaries.
Keep one model per concept; avoid parallel CLI-only representations, cache adapters for
obsolete alpha schemas, or a second authoritative inventory.

## Implementation Plan

### Phase 1: Correct Counts and Population Work

- [x] Repair Rust syntax cases (`fdu-ov8o`), multiline/heredoc forms (`fdu-f1m3`), and
  JavaScript regex handling (`fdu-lr38`) with independently expected fixtures.
- [x] Implement unified population/default/request semantics (`fdu-gdg0`) and traversal
  pruning (`fdu-a0kr`) as one complete public capability.
- [x] Version affected analyzer/content/snapshot identities and verify rule-change and
  all-to-selected/selected-to-all histories.
- [x] Prove excluded bodies are not analyzed and safely excluded descendants are not
  enumerated, separating necessary ancestor/control work.

**Exit:** Correct fixture partitions, honest unavailable states, and equivalent
cold/warm answers across the public surfaces.
Do not ship `exclude` as a report-only filter.

### Phase 2: Reports, Limits, Telemetry, and Cache Destination

- [x] Implement code overview (`fdu-zdbq`) and metric sorting/classification
  explanations (`fdu-n4hp`).
- [x] Apply shared presentation roles (`fdu-uj14`) across every relevant renderer.
- [x] Implement adaptive tree defaults, significant file leaves, uniform section limits,
  and typed omissions (`fdu-3y5z`).
- [x] Add rule counts and precisely defined total rates (`fdu-xlw4`).
- [x] Implement the shared Unix cache default, explicit destination overrides, and
  descriptive cache filenames (`fdu-smhw`).
- [x] Update Rust/Python models, schemas, help, embedded docs, usage, machine-output
  reference, skill examples, and architecture descriptions with the owning change.

**Exit:** Every requested output obeys the same population/measurement/limit contract;
all new syntax works on every supported surface.
The cache destination and file roles are visible and consistent, including status and
cleanup.

### Phase 3: Integrated Evidence and Bounded Follow-Ups

- [x] Run the acceptance matrix and release-quality validation (`fdu-7jtp`), including
  package installation and cross-platform cache tests.
- [x] Record paired cold/warm costs and work counters on mixed repositories, long-line
  files, generated sources, and trees dominated by ignored content.
- [x] Evaluate classification gaps, selected `.gitattributes` overrides, and a reviewed
  counter library only against an independent compatibility corpus (`fdu-3ou8`).
- [x] After lexical correctness, prototype optional per-language branch-point estimates
  in the same read pass.
  Record token rules, coverage, counterexamples, and incremental CPU/memory cost; retain
  only with useful ranking evidence and acceptable measured cost.
- [x] Record explicit retain/defer verdicts for both evaluations.
  A retained capability requires a spec addendum, public analyzer/classification
  contract, dependency review if needed, and the same parity/golden gates before
  shipping.

**Exit:** Required capabilities have passing evidence and updated documentation.
Conditional investigations have recorded outcomes; an experiment alone cannot be
reported as shipped functionality.
AST metrics and exact Linguist compatibility remain outside this increment.

### Phase 4: Testing Architecture Review and Consolidation

Review the complete testing architecture and the implementation delta using
`tbd guidelines golden-testing-guidelines general-testing-rules` and the installed
tryscript documentation.
Optimize meaningful regression coverage per maintained line of test code, with readable
scenarios and flexible fixtures.
Test counts and raw coverage percentages are diagnostics, not completion targets.

- [x] Audit test ownership, coverage, duplication, portability, determinism, and runtime
  (`fdu-tbtm`). Map the acceptance contracts to the tests that prove them, identify
  untested failure paths, and record keep/consolidate/replace decisions with reasons.
- [x] Implement warranted improvements (`fdu-sc1w`). Prefer concise language-neutral
  tryscript sessions for public behavior when they preserve equivalent evidence.
  Reuse authoritative fixtures and remove redundant setup or assertions.
  Keep focused tests for lexer chunk boundaries, exact arithmetic, allocations,
  concurrency, and failure injection where a golden would lose coverage or precision.
- [x] Verify the revised suite (`fdu-cdp6`). Review complete golden diffs and preserve
  narrow stable-field patterns, exit codes, stderr, side effects, and schema coverage.
  Demonstrate that representative critical guards reject deliberately broken behavior.
  Record before/after test-code and fixture size and observed runtime, with the tested
  platform and selection; explain necessary growth and any remaining gaps.

Do not blindly regenerate goldens, broaden elisions to hide differences, duplicate the
CLI corpus for Python, or weaken an independent oracle to make a test shorter.
Preserve Linux authority for parity recordings, cache-serving proofs, bounded native
watch checks, and the portability, observability, and invocation guards.
Use the existing tooling; add a harness only when a concrete missing contract requires
it.

**Exit:** A written review identifies the evidence each test layer owns; actionable
findings are addressed; the shared corpus and required handoff checks pass.
The suite has no known loss of coverage from consolidation, and its maintenance cost and
remaining platform limits are explicit.

### Implementation Evidence

- [Implementation review](../../reviews/review-2026-09-26-code-analysis-implementation.md)
  records fixed findings and remaining handoff checks.
- [Testing architecture review](../../reviews/review-2026-09-26-code-analysis-testing.md)
  maps behavior to test layers and records coverage and maintenance decisions.
- [Paired cost report](../../reports/report-2026-09-26-code-analysis-paired-costs.md)
  links the fixed subjects, raw samples, and work counters.
- [Correctness evidence](../../research/evidence/code-analysis-correctness-2026-09-26.json)
  records cold/warm/cache-only answers and deliberate cache-failure proofs.
- [Conditional evaluation](../../research/evidence/codebase-analysis-conditional-2026-09-26.json)
  records the adjudicated counting corpus and explicit deferrals.

### Tracking and Dependencies

All implementation beads link to this spec under epic `fdu-ccf7`. `fdu-a0kr` depends on
the unified population contract in `fdu-gdg0`; the overview depends on both population
tasks and the lexical fixes.
Metric drill-downs depend on corrected metrics and population semantics.
Adaptive limits and shared styles can be developed independently against the agreed
report model.
Telemetry depends on unified population semantics so its scope is accurate.
Cache destination is independently implementable.

Integrated validation waits for all required Phase 1/2 tasks.
Conditional evaluation waits for that useful, verified baseline.
Keep implementation and conditional evaluation beads open until their own acceptance
evidence exists; publishing this plan closes only `fdu-r55w`. Reuse the existing
research follow-ups rather than duplicating them.
The testing-review audit (`fdu-tbtm`) leads to improvements (`fdu-sc1w`), then final
sensitivity and portability evidence (`fdu-cdp6`), which also depends on integrated
validation (`fdu-7jtp`). The audit may run alongside implementation; final verification
uses the completed implementation.

## Testing Strategy

| Contract | Required Evidence |
| --- | --- |
| Line counting | All adjudicated research failures, comparator counterexamples, chunk splits, CRLF, no final newline, raw/ordinary strings, heredocs, regex versus division |
| Population work | Six direct requests and two composed cases; no excluded body opens; safe subtree pruning; `only` through non-ignored ancestors; negations, nested controls/repos and refused rules |
| Cache correctness | Cold/warm/cache-only routes, worker changes, analyzer/population transitions, rule edits exposing pruned descendants; explicit stale/partial outcomes |
| Overview/ranking | Empty/zero/unsupported/unreadable sources; known and unknown populations; complete language table, correct denominator and metric sort ties; unchanged totals under display bounds |
| Tree limits | Exact 1%, eleven 2% siblings, root-relative versus parent-relative shares, depths 0/5/6/all, large direct file, zero/unknown root, breadth/row zero and all, mixed views, omission remainders |
| Styling/formats | Targeted ANSI spans, stripped-output equality, literal parentheses, escaped controls, Unicode widths, light/dark normal foreground, NO_COLOR/forced color, no ANSI in machine output |
| Rule/rate accounting | Duplicate/negated/escaped/shadowed rules, shared parsed sources, mutations/refusals, cached coverage, known fixed-duration arithmetic, zero work/time and both size bases |
| Cache destination | Explicit/env/XDG/default precedence, macOS/Linux/Windows defaults, unset/invalid roots, cache-off, metadata/analysis pairing, staging and orphan recognition, symlink/unknown-file preservation, failure reporting |
| Surface parity | One shared corpus over Rust/CLI/Python and all formats; installed wheel exercises the new options and schemas |

Use counters and controlled filesystem seams for work-avoidance assertions; elapsed time
alone is insufficient.
Keep golden fixtures portable and review each changed expectation.
Record Linux-owned parity artifacts in CI. Exercise metadata watch framing and ensure
unsupported live content remains a direct error.

Run `make docs-format` and `make check` for handoff, and `make cross-lint` for the cache
platform changes or other platform-gated code.
Give each worktree a dedicated Cargo target and follow local external-scratch
instructions; never put unique research evidence in disposable build directories.
Run the [correctness runbook](../../guides/correctness-runbook.md) after
identity/reconciliation changes and before a separately authorized release.

Measure performance using the [existing protocol](../../guides/performance-loop.md).
State host, platform, cache regime, file population, work counts, allocation/peak-memory
results, and paired timing uncertainty.
Record trade-offs and rejected experiments.
Do not invent a speedup threshold from a single run or hide a metadata-only regression
behind improved ignored-heavy analysis.

## Rollout Plan

Deliver focused implementation changes in dependency order, with tests and public-model
updates in each change.
Keep `include` available throughout, but do not expose the new exclusion promise until
both pruning and body selection pass.
Update all surfaces atomically when replacing flags or schema fields; rebuild
incompatible cache data.

Publish user-facing examples for the code overview, cheaper excluded analysis, optional
ignored-only inspection, lifted/composed display bounds, and cache location overrides.
Explain the new default tree and the cache location and file roles in release notes when
shipping. Update the usage/skill install smoke checks to exercise the published wheel
without a compiler. Tagging or publishing the next release is a separate operation from
accepting or implementing this plan.

## Open Questions

No unresolved choice blocks Phase 1 or Phase 2. The following are explicit investigation
outcomes, not missing defaults:

- Which additional classification constructs and `.gitattributes` overrides justify
  support after the independent corpus is assembled?
- Do remaining native lexer repairs justify adopting a reviewed counter library once
  semantic differences and runtime costs are measured?
- Does a per-language branch-point estimate improve navigation enough to retain as a
  separate opt-in analyzer, and what measured cost is acceptable?

Record those decisions in `fdu-3ou8` and amend this spec before adding public behavior.
The name of an optional complexity analyzer remains unset until the prototype is
retained.

## References

- [Codebase-analysis research and evidence](../../research/research-2026-09-26-codebase-analysis.md)
- [Presentation research and evidence](../../research/research-2026-09-26-presentation-design.md)
- [Content-metrics research](../../research/research-2026-08-12-fast-file-content-metrics.md)
- [Design principles](../../architecture/fdu-design-principles.md)
- [Engine architecture](../../architecture/fdu-engine-architecture.md)
- [Surface architecture](../../architecture/fdu-surface-architecture.md)
- [Machine-output reference](../../../machine-output.md)
- [Cache design](../../guides/cache-design.md)
- [Correctness runbook](../../guides/correctness-runbook.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
