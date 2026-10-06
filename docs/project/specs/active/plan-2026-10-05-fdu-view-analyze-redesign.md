# Plan: Views Imply the Analysis They Need

**Date:** 2026-10-05

**Author:** fdu project, with Claude Fable 5.1

**Status:** Draft, for the maintainer’s review.
No compatibility constraint applies: flags, Python arguments, view names, analyzer
names, and structured output may all change.

## Overview

fdu reads file bodies only when asked, through `--analyze`, and chooses what to print
through `--view`. The two axes are right, and the rule that keeps them apart — a display
choice must never quietly turn into a read of every file in the tree — is right too.
But the rule is applied to two views that have no display without analysis, `code` and
`documents`, so a user who types the obvious command is refused and told to type a
longer one:

```console
$ fdu . --view documents
fdu: --view documents requires content analysis: add --analyze lines, code, words, or all; views never enable content analysis implicitly
```

Getting code lines by language and words by document format in one report takes four
names today, which is how the announcement demo spells it:

```shell
fdu linux --analyze code,words --view languages,documents --limit 6
```

This plan makes a content view a request for the analysis it shows, so the common
reports need one flag, and `--analyze` is needed only for control a view does not give:
analysis shown in a metadata view, analysis run without display, or a wider set than the
views need.

```shell
fdu . --view code                  # code lines by language
fdu . --view documents             # words and pages by document format
fdu . --view code,documents        # both, from one scan and one analysis pass
```

The rule moves into the request model, so the engine, the command line, and the Python
package derive it identically, and nothing a surface does can disagree with it.

## Goals

- A report that only analysis can show needs one name to ask for it, on every surface.
- `--analyze` stays, as the control axis: it adds analyzers a view does not imply and
  never has to be repeated beside a view that implies them.
- Both directions of derivation live in one typed model: a view states the analyzers it
  needs, an analyzer set states the view it displays in by default, and a test pins that
  the two agree.
- A view with a metadata meaning (`languages`, `types`, `families`, `full`, the tree)
  never acquires a read of file bodies by implication.
  Only a view with no metadata meaning implies anything.
- A held basis (an opened or retained index) is never widened by a read: the refusal
  stays, and names the way to open the root with the analyzer.
- Every explanation is a classified `note:` or `tip:` line after the output, as concise
  as the epilogue work under `fdu-qci5`’s parent epic makes the rest.

## Non-Goals

- Renaming the analyzers or the views.
  The vocabulary split — analyzers name measurements, views name populations — is kept;
  the case for and against changing it is recorded below.
- Letting a sort key, a format, or a selection filter enable an analyzer.
- Live content analysis under `--watch`, or any change to what a watch can serve.
- A combined preset view for code and documents.
  `--view code,documents` is the composition, and `full` remains the digest.

## Background

### The two axes today

| Axis | Flag | Python | Values | Part of |
| --- | --- | --- | --- | --- |
| Content | `--analyze` | `AnalysisOptions(analyze=...)` | `none`, `lines`, `code`, `words`, `all`, comma sets | `Basis` (cache and index identity) |
| View | `--view` | `Query(views=...)` | `list`, `summary`, `tree`, `families`, `types`, `extensions`, `languages`, `code`, `documents`, `largest`, `recent`, `files`, `full` | `Query` (one read) |

`AnalysisSet` (`crates/fdu-core/src/content/content_model.rs`) is a set over three
independent analyzers; `code` and `words` each include `lines`. `ViewSpec`
(`crates/fdu-core/src/query/query_report.rs`) is the view vocabulary.
`Request::build` (`crates/fdu-core/src/query/query_request.rs`) parses one
surface-neutral `RequestSpec` for both the command line and Python, and `Request::read`
parses a `ReadSpec` against a basis an index already holds.

Four rules relate the axes, each in the request model:

1. **Analyzers choose a default view.** `ViewSpec::default_for` and
   `ViewSpec::resolve_rejecting`: with no `--view`, `code` shows `code`, `words` shows
   `documents`, `lines` shows `families`, and `code,words` or `all` show
   `code,documents`. This exists because the first content axis shipped without it, and
   `--analyze` changed nothing but the performance footer.
