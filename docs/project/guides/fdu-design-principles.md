# fdu Design Principles

The rules the command line, the query layer, and the library hold themselves to, as
actually implemented.
A change that violates one of these needs this document amended first, not a silent
exception.

Distilled from
[the composable CLI and query surface plan](../specs/active/plan-2026-08-10-fdu-composable-cli-surface.md)
after building it. Where implementation forced an amendment, the amendment is recorded
here rather than the original intent.

## The Governing Aspiration

The design should fit the contours of the real problem: no more complexity, but no less
either. Simple things stay simple — `fdu` alone is a good answer — and complex things
stay possible, because any axis composes with any other.

The concrete test for “no more complexity”: before adding a view or a flag, show it
cannot be expressed as a composition of what exists.
`largest` and `recent` were removed from the design by exactly that test; they are
`--view files --sort size --limit N` and `--view files --modified-since 2h`.

## 1. Five Axes, No One-Off Flags

Every option belongs to exactly one axis:

| Axis | Question | Options |
| --- | --- | --- |
| Scope | What is scanned and cached? | `PATH`, `--scan-depth` |
| Selection | Which retained entries does this query consider, and how are results shaped? | `--include`, `--exclude`, `--min-size`, `--modified-since`, `--modified-before`, `--kind`, `--depth`, `--limit`, `--sort`, `--reverse`, `--size` |
| View | Which roll-up is reported? | `--view tree,types,files,summary` |
| Format | How is it serialized? | `--format`, `--color` |
| Mode | One answer or a live feed, and how is the cache used? | `--watch`, `--interval`, `--cache`, `--allow-partial` |

A proposed flag that fits no axis is a design smell: either it generalizes into an axis
value, or it does not ship.

**Scope versus selection is the load-bearing distinction.** Scope decides what is
observed and cached, so one snapshot serves every query; selection filters the retained
index at view time and is never part of the cache key.
That is why narrowing a filter never costs a rescan, and it is the same reasoning as
tagging ignored entries rather than pruning them.

## 2. Intuitive by Default, Everything by Composition

There are no subcommands.
The grammar is always “report on a path”, so a path argument can never be shadowed by a
verb. `--help` documents each axis, its values, and its defaults plainly enough that the
design is legible from the help text alone.

## 3. One Scan, Many Views

Views are projections over the in-memory index.
Requesting more views never adds filesystem work, and two reports over the same tree
come from one consistent state — a property with its own test, because “one scan” would
otherwise be an aspiration rather than a guarantee.

Two performance tiers follow from this, and both are milliseconds warm: an unfiltered
request reads pre-computed roll-up state directly, while any selection filter triggers
one traversal that re-aggregates what it admits.
One traversal serves every filtered view in a request.
A test pins that the two tiers answer identically when the filter admits everything.

## 4. Views Are Readers

The delta contract stands: `scan` and `watch` produce observations, the index consumes
them, and views only read.
`report()` is a pure function of an index, a query, and provenance — no filesystem
access, no mutation, and the same inputs always produce the same report.

*Amended during implementation.* The plan sketched `report(index, query)`. Provenance is
a third argument because `generated_at` cannot be sampled inside a pure function; making
it an input is what keeps the goldens meaningful.

## 5. Fastest Answer the Data Allows, Never Silently Stale

Cache behavior is one explicit policy axis, and every report labels its `source`,
`freshness`, `complete`, and `errors` in every format.
Warm, cold, and cache-only runs are user choices rather than heuristics.

`--cache only` is the one tier that can be stale, and it says so: the loaded index is
marked unverified rather than replaying the freshness it was saved with.
It fails when no usable snapshot exists rather than silently scanning, because a fast
path that is sometimes a full walk — with nothing in the output to say which happened —
is worse than no fast path.

See [the cache design](cache-design.md) for the two layers and what verification costs.

## 6. Same Concepts at Every Level; the CLI Invents Nothing

