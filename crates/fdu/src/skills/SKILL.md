---
name: fdu
description: >-
  Inspect directory trees with hierarchical file counts, apparent and allocated sizes,
  recency, and extension tallies. Use when investigating disk usage, finding large
  directories, summarizing file types, listing files by size or age, or collecting stable
  JSON filesystem roll-ups for scripts and coding agents.
---
# fdu Directory Roll-Ups

Use `fdu` to summarize a directory tree without modifying files in that tree.
`fdu --docs` prints the full usage guide -- the report ladder, both axes, and the output
contracts -- without a PATH and without scanning.
Every report requires an explicit `PATH`; bare `fdu` prints help instead of scanning the
current directory.

## Run fdu

Start with the report that answers the question:

```bash
fdu --view languages PATH                 # detected language sizes; metadata only
fdu --analyze code --view languages PATH  # add standard LOC; reads content
fdu --view types PATH                     # all file types; metadata only
fdu PATH                                  # folder-size tree; metadata only
fdu --view summary PATH                   # one totals row; no retained index
```

`--analyze` chooses what may be read and `--view` chooses what is printed.
The two language commands differ only on the analysis axis: the first uses byte shares
without content reads, while the second adds code, comment, and blank-line metrics.
Use `--size apparent` when logical file lengths are wanted instead of allocated bytes.

For a bounded machine-readable tree, use:

```bash
fdu --format json --view tree --depth 2 --limit 20 PATH
```

If no local command exists and this release is published on PyPI, use the exact reviewed
version. Never use an unversioned `uvx` runner or `latest` in agent instructions:

```bash
uvx --from fdu==__FDU_VERSION__ fdu --format json --view tree PATH
```

## Compose the Request From Five Axes

Every option belongs to exactly one axis, and any axis composes with any other.
There are no subcommands: the grammar is always “report on a path”.

| Axis | Question | Options |
| --- | --- | --- |
| Scope | What is scanned and cached? | `PATH`, `--scan-depth N`, `--order`, `--threads`, `--type-rules FILE`, `--tag-rules LIST`, `--promote LIST`, `--hidden`, `--hidden-allow LIST` |
| Selection | Which entries does this query consider? | `--include`, `--exclude`, `--min-size`, `--modified-since`, `--modified-before`, `--kind`, `--tag`, `--not-tag`, `--plane TAG`, `--depth`, `-n/--limit`, `--sort`, `--reverse`, `--size` |
| View | Which roll-up is reported? | `--view tree,groups,families,types,extensions,languages,documents,files,summary` |
| Format | How is it serialized? | `--format text\|json\|jsonl\|yaml`, `--color` |
| Mode | How is work performed? | `--cache auto\|refresh\|read-only\|only\|off`, `--analyze none\|basic\|code\|documents\|full` |

Scope versus selection is the distinction that matters: scope decides what is scanned
and cached, so one cache serves every query, while selection filters the retained index
at query time. Narrowing a selection never costs a rescan.

`--type-rules FILE` is scope for the same reason: a `[[kind]]` manifest decides what
every type and group row *means*, so a snapshot taken under one file is not answerable
under another and the cache invalidates accordingly.
Without it, fdu classifies against its compiled taxonomy, which is also the fast path —
there is no file to find and no startup parse.

`--tag-rules LIST` is scope for the same reason, and `--tag`/`--not-tag` are selection.
A tag is one named boolean fact recorded on an entry — `dotfile`, `vendored`,
`documentation` and `gitignore` are the rules that ship — so an index built without a
rule carries no bit for it and genuinely cannot answer.
Filtering on a rule that is not enabled is refused rather than answered with nothing,
because a filter that matches nothing is indistinguishable from no filter at all.
A tag rides on the entry itself, never on its ancestors: `--not-tag dotfile` drops
`.git` and keeps what is inside it, which is what separates a tag from scope pruning.
`gitignore` reads the tree’s `.gitignore` files with git’s own precedence, so a nested
`!keep.log` beats a broader `*.log` above it.

`vendored` and `documentation` are the same facts a row’s `classification.flags` already
reports, and now one predicate decides both: a caller filtering with
`--not-tag vendored` and a row saying `vendored: true` cannot disagree about a file,
because there is no second copy of the rule to drift.
`generated` is not among them, and the tier check is the reason rather than an oversight
— it reads the file’s opening bytes, so enabling it as a tag would turn a metadata walk
into a content walk.
It stays on the classification of a file whose bytes were read anyway, which is the only
place it is free.

