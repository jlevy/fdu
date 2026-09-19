---
title: QA Playbook — Installed CLI End-to-End
description: Repeatable manual and scripted QA for the PATH-installed fdu command
author: Joshua Levy (github.com/jlevy) with LLM assistance
---
# QA Playbook: Installed CLI End-to-End

Manual and scripted QA for the **installed** `fdu` command (the user’s PATH binary, not
a just-built `target/` binary unless `FDU` points there).

**Purpose**: Prove the advertised views, analyzers, formats, cache policies, and a
bounded watch loop behave on a small typed tree; then time the same metadata questions
on a larger working tree; then escalate carefully on a hostile home-Library tree.
Record wall time and peak RSS so later runs can revise the numbers.

**Estimated Time**: 15–20 minutes for the small tree and cache benchmark; 20–40 more if
the medium and large fixtures are set.
Library steps are time-boxed and must stay bounded.

> This is a kind of “manual test”: it is not a strict end-to-end integration test,
> because it is too costly or pass/fail is not clear.
> Goals:
> 
> - Document expected behavior that is difficult to capture fully in unit, integration,
>   or end-to-end/golden tests.
> - Make it possible for an agent to follow the instructions and evaluate if things are
>   working.
> - Make it possible for an agent to walk through and share the results with a human for
>   review as it progresses or when done.

* * *

## Current Status (Last Update 2026-09-18)

| Phase | Status | Notes |
| --- | --- | --- |
| Phase 1: Setup | ✅ Passed | `fdu 0.1.0-dev+gcb9666a2a`; `XDG_CACHE_HOME` isolation |
| Phase 2: Small-tree views | ✅ Passed | All advertised views; `documents` without `--analyze` exits 2 |
| Phase 3: Cache × analyze | ✅ Passed | `auto` second run 3.5× faster; `off` stayed `0 cached` |
| Phase 4: Medium tree | ✅ Passed | Whole-tree metadata only; analyze on `docs/` |
| Phase 5: Bounded Library | ✅ Passed | Depth 2 exit 2 (TCC); no SIGKILL |
| Phase 6: Results | ✅ Passed | [report-2026-09-18-cli-installed-qa.md](../../docs/project/reports/report-2026-09-18-cli-installed-qa.md) |

**Status Legend**: ✅ Passed | ❌ Failed | ⏳ Pending | ⏸️ Blocked

**Test Results (last update 2026-09-18):** see
[report-2026-09-18-cli-installed-qa.md](../../docs/project/reports/report-2026-09-18-cli-installed-qa.md).

**Next Steps:**

1. Run `scripts/run_installed_cli_qa.py` with the fixture env vars for this machine.
2. Replace the dated report table when revising numbers.
3. File beads for product failures; do not treat a Library TCC partial (exit 2) as a
   crash.

* * *

**Prerequisites**:

- An installed `fdu` on `PATH`, or `FDU` / `--fdu` pointing at the binary under test
- Do not rebuild unless `fdu --version` is not the revision you intend to measure
- `/usr/bin/time` (`-l` on macOS; GNU `-f` on Linux)
- Python 3.11+ (stdlib only) to run the harness
- Fixture trees supplied as env vars (see Local Fixtures).
  The playbook never assumes one machine’s absolute paths.

* * *

## Related Documentation: Read for Context

- [usage.md](../../docs/usage.md) — views, analyzers, cache policies, exit status
- [cache-design.md](../../docs/project/guides/cache-design.md) — snapshots and content
  sidecars
- [README.md](../../README.md) — landing-page command table
- [integration-runbook.md](../../docs/project/guides/integration-runbook.md) — workflow
  proof; this playbook is the installed-CLI counterpart
- Dated numbers:
  [report-2026-09-18-cli-installed-qa.md](../../docs/project/reports/report-2026-09-18-cli-installed-qa.md)

## Local Fixtures

The harness reads trees from the environment.
It does not invent defaults that point at a home directory.

| Variable | Role | Bound |
| --- | --- | --- |
| `FDU` | Binary under test | PATH `fdu` if unset |
| `FDU_QA_SMALL` | Typed / engine checkout used for the view and analyze matrix | Required for those phases |
| `FDU_QA_MEDIUM` | Larger working tree | Optional; omit to skip |
| `FDU_QA_MEDIUM_ANALYZE` | Subtree for `--analyze=code` on the medium tree | Optional; default `$FDU_QA_MEDIUM/docs` if that directory exists |
| `FDU_QA_LARGE` | Hostile wide tree (often a home Library) | Optional; omit to skip |
| `FDU_QA_OUT` | Transcript and results directory | Optional temp dir |

