# Path-Independence Matrix

An empirical check that an fdu answer does not depend on how it was produced: which
requests warmed the cache, which cache policy was used, which files changed since, or
which surface asked.
It is the evidence behind
[the explicit core models plan](../../docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md)
and the starting point for the path-independence test that plan adds under `tests/`.

## What It Runs

`make_fixture.sh` builds a small tree with nested `.gitignore` files (one over a 1 KiB
budget), code in supported and unsupported languages, prose, a binary, hidden and empty
entries, a symlink, and varied mtimes.

`matrix.py` runs 63 request variants (every scope flag, selection filter, view, analyzer
set, and size metric) through five phases, each fdu invocation with its own
`XDG_CACHE_HOME`:

| Phase | What it compares with the cold answer |
| --- | --- |
| `cold` | Each request under `--cache off`, and under `auto` on an empty cache |
| `selfwarm` | Each request after itself, under `auto`, `read-only`, and `only` |
| `warm` | Each request after 10 warming requests, under the same three policies |
| `mutation` | Warm histories followed by a touch, an added or deleted file, same-size rewrites, `.gitignore` edits, and a symlink retarget |
| `cross` | The command line against Python `fdu.report`, `fdu.open`, and `fdu.scan` |

Answers are parsed JSON with `generated_at` and `scan_started_at` removed; `source` and
`freshness` are recorded separately.
`analyze.py` groups the differences into `out/summary.txt`, `repro.sh` holds a minimal
command sequence for each distinct difference, and `seqprobe.sh` replays a sequence of
analysis runs and reports what `--cache only` can still serve.

## Running It

Install the build under test into a virtual environment, for example a release wheel,
then point the harness at it:

```shell
export FDU_BIN=/path/to/venv/bin/fdu
export FDU_PYTHON=/path/to/venv/bin/python
python3 matrix.py all
python3 analyze.py
./repro.sh
```

A full run is about 20,000 fdu invocations and takes a few minutes; fixtures, per-run
caches, and outputs stay in this directory and are ignored by Git.

## Results

[`results/2026-09-17-fdu-0.1.0-rc-5f2d36d-summary.txt`](results/2026-09-17-fdu-0.1.0-rc-5f2d36d-summary.txt)
is the summary for release candidate `5f2d36d`: metadata-only requests were identical to
cold on every history, policy, mutation, and surface; content-analysis requests were
not. The plan’s Background section interprets it.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