`--promote LIST` is the third tag flag, and the one that costs.
Promoting a rule makes every directory maintain a second set of totals beside its own:
the same files, dirs, bytes and per-extension tallies, restricted to the entries that do
*not* carry the tag.
`--plane TAG` then answers in that set — `--promote gitignore --plane gitignore` reports
the tree as a browser would show it, skipping what git ignores.

Promotion is scope and the plane is selection, which is the same line `--tag-rules` and
`--tag` hold, and it is worth being precise about why.
Enabling a rule is a branch per insert; promoting one multiplies the ancestor-merge path
on every mutation, whether or not anyone ever asks for the plane.
That cost is what buys the read: `--plane` is a roll-up lookup rather than a walk, so it
narrows the answer without invalidating the cache, while `--not-tag gitignore` gives the
same numbers by re-aggregating the whole index.
Naming a plane that was not promoted is refused, listing what was — a plane served from
the totals would look right on exactly the trees that cannot tell the difference.

`--hidden prune` is the strongest scope flag fdu has, and the axis test again from the
other side. `--not-tag dotfile` leaves both numbers visible and still walks `.git` in
order to report it; pruning decides what is scanned at all, so a hidden entry has no
row, no tally, and its subtree is never read.
That is why visibility is not a tag: a maintained plane for hidden entries would have to
walk the caches and virtualenvs it exists to exclude.
`--hidden-allow LIST` names exact entries to admit anyway — `.github`, `.cargo` — and is
refused where nothing is being pruned, because it can only have been written by someone
who believed it was.
Governing control files stay readable without being retained, so a `.gitignore` still
decides what `gitignore` tags even where the file itself is outside the index.

fdu counts what is there by default.
A `du` replacement that quietly omitted half a working tree would be answering a
question nobody asked, so pruning is opt-in and, like every scope rule, fingerprinted
into snapshot identity.

Cost has three layers.
A single unfiltered `--view summary PATH` is the one exact composition that retains only
aggregate tallies and no index, under every cache policy except `only` and `refresh`,
whose contracts are about the snapshot itself.
Under the rest a snapshot cannot save the walk that request is already doing, so it
neither reads nor writes one.
Ordinary metadata requests retain the reusable index but never read regular-file
contents. Any non-`none` `--analyze` profile opts into streaming reads through every
eligible file and a separate profile-scoped sidecar.
A repeated run with the same profile and semantic settings reuses unchanged content
records. Coverage is profile-scoped too: an unsupported deeper analyzer leaves byte
metadata visible but does not retain a separate lower-level metric record for that file.

## Pick the View, Then Shape It

- `--view tree` (default) for per-directory roll-ups.
- `--view extensions` for the original raw-extension breakdown.
  Rows partition the tree and so sum to its total; a derived extension always carries a
  leading dot, and names having none are tallied under the literal `(none)`.
- `--view types` for stable detected file types and exact byte shares.
- `--view families` for code, prose, markup, data, binary, and unknown roll-ups.
  This is the analysis axis: it says which analyzer may open a file, so every image,
  video, PDF, and archive is `binary`.
- `--view groups` for the browsing axis the active rule registry declares — where a
  reader would look for a file rather than which analyzer may open it.
  Rows carry a stable `id` and a display `label`; a registry declaring no `[[group]]`
  reports none.
- `--view languages` for code-family rows and byte shares from path-only detection.
- `--view documents` for prose metrics; it requires any enabled analysis profile.
- `--view files` for a flat listing.
  One-shot text adds the performance footer described below; use a machine format when
  output is consumed programmatically.
- `--view summary` for one aggregate row.
- Several views in one run share one scan: `--view summary,types,families`. Text then
  labels each block with an all-caps header naming its view; a single-view text report
  has no header. Machine formats tag every report with `view` either way.

`--analyze` names a set of analyzers, comma-separated, from `lines`, `code`, and
`words`; `none` and `all` are totals and cannot be combined with anything else.
Anything but `none` opens and reads every eligible file, which is the only setting that
makes a run cost more than one metadata walk.

Add `--analyze lines` to stream physical, blank, and nonblank lines and raw word counts.
Add `--analyze code` for standard LOC, comment, and code-blank partitions across
supported common languages; the percentage column then uses code lines instead of bytes.
Use `--analyze words` for normalized word volume, paragraphs, aggregate-derived pages,
and reader-visible Markdown that excludes destinations and code.
`--analyze code,words` — or `all` — computes both in one streaming pass.

