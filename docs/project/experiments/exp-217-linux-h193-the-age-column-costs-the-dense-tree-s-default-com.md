---
title: "Linux: H193, the age column costs the dense tree's default command 5.7%, one statx per directory"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-217
  title: "Linux: H193, the age column costs the dense tree's default command 5.7%, one statx per directory"
  date: "2026-10-10"
  hypotheses:
    - H193
  subject:
    tree_label: node-modules-dense
    tree_root_id: ffbd3cb6be62b77f7e2e9b5964d1e380e8a3f3b7bb4fda55f462ae4b189841b5
    tree_engine_digest: 62125c60d0dddf9fdb9f89b8c2542c8afd573f3460ccc6f052626099669b7751
    tree_provenance: "npm dependency trees of react-scripts 5.0.1 and gatsby 5.13.7: copy docs/project/experiments/evidence/exp-190/subject-package.json.txt and subject-package-lock.json.txt to package.json and package-lock.json in an empty directory and run npm ci --ignore-scripts (node 22.22.0, npm 10.9.4); rebuilt 2026-10-10 on this host: 79,957 entries, 9,439 directories, 107 symlinks, no .gitignore, the counts exp-197 recorded."
    tree_reconstructible: true
    tree_entries: 79957
    tree_directories: 9439
    tree_files: 70411
    tree_symlinks: 107
    tree_apparent_bytes: 538992241
    tree_allocated_bytes: 757870592
    tree_max_depth: 12
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.10GHz"
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16874930176
    host_system: Linux 6.18.44-fc-v114
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 100
    warmups: 3
    interleaved: true
    control: 148ef78e probe (v0.4.0)
    candidate: "0d6011da probe (the 0.5.0 stack's head)"
    control_binary:
      name: v040
      sha256: d84fbd9ce11be5c2b653163bbef2b2747a1eb2cebcaf34d292f600dcd2fa00ab
      size_bytes: 3947240
      args: []
    candidate_binary:
      name: roots
      sha256: d93d5a4361b6717accb93f52eed531936ca1d1f513555484d2f4c4658086c7a0
      size_bytes: 4266000
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-217/run.json.gz
  results:
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 94349087.0
          candidate_median: 101907421.5
          control_p95_over_median: 1.154
          candidate_p95_over_median: 1.146
          change_pct: 5.723
          ci95_low_pct: 3.526
          ci95_high_pct: 10.464
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 100
        component_ns:
          control_median: 91193171.0
          candidate_median: 98313975.5
          control_p95_over_median: 1.15
          candidate_p95_over_median: 1.145
          change_pct: 5.905
          ci95_low_pct: 3.556
          ci95_high_pct: 10.483
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 100
        cpu_ns:
          control_median: 339048000.0
          candidate_median: 354265500.0
          control_p95_over_median: 1.145
          candidate_p95_over_median: 1.124
          change_pct: 4.482
          ci95_low_pct: 2.436
          ci95_high_pct: 7.198
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 100
        user_cpu_ns:
          control_median: 58779000.0
          candidate_median: 59361500.0
          control_p95_over_median: 1.441
          candidate_p95_over_median: 1.506
          change_pct: 2.687
          ci95_low_pct: -5.519
          ci95_high_pct: 10.231
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 100
        system_cpu_ns:
          control_median: 280306500.0
          candidate_median: 297199500.0
          control_p95_over_median: 1.196
          candidate_p95_over_median: 1.129
          change_pct: 3.869
          ci95_low_pct: 0.691
          ci95_high_pct: 7.442
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 100
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - peak_rss_bytes is missing a paired percent interval
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: within-limit
          major_faults: within-limit
          minor_faults: within-limit
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: rejected
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: 5.723
    reason: "not a speed decision: the age column ships regardless; default-tree on node-modules-dense +5.72% [+3.53%, +10.46%] at 100 pairs against +2% to +4% predicted, all of it the column (+5.63% against the no-ages arm, exp-218); statx +7,887, +5,181 and +116,391 against +7,886, +5,180 and about +125k; placebo aggregate-summary --no-controls includes zero; callgrind cold-scan-index +0.07%, warm-snapshot-load +0.30%, opened-discovery -0.002%, inside their bounds; peak RSS +0.2 MiB on the generated tree against +0.75 to +2 MB; the generated-tree screen +10.40% [+4.20%, +11.86%] from a middle slot; tool cells: no resolved lead over pdu --max-depth 2 or diskus on the real trees this session, where 0.4.0 is level too; tree tier 1.21x to 1.26x floor, screening-grade"
    commit: 0d6011da
    kept: candidate
---
## What was predicted

