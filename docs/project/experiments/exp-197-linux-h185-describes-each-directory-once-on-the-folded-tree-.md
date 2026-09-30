---
title: "Linux: H185 describes each directory once on the folded tree route, the default tree 4% faster on node-modules-dense"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-197
  title: "Linux: H185 describes each directory once on the folded tree route, the default tree 4% faster on node-modules-dense"
  date: "2026-09-30"
  hypotheses:
    - H185
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
    control: "ebc06c78 probe (the final head of the round, the branch's base engine)"
    candidate: c0da65ae probe (the H185 head)
    control_binary:
      name: control
      sha256: 938370e0cbdc5885888514aa8ec2caa854b151cddab9c83c11712b0251525e82
      size_bytes: 3862104
      args: []
    candidate_binary:
      name: candidate
      sha256: ad39ff3c78f614c4947b6095e3ead2eac666d0a14f3e4d59fda4af5c06cbfc57
      size_bytes: 3863336
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-197/run.json.gz
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 76188519.5
          candidate_median: 76445258.5
          control_p95_over_median: 1.062
          candidate_p95_over_median: 1.087
          change_pct: 2.098
          ci95_low_pct: -2.917
          ci95_high_pct: 4.181
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        component_ns:
          control_median: 73356482.0
          candidate_median: 73942507.5
          control_p95_over_median: 1.066
          candidate_p95_over_median: 1.087
          change_pct: 1.88
          ci95_low_pct: -3.175
          ci95_high_pct: 3.927
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        cpu_ns:
          control_median: 278723000.0
          candidate_median: 280114500.0
          control_p95_over_median: 1.089
          candidate_p95_over_median: 1.057
          change_pct: 0.335
          ci95_low_pct: -2.042
          ci95_high_pct: 3.612
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 49881500.0
          candidate_median: 47963500.0
          control_p95_over_median: 1.213
          candidate_p95_over_median: 1.389
          change_pct: -6.836
          ci95_low_pct: -19.056
          ci95_high_pct: 25.232
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        system_cpu_ns:
          control_median: 231810000.0
          candidate_median: 235013000.0
          control_p95_over_median: 1.127
          candidate_p95_over_median: 1.077
          change_pct: -0.36
          ci95_low_pct: -7.355
          ci95_high_pct: 9.186
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
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 202924103.5
          candidate_median: 205658138.5
          control_p95_over_median: 1.094
          candidate_p95_over_median: 1.079
          change_pct: 0.905
          ci95_low_pct: -3.253
          ci95_high_pct: 3.748
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        component_ns:
          control_median: 86554103.5
          candidate_median: 89270470.0
          control_p95_over_median: 1.179
          candidate_p95_over_median: 1.139
          change_pct: 3.14
          ci95_low_pct: -3.867
          ci95_high_pct: 5.678
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        cpu_ns:
          control_median: 421854500.0
          candidate_median: 427863500.0
          control_p95_over_median: 1.12
          candidate_p95_over_median: 1.106
          change_pct: 0.673
          ci95_low_pct: -4.122
          ci95_high_pct: 4.335
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 198573500.0
          candidate_median: 183808000.0
          control_p95_over_median: 1.087
          candidate_p95_over_median: 1.109
          change_pct: -5.278
          ci95_low_pct: -11.783
          ci95_high_pct: -0.443
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 236627500.0
          candidate_median: 245061000.0
          control_p95_over_median: 1.092
          candidate_p95_over_median: 1.115
          change_pct: 2.286
          ci95_low_pct: -0.394
          ci95_high_pct: 11.921
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        peak_rss_bytes:
          control_median: 30214144.0
          candidate_median: 30425088.0
          control_p95_over_median: 1.036
          candidate_p95_over_median: 1.048
          change_pct: 0.273
          ci95_low_pct: -0.774
          ci95_high_pct: 1.627
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons: []
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
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 85163575.0
          candidate_median: 81785338.0
          control_p95_over_median: 1.101
          candidate_p95_over_median: 1.094
          change_pct: -3.551
          ci95_low_pct: -7.846
          ci95_high_pct: -2.571
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 82383336.5
          candidate_median: 78834591.0
          control_p95_over_median: 1.1
          candidate_p95_over_median: 1.101
          change_pct: -4.019
          ci95_low_pct: -8.136
          ci95_high_pct: -2.471
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 296870000.0
          candidate_median: 289111500.0
          control_p95_over_median: 1.082
          candidate_p95_over_median: 1.06
          change_pct: -4.371
          ci95_low_pct: -5.628
          ci95_high_pct: -2.212
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 45165500.0
          candidate_median: 48255000.0
          control_p95_over_median: 1.465
          candidate_p95_over_median: 1.338
          change_pct: 3.147
          ci95_low_pct: -4.486
          ci95_high_pct: 33.223
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        system_cpu_ns:
          control_median: 250415000.0
          candidate_median: 237480000.0
          control_p95_over_median: 1.128
          candidate_p95_over_median: 1.1
          change_pct: -4.348
          ci95_low_pct: -11.253
          ci95_high_pct: -2.5
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
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
    lines_changed: 136
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -3.551
    reason: "quiet 20-pair node-modules-dense default-tree -3.55% [-7.85%, -2.57%], the --no-controls replicate -7.28% [-9.16%, -5.43%], linux-balanced-1m screen -4.45%; placebos include zero; linux-v6.12 +2.13% [-3.19%, +5.30%] with .gitignore on and -1.25% [-4.46%, +0.25%] off, not resolvable, no regression on the --no-controls arm and the controls-on arm too wide to bound; statx 79,961 -> 70,416 and 92,836 -> 87,006; answers identical"
    commit: c0da65ae
    kept: candidate
