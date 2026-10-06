# fdu Performance Loop: Method, History, and What Remains

**Date:** 2026-08-14 (revised 2026-10-04, covering exp-000 through exp-202)

**Author:** fdu project, with Claude Code assistance

**Status:** The history of fdu’s performance loop from its first experiment to the 0.3.0
release standing. The per-round verdicts and figures are in
[the performance evidence report](report-2026-08-20-fdu-performance-evidence.md#every-round-in-full);
the next action is in
[Current Pickup](../guides/performance-loop-runbook.md#current-pickup-2026-09-30).

## Who This Is For

You need no prior context.
This report explains what fdu is, how its performance work is organized, how that method
has been run and changed over seven weeks, what it found, what it taught about itself,
and where the evidence is weak.
It is written to be handed to someone, or some agent, arriving cold.

For the condensed, domain-neutral method, read
[the instrumentation playbook](../guides/performance-instrumentation-playbook.md).
For the protocol and the live hypothesis registry, read
[the performance loop](../guides/performance-loop.md).
For every verdict and where each platform stands, read
[the evidence report](report-2026-08-20-fdu-performance-evidence.md).

## The Record in Brief

From 2026-08-10 to 2026-09-30 the loop produced **200 experiment records**, exp-000
through exp-202 (exp-113, exp-168 and exp-169 are unused ids), over 21 measurement days
and 136 hypothesis ids named in the records:

| Verdict | macOS | Linux | Total |
| --- | ---: | ---: | ---: |
| Accepted | 70 | 41 | 111 |
| Rejected | 46 | 15 | 61 |
| Baseline | 10 | 7 | 17 |
| Superseded | 5 | 0 | 5 |
| In progress | 4 | 0 | 4 |
| Blocked | 2 | 0 | 2 |
| **Total** | **137** | **63** | **200** |

An accepted verdict is not always a speed-up.
The 111 include cumulative checkpoints and transfer validations of earlier changes,
noninferiority claims, instrumentation, profiles, and leftover determinations, which are
accepted when they settle a question; 38 record no changed lines.

Where that leaves the product, each on its own host and in its own regime:

- **Linux, 4-vCPU virtualized guest, ext4, warm cache, quiet.** The 0.3.0 engine’s
  default `fdu PATH` takes 48.00% [−50.45%, −44.79%] less time than 0.2.1’s on the Linux
  v6.12 source tree, and 14% to 15% less on a directory-dense `node_modules` tree and a
  generated million-entry tree.
  On all three it leads pdu’s default, `pdu --max-depth 2` and diskus; the narrowest
  lead is +10% [+2%, +12%] over `pdu --max-depth 2` on the kernel tree
  ([exp-202](../experiments/exp-202-linux-the-0-3-0-release-end-to-end-the-default-tree-48-faste.md)).
- **macOS, one M1 Pro, APFS, warm cache, uncontrolled.** A pre-0.2.0 build built its
  reusable index and a ten-row tree on the generated million-entry tree in 6.4 s, ahead
  of every peer; dumac, which returns only a total, took 9% longer
  ([macOS comparison](report-2026-09-26-fdu-live-tool-comparison.md)). No macOS cell has
  measured anything since exp-172 (2026-09-28), so the 0.3.0 engine’s effect there is
  unknown.

## 1. What fdu Is, and What Makes It Hard to Optimize

`fdu` reports disk usage.
It walks a directory tree, collects size and metadata for every entry, and renders
views: a tree, a file list, extension tallies, a summary.
It competes with `du`, `dust`, `dut`, `pdu`, `diskus`, and `dumac`.

Four properties make its performance work harder than a typical benchmark exercise.

**It has three tiers of retained state, and they behave differently.** An *aggregate*
run keeps only running totals and discards paths.
An *index* run retains an entry per file so later views are projections rather than
rescans. A *content* run additionally reads file contents for derived metrics.
A change that helps one tier routinely does nothing for the others, so every result must
name the tier it applies to.

**It is warm as well as cold.** A cold run walks the filesystem.
A warm run loads a snapshot from a previous run and revalidates it.
These paths share code but have opposite cost profiles: cold is syscall-bound, warm is
deserialization-bound.

**It is parallel on one side and serialized on the other.** Worker threads read
directories concurrently and hand observations to a single consumer that applies them
under a delta contract, so snapshots, queries, and change feeds cannot diverge.
That serialization is a correctness feature, and on Linux, where each entry’s walk is
cheap, whatever the consumer does per entry can become the critical path.

**It reads `.gitignore` by default.** The default command classifies every entry against
the ignore rules that govern it.
A generated tree has no ignore rules, so a benchmark on one never exercises that work;
on a real repository it was, until the 2026-09-29 overnight round, most of the Linux
default command’s time (Section 4, Phase 5).

Correctness is not negotiable: every optimization must leave output byte-identical.
A faster wrong answer is not a result.

## 2. The Method

The loop’s value is not any single step but that each pass leaves the next one cheaper.

1. **Instrument**, so a run says what it *did*, not only how long it took.
2. **Profile** before forming a hypothesis.
   Read a caller tree, not a flat profile.
3. **Write the hypothesis down** in the registry, naming the metric it moves, the tier
   and platform it applies to, and a predicted range.
4. **Change one thing.**
5. **Verify identical output**, then measure paired and interleaved against the code it
   came from.
6. **Apply the accept rule** without negotiation: the median paired change at least 3%
   faster, the whole 95% bootstrap interval below zero, no invalidated sample, and the
   complexity worth it.
7. **Record the result** as a schema-validated artifact, rejections especially.
8. **Re-screen the queue**, because the change just landed may have eaten the next
   hypothesis’s headroom.

[The performance loop](../guides/performance-loop.md#the-loop) has the full protocol,
including the pre-registered-metric exception and the two tracks (single-variable tuning
and a structural composite judged as one experiment).

### Why the Accept Rule Is Strict

61 of the 200 records are rejections.
Many had a real, working mechanism: counts fell, instructions fell, allocations fell,
and wall time did not move by 3%. That is the rule doing its job.
A real mechanism is exactly what makes a small number feel worth keeping, and a codebase
that accumulates 1% wins for 50 lines each becomes unmaintainable without becoming fast.

### Why Rejections Are Recorded as Carefully as Wins

The negative results are the most reusable part of the record.
They stop the next person re-running a dead end, and several have been reused already:

- **H13** proposed accumulating roll-up contributions per parent instead of merging to
  the root per entry. Obvious, and refuted at −2.5%, because H18’s interning had already
  taken the expensive part.
- **H158, H181 and H187** cut futex calls by 83%, condvar wakes from 1,426 to 3–10 per
  run, and consumer instructions by 20–38%. None moved wall time.
  One mechanism explains all three: work removed from a thread the clock is not waiting
  on does not shorten the run.
- **The walker count** was screened four times on Linux (exp-146, exp-148, exp-149,
  exp-182) and never kept: more walkers help only when nothing else is the bottleneck.

[Dead Ends Worth Knowing](report-2026-08-20-fdu-performance-evidence.md#dead-ends-worth-knowing)
lists the closed families with the mechanism behind each.

### The Measurement Harness

Two release probes are built, a control and a candidate, and run **paired and
interleaved** (A, B, A, B) against a real tree, so drift in machine state hits both
arms. The verdict uses the median of the paired differences, never a ratio of the two
arms’ medians: when the host drifts mid-run the two disagree, and only the paired figure
controls for it.

Before timing, output equivalence is verified; in the Linux rounds since 2026-09-29 the
product command’s text, JSON, and JSONL output was byte-compared between arms on every
subject before each cell.
During timing, a **quiet gate** invalidates any sample taken while the host was busy: at
most 25% CPU busy over one second, on both platforms since 2026-09-27. A cell with any
invalid sample reads **INCONCLUSIVE** and cannot be recorded as an accept.

Performance gates are deliberately **not** in CI, because a timing gate on a shared
runner measures the runner.
What CI does run is everything that decides what a measurement means: the record schema,
each record’s consistency with its own measurements, and drift between the records and
every generated view.

## 3. Instrumentation: the Three Tiers

Instrumentation lives at three levels, and using one alone is the most common route to a
confident wrong conclusion.

| Tier | Source | Cost | Answers |
| --- | --- | --- | --- |
| Application | counters at call sites | ~1 ns/event | *which layer* did the work |
| Process | kernel, sampled per phase | one file read | what the process *really* did |
| External | `strace -c`, `perf`, callgrind | 2×–50× | ground truth, and call sites |

The application tier attributes cost to a layer, which is what tells you where to change
code, but it counts what the code *believes* it did.
The process tier is real kernel data that cannot be fooled and cannot attribute.
The external tier is authoritative and far too slow to leave on.

The mechanism lives in the
[`fdu_core::counters`](../../../crates/fdu-core/src/counters.rs) subsystem: thread-local
non-atomic storage folded into process globals, a runtime enable flag, a certified
counting global allocator, and capability-specific Linux and macOS process collectors.
Recording is off by default and enabled per run with `FDU_COUNTERS=1`, so visibility
costs an environment variable instead of a rebuild.
Its cost is measured, not asserted: compiled in and off measured −1.26% [−2.96%, +1.40%]
against no instrumentation
([exp-053](../experiments/exp-053-move-instrumentation-to-a-runtime-toggle-and-measure-all-thr.md)),
and recording on against off measured +0.64% [−0.68%, +2.13%], bounding its cost below
about 2.1% (same record).

### What the Tiers Catch That Each Other Cannot

Application counters reported 2,559 directory opens over a 17,128-entry tree.
`strace -c` on the same run reported 2,565 `openat` and 17,131 `statx`, so those
counters are sound, and **5,118 `getdents64`, exactly 2.00 per directory read**. Half of
every pair carries no data: the second call returns zero to say the directory is
exhausted. No application counter could show that, because at the call site there is one
call.

Two Linux findings came from outside the counters.
A `.gitignore`-off arm, a placebo rather than a profile, showed that the default command
spent 590 ms with ignore rules and 82 ms without (exp-173). Then a per-thread callgrind
of the H169 head attributed the remaining consumer instructions to `memcmp` and the
residual rules’ pre-checks; H183 removed most of them (436M to 238M consumer
instructions, exp-193).

### A Counter That Reads Zero Is Worse Than No Counter

A page of zeroes invites the conclusion that the work did not happen.
This failed three times while the instrumentation was being built: a counter added to
the serial walker while the parallel walker went untouched; a lint fix that hoisted a
call out of a `match` scrutinee and took the counter with it; and an entire platform
backend (`getattrlistbulk` on macOS) that replaces both `read_dir` and `stat` and
reported neither. All three compiled, passed every other test, and reported zero.
The guard is a test asserting against the system’s own totals, covering every path the
work can take, and verified by deleting a counter and watching it fail.

## 4. How the Loop Has Been Run

The loop ran in six phases.
Each changed what was being optimized, and most changed the method too, usually because
the method had just produced a wrong answer.

| Phase | Dates | Records | Platform | Accepted / rejected / other |
| --- | --- | --- | --- | --- |
| 1. Building the loop | 2026-08-10 to 08-23 | exp-000–065 | macOS, then Linux | 33 / 28 / 5 |
| 2. A denominator and a strategy | 08-23 to 08-24, 09-14 | exp-066–070, exp-104 | macOS, Linux | 4 / 1 / 1 |
| 3. Closing a rewrite’s regression | 09-01 to 09-07 | exp-071–103 | macOS, one Linux stage | 14 / 11 / 8 |
| 4. Unattended rounds | 09-19 to 09-21 | exp-105–155 | macOS, then Linux | 38 / 9 / 3 |
| 5. Release-driven questions | 09-24 to 09-28 | exp-156–174, exp-187–191 | both | 10 / 8 / 4 |
| 6. Linux against its peers | 09-29 to 09-30 | exp-175–186, exp-192–202 | Linux | 12 / 4 / 7 |

[The Loops in Order](report-2026-08-20-fdu-performance-evidence.md#the-loops-in-order)
divides the same records into 21 rounds by line of inquiry, with every verdict.

### Phase 1: Building the Loop (2026-08-10 to 2026-08-23)

**What it asked.** Where does a cold scan and a warm open spend its time on a real tree?
Campaign 1 on macOS (exp-000 to exp-039) answered with a bounded parallel producer (H1,
−50.03% `cold-scan-index`), macOS bulk metadata through `getattrlistbulk` (H3 with H26,
−30.13%), and bounded parallel reconciliation (H12 with H9, −59.53% `warm-revalidate`),
among 20 accepts. At its exp-032 checkpoint, measured against the original binary, the
cold index was 54.53% faster and warm revalidation 51.99%. An exact summary without an
index (H59) cut that job 14.56% and its peak RSS 95.28% (exp-040). The Linux campaign
(exp-051 to exp-053 and exp-060 to exp-065) then took the consumer and snapshot paths: a
one-slot parent memo (−7.35%), CRC-32C slicing-by-8 (−12.20% on its pre-registered
component), the index shared with the snapshot writer (−10.50%, RSS −35.26%), and hashed
content roll-ups (−30.31% on a generated subject, −25.78% on a dense real one).

**What it changed in the method.** The loop itself was built here: the accept rule,
paired interleaved measurement, the soft-schema record, the counters, and per-platform
tuning as a table with provenance.
Its first correction came on day two.
The harness had computed one flag, `significant`, from the accept rule and printed its
negation as “not significant”, so a metric whose interval sat entirely *above* zero read
as silence. Regenerating the ledger exposed regressions across the history that had been
reading as noise, including exp-001’s +58% CPU, and corrected a record that had called a
change “free” when its own artifact measured peak RSS up 1.5% (`22dd6543`). Evidence
direction has been reported separately from the accept rule ever since.

**What it found about direction.** mimalloc won 23% on the aggregate tier and was not
adopted: a C-building dependency for a one-tier win, with peak RSS up 139%. Its value
was diagnostic. Because changes that cut allocation *counts* were refuted while an
allocator that changes no counts won, the cost was located in glibc’s cross-thread free
path, which became H85. An adaptive-worker campaign on APFS (exp-057 to exp-059) found
every alternative controller 36–61% slower and kept the shipped one;
[the gap-closure report](report-2026-08-15-adaptive-worker-gap-closure.md) has the
policy analysis.

### Phase 2: A Denominator and a Strategy (2026-08-23 to 2026-09-14)

**What it asked.** How much is left?
The accept rule compares a candidate against yesterday’s binary, which can say a change
paid and never how much remains.
[The floor report](report-2026-08-23-metadata-walk-floor.md) measured the parallel
syscall floor per tier and subject, and
[the campaign-2 plan](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md)
anchored every priority to it, with termination criteria: a tier closes at a ×floor
threshold, or after two re-screens that name nothing worth 3%. The default command,
`fdu PATH`, was measured for the first time (exp-066), and found rewriting a 13.9 MB
snapshot it never read on every repeated run; skipping that rewrite cut it 10.61%
(exp-067).

**What it changed in the method.** Three corrections, each from a wrong answer:

- **A generated tree had inverted a ranking.** The floor report first claimed fdu beat
  ripgrep’s walker by 22% on the primary generated subject; a paired sweep across six
  trees put fdu 12–26% ahead on every generated tree, level once real filenames appear,
  and 11.8% behind on `/usr`. Accept evidence has needed a nominated real tree ever
  since, with generated trees as screens.
- **A subject’s shape had been invisible.** exp-064’s cold content win collapsed from
  −13.40% to −2.38% on a dense real tree because the sparse generated subject put the
  saving on the critical path: 0.95 of the saved CPU became wall there, against 0.29 on
  the dense tree. Each record now says how its tree was obtained and whether it can be
  rebuilt (`ef255a2a`).
- **The evidence gates had not bound.** The ledger chose its headline by searching
  titles for “cumulative” and picked a validation run, reporting the campaign as +1.4%
  against the pre-work baseline instead of exp-032’s −54.5%; and no performance-evidence
  check ran in CI. Both were fixed, and the evidence checks joined `make check`
  (`4b80fab2`).

A runbook made one round executable start to finish by an unattended agent, with
autonomy rules: no merge, no verdict from a screening subject or fewer than 12 pairs.

### Phase 3: Closing a Rewrite’s Regression (2026-09-01 to 2026-09-07)

**What it asked.** [PR #51](https://github.com/jlevy/fdu/pull/51)’s streaming engine
halved its base’s cost and was still 144% slower than the pre-rewrite `main` (exp-071,
exp-073); what closes the gap without changing an answer?
Five accepts closed it (exp-077 to exp-090), and with exp-079 the stack matched or beat
the pre-rewrite control.
The H86 structural composite (exp-091 to exp-103) then replaced the consumer
representation as one experiment under a differential oracle: fixed controls applied
once per directory cut `cold-scan-index` 33.55% with controls on, and compact child
topology cut the default tree 7.70% and RSS 37.79%.

**What it changed in the method.** The structural track ran for the first time: five
non-inferior steps judged as a composite, because per-piece 3% gates would have measured
conversion costs the end state deletes.
Its Linux evidence stage (exp-103) passed the relative gates and failed the floor gates,
2.60× the floor against 1.4×, the first time the denominator said no to a set of
accepts. And two harness fixes made the measured job match the shipped one: each
artifact’s source is verified in cross-revision comparisons, and probes measure with the
shipped control semantics enabled (`0bfb2cf8`, `1a39be9f`).

### Phase 4: Unattended Rounds (2026-09-19 to 2026-09-21)

**What it asked.** On the 0.1.0 engine, what is left in the default command and in a
warm content open, first on macOS and then on Linux?
The Darwin revisit ran 32 experiments dated 2026-09-19, most of one overnight session
(exp-105 to exp-137). Five restore cuts took `content-cache-hit` on a 146k-entry
checkout from 1,218.0 to 778.0 ms (five separate pairs, not one comparison), a sixth cut
its peak RSS 10%, and one shared walk for unfiltered views cut `content-query` 18.76%.
The Linux validation (exp-138 to exp-155) found the Darwin wins transferring (−22.48% on
the cache hit) and the floor gates still failing: the index tier 1.78× the floor against
1.4× (exp-141).

**What it changed in the method.** The **leftover determination** became a verdict type:
a profile or check that changes no code and is accepted when it names what remains.
23 of this phase’s 38 accepts are determinations.
The cost of that productivity showed up two days later: two records carried headline
figures that were cross-job ratios rather than their own paired results (exp-116 read
−99.9% where its paired figure was −2.16%), and a hand-maintained list decided which arm
the page drew as the product’s cost, and had mislabelled seven records.
A person reading caught the two headlines; no gate did.
Since `36048330` every record is validated against its own measurements, so a wrong
headline fails the build instead of regenerating cleanly into every view.

**What it got wrong, found later.** Two Linux determinations (exp-139, exp-147) read the
default command as the `getdents64` and `statx` walk and concluded the rest was the
floor.
The phase timer spanned `.gitignore` classification running on the consumer during
the walk, and no `.gitignore`-off arm was measured.
That reading stood for eight days, until Phase 5 measured the arm it lacked.

### Phase 5: Release-Driven Questions (2026-09-24 to 2026-09-28)

**What it asked.** Questions the 0.2.0 release raised: the progress indicator’s cost
(none without a handle, exp-156); multi-view report construction (H153, −47.01%,
provisional, exp-159); why fdu’s indexed tree trailed pdu and diskus on the generated
tree (H156 and H160, exp-160 and exp-163); and then why the default command took about
eight times pdu’s time on a real repository.

**The finding of the record.** On Linux v6.12, the default tree took 590 ms with
`.gitignore` and 82 ms without, in the same quiet run (exp-173). One consumer thread did
all the matching, at 78 allocations per entry.
On macOS, where the walk costs several µs per entry, the same matching stayed off the
critical path (exp-106 measured +1.6%, an interval across zero), and the generated trees
have no ignore rules.
Allocation-free matching (H162) cut the default tree 46.19%, and control chains resolved
once per listing (H163) another 35.86%: 590 to 211 ms in two changes.
The default summary also stopped building an index to classify ignore rules (H161): peak
RSS −69% on macOS, wall −6.93% on Linux.

**What it changed in the method.** A phase share cannot attribute work that runs
concurrently inside the phase, and a placebo with the suspected work turned off is the
cheap check; new determinations need a `--no-controls` arm.
The Linux quiet gate was rebuilt: it had read load average, which on Linux counts the
benchmark’s own previous samples, so every sample of the first Linux tool comparison was
invalidated.
Linux now uses the same one-second CPU-occupancy gate as macOS (`fabc850d`).

### Phase 6: Linux Against Its Peers (2026-09-29 to 2026-09-30)

**What it asked.** Can fdu’s default `fdu PATH` beat pdu’s default on two real trees,
with `.gitignore` on and no answer changed, and then every pdu mode?
One unattended night ran
[the overnight plan](../specs/active/plan-2026-09-29-linux-overnight-performance-loop.md):
bucketed `.gitignore` matching (H171, −29.62%), derived control chains (H175, −3.31%),
an exact transient tree tier (H172, −13.48%), summary walker trims (H180), a
Linux-native directory reader (H169 phase 1), and cheap matcher pre-checks (H183,
−7.62%). End to end the round cut the kernel tree’s default tree 39.00% (exp-194) and
took fdu from 2.4 times pdu’s default to level.
The pdu track the next day (exp-197 to exp-201) added H185, H186 and H188 with H189 and
put fdu ahead of both pdu modes and diskus on both real trees.
The release cell (exp-202) measured the shipped engine against 0.2.1 directly: −48.00%
on the kernel tree.

**What it changed in the method.** The night’s baselines were four-arm A/A cells, each
with two copies of the same binary.
One read −3.74% [−12.33%, −0.59%], an accept by the rule’s arithmetic with both arms
identical. From then on a candidate predicted below 10% was measured at 20 pairs or
recorded as a screen, and a placebo that excluded zero by more than 3% blocked its
verdict. A plan review found that the harness could print ACCEPT for a cell the quiet
gate had partly invalidated; it now prints INCONCLUSIVE and the recorder refuses the
accept (`fdu-c2c6`). Two more fixes closed ways a cell could measure the wrong thing: a
target directory shared across worktrees could hand a probe built from another
checkout’s sources (`fdu-8whh`), and every harness artifact is now written whole.
Finally, a baseline that compares two builds, such as an end-to-end or release cell, is
drawn with both arms; every view had read a baseline as one build against itself, so the
ledger printed exp-202’s 0.2.1 control as the whole record of a 48% gain.

**What review added.** The pull requests carrying the round were reviewed before
merging, and two findings changed measured code on correctness grounds.
H169 had put `AT_NO_AUTOMOUNT` on the parallel walk’s stats alone, so the same request
could answer differently by route on a tree holding an unmounted autofs trigger; every
route now lists through the native reader (H184, exp-196, screened for non-regression).
And H185’s identical answers held on three subjects, none of which has a directory that
lists but refuses search; the shipped form proves a directory searchable first and keeps
83% and 89% of its `statx` saving.
The per-cell answer comparison could not have found either, because neither case exists
in the subjects.

## 5. What the Loop Has Achieved

[Where Each Platform Stands](report-2026-08-20-fdu-performance-evidence.md#where-each-platform-stands)
has every standing figure with its regime.
The largest individual changes, each on its own primary job and subject:

| Change | Paired change | Job | Platform | Record |
| --- | ---: | --- | --- | --- |
| H12 with H9, bounded parallel reconciliation | −59.53% | `warm-revalidate`, 720k tree | macOS | [exp-030](../experiments/exp-030-elide-unchanged-entries-in-bounded-parallel-reconciliation-w.md) |
| H1, bounded parallel producer | −50.03% | `cold-scan-index`, 60k | macOS | [exp-001](../experiments/exp-001-bounded-parallel-directory-producer.md) |
| Point lookup for public mutation preflight | −49.78% | `delta-apply-large`, 98k | macOS | [exp-102](../experiments/exp-102-point-lookup-for-public-mutation-preflight.md) |
| H162, allocation-free `.gitignore` matching | −47.02% | default summary, `linux-v6.12` | Linux | [exp-173](../experiments/exp-173-linux-h162-allocation-free-gitignore-matching-halves-the-def.md) |
| H163, control chains once per listing | −36.43% | default summary, `linux-v6.12` | Linux | [exp-174](../experiments/exp-174-linux-h163-per-listing-control-chains-cut-another-third-from.md) |
| H53, bulk metadata during reconciliation | −34.39% | `warm-revalidate`, 720k | macOS | [exp-026](../experiments/exp-026-reuse-macos-bulk-metadata-during-full-reconciliation.md) |
| H86, fixed controls once per directory | −33.55% | `cold-scan-index`, controls on | macOS | [exp-096](../experiments/exp-096-apply-fixed-controls-once-per-detached-directory.md) |
| H102, byte-ordered content file map | −31.00% | `content-cache-hit` | macOS | [exp-069](../experiments/exp-069-order-the-content-file-map-by-path-bytes-instead-of-componen.md) |
| H94 with H95, hashed content roll-ups | −30.31%, −25.78% real | `content-cache-hit` | Linux | [exp-064](../experiments/exp-064-content-roll-up-lookup-and-indexed-type-rule-tiers.md), [exp-065](../experiments/exp-065-validate-the-content-roll-up-change-on-a-dense-real-tree.md) |
| H3 with H26, `getattrlistbulk` | −30.13% | `cold-scan-index`, 720k | macOS | [exp-022](../experiments/exp-022-batch-macos-scan-metadata-with-getattrlistbulk.md) |
| H171, bucketed `.gitignore` matching | −29.62% | default tree, `linux-v6.12` | Linux | [exp-178](../experiments/exp-178-linux-h171-bucketed-gitignore-matching-cuts-the-default-tree.md) |
| H59, exact summary without an index | −14.56%, RSS −95.28% | summary, 978k | macOS | [exp-040](../experiments/exp-040-derive-an-exact-rich-summary-without-building-an-index.md) |
| H161, ignore-aware summary | RSS −69.12% | default summary, 137k | macOS | [exp-170](../experiments/exp-170-macos-ignore-aware-transient-summary-cuts-default-summary-pe.md) |

The end-to-end cells are the comparisons to quote, because each step above was measured
against the head before it, in its own session:

| Cell | What it compares | Headline change |
| --- | --- | --- |
| exp-032, macOS | campaign 1’s code against the original binary | `cold-scan-index` −54.53%, 635 to 290 ms on the 60k checkout |
| exp-194, Linux | the overnight round’s head against 0.2.1 | default tree −39.00%, 200.3 to 119.8 ms on `linux-v6.12` |
| exp-201, Linux | the pdu track against the overnight head | default tree −3.05% and −8.94% on the two real trees |
| exp-202, Linux | the 0.3.0 release against 0.2.1 | default tree −48.00%, −14.09% and −14.57% on the kernel, dense and generated trees |

No later cell repeats exp-032’s comparison with the original binary, so it describes
campaign 1, not the current engine.

## 6. What the Loop Taught About Itself

Each of these was learned by the loop producing a wrong or misleading answer first.

**The instrument needs as much scrutiny as the engine.** Regressions printed as silence
(Phase 1); a headline selected from the wrong artifact (Phase 2); records whose headline
was not their own measurement (Phase 4); a tool-comparison quiet gate that invalidated
every Linux sample (Phase 5); an A/A cell that passed the accept rule, and a verdict
that could print ACCEPT for an invalidated cell (Phase 6); and probes built from another
checkout’s sources (`fdu-8whh`, found 2026-09-13, fixed 2026-09-30). Each passed its
tests at the time, and most were found by a person or reviewer reading output rather
than by a gate. Each now has a gate, and the record schema, self-consistency, and
view-drift checks run in CI.

**The measured job has to be the shipped command.** The default command went unmeasured
for the first 66 records; `aggregate-summary` measured the full-index plan under a
transient label from [#65](https://github.com/jlevy/fdu/pull/65) until a re-measurement
caught it and the guide was corrected on 2026-09-16 (`fdu-hkyh`); and no Linux benchmark
measured a `.gitignore`-off arm on a real repository until 2026-09-28 (exp-173), which
hid the largest Linux cost in the record.

**Generated trees screen; real trees decide.** A generated tree inverted a walker
ranking (Phase 2), flattered a cold content win more than fivefold (exp-064 against
exp-065), and holds no ignore rules.
The charted page counts 7 of 28 Linux improvements as decided on generated trees, none
of them since 2026-09-29.

**Cutting work only pays where the clock waits for it.** With `.gitignore` on, the Linux
default tree kept two of four vCPUs busy and wall followed the consumer; with it off,
wall followed total CPU divided by about 3.4. That one observation ordered the overnight
queue, and it explains why H157, H158, H181, H182 and H187 cut real work and moved
nothing. Instruction counts are evidence about a thread, not about wall time.

**Only within-cell ratios are evidence across sessions.** Over one night on one
virtualized host, an unchanged binary’s absolute time drifted 50–70% between cells.
Every standing figure is therefore a paired change inside one cell, and the report says
so when a figure chains cells.

**A result is evidence about its platform.** The `.gitignore` cost was invisible on
macOS because the macOS walk is slower per entry; H159 saves per directory, so it
appeared on a dense tree and vanished on a sparse one; H156 and H160, clear on Linux,
moved macOS wall by less than the host could resolve.
[The platform tuning guide](../guides/platform-tuning.md) records which regime each
shipped constant was measured in.

**Unattended rounds work when the rules are mechanical.** An agent ran 32 experiments in
one day on macOS, most of them overnight, and 16 in one night on Linux, recorded
rejections as carefully as accepts, and stopped where the runbook said to.
What it could not do was notice a defect in the instrument it was measuring with; those
were found afterwards by people and reviewers, which is why the pull requests that carry
a round are reviewed before they merge.

## 7. Platform Status

|  | macOS | Linux | Windows |
| --- | --- | --- | --- |
| Records | 137 (last: exp-172, 2026-09-28) | 63 (last: exp-202, 2026-09-30) | 0 |
| Host | one M1 Pro, bare metal, APFS | 4-vCPU KVM and Firecracker guests, ext4 | none |
| Regime | uncontrolled since 0.1.0 | mostly quiet since 2026-09-20 | none |
| Directory read | `getattrlistbulk` | native `getdents64` reader with `statx` (H169) | `read_dir` |
| Profiler | `/usr/bin/sample`, driven by the harness | callgrind, `strace` | none |
| Instruction-count gating | unavailable (no Valgrind on Apple Silicon) | possible | unavailable |
| Floor denominator | none | stale since exp-141 | none |

**macOS has the larger record and the older one.** Its 137 records built the walker, the
bulk reader, the snapshot path, and the content cache; none since 0.1.0 held the quiet
gate for a whole cell, and none measures the 2026-09-29 round, the pdu track, or the
release.
Those changes are portable code except H169’s reader, which is Linux glibc only,
and H185, which is a no-op on macOS.

**Linux has the recent record.** Every Linux cell since 2026-09-20 ran on a 4-vCPU
virtualized guest, most of them quiet; every 0.3.0 speed claim is Linux evidence.

**A constant measured on one platform is inherited, not proven, on the other.** Tuning
is a table with a per-entry provenance marker, and a parity test sweeps every platform’s
table on every CI platform.

## 8. Where the Evidence Is Weak

[Qualifications on Current Results](report-2026-08-20-fdu-performance-evidence.md#qualifications-on-current-results)
lists the qualification on each standing verdict.
The structural gaps are these:

- **Cold cache.** Every run is warm-steady; dropping the page cache needs root, and
  nothing in the record describes a first read from disk.
- **Bare-metal Linux.** Every Linux host is virtualized, and a hypervisor’s page cache
  sits under the guest’s, so device-latency and I/O-ordering hypotheses (H28, H73)
  remain untestable.
- **One host per platform.** macOS is one M1 Pro; Linux is 4-vCPU Xeon guests.
  The walker-count screens are evidence about four vCPUs, not wider hosts.
- **macOS since 2026-09-28**, and **Windows** at all.
- **The release cell cannot be chained to the development cells.** exp-202 ran in a
  session 29–38% slower than exp-201’s and carries no exp-201 engine.
  H184 was screened alone on the #161 head (exp-196) but not on exp-201’s engine, the
  R163-1 latch and the stability layer were never timed alone, and part of the release’s
  lead on the generated tree is the session’s.
- **Provisional and in-progress verdicts.** H153’s −47.01% awaits a quiet confirmation;
  H70, H151 and the exp-097 lifecycle audit are in progress.
- **The floor scoreboard is stale.** It was last derived on 2026-09-20 (exp-141), before
  eighteen accepted hypotheses.
- **Two hypotheses can compete for one cost.** H13 lost to H18, H74 to the snapshot
  loader fix, H89 to H86; any queued estimate is an upper bound until re-screened.

## 9. What Remains

[Open Work](report-2026-08-20-fdu-performance-evidence.md#open-work) has the full list
and [Current Pickup](../guides/performance-loop-runbook.md#current-pickup-2026-09-30)
the order. In brief:

- **On Linux**, walker-side cuts come first, because exp-200 showed the tree route’s
  consumer has slack: H169 phase 3 (directories opened relative to the parent’s
  descriptor) and H177 (a per-listing name arena), then H178, then the full
  generated-tree peer table on the shipped engine.
- **On macOS**, measure H162 and H163 (`fdu-dv07`), the 2026-09-29 round, the pdu track,
  and the release engine (the platform review’s cells, exp-203 onward).
- **For the record**, re-derive the Linux ×floor scoreboard (`fdu-z2h6`), confirm H153
  on a quiet host (`fdu-9e9d`), and close the regime gaps: bare metal (`fdu-lf3v`), a
  quiet-host peer cell (`fdu-ow8y`), and a macOS floor (`fdu-9hdc`).

[Campaign 2](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md) defined how
a tier closes, and no tier has been recorded as closed:
[Campaign 2 Termination](report-2026-08-20-fdu-performance-evidence.md#campaign-2-termination)
has each tier’s status.
The `.gitignore`-on default command, which held the largest gap, was never a campaign-2
tier; after the overnight round the kernel tree’s default tree was 1.6% above its
`.gitignore`-off arm in exp-194’s cell and 8.6% in exp-193’s, both ratios of medians.

## 10. Open Questions About the Method

**Can a regression gate exist at all?** A timing gate on a shared runner measures the
runner. Instruction counts are deterministic, and `iai-callgrind` could gate them, but
Valgrind does not instrument kernel code, which is most of the Linux walk’s time, and
Phase 6 showed instruction cuts that moved no wall (H182, H187). A partial gate honestly
labelled may still be worth having (`fdu-slgp`).

**How should an unattended round find defects in its own instrument?** Every harness
defect in Section 6 passed its tests and was found by reading.
The overnight round’s same-binary A/A cells caught one class, and review caught others;
whether something cheaper and routine would catch the rest is open.

**When does a campaign end?** The termination rule exists and has never fired, partly
because the floor it reads has not been re-derived since exp-141.

**Can the loop be reused elsewhere?** Extracting it as a framework is `fdu-7yx4`;
[the instrumentation playbook](../guides/performance-instrumentation-playbook.md) is the
domain-neutral half already written.

## 11. How to Reproduce Any of This

[Running It](../guides/performance-loop.md#running-it) has the commands to record a
subject (`make perf-baseline`), profile (`make perf-profile`), and measure
(`make perf-compare CONTROL=... JOBS=... TRIALS=...`);
[Publishing the Evidence](../guides/performance-loop.md#publishing-the-evidence) has
`make perf-record`, `make perf-ledger`, and `make perf-report`.
[The runbook](../guides/performance-loop-runbook.md) is one round of that, start to
finish, with the rules an unattended agent follows.

## 12. Document Map

| Document | What it is |
| --- | --- |
| This report | How the loop works, how it has been run, and what it taught |
| [Performance evidence](report-2026-08-20-fdu-performance-evidence.md) | The full record: every round from exp-000 to exp-202, and where each platform stands |
| [Charted page](performance-evidence/index.html) | Absolute timings and paired effects across every experiment, generated by `make perf-report` |
| [Experiment ledger](report-2026-08-10-fdu-performance-experiments.md) | Every experiment in full, generated by `make perf-ledger` |
| [Performance loop](../guides/performance-loop.md) | Protocol and live hypothesis registry |
| [Performance-loop runbook](../guides/performance-loop-runbook.md) | One round, the current pickup, and standing host context |
| [Instrumentation playbook](../guides/performance-instrumentation-playbook.md) | The reusable method, domain-neutral |
| [Platform tuning](../guides/platform-tuning.md) | Which constants were measured where |
| [Metadata-walk floor](report-2026-08-23-metadata-walk-floor.md) | The measured floor for this workload, and every tier and peer read against it |
| [Campaign-2 plan](../specs/active/plan-2026-08-23-fdu-performance-campaign-2.md) | Floor-anchored priorities and termination |
| [macOS](report-2026-09-26-fdu-live-tool-comparison.md) and [Linux](report-2026-09-27-fdu-linux-tool-comparison.md) tool comparisons | fdu against its peers |
| [Performance architecture](report-2026-08-12-fdu-performance-architecture.md) | Cost model and architectural conclusions |
| [Structural review](../research/research-2026-08-14-structural-performance-review.md) | What 30 hypotheses had in common, and missed |

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
