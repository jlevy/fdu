# Project Instructions for AI Agents

This file provides instructions and context for AI coding agents working on this
project.

<!-- BEGIN TBD INTEGRATION format=f06 surface=agents-md -->
## tbd

This repository uses **tbd** for git-native issue tracking (beads), spec-driven
planning, and on-demand engineering guidelines.
As the agent, you operate tbd on the user’s behalf: translate their requests into tbd
actions rather than telling them to run commands.

- Run `tbd prime` to load current project state and the full tbd workflow.
- Run `tbd skill` for the complete reusable tbd skill instructions.
- Run `tbd shortcut --list` and `tbd guidelines --list` for on-demand resources.
- Track all work as beads: `tbd create`, `tbd ready`, `tbd close`, and `tbd sync`.

<!-- END TBD INTEGRATION -->

## Build and Test

```shell
make build      # debug build, all features
make test       # test suite
make check      # handoff gate: fmt, clippy, tests, docs, lib-only build
make fix        # apply formatting and machine-applicable lint fixes
make audit      # cargo-deny advisory and license audit
```

`make check` is the required handoff gate.
If it passes, CI should.
It runs the same feature combinations CI does — notably `--no-default-features`, which
is how library consumers build and is otherwise never exercised locally.

## Performance work

Speed changes are decided by measurement, never by argument.
The loop, the metrics, and the rule that decides whether a change is kept are in
[docs/project/guides/performance-loop.md](docs/project/guides/performance-loop.md);
every experiment and its verdict is in
[the experiment ledger](docs/project/reports/report-2026-08-10-fdu-performance-experiments.md).

- Profile before changing anything.
  Intuition about where a walker spends its time is reliably wrong, and the ledger has
  the rejected experiments to prove it.
- Measure with `make perf-compare` against a real tree, interleaved and paired.
  A change is kept only when the median improves at least 3% *and* the 95% interval lies
  entirely below zero.
- Record every experiment, including the ones that failed, with
  `python -m benchmarks.realtree.record`. The negative results are the most reusable
  part of the ledger: they stop the next person re-running a dead end.
- None of this is in `make check`. A timing gate on a shared CI runner measures the
  runner.

## Architecture

Read
[docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md](docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md)
first for what is built and what is next, then
[docs/project/research/research-2026-08-06-file-rollup-engine.md](docs/project/research/research-2026-08-06-file-rollup-engine.md)
for why the design is shaped this way and which prior art each piece comes from.

Three artifacts and one contract:

- **Index** (`index.rs`) — in-memory parent-pointer tree; every directory carries
  pre-computed roll-up state.
- **Snapshot** (`snapshot.rs`) — the index serialized, invalidated wholesale by an
  engine fingerprint.
- **Delta** (`types.rs`) — a typed, clocked change, and the **only** way the index or
  the cache is ever modified.

`scan.rs` and `watch.rs` are delta *producers*. `index.rs` and `snapshot.rs` are delta
*consumers*. Nothing else mutates state.

## Conventions

- **Data structures are partial-friendly as well as delta-friendly.** A partially walked
  tree is a valid, useful answer as long as the boundary of incompleteness is knowable:
  roll-ups are correct lower bounds, unvisited subtrees are identifiable, and per-value
  provenance carries `status: Partial`. Queries, sessions, and reducers accept partial
  structures as first-class inputs; code that genuinely requires completeness must demand
  it explicitly, never assume it.
  The two properties compose — a delta stream applied to a partial structure yields
  another valid partial structure — and that composition is what progressive results
  are.
  **Serialization is the documented exception, and it is an exception because no format
  has been designed for it yet, not because partial snapshots are unwanted.** Saving
  rejects a non-fresh index today: there is no encoding for an unfinished frontier,
  unknown children, evicted nodes, or a cancelled walk, and inventing one silently would
  produce a snapshot that reloads as if it were complete. Until a format version carries
  a completeness boundary, `save` demands a complete index in its signature and says so
  in its error rather than quietly writing a partial tree.
  Note also that `Status::Partial` records *coverage*, not direction: a value is a
  monotone lower bound only while an additive walk is running, and one truncated by
  errors can move either way.
- **Do not add a mutation path that bypasses `Delta`.** The contract is what keeps the
  in-memory structure, the serialized form, and the change feed from drifting apart.
  A new producer emits deltas; it does not reach into the index.
