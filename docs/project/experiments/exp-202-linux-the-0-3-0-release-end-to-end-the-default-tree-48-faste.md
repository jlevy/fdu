---
title: "Linux: the 0.3.0 release end to end, the default tree 48% faster than 0.2.1 and ahead of pdu and diskus on all three trees"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-202
  title: "Linux: the 0.3.0 release end to end, the default tree 48% faster than 0.2.1 and ahead of pdu and diskus on all three trees"
  date: "2026-09-30"
  hypotheses: []
  subject:
    tree_label: linux-v6.12
    tree_root_id: 4ffe9d749fe638cd6c746668f7ae66d88e50225fa6903eabd8c4b9405b1aa245
    tree_engine_digest: e9ca9ca9713695a9fffa9a2937e066f4f584c03c2aa921257dee9db56102b05b
    tree_provenance: "Shallow clone of github.com/torvalds/linux tag v6.12 at adc218676eef25575469234709c2d87185ca223a. Reconstructible: git clone --depth 1 --branch v6.12 https://github.com/torvalds/linux.git. Shape is the published v6.12 source tree plus the clone's .git directory as git left it; no extra workspace install."
    tree_reconstructible: true
    tree_entries: 92474
    tree_directories: 5769
    tree_files: 86643
    tree_symlinks: 62
    tree_apparent_bytes: 1759293209
    tree_allocated_bytes: 1965477888
    tree_max_depth: 14
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.10GHz"
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16876515328
    host_system: Linux 6.18.44-fc-v50
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 20
    warmups: 3
    interleaved: true
    control: c1644575 probe (v0.2.1)
    candidate: b82f26e1 probe (the 0.3.0 release head)
    control_binary:
      name: v021
      sha256: 5272a36eeb5673151e9c761178dc6a80d7dd625695662d256024ef31b6affdc6
      size_bytes: 3758768
      args: []
    candidate_binary:
      name: release
      sha256: ec8b40fe5e8f581e696e6996985c67aafdf9bbe7ff2478009c9095fb41dcc283
      size_bytes: 3933176
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-202/run.json.gz
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 184632480.0
          candidate_median: 128134218.5
          control_p95_over_median: 1.36
          candidate_p95_over_median: 1.337
          change_pct: -34.635
          ci95_low_pct: -40.366
          ci95_high_pct: -27.19
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 180954409.0
          candidate_median: 125252577.5
          control_p95_over_median: 1.366
          candidate_p95_over_median: 1.338
          change_pct: -35.171
          ci95_low_pct: -40.606
          ci95_high_pct: -27.495
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 626401500.0
          candidate_median: 471867500.0
          control_p95_over_median: 1.346
          candidate_p95_over_median: 1.377
          change_pct: -27.626
          ci95_low_pct: -30.434
          ci95_high_pct: -23.891
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 266899000.0
          candidate_median: 83458000.0
          control_p95_over_median: 1.355
          candidate_p95_over_median: 1.517
          change_pct: -68.526
          ci95_low_pct: -73.474
          ci95_high_pct: -65.374
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 365694500.0
          candidate_median: 390801500.0
          control_p95_over_median: 1.346
          candidate_p95_over_median: 1.377
          change_pct: 1.708
          ci95_low_pct: -4.274
          ci95_high_pct: 9.135
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - peak_rss_bytes is missing a paired percent interval
          - "involuntary_context_switches exceeds its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: rejected
          major_faults: within-limit
          minor_faults: within-limit
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 208639416.5
          candidate_median: 109873005.5
          control_p95_over_median: 1.119
          candidate_p95_over_median: 1.097
          change_pct: -48.0
          ci95_low_pct: -50.453
          ci95_high_pct: -44.794
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 203674777.0
          candidate_median: 106792550.5
          control_p95_over_median: 1.12
          candidate_p95_over_median: 1.099
          change_pct: -48.134
          ci95_low_pct: -50.84
          ci95_high_pct: -45.149
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 585250500.0
          candidate_median: 406522000.0
          control_p95_over_median: 1.14
          candidate_p95_over_median: 1.079
          change_pct: -29.108
          ci95_low_pct: -33.358
          ci95_high_pct: -27.748
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 232650000.0
          candidate_median: 57670500.0
          control_p95_over_median: 1.171
          candidate_p95_over_median: 1.434
          change_pct: -75.148
          ci95_low_pct: -79.12
          ci95_high_pct: -72.483
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 352470000.0
          candidate_median: 343703500.0
          control_p95_over_median: 1.129
          candidate_p95_over_median: 1.114
          change_pct: -1.561
          ci95_low_pct: -5.537
          ci95_high_pct: 3.26
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - peak_rss_bytes is missing a paired percent interval
          - "voluntary_context_switches exceeds its +50% regression limit"
          - "involuntary_context_switches exceeds its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: rejected
          major_faults: within-limit
          minor_faults: within-limit
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: rejected
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
    decision: baseline
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -48.0
    reason: "The release engine end to end on every subject this host has: against 0.2.1 in one paired probe cell per tree, and against pdu default, pdu --max-depth 2, diskus and the 0.2.1 CLI in one interleaved tool cell per tree with the release CLI as anchor; no decision rests on it."
    commit: b82f26e1
    kept: neither