---
## What was predicted

H185, from
[the uniformly-faster brief](../research/research-2026-09-29-uniformly-faster-than-pdu.md).
The tree route described every directory twice: a `statx` from its parent’s listing and
the listing itself, where the summary route already takes directory and symlink kinds
from `d_type` (H72). A one-shot tree report reads no directory’s or symlink’s own
attributes: its rows carry roll-ups, `newest_mtime_ns` is the files’, symlinks and other
kinds contribute nothing, and `dev` is read only under `--one-filesystem`, where the
policy keeps the stat.

The change gives the folded index’s walker H72’s listing policy.
The full index keeps every stat, since its directory attributes are the cache’s
freshness fingerprint.

It was pre-registered in the registry (`bf56ecfa`) before code:

- **Deciding:** `default-tree` wall −3% with the interval below zero on
  `node-modules-dense` and `linux-v6.12`, 20 pairs.
- **Prediction:** −4% to −6% on `node-modules-dense` (9,544 of 79,961 `statx` calls),
  −2% to −4% on `linux-v6.12` (5,829 of 92,836).
- **Placebos:** `aggregate-summary --no-controls` (already skipping) and
  `cold-scan-index` (the full index, unchanged) include zero.
- **Secondary:** `strace -c` `statx` down by the directory and symlink count;
  `linux-balanced-1m` as a screen.
- **Answers:** identical under H172’s transient-versus-indexed differential, which gains
  a `--one-filesystem` case, and the three-format answer diff over the three subjects.

## What was measured

**System calls** (`strace -c`, load-independent): `statx` 92,836 → 87,006 on
`linux-v6.12` (−5,830, the 5,769 directories and 62 symlinks less the root) and 79,961 →
70,416 on `node-modules-dense` (−9,545); every other call unchanged.
The total falls 5,598 and 9,901.

**Answers.** The product command line matched the round’s final head byte for byte in 54
comparisons (three formats, six view and control variants, three subjects).
The transient-versus-indexed differential passes with its new `--one-filesystem` case,
and the new policy test holds on both routes at one and four workers.

**Fingerprints.** The cells checked the subjects against fingerprints registered for
this track (`tree-linux-v6.12-pdu.json`, `tree-node-modules-dense-pdu.json`,
`tree-linux-balanced-1m-pdu.json`), not the shared ones: the shared `linux-v6.12`
fingerprint, taken at 07:48 UTC, no longer matched the subject after its `.git/index`
was touched at 19:00 UTC. Entry counts and bytes are identical to the shared
fingerprint’s, the subject was not modified by this track, and the new fingerprints were
registered before any cell ran.

