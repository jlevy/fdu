---
title: "Linux: H186 admits tree rows before building them, the default tree 5% faster on node-modules-dense"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-199
  title: "Linux: H186 admits tree rows before building them, the default tree 5% faster on node-modules-dense"
  date: "2026-09-30"
  hypotheses:
    - H186
  subject:
    tree_label: node-modules-dense
    tree_root_id: db6d49b68990d6b3a741e6289f360187d4317abfdda8be3f70915f799eb7acd4
    tree_engine_digest: 11c4122bec53693180e2e240899b75cdccc707ca9759b9acfadb80a0b19bafac
    tree_provenance: "npm dependency trees of react-scripts 5.0.1 and gatsby 5.13.7: copy docs/project/experiments/evidence/exp-190/subject-package.json.txt and subject-package-lock.json.txt to package.json and package-lock.json in an empty directory and run npm ci --ignore-scripts (node 22.22.2, npm 10.9.7); rebuilt 2026-09-29 on this host: 79,957 entries (exp-190 recorded 79,953), 9,439 directories, no .gitignore."
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
    host_memory_bytes: 16876515328
    host_system: Linux 6.18.44-fc-v50
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 20
    warmups: 3
    interleaved: true
    control: 7a3a7058 probe (the H188 part-2 head)
    candidate: a356d456 probe (the H186 head)
    control_binary:
      name: control
      sha256: 6d966cea2195da057e2eb909583ad29912f3d09505ea9b084887ee4aa7ec99c1
      size_bytes: 3870248
      args: []
    candidate_binary:
      name: candidate
      sha256: 28d57669f1b5cd8bd311157ccdd3006f76df1a3eaf30df915a3d2de6e589af60
      size_bytes: 3863864
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-199/run.json.gz
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 75367584.5
          candidate_median: 75142659.0
          control_p95_over_median: 1.12
          candidate_p95_over_median: 1.076
          change_pct: 0.379
          ci95_low_pct: -1.59
          ci95_high_pct: 3.815
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        component_ns:
          control_median: 72825289.0
          candidate_median: 72482124.5
          control_p95_over_median: 1.11
          candidate_p95_over_median: 1.076
          change_pct: 0.158
          ci95_low_pct: -2.058
          ci95_high_pct: 3.926
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        cpu_ns:
          control_median: 279224000.0
          candidate_median: 275641500.0
          control_p95_over_median: 1.11
          candidate_p95_over_median: 1.079
          change_pct: 0.068
          ci95_low_pct: -3.256
          ci95_high_pct: 3.558
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 44457000.0
          candidate_median: 50427500.0
          control_p95_over_median: 1.35
          candidate_p95_over_median: 1.287
          change_pct: 1.499
          ci95_low_pct: -14.077
          ci95_high_pct: 21.231
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        system_cpu_ns:
          control_median: 233008500.0
          candidate_median: 232756500.0
          control_p95_over_median: 1.153
          candidate_p95_over_median: 1.092
          change_pct: 0.46
          ci95_low_pct: -4.71
          ci95_high_pct: 4.083
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - peak_rss_bytes is missing a paired percent interval
          - "minor_faults straddles its +10% regression limit"
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
          minor_faults: inconclusive
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
          control_median: 82113023.5
          candidate_median: 77240115.0
          control_p95_over_median: 1.059
          candidate_p95_over_median: 1.127
          change_pct: -4.913
          ci95_low_pct: -6.936
          ci95_high_pct: -2.861
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 78996180.0
          candidate_median: 74006873.5
          control_p95_over_median: 1.059
          candidate_p95_over_median: 1.123
          change_pct: -5.327
          ci95_low_pct: -7.593
          ci95_high_pct: -2.558
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 284099500.0
          candidate_median: 280538000.0
          control_p95_over_median: 1.068
          candidate_p95_over_median: 1.074
          change_pct: -2.051
          ci95_low_pct: -3.146
          ci95_high_pct: -0.34
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 20
        user_cpu_ns:
          control_median: 57424500.0
          candidate_median: 50619500.0
          control_p95_over_median: 1.475
          candidate_p95_over_median: 1.551
          change_pct: -16.137
          ci95_low_pct: -34.953
          ci95_high_pct: -4.037
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 226982000.0
          candidate_median: 231254000.0
          control_p95_over_median: 1.101
          candidate_p95_over_median: 1.118
          change_pct: 1.673
          ci95_low_pct: -1.353
          ci95_high_pct: 9.669
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
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
    lines_changed: 178
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -4.913
    reason: "quiet 20-pair node-modules-dense default-tree -4.91% [-6.94%, -2.86%], the --no-controls replicate -0.93% [-3.60%, +3.07%] putting the effect near the top of the predicted -2.5% to -4%; placebos include zero; linux-v6.12 -1.24% [-6.47%, +1.45%], in the predicted range, not resolvable, no regression; answers identical"
    commit: a356d456
    kept: candidate
---
## What was predicted

H186, from
[the uniformly-faster brief](../research/research-2026-09-29-uniformly-faster-than-pdu.md).

After the walk, the default tree’s serial tail was 2.8 ms on `linux-v6.12` and 4.5 ms on
`node-modules-dense`, 3.5–6% of the run, on the thread that renders the answer while
every other core is idle.
Two things made most of it: the tree query built a row, with a `PathBuf` and a `String`,
for every child of every expanded directory, sorted them with a comparator that cloned
two `String`s per comparison, cloned every row the share threshold then omitted, and
only then applied the threshold (9.1M and 16.3M instructions); and the folded index,
below H156’s 64Ki-entry threshold, was freed inline (3M and 5.3M).

The change decides each child’s admission from its roll-up before building anything for
it, sums the omitted children from their scalars, borrows names in the sort, and
releases any index of 4Ki entries or more on the detached thread.

