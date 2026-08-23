# Feature: View Vocabulary and the Output Contract

**Date:** 2026-08-21

**Author:** fdu project

**Status:** Draft

## Overview

Split the `files` view into three views whose defaults each answer one question, rename
the view total from `all` to `full`, and make every bound and every format a stated
contract rather than an assumption.

The work comes from one defect and its root cause.
`fdu --view files` on a 192,871-entry tree printed ten rows and said nothing about the
other 192,861. Those ten were the alphabetically first, because the view defaulted to
name-ascending order *and* to a ten-row cap — two ordinary choices that do not compose.

## Goals

- Each view’s defaults answer a question that can be stated in one sentence.
- No output is bounded without saying so, in every format, with the flag that lifts it.
- Every view renders in every format, proven rather than claimed.
- `largest` and `recent` are documented as aliases, so a reader learns the composition
  instead of memorising two more views.

## Non-Goals

- Backward compatibility.
  fdu is pre-release and no consumer is owed a migration.
- A general query language.
  `largest` and `recent` are presets over the existing axes, not a new grammar.
- Content-analysis changes.
  The content axis is settled and untouched here.

## Background

`files` was three views wearing one name.
Its stated job was to subsume `fd` and `find`, which enumerate; its default cap made it
a sample; and its name-ascending order only makes sense for a complete listing, because
the diffability that justifies that order disappears the moment the list is truncated.

