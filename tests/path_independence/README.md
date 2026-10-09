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
Three outcomes other than equality are allowed: a named failure under `--stale-ok`, a
stale answer under `--stale-ok` that equals a cold run before the tree changed and says
`provenance.freshness: stale`, and a refusal where the cold run refuses too.
Every route that reads after the same history under the same policy must return the same
kind of outcome.

| Phase | History before the measured request |
| --- | --- |
| `cold` | None, under `auto` on an empty cache |
| `warm` | One warming request, then `auto`, `on`, and `--stale-ok` |
| `serves` | A complete `--cache on` run of the identical request, then `--stale-ok` through the command line and both cache-reading Python routes |
| `implied` | A content view and the analyzer it implies (`--view code` and `--analyze code`): equal cold answers; then a complete `--cache on` run of either and `--stale-ok` of the other on every cache-reading route, which must serve from the shared sidecar; and an `auto` run of either and `auto` of the other, whose content tier must report `revalidated` |
| `selfwarm` | The request itself, then `auto`, `on`, and `--stale-ok` in turn |
| `mutation` | A warming request, then a file change: rewrite, touch, add, delete, `.gitignore` edits, a symlink retarget, or an unreadable directory |
| `cross` | Cold and warm, read through `fdu.report`, `fdu.open`, and `fdu.scan`, one-shot CLI reports, and the complete initial CLI watch report |
| `roots` | Several disjoint subtrees as one report: the cold answer against the merge of each subtree’s own cold answer (sums, unions, and each root’s own tree), then the same answer served by `--stale-ok` from snapshots each subtree left alone, and through `fdu.report` with a list |

[`matrix.py`](matrix.py) defines the requests, warmers, mutations, and the two tiers.
The watch route participates only where its delivery is supported: metadata analysis
with no content view, a full scan scope, and any delivery but `--stale-ok`. Core request
tests and the CLI golden corpus separately pin the named refusals for unsupported watch
deliveries. The index routes, `fdu.open` and `fdu.scan`, are opened with the analyzers
the cold answer’s `request.analyze` says the request enabled, because an index holds
only what it was opened with and a content view never widens it.
The subset runs in `make check` and in CI on every pull request.
It includes code-only warming before mutations, so a later lines request exercises the
unsupported-language history that once changed the answer, and a `--view documents`
warmer, so mutations are also detected from a sidecar that a content view wrote with no
`--analyze`. The `serves` phase requires a cache-only answer marked stale; refusing
every snapshot or silently scanning cold cannot pass it.
Answer equivalence and positive serving controls protect opposite directions of the
contract, and both are required.
The full matrix runs in
[its own workflow](../../.github/workflows/path-independence.yml) weekly, on demand, and
on a pull request labelled `path-independence-full`.

## Known Violations

The conformance gate requires [`known-violations.toml`](known-violations.toml) to be
empty, including its classes.
Classifying or recording a regression cannot make the gate pass.
The registry tools remain available to diagnose historical failures and merge platform
evidence while a fix is in progress; their output is evidence, not a waiver.

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
make path-independence-full   # requires no violations or registered exceptions
```

`make path-independence-record` records the local platform for investigation.
It still fails when any violation remains.
Use the three CI recordings to establish that the registry can be emptied.

## Running It

```shell
make test-path-independence    # the harness's own tests; no build needed
make path-independence         # the subset, against cargo’s debug fdu and .venv-parity
python tests/path_independence/runner.py --tier full --out /tmp/pi-diffs
```

`FDU_BIN` and `FDU_PYTHON` select other builds; both must be absolute paths, and the
Python route refuses to run against the source tree’s package instead of an installed
wheel. Every invocation gets its own `XDG_CACHE_HOME`, so no case reads another’s stored
state.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
