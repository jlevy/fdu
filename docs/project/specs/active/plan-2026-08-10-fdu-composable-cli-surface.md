# Feature: Composable CLI and Query Surface

**Date:** 2026-08-10

**Author:** fdu project

**Status:** Completed.
[PR #5](https://github.com/jlevy/fdu/pull/5) shipped the five-axis surface (Phases 1–4),
and [PR #37](https://github.com/jlevy/fdu/pull/37) shipped this revision’s content axis
and display contract (Phase 5). [PR #39](https://github.com/jlevy/fdu/pull/39) then
reshaped the view vocabulary under
[the view vocabulary plan](plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md).
Later work changed parts of the surface this plan specifies: `.gitignore` observation on
by default with its scope and selection flags (PRs #63 and #65), report schemas
`fdu.report/5` and `/6`, and cache states with stale-snapshot clearing (PR #67). Where a
section below states the shipped behavior, it has been corrected to match `origin/main`
on 2026-09-16; checked implementation items record what each PR did.
`fdu --help` and `fdu --docs` are the live reference.
Follow-ups remain open under the epics `fdu-pxeb` and `fdu-ktyl`.

## Overview

Reshape the fdu command line and the library query layer around a small set of
orthogonal, reusable concepts instead of accumulating per-feature flags.
One invocation composes six axes — scan scope, content, selection, views, format, and
mode — over the same engine, so one command grammar covers what du, dust, dut, diskus,
fd, find, and tokei each do separately, plus a `tail -f`-style change feed and
timestamp-watermark queries reliable enough to drive backup and sync tooling.
The same concepts appear as typed values in the Rust API and the Python API, so the CLI
is a thin composition of the library rather than a second implementation.

This plan supersedes the flag-by-flag surface produced by the
[CLI UX plan](plan-2026-08-09-fdu-cli-ux-and-agent-skill.md) and gives the remaining
Phase 1 CLI beads (`fdu-oqoy`, `fdu-jej9`) a coherent target rather than a list of
incremental additions.

## Goals and Design Principles

These are the accountability criteria for every iteration of this design.
A change that violates one of these needs this spec amended first, not a silent
exception. The governing aspiration: the design fits the contours of the true problem —
no more complexity, but no less either.
Simple things are simple (`fdu PATH` is a good answer), and complex things are possible
(any axis composes with any other), as with all good developer tools.
Bare `fdu` is deliberately safe discovery: it prints help instead of assuming that a
possibly enormous current directory should be scanned.
The concrete test for “no more complexity”: before adding a view or flag, show it cannot
be expressed as a composition of the existing axes — `largest` and `recent` views were
first removed from this design by exactly that test, and later reinstated as presets
because the test conflated capability with interface (see Views).

1. **Six axes, no one-off flags.** Every option belongs to exactly one axis: scan scope,
   content, selection, view, format, or mode.
   A proposed flag that does not fit an axis is a design smell; either it generalizes
   into an axis value or it does not ship.
   The content axis was added by this revision: `--analyze` shipped after the original
   five and was never given a slot, which is the direct cause of the display gap fixed
   below. An axis whose values are a closed vocabulary is a comma-delimited list, never a
   hand-enumerated enum of the combinations — enumerating a power set is the same smell
   as a one-off flag, one level down.

2. **Intuitive by default, everything by composition.** `fdu <path>` with no flags gives
   a useful, fast report with sensible, obvious defaults.
   The path is mandatory for reports; bare `fdu` prints help and performs no scan.
   There are no subcommands: the grammar is always “report on a path,” customized by
   composable flags, so a path argument can never be shadowed.
   Help documents each axis, its options, and its defaults plainly enough that the
   design is legible from `--help` alone.

3. **One scan, many views.** Views are projections over the in-memory index.
   Requesting more views in one invocation never adds filesystem work, and two reports
   over the same tree come from the same consistent index state.

4. **Views are readers.** The Delta contract stands: `scan` and `watch` produce
   observations, the index consumes them, and views only read the index.
   No view or format may add a mutation path or reach into producer state.

5. **Speed may be traded for certainty, never for honesty.** Every value carries its
   provenance — where it came from, when it was observed, whether it is final — so a
   consumer rendering a thousand rows knows which ones to trust, not merely that the run
   was partial (Goal 7 of the research;
   [the provenance model](plan-2026-08-11-fdu-progressive-results.md)). Provenance is a
   property of the value in the library; every surface displays it rather than inventing
   it. `auto` may choose the cheapest sound path per tree, and that choice is legible in
   the report’s `source` — a hidden choice would violate this principle, a visible one
   is how it is implemented.
   The explicit policies (`refresh`, `read-only`, `only`, `off`) remain for callers who
   want a particular path rather than the cheapest one.

6. **Benefit from the OS; never depend on it.** Three tiers, each fast in its own right:
   a portable walk, a cache for the cases a cache can help, and platform APIs (bulk
   stat, `statx`, `io_uring`, fanotify, the FSEvents journal) probed at runtime as
   accelerators for the first two.
   A missing or degraded tier falls back to the one below, losing speed and never
   accuracy. The most valuable platform APIs do not compute answers; they **bound
   uncertainty** — naming in milliseconds which parts of a million-entry tree could have
   changed, so verification goes only there.
   That is what makes near-real-time views of huge trees possible, and why such an API
   may narrow what is checked but never substitute for checking it.

7. **Same concepts at every level; the CLI invents nothing.** `Query`, `View`, `Report`,
   and `CachePolicy` are typed values in the Rust library; the CLI parses flags into
   them and renders `Report`s; Python exposes the same types.
   Any logic beyond flag parsing, terminal handling, and exit codes belongs in the
   library, where Rust and Python callers get it too.
   A capability that exists in one surface and not the others is unfinished, and
   complexity that exists only at the CLI layer is misplaced.

8. **Subsume the neighbors.** The compositions below must each be expressible in one
   invocation; the mapping table in the Design section is a living checklist.
   Where a neighbor’s behavior is out of scope (e.g. ncdu’s TUI), that is recorded as a
   non-goal, not left ambiguous.

9. **Formats are serializations, not features.** Every view renders in every format
   (`text`, `json`, `jsonl`, `yaml`). Machine formats are schema-versioned; a schema
   change without a version bump fails a golden test.

10. **Watch is the same query, repeated.** A watch run evaluates the same selection and
    views as a one-shot run, re-applied as deltas arrive.
    There is no separate watch grammar to learn, and overflow or invalidation is
    reported explicitly in the stream, never dropped.

11. **Utilities are explicit flags, never side effects.** Cache inspection and clearing
    are lifecycle flags on the same grammar (`--cache-status`, `--cache-clear`) that run
    without scanning and without a report; a report run never deletes anything, and
    utility output composes with the format axis like any other.

12. **No unbenchmarked performance claims.** Each new output surface becomes a named
    benchmark job per the
    [performance evidence research](../../research/research-2026-08-09-end-to-end-performance-evidence.md),
    and flags are part of benchmark identity: renaming one means updating the job
    manifests in the same change.

13. **Cost flows one way; display follows cost.** `--analyze` authorizes filesystem
    reads, so a view may never enable an analyzer: choosing how to display a result must
    never silently turn a metadata walk into one that opens every eligible file.
    The reverse direction carries no such hazard — it re-projects state already paid for
    — so `--analyze` selects the *default* view, and an explicit `--view` always wins.
    The invariant this protects is that **a run displays what it paid for**: any request
    that reads content and renders none of it is a defect, and any view omitted for lack
    of analysis is named in the report rather than silently dropped.

These principles graduate into
[the standalone design doc](../../architecture/fdu-design-principles.md) once
implementation has validated them (Phase 4), so they outlive this spec.

## Non-Goals

- An interactive TUI (ncdu/gdu style).
  The composable one-shot and streaming surfaces come first; a TUI would be a consumer
  of the same `Query`/`Report` layer later.
- ~~Content-tier metrics (word counts, code metrics) and the type-rule dialect.~~
  **Superseded by this revision.** The original plan deferred these to `fdu-v4lc` and
  shipped no analyzer, and the derived-data cache layer was shaped so they could arrive
  without a format break — which worked.
  They then arrived as `--analyze` without an axis to belong to, so nothing connected
  them to the view axis and no default view displayed them.
  The content axis above adopts them; the Views section below closes the gap.
- Regex name matching.
  Selection uses globs; fd-style regex can layer on later without changing the axis.
- Multiple root paths in one invocation.
  One root, one index, one cache remains the contract; see Open Questions.
- Configuration files, prompts, pagers, and progress animation (unchanged from the CLI
  UX plan).
- Publishing or performance claims; release gates are owned by `fdu-9cf0` and the
  performance plan.

## Background

The current CLI (see the CLI UX plan) is one flat command with two views and two formats
expressed as mutually exclusive booleans (`--by-type`, `--json`), hardcoded
size-descending sort, no filtering, and rendering welded to the flag struct as private
methods. Watch exists only as a library feature with zero CLI surface.
There is no cache status, list, path, or clear operation anywhere — cache files are
opaque path-hash names under the user cache directory, with no way to map a file back to
its root. `ScanConfig` already models scope correctly (`max_depth`, `one_filesystem`,
`follow_symlinks`, `threads`) and already excludes non-semantic knobs from the cache
key, but the CLI exposes only `--max-depth`.

flowmark-rs (checked out under `attic/flowmark-rs`) is the closest first-party prior art
for cache UX and is worth borrowing from deliberately:

- `--show-cache` prints a six-line status covering both the aggregate cache root and the
  current project’s manifest; `--clear-cache` is idempotent, non-interactive, and always
  echoes the cache directory before acting.
- Cache lifecycle operations run before file-argument validation, so they need no other
  arguments, and clearing works even when the cwd is unresolvable.
- Cache-root resolution is a pure, testable function returning a path plus a named
  source tier (OS cache dir, home fallback, temp fallback), with the CLI emitting a
  distinct warning per fallback tier.
- Comma-delimited list values are split, trimmed, checked per token, and rejected with
  the valid values named inline.

fdu’s snapshot layer is stronger than flowmark’s manifest (binary format, CRC, engine
fingerprint, atomic replace, fail-closed corrupt handling), so this plan adopts
flowmark’s *surface* patterns, not its storage.

## Design

### Concept Model: Six Axes

| Axis | Question it answers | Options | Engine binding |
| --- | --- | --- | --- |
| **Scope** | What does the engine observe and retain? | `PATH`, `--scan-depth <N>`, `--one-filesystem`, `--no-gitignore`, `--gitignore-budget <SIZE\|all>`, `--gitignore-line-limit <SIZE\|all>` | `ScanConfig` → `ScanScope`, the cache identity |
| **Selection** | Which retained entries does this query consider, and how are results shaped? | `--include <GLOB>`, `--exclude <GLOB>`, `--min-size <SIZE>`, `--modified-since <WHEN>`, `--modified-before <WHEN>`, `--kind <LIST>` of `file\|dir\|symlink\|other`, `--exclude-ignored`, `--only-ignored`, `--depth <N>`, `-n/--limit <N>`, `--sort <size\|count\|mtime\|name>`, `--reverse`, `--size <allocated\|apparent>` | View-time filter over the index; never part of the cache key |
| **Content** | Which file bodies may be read, and which analyzers run over them? | `--analyze <LIST>` — comma-delimited from `lines`, `code`, `words`, plus the totals `none` and `all`; default `none` | `AnalysisSet` → the analyzer registry; part of the content sidecar identity |
| **View** | Which roll-ups or listings are reported? | `--view <LIST>` — comma-delimited from `summary`, `tree`, `families`, `types`, `extensions`, `languages`, `documents`, `largest`, `recent`, `files`, plus the total `full`; default derived from `--analyze` | Pure projections over `Index` |
| **Format** | How is the report serialized? | `--format <text\|json\|jsonl\|yaml>`, `--color <auto\|always\|never>` | Serializers over `Report`; schema-versioned |
| **Mode** | One answer or a live feed, and how is the cache used? | one-shot (default) vs `--watch [--interval <DUR>]`; `--cache <auto\|refresh\|read-only\|only\|off>`; `--allow-partial` | `open()` path selection; `watch::Watcher` driving the same query |

Content versus view is the second load-bearing distinction, and it is the one this
revision adds: content decides what is *read* (the only axis that can make a run
expensive in I/O), while view decides what is *printed* from whatever was retained.
Every view is available at every content setting; a view that has no metadata-only
projection says so rather than reading on its own behalf (Principle 13).

Scope versus selection is the load-bearing distinction, and it is why filters are cheap:
scope determines what is scanned and cached (one cache serves every query), while
selection is evaluated at view time against the retained index.
This is the same reasoning as the research’s gitignore *tag, don’t prune* decision.
The rename from `--max-depth` to `--scan-depth` (alongside render `--depth`) makes the
distinction legible; the benchmark job manifests that reference the old spelling are
updated in the same change (Principle 12).

### CLI Surface

```text
fdu [OPTIONS] <PATH>                                      # reports name their root
fdu [PATH] --cache-status[=<SCOPE>] [--cache-clear[=<SCOPE>]] # lifecycle; never scan
fdu [PATH] --cache-clear[=<SCOPE>]                        # lifecycle; never scan
fdu --docs / --skill / --help / --version                 # discovery surfaces
```

Representative compositions, which double as the subsumption checklist (Principle 8):

| Instead of | Run | Notes |
| --- | --- | --- |
| `dust` / `dut` | `fdu PATH` | tree view, warm when a cache exists |
| `du -sh` / `diskus` | `fdu --view summary PATH` | one-line totals |
| `du -a --max-depth 3` | `fdu --depth 3 -n all PATH` | unlimited entries per directory |
| `fd -e rs` / `find -name` | `fdu --view files --include '*.rs' PATH` | complete flat listing, one path per line in text |
| biggest files (`dust -f`, `find -size +10M`) | `fdu --view largest PATH` | a preset: `files --sort size --limit 20`, regular files only |
| recently modified (`find -mmin -60`) | `fdu --view files --modified-since 1h --sort mtime PATH` | humane durations, not day counts |
| `du` by type | `fdu --view types PATH` | current `--by-type` |
| `tokei` / `scc` | `fdu --analyze code --view languages PATH` | SLOC per language, one walk |
| `wc -w` over a doc tree | `fdu --analyze words --view documents PATH` | normalized and reader-visible words |
| everything fdu can measure | `fdu --analyze all --view full PATH` | one walk, every analyzer, every summary view |
| two reports, one scan | `fdu --view types,tree PATH` | one index, both roll-ups |
| `tail -f` for a tree | `fdu --watch --view files --format jsonl PATH` | one record per applied change |
| live dashboard poll | `fdu --watch --view tree,types --interval 2s PATH` | aggregate views re-render when dirty |
| forced cold measurement | `fdu --cache refresh PATH` | ignores and rewrites the snapshot |
| answer from cache alone | `fdu --cache only PATH` | never touches the tree; labeled stale |
| incremental backup sync | `fdu --view summary PATH`, then later `fdu --view files --modified-since <scan_started_at> PATH` | exact-timestamp watermark; see below |

Grammar conventions, applied uniformly so every flag behaves the way its neighbors do:

- **Reports require an explicit root.** Bare `fdu` is identical to `fdu --help` and
  returns without filesystem work; `fdu .` is the deliberate current-directory form.
- **Discovery surfaces answer without a root and never scan.** `--help` is the flag
  reference; `--docs` is the prose guide — the report ladder, both axes, and the output
  contracts; `--skill` is the same surface written for an agent.
  Prose that names a flag, view, analyzer, or schema string is tested against the
  binary, because a guide advertising something that does not exist spends the reader’s
  trust before they find out: the stale `fdu.report/2` in help survived a vocabulary
  sweep and a full gate, and only a test comparing prose to the constants caught it.
  This is the reason help carries one pointer line rather than the guide itself — the
  guide used to live in `before_help`, which printed a page of prose above the command’s
  own description.
- **Closed identifier vocabularies are comma-delimited lists** (`--view`, `--kind`):
  split on commas, trim, reject empty tokens, and name the valid values in the error for
  an unknown token (flowmark’s list pattern).
  Views render in the order given; duplicates are an error.
- **Open pattern values are repeatable flags** (`--include`, `--exclude`), never comma
  lists, because glob brace syntax (`*.{rs,toml}`) contains commas.
- **Humane value grammars, shared with the API**: sizes accept `10M`/`1.5GiB`; times
  accept ages (`45s`, `2h`, `1h30m`), absolute timestamps, `@epoch`, or `now`, per the
  single `WHEN` grammar defined under Timestamps and Sync Watermarks.
  Bounded values accept `all` for unbounded (`-n all`, `--depth all`); `--depth 0` keeps
  du’s meaning of “root totals only.”
- **Lifecycle flags take optional values** (`--cache-status[=root|all]`), defaulting to
  the current root.

Further notes:
- Replaced flags from the current pre-release surface: `--by-type` → `--view types`,
  `--json` → `--format json`, `--apparent-size` → `--size apparent`, `--number` →
  `-n/--limit`, `--max-depth` → `--scan-depth`, `--no-cache` → `--cache off`. No aliases
  are retained; the interface is explicitly pre-release and the SKILL.md, `AFTER_HELP`,
  README, and benchmark manifests are updated in the same change.
- Cache lifecycle flags follow flowmark’s optional-value pattern:
  `--cache-status[=root|all]` and `--cache-clear[=root|all]`, both defaulting to `root`
  (the resolved `PATH`). They run before scan validation, so they need no readable tree,
  suppress the report, and may be combined in one invocation (clear runs first, then
  status).
- Exit codes are unchanged: 0 complete (or partial with `--allow-partial`, or broken
  pipe), 1 fatal, 2 partial or usage error.
  `--cache only` with no usable snapshot is fatal (exit 1) — there is no data to answer
  with, and guessing would violate Principle 5.

### Views

Each view is a pure function of the index and a `Selection`, returning one section of
the `Report`. Each is a different grouping of the same underlying entries, and the
grouping is the only thing that distinguishes them:

| View | Grouping | Uses `--analyze` | Default sort | Subsumes |
| --- | --- | --- | --- | --- |
| `tree` | by directory hierarchy | no | size desc | du, dust, dut |
| `summary` | everything in one group | no | — | du -s, diskus |
| `files` | none (individual entries), complete | no | name asc | fd, find |
| `largest` | none (regular files), 20 rows | no | size desc | dust -f |
| `recent` | none (regular files), 20 rows | no | mtime desc | find -mmin |
| `extensions` | by raw filename extension | no | size desc | du by extension |
| `types` | by detected file type | yes | size desc | current `--by-type` |
| `families` | by content family (`code`, `prose`, `markup`, `data`, `binary`, `unknown`) | yes | size desc | — |
| `languages` | by detected language | yes | size desc | tokei, scc |
| `documents` | prose and markup rows | **requires** | size desc | wc |

The `Uses --analyze` column is a contract, not a description: it is what makes Principle
13 checkable. Four views ignore content analysis entirely, so requesting analysis while
selecting only those views is a request that pays for I/O it cannot display, and the run
says so (see *Displaying what was paid for* below).

`largest` and `recent` are named presets over `files`, not separate machinery: `largest`
is `files --sort size --limit 20` and `recent` is `files --sort mtime --limit 20`, each
restricted to regular files, and `--sort` and `--limit` still override them.
This plan first removed both, on the test “can it be expressed as a composition of
existing axes?” That test conflated capability with interface.
A composition the caller must already know how to build is not a default, and the cost
showed up in `files` itself: asked to serve as enumeration and top-N at once, it paired
name order with a ten-row cap and printed the ten alphabetically first entries of a
192,871-entry tree.
[The view vocabulary plan](plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md)
reinstated them as presets and made `files` complete.
The same composability makes `tree --sort mtime` an activity map of a project with no
extra machinery. When the reducer registry (Goal 6) and type rules (`fdu-v4lc`) land,
they extend this table’s *columns and groupings* — new metrics per row, content-aware
`types` — rather than adding parallel flags, which is the extension contract that keeps
the axis count fixed.

In text format, `files` prints one path per line, followed in a one-shot run by the
standard performance footer.
Programmatic consumers use a machine format, where per-entry fields (size, mtime, kind)
are carried and transient performance telemetry is omitted.
Two performance tiers, stated so nobody is surprised later: an unfiltered `tree`,
`types`, or `summary` view reads the pre-computed `RollUp` state directly; any selection
filter (and the `files` view) traverses the retained index in memory.
Both are milliseconds warm; neither touches the filesystem.
`ExtTally` gains an `allocated` field so the `types` view honors `--size allocated`
instead of silently switching metrics.

*Performance-only implementation amendment.* A one-shot `--no-gitignore --view summary`
with no selection filter and no content analysis derives an exact-summary plan instead
of retaining an index: it retains only aggregate tallies and neither reads nor writes a
snapshot, under every cache policy except `only`, and `refresh` with a cache path.
Every other request retains the full index, including the default summary, which
observes `.gitignore` to report its ignored share and so needs the table the tallies do
not keep (`fdu-if7o` records the memory this costs).
This changes neither the axis model nor the report bytes: there is no fast-mode flag,
output depth does not prune scanning, and filters, multiple views, watch mode, and every
unproved composition fall closed to the full index.
The natural text and all three machine-format summary goldens exercise the same command;
the performance harness additionally compares stable semantic hashes against the
pre-plan indexed binary.

### The Content Axis

`--analyze` is a **set of analyzers**, not a ladder of levels.
The shipped registry already models it that way — `content-basic-v1`, `code-sloc-v1`,
`text-logical-v1`, `markdown-prose-v1`, each versioned independently, with the exact set
recorded in the content sidecar’s provenance.
The original flag flattened that set into five ordered values, which could express only
four of the eight meaningful combinations and made `text-logical-v1` without
`markdown-prose-v1` — normalized word counts without the expensive Markdown projection —
unreachable.

| Value | Analyzers | Adds |
| --- | --- | --- |
| `none` | — | nothing; no file body is opened. The default |
| `lines` | `content-basic-v1` | physical, blank, and nonblank line counts |
| `code` | `code-sloc-v1` | standard SLOC — code, comment, blank — for the `code` family |
| `words` | `text-logical-v1`, `markdown-prose-v1` | raw, normalized, and reader-visible word volume |
| `all` | every registered analyzer | everything above |

`lines` is the base: any analyzer that runs has already streamed the file, so line
counts are free and implicit whenever the set is non-empty.
Naming it alone is still meaningful — it is the cheapest tier that opens files at all.
`all` is defined as *every registered analyzer*, so adding an analyzer extends it
without a grammar change, exactly as `--view full` extends with a new summary view.

Two consequences follow from modelling the set honestly rather than by rank:

- **Sidecar reuse becomes containment.** The current comparison is
  `record.profile == request.profile`, so a sidecar written by `all` forces a complete
  re-read for a later `code` query even though every metric it needs is already stored.
  The test becomes `stored ⊇ requested`, projecting the stored record down to the
  requested set. This is strictly faster and strictly more correct.
- **Word metrics stop being silently discarded.** `LogicalWordStats` is computed for
  every admitted text file and then zeroed for anything outside `prose`/`markup`. Under
  `words` the metric is retained wherever it was measured, including the `code` family,
  because the work is already done and normalized word volume is the cheap proxy for
  context-window sizing that agent consumers actually ask for.
  Aggregates stay meaningful because `families` and `types` keep the rows separate.

*Extension point, deliberately not shipped now.* Splitting `words` into its two
analyzers — normalized counts without the Markdown projection — is additive under this
grammar and needs no new axis, so it waits for a demonstrated need (Principle 1).

### The Default View Follows the Content Axis

A view may never enable an analyzer, because that would let a display choice authorize
filesystem reads. The reverse carries no such hazard, so the **default** view — and only
the default — is derived from the requested analyzer set:

| `--analyze` contains | Default `--view` | Why |
| --- | --- | --- |
| nothing | `tree` | unchanged; the du-replacement answer |
| `lines` only | `families` | the broadest grouping that shows line counts |
| `code` | `languages` | the view whose rows carry SLOC |
| `words` | `documents` | the view whose rows carry text volume |
| both `code` and `words` | `families` | the one view that shows both metric groups at once |

An explicit `--view` always wins; this table only supplies the default.
Without it, `fdu --analyze all PATH` reads every eligible file in the tree and prints a
directory tree containing none of the results — a report byte-identical to the one the
same command produces with no analysis at all, differing only in the performance footer.
That is the concrete defect this revision exists to fix.

### `--view full`

`full` expands to every summary view the requested analyzer set can answer — every view
except `files`, because an unbounded enumeration inside a digest destroys the digest.
It is not a synonym for `--analyze all`: `all` on the content axis means every analyzer,
while `full` names a curated report, and the different word marks the different
semantics. The view vocabulary plan records the rename from the original `--view all`.

Because `documents` requires analysis, `--view full` without it would otherwise fail the
whole run over one unsatisfiable view.
Instead the run renders what it can and **names what it skipped**, which keeps Principle
5’s honesty rule (never present an unmeasured value, never hide that something was
omitted) without making the obvious command an error:

```console
$ fdu --view full PATH
... nine view sections ...
note: omitted documents — requires content analysis: add --analyze lines, code, words, or all
```

Machine formats need no new field: the `reports` array already enumerates exactly which
views were produced, so a consumer reads the omission directly from what is absent.
The note is a human-format affordance only, and carries no schema change.

### Displaying What Was Paid For

The complement of the rule above.
When an explicit `--view` selects only views that ignore content analysis, the run has
bought I/O it cannot show:

```console
$ fdu --analyze all --view tree PATH
... tree ...
note: --analyze all read 1.2 GiB; no selected view displays content metrics
      — try --view families, languages, or full
```

This is a note, not an error, for one reason: warming the content sidecar so a later run
is warm is a legitimate use, and `--cache`-aware callers depend on it.
An error would break that; silence would hide a potentially enormous cost.

### Timestamps and Sync Watermarks

Every report carries two instants in every format: `scan_started_at` (when the walk or
revalidation began) and `generated_at` (when the report was rendered), as RFC 3339 UTC
with nanosecond precision; the text header prints the same “as of” instant.
Reports are therefore self-describing evidence: a summary is not just “the disk usage”
but “the disk usage as observed starting at T.”

Time selection uses one shared, fully specified grammar (`WHEN`), covering ages and
offset-bearing absolute times:

```text
WHEN      = "now" | AGE | TIMESTAMP
AGE       = 1*( INT UNIT )                ; that long before now: 45s, 2h, 7d, 1h30m
UNIT      = "s"|"sec"|"secs"|"second"|"seconds"
          | "m"|"min"|"mins"|"minute"|"minutes"
          | "h"|"hr"|"hrs"|"hour"|"hours"
          | "d"|"day"|"days" | "w"|"week"|"weeks"
TIMESTAMP = RFC3339                       ; 2026-08-10T18:22:31.482919114Z — exact
          | "@" INT [ "." FRAC ]         ; seconds since the Unix epoch, UTC
```

The rules that keep it well defined:

- The surface grammar deliberately follows fd’s `--changed-within`/`--changed-before`
  vocabulary for durations, RFC 3339, and `@epoch`, extended with compound ages
  (`1h30m`) and the `now` keyword, so existing muscle memory transfers.
- We borrow the grammar, not the implementation: `humantime`, the crate behind fd’s
  parser, is unmaintained (RUSTSEC-2025-0014, and fd has an open issue to replace it),
  so `parse_when` is a small first-party parser with table-driven tests.
  If scope ever outgrows it, the vetted fallback is `jiff`, not a revival of
  `humantime`; either way the *grammar above* is fdu’s contract, independent of the
  parser behind it.
- Ages subtract exactly from one `now` captured per invocation.
  Calendar units (months, years) are rejected with an error suggesting `30d`/`365d`:
  they require calendar arithmetic that approximates (humantime counts a month as 30.44
  days), and a grammar for file ages must not approximate.
  Fractions are rejected the same way (`1.5h` → “use 1h30m”); `@epoch` is the one place
  a fraction is meaningful and allowed, because that is its RFC precedent.
- Natural-language forms (`yesterday`, `2 weeks ago` à la GNU date and journalctl) are
  rejected: locale-dependent and unbounded; the closed grammar above is the whole
  language.
- Bare local dates and date-times are rejected with guidance to supply an RFC 3339
  offset. Resolving civil time correctly requires a time-zone policy and database; that
  decision remains tracked as `fdu-f6dn` rather than silently treating local input as
  UTC.
- RFC 3339 timestamps carry their own offset and round-trip a report’s `scan_started_at`
  exactly.

`--modified-since` is inclusive and `--modified-before` is exclusive, forming the
half-open window `[since, before)`.

Those two pieces compose into a reliable incremental-sync watermark with no new
machinery:

```text
fdu /data --view summary --format json   # record scan_started_at, back everything up
# ... later ...
fdu /data --view files --format jsonl \
    --modified-since 2026-08-10T18:22:31.482919114Z   # exactly what needs re-sync
```

The rules that make this trustworthy, stated so they hold under iteration:

- The watermark is the *previous scan’s start*, not its end: a file modified mid-scan
  may have been observed before the modification, and only the start bound is
  conservative.
- The boundary is inclusive, so a file whose mtime equals the watermark re-lists.
  For sync, duplicates are safe and omissions are not.
- The follow-up query answers from a revalidated index under the default `--cache auto`;
  under `--cache only` it answers from the snapshot alone and says so (Principle 5).
- mtime-window queries trust mtimes, exactly as find and rsync do; a backdated mtime is
  invisible to all of them.
  The live exact feed is `--watch` (clocked deltas, no mtime trust); a durable
  cross-restart journal that upgrades one-shot queries to exact deltas is future work
  owned by `fdu-3dtq`, and this schema leaves room for it rather than pre-building it.

### Cache Policy and Utilities

`--cache <POLICY>` replaces the single `--no-cache` boolean:

| Policy | Reads snapshot | Touches filesystem | Writes snapshot | Use |
| --- | --- | --- | --- | --- |
| `auto` (default) | when it is cheaper | the cheapest sound verification | on complete | fastest trustworthy answer |
| `refresh` | no | full scan | on complete | forced cold start; benchmark control |
| `read-only` | yes | revalidates | never | warm answer without touching the cache |
| `only` | yes | never | no | instant answer from data on hand, labeled `freshness: stale` |
| `off` | no | full scan | no | leave no trace |

Disabling the always-write behavior is therefore a policy value, not a separate flag:
`read-only` keeps the warm read path and suppresses the write, `off` suppresses both.

Every report carries `source` (`cold_scan`, `warm_revalidate`, `cache_only`),
`freshness`, and `complete`, in all formats, so no policy can silently lie.

**`auto` is a cost decision, not a habit.** Measurement settled this: on a 60k-entry
tree a parallel rescan costs 37 ms while loading the snapshot and verifying it costs 102
ms, so reading the cache is a *loss* at project scale — and for stat-tier queries the
full sweep is dominated by rescanning at every size, because it performs the same
enumeration and the same one-stat-per-entry and then adds a load.
At home-folder scale the reverse holds: the tree cannot fit the OS metadata cache
(`kern.maxvnodes` is ~263k on a 32 GiB Mac), every scan is effectively cold, and the
snapshot plus a journal resume is the only affordable answer.

What ships is a fixed rule rather than an estimate.
A one-shot report under `auto` or `read-only` reads the snapshot only when content
analysis is requested, because the sweep’s stats are what avoid re-reading unchanged
files; a metadata report scans cold, since revalidation stats every entry anyway
(`plan_report` in `crates/fdu-core/src/execution.rs`). `open()` under `auto` reads and
revalidates the snapshot, because its caller asked for a retained index.

The planned refinement is for `auto` to estimate before it acts, from the snapshot
header alone: entry count and the µs/entry that tree’s own last scan achieved, against
the platform’s metadata-cache capacity and the reducer tier the requested views need.
Small tree, stat-tier query: rescan and refresh the snapshot.
Large tree with a usable journal: load, replay, verify only what changed.
Content-tier query at any size: load and sweep.
The decision function, its self-calibrating cost model, and the derived replay budget
are specified in the
[FSEvents-scoped revalidation plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md)
and are not implemented (bead `fdu-6ld9`, open); `refresh`, `read-only`, `only`, and
`off` remain explicit overrides for anyone who wants a specific path rather than the
cheapest one.

**When the cache is written.** The policy axis decides *whether* a run may write; these
rules decide *what and when*, and they are rules, not heuristics (Principle 5):

- The core snapshot is written by `auto` and `refresh` only when the scan is complete
  and the index is `Fresh` (the existing invariant), and **on a background thread
  overlapped with rendering** — once producers finish, the index is read-only, so
  serialization and rendering are two concurrent readers.
  The save never delays first output; the process joins the save thread before exit so a
  write is never abandoned, and the save still completes when rendering ends early
  (broken pipe must not discard a finished scan’s work).
  A failed save (read-only cache dir, quota) is a stderr warning, never a changed exit
  code.
- It is written on every platform, for every tier of query, whenever complete —
  including pure stat roll-ups.
  The tempting refinement “stat-only runs skip the write” is rejected on the
  [performance frontier research](../../research/research-2026-08-10-performance-frontier.md):
  the write is tens of milliseconds off the hot path, while the stat-tier snapshot is
  exactly what the two decisive warm paths consume — on cloud runners the snapshot is
  the *only* possible warm state (the OS metadata cache does not fit in RAM), and on
  macOS the planned FSEvents journal resume (O(changes) warm opens) anchors on
  snapshot-carried resume tokens.
  What makes the cache feel unnecessary on a warm laptop today is the measured
  warm-costs-more-than-cold defect (the loop’s H9 family), which is owned by the
  performance loop, not by write policy.
- The snapshot format reserves the journal-resume fields (event ID, volume UUID,
  platform tag) now, per that research, so the macOS rung can land without a format
  break.

**Two cache layers.** Content-derived metrics (line counts and future analyzers — where
the user-visible payback is largest, minutes cold to seconds warm) live in a
**derived-data layer, not the core snapshot**: separate per-analyzer files keyed by
`(fingerprint, analyzer id, analyzer version)`, written whenever an analyzer ran under a
writing policy, loaded lazily, invalidated per analyzer without touching tree truth,
size-bounded, and purgeable via `--cache-clear`. The core snapshot stays small and fast
to open; runs that request richer roll-ups enrich the derived layer additively.
No analyzer ships under this plan, but the layer’s shape is fixed here so the content
tier arrives without a format break.

**Verification cost follows the query.** Under `auto`, “revalidates” means the cheapest
*sound* verification for the reducers the requested views actually use, per the frontier
research’s tier rule: name-tier questions (counts, tree shape) verify with one stat per
directory; any stat-tier metric (sizes, mtimes — every current default view) requires
one stat per entry, because in-place edits are invisible to directory fingerprints;
content-tier questions add re-reads of changed files only.
This is exact and needs no staleness label — view selection changes verification cost by
integer factors while staying trustworthy.
Reducers declare their tier when the reducer registry (`fdu-a6dz`) lands; until then all
shipped views are stat-tier and verification is the N-stat sweep.
A future *labeled* stale-sizes mode (the research’s H44) is possible but never a default
and not part of this plan.

The lifecycle flags are backed by new library functions rather than CLI-side directory
walking (Principle 7 — the CLI invents nothing):

- `cache_status(path) -> CacheStatus` — the file’s path, its size and its content
  sidecar’s, and a `state`, one of five: `current` (a snapshot this build serves, with
  its root and entry count), `stale` (fdu’s snapshot that this build cannot serve, with
  a reason — older format, newer format, other engine, or unreadable — and the format
  version when that is the reason), `leftover` (a staging temporary or an orphaned
  content sidecar fdu left behind), `unrecognized`, or `absent` (`CacheState` in
  `crates/fdu-core/src/cache.rs`; Python `CacheStatus.state`).
- `list_caches(dir) -> Vec<CacheStatus>` — enumerates the cache directory and reads each
  snapshot’s bounded header to recover the root path, fixing the earlier “opaque hash
  files with no reverse mapping” problem.
  This backs `--cache-status=all`; a file fdu cannot identify is listed as unrecognized.
- `clear_cache(path)` / `clear_all_caches(dir)` — idempotent, with no prompt and no
  `--force`. `--cache-clear` removes the root’s snapshot, current or stale, with its
  content sidecar, then names the cache file and reports `Cache cleared.` or
  `Cache already empty.` (flowmark’s pattern).
  `--cache-clear=all` echoes the directory, removes every current or stale snapshot, and
  also reclaims leftovers: staging temporaries older than 24 hours and content sidecars
  whose snapshot is gone.
  It reports snapshots and leftovers as two counts (`Cache cleared: N snapshots.` and
  `Also reclaimed: N files fdu left behind.`), and it never removes an unrecognized
  file, saying how many it left in place.

`--cache-status` renders through the same format axis (`--format json` works) as its own
`fdu.cache/1` document rather than as a report, so agents get cache observability in
every machine format.

### Watch Mode

Change detection is event-driven, never polling: the watcher binds the native OS backend
through `notify` (FSEvents on macOS, inotify on Linux, ReadDirectoryChangesW on
Windows), coalesces kernel-pushed events, and verifies each coalesced path with one
fresh stat — idle cost is zero filesystem work.
`--interval` below throttles only how often *aggregate views re-render*; it plays no
part in detection. Polling exists solely as the deliberate fallback for filesystems
without native events (NFS/FUSE/SMB), selected per-filesystem by the watch-hardening
work (`fdu-lka2`).

`--watch` runs the same query continuously (Principle 10):

1. Open the index per the cache policy and emit the initial report exactly as a one-shot
   run would.
2. Drive `watch::Watcher::apply_next` on the consuming thread.
   Each applied batch is filtered through the same `Selection`.
3. The `files` view streams per batch: one line (text) or one record (jsonl) per
   effective applied op — path, op (`upsert`/`remove`), kind, size, mtime, and the index
   clock. This is the `tail -f` surface, and the same selection window applies:
   `--modified-since 1h --watch` bounds the initial report, then streams everything
   after it, and `--modified-since now --watch` is a pure tail with an empty initial
   listing — no dedicated suppress-initial flag needed.
4. Aggregate views (`tree`, `types`, `summary`) re-render at most once per `--interval`
   (default 2s), only when dirty, separated in text by a timestamped header.
5. Overflow or subtree invalidation from the watcher appears as an explicit change
   record with op `invalidate` naming the invalidated path, whatever the selection, and
   aggregate views re-render as they do for any other change; it is never dropped
   (Principle 5).
6. SIGINT/SIGTERM exit 0 after a final snapshot save when the index is `Fresh` and
   policy allows writes; watch errors exit 1.

Streamed changes use the `fdu.stream/1` JSONL schema: each line is a tagged
`"record": "change"` carrying an `op` of `upsert`, `remove`, or `invalidate`
(`render_change` in `crates/fdu-core/src/report_format.rs`). The initial report and each
aggregate repaint are ordinary reports, and machine output for a report uses
`fdu.report/5`, or `fdu.report/6` when a metric summary or content analysis is present
(below). The `report` and `status` stream record types this plan first proposed were not
built. Constraint carried from the engine: watch requires full scope, so `--watch` with
`--scan-depth` or `--one-filesystem` is a usage error (exit 2) until
`validate_for_watch_scope` learns otherwise.

### Rust API

The new `query` module (feature-independent, usable by CLI, Python, and external
consumers alike):

```rust
pub struct Query {
    pub selection: Selection,      // include/exclude globs, min_size, kinds,
                                   // modified window (since/before), depth, limit,
                                   // sort, size metric
    pub views: Vec<ViewSpec>,      // Tree, Types, Files, Summary
}

pub fn report(index: &Index, query: &Query) -> Report;   // pure; never scans

pub enum CachePolicy { Auto, Refresh, ReadOnly, Only, Off }   // consumed by open()

pub struct Report { /* scan_started_at, generated_at, source, freshness,
                       complete, scope, one section per view */ }

pub fn parse_when(s: &str, now: SystemTime) -> Result<SystemTime>;  // "2h" | RFC 3339
pub fn parse_size(s: &str) -> Result<u64>;                          // "10M" | "1.5GiB"
```

The value grammars (`parse_when`, `parse_size`) are small first-party parsers in the
library, not CLI helpers and not new dependencies, so the CLI, Rust callers, and Python
all accept identical strings (Principle 7); `parse_when` takes `now` as an argument so
callers and tests control the reference instant.

`Report` and its sections remain dependency-light library values; deterministic,
hand-written text/JSON/JSONL/YAML serializers live in `fdu-core`’s ungated
`report_format` module, so the command line and the Python package render the same
bytes. `cli.rs` shrinks to parsing flags into `(ScanConfig, CachePolicy, Query, Format)`
and routing streams — the current private rendering methods on `Cli` move behind
`query`/`format` types with their own unit tests.
Watch composes the same pieces: a `Session` owning `IndexHandle` + `Watcher` yields
batches already filtered through the `Selection`, and the CLI loop is a thin consumer.

The derived summary planner is an execution strategy in `fdu-core`, not a second public
query API: `plan_report` stays private behind the public `prepare_report`, which the
command line and Python `fdu.report` both call.
It decides only what state a one-shot report retains; the public Rust
`report(index, query, provenance)` and Python `Index.report(...)` contracts remain
unchanged and pure.

The parity test for Principle 7 is mechanical: the CLI’s six axes map one-to-one onto
these library types, so any capability reachable by flags is reachable as one typed
call, with the same defaults.
If implementing a flag ever requires logic that does not fit `Query`, `AnalysisSet`,
`CachePolicy`, or a `Report` serializer, the library types are wrong and get fixed
first; each phase ends with an explicit review of what, if anything, lives only in
`cli.rs`. The content axis is the cautionary case: `--analyze` shipped without a
corresponding library-level set type, and the default-view derivation it needed had
nowhere to live, so neither surface got it — the parity review is what should have
caught that.

Supply-chain outcome: the serializers are small first-party writers over the closed
`Report` shape. This avoided adding `serde`, a JSON crate, or the unmaintained
`serde_yaml`, while keeping key order and number formatting byte-stable for goldens.

### Python API

Mirror, not wrapper-of-CLI. As shipped, the package is `fdu` and the axes are typed
values ([the release plan](plan-2026-08-14-fdu-release-packaging-python-api-polish.md)
owns the full API):

```python
import fdu

index = fdu.open(root, cache=fdu.CachePolicy.AUTO, scan=fdu.ScanOptions(max_depth=None))
r = index.report(
    fdu.Query(
        views=(fdu.View.TYPES, fdu.View.TREE),
        selection=fdu.Selection(include=("*.rs",), min_size="10M"),
    )
)
changed = index.report(
    fdu.Query(views=(fdu.View.FILES,), selection=fdu.Selection(modified_since="2h"))
)
resync = index.report(
    fdu.Query(
        views=(fdu.View.FILES,),
        selection=fdu.Selection(modified_since=r.scan_started_at),
    )
)
with index.watch(fdu.WatchOptions(interval=2.0)) as watch:  # batches of changes
    for batch in watch:
        ...
```

String values accept exactly the CLI grammars (`"2h"`, `"10M"`); native types
(`datetime`, `int`) are accepted wherever a string is.

`open`, `scan`, and `Index` keep their existing contracts; `report` and `watch` are
additive.
The wheel’s console `fdu` automatically gains the whole CLI surface through the
shared process boundary, as today.

### Schemas and Compatibility

- The report schema supersedes `fdu.tree/2`. This plan introduced it as `fdu.report/1`
  and explicitly authorized the schema replacement the CLI UX plan forbade; the golden
  fixture and schema-bump test moved with it.
  It is now `fdu.report/5`, or `fdu.report/6` when a metric summary or content analysis
  is present, and its top level carries `schema`, `generator`, `root` (with `root_raw`
  when the root is not UTF-8), `scan_started_at`, `generated_at`, `source`, `freshness`,
  `complete`, `errors`, `ignore_rules`, `analysis` under `/6` only, and `reports` (one
  entry per requested view, in request order), per `write_envelope_json` in
  `crates/fdu-core/src/report_format.rs`. The `cache`, `scope`, and `selection` fields
  this plan first listed are not in the envelope.
- The interface remains pre-release; no aliases for replaced flags.
- Library compatibility: existing `Index`, `scan`, `snapshot`, and `watch` contracts are
  preserved; `query` is additive, `ExtTally` gains a field (semver-minor while
  unpublished), and this plan makes no snapshot format changes — the v2 → v3 cursor
  section is owned by the
  [FSEvents-scoped revalidation plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md).
- Benchmark identity: `cli-human` and `cli-json` job definitions are re-pointed at the
  new argument vectors in the same change, and `cli-summary`, `cli-files`, and
  `watch-stream` become named jobs when their surfaces land (Principle 12).
- The help text, SKILL.md, and README currently drift as three hand-maintained copies of
  the contract; each phase updates all three, and consolidating them into one generated
  source is tracked as follow-up work, not assumed.

## Implementation Plan

### Phase 1: Query and Report Core

- [x] Add `query` module: `Selection`, `ViewSpec` (tree/types/files/summary), `Query`,
  `Report`, pure `report()` with unit tests per view × selection, including the roll-up
  fast path vs traversal tier
- [x] Implement the shared value grammars (`parse_when`, `parse_size`) and the
  `--modified-since`/`--modified-before` half-open window; stamp `scan_started_at` and
  `generated_at` on every report.
  Local date-times are rejected pending a time-zone decision (`fdu-f6dn`); the watermark
  round-trip is pinned by `crates/fdu/tests/scan_watermark_integration.rs` (`fdu-3vgt`,
  closed)
- [x] Add `allocated` to `ExtTally` and thread the size metric through all views
- [x] Implement dependency-light `Report` values plus hand-written `text`, `json`,
  `jsonl`, and `yaml` formatters; pin both `fdu.report/1` and `fdu.stream/1` with schema
  assertions and whole-record goldens (`fdu-rti1`, closed)
- [x] Rework CLI parsing to the five axes (view list parsing, replaced flags, exit
  contract), require an explicit report path with bare `fdu` as help, and update
  SKILL.md, `AFTER_HELP`, README, tryscript goldens, and the benchmark job manifests
  together
- [x] Python `Index.report(...)` with the same defaults and names
- [x] Derive a cache-off, one-view, unfiltered summary plan internally with indexed
  fallback and byte-identical golden/semantic-hash coverage (exp-040)

### Phase 2: Cache Policy and Utilities

- [x] `CachePolicy` in `open()` covering auto/refresh/read-only/only/off, with `only`
  failing closed when no usable snapshot exists
- [x] Snapshot write ordering and failure semantics: save on a background thread
  overlapped with rendering, only when complete and `Fresh`, joined before exit,
  completing even on broken-pipe rendering; a failed save warns on stderr without
  changing the exit code; `read-only` policy suppresses the write entirely.
  The journal-resume fields (event ID, volume UUID, platform tag) are reserved by the
  [FSEvents-scoped revalidation plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md)
  as snapshot format v3 (bead `fdu-2cdv`), not duplicated here
- [x] Document the two-layer cache design and the tier-derived verification contract in
  help, SKILL.md, and the schema docs (implementation of tiered verification lands with
  the reducer registry, cross-plan)
- [x] Library `cache_status`, `list_caches`, `clear_cache`, `clear_all_caches` with
  bounded header reads and never-delete-unrecognized semantics
- [x] `--cache-status[=root|all]` and `--cache-clear[=root|all]` lifecycle flags
  rendering through the format axis, running before scan validation; tryscript coverage
  per flowmark’s cache-behavior suite
- [x] Python `cache` accessors mirroring the library functions

### Phase 3: Watch Mode

- [x] `Session` API composing `IndexHandle`, `Watcher`, and `Query`; batch filtering
  through `Selection`
- [x] `--watch`/`--interval` CLI loop: initial report, streamed `files` records,
  dirty-gated aggregate re-render, explicit invalidation records, `fdu.stream/1` schema,
  and a persisting save.
  Delivered as a save after each dirty batch rather than a signal handler: std has no
  portable one, and a watch session ends by signal far more often than it ends politely,
  so an exit-time save would be the one that never runs.
  Pinned by `crates/fdu/tests/watch_persistence.rs`, which SIGKILLs the real binary
- [x] Deterministic goldens for streamed records through the bounded `watch-capture`
  helper (`fdu-t9nv`, closed)
- [x] Scope validation errors for `--watch` + `--scan-depth`/`--one-filesystem`
- [x] Python `Index.watch(...)` iterator with deterministic shutdown tests
- [x] `watch-stream` benchmark job registration — the job vocabulary only, which is what
  this item asked for; the runner is `fdu-g8ks`

### Phase 4: Design Principles Documentation

- [x] Distill the Goals and Design Principles of this spec — as actually implemented,
  with any amendments iteration forced — into
  [the design doc](../../architecture/fdu-design-principles.md), following
  common-doc-guidelines: the five axes, the CLI-invents-nothing parity rule, and the
  subsumption checklist.
  The doc already carried the engine principles, including the delta contract and cache
  honesty; the CLI-specific axes were first distilled on this branch and folded into it
  when the two histories merged
- [x] Run the end-of-plan parity review (what, if anything, lives only in `cli.rs`) and
  record its outcome in the design doc
- [x] Point AGENTS.md, README, and the architecture references at the design doc
- [x] Merge PR #5, reconcile the implemented surface, and leave remaining product
  decisions on the explicit follow-up beads below

### Phase 5: The Content Axis and the Display Contract

This revision. Ordered so each step is independently reviewable and the behavior change
lands last.

- [x] Replace `AnalysisProfile` with an `AnalysisSet` over the existing analyzer
  registry: `includes_code()`/`includes_documents()` become real membership tests,
  `--analyze` parses a comma-delimited list through the same `parse_list` helper as
  `--view` and `--kind`, and `none`/`all` are the two totals.
  Rename `basic` → `lines` and `documents` → `words` in the same change; the interface
  is pre-release and no aliases are retained, per the precedent set by `--by-type` →
  `--view types`
- [x] Encode the set as a bitmask in the content sidecar and change record reuse from
  profile equality to `stored ⊇ requested`, projecting the stored record down to the
  requested set. Add a unit test that an `all` sidecar satisfies a `code` request with
  zero fresh reads — the regression this replaces is a silent full re-read
- [x] Retain word metrics wherever they were measured rather than zeroing them outside
  `prose`/`markup`, so `words` reports normalized volume for the `code` family too
- [x] Derive the default view from the analyzer set; an explicit `--view` always wins.
  This changes output for every existing `--analyze` invocation, so the goldens are
  re-recorded and the diff reviewed as the record of the change
- [x] Add `--view all` with profile-aware expansion and the omission note; confirm the
  machine formats need no schema change because `reports` already enumerates what was
  produced
- [x] Add the paid-for-nothing note when no selected view consumes the requested
  analysis, and a test that it is a note and never an error, because sidecar warming is
  a supported use
- [x] Update SKILL.md, `AFTER_HELP`, README (the Five Common Reports table), the
  `--analyze` help text — which must state plainly that it opens and reads eligible
  files — and the benchmark job manifests together, per Principle 12
- [x] Fold Principle 13 and the six-axis model into
  [the design doc](../../architecture/fdu-design-principles.md)

## Testing Strategy

- Unit tests per view over a fixed synthetic index, crossed with selection filters, both
  size metrics, and both performance tiers; property test that adding a view never
  changes another view’s section.
- Golden tryscript sessions per axis: view lists, each format, cache policies (using a
  scratch `XDG_CACHE_HOME`), cache utilities, and watch streaming with injected changes;
  stable fields are byte-exact, while performance duration and throughput values use
  named patterns that keep every field visible.
  The watch stream uses a bounded, causally sequenced capture helper rather than timing
  a process that never exits.
- One concise, realistic nested-project session pins the complete natural human report
  at its default depth and limit.
  Its rows jointly cover size ranking, hierarchy, alignment, fixed ten-cell bars,
  rolled-up descendants below the display depth, and no spurious omission marker.
  Focused sessions retain the actual limit-marker boundary and other combinatorial edges
  instead of inflating this product example.
- Schema tests: report (`fdu.report/1` when this plan landed, `/5` and `/6` now) and
  `fdu.stream/1` fixtures that fail on unversioned change.
  `--view full` and the analyzer-set rename must *not* bump either schema; a test pins
  that the `reports` array alone communicates which views were produced.
- Content-axis tests: every analyzer set round-trips through the sidecar bitmask; an
  `all` sidecar satisfies a `code` request with zero fresh reads (containment, not
  equality); the default view derived from each set matches the table, and an explicit
  `--view` overrides every one of them.
- Display-contract tests, one per direction of Principle 13: a run whose selected views
  all ignore analysis emits the paid-for-nothing note and still exits 0; `--view full`
  without analysis renders the satisfiable views, names the omitted one, and exits 0;
  and no view, under any content setting, causes a file body to be opened that
  `--analyze` did not authorize.
- Time-window tests: table-driven `parse_when`/`parse_size` grammar units with injected
  `now`, covering every accepted form (`now`, compound ages, RFC 3339, `@epoch` with
  fraction) and every rejection with its suggestion (months/years → days, fractional
  ages → compounds, bare local date-times → offset-bearing RFC 3339, natural language);
  boundary inclusivity at exact-equal mtimes; a watermark round-trip proving a report’s
  `scan_started_at` fed back as `--modified-since` lists exactly the files touched after
  scan start, including one touched mid-scan; timestamp fields are normalized in
  goldens.
- The existing partial-result, non-UTF-8 identity, broken-pipe, and stack-depth process
  tests are retargeted, not weakened; deep-tree rendering stays iterative.
- `make check` remains the handoff gate, including `--no-default-features` (the `query`
  module must build without the CLI feature).

## Rollout Plan

Implementation proceeded phase by phase on one feature branch, with each phase leaving
the CLI working and documented.
Phase 1 carried the breaking rename so churn on SKILL.md, goldens, and benchmark
manifests happened once.
Phase 4 is small but not optional: the principles live in `docs/project/architecture/`
so they govern future work, not just this plan.
No publishing; `fdu-9cf0` gates remain.

## Remaining work

The four implementation phases are complete.
Post-merge integration was reproduced against the performance branch rather than
assumed: exp-033 exercised all five engine jobs with exact oracles, and exp-035 repeated
the cold path on a heterogeneous 1M-entry workspace.
The concise realistic `cli-overview` tryscript fixture remains the human-output contract
while focused goldens own boundary cases.
No performance experiment changed CLI text, query semantics, or the mandatory-root/help
behavior. The remaining product decisions and follow-ups are mapped to beads so they
cannot be lost by being described only here:

| Gap | Bead |
| --- | --- |
| `watch-stream` benchmark **runner** (only the job vocabulary is registered) | `fdu-g8ks` |
| Local date-times in `parse_when`, pending a time-zone decision | `fdu-f6dn` |
| Cache retention: nothing prunes snapshots or bounds total size (open question 5) | `fdu-558j` |
| Open questions 1, 2, and 4 | `fdu-khu8` |
| Automate the runbook’s bead-sync check as a periodic guard | `fdu-qut8` |

The watch stream is goldened (`fdu-t9nv`, closed): tryscript compares one command’s
completed output and a watch process never exits, so
`tests/golden/bin/watch-capture.mjs` turns watching into a command that does — it spawns
`fdu --watch`, applies a scripted change sequence, waits for each change’s own record
before making the next, and prints the captured stream.
Determinism is causal, not timed.
`tests/golden/cli-watch.tryscript.md` pins the schema on every record, the op
vocabulary, per-op field presence, and the absent-not-null contract for removals.
Alongside it: `crates/fdu/tests/watch_session_integration.rs` for event semantics,
`watch_persistence.rs` for save-surviving-SIGKILL (cold and warm start), and section 6
of [the integration runbook](../../guides/integration-runbook.md) for the one property
no automated test asserts well: that an idle tree costs 0% CPU.

## Open Questions

1. Short flag for `--view` (`-v` collides with the verbose convention; no short flag is
   proposed initially).
2. Multiple roots per invocation (fd/find allow several): one index per root is easy to
   compose in the library; the CLI ergonomics and cache story are not designed here.
3. Disposition of `fdu-oqoy` (adaptive terminal width, gitignore display) and `fdu-jej9`
   (JSONL, schema docs): this plan subsumes their JSONL/sorting/summary scope, and
   gitignore tagging shipped with PR #65 (each row’s ignored share, `--exclude-ignored`,
   `--only-ignored`, `--no-gitignore`); the remainder (adaptive width) likely re-homes
   under this epic — needs maintainer sign-off before closing or re-parenting either
   bead.
4. Whether a general `--group-by` ever surfaces once the reducer registry lands
   (generalizing `types`), or named views remain the entire vocabulary and new groupings
   arrive only as new views.
   Examined again during this revision and still not adopted: `--by language`,
   `--by type`, `--by family` reads well and would collapse four views into one axis
   value, but `files`, `summary`, and `documents` are not groupings, so the axis would
   fracture rather than generalize.
   Revisit only if the non-grouping views find another home.
5. Cache retention: nothing yet prunes snapshots for roots that are never queried again
   or bounds the derived-data layer’s total size (`fdu-558j`). Age-based GC, size caps,
   or manual-only (`--cache-clear`) needs a decision before the derived layer ships.
   Measured on a development machine 2026-08-20: 63 MB across 52 entries, of which 26
   were unreadable by the current binary — pre-release format churn, handled correctly
   as absent, but reclaimed by nothing.
   PR #67 answered the dead-entry half: `--cache-status` lists a snapshot another fdu
   version wrote as `stale`, and `--cache-clear` removes stale snapshots and reclaims
   the leftovers fdu wrote.
   What remains open is retention and a size bound for roots that are still readable but
   never queried again.
6. Whether `--analyze` should expose `text-logical-v1` and `markdown-prose-v1`
   separately rather than jointly as `words`. The registry and the sidecar already
   support it and the grammar makes it additive; deferred under Principle 1 until
   someone needs normalized counts without the Markdown projection.

Resolved by composition rather than by new surface, recorded so they stay resolved:
suppressing watch’s initial report is `--modified-since now --watch`. Top-N largest and
recent listings were once on this list as `files` plus `--sort`/`--limit`; they are now
the `largest` and `recent` presets, for the reason given under Views.

## References

- [Phase 1 plan](plan-2026-08-08-fdu-phase-1.md)
- [CLI UX and agent skill plan](plan-2026-08-09-fdu-cli-ux-and-agent-skill.md)
- [Rollup engine research](../../research/research-2026-08-06-file-rollup-engine.md)
  (Goals 1–7; delta contract; tag-don’t-prune)
- [End-to-end performance evidence research](../../research/research-2026-08-09-end-to-end-performance-evidence.md)
  (benchmark job identity; time-to-first-output vs time-to-complete)
- [Performance frontier research](../../research/research-2026-08-10-performance-frontier.md)
  (verification tiers by reducer; two cache layers; snapshot write economics; journal
  resume fields; the composability rule that scope/view/format never select engine
  variants)
- flowmark-rs cache surface: `attic/flowmark-rs/src/incremental_cache.rs`,
  `src/settings.rs`, `docs/cache.md`, `tests/tryscript/cache-behavior.tryscript.md`
- WHEN grammar prior art: [fd man page](https://www.mankier.com/1/fd)
  (`--changed-within`/`--changed-before` formats),
  [RUSTSEC-2025-0014](https://rustsec.org/advisories/) (`humantime` unmaintained),
  [fd issue #1689](https://github.com/sharkdp/fd/issues/1689) (fd replacing `humantime`)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