This machine’s 2026-09-18 bindings are recorded only in the dated report, so a later run
can point the same variables at other trees.

## How to Run

One process at a time.
The harness never overlaps fdu invocations and is not wired into `make check`.

```bash
export FDU="${FDU:-fdu}"
export FDU_QA_SMALL=/path/to/small-tree
# optional:
# export FDU_QA_MEDIUM=/path/to/medium-tree
# export FDU_QA_LARGE=/path/to/large-tree
export FDU_QA_OUT=/tmp/fdu-qa-out

python3 scripts/run_installed_cli_qa.py
# or a subset:
python3 scripts/run_installed_cli_qa.py --phases sanity,views,cache-analyze,watch
```

**Expected output**:

- `FDU_QA_OUT/results.md` — timing table (phase, name, verdict, exit, real s, RSS MiB)
- `FDU_QA_OUT/results.tsv` and `results.json` — the same rows
- Per-command `stdout.txt` / `stderr.txt` / `time.txt` under `FDU_QA_OUT/<name>/`

**Verify**:

- [ ] `fdu --version` in the transcript matches the binary you meant to test
- [ ] No two fdu scans ran at once
- [ ] Small-tree views exited 0 with non-empty stdout (except a documented note)
- [ ] Cache-off second analyze still reports `0 cached`
- [ ] Cache-on second analyze reports a non-zero `cached` count in the performance
  footer
- [ ] The dated report table was replaced after the run

**Troubleshooting**:

- **Issue**: `/usr/bin/time: illegal option` **Fix**: the wrapper must pass `--` before
  `fdu`. The harness does this; do not invoke `/usr/bin/time` with a flag-looking first
  operand.
- **Issue**: First “cold” metadata run looks warm **Fix**: the harness sets
  `XDG_CACHE_HOME` to a fresh temp dir per arm.
  Do not export a leftover `XDG_CACHE_HOME` that already holds a snapshot for that tree.
- **Issue**: Numbers cannot be compared to the last report **Fix**: record host, OS,
  `fdu --version`, and whether the filesystem cache was already warm.
  This suite is directional, not a paired `make perf-compare` claim.

* * *

## Phase 1: Setup

### 1.1 Identify the Binary

```bash
which "${FDU:-fdu}"
fdu --version
fdu --help | sed -n '1,80p'
```

**Expected output**:

```
fdu 0.1.0-dev+g<sha>
```

Help must list `--view`, `--analyze`, `--cache` (`auto`, `refresh`, `read-only`, `only`,
`off`), `--cache-status`, `--cache-clear`, `--scan-depth`, `--watch`, and `--interval`.

**Verify**:

- [ ] Version is the revision under test
- [ ] `--cache=off` is the disable switch (there is no `--no-cache`)
- [ ] `XDG_CACHE_HOME` is honored (see `user_cache_dir` in `fdu-core`); the harness uses
  a fresh temp dir per arm and does not call `--cache-clear` on the user cache

**Troubleshooting**:

- **Issue**: Version is not the intended revision **Fix**: stop.
  Do not rebuild unless the operator asked for a rebuild.

### 1.2 Isolate Cache

The harness sets `XDG_CACHE_HOME` to a new directory for:

- the small-tree view matrix (`--cache=auto`)
- the cache-off analyze arm (`--cache=off`, still redirected so a bug cannot write the
  user cache)
- the cache-on analyze arm (empty dir, then the second run)
- the medium and large trees

`--cache=off` neither reads nor writes fdu cache data.
`--cache=auto` writes a snapshot and a content sidecar on a complete indexed analyze.

* * *

## Phase 2: Small-Tree View Robustness

Tree: `FDU_QA_SMALL`. First metadata command is a cold/warm pair of the default tree
view. Remaining views reuse that isolated `auto` cache.

### 2.1 Views the CLI Advertises

The harness runs, in order:

