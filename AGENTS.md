# Project Instructions for AI Agents

Instructions and context for AI coding agents working on this project.

The design of fdu, and the principles any change must respect, are in
[fdu-design-principles.md](docs/project/architecture/fdu-design-principles.md).
Read it before changing engine behavior, and read its **First Principles** section
before choosing a default, an ordering, an output shape, or a bound — those rules exist
because each was broken once by a choice that looked entirely normal.
The target ownership, commit, paging, shutdown, and client boundaries for long-lived
interactive work are in
[arch-2026-08-25-fdu-opened-root.md](docs/project/architecture/arch-2026-08-25-fdu-opened-root.md).
Read it before changing opened-root, progressive discovery, refresh, journal, observer,
or continuation behavior.
This file covers how to operate on the repository; that one covers what the code must be
true to.

## Three Surfaces, One Engine

fdu ships the same capability three ways: the `fdu-core` engine, the `fdu` command line,
and the `fdu` Python package.
[The surface architecture](docs/project/architecture/fdu-surface-architecture.md) says
what each is and how they are held together; read it before adding a capability to any
one of them, because two rules constrain where a change may go.

**The command line invents nothing.** A capability reachable only by flag is one a
library caller cannot have.
The command line is a separate crate depending on the engine as any consumer does, so
this is a compile error rather than a review comment — but it also means a new
capability belongs in `fdu-core` first, and the command line presents it.

**Every surface gives the same answer.** `make check` replays one golden corpus against
the command line and against the Python package.
Differences are recorded and classified, and one matching no known cause fails the
build.

Two consequences worth knowing before you start:

- Changing what fdu prints changes a golden.
  That is expected; regenerating one without reading the diff is not, and
  `tryscript run --update` writes *what it saw*, which expands named patterns into
  literals and produces a golden that passes only on your machine.
  `make check` runs a portability check that refuses those.
- The parity artifact is recorded by CI on Linux, not locally.
  It holds platform-dependent values, so a local recording cannot falsify itself.

<!-- BEGIN TBD INTEGRATION format=f08 surface=agents-md -->
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

`make check` is the required handoff gate, and `make cross-lint` is its companion for
anything platform-specific.
If it passes, CI should.
It runs the same feature combinations CI does, notably `--no-default-features`, which is
how library consumers build and is otherwise never exercised locally.

### Platform-gated code

`cfg(target_os = ...)` code is invisible to a single-platform lint run, and this
repository keeps its one unsafe exception behind exactly such a gate.
CI lints on ubuntu only, so before `make cross-lint` existed that module had never been
linted anywhere, and the MSRV job had never checked the Windows-only paths — two of
which used an API stable since 1.87 against a declared MSRV of 1.85, so a Windows user
on the minimum could not have built the crate.

Run `make cross-lint` after touching anything under a platform gate.
It checks rather than builds, so no cross-linker is needed:

```shell
rustup target add x86_64-apple-darwin x86_64-pc-windows-msvc
make cross-lint
```

It skips targets that are not installed rather than failing, so it stays usable
anywhere.

### Toolchain Versions

A fresh clone does not have everything `make check` needs, and a container image that
looks provisioned usually is not.
Three tools have to be there before the gate runs, each failing in a different target,
so install them up front rather than discovering them one by one:

```shell
curl -LsSf https://astral.sh/uv/0.12.1/install.sh | sh       # match UV_MIN_VERSION
cargo install cargo-deny --locked --version 0.20.2            # make audit
rustup toolchain install 1.85.0 --profile minimal             # match MSRV
rustup target add x86_64-apple-darwin x86_64-pc-windows-msvc  # make cross-lint
```

The uv and Rust versions must match `UV_MIN_VERSION` and `MSRV` in the Makefile, which
are what CI pins; the commands above spell them out for copy-paste, so re-read the
Makefile rather than trusting these literals if the gate disagrees.
`node_modules` needs no step: `$(NODE_INSTALL_STAMP)` runs `npm ci` on demand, and the
supply-chain and module-name checks use only Node built-ins, so they pass on a clone
that has never run npm.

Pin what you install here as deliberately as any dependency.
cargo-deny is the one tool with no pin in the repository — CI gets it from a SHA-pinned
action, so a local install is the only version a reviewer never sees.
Choose a release that clears the 14-day cool-off, the same rule
[SUPPLY-CHAIN-SECURITY.md](SUPPLY-CHAIN-SECURITY.md) applies to dependencies.