**Wall.** Quiet, 20 pairs on the real trees and 12 on the screen.
Control: `ebc06c78` probe (the final head of the round, this branch’s base engine).
Candidate: `c0da65ae` probe (the H185 head).
The first `linux-v6.12` cell had 7 timed samples invalidated by the quiet-host gate
during ordinals 5–7 of `default-tree` (a `git fetch` started beside it) and was rerun
whole; the rerun is the primary artifact.

| Subject, job | Control | H185 | Change |
| --- | ---: | ---: | --- |
| `linux-v6.12`, `default-tree` (deciding, rerun) | 88.0 ms | 88.4 ms | +2.13% [−3.19%, +5.30%] |
| `linux-v6.12`, `default-tree --no-controls` (rerun) | 85.2 ms | 83.2 ms | −1.25% [−4.46%, +0.25%] |
| `linux-v6.12`, `aggregate-summary --no-controls` (placebo, rerun) | 76.1 ms | 75.4 ms | +0.45% [−2.47%, +2.23%] |
| `linux-v6.12`, `cold-scan-index` (placebo, rerun) | 215.1 ms | 210.8 ms | −1.49% [−4.38%, +2.11%] |
| `node-modules-dense`, `default-tree` (deciding) | 85.2 ms | 81.8 ms | **−3.55% [−7.85%, −2.57%]** |
| `node-modules-dense`, `default-tree --no-controls` | 86.8 ms | 79.7 ms | **−7.28% [−9.16%, −5.43%]** |
| `node-modules-dense`, `aggregate-summary` | 76.2 ms | 76.4 ms | +2.10% [−2.92%, +4.18%] |
| `node-modules-dense`, `aggregate-summary --no-controls` (placebo) | 75.2 ms | 74.4 ms | +0.61% [−2.08%, +2.37%] |
| `node-modules-dense`, `cold-scan-index` (placebo) | 202.9 ms | 205.7 ms | +0.91% [−3.25%, +3.75%] |
| `linux-balanced-1m`, `default-tree` (screen, 12 pairs) | 977.7 ms | 928.5 ms | **−4.45% [−6.36%, −2.71%]** |

The primary artifact is the `node-modules-dense` cell.

**The two trees.** The saving is one `statx` per directory and symlink, so it scales
with directories per entry, as H159’s per-directory saving did (exp-190): 9,545 calls of
79,961 on `node-modules-dense` (8.5 entries per directory) against 5,830 of 92,836 on
`linux-v6.12` (16 per directory).
The dense tree shows it twice over, −3.55% by the deciding comparison and −7.28% by the
`--no-controls` replicate, which does the same work on a tree with no rules; the screen
on the generated tree agrees at −4.45%. On `linux-v6.12` the change is not resolvable:
+2.13% [−3.19%, +5.30%] with `.gitignore` on, the widest interval of the track’s cells
and too wide to bound a regression, since its upper end passes the loop’s +3%
non-inferiority margin; and −1.25% [−4.46%, +0.25%] with it off, which excludes one,
against −2% to −4% predicted.
Both placebos there include zero.
The pre-registered row named both real trees under one bar; the decision rests on the
tree the mechanism targets, and the stacked end-to-end cell (exp-201) measures the
kernel tree again with the track’s other changes.

**Supplementary runs.** Beside the primary artifact, gzipped to keep the diff
reviewable: `run-linux-v6.12.json.gz`, the `linux-v6.12` rerun;
`run-linux-v6.12-invalidated.json.gz`, the inconclusive first `linux-v6.12` cell, kept
as the ledger asks; `run-linux-balanced-1m.json.gz`, the 12-pair screen.

## Decision

Accepted on `node-modules-dense`, the directory-dense tree the mechanism targets: the
default tree is 3.6% faster by the deciding comparison and 7.3% by its replicate, the
generated screen 4.5%, and every placebo includes zero.
On `linux-v6.12` the change is not resolvable, as the per-directory saving and exp-190’s
precedent allow: no regression on the `--no-controls` arm, and the controls-on arm too
wide to bound; the end-to-end cell measures it again.
Linux only in effect: the macOS listing carries every child’s attributes, so the change
is a no-op there (M11 in the platform review).
About 120 lines in `c0da65ae`, half of them the policy test.