2. **A view never enables an analyzer.** `check_views`: `--view code` without code
   analysis and `--view documents` without any analyzer are usage errors (exit 2), with
   the sentence quoted above.
   A metric sort such as `--sort code_lines` is refused the same way.
3. **`full` drops what it cannot answer.** `ViewSpec::full_report` partitions the
   summary views into shown and omitted; the report says
   `note: omitted documents: content analysis required` and
   `tip: include omitted views: add --analyze lines`.
4. **Analysis with no displaying view is noted.** `query_report.rs` adds
   `note: requested analysis is not displayed by the selected views` and
   `tip: show analysis: --view families, languages, or full`.

Three views change shape with the analyzer set rather than requiring one: `languages` is
byte shares alone, and gains code, comment, and blank lines and a code-line share under
`code`; `types` and `families` gain line metrics under any analyzer and word metrics
under `words`.

### Where it rubs

- **The obvious command is refused.** `code` and `documents` display nothing without
  analysis, so refusing `--view code` protects no cheap meaning.
  The only outcome is a second attempt with `--analyze code` added.
- **Two vocabularies, one mapping to learn.** `code` is both an analyzer and a view;
  `words` displays in `documents`; `lines` displays in `families`. The usage guide,
  `--docs`, and the skill each carry a table for it, and the error for `--view words`
  exists only to teach it.
- **The combined report is long or indirect.** `fdu . --analyze code,words` already
  gives CODE and DOCUMENTS in one run, but it reads as “run analysis”, not “show me code
  and documents”, so the demo spelled the views out as well and chose `languages` for
  rows that match the document rows.
- **The guidance is spread across four notes and tips** whose wording was written one at
  a time: “content analysis required”, “include omitted views”, “not displayed by the
  selected views”, “show analysis: families, languages, or full”.

### What the rule protects, exactly

The design principle *Cost flows one way; display follows cost* exists so that a display
choice cannot silently authorize reading every file.
Its hazard is concrete for a view that is cheap today: if `--view languages` started
reading file bodies because it *can* show code lines, a metadata report on a
million-file tree would become a content walk with nothing in the command to say so.
For `code` and `documents` there is no cheap version to protect, so the principle, read
by its reason, does not reach them.

Two things the rule also protects do still hold under this plan.
The analyzer set is part of the basis, so it decides cache identity and which sidecar
answers; implying it from a view changes where the set comes from, not what it means,
and `--view code` and `--analyze code` must share one sidecar and one answer.
And an index holds the analyzers it was opened with; a read can display only what the
basis paid for, so a view a held basis cannot answer is still refused.

## Options Considered

The maintainer pressure-tests designs, so each option carries its case against,
including the recommendation.

### A. Keep the model; teach it better

Leave rules 1 through 4, shorten the four messages, and document
`fdu . --analyze code,words` as the one-flag way to get both reports.

*For:* no model change, no golden churn beyond wording, and the “a view never reads”
rule stays absolute, which is the easiest form to state.

*Against:* the refusal of `--view code` remains ceremony: the request is unambiguous and
the only effect is a retype.
The two-vocabulary mapping stays the first thing a user learns.
The demo command gets no shorter unless it drops to the `code` table.

### B. A content view implies its analyzer (recommended)

`code` implies `code`; `documents` implies `words`; no other view implies anything.
Rule 1 stays, so `--analyze` alone still displays.
`--analyze` adds to what the views imply.

*For:* the shortest obvious command works on every surface; the model gains one table
beside the one it has, and a round-trip test ties them; metadata views keep their cost;
a held basis is still never widened.

*Against:* the rule “a view never enables an analyzer” becomes “a view with a metadata
meaning never enables an analyzer”, which is one clause longer and needs the reason
stated beside it or the next view added will get it wrong.
`--view documents` now reads every eligible text file in the tree, which on a large tree
is seconds where the refusal was instant; the progress line and the `perf:` line are
what say so.
`--analyze none --view code` runs code analysis, because `none` is the empty
set and the command line passes its default `none` whether or not the user typed it, so
the model cannot refuse the pair without the command line owning a rule.

### C. Unify the names

Either rename `documents` to `words` (and perhaps `families` under `lines` to `lines`),
so each content view carries its analyzer’s name and `--view words` is the report, or
accept analyzer names in `--view` as synonyms.

*For:* one vocabulary; the mapping table disappears from three documents.

