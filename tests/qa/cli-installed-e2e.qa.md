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
on a larger working tree; then escalate carefully on a hostile home-Library tree; then
check that fdu’s totals on real trees, `~/Library` among them, agree with other
disk-usage tools, and that every difference has a named cause.
Record wall time and peak RSS so later runs can revise the numbers.

**Estimated Time**: 15–20 minutes for the small tree and cache benchmark; 20–40 more if
the medium and large fixtures are set; about half an hour for peer agreement (Phase 7),
up to an hour on a loaded machine.
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

## Current Status (Last Update 2026-09-29)

This table records the 0.2.1 release commit, `b10fe7b39` on `main`. The pass ran on
`672c2188f`, whose tree (`761850ec6`) the release commit and the release branch head
`ee97bf337` share byte for byte, so every result applies to `b10fe7b39` by tree
identity; no crate or script changed after `e889694c`, the first 0.2.1 release layer.
The installed command was the `cp312-abi3` `manylinux_2_34_x86_64` wheel built from
`672c2188f` as [Install the Candidate](#11-install-the-candidate) describes, installed
with `uv tool install` into an isolated tool directory, and `fdu --version` printed
`fdu 0.2.1-dev+g672c2188f`. The host was a 4-vCPU Linux x86_64 virtual machine
(Firecracker) on ext4, running as root and otherwise quiet; the gates on `672c2188f`
(`make check`, `make cross-lint`, `make release-rehearse`, and `make semver-check`)
passed with `FDU_TEST_ALLOW_NO_PERMISSION_BITS=1` and `FDU_TEST_ALLOW_NO_NATIVE_WATCH=1`
declared, as AGENTS.md prescribes for such a host, and with `UV_PYTHON=3.12`, because
this host’s default `python3` is 3.11 and `test-performance` runs the benchmark tests,
which need 3.12, on the interpreter uv finds.
Each timing is a single run.

| Phase | Status | Notes |
| --- | --- | --- |
| Phase 1: Setup | ✅ Passed | Wheel from `672c2188f`, installed as a uv tool; help lists every flag Phase 1.2 names and no `--no-cache`; `XDG_CACHE_HOME` per arm; user cache listing unchanged before and after |
| Phase 2: Small-tree views | ✅ Passed | This repository’s checkout with its gate-built virtualenvs and `node_modules` (13,656 files, 419 MiB); all views exit 0; `--ignored=exclude` gives 31 MiB and `--scan-depth=1` 280 KiB; JSON and YAML totals agree; `documents` without `--analyze` exits 2 with the usage line |
| Phase 3: Cache × analyze | ✅ Passed | `off` stayed `0 cached` and wrote no snapshot; `auto` reused all 13,656 records (0.90 s to 0.18 s for `code`). The second `lines` run re-read 6 files: another process linked the shared uv cache into the checkout’s virtualenvs mid-run, changing their ctime, and a repeat of the sequence re-read none. `words` and `all` exit 0; JSON `physical_lines` is 2,689,062; watch exited on SIGINT |
| Phase 4: Medium tree | ✅ Passed | A Linux v6.12 source checkout (86,643 files, 358 `.gitignore` files), 0.25 s per metadata run, both `cold scan` as documented; `--analyze=code` on `Documentation/` only (10,121 files, 1.80 s), reused on the second run (0.08 s) |
| Phase 5: Bounded large tree | ✅ Passed | `~/Library` does not exist on Linux, so `/` stood in as the hostile wide tree (`/proc`, `/sys`, `/dev`, a live `/tmp`). Depth 1 and 2 exit 0, and depth 2’s 148 files match `find`; no SIGKILL; peak 14 MiB. No `Preferences` or `Logs` leaf exists, so none ran |
| Phase 6: Terminal Progress | ⏳ Pending | `make test-terminal` passed against the installed command. A pty probe passed all 28 of its checks: frames on a 4 s scan, erased before the report; nothing for a small tree, a redirected stderr (only `note:`, `tip:`, and `perf:` lines), `--format json`, `--progress never`, `CI=1`, or `TERM=dumb`; `--progress always --format json` draws; Ctrl-C erases, prints `fdu: interrupted`, and dies by SIGINT; `NO_COLOR` keeps the animation; frames fit at 60 to 12 columns and after narrowing mid-run, spinner and phase word only below 20; `Analyzing` shows a percentage. A person has not watched a window, and Windows has not run |
| Phase 7: Peer agreement | ✅ Passed | GNU du 9.4, dust 1.2.4, pdu 0.24.0, dua 2.41.1, diskus 0.9.0; BSD du is macOS-only, and so are `/Applications` and `~/Library`. Self-test: all 12 Linux readings agree exactly. On this repository’s checkout, `~/.rustup`, `/usr`, and the Linux checkout, all 48 readings agree exactly, each tree quiet. The script still exits 1, because on ext4 each top-level row differs from GNU du -l by the subtree’s own directory and symbolic-link blocks (`fdu-83km`): 4 self-test rows and 50 of 50 subject rows, each matched to the byte by a separate walk |
| Phase 8: Results | ✅ Passed | 49 harness checks: 48 ok, and 1 expected warning for `documents` exit 2. That row’s 2.000 s is its exit status, which the harness reads from GNU time’s `Command exited with non-zero status 2` line; it took 0.02 s. Full tables are in the pull request that records this run; the correctness record is in the [correctness runbook](../../docs/project/guides/correctness-runbook.md#last-recorded-run) |

This pass says nothing about the macOS walk.
The last macOS pass, for 0.2.0 at `6ec77163a` on bare-metal Apple silicon and APFS,
passed every phase but Phase 6’s watched window; its table is in this file’s history.

**Status Legend**: ✅ Passed | ❌ Failed | ⏳ Pending | ⏸️ Blocked

**Test Results:** the 0.1.0 numbers are in
[report-2026-09-25-release-candidate-qa.md](../../docs/project/reports/report-2026-09-25-release-candidate-qa.md).
The 0.2.0 and 0.2.1 tables are in the pull requests that record those runs.

**Next Steps:**

1. Run `scripts/run_installed_cli_qa.py` with the fixture env vars for this machine.
2. Replace the dated report table when revising numbers.
3. File beads for product failures; do not treat a Library TCC partial (exit 2) as a
   crash.
4. On Linux, until `fdu-83km` teaches the top-level check ext4’s directory blocks, check
   each top-level row against the subtree’s own directory and symbolic-link blocks.

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
  [report-2026-09-25-release-candidate-qa.md](../../docs/project/reports/report-2026-09-25-release-candidate-qa.md)

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
- **Issue**: First “cold” analysis run looks warm (a metadata run under `auto` never
  reads a snapshot) **Fix**: the harness sets `XDG_CACHE_HOME` to a fresh temp dir per
  arm. Do not export a leftover `XDG_CACHE_HOME` that already holds a snapshot for that
  tree.
- **Issue**: Numbers cannot be compared to the last report **Fix**: record host, OS,
  `fdu --version`, and whether the filesystem cache was already warm.
  This suite is directional, not a paired `make perf-compare` claim.

* * *

## Phase 1: Setup

### 1.1 Install the Candidate

Before a release, the binary under test is the release candidate: the commit on `main`
that will be tagged.
Build its release wheel in a clean worktree of that commit and install it as the `fdu`
on `PATH`, replacing any earlier install:

```bash
git worktree add --detach "$CANDIDATE_DIR" <commit>
cd "$CANDIDATE_DIR/crates/fdu-py"
uv run --frozen --only-group dev maturin build --locked --release --out "$FDU_QA_OUT/wheel"
uv tool install --force --python 3.12 --no-index "$FDU_QA_OUT"/wheel/fdu-*.whl
fdu --version
```

The version names the commit it was built from (`0.2.0-dev+g` and its first nine hex
digits); it must be the candidate’s, with no `.dirty` suffix.
Keep the worktree and its build on a volume with room: a release build takes a few
gigabytes.
The wheel the release rehearsal built for this platform is an alternative that
is closer to what users install: `make release-candidate` downloads it into
`$RELEASE/rehearsal/files`, and
[the release process](../../docs/project/guides/release-process.md#stability-pass) gives
the command that installs it.
It reports the bare release version, `fdu X.Y.Z`.

### 1.2 Identify the Binary

```bash
which "${FDU:-fdu}"
fdu --version
fdu --help | sed -n '1,80p'
```

**Expected output**:

```
fdu 0.2.0-dev+g<sha>
```

Help must list `--view`, `--analyze`, `--cache` (`auto`, `on`, `off`), `--stale-ok`,
`--cache-status`, `--cache-clear`, `--scan-depth`, `--watch`, and `--interval`.

**Verify**:

- [ ] Version is the revision under test
- [ ] `--cache=off` is the disable switch (there is no `--no-cache`)
- [ ] `XDG_CACHE_HOME` is honored (see `user_cache_dir` in `fdu-core`); the harness uses
  a fresh temp dir per arm and does not call `--cache-clear` on the user cache

**Troubleshooting**:

- **Issue**: Version is not the intended revision **Fix**: stop.
  Do not rebuild unless the operator asked for a rebuild.

### 1.3 Isolate Cache

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
view.
Under the default `auto` policy a one-shot metadata report neither reads nor writes
a snapshot, so both runs, and every view after them, report `cold scan`.

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
| exclude-ignored-summary | `--ignored=exclude --view=summary` |
| depth-limit-tree | `--depth=1 --limit=5` |
| scan-depth-1-summary | `--scan-depth=1 --view=summary` |
| json-summary / yaml-summary | `--format=json` and `--format=yaml` |

**Expected behavior**:

- Exit 0
- Tree and summary show a non-zero size and file count
- Combined kinds print three breakdowns from one walk
- `--ignored=exclude` totals are ≤ default totals
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
- [ ] `--cache=auto` must not write the default `~/.cache/fdu` while `XDG_CACHE_HOME` is
  set

### 3.3 Extra Analyzers and Formats

After the cache-on arm, the harness runs `--analyze=words`, `--analyze=all`, JSON
languages+code, YAML summary+lines, and text summary+code.
The sidecar answers only the analyzer set that wrote it, so a run with a different set
reads the files again ([usage guide](../../docs/usage.md#understand-the-cache)): the
0.2.0 pass saw `analyze-words` report `fresh=11280, cached=0`.

**Verify**:

- [ ] `words` and `all` exit 0
- [ ] JSON `reports[0].metrics.total.metrics.physical_lines` is present and not 0 when
  lines ran; the harness fails the row when it is missing and notes its value

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

- [ ] Both tree runs report `cold scan`: a one-shot metadata report re-walks by design,
  so the warm run need not be faster.
  Reuse shows on the second subdirectory analyze instead, as `warm revalidation` with
  non-zero `cached`
- [ ] Analyze stays on the subdirectory
- [ ] Output is not a multi-megabyte `files` dump (`--limit=10` on list views)

* * *

## Phase 5: Bounded Large Tree

Skip if `FDU_QA_LARGE` is unset.
**Do not** start with an unbounded full tree or `--analyze` on the whole Library.
Escalate here first; once these bounded runs pass, Phase 7 walks the whole of it,
metadata only.

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

## Phase 6: Terminal Progress

Run by hand in a real terminal window (not through the harness, which captures output
and is therefore non-interactive).
The automated `make test-terminal` covers drawing, erasing, and Ctrl-C in a
pseudo-terminal; this phase covers what only a person judges.
Use a tree that takes several seconds, such as `$FDU_QA_MEDIUM`.

**Verify**:

- [ ] `fdu "$FDU_QA_MEDIUM"` shows one animated line on stderr after about half a
  second: spinner, the root, then `Scanning` with climbing counts, `Indexing` briefly,
  then the report, with no line left above it
- [ ] `fdu --analyze all` on a medium subdirectory shows `Analyzing` with a climbing
  percentage (the line is erased as soon as the work ends, so `100%` may never be seen)
- [ ] A small tree (`fdu .` in this repository) shows no indicator at all
- [ ] `fdu "$FDU_QA_MEDIUM" 2>/tmp/fdu-stderr` writes no progress to `/tmp/fdu-stderr`:
  no carriage return or escape sequence, only the report’s `note:`, `warn:`, `tip:`, and
  `perf:` lines, which go to stderr whether or not it is a terminal
- [ ] `fdu --format json "$FDU_QA_MEDIUM" >/dev/null` shows nothing;
  `--progress always --format json` shows the indicator; `--progress never` shows
  nothing for any format
- [ ] `CI=1 fdu "$FDU_QA_MEDIUM"` and `TERM=dumb fdu "$FDU_QA_MEDIUM"` show nothing
- [ ] Ctrl-C during the scan erases the line, prints `fdu: interrupted`, and returns to
  a clean prompt; `echo $?` prints 130
- [ ] The counts and the size hold their columns as they grow: nothing after them moves,
  and the size is the one the report will print (allocated unless `--size apparent`),
  never more than the disk holds
- [ ] Narrowing the window while it runs shrinks the frame without wrapping, dropping
  parts in the documented order (the phase padding, then the middle of the root, then
  the counts’ alignment, then dirs, then bytes), and below 20 columns only the spinner
  and the phase word remain
- [ ] `NO_COLOR=1` removes the colors but keeps the animation
- [ ] On Windows Terminal, the same checks hold; with stdout piped (`fdu … | more`), the
  run counts as non-interactive and shows nothing, because virtual-terminal support is
  enabled for stdout and stderr together
- [ ] `FDU_BIN="$(command -v fdu)" make test-terminal` passes against the installed
  command, so a wheel’s console script erases the line and dies by `SIGINT` exactly as
  the cargo-installed binary does

A line left on screen after any of these fails the phase.

* * *

## Phase 7: Peer Agreement on Real Trees

Run on the release candidate before tagging, and after any change to how sizes are
counted. It answers the question a user asks first: does fdu agree with the tool they
already trust?
Not always byte for byte, because tools count some things differently, but
every difference must have a measured cause.
It is written for macOS and APFS, where directories and symbolic links occupy no blocks;
on another filesystem, run the self-test there first.

**Subjects**, at least these four: this repository (a `.gitignore`, build output,
symbolic links), `~/.rustup` or another quiet mid-size tree, `/Applications` (bundles,
symbolic links, compressed files), and `~/Library`: permission-denied folders, sparse
virtual-machine disks, cloud placeholders, clones, and files changing while it is
measured. The whole `~/Library` metadata walk is required here; never `--analyze` it.
Allow about half an hour; on a loaded machine, an hour.

**Tools**: GNU du is required, as the reference (`gdu` from coreutils on macOS); the
script refuses to run without it.
dust, pdu, dua, diskus, and the system du run when installed, and the self-test fails if
any is missing. dumac prints only rounded sizes, so it is not compared.

```bash
python3 scripts/qa_peer_agreement.py --self-test
python3 scripts/qa_peer_agreement.py . ~/.rustup /Applications ~/Library \
  --json "$FDU_QA_OUT/peer-agreement.json" | tee "$FDU_QA_OUT/peer-agreement.md"
```

The self-test builds a small tree with every case below (hard links within and across
directories; symbolic links to a file, a directory, and nowhere, directly inside the
root and deeper; an unreadable folder; a sparse file; a name with spaces) and requires
all 13 tool readings to agree with their counting models exactly.
Run it first, and after upgrading any peer tool.
On APFS it can test the directory and link terms only in apparent size, since they
occupy no blocks. The script exits non-zero if any reading is `UNEXPLAINED` or missing.
`--rejudge FILE` judges saved readings again without measuring.

**Privacy.** The `--json` file holds absolute paths, and directory names can identify
people: a messaging app’s folders carry phone numbers and group identifiers, and even an
application container’s name says what is installed.
Keep the file out of the repository.
The tables name only the first two path components of a folder a tool gave up on, unless
`--full-paths` asks for more; a published report should count them rather than name
them.

**How the tools count.** fdu counts regular files once per path, and nothing else.
GNU du with `--count-links` counts the same way apart from links’ and directories’ own
sizes, so on a quiet APFS tree its allocated total equals fdu’s to the byte.
Every other difference is computed from the tree itself, in one walk:

| Difference | Tools | Expected difference from fdu |
| --- | --- | --- |
| Hard links | fdu, `du -l`, pdu, and dust’s apparent size count a hard-linked file once per path; du, dua, diskus, and dust’s allocated size count it once | Lower by what per-path counting adds, measured from the paths that share an inode |
| Symbolic links | du, dust, pdu, and diskus count a link’s own size, which is its target text; dua does too, except for links directly inside the root, which it takes as inputs | Apparent size higher by the links’ total; allocated unchanged on APFS |
| Directory sizes | dust `-s` and pdu’s apparent size add every directory’s own size; dua adds every directory’s but the root’s; du and diskus add none to apparent size | Apparent size higher by the directories’ total; allocated unchanged on APFS |
| Allocated or apparent | fdu, du, dust, dua, and diskus default to allocated; a sparse disk image, a cloud placeholder, or a compressed file makes the two differ | Compare like with like, never fdu’s default with a tool’s apparent figure |
| A live tree | `~/Library` changes while it is measured | fdu runs just before each tool and once at the end; a tool must fall within its two fdu readings, or outside them by no more than fdu moved during or next to them |
| Folders a tool gave up on | GNU du and pdu on macOS give up on a directory whose read is interrupted, and diskus on one that fails for a moment without saying why; each leaves that subtree out | Every folder a tool reports it could not read, and that the script’s walk could list, is measured afterwards; the tool may be short by what those folders hold, within the fdu readings around it |
| Folders skipped without a name | dua reports only a count of failures | When the count exceeds the paths no tool can read, dua is run again, twice at most; if every run skipped folders, a short reading is marked “not verifiable” and never agrees, and a reading over fdu’s readings beyond fdu’s nearby movement fails |

**fdu against itself.** On a tree that otherwise moved at most once, a stretch of fdu
readings that leaves a value and returns to it, or readings at either end that differ
from the value most readings share, fail on their own rows with their exact bytes: fdu
disagreed with itself, or the tree changed and changed back, so rerun.
fdu may be denied a folder, or find one gone mid-scan; any other error it details fails
the run, and so do more counted-but-undetailed errors than there are paths no tool can
read. It details its first 64 errors and counts the rest.
fdu’s fast macOS reader declines on any failure and its portable reader reads the
directory again, and a second failure would be reported.

**Verify**:

- [ ] The self-test agrees exactly for all 13 readings, with no tool missing
- [ ] The script ends with “Every reading is explained”, or with every other reading
  explained and a dua reading that could not be checked: each tool agrees exactly on a
  quiet tree, one whose every fdu reading was the same; within the tree’s movement or
  fdu’s nearby movement on a live one; or short by what the folders it gave up on hold
- [ ] Every top-level directory’s allocated size agrees with GNU du’s, exactly on a
  quiet tree and within fdu’s readings around it on a live one, and none is present on
  only one side
- [ ] fdu’s error count on `~/Library` equals GNU du’s denied count and the script’s
  walk (its unlistable directories plus the files it could not stat), and fdu exits 2
  (partial)
- [ ] fdu’s `~/Library` total is plausible against the volume (`df -h ~`): a figure
  larger than the disk means a sparse file was counted by its apparent size
- [ ] The tables are recorded in a dated report under `docs/project/reports/`

An `UNEXPLAINED` row is a finding: identify the files responsible (compare the top-level
directories, then descend with `fdu --view tree --depth 1 --min-share 0%` and `du -d 1`,
since the tree’s default 1% floor hides small directories), and either extend the model
and the self-test with the new case or file a bead.

* * *

## Phase 8: Results

### 8.1 Record the Table

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

## Phase 9: Cleanup

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
- [ ] The terminal progress phase passed, or was explicitly skipped with a reason
- [ ] The peer-agreement phase shows no `UNEXPLAINED` row on any of its trees
- [ ] Medium and large phases were either run under bounds or explicitly skipped
- [ ] No critical panic, hang, or SIGKILL on the small tree
- [ ] Product bugs were filed as beads

* * *

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