Each default passed review on its own.
Nothing examined the pair, which is why
[the design doc’s First Principles](../../architecture/fdu-design-principles.md#first-principles)
now require a default to answer a statable question and require defaults to be checked
in combination.

The composable CLI spec removed `largest` and `recent` by asking whether they could be
expressed as a composition of existing axes.
They can — and that was the wrong test.
It conflates capability with interface: `files --sort size --limit 20 --kind file` is a
composition the caller must already know how to build, while `largest` is an answer to a
question they arrived with.
Principle 1 guards against parallel *machinery*; a named preset over the same code path
is not that.

## Design

### The view vocabulary

| View | Sort | Bound | Kinds | The question it answers |
| --- | --- | --- | --- | --- |
| `files` | name asc | none | all entries | What is in here? |
| `largest` | size desc | 20 | files only | What is eating my disk? |
| `recent` | mtime desc | 20 | files only | What changed? |

`largest` and `recent` are aliases, and the documentation states the equivalence rather
than describing them twice:

```text
largest ≡ files --sort size  --limit 20 --kind file
recent  ≡ files --sort mtime --limit 20 --kind file
```

An explicit `--sort`, `--limit`, or `--kind` overrides the preset, so
`--view largest -n 100` behaves exactly as the composition would.
Directories are excluded from both because `tree` already reports directory sizes; a
`largest` listing directories duplicates it at a coarser grain and pushes the actual
files out of the window.

`files` returning everything is a correctness matter, not a preference.
The spec’s own subsumption table uses it for an incremental-sync watermark, and a
watermark query that silently returns twenty of 192,871 changed files loses data.

### `--view all` becomes `--view full`

`full` is every *summary* view, in table order, and now includes both `largest` and
`recent`. `files` is excluded: an unbounded enumeration is not a summary, and putting
one inside a digest destroys the digest.

The rename carries meaning.
`--analyze all` means literally every analyzer, and a view total cannot mean literally
every view once one view is an enumeration.
`full` reads as “the full report” rather than “every value”, and the different word
marks the different semantics — which is the distinction wanted when the two totals were
first named.

`documents` is still omitted without content analysis, and the omission is still named.

### The truncation contract

Every bounded view states its bound, per
[Truncate Freely; Never Truncate Silently](../../architecture/fdu-design-principles.md#truncate-freely-never-truncate-silently):

```console
LARGEST  (20 of 192,871 files, by size; --limit all for every one)
```

A count rather than a bare marker, because `…` after twenty rows of 192,871 understates
the situation by four orders of magnitude, and the flag that lifts the bound is named
where the bound is stated.

Machine formats carry the same honesty as a section-level field, because a consumer
reading twenty rows must be able to tell it received twenty of 192,871. That changes the
machine shape and therefore bumps the report schema.

### Every view in every format

Principle 9 says formats are serializations, not features, and that every view renders
in every format. That is currently claimed rather than proven: `extensions` and `files`
are exercised only in text, and several `yaml` combinations are untested.

The matrix becomes a test rather than an assertion — every view crossed with `text`,
`json`, `jsonl`, and `yaml`, so a view that renders in three formats and panics in the
fourth fails here.

Rendering is only half of it.
A byte-stable golden proves the output has not *changed*, never that it is *valid*: a
consistently malformed document passes forever, and these serializers are hand-written
because the project avoids serde, so nothing else would notice.
Audited today, `json` is parsed by the content self-check, `jsonl` is only ever
substring-matched, and `yaml` has never been read by a YAML parser at all — which is the
sharp end, since quoting is the fiddly part of that format.

So each format is *consumed* in a golden rather than only compared: run the command,
pipe it into a parser, print a field.
That demonstrates the format working, and the demonstration is also the example a reader
learns from. `jsonl` parses line by line, which is the whole of its contract.
`yaml` needs a parser that does not exist here yet — CI installs neither jq nor yq, and
node ships no YAML support — so a pinned `yaml` devDependency goes through
[the supply-chain policy](../../../../SUPPLY-CHAIN-SECURITY.md) first.

### API Changes

- `ViewSpec` gains `Largest` and `Recent`; `files` loses its default bound.
- The view total parses as `full`; `all` is no longer accepted on the view axis.
- Flat sections carry the total they were drawn from, so a bound can be reported.
- The report schema bumps for that field.
- The Python `View` enum tracks the vocabulary, and the native `contract()` list with
  it.

## Implementation Plan

### Phase 1

- [ ] `files` becomes complete: name ascending, no default bound (`fdu-qbwf`)
- [ ] Add `largest` and `recent` as presets resolved at the CLI layer, overridable by
  `--sort`, `--limit`, and `--kind` (`fdu-xc1v`)
- [ ] `--view all` becomes `--view full`, membership defined as the summary views
  including `largest` and `recent` (`fdu-j1dc`)
- [ ] Flat sections carry their source total; every bounded view states what it dropped
  in its header, and names the flag that lifts it, in text and machine formats
  (`fdu-c1qh`)
- [ ] Cross every view with every format so the render matrix is tested rather than
  claimed, closing the `extensions`, `files`, and `yaml` gaps (`fdu-5akc`)
- [ ] Consume each machine format with a parser rather than only comparing bytes; the
  `yaml` dependency goes through the supply-chain policy first (`fdu-c2ml`)
- [ ] Carry the vocabulary through `--docs`, README, SKILL.md, help, the `--view` error
  message, the composable CLI spec, and the goldens (`fdu-k4ad`)

The schema bumps once, for `fdu-c1qh` and `fdu-c2ml` together, rather than twice.

## Testing Strategy

Golden sessions are the text contract, so each new view gets one and the bound statement
is pinned rather than eyeballed.

The format matrix is a unit test rather than golden sessions: crossing eight views with
four formats as goldens would add thirty-two blocks that mostly restate one another,
while a table-driven test asserts the property — every view renders, in every format,
without panicking and with the schema its content warrants.

Two properties the golden suite structurally cannot check, so both are unit tests:

- colour never changes layout, because goldens run under `NO_COLOR=1` and only ever see
  the uncoloured form — which is how the extensions view shipped misaligned
- a bounded view’s stated count matches the number it actually dropped, since a golden
  fixture is too small for the difference to show

## Rollout Plan

One change on the existing branch, since the vocabulary is not coherent halfway: `files`
complete but `largest` absent would leave no bounded flat view at all.

No migration is owed.
The vocabulary change is announced by the `--view` error message, which lists the
accepted values, and by `--docs`.

## Decisions Taken

**The bound is stated in the section header.** A footer is lost to `head`, which is the
one place the notice matters most: `fdu --view largest | head -5` cuts a footer off and
loses the very warning it exists to give, while a header survives.
In a `full` report a header also keeps each bound attached to its own section, where a
footer would float free of the rows it describes.
The name stays ALL CAPS in the heading colour and the qualifier follows in the telemetry
colour — the role the performance footer already uses for “what the tool did” as against
“what it found”.

**`recent` bounds by count alone.** It answers “what changed most recently”, and twenty
rows answer it. A time window answers a different question — “what changed in the last
hour” — and that one is already `--modified-since 1h`, which composes with `recent` for
a caller who wants both.
A second default would mean defending a specific window, and no window is defensible for
every tree.

## Open Questions

- None blocking. The `yaml` parser dependency is a supply-chain decision rather than a
  design one, and is tracked on `fdu-c2ml`.

## References

- [Design principles: First Principles](../../architecture/fdu-design-principles.md#first-principles)
- [Composable CLI and query surface](plan-2026-08-10-fdu-composable-cli-surface.md)
- Beads: `fdu-yov0` (epic), `fdu-qbwf`, `fdu-xc1v`, `fdu-j1dc`, `fdu-c1qh`, `fdu-k4ad`,
  `fdu-1lj3` (the original silent-truncation report)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