*Against:* `--view words` names a measurement where every other view names a population
or grouping, and the output design records that distinction as deliberate: headers name
views and columns name measurements, so `DOCUMENTS` over a table whose column is `words`
is the shape that keeps a word share from reading as a byte share.
Synonyms give one name two meanings, which the design principles refuse elsewhere.
Option B gets the same keystroke saving without touching the vocabulary; C can follow it
later if the names still rub.

### D. Shorthand flags

`--code` and `--words` as `--tree`, `--long`, and `--full` are; or a bare `--analyze`
meaning `all`.

*For:* `fdu . --code --words` is short, and the format and bound shorthands set a
precedent.

*Against:* those three shorthands each stand for one value on one axis; content
shorthands multiply per analyzer and per view.
A bare `--analyze` needs an optional value, and `fdu --analyze linux` then reads `linux`
as the value, or needs `require_equals`, which breaks the `--analyze code` spelling the
documentation uses.

### E. One axis: drop `--analyze`

Views alone decide what is read: `code`, `documents`, and a `lines` view.

*For:* the simplest story.

*Against:* it loses the two uses `--analyze` exists for beyond display: analysis shown
in a metadata view (`--analyze code --view languages`, the demo’s compact rows;
`--analyze lines --view languages`, physical lines across every language;
`--analyze words --view types`, word metrics across every text type) and analysis run to
warm the sidecar without display.
Each would need a new view or an option on a view, which is the same axis under another
name.

### F. A combined preset view

`--view content`, or similar, as `code,documents`.

*For:* one word for the demo.

*Against:* `code,documents` is two words with a comma, both already learned; `full`
exists as the digest; a preset earns its place as `largest` and `recent` did, by
replacing a composition people type repeatedly, and nothing shows that yet.

### Sub-decisions inside B

- **`documents` implies `words`, not `lines`.** `--view documents` answers “how much
  prose is here, by format?”, and words with pages is that answer; raw words from
  `lines` are a cheaper approximation nobody asks for by name.
  The check for a held basis tightens to match: `documents` needs `words`, so
  `--analyze lines --view documents` is refused instead of showing raw words.
  *Against:* that combination worked, and raw words by document type over an index
  opened with `lines` is lost; `--analyze lines --view types` still shows raw words per
  type.
- **Metric sorts do not imply.** `--sort code_lines` without code analysis stays a
  refusal naming the analyzer.
  A sort orders rows; it does not say what the report is about, and a selection knob
  that turns on a full-tree read is more surprising than a view named `code`. *Against:*
  the request is unambiguous, so by B’s own argument the refusal is ceremony; it is kept
  because the rule “only a view implies” is one sentence and “a view or a sort implies”
  needs a list.
- **`full` implies nothing.** It is the metadata digest by default and names what it
  skipped; `--view full --analyze all` is the whole report.
  *Against:* a reader who learned B may expect `full` to run everything; the note and
  tip carry the correction.
- **No note when a view implies an analyzer.** The report shows word or code columns,
  the progress line shows the analysis phase, and the `perf:` line counts fresh and
  cached records; a note restating that the user got what they asked for adds a line to
  every such report. *Against:* a first-time user does not learn that `--analyze` exists
  from this path; the usage guide and `--docs` carry that.

## Design

### The rule

A view with no metadata meaning is a request for the analysis it shows.
Building a request from a fresh basis — the command line, `fdu.report`, and a Rust
caller of `Request::build` — enables the analyzers the views imply, in union with the
ones `--analyze` names.
Reading a basis an index already holds enables nothing: a view the basis cannot answer
is refused with the analyzer named.
Analyzers requested with no view still choose their default view.

### Core model

In `query_report.rs`, beside `ViewSpec::default_for`:

```rust
/// The analyzers a view shows and so, on a fresh basis, requests.
///
/// Only a view with no metadata meaning: `languages`, `types`, and `families` gain
/// metrics when an analyzer is present but display without one, so they never imply.
pub const fn implies(self) -> AnalysisSet {
    match self {
        Self::Code => AnalysisSet::CODE_ONLY,
        Self::Documents => AnalysisSet::WORDS_ONLY,
        _ => AnalysisSet::NONE,
    }
}
```