Requesting analysis without naming a view selects one that displays it: `code` reports
`languages`, `words` reports `documents`, and either both or `lines` alone reports
`families`. Naming `--view` overrides that; a view never enables an analyzer, so a
`--view` that displays no content metric prints a note saying what was read for nothing.
`--view all` reports every view the requested analyzers can answer and names any it
skipped. Use `--analysis-workers` to bound concurrent reads and `--words-per-page` to
control page derivation.
Analysis never truncates a file or excludes it because of size.
Invalid UTF-8, binary data, and unsupported SLOC languages remain visible as normal
coverage outcomes. Only I/O failures, files changed during a read, or stale commits make
analysis operationally partial.
Content analysis is currently one-shot and cannot be combined with `--watch`.

One-shot text reports end with a compact performance line.
It reports regular files and apparent bytes walked, content bytes actually read,
fresh-analysis file and byte rates, content-sidecar files and apparent bytes restored
from cache, the metadata cache tier, and total report time.
Known binary files can contribute walked bytes but zero read bytes.
Cache-only runs report zero walked files because they never consult the tree.
The line is gray only when color is active and has no ANSI escapes otherwise.
JSON, JSONL, YAML, skill output, lifecycle output, and watch streams omit it.

Common shapes are compositions rather than dedicated flags:

```bash
fdu --view files --sort size --limit 20 PATH          # largest files
fdu --view files --modified-since 2h PATH             # changed in the last two hours
fdu --view files --include '*.{rs,toml}' PATH         # by pattern
fdu --view tree --sort mtime PATH                     # an activity map
```

`--depth` and `--limit` bound only the rendered view; `--scan-depth` bounds what is
scanned and retained, so do not reach for it merely to shorten output.

## Value Grammars

- Sizes: `512`, `10k`, `10M`, `1.5GiB`. Decimal and binary units, case-insensitive.
- Times: `now`, a compound age (`45s`, `2h`, `1h30m`), an RFC 3339 timestamp with an
  offset (`2026-08-10T18:22:31Z`), or `@` epoch seconds.
  Calendar units and fractional ages are rejected with the spelling to use instead; a
  bare local date-time is rejected because resolving it needs a time-zone database.
- `--modified-since` is inclusive and `--modified-before` is exclusive.

## Use Timestamps as a Sync Watermark

Every report carries `scan_started_at`. Feeding it back selects exactly what changed
after that scan began, which is what makes incremental follow-up sound:

```bash
fdu --view summary --format json PATH                       # record scan_started_at
fdu --view files --format jsonl --modified-since <that> PATH
```

Use the scan’s *start*, not its end: a file modified mid-scan may have been observed
before the modification, so only the start bound is conservative.

## Validate Every Automated Result

Check the process exit status and these fields:

- `schema` before parsing anything else: a metadata-only report carries `fdu.report/4`,
  a report that ran content analysis carries `fdu.report/5`, and a `--watch` stream
  carries `fdu.stream/1`. Treat an unrecognized value as a version you cannot parse
  rather than guessing at the fields.
- `complete` and `errors` before trusting totals
- `freshness` and `source` before presenting data as current
- `truncated` on a tree node before treating it as exhaustive, and `remainder` for what
  it withheld: `rows`, `files`, `dirs`, `bytes`, and `allocated` for the child rows not
  emitted, or `null` when none were.
  Emitted children plus `remainder` account for every directory beneath the node, which
  is what makes an “other” row addable without a second query.
- `coverage` before presenting a metric summary as complete
- `detection.sources`, `detection.confidence`, and `detection.flags` before treating a
  deep-detected type or origin label as exact

`source` is `cold_scan`, `warm_revalidate`, or `cache_only`. Only `--cache only` can
return `freshness: stale`, and it says so rather than implying currency; it fails
outright when no usable snapshot exists rather than silently scanning.

Exit 0 is accepted success, exit 1 is a fatal failure, and exit 2 is incomplete data or
invalid usage. Do not discard useful stdout from exit 2; inspect the completeness fields
and use `--allow-partial` only when incomplete totals are acceptable.

## Cache Behavior

The snapshot is one file per root under the user cache directory.
`--cache-status` maps a hash-named file back to the tree it describes, and
`--cache-clear` removes it; both run without scanning and never touch files this build
cannot identify.

Verification cost follows the question asked.
Sizes and timestamps need one stat per entry, because an in-place edit changes a file
without changing any directory.
Questions answerable from names alone need only one stat per directory.
Adding metrics within a tier is free; crossing a tier boundary is what costs.

Exact names and ordinary extensions remain path-only classifications.
When analysis is enabled, unresolved files and ambiguous `.h` headers may use bounded
shebang, modeline, literal, or signature probes.
Do not collapse their provenance into an unqualified language claim; retain the report’s
source and confidence fields when summarizing or transforming machine output.

Run `fdu --help` for the complete flag, cache, color, scope, and exit contract.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