| Name | Command |
| --- | --- |
| tree-cold / tree-warm | `fdu $SMALL` |
| summary | `--view=summary` |
| languages, families, types, extensions | one view each |
| documents-no-analyze | `--view=documents` (no `--analyze`; expect a note, not a crash) |
| recent / largest / files | `--limit=10` on files/recent |
| full | `--view=full` |
| combo-kinds | `--view=families,types,extensions` |
| exclude-ignored-summary | `--exclude-ignored --view=summary` |
| depth-limit-tree | `--depth=1 --limit=5` |
| scan-depth-1-summary | `--scan-depth=1 --view=summary` |
| json-summary / yaml-summary | `--format=json` and `--format=yaml` |

**Expected behavior**:

- Exit 0
- Tree and summary show a non-zero size and file count
- Combined kinds print three breakdowns from one walk
- `--exclude-ignored` totals are ≤ default totals
- `--scan-depth=1` is smaller than a full summary
- JSON parses; YAML is non-empty and names the same totals
- `--view=documents` without `--analyze` exits 2; stderr says views never enable
  analysis. That is success for the negative check.

**Verify**:

- [ ] Each view exited 0
- [ ] `documents` without `--analyze` exits 2 with a usage line on stderr (views never
  enable analysis)
- [ ] JSON/YAML summary is valid and not megabytes

**Check for ERROR conditions** (any of these = FAIL):

- [ ] No hang with no output beyond the large-tree time box
- [ ] No empty stdout on a view that should print rows
- [ ] No traceback or panic

* * *

## Phase 3: Cache Off vs Cache On Analyze

Fresh `XDG_CACHE_HOME` per arm.
Same tree. Sequential.
`/usr/bin/time -l` on macOS.

### 3.1 Cache Disabled

```bash
export XDG_CACHE_HOME=/tmp/fdu-qa-cache-off
fdu "$FDU_QA_SMALL" --analyze=code --cache=off
fdu "$FDU_QA_SMALL" --analyze=code --cache=off
fdu "$FDU_QA_SMALL" --analyze=lines --cache=off
fdu "$FDU_QA_SMALL" --analyze=lines --cache=off
fdu "$FDU_QA_SMALL" --cache-status
```

**Expected behavior**:

- Both runs of each analyzer are content-cold: footer `0 cached`, non-zero `fresh`
- `--cache-status` reports no snapshot for this isolated dir
- Second run is not required to be faster

**Verify**:

- [ ] `--analyze=code` fills language / SLOC rows
- [ ] `--analyze=lines` fills physical / blank / nonblank (and raw words)
- [ ] Second off run still shows `0 cached`

### 3.2 Cache Enabled

```bash
export XDG_CACHE_HOME=/tmp/fdu-qa-cache-on   # empty
fdu "$FDU_QA_SMALL" --analyze=code --cache=auto
fdu "$FDU_QA_SMALL" --analyze=code --cache=auto
fdu "$FDU_QA_SMALL" --analyze=lines --cache=auto
fdu "$FDU_QA_SMALL" --analyze=lines --cache=auto
fdu "$FDU_QA_SMALL" --cache-status
```

**Expected behavior**:

- First run: `cold scan`, non-zero `fresh`, `0 cached`
- Second run: `warm revalidation`, non-zero `cached`, content bytes read at or near 0
- Second run materially faster on an unchanged tree
- `--cache-status` lists a snapshot under the isolated dir

**Verify**:

- [ ] Footer `cached` count rose on the second auto run
- [ ] If the second auto run is not materially faster, record that as a finding (not a
  silent pass)

**Check for ERROR conditions**:

- [ ] `--cache=off` must not populate the isolated cache dir
- [ ] `--cache=auto` must not write `~/Library/Caches/fdu` while `XDG_CACHE_HOME` is set

### 3.3 Extra Analyzers and Formats

After the cache-on arm, the harness reuses that warm sidecar for `--analyze=words`,
`--analyze=all`, JSON languages+code, YAML summary+lines, and text summary+code.

**Verify**:

- [ ] `words` and `all` exit 0
- [ ] JSON `analysis` is present and `physical_lines` is not 0 when lines ran

### 3.4 Watch (SIGINT)

The harness watches a **temp directory** (not the fixture tree), writes one file, then
sends SIGINT. `--interval` only throttles repaint.

**Verify**:

- [ ] Process exits on SIGINT (0, 130, or the platform SIGINT status)
- [ ] It does not remain after the harness returns
- [ ] The temp directory is deleted