A unit test pins the round trip for every view that implies: `default_for(v.implies())`
is `v`, so the two tables cannot drift.

In `query_request.rs`:

- `Request::build` parses `--analyze`, then the view list (without `full` expansion,
  which depends on content), takes the union of each named view’s `implies()`, and
  builds the basis with that union.
  Refusal order is unchanged: content grammar, view grammar, selection, page
  denominator, scope.
- `Request::read` is unchanged in shape: it resolves views against the held basis.
- `check_views` becomes one test, `content.contains(view.implies())`, replacing the two
  arms for `Code` and `Documents`. It can fail only for a held basis (and for a metric
  sort, whose rule is separate), so `RequestError::ViewNeedsContent` and `NeedsAnalyzer`
  merge into one refusal that names the view, the analyzer, and the held set, in the
  surface’s names:
  `view code needs code analysis; this index was opened with analyze none`.
- `validate_delivery`: a watch whose content is non-empty is refused today by
  `WatchContent`, which names `--analyze`. When the content came from a view, the
  refusal names the view:
  `--view code needs code analysis, which --watch cannot keep current; use a one-shot report`.
  The request records whether any analyzer was implied, for this message alone; it is
  not part of the answer.
- `Basis::build` is unchanged: a holder fixes its analyzers explicitly.
- `AxisNames` already names both axes on both surfaces.

`RequestDefaults.content` stays `none`. `AnalysisSet::NONE_LABEL` stays `none`, now read
as “no analyzers beyond what the views imply”.

### Command line

`--analyze` keeps its grammar and its default.
Its help line becomes “Analyzers to run beyond what the views imply: none, lines, code,
words, or all”. `--view`’s help names `code` and `documents` as views that read file
contents. `--docs` and the start-here examples add `fdu . --view=code`,
`fdu . --view=documents`, and `fdu . --view=code,documents`, and keep
`--analyze lines --view languages` and `--analyze code --view languages` as the control
examples. Nothing else in `cli.rs` changes: the flag reaches `RequestSpec.analyze` as it
does today, and the model does the rest.

### Python

`fdu.report(root, Query(views=(View.CODE,)))` runs code analysis, as the command line
does, because `report_once` builds a fresh basis through `Request::build`.
`AnalysisOptions` keeps its place as the control axis and is required for `open` and
`scan`, which fix a basis that serves many reads.
`index.report(Query(views=(View.DOCUMENTS,)))` on an index opened without `words` raises
`InvalidArgumentError` with the refusal above in Python names.
Docstrings on `Query.views`, `AnalysisOptions`, `report`, and `open` state this split;
no parameter is added or renamed.
The parity shim passes both strings through untouched and needs no change.

### Notes and tips

Every message is a `note:` or `tip:` after the output.
Final wording belongs to the concision pass (`fdu-wzpx`); these are the facts each line
carries and a proposed spelling.

| Situation | Today | Proposed |
| --- | --- | --- |
| `--view documents` without analysis | usage error, exit 2 | a report; no note |
| `--analyze code --view summary` | `note: requested analysis is not displayed by the selected views` and `tip: show analysis: --view families, languages, or full` | `note: code analysis not shown by summary` and `tip: show it: --view code`, naming the default view for the enabled set (`--view code,documents` under `all`) |
| `--view full` without analysis | `note: omitted documents: content analysis required` and `tip: include omitted views: add --analyze lines` | `note: full omits code, documents without analysis` and `tip: include them: --analyze all`, naming the union of what ran and what the omitted views imply |
| `--view words` | `invalid --view "words": words is an analyzer; use --analyze=words with --view=documents` | `invalid --view "words": words is an analyzer; its view is documents` |
| held index, `views=code`, opened with `none` | `code view requires code analysis: add analyze code; views and sorts never enable analysis implicitly` | `view code needs code analysis; this index was opened with analyze none` |
| `--watch --view code` | `--analyze is not yet supported with --watch; use a one-shot report` | `--view code needs code analysis, which --watch cannot keep current; use a one-shot report` |
| `--sort code_lines` without code | unchanged refusal, shortened | `--sort code_lines needs code analysis: add --analyze code` |