H193, pre-registered on 2026-10-09 before any timed sample (review C1 on
[#191](https://github.com/jlevy/fdu/pull/191), `fdu-088k`, the 0.5.0 release gate):
route-equal ages make the folded one-shot tree state every directory and symlink again
(`1b3ac793`), giving back H185 (exp-197). Predicted on the v0.4.0 control against the
0.5.0 candidate: `default-tree` wall +2% to +4% on `node-modules-dense` (deciding, about
3%), 0 to +2% on `linux-v6.12`, +3.5% to +5.5% on the `linux-balanced-1m` screen with
peak RSS +0.75 to +2 MB; `statx` up by +7,886, +5,180, and about +125k, and unchanged on
`aggregate-summary`; the placebo `aggregate-summary --no-controls` on
`node-modules-dense` includes zero; callgrind instructions for `cold-scan-index`,
`warm-snapshot-load`, and `opened-discovery` below +0.5%, +1%, and +1%; the tool cells’
leads shrink by the give-back.
Amended twice before the cells it changed ran: a measurement-only no-ages arm
(`f63af680`, exp-218), and, once the first two four-arm cells resolved only ±8%, the
deciding pair replicated in a two-arm cell of 100 pairs (`8e23f2df`).

## What was measured

Host: a 4-vCPU Firecracker guest (Intel Xeon at 2.1 GHz, Linux 6.18.44-fc-v114, ext4),
as root, warm page cache.
The subjects were rebuilt for this session and match their records: `linux-v6.12`
(92,474 entries, 5,769 directories, 62 symlinks, 358 `.gitignore`), `node-modules-dense`
(79,957 entries, 9,439 directories, 107 symlinks), and `linux-balanced-1m` (1,000,001
entries, semantic digest `4bbd97c0`). Every cell ran under the quiet regime: it started
only after one-second CPU busy had stayed under 15% and load per core under 0.25 for ten
seconds, and the harness held CPU busy under 25% before and after every sample.
Binaries were built in clean worktrees, one per commit, and copied out of them: v0.4.0
is `148ef78e` (probe sha256 `d84fbd9c`), the age column’s head `23d11b19` (`683bf0cb`),
the shipped head `0d6011da` (`d93d5a43`), and the no-ages arm `61968412` (`6a447e7d`).

**The deciding pair** (this record’s run: two arms, 3 warmups, 100 pairs fixed in
advance, no invalid sample; a first attempt lost one sample to the quiet gate and was
rerun whole): `default-tree` on `node-modules-dense`, the shipped head against v0.4.0,
**+5.72% [+3.53%, +10.46%]**. Against the same code without the directory `statx` the
shipped head is +5.63% [+4.07%, +9.27%] (exp-218), so the age column is the whole of it.

**The four-arm cells** (v0.4.0, `23d11b19`, `0d6011da`, no-ages; 20 pairs; the harness
alternates each ordinal’s order, so the two middle arms never run first or last):

| Subject | Shipped head against v0.4.0 | No-ages arm against v0.4.0 |
| --- | --- | --- |
| `node-modules-dense` | +5.13% [−5.76%, +12.67%] | +4.19% [−4.16%, +8.42%] |
| `linux-v6.12` | −5.52% [−9.18%, +0.12%] | −5.76% [−13.22%, −0.22%] |
| `linux-balanced-1m` (screen) | **+10.40% [+4.20%, +11.86%]** | +2.40% [−0.09%, +4.82%] |

Twenty pairs resolve about ±8% here, too wide for the prediction on the real trees.
An A/A cell of one binary under three names, `a`, `b`, and `c` (`node-modules-dense`, 20
pairs), compared the middle slot with the outer ones.
Arm `a` ran first in 10 ordinals and last in the other 10, as `c` did, because the
harness reverses each ordinal’s order; `b` was always in the middle.
The middle slot was +3.48% [−2.17%, +11.95%] against `a` on `default-tree` and +2.66%
[−2.58%, +5.98%] on `render-yaml`, and `c` was −4.60% [−10.56%, +2.78%] and −2.49%
[−4.78%, +3.49%] against the middle, while the two outer arms differ by −1.63%
[−4.71%, +3.81%] and −0.22% [−2.68%, +2.51%]. That is unresolved, but it is the size of
the effects being read, so a middle arm’s figure in these cells is read only beside its
two-arm replicate.
On the screen the shipped head ran in a middle slot; its +10.40% is an
upper reading, and H197’s two-arm cells bound the column’s price there at +1.6% to +4%
(exp-219).

**Placebo** (`--no-controls`, three arms, 20 pairs, the shipped head in the middle
slot): `aggregate-summary` +2.35% [−0.61%, +5.20%] against v0.4.0, including zero.

**System calls** (`strace -f -c` of the probe’s `default-tree`): `statx` +7,887 on
`node-modules-dense` (79,966 against 72,079), +5,181 on `linux-v6.12` (92,841 against
87,660), and +116,391 on `linux-balanced-1m` (1,000,010 against 883,619), against
+7,886, +5,180, and about +125k predicted; the generated tree’s shortfall is the H185
policy’s own stats, one per listing until a child proves it searchable.
`openat`, `getdents64`, and `close` are equal to within one in every arm: v0.4.0 makes
one more `openat` and `close` on `node-modules-dense` (9,448 against 9,447), and the
no-ages arm’s `statx` equals v0.4.0’s. `futex`, the walker threads’ waits, differs by a
few percent between arms.
`aggregate-summary` makes 72,076 `statx` on both builds on `node-modules-dense` and
87,658 against 87,659 on `linux-v6.12`.

**Instructions** (callgrind, user space, one run each, `node-modules-dense`):
`cold-scan-index` 363.35M to 363.62M (+0.07%, predicted below +0.5%),
`warm-snapshot-load` 606.27M to 608.12M (+0.30%, below +1%), `opened-discovery`
8,481.59M to 8,481.42M (−0.002%, below +1%), `aggregate-summary` 141.73M to 141.71M
(−0.02%), and `default-tree` 212.14M to 214.13M (+0.94%), of which +0.26% is the age
column (the no-ages arm runs 213.57M). Whole-process counts move a few million between
runs because the walker pool scales differently under valgrind; the consumer thread,
which runs alone, moves +1.2% (103.37M to 104.63M).

**Peak RSS** (`default-tree`, a fork-and-exec launcher reading `wait4`, 7 runs each,
median): `node-modules-dense` 12.0 MiB at v0.4.0, 11.7 at the age head, 12.3 at the
shipped head; `linux-v6.12` 10.4, 10.2, 11.0; `linux-balanced-1m` 57.3, 57.2, 57.5 MiB
(+0.2 MiB against +0.75 to +2 MB predicted).
The age column adds none; the shipped head’s few hundred KiB arrive with #192, whose
probe text is 287 KB larger, which is plausibly the difference and not shown to be.
The harness withholds peak RSS below its own 40 MiB, so these are from a separate
launcher, not the timed cells; the cells’ figure on the generated tree is +0.35%
[+0.07%, +0.57%].

**Machine formats** (`render-yaml`, the render alone, untimed report construction
excluded; three arms, 20 pairs, v0.4.0 and the shipped head both outer arms): +17.69%
[+12.49%, +23.27%] on `node-modules-dense` and +15.43% [+12.82%, +22.61%] on
`linux-v6.12`, about 0.14 µs a row over roughly 150,000 tree and file rows, the cost
exp-213 measured on macOS (0.12 µs a row).
Wall, which includes the walk, +1.34% and +2.73%, neither resolved.

**Retained reports** (the second tree report, timed alone; same three-arm cells, both
compared arms outer): over a retained `Index` on `node-modules-dense` +5.75%
[+1.94%, +9.86%] in component, 0.52 to 0.55 ms, the per-row age work exp-212 recorded on
macOS; on `linux-v6.12` +2.45% [−7.30%, +8.96%]. Over an opened root +8.56%
[−8.23%, +16.48%] and +6.20% [−19.41%, +35.73%], unresolved.
Wall, dominated by the walk, is flat.

**The summary** (`aggregate-summary` wall, the same three-arm cells
`run-b1-nmd-retained` and `run-b2-lnx-retained`, both compared arms outer): the shipped
head against v0.4.0 is +5.45% [+1.99%, +7.20%] on `node-modules-dense` and −1.47%
[−5.61%, +6.98%] on `linux-v6.12`. The age head, from the middle slot, is +1.53%
[−0.64%, +5.01%] and +2.58% [−5.98%, +7.63%] against v0.4.0, unresolved.
The summary makes the same calls at v0.4.0 and at the shipped head (the counts above),
so the dense tree’s slower summary is read with H195’s two-arm rerun, exp-220.

**Answers.** On all three subjects the 0.5.0 command line’s folded default tree equals
the full index (`--cache on` into an empty cache directory) on every row’s name, kind,
sizes, counts, newest file, activity time, and completeness, and 0.4.0’s on every size
and count: 79,850, 92,412, and 8,675 rows (`--full`; depth 4 at every breadth on the
generated tree).

**Tool cells** (the 0.5.0 command line, `--color never PATH`, the `fdu-default-tree`
contract, as the anchor; each competitor paired 20 times with the adjacent anchor run in
one interleaved run per tree, 3 warmups, quiet; positive means the competitor took
longer): run after a container restart on the same host, with the subjects re-warmed
before the first cell.

| Subject | 0.5.0 `fdu` | 0.4.0 `fdu` | pdu `--max-depth 2` | pdu default | diskus |
| --- | ---: | ---: | ---: | ---: | ---: |
| `node-modules-dense` | 0.13 s | −6% [−11%, +0%] | +2% [−9%, +5%] | +11% [−1%, +17%] | −2% [−5%, +2%] |
| `linux-v6.12` | 0.13 s | −3% [−6%, +2%] | −2% [−6%, +2%] | +4% [−0%, +6%] | +4% [−2%, +9%] |
| `linux-balanced-1m` | 1.2 s | −3% [−11%, −1%] | **+4% [+2%, +6%]** | **+12% [+9%, +15%]** | **+12% [+6%, +15%]** |

On the generated tree 0.5.0 still leads every peer.
On the two real trees it no longer holds a lead whose interval excludes zero over pdu
`--max-depth 2` or diskus, where exp-202 put the 0.3.0 release ahead of both with lower
bounds of +1.8% and +1.9% over pdu `--max-depth 2`. That is not the age column alone:
0.4.0, which takes directory kinds from the listing on this route (H185), is level with
the same peers in this session too, and every tool ran slower than in exp-202’s (0.13 s
here for fdu against 0.106 s and 0.110 s there), on a newer build of the guest kernel
(fc-v114 against fc-v50). Read across sessions, the published standing on the real trees
is no longer supported on this host; read within this session, 0.5.0 trails 0.4.0 by 3%
to 6% on the command line, the price recorded above.

**The floor** (`make perf-floor`, with the tree tier this branch adds to the scoreboard
(`d6302e8d`): every instrument at a fixed pool of 4 workers, 30 trials and 3 warmups,
`.gitignore` off for fdu’s tiers as the floor reads none; the probe is the shipped
head’s). One trial of 540 breached the quiet gate, so the harness downgraded the regime
and the table is screening-grade; it is committed beside this record as
`scoreboard-2026-10-10-linux-tree-floor`.

| Subject | Floor (`parfloor stat`) | Aggregate | Tree | Index |
| --- | ---: | ---: | ---: | ---: |
| `node-modules-dense` | 97.3 ms | 1.14× | **1.23×** | 1.35× |
| `linux-v6.12` | 91.6 ms | 1.13× | **1.26×** | 1.40× |
| `linux-balanced-1m` | 1,133 ms | 0.97× | **1.21×** | 1.25× |

The tree tier now describes each directory once, as the floor does, and sits 21% to 26%
above it; the floor’s own `statx` per entry is what H185 had taken the tree below.

**Python** (`fdu-fy0l`; the 0.5.0 wheel and the 0.4.0 wheel, each built from its commit
and installed into its own Python 3.12 environment; a 100,000-row list report,
`Query(views=(View.LIST,), format=Format.JSON, selection=Selection(limit=100000))`, on
`linux-balanced-1m`; quiet; 20 pairs each): converting the report’s JSON to models takes
1,298 ms with each row’s `modified_at` derived and 1,142 ms with the derivation stubbed
out, a paired median of 145 ms, about 1.45 µs a row (the pairs range from −176 to +318
ms). The whole `fdu.report` call takes 10.6 s at 0.5.0 against 11.7 s at 0.4.0 (−7.1%
paired median), so the derivation is about 1.4% of the call.
Most of the call is native: the conversion to models is about 1.3 s of the 10.6 s, which
leaves about 9.3 s in the native call, against 1.2 s for the 0.5.0 command line’s
default tree over the same tree (the tool cells above).
The command-line and native-call timings of the list that an earlier version of this
record quoted have no committed evidence, so they are withdrawn; `fdu-yp6l` rests on the
figures above.

`python-modified-at.json` cannot tell the two wheels apart by itself.
It reports version 0.4.0 for both, because the version is bumped only by the release
commit: `148ef78e` and `0d6011da` both declare 0.4.0. No commit or wheel hash was
recorded when it ran, so which wheel each end-to-end arm ran rests on this record’s
account. The component cell needs the 0.5.0 build: it stubs `fdu._models._modified_at`,
which 0.4.0 does not have.

## Decision

Not a speed decision: the age column ships regardless, and this is the release gate’s
measurement of its price.
On the deciding tree the default command takes 5.72% longer than 0.4.0’s
[+3.53%, +10.46%], about twice the +2% to +4% predicted, and all of it is the column:
one `statx` per directory and per symlink, exactly the counts predicted, and kernel
time, since user-space instructions move 0.26% for it.
On the directory-sparse kernel tree the price is inside noise.
Every placebo and instruction bound holds, peak RSS moves less than predicted, and
beyond the walk the column costs per row, never per entry: about 30 µs on a second
report over a retained index of this tree, and about 0.14 µs a row in the machine
formats. Recorded `rejected` as a speed claim with the shipped head kept.
H197 (exp-219) wins back about three points of it on the dense tree.