Pre-registered in the registry (`bf56ecfa`) before code:

- **Deciding:** `default-tree` wall −3% with the interval below zero on
  `node-modules-dense` and `linux-v6.12`, 20 pairs; predicted −2.5% to −4% and −1.5% to
  −2.5%.
- **Placebo:** `aggregate-summary --no-controls` includes zero.
- **Screen:** `--format json` with the List view on `node-modules-dense`, whose sort was
  4.6G instructions.
- **Answers:** goldens, parity and the three-format answer diff unchanged.

## What was measured

**Answers.** The 212 goldens pass on the change; the report tests, the
transient-versus-indexed differential and the folded-builder tests pass (62 tests); the
product command line matched the H188 head byte for byte in the extended answer diff
(171 comparisons: three formats, nineteen view, sort, share, depth, breadth and size
variants, three subjects).

**Instructions** (callgrind per thread, the stripped release binaries): the consumer
thread fell from 239.3M to 228.5M on `linux-v6.12` and from 120.6M to 103.1M on
`node-modules-dense` (the report and the release together were 12M and 22M inclusive at
the H188 head, and the release now runs on its own thread, which the walkers’ sum shows:
104.9M to 110.3M on the dense tree).
The List view’s JSON on `node-modules-dense` fell only from 6.18G to 5.76G on its main
thread: the sort’s cost is not the two `String` clones alone (recorded as an open item
for `fdu-faqa`, below).

**Fingerprints.** The cells checked the subjects against fingerprints registered for
this track (`tree-linux-v6.12-pdu.json`, `tree-node-modules-dense-pdu.json`,
`tree-linux-balanced-1m-pdu.json`), not the shared ones: the shared `linux-v6.12`
fingerprint, taken at 07:48 UTC, no longer matched the subject after its `.git/index`
was touched at 19:00 UTC. Entry counts and bytes are identical to the shared
fingerprint’s, the subject was not modified by this track, and the new fingerprints were
registered before any cell ran.

**Wall.** Quiet, 20 pairs, no invalid samples in either cell.
Control: `7a3a7058` probe (the H188 part-2 head).
Candidate: `a356d456` probe (the H186 head).

| Subject, job | Control | H186 | Change |
| --- | ---: | ---: | --- |
| `linux-v6.12`, `default-tree` (deciding) | 90.1 ms | 87.2 ms | −1.24% [−6.47%, +1.45%] |
| `linux-v6.12`, `default-tree --no-controls` | 83.6 ms | 80.6 ms | −1.41% [−5.68%, +0.30%] |
| `linux-v6.12`, `aggregate-summary` | 84.0 ms | 83.6 ms | −1.07% [−4.78%, +0.41%] |
| `linux-v6.12`, `aggregate-summary --no-controls` (placebo) | 76.7 ms | 78.0 ms | −0.48% [−2.47%, +6.08%] |
| `node-modules-dense`, `default-tree` (deciding) | 82.1 ms | 77.2 ms | **−4.91% [−6.94%, −2.86%]** |
| `node-modules-dense`, `default-tree --no-controls` | 81.2 ms | 79.9 ms | −0.93% [−3.60%, +3.07%] |
| `node-modules-dense`, `aggregate-summary` | 75.4 ms | 75.1 ms | +0.38% [−1.59%, +3.81%] |
| `node-modules-dense`, `aggregate-summary --no-controls` (placebo) | 77.5 ms | 77.5 ms | −1.19% [−3.51%, +1.02%] |

The primary artifact is the `node-modules-dense` cell.

**The bar and the kernel tree.** The pre-registered row (H186 in the registry) names
both real trees under one bar, “wall −3% with the interval below zero”, while predicting
−1.5% to −2.5% on `linux-v6.12`, below that bar: the kernel tree could not have cleared
it by its own prediction, because its serial tail was 2.8 ms of 86. The decision rests
on the tree the mechanism targets, as H159 was accepted on `node-modules-dense` with no
effect on the directory-sparse tree (exp-190, with exp-188 and exp-189 as its
`linux-v6.12` cells).
On `linux-v6.12` the change is in the predicted range, not resolvable, and no
regression: −1.24% [−6.47%, +1.45%] with `.gitignore` on, −1.41% [−5.68%, +0.30%] with
it off, and the summary at zero.

**The replicate.** On a tree with no rules the `--no-controls` `default-tree` arm does
the same work as the deciding arm, so it is a second measurement of the same effect, and
it measured −0.93% [−3.60%, +3.07%] against the primary’s −4.91% [−6.94%, −2.86%]. The
four medians (82.1 and 81.2 ms against 77.2 and 79.9) put the effect between the two:
most likely near the top of the predicted range, about −2.5% to −4%, rather than −4.9%.
The stacked end-to-end cell of the track (the round’s final head against the H187 head,
with the peer tools in the same run) will confirm it.

**Supplementary runs.** Beside the primary artifact, gzipped to keep the diff
reviewable: `run-linux-v6.12.json.gz`, the `linux-v6.12` cell.

## Decision

Accepted on `node-modules-dense`, the tree the mechanism targets: the default tree is
4.9% faster by the pre-registered comparison, most likely 2.5–4% by the replicate, and
both placebos include zero.
On `linux-v6.12` the change is in the predicted range, not resolvable, and no
regression; by its own prediction it could not clear the row’s bar, as H159’s
kernel-tree cells could not (exp-190). The stacked end-to-end cell settles the size of
the effect. The List view’s JSON sort stays an open item.
About 180 lines in `a356d456`, none of them a test: the admission was guarded by the
goldens and the transient-versus-indexed differential, and the unit test of `child_rows`
(admission, order, and the share omission’s sums) was added after review (R163-4).
