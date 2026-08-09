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

- **Do not add a mutation path that bypasses `Delta`.** The contract is what keeps the
  in-memory structure, the serialized form, and the change feed from drifting apart.
  A new producer emits deltas; it does not reach into the index.
- **Do not claim performance the benchmarks have not shown.** The current walker is a
  portable `read_dir` + `symlink_metadata` implementation and is explicitly scaffolding.
  Goal 1 is not met until the `getdents64`/`statx` layer replaces it *and* the benchmark
  gate against dut and gdu passes.
  Benchmarks must report cold and warm separately, and raw-walk and with-stats
  separately, or they compare different jobs.
- **The cache may never silently lie.** Fingerprints are size + mtime + ctime + inode.
  A corrupt or unrecognized snapshot is treated as absent, never as data.
  Producers that lose precision escalate with `InvalidateSubtree` rather than guessing.
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

Read `tbd guidelines supply-chain-hardening` before any dependency change.
Preserve the 14-day cool-off; first-party tools (tbd, flowmark) are the documented
exceptions. Commit `Cargo.lock` when dependencies change, and keep the core crate’s
always-on dependency list short — `deny.toml` documents the policy.

## Git

Keep changes focused and preserve unrelated work.
Before handoff: review the diff, run `make check`, update and close the relevant tbd
issues, run `tbd sync`, commit, push, open or update the pull request, and watch CI to
completion.