---
## What was predicted

The last measurement before the 0.3.0 release: the release engine, end to end, against
0.2.1 and against the peers, on every subject this host has.
It is a baseline record, and no decision rests on it.
Its purpose is that the release notes state what the shipped engine does, rather than
what the development heads before it did.

- **Control:** 0.2.1, `c1644575` (the `v0.2.1` tag), whose crate sources are identical
  to the Q0 engine’s (`e5a71c8a`) that exp-175, exp-176 and exp-194 measured.
- **Candidate:** the release head, `b82f26e1` on `claude/release-0.3.0`, whose crate
  sources are identical to those of `9e4953b0`, on which `make check` passed.
- **What the release adds to exp-201’s engine (`a356d456`).** H184, every route listing
  through the native reader so that no stat of a listed child triggers an automount
  (screened for non-regression in exp-196, `default-tree` +0.42% [−2.67%, +4.11%]); the
  review round’s searchability latch, a stat per listing until one succeeds before its
  kinds are trusted (R163-1: 89% and 83% of exp-197’s `statx` saving kept); the
  every-route overflow check, a checked running total per file; `.gitignore` byte-order
  marks and NUL bytes read as git reads them; and, off the metadata routes measured
  here, analysis batching and the streaming code-line classifier.
  None was measured on wall time before this cell.
- **Expected, from the development records compounded:** the default tree about −41% on
  `linux-v6.12` (exp-194’s −39.00%, then exp-201’s −3.05%), about −18% on
  `node-modules-dense` (−9.75%, then −8.94%), and about −13% on `linux-balanced-1m`
  (exp-194’s 12-pair screen −9%, then exp-197’s screen −4.45%); the default summary
  about −31%, −7% and −20%. The later changes above are expected to cost nothing
  measurable.
- **Standing:** as exp-201 left it on the two real trees (fdu ahead of pdu’s default by
  13% and 15%, of `pdu --max-depth 2` by 3% and 10%, of diskus by 12% and 11%), and, on
  the million-entry tree, unknown: the last run there, on `ebc06c78`, had
  `pdu --max-depth 2` 2.5% faster than fdu’s default tree.

## What was measured

Quiet regime on a 4-vCPU Firecracker guest (Intel Xeon at 2.1 GHz, Linux 6.18.44-fc-v50,
ext4 on virtio, as root), warm steady cache, 20 pairs and 3 warm-ups in every cell,
2026-09-30 between 13:36 and 14:13 UTC. The four binaries (each version’s `fdu` CLI and
`perf_probe`) were built with Rust 1.97.1 and the release profile before the first cell,
copied out of the build tree, and the build tree deleted; no build or other timed work
ran during the cells.
Every cell waited under the measurement lock for ten consecutive one-second readings
below 15% CPU busy (each started after the minimum ten seconds), and the harness’s own
gate of 25% CPU busy before and after each sample never tripped: the highest reading in
any cell was 11%. No invalid sample, no baseline drift, and no mutation during any run;
the probe cells matched the subjects against the fingerprints exp-201 registered
(`tree-*-pdu.json`), and the tool cells wrote their own pre-run fingerprints.