* * *

## Phase 4: Medium Tree

Skip if `FDU_QA_MEDIUM` is unset.

### 4.1 Metadata, Then Bounded Analyze

Same views as the small tree’s headline set: tree cold/warm, summary, languages,
families+types+extensions, recent `--limit=10`, JSON summary.

**Do not** `--analyze` the whole medium tree if it holds large generated or vendor
bodies. The harness analyzes `FDU_QA_MEDIUM_ANALYZE` or `$FDU_QA_MEDIUM/docs` only.

**Verify**:

- [ ] Warm tree is faster than cold, or the footer says `warm revalidation`
- [ ] Analyze stays on the subdirectory
- [ ] Output is not a multi-megabyte `files` dump (`--limit=10` on list views)

* * *

## Phase 5: Bounded Large Tree

Skip if `FDU_QA_LARGE` is unset.
**Do not** start with an unbounded full tree or `--analyze` on the whole Library.

Known risk: a home-Library scan has been SIGKILLed (137) from unbounded growth.
Escalate only:

1. `--view=summary --scan-depth=1 --limit=20`
2. `--view=summary --scan-depth=2 --limit=20`
3. Bounded `--view=tree --scan-depth=2 --depth=1 --limit=10` on a few top-level
   directories (`Preferences`, `Logs` when present).
   Do not point the harness at `Containers` or `Caches` as the large root.

**Stop the phase** if any of these occur:

- Exit 137 / SIGKILL
- Peak RSS ≥ `--rss-limit-mib` (default 2048)
- No completion within `--large-timeout` (default 180s) — interrupt and record

TCC denials and exit 2 are expected on a home Library.
Distinguish “fdu returned a partial result” from “fdu exploded or was killed.”

**Verify**:

- [ ] Depth-1 summary completed or failed with a documented partial
- [ ] The harness did not launch a second fdu after a SIGKILL
- [ ] No `--analyze` on the large root

* * *

## Phase 6: Results

### 6.1 Record the Table

Copy `FDU_QA_OUT/results.md` into a dated file under `docs/project/reports/` (or replace
the table in the current report).
Keep the playbook’s procedure stable; revise numbers in the report.

```bash
python3 scripts/run_installed_cli_qa.py   # writes FDU_QA_OUT/results.md
make docs-format
```

**Quality checklist**:

| Item | Check | Status |
| --- | --- | --- |
| Version | Matches the PATH binary | ⏳ |
| Sequential | One fdu at a time | ⏳ |
| Cache isolation | `XDG_CACHE_HOME` per arm; user cache untouched | ⏳ |
| Cache-off | Second run still `0 cached` | ⏳ |
| Cache-on | Second run shows cached reuse | ⏳ |
| Large tree | Bounded; no full-Library analyze | ⏳ |

* * *

## Phase 7: Cleanup

The harness uses temp dirs for cache homes and the watch tree.
They live under the system temp directory.
The operator may delete `FDU_QA_OUT` after copying the table.

Do not `--cache-clear=all` on the user cache as part of this playbook.

* * *

## Troubleshooting

### A command prints nothing for minutes

On the large tree, interrupt at `--large-timeout` and record a hang.
Do not raise the timeout and wait out an unbounded scan.

### SIGKILL (137)

Stop the large phase.
File or update a scale bead with argv, wall time, and peak RSS if known.
Do not retry the same unbounded command.

### Second cached analyze is not faster

Record it. Confirm the footer: if `cached` is 0, the sidecar was not reused (wrong
analyzer set, `XDG_CACHE_HOME` changed, or `--cache=off` left on).
If `cached` is high but wall time did not drop, metadata revalidation may dominate on
that tree — say so.

### `make check` and this suite

This harness is not a Make target in the handoff gate.
Do not add the Library fixture to `make check`.

* * *

## Success Criteria

Before marking this test as **PASSED**, verify:

- [ ] Small-tree views and sanity flags exited 0
- [ ] `code` and `lines` produced filled analysis slots
- [ ] Cache-off vs cache-on first/second timings are in the dated report
- [ ] Watch exited on SIGINT
- [ ] Medium and large phases were either run under bounds or explicitly skipped
- [ ] No critical panic, hang, or SIGKILL on the small tree
- [ ] Product bugs were filed as beads

* * *

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
