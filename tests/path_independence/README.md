# Path-Independence Harness

An fdu answer must not depend on how it was produced: which requests warmed the cache,
which cache policy served it, what changed on disk since, or which surface asked.
This harness checks that invariant, stated in
[the explicit core models plan](../../docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md#the-rule),
against the built command line and the installed Python wheel.

## What It Checks

Each case asks a request after some history and compares the parsed answer with a cold
run of the same request on the same tree (`--cache off`). Content and tree status
(`complete`, `errors`, coverage) must match; provenance (`source`, `freshness`,
`scan_started_at`, `generated_at`) is excluded.
Three outcomes other than equality are allowed: a named failure under `--cache only`, a
stale answer under `--cache only` that equals a cold run before the tree changed and
says `freshness: stale`, and a refusal where the cold run refuses too.
Every route that reads after the same history under the same policy must return the same
kind of outcome.

| Phase | History before the measured request |
| --- | --- |
| `cold` | None, under `auto` on an empty cache |
| `warm` | One warming request, then each policy |
| `selfwarm` | The request itself, then `auto`, `read-only`, and `only` in turn |
| `mutation` | A warming request, then a file change: rewrite, touch, add, delete, `.gitignore` edits, a symlink retarget, or an unreadable directory |
| `cross` | Cold and warm, read through `fdu.report`, `fdu.open`, and `fdu.scan` as well as the command line |

[`matrix.py`](matrix.py) defines the requests, warmers, mutations, and the two tiers.
The subset runs in `make check` and in CI on every pull request, in about 15 seconds.
The full matrix runs in
[its own workflow](../../.github/workflows/path-independence.yml) weekly, on demand, and
on a pull request labelled `path-independence-full`.

## Known Violations

[`known-violations.toml`](known-violations.toml) lists every difference the engine is
known to have, grouped into classes that name the plan item that removes them.
A run fails on a new difference, on a registered difference whose shape changed, and on
a registered case that now conforms, so the registry shrinks as the fixes land.
Read it like a golden: every entry is a wrong answer the code gives today.

When a change adds or removes violations, re-record from a full run, then read each new
entry and give it a class:

```shell
make parity-venv               # the wheel the Python routes run
make path-independence-record  # rewrites known-violations.toml; new entries are unclassified
make path-independence-full    # passes once every entry is classified
```

A failing full run in CI uploads the registry that platform observed and a diff per
failing case, with the history that produced it.
Linux is the authority for the committed registry; an entry that occurs on only one
platform carries `platforms` and is a finding to explain.

## Running It

```shell
make test-path-independence    # the harness's own tests; no build needed
make path-independence         # the subset, against target/debug/fdu and .venv-parity
python tests/path_independence/runner.py --tier full --out /tmp/pi-diffs
```

`FDU_BIN` and `FDU_PYTHON` select other builds; both must be absolute paths, and the
Python route refuses to run against the source tree’s package instead of an installed
wheel. Every invocation gets its own `XDG_CACHE_HOME`, so no case reads another’s stored
state.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