**Against 0.2.1, the engine (probe cells).** `release_vs_v021`, paired change in wall
time with its 95% interval:

| Subject | Job | 0.2.1 | Release | Change |
| --- | --- | ---: | ---: | --- |
| `linux-v6.12` | `default-tree` | 208.6 ms | 109.9 ms | **−48.00% [−50.45%, −44.79%]** |
| `linux-v6.12` | `aggregate-summary` | 184.6 ms | 128.1 ms | **−34.63% [−40.37%, −27.19%]** |
| `node-modules-dense` | `default-tree` | 135.0 ms | 115.6 ms | **−14.09% [−21.53%, −11.50%]** |
| `node-modules-dense` | `aggregate-summary` | 115.3 ms | 99.5 ms | **−12.44% [−15.85%, −11.28%]** |
| `linux-balanced-1m` | `default-tree` | 1,109.1 ms | 947.8 ms | **−14.57% [−18.46%, −12.71%]** |
| `linux-balanced-1m` | `aggregate-summary` | 1,018.5 ms | 834.2 ms | **−16.65% [−20.07%, −13.97%]** |

The primary artifact is the `linux-v6.12` probe cell.
The default tree came out faster than the compounded records predicted on the kernel
tree (−48% against about −41%) and on the generated tree (−15% against about −13%), and
slower on the dense tree (−14% against about −18%); the summary −35%, −12% and −17%
against about −31%, −7% and −20%. Each of those predictions compounds cells run in
different sessions, so the direct figures are the ones to quote.
User CPU fell 75%, 28% and 28% on the default tree and system CPU far less (−1.6%,
−5.6%, −8.0%): what remains is the walk’s kernel time.
The release’s involuntary context switches rose from 56 to 273 on the kernel tree’s
default tree, which is what fails the qualification block’s switch gate, as exp-198
recorded for the summary.
The million-entry default tree’s peak RSS fell 80.43%, from 292.7 to 57.3 MiB, where
exp-194 measured 62 MiB on the round’s final head.

**The standing, product command lines (tool cells).** The release’s `fdu` CLI
(`fdu 0.3.0-dev+gb82f26e17`, `--color never PATH`, the `fdu-default-tree` contract) is
the anchor, and each competitor is paired 20 times with the adjacent anchor run in one
interleaved run per tree: 0.2.1’s `fdu` CLI on the same contract, pdu 0.24.0 at its
default (`--silent-errors PATH`) and at `--max-depth 2`, and diskus 0.9.0. Positive
means the competitor took longer than the adjacent release run.
Because the release is the anchor, every pair below is the harness’s own; nothing is
derived.

| Subject | Release `fdu` | 0.2.1 `fdu` | pdu default | pdu `--max-depth 2` | diskus |
| --- | ---: | ---: | ---: | ---: | ---: |
| `linux-v6.12` | 0.110 s | 0.215 s, **+94.7% [+85.8%, +112.4%]** | 0.124 s, **+15.3% [+12.0%, +19.3%]** | 0.118 s, **+9.7% [+1.8%, +12.2%]** | 0.123 s, **+16.8% [+3.3%, +20.5%]** |
| `node-modules-dense` | 0.106 s | 0.126 s, **+20.5% [+18.3%, +22.4%]** | 0.126 s, **+18.2% [+15.2%, +23.0%]** | 0.119 s, **+11.5% [+1.9%, +18.9%]** | 0.117 s, **+12.1% [+9.3%, +16.5%]** |
| `linux-balanced-1m` | 0.951 s | 1.131 s, **+20.6% [+17.9%, +21.7%]** | 1.196 s, **+25.3% [+20.5%, +27.7%]** | 1.125 s, **+18.8% [+15.0%, +19.7%]** | 1.171 s, **+23.9% [+20.4%, +28.4%]** |

