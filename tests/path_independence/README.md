# Path-Independence Harness

An fdu answer must not depend on how it was produced: which requests warmed the cache,
which cache policy served it, what changed on disk since, or which surface asked.
This harness checks that invariant, stated in
[the explicit core models plan](../../docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md#the-rule),
against the built command line and the installed Python wheel.

## What It Checks

Each case asks a request after some history and compares the parsed answer with a cold
run of the same request on the same tree (`--cache off`). The effective `request`,
`status` (completeness, errors, and coverage), and all answer content must match.
Only the nested `provenance` object is excluded.
Three outcomes other than equality are allowed: a named failure under `--cache only`, a
stale answer under `--cache only` that equals a cold run before the tree changed and
says `provenance.freshness: stale`, and a refusal where the cold run refuses too.
Every route that reads after the same history under the same policy must return the same
kind of outcome.

| Phase | History before the measured request |
| --- | --- |
| `cold` | None, under `auto` on an empty cache |
| `warm` | One warming request, then each policy |
| `selfwarm` | The request itself, then `auto`, `read-only`, and `only` in turn |
| `mutation` | A warming request, then a file change: rewrite, touch, add, delete, `.gitignore` edits, a symlink retarget, or an unreadable directory |
| `cross` | Cold and warm, read through `fdu.report`, `fdu.open`, and `fdu.scan`, one-shot CLI reports, and the complete initial CLI watch report |

[`matrix.py`](matrix.py) defines the requests, warmers, mutations, and the two tiers.
The watch route participates only where its delivery is supported: metadata analysis, a
full scan scope, and a cache policy other than `only`. Core request tests and the CLI
golden corpus separately pin the named refusals for unsupported watch deliveries.
The subset runs in `make check` and in CI on every pull request.
The full matrix runs in
[its own workflow](../../.github/workflows/path-independence.yml) weekly, on demand, and
on a pull request labelled `path-independence-full`.

## Known Violations

[`known-violations.toml`](known-violations.toml) lists every difference the engine is
known to have, grouped into classes that name the plan item that removes them.
A run fails on a new difference, on a registered difference whose shape changed, and on
a registered case that now conforms, so the registry shrinks as the fixes land.
Read it like a golden: every entry is a wrong answer the code gives today.

Each case has one class, the cause that clears last.
A case with two causes shows as a changed shape when the first is fixed; re-read it
rather than deleting it.
An entry that occurs, or takes its shape, on only some platforms names them in
`platforms`, which is itself a finding to explain: a directory’s size changes with its
entries on APFS but not on ext4, for example.

The registry is regenerated from what each platform observed, not from one machine.
The [full-matrix workflow](../../.github/workflows/path-independence.yml) uploads every
case it judged on Linux, macOS, and Windows; download the three `judged-*.json` files
from a run, merge them, then read each new entry and give it a class:

```shell
python tests/path_independence/registry.py judged-Linux.json judged-macOS.json judged-Windows.json
make path-independence-full   # passes once every entry is classified
```

`make path-independence-record` does the same for the local platform alone, which is
enough to classify a change while working; CI then shows what the other platforms add.

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