As implemented, two lines go further than the table’s first draft.
Under one analyzer `full` names the one that is missing
(`note: full omits documents without words analysis` after `--analyze code`), and its
tip names the union of what ran and what the omitted views imply rather than the omitted
views’ set alone, because `--analyze words` after `--analyze code --view full` would be
a command that drops the code analysis it already had; in practice the tip is always
`--analyze all`. And `--view lines`, whose view does not imply it, is refused as
`its view is families, with --analyze lines`.

The parity class `sameAnalysisTip` in `scripts/parity-classes.mjs` pins the
omitted-views tip in both vocabularies and is updated to the new text; `KNOBS` already
elides the axis names.

### Structured output

`request.analyze` continues to carry the analyzers the request enabled, now the union of
named and implied; `analysis.analyze` and the sidecar identity follow it, so
`--view code` and `--analyze code` write and read one sidecar.
`request.views` and `request.omitted_views` are unchanged.
No field is added or renamed and no meaning changes, so `fdu.report/10` is not bumped.
Whether an analyzer was implied is request provenance, not part of the answer, and is
not serialized.

### Before and After

| Question | Before | After |
| --- | --- | --- |
| Code lines by language | `fdu . --analyze=code` | `fdu . --view=code` (the old form still works) |
| Words and pages by document format | `fdu . --analyze=words` | `fdu . --view=documents` (the old form still works) |
| Both | `fdu . --analyze=code,words` | `fdu . --view=code,documents` (also `--analyze=all`) |
| The announcement demo | `fdu linux --analyze code,words --view languages,documents --limit 6` | `fdu linux --view code,documents --limit 6` |
| The demo with compact per-language rows | same | `fdu linux --analyze code --view languages,documents --limit 6` |
| Code metrics in the languages rows | `fdu . --analyze=code --view=languages` | unchanged (control) |
| Physical lines across every language | `fdu . --analyze=lines --view=languages` | unchanged (control) |
| Warm the sidecar, show the tree | `fdu . --analyze=all --view=tree` | unchanged, with the shorter note and tip |
| Everything | `fdu . --view=full --analyze=all` | unchanged |
| Python, one report | `fdu.report(root, Query(views=(View.CODE,)), analysis=AnalysisOptions(analyze="code"))` | `fdu.report(root, Query(views=(View.CODE,)))` |
| Python, an index | `fdu.open(root, analysis=AnalysisOptions(analyze="code")).report(Query(views=(View.CODE,)))` | unchanged |

The demo’s second command drops two names.
Whether it should show the `code` table or the `languages` rows is the maintainer’s
call; the spec records both spellings.

### Documentation

- `docs/project/architecture/fdu-design-principles.md`: *Cost flows one way; display
  follows cost* is restated as above, with the reason beside the clause so the next view
  is classified correctly; the six-axes table’s content row reads “which file bodies are
  read beyond what the views imply”.
- `docs/project/architecture/fdu-output-design.md`, *Content Reports and Names*: the
  mapping stays, with the sentence that each content view requests its analyzer.
- `docs/project/architecture/fdu-surface-architecture.md`: the deviation-class list
  names the new omitted-views tip; the one-behavioural-difference section gains the
  `report` versus `open` asymmetry for implied analyzers.
- `docs/usage.md`: *Start with the Current Directory* adds the three view commands;
  *Analyze File Contents* and *Measurements, Views, and Headers* are rewritten around
  the rule, keeping the mapping table.
- `docs/machine-output.md`: `request.analyze` is the enabled set, named or implied.
- `README.md`: the quick start uses `fdu . --view=code,documents`; the question table
  adds “Words in documents”.
- `crates/fdu/src/skills/SKILL.md` and the `--docs` text in `cli.rs`: the run list, the
  axes table, and the “Pick the View” section; the skill test
  `the_skill_only_names_views_and_analyzers_that_parse` covers the vocabulary.
- `crates/fdu-py/README.md` and the Python docstrings named above.
- `packages/cli-animate/examples/fdu/linux.yaml`, `showcase.yaml`, `views.yaml`, and the
  cli-animate README: the demo commands, in whichever spelling the maintainer picks; the
  recording is regenerated under `fdu-qci5`.
- `tests/qa/cli-installed-e2e.qa.md`: the `documents-no-analyze` row expects a report.

## Implementation Plan