The uv floor is not cosmetic.
The uv.toml files express the supply-chain cool-off as a relative `exclude-newer`
(`"14 days"`), and releases older than the pin cannot parse that form: they abort with
`failed to parse year in date "14 days"`, which reads like a corrupt config rather than
a stale tool. That one error takes out docs formatting, the performance harness, and the
Python jobs at once, so an old uv looks like several unrelated repository failures.
The `uv-version` preflight now fails fast with a version message instead, and every
directly uv-backed Make target depends on it, including the check, documentation,
Python, and performance entry points.
If you hit it on a pre-provisioned image, install the exact reviewed version the
preflight names rather than an unreviewed latest release, and never work around the
config. Prefer the version-pinned installer the preflight prints.
`uv self update <version>` is the more obvious command and is worth trying, but it only
works when uv owns its own install: where an external manager put the binary there it
fails with `The version <x> was not found for the app uv in workspace uv`, which reads
like the release does not exist rather than like the wrong updater.
The installer is unambiguous, so reach for it first and keep the pinned version either
way.

The bootstrap policy enforces one reviewed version across `UV_MIN_VERSION` and both CI
pins.

## Performance Work

The rules that decide whether a speed change is kept are in
[fdu-design-principles.md](docs/project/architecture/fdu-design-principles.md); the
current strategy — what to work on next, with floor-anchored priorities and per-tier
termination criteria — is
[the campaign-2 plan](docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md);
the measured denominator behind it is
[the metadata-walk floor report](docs/project/reports/report-2026-08-23-metadata-walk-floor.md);
the protocol is [the performance loop](docs/project/guides/performance-loop.md), every
verdict is in
[the experiment ledger](docs/project/reports/report-2026-08-10-fdu-performance-experiments.md),
the charted view across all of them — absolute milliseconds as well as paired effects —
is
[the performance evidence report](docs/project/reports/report-2026-08-20-fdu-performance-evidence.md),
and which regime each shipped tuning constant was measured in is in
[the platform tuning guide](docs/project/guides/platform-tuning.md).

The reusable method — how to instrument a system so each pass of the loop is cheaper
than the last, which tier answers which question, and how to keep the instrument from
distorting the measurement — is
[the instrumentation playbook](docs/project/guides/performance-instrumentation-playbook.md).
Read it before adding instrumentation or starting a fresh optimization campaign; the
mechanism it describes lives in the
[`fdu_core::counters`](crates/fdu-core/src/counters.rs) subsystem.

In practice:

- Instrument before optimizing.
  Counters are compiled in and off by default; `FDU_COUNTERS=1` turns them on for any
  run, so visibility costs a variable rather than a rebuild.
- Profile before changing anything.
  Intuition about where a walker spends its time is reliably wrong, and the ledger has
  the rejected experiments to prove it.
- Measure with `make perf-compare` against a real tree, interleaved and paired.
- Record every experiment, including the ones that failed, with `make perf-record`. The
  negative results are the most reusable part of the ledger: they stop the next person
  re-running a dead end.
- Republish after recording: `make perf-ledger` then `make perf-report`, committed with
  the artifact. `make check` fails if either generated file has drifted from the
  evidence, so an unpublished experiment is caught before merge.
  The rules a new figure has to respect are in
  [the performance loop’s publishing section](docs/project/guides/performance-loop.md#publishing-the-evidence).
- Record the regime, not just the number.
  Platform, host (bare metal or virtualized), and cache state decide what a result is
  evidence about; a constant tuned in one regime is inherited, not proven, in the
  others.
- Do not use a RAM disk for ordinary builds or claim-grade real-tree measurements.
  If a named synthetic experiment requires one on macOS, follow
  [the temporary-volume lifecycle](docs/project/guides/performance-loop.md#temporary-volumes-on-macos):
  audit existing images first, use at most one, keep source and unique results outside
  it, and detach it before handoff.
- None of this is in `make check`. A timing gate on a shared CI runner measures the
  runner.

## Documentation

- Apply `tbd guidelines common-doc-guidelines` to every human-authored document and
  retain the standard footer.
- Run `make docs-format` before handoff.
- Link to source documentation instead of duplicating long policy text.
- Never add credentials, private organization or repository names, private issue IDs,
  personal absolute paths, or customer data.

## Dependencies

Read [SUPPLY-CHAIN-SECURITY.md](SUPPLY-CHAIN-SECURITY.md) and
`tbd guidelines supply-chain-hardening` before any dependency change.

Preserve the 14-day cool-off.
First-party tools are the documented exceptions, listed once by identity under
`firstParty` in [supply-chain-policy.json](supply-chain-policy.json) and as
`exclude-newer-package` entries in [uv.toml](crates/fdu-py/uv.toml) and
[the benchmark environment](explorations/benchmarks/uv.toml).
They carry no version, so upgrading one never means editing a waiver.
The cool-off exists so a compromised upstream release is noticed by somebody else before
we take it, and that argument does not apply to a package this project’s own authors
publish.

Commit `Cargo.lock` when dependencies change, and keep the core crate’s always-on
dependency list short.
`deny.toml` documents the policy.

## Git

Keep changes focused and preserve unrelated work.

Before handoff: review the diff, run `make check`, update and close the relevant tbd
issues, run `tbd sync`, commit, push, open or update the pull request, and watch CI to
completion.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
