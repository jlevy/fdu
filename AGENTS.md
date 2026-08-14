# Project Instructions for AI Agents

Instructions and context for AI coding agents working on this project.

The design of fdu, and the principles any change must respect, are in
[fdu-design-principles.md](docs/project/architecture/fdu-design-principles.md).
Read it before changing engine behavior.
This file covers how to operate on the repository; that one covers what the code must be
true to.

<!-- BEGIN TBD INTEGRATION format=f07 surface=agents-md -->
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
It runs the same feature combinations CI does, notably `--no-default-features`, which is
how library consumers build and is otherwise never exercised locally.

### Toolchain versions

`make check` needs `uv` at or above the version CI pins through `astral-sh/setup-uv` in
[the workflow](.github/workflows/ci.yml), and `cargo-deny` on the path for `make audit`.

The uv floor is not cosmetic.
The uv.toml files express the supply-chain cool-off as a relative `exclude-newer`
(`"14 days"`), and releases older than the pin cannot parse that form: they abort with
`failed to parse year in date "14 days"`, which reads like a corrupt config rather than
a stale tool. That one error takes out docs formatting, the performance harness, and the
Python jobs at once, so an old uv looks like several unrelated repository failures.
The `uv-version` preflight now fails fast with a version message instead, and guards
`check`, `docs-format`, `docs-format-check`, and `test-performance` — the targets that
would otherwise report the parse error.
If you hit it on a pre-provisioned image, upgrade with `uv self update` rather than
working around the config.
`UV_MIN_VERSION` in the Makefile tracks the CI pin, so move both together.

## Performance Work

The rules that decide whether a speed change is kept are in
[fdu-design-principles.md](docs/project/architecture/fdu-design-principles.md); the
protocol is [the performance loop](docs/project/guides/performance-loop.md), every
verdict is in
[the experiment ledger](docs/project/reports/report-2026-08-10-fdu-performance-experiments.md),
and which regime each shipped tuning constant was measured in is in
[the platform tuning guide](docs/project/guides/platform-tuning.md).

In practice:

- Profile before changing anything.
  Intuition about where a walker spends its time is reliably wrong, and the ledger has
  the rejected experiments to prove it.
- Measure with `make perf-compare` against a real tree, interleaved and paired.
- Record every experiment, including the ones that failed, with `make perf-record`. The
  negative results are the most reusable part of the ledger: they stop the next person
  re-running a dead end.
- Record the regime, not just the number.
  Platform, host (bare metal or virtualized), and cache state decide what a result is
  evidence about; a constant tuned in one regime is inherited, not proven, in the
  others.
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
First-party tools (tbd, flowmark, softschema) are the documented exceptions, recorded as
`exclude-newer-package` entries in [uv.toml](crates/fdu-py/uv.toml) and
[the benchmark environment](benchmarks/uv.toml).
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