- **Do not claim performance the benchmarks have not shown.** The current walker is a
  portable `read_dir` + `symlink_metadata` implementation and is explicitly scaffolding.
  Goal 1 is not met until the `getdents64`/`statx` layer replaces it *and* the benchmark
  gate against dut and gdu passes.
  Benchmarks must report cold and warm separately, and raw-walk and with-stats
  separately, or they compare different jobs.
  The cache is not assumed to be a speed-up: its benefit depends on platform and on
  which reducer tiers a view uses (see
  [research-2026-08-10-performance-frontier.md](docs/project/research/research-2026-08-10-performance-frontier.md)),
  and a warm path that loses to a cold scan of the same view is a defect, not a
  trade-off.
- **The cache may never silently lie.** Fingerprints are size + mtime + ctime + inode.
  A corrupt or unrecognized snapshot is treated as absent, never as data.
  Producers that lose precision escalate with `InvalidateSubtree` rather than guessing.
- **Fast without the OS’s help; faster with it.** Every platform API is an optional
  accelerator layered on a portable path that is already fast on its own, and the
  portable path is what correctness depends on.
  Three tiers, in order: an explicit walk that is quick by itself, a cache that is quick
  where a cache can help, and OS-specific enhancements that make the first two cheaper
  where the platform offers them.
  A feature that is unavailable, disabled, or degraded must fall back to the tier below
  it and lose speed, never accuracy — so `getattrlistbulk`, `statx`, `io_uring`,
  fanotify, and the FSEvents journal are all probe-and-fallback, never load-bearing.
- **The journal’s job is bounding uncertainty, not replacing verification.** A change
  journal’s real value on a multi-million-entry tree is that it identifies *where the
  imprecision could be* in milliseconds, which a full walk can only do in minutes.
  That is what makes near-real-time visibility possible at that scale, and it composes
  with the rule above: the journal narrows what must be checked, the walk remains the
  thing that checks it, and provenance records which of the two answered.
- **Trade speed for certainty in the open, never in secret.** A verified answer over a
  huge tree costs minutes and a cached one costs milliseconds, so the trade is
  legitimate and often necessary — but only when every value carries its provenance:
  where it came from, when it was observed, and whether it is final.
  Label per value, not per run; a consumer rendering a thousand rows needs to know which
  of them to trust. Anything that returns a number without that context is the silent lie
  the rule above forbids.
- **Never size an allocation from untrusted input.** Snapshot and journal parsers check
  declared counts against the bytes actually present first; a corrupt file must fail
  closed, not abort on an allocation.
- **Keep the watch layer deletable.** It is behind a feature flag and strictly additive:
  removing it leaves scan, index, snapshot, CLI, and Python surfaces working.
  The index must never learn what a filesystem event is.
- **Two crates, not more.** `fdu` is the library and CLI; `fdu-py` exists only because a
  cdylib cannot also be the crate Rust consumers depend on.
  Module boundaries are free; crate boundaries cost a version number, a publish, and a
  semver promise each.
  Extract a module into a crate when an external consumer exists, not before.
- **GPL-derived designs are clean reimplementations.** `dut`’s atomic-refcount roll-up
  and `fsearch`’s record layout are described in the research doc and must be written
  from those descriptions, not transliterated from their source.
- Complete type annotations on changed code; catch only errors the current layer can
  handle and preserve exception causes.

## Documentation

- Apply `tbd guidelines common-doc-guidelines` to every human-authored document and
  retain the standard footer.
- Link to source documentation instead of duplicating long policy text.
- Never add credentials, private organization or repository names, private issue IDs,
  personal absolute paths, or customer data.

## Dependencies

Read [SUPPLY-CHAIN-SECURITY.md](SUPPLY-CHAIN-SECURITY.md) and
`tbd guidelines supply-chain-hardening` before any dependency change.
Preserve the 14-day cool-off; first-party tools (tbd, flowmark, softschema) are the
documented exceptions, recorded as `exclude-newer-package` entries in
[crates/fdu-py/uv.toml](crates/fdu-py/uv.toml).
The cool-off exists so a compromised upstream release is noticed by somebody else before
we take it, and that argument does not apply to a package this project’s own authors
publish. Commit `Cargo.lock` when dependencies change, and keep the core crate’s
always-on dependency list short — `deny.toml` documents the policy.

## Git

Keep changes focused and preserve unrelated work.
Before handoff: review the diff, run `make check`, update and close the relevant tbd
issues, run `tbd sync`, commit, push, open or update the pull request, and watch CI to
completion.