One phase, one draft pull request, landed after or merged with the epilogue work
(`fdu-cmr4`, `fdu-wzpx`), which edits `report_format.rs`, `report_epilogue.rs`,
`query_report.rs`, and `cli.rs`. The order below is the dependency order of the beads.

- [x] Request model: `ViewSpec::implies`, the round-trip test, `Request::build` taking
  the union, `check_views` as one containment test, the merged held-basis refusal.
- [x] Delivery refusal: `--watch` with implied content names the view.
- [x] Notes and tips: the not-displayed tip names the default view; the omitted-views
  tip names the implied union; the `--view words` refusal; wording settled with
  `fdu-wzpx`.
- [ ] Goldens, parity class, Python tests, and the path-independence matrix.
- [ ] Help, `--docs`, skill, usage guide, README, machine-output reference, output
  design, surface architecture, design principles.
- [ ] Demo scripts and the cli-animate README.

## Testing Strategy

- **Unit, `query_request.rs`:** `--view code` alone builds a basis with `lines,code`;
  `--view documents` alone builds `lines,words`; `--view code,documents` builds `all`;
  `--view full` and `--view languages` alone build `none`; `--analyze lines --view code`
  builds `lines,code`; `Request::read` with `views=code` against a basis of `none`
  returns the merged refusal in both vocabularies; `--watch --view code` returns the
  view refusal; every refusal is quoted whole in
  `every_refusal_renders_in_flag_and_field_wording`.
- **Unit, `query_report.rs`:** `default_for(v.implies()) == v` for `Code` and
  `Documents`; `implies()` is `NONE` for every other view; `full_report` is unchanged.
- **Goldens, `tests/golden/cli-content.tryscript.md`:** the `--view documents` session
  turns from a refusal into a report; a `--view code,documents` session is added beside
  the `--analyze code,words` one and must print the same stdout; the not-displayed and
  omitted-views sessions carry the new lines; the “Analyzer Set Chooses the View”
  section gains its converse.
  `cli-surface.tryscript.md` follows the help, docs, and skill text.
  Every diff is read; `--update` output is checked for expanded patterns.
- **Parity:** `make check` replays the corpus against Python; `sameAnalysisTip` is the
  only class that changes.
  `crates/fdu-py/tests/public_smoke.py` asserts that `fdu.report` with `documents`
  succeeds and that `index.report` with `documents` on an index opened without `words`
  is refused in Python names.
- **Path independence, `tests/path_independence/matrix.py`:** `v_code`, `v_documents`,
  and `v_code_documents` specs built from views alone, asserted equal to `a_code`,
  `a_words`, and `a_all` on content, tree status, and sidecar identity, across cold,
  warm, and `stale_ok` histories.
- **Correctness runbook:** cache identity is unchanged in form but the set now has a
  second source, so the runbook’s content-cache pass is run once before the release that
  ships this.
- `make check` is the gate; no platform-gated code changes.

## Open Questions

1. Should the demo show the `code` table (`--view code,documents`) or the compact
   per-language rows (`--analyze code --view languages,documents`)? The first is the
   one-flag form this plan exists for; the second is what the demo author chose for row
   symmetry.
2. Is `documents` requiring `words` on a held basis acceptable, losing
   `--analyze lines --view documents` (raw words by document format)?
3. Should `--analyze none` beside a content view be refused rather than read as the
   empty set? Refusing needs the command line to distinguish a typed `none` from its
   default, which is a rule outside the model.
4. Is the vocabulary split (option C) settled, or should `documents` be revisited once B
   has shipped and the mapping table is no longer the first thing a user meets?

## References

- [Design principles](../../architecture/fdu-design-principles.md), *First Principles*
  and *One Scan, Many Views*
- [Surface architecture](../../architecture/fdu-surface-architecture.md)
- [Output design](../../architecture/fdu-output-design.md)
- [Usage guide](../../../usage.md), *Analyze File Contents*
- [Code analysis presentation plan](../done/plan-2026-09-26-code-analysis-presentation.md),
  which fixed the current mapping and the `code` view
- [File content metrics plan](../done/plan-2026-08-12-fdu-file-content-metrics.md),
  which introduced “a view never silently enables an analyzer”
- [View vocabulary plan](../done/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md)
- Epilogue work in progress: `fdu-cmr4`, `fdu-wzpx`, and their parent epic

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
