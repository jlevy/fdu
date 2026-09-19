# The Performance Loop Runbook: One Unattended Round

How to run one turn of [the performance loop](performance-loop.md) on this host without
a person watching, from picking the hypothesis to the commit that records the verdict.

The loop guide is the protocol: why each step exists and what a result means.
This document is the checklist an agent follows at 3 a.m., and it is written to the
resume rule of
[the experiment-loop method](../specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md):
everything needed to pick the loop up mid-stream lives in the registry (what to try
next), the record (what has been tried), and here (how to run one round).
Every command below was run once while writing it.

Start at [Current Standing](#current-standing-2026-09-18). That section is the pickup:
standing best, host regime, Darwin subjects, and the next-up list with enough context to
start each item. Do not reconstruct the queue from chat, from `macos-agenda` priority
order, or from the 2026-08-23 Tier 1 list alone.

## Current Standing (2026-09-18)

Post-0.1.0 Darwin revisit on this desktop (Apple M1 Pro, Darwin 25.5.0, bare metal,
APFS). The engine that shipped in 0.1.0 has a unified request model, opened-root
serving, watch, `.gitignore` default-on, and a content sidecar.
Campaign 1 and campaign 2 remain the history; this standing is a registry and
measurement layer on top of them, not a rewrite of H86.

Branch `perf/campaign-quiet-2026-09-18`, in a linked worktree beside the primary
checkout, PR [#91](https://github.com/jlevy/fdu/pull/91). Until that PR merges, continue
on it. Do not open a second performance PR. Never merge.
Never force-push.

### Standing Best and Regime

**exp-105** is the current rustup *probe* self-comparison baseline, 12-pair,
`os_cache: warm-steady`, **uncontrolled**.

| Job | Wall median | Peak RSS | Subject |
| --- | ---: | ---: | --- |
| `default-tree` | 149.8 ms | 33.0 MiB | rustup-toolchains, 77,132 entries / 73,714 files |
| `cold-scan-index` | 297.1 ms | 24.7 MiB | same |

Probe `default-tree` is about 492k files/s on that tree.
That sits above the README ballpark of 200K files/s, so the ballpark is not an overclaim
of engine capability, and it is **not a reason to raise it**.

**exp-107 / H108** is the installed-CLI determination on an immutable deciding tree:
`system-private-frameworks`, 158,705 entries / 96,542 files (35% directories), this
branch’s release CLI (`fdu 0.1.0-dev+gbd03cd6cc`). One OS warmup, then 12 isolated-cache
first/second `fdu PATH` pairs.
Quiet start gate passed (CPU busy 20.44%); final 26.08% broke the cell.
Labeled **uncontrolled**. The 25% bar was not lowered.

| Arm | Wall median | Peak RSS | files/s |
| --- | ---: | ---: | ---: |
| first `fdu PATH` | 2.100 s | 89.7 MiB | 46.0k |
| second `fdu PATH` | 2.060 s | 89.3 MiB | 46.9k |

Every second run stayed `cold scan`. Wall −0.63% [−4.97%, +4.05%]. **Confirmed.** No
engine patch.
This CLI cell is a directory-heavy system prefix, not a reason to lower the
README 200K capability ballpark (the rustup probe still sits above it).
Do not quote probe files/s as a product claim.

The 2026-09-18 [installed-CLI QA](../reports/report-2026-09-18-cli-installed-qa.md) is
still a different table: 34,145 files in 0.43 s (~79k files/s) on a mutating fdu
checkout. Cached lines/s was not re-measured.

**exp-106 / H107** (rejected): shipped `read_controls` vs `--no-controls` on the live
metabrowser checkout (145,931 entries).
Wall +1.64% [−4.00%, +4.37%]. User CPU +22% and RSS −6.9% cancelled on the critical
path. No engine patch was kept.

### Darwin Subjects

The 2026-08 nominated metabrowser corpus path is gone from disk.
The rustup store is 77k entries, not the 175k recorded in exp-066. Re-observed shapes
live in
[`nominated-subjects-darwin-arm64.json`](../reports/nominated-subjects-darwin-arm64.json).
Absolute paths live only in the gitignored `explorations/benchmarks/subjects.local.json`
(labels: `rustup-toolchains`, `metabrowser-clone`, `system-private-frameworks`,
`cargo-registry-src`). Read them from there.
Do not type a path into a commit.

`cargo-registry-src` (~22k) screens; it cannot decide a 3% verdict.
`system-private-frameworks` was the H108 subject (exp-107); digest unchanged from the
nomination. The CLI QA medium tree was skipped: deciding-scale but mutating.

### Next Up

Take these in order.
The registry row in [the loop guide](performance-loop.md#current-engine-010) is the full
statement.
Next free hypothesis id is **H112**. Do not mint another meaning for H91–H106.
Next free experiment id is **exp-108**.

1. **H109** (`fdu-8nwq` / `fdu-hzyb`). First a deciding-scale `content-cache-hit`
   **profile** on a controls-bearing tree (`metabrowser-clone`). The exp-107 metadata
   CLI profile is not that instrument: 0 control reads, 0 same-parent path comparisons,
   walk ~96% of wall. A 667-file content pair with no `.gitignore` also showed 0 path
   comparisons. Skip the Path rewrite if the deciding content profile share collapses.
   Predicted only after that profile: `content-cache-hit` wall ≥3% with the interval
   below zero and control goldens identical.
   Do not land an instruction-only trim (exp-104).

2. **H107** (`fdu-jcfn`, closed).
   Re-open only for a tree whose *ignored share is the walk* (a checkout sitting on
   `node_modules` that `.gitignore` drops).
   Job: `default-tree` wall, controls-on vs `--no-controls`; |median| ≥3% and the
   interval off zero, either direction.
   Refuted on wall on metabrowser (exp-106). The H108 subject had 0 control files.
   Do not retry on a metabrowser-like tree whose ignore set is not the critical path.

3. **H108** (`fdu-1a4z`, confirmed in exp-107). Do not open a cache/one-shot patch:
   `ReportPlan::read_snapshot` is already false for metadata one-shot, the second CLI
   run repeats the walk, and loading a snapshot is the H9 loss.
   Do not treat a second `cold scan` as a regression.

4. **H111** (`fdu-jekg`). Linux floor stage of H86: index ≤1.4× floor, aggregate ≤1.25×,
   RSS ≤3× `arena_spike`, on the 450k Linux subject, quiet `make perf-floor`. Darwin
   composite landed (exp-091–102); Linux floor failed (exp-103). A Darwin vs pre-H86
   validation is not this claim.
   Do not restart the rewrite.
   A Darwin agent records that this is not tonight’s item.

H110 needs a new named mechanism.
Do not retry H104–H106.

### Dead Ends

- Do not restart the H86 structural rewrite (`fdu-xde5` remains for H111 only).
- Do not pad the night with unmeasured engine refactors.
- Do not invent a capability that exists only on the command line.
- Do not raise or lower the README 200K files/s or 4M cached lines/s from a probe cell
  or from one directory-heavy CLI tree.
- Do not treat campaign-1 no-controls walls as the current default speed; treat their
  tallies as a different answer (exp-106).
- Do not force a metadata one-shot to load its snapshot (H108 / H9).
- A quiet cell may not hold on this desktop.
  Attempt `PERF_HOST_REGIME=quiet` first; if it fails or the final snapshot exceeds 25%
  busy, label **uncontrolled** and do not claim quiet.
  Do not lower the 25% busy bar so the cell passes.

### Process Pack

| Document | Role |
| --- | --- |
| This standing section | Pickup: what to run next, on which subject, under which regime |
| [The loop guide](performance-loop.md) | Protocol, accept rule, hypothesis registry |
| [The campaign-2 plan](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md) | Floor-anchored strategy; the 2026-08-23 Tier 1–3 list is history |
| [The instrumentation playbook](performance-instrumentation-playbook.md) | Instrument before optimizing; `FDU_COUNTERS=1` |
| [First Principles](../architecture/fdu-design-principles.md#first-principles) | Caching never changes semantics; one model per concept; keep/reject rules |
| [Engine architecture](../architecture/fdu-engine-architecture.md) | Index ownership, one-shot vs opened serving, journals, observers |
| [The ledger](../reports/report-2026-08-10-fdu-performance-experiments.md) | Every verdict, including negatives |
| [The evidence report](../reports/report-2026-08-20-fdu-performance-evidence.md) | Charted view; regenerate after every record |
| [Platform tuning](platform-tuning.md) | Which shipped constant was measured in which regime |
| [Installed-CLI QA](../reports/report-2026-09-18-cli-installed-qa.md) | Product-path table; not interchangeable with probe jobs |

## Before the First Round

Do these once per session, in order.
Each one has caught a real mistake.

1. **Use a linked worktree, one branch, one PR.** Leave the primary checkout intact.

   ```shell
   git fetch origin
   git worktree add -b perf/campaign-$(date +%Y-%m-%d) \
     ../fdu-perf-$(date +%Y%m%d) origin/main
   ```

   If PR #91 is still open, continue on `perf/campaign-quiet-2026-09-18` in its existing
   worktree instead of creating a second branch.
   One pull request, updated after every experiment, never merged unattended.

2. **Find the queue.** Start from [Current Standing](#current-standing-2026-09-18), not
   from the `macos-agenda` label in isolation.
   That label still exists and still holds older campaign-2 items; several have landed,
   and H86’s remaining gap is H111 on Linux.

   ```shell
   tbd show fdu-1a4z fdu-8nwq fdu-jekg fdu-even
   tbd list --label macos-agenda
   ```

   Read the bead before starting: its notes hold the recorded attempts and the blocker
   that may have moved.

3. **Audit for leftovers.** An earlier agent may have left a RAM disk or a worktree; an
   unexplained one is cleanup, not a shared cache, and the loop guide’s
   [temporary-volume section](performance-loop.md#temporary-volumes-on-macos) says how
   to resolve it.

   ```shell
   hdiutil info | grep -c "image-path" ; git worktree list
   ```

4. **Confirm the subjects are the ones on record.**

   ```shell
   make perf-subjects-check
   ```

   Nominated trees drift; the check says what moved.
   Drift in a deciding subject means re-observing it (`make perf-subjects`) and
   committing the new document with the first experiment, so the ledger’s fingerprints
   and the subject document agree.

5. **Build the control probe from the starting commit and copy it out of the tree.**

   ```shell
   make perf-probe-release
   mkdir -p /tmp/fdu-realtree && cp target/release/examples/perf_probe /tmp/fdu-realtree/perf_probe.control
   git rev-parse --short HEAD > /tmp/fdu-realtree/perf_probe.control.commit
   ```

   After an accepted experiment, the candidate becomes the next control: repeat this
   step. After a rejected one, the control is unchanged.

6. **Fingerprint each subject you will measure on.** The label must be the nominated
   label, because the artifact records it and the ledger groups by it.
   The paths live only in the gitignored nominations file, so read them from there
   rather than typing them anywhere a commit could pick them up:

   ```shell
   python3 -c 'import json;[print(s["label"],s["path"]) for s in json.load(open("explorations/benchmarks/subjects.local.json"))]' \
     | while read -r label path; do make perf-baseline PERF_TREE="${path/#\~/$HOME}" PERF_LABEL="$label"; done
   ```

## One Round

```
PICK → PREDICT → CHANGE → MEASURE → DECIDE → RECORD → COMMIT → RE-SCREEN
```

### PICK

Take the next item on the agenda.
Mark it and say so in the bead:

```shell
tbd update fdu-XXXX --status in_progress
```

### PREDICT

Before touching code, write down in the bead notes: the hypothesis id (an existing `HNN`
from the registry, or the next free number — the registry header says which), the tier
and the job that measures it, the subject, the metric and direction, and the regime.
If the change is expected to move a component rather than wall, say so now; a metric
chosen after the run is never an accept.

For a new hypothesis, add its row to the registry table in the loop guide in the same
commit as the artifact.

### CHANGE

The smallest diff that tests the one idea.
One idea per experiment; a second idea is a second round.
Run the unit tests that cover the code you touched before measuring, so the round does
not measure a bug:

```shell
cargo test --locked -p fdu-core
make perf-probe-release
```

### MEASURE

Measurement is the only step that needs the host to itself.
Nothing else may run: no builds, no other agent, no `make check`. Check first, and wait
rather than proceed:

```shell
uptime   # load average per core should be well under 1 before declaring quiet
```

Then run the paired comparison.
`JOBS` names the job the hypothesis predicts plus any job its mechanism could plausibly
move; `NAME` is the experiment id and a slug.

```shell
make perf-compare PERF_TREE=$HOME/.rustup PERF_LABEL=rustup-toolchains \
  CONTROL=/tmp/fdu-realtree/perf_probe.control \
  JOBS="default-tree cold-scan-index" TRIALS=12 \
  PERF_HOST_REGIME=quiet NAME=exp-067-skip-identical-snapshot-rewrite
```

What the flags mean, and what happens when they bite:

- `PERF_HOST_REGIME=quiet` makes the harness refuse to start if the host is over 25%
  busy, and invalidates any sample whose boundary observations exceed it.
  An invalidated sample is kept and counted, never replaced: if too many are invalid the
  round is inconclusive and is run again later, not topped up.
  Attempt quiet first.
  On this desktop a quiet cell often cannot hold (see
  [Current Standing](#current-standing-2026-09-18)): label `uncontrolled` and say so in
  the record. Do not lower the 25% bar so the cell passes.
  `uncontrolled` is exploration, not a held-out claim.
- `TRIALS=12` is the minimum for a verdict.
  A change predicted under 5% wants 16 or 20.
- The harness fingerprints the tree before and after.
  If it changed, the run exits nonzero and the numbers are not comparable: find what
  wrote to the subject, and run again.
- Content-tier hypotheses use `make perf-content-compare` with the same variables; its
  jobs are the content set.
- An aggregate-tier hypothesis (H72, H85) cannot name `aggregate-summary` alone.
  That job measures `fdu --view summary`, which reads `.gitignore` and so retains the
  index; the transient tier needs the probe’s `--no-controls` on both variants.
  `make perf-compare` cannot add it to the candidate, so run `measure` directly, as
  [the aggregate tier](performance-loop.md#the-aggregate-tier) shows.

Run on at least one deciding subject.
A screening subject (`cargo-registry-src`) is for checking that a job works, and its
numbers do not decide anything.

### DECIDE

The harness prints `ACCEPT` or `REJECT` per job from the arithmetic in the accept rule:
median at least 3% faster, the 95% interval entirely below zero, no sample invalidated.
Read it with the three checks the arithmetic cannot make:

- Was it the **predicted** job and metric?
  A win on a job the hypothesis did not name is a new hypothesis, not this one’s
  verdict.
- Is the **tail** acceptable?
  The run JSON records `p95_over_median` per arm and the ledger prints it beside the
  verdict once it reaches 1.5×; a median win with a worse tail is recorded as such.
- Is the **complexity worth it**? Write the one judgment sentence.

A `REJECT` is a result.
It is recorded exactly like an accept, and the code is reverted.

### RECORD

The artifact is lifted from the run JSON; the operator supplies only what a measurement
cannot know. Write the body first — what the profile or the bead suggested, what was
built (by commit and entry point), what the prediction got right and wrong — then
record:

```shell
PROV=$(python3 -c 'import json;print(next(s["provenance"] for s in json.load(open("docs/project/reports/nominated-subjects-darwin-arm64.json"))["subjects"] if s["label"]=="rustup-toolchains"))')
make perf-record ARGS="--run /tmp/fdu-realtree/results/run-exp-067-skip-identical-snapshot-rewrite.json \
  --id exp-067 --title 'Skip the identical snapshot rewrite on the cold-scan path' \
  --hypothesis H100 --control 'main at 778aa74' \
  --candidate 'byte-compare the encoded snapshot against the file before writing' \
  --decision accepted --primary-job default-tree \
  --reason 'one sentence: the number, the gate, the judgment' \
  --commit $(git rev-parse --short HEAD) --lines-changed 40 \
  --tree-provenance \"$PROV\" --body /tmp/fdu-realtree/exp-067-body.md"
make perf-ledger
make perf-report PREPARED=$(date +%Y-%m-%d)
make perf-test
```

`--tree-provenance` is required and has no default; the nominated-subjects document
holds each subject’s sentence, read into `PROV` above, and `--tree-reconstructible` is
added only when that document says `true` for the subject.
`ARGS` is re-parsed by the recipe’s shell, so keep titles and reasons free of
apostrophes, or double-quote them with the quotes escaped as `PROV` is.

`--commit` names the commit that **contains the change**, which is not the commit that
is checked out while recording: the schema means it as the place a reader goes to find
the code. So an accepted experiment lands in two commits — the change alone first, then
the artifact and the regenerated views naming its hash.
Recording before committing puts the *control’s* hash in the field, which points a
reader at the code without the change; that had happened to four artifacts before it was
caught. `--primary-metric` is added only when the hypothesis pre-registered a component.
The id is the next free `exp-NNN`; two agents in one night reserve ranges first, because
a collision is silent until `perf-ledger` fails.

### COMMIT

One commit per experiment, with the numbers in the message:

- **Accepted:** the code, the artifact, the regenerated ledger and evidence page, the
  registry row, and the bead update.
- **Rejected:** the artifact and the views only; the code is reverted first.

Then the gate and the push:

```shell
make check
tbd sync
git push -u origin HEAD
gh pr create --fill   # first experiment only; afterwards, gh pr edit to update the body
```

`make check` fails if the ledger or the evidence page does not match the artifacts,
which is the point: an experiment that is not published cannot merge.
It also takes about seven minutes and loads every core, so it runs *after* measurement,
never during.

### RE-SCREEN

Update the registry row’s status, close or update the bead with the verdict and the
experiment id, and check whether the change moved the next item’s headroom: two
hypotheses aimed at the same cost divide one budget, and this record has seen it three
times. If it did, say so in that bead before starting it.
Rewrite [Current Standing](#current-standing-2026-09-18) so the next-up table and
standing-best numbers match the ledger; a stale standing is how the next agent repeats a
finished experiment.

```shell
tbd close fdu-XXXX --reason "exp-067: accepted, default-tree -18.2% [-21.0%, -15.1%]"
tbd sync
```

## What an Unattended Agent Does Not Do

Each of these is either irreversible, a decision that belongs to a person, or a way of
producing a number that means nothing.

- Merge to `main`, or rebase or force-push the night’s branch.
- Change the accept rule, the bootstrap, the schema’s statistics, or a subject’s
  nomination. The set is re-observed (`make perf-subjects`) when it drifts, not edited.
- Start a person-gated item: the H86 rewrite (`fdu-xde5` is H111’s parent, not a license
  to restart the composite), the `searchfs` spike, the FSEvents journal, hardware
  CRC-32C (`unsafe` or a dependency), or `fdu-n75m` parts 2 and 3 (durability policy).
- Raise or lower the README 200K files/s or 4M cached lines/s from a probe cell.
- Retry H107 on a tree whose ignored share is not the walk.
- Add a dependency, an `unsafe` block, or a platform gate without `make cross-lint`.
- Create a RAM disk, or write anything into a subject tree.
  Snapshots, results and scratch go under `/tmp/fdu-realtree/`.
- Record a verdict from a generated corpus, a screening subject, fewer than 12 trials, a
  run with invalidated samples, or a run whose fingerprint drifted.
- Regenerate a golden with `--update` without reading the diff; change what fdu prints
  without a reason the commit states.
- Measure while anything else is running, including `make check` or a second agent’s
  build.
- Continue past a failing `make check` by narrowing it; fix the failure or stop and say
  so in the PR.

When a step cannot proceed — the host never goes quiet, a subject keeps drifting, a
build fails for a reason outside the change — the right move is to record what was
observed in the bead and the PR body and move to the next agenda item, not to lower a
bar so the step passes.

## The Handoff

The pull request body is the night’s report and the morning’s reading.
After every experiment it carries a table — experiment id, hypothesis, subject, primary
job, change with interval, verdict — and a line for anything skipped and why.
A reader should learn the night’s result from the ledger diff and the PR body without
opening the transcript.
[Current Standing](#current-standing-2026-09-18) is the in-repo pickup for the next
agent; the PR body is not a substitute for updating it.

Before stopping:

```shell
make perf-subjects-check     # the subjects are still what the artifacts say
git status --short           # nothing uncommitted
hdiutil info | grep -c image-path   # no RAM disk left behind: expect 0
tbd sync
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
