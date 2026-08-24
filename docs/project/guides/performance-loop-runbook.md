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

## Before the First Round

Do these once per session, in order.
Each one has caught a real mistake.

1. **Start from fresh `main`, on one branch for the night.**

   ```shell
   git fetch origin && git checkout -b claude/perf-loop-$(date +%Y-%m-%d) origin/main
   ```

   One branch, one pull request opened after the first experiment and updated after
   every one, never merged unattended.

2. **Find the queue.** The ordering is the macOS agenda in
   [the campaign-2 plan](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md);
   the beads carry it:

   ```shell
   tbd list --label macos-agenda
   ```

   Take items in the plan’s order, not by priority column, and read the bead before
   starting: its notes hold the recorded attempts and the blocker that may have moved.

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
  Leave the default (`uncontrolled`) only for screening runs, and say so in the record.
- `TRIALS=12` is the minimum for a verdict.
  A change predicted under 5% wants 16 or 20.
- The harness fingerprints the tree before and after.
  If it changed, the run exits nonzero and the numbers are not comparable: find what
  wrote to the subject, and run again.
- Content-tier hypotheses use `make perf-content-compare` with the same variables; its
  jobs are the content set.

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
- Start a Tier 3 item from the agenda: H86 (`fdu-xde5`), the `searchfs` spike, the
  FSEvents journal, hardware CRC-32C (`unsafe` or a dependency), or `fdu-n75m` parts 2
  and 3 (durability policy).
  These are listed there with the reason.
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

Before stopping:

```shell
make perf-subjects-check     # the subjects are still what the artifacts say
git status --short           # nothing uncommitted
hdiutil info | grep -c image-path   # no RAM disk left behind: expect 0
tbd sync
```

* * *

*Part of the fdu project documentation.
See [AGENTS.md](../../../AGENTS.md).*