`Query`, `Selection`, `ViewSpec`, `Report`, and `CachePolicy` are typed values in the
library. The CLI parses flags into them, renders what comes back, and does nothing else.
Python exposes the same types through the same value grammars.

The parity rule is mechanical: a capability reachable by flag must be reachable as one
typed call, with the same defaults.
A capability that exists in one surface and not the others is unfinished, and complexity
that exists only at the CLI layer is misplaced.

What legitimately lives only in `cli.rs`: flag parsing, terminal and colour decisions,
exit-code mapping, and the human text layout.
Everything else — value grammars, selection semantics, view construction, cache policy,
session coordination — is library code.

## 7. Subsume the Neighbours

Each of these must be one invocation:

| Instead of | Run |
| --- | --- |
| `dust`, `dut` | `fdu` |
| `du -sh`, `diskus` | `fdu --view summary` |
| `du -a --max-depth 3` | `fdu --depth 3 -n all` |
| `fd -e rs`, `find -name` | `fdu --view files --include '*.rs'` |
| biggest files | `fdu --view files --sort size -n 100` |
| `find -mmin -60` | `fdu --view files --modified-since 1h` |
| `du` by type | `fdu --view types` |
| two reports, one scan | `fdu --view types,tree` |
| `tail -f` for a tree | `fdu --watch --view files --format jsonl` |

An interactive TUI is a recorded non-goal, not an omission: it would be a consumer of
the same `Query`/`Report` layer.

## 8. Formats Are Serializations, Not Features

Every view renders in every format.
Machine formats are schema-versioned, never colourized, and a schema change without a
version bump fails a golden test.

This principle inverted a rule that used to exist: `--by-type` conflicted with `--json`,
because the type breakdown was human-only.
Under the axis design that combination is not merely legal but required to work.

## 9. Watch Is the Same Query, Repeated

A watch run evaluates the same selection and views as a one-shot run, re-applied as
changes arrive. There is no separate watch grammar to learn.

Detection is event-driven — the OS notification backend, never polling — so an idle tree
costs no filesystem work, a property asserted by test rather than described.
`--interval` throttles only how often aggregate views repaint; it plays no part in
detection. Overflow and subtree invalidation appear explicitly in the stream and are
never dropped, because they say the consumer’s own view may have gaps.

Two deliberate asymmetries in filtering: a removal is filtered only by path, since
filtering a deletion on a size bound would hide the disappearance of something the
caller was watching; and an escalation is never filtered at all.

## 10. Utilities Are Explicit Flags, Never Side Effects

`--cache-status` and `--cache-clear` run before scan validation, need no readable tree,
and suppress the report.
A report run never deletes anything.
Clearing echoes its target before acting, and never removes a file this build cannot
identify.

## 11. No Unbenchmarked Performance Claims

Each output surface becomes a named benchmark job, and flags are part of benchmark
identity: renaming one means updating the job manifests in the same change.

## Testing Discipline

Golden tests are the text contract, and their value depends on one habit: **classify
every field as stable or unstable.** Paths inside the fixture, byte counts, entry
counts, kinds, and schema strings are stable and must match exactly.
Sandbox paths, timestamps, allocated sizes, and inode-derived values are unstable and
get a *named pattern* — never a bare line elision, which would hide the field instead of
freeing its value.

Re-recording is normal; **reading the diff is the point.** In this workstream the
goldens caught four defects no unit test did, including JSON that was balanced and
invalid because the fixture had no directory with two children.

Two hazards worth remembering, both found the hard way:

- Comparing roll-ups by raw interned extension id fails across walks.
  Ids are assigned in first-seen order, so serial and parallel runs assign them
  differently. Compare through `by_ext_named()`.
- Deep trees must be handled iteratively everywhere — expansion, all three renderers,
  and `Drop`. Derived drop glue recurses per level, so a deep tree overflowed the stack
  on release even after rendering was fixed.

* * *

*Part of the fdu project documentation.
See [AGENTS.md](../../../AGENTS.md).*
