# Feature: The Command Line on the Public API

**Date:** 2026-08-22

**Author:** fdu project

**Status:** Draft

## Overview

Move the command line into its own crate, depending on `fdu` the way any other consumer
does, so that “the CLI invents nothing” is enforced by the compiler rather than asserted
by review.

This replaces Phase 2 of
[the Python CLI parity plan](plan-2026-08-21-fdu-python-cli-parity.md), which proposed a
test-only shim written on the Rust library API and run against the golden corpus.

That plan was wrong, and it is worth saying why before describing what replaces it.

## Why the original Phase 2 was wrong

The Python shim earns its keep because Python is a **different surface**: a separate
binding, in a separate language, holding its own copy of the grammar.
It had drifted five times, and every drift was invisible to `make check`.

A Rust shim has none of those properties.
The command line already *is* the Rust library’s consumer, in the same language, against
the same API, calling the same renderer.
A second one would exercise the same code path and produce a deviation file that is
**empty** — which the parity harness reads as “the shim never ran”, because an empty
diff is exactly what a fallthrough produces.
The design did not fit the thing it was pointed at.

It would also be permanent dead weight: a second implementation of the command line,
maintained forever, whose only output is a file asserting it matches the first.

The question Phase 2 was reaching for is still the right question.
It is just answerable directly.

## The question, asked properly

> Can everything the command line does be done through the public API?

A test can only ever sample that.
A **crate boundary decides it**: if the command line lives in `crates/fdu-cli` and
depends on `fdu` as an ordinary dependency, then `crate::` does not resolve, private
items are unreachable, and the compiler answers the question on every build.
Anything the CLI still needs must be made public — a visible, reviewable act rather than
a `pub(crate)` nobody sees.

The answer turns out to be “almost”, which is why this is worth doing now rather than
treating as a rewrite.

## What actually stands in the way

Every path `cli.rs` reaches for was enumerated.
All but three resolve to items that are **already public** — `query`, `scan`,
`snapshot`, `watch`, `watch_session`, `content`, and the cache functions.
The command line simply spells them `crate::` because it lives inside the crate.

Three production dependencies are genuinely not public:

| Item | Module | Disposition |
| --- | --- | --- |
| `human_bytes` | `report_format` | Promote. A caller formatting fdu’s numbers should not reimplement its unit rules. |
| `human_count` | `report_format` | Promote, same argument. |
| `prepare_report_with_scan_diagnostics` | `execution` | Decide. It exists for repository-controlled measurement, so it may belong behind a feature rather than in the general surface. |

Three more are test-only and move with the tests, or use a public equivalent that
already exists: `apply_ok`, `set_initial_freshness`, and `view_label` — the last is
`ViewSpec::label` now.

Two feature gates also have to be settled, because a consumer taking
`default-features = false` cannot currently render a report at all:

- `report_format` is gated behind `cli`. Rendering is not a command-line concern; a
  library caller that can produce a `Report` and not print it is half an API.
- `prepare_report` is gated with it (`fdu-z7sp`). One-shot planning is an execution
  strategy, not a front end.

## Design

`crates/fdu-cli`, a binary crate depending on `fdu`. The library keeps no `cli` module
and ships no binary.

```text
crates/fdu-cli/
  Cargo.toml          depends on fdu; owns clap and anyhow
  src/main.rs         the entry point the `fdu` binary builds from
  src/cli.rs          today's cli.rs, with every `crate::` rewritten to `fdu::`
```

The `cli` feature on `fdu` disappears.
What it gated was clap, anyhow, and the binary, and all three move to the new crate.
`report_format` becomes unconditional.

### What proves it

Nothing new. The 129 golden sessions already are the parity test, and
`scripts/run-golden.mjs` already selects a surface by path.
The new binary must produce **byte-identical** output for all 129, which is a stronger
claim than any shim’s deviation file, because zero differences is the pass condition
rather than a suspicious one.

The Python parity harness continues to work unchanged and keeps its own artifact.

## Non-Goals

- Changing any command-line behaviour.
  This is a move; the goldens must not be regenerated, and a regenerated golden in this
  work is a bug.
- Splitting `cli.rs` into smaller modules.
  Worth doing, but it would hide the move inside a refactor and make the diff
  unreviewable.
- Publishing `fdu-cli` separately.
  The binary is still `fdu`.

## Implementation Plan

- [ ] Promote `human_bytes` and `human_count`, with doc comments saying why a caller
  wants them (`fdu-????`)
- [ ] Decide `prepare_report_with_scan_diagnostics`: promote, or move its one caller
- [ ] Ungate `report_format` and `prepare_report` from the `cli` feature
- [ ] Create `crates/fdu-cli` with the binary target, leaving `fdu` library-only
- [ ] Move `cli.rs`, rewriting `crate::` to `fdu::`; the compiler enumerates anything
  missed
- [ ] Move the CLI’s tests with it
- [ ] Delete `fdu::cli` and the `cli` feature
- [ ] Prove parity: all 129 goldens byte-identical, no golden regenerated
- [ ] Update `make check`, CI, and the release packaging for the new crate layout

## Testing Strategy

The corpus is the test.
In addition:

- `make check` must pass with the same feature combinations, including
  `--no-default-features`, which is what library consumers build.
- `cargo tree` must show `fdu` free of clap and anyhow.
- The parity harness must still run, since `fdu-py` links the library and not the CLI.

## References

- [Python CLI parity](plan-2026-08-21-fdu-python-cli-parity.md), whose Phase 2 this
  replaces
- [fdu design principles](../architecture/fdu-design-principles.md), Principle 7: same
  concepts at every level; the CLI invents nothing

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