On all three trees the release’s default command took less time than every peer mode and
than 0.2.1, and every interval excludes zero.
The narrowest lower bounds are over `pdu --max-depth 2` on both real trees (+1.8% and
+1.9%) and over diskus on the kernel tree (+3.3%). On the million-entry tree the release
led `pdu --max-depth 2` by 18.8%, pdu’s default by 25.3% and diskus by 23.9%, with a
measured peak RSS of 58.5 MiB against 0.2.1’s 293.8 MiB and pdu’s default 93.4 MiB
(`pdu --max-depth 2` and diskus stay under the harness’s own 39.6 MiB, a bound).
The release’s standard output was byte-identical to 0.2.1’s on each tree in every sample
(one distinct hash per tree for both), and the harness found no semantic or
summary-oracle mismatch.

**What the session adds.** The absolute levels are not exp-201’s: on the two real trees
every tool’s median was 29–38% above its exp-201 figure (pdu’s default 94.4 to 124.3 ms
on `linux-v6.12`, the anchor 84.5 to 109.5 ms against exp-201’s engine), so this was a
slower session on the same kind of guest and kernel build.
The release’s lead over pdu’s default is about where exp-201 left it (15% and 18%
against 13% and 15%); its leads over `pdu --max-depth 2` (10% and 12% against 3% and
10%) and over diskus (17% and 12% against 12% and 11%) are wider.
This run carries no exp-201 engine, so it does not separate the release’s later changes
from the session; the changes were expected to cost nothing, not to gain.
The in-run control is 0.2.1, whose standing against each peer is derived in
`derived-pairs.txt` at equal ordinals, as exp-201 derived the final head’s. On
`linux-v6.12`, pdu’s default took 41.4% [−45.9%, −37.4%] less time than 0.2.1, where
exp-175 measured 58% less on the same engine at Q0; on `node-modules-dense` it was level
with 0.2.1 (−0.9% [−7.4%, +2.6%]), where exp-176 measured 11% less.
So, measured against 0.2.1, the peers stood further back in this session than at Q0. On
the million-entry tree 0.2.1 was level with `pdu --max-depth 2` (+0.8% [−2.4%, +1.7%]).
On 2026-09-29, `pdu --max-depth 2` was 2.5% faster than the round’s final head, which
exp-194’s screen had measured 9% faster than 0.2.1, so by that session’s figures it
would have been about 11% faster than 0.2.1. By that chain, about ten points of the
release’s 18.8% lead over `pdu --max-depth 2` there are this session’s. The chain
crosses sessions and a probe screen, so it is an estimate, not a paired figure.
The lead itself is paired and resolved in this run.

**Harness notes.** The measurement lock printed a warning before each probe cell that a
harness process was running: its pattern matched the lock’s own wrapper command line,
which named `python -m benchmarks`, and no other process was running.
The qualification blocks read `inferior` against every peer on context switches and
missing paired RSS, as they did in exp-201’s tool cells; they are resource gates, not
wall verdicts.

**Supplementary runs.** Beside the primary artifact, gzipped:

- `run-node-modules-dense.json.gz` and `run-linux-balanced-1m.json.gz`: the probe cells
  on the other two subjects;
- `run-tools-linux-v6.12.json.gz`, `run-tools-node-modules-dense.json.gz` and
  `run-tools-linux-balanced-1m.json.gz`: the tool cells;
- `derived-pairs.txt`: 0.2.1’s standing against each peer, with the script.

## Decision

Baseline: the release engine measured end to end, and no decision rests on it.
Against 0.2.1 the release’s default tree takes 48.00% less time on `linux-v6.12`, 14.09%
less on `node-modules-dense` and 14.57% less on `linux-balanced-1m`, and its default
summary 34.63%, 12.44% and 16.65% less.
The million-entry default tree’s peak RSS is 80% lower.
On this host, in this session, the release’s default command is ahead of pdu’s default,
`pdu --max-depth 2` and diskus on all three trees, including the million-entry tree,
where the previous standing (`ebc06c78`) had `pdu --max-depth 2` 2.5% ahead.
The leads over `pdu --max-depth 2` are the narrowest, with lower bounds of +1.8% and
+1.9% on the real trees; on the million-entry tree about ten points of the 18.8% lead
are this session rather than the engine, by a cross-session estimate.
