---
title: "Linux H183 cheap matcher pre-checks cut the default tree 8% on linux-v6.12"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-193
  title: "Linux H183 cheap matcher pre-checks cut the default tree 8% on linux-v6.12"
  date: "2026-09-29"
  hypotheses:
    - H183
  subject:
    tree_label: linux-v6.12
    tree_root_id: 4ffe9d749fe638cd6c746668f7ae66d88e50225fa6903eabd8c4b9405b1aa245
    tree_engine_digest: 55a09ea65f48d38d88d7e2eacde95fa81d0a9970baabfeeba6f9e9418f478704
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
    host_system: Linux 6.18.44-fc-v49
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 20
    warmups: 3
    interleaved: true
    control: "20933081 probe: H169 head"
    candidate: "e2ef8bcb probe: H183 matcher pre-checks"
    control_binary:
      name: control
      sha256: 1b69d391a91fdb3296d51c97a396d2365731941ab631ef333f03ff0a016882b8
      size_bytes: 3861144
      args: []
    candidate_binary:
      name: h183
      sha256: 3284bacdc68f77e4776a569da46cbd20184e90a56b6b3ecef4cc68379a78f867
      size_bytes: 3861928
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-193/run.json.gz
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 71833339.0
          candidate_median: 67908330.0
          control_p95_over_median: 1.116
          candidate_p95_over_median: 1.118
          change_pct: -6.315
          ci95_low_pct: -8.517
          ci95_high_pct: 0.455
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        component_ns:
          control_median: 68978935.5
          candidate_median: 64716541.5
          control_p95_over_median: 1.117
          candidate_p95_over_median: 1.132
          change_pct: -6.696
          ci95_low_pct: -9.005
          ci95_high_pct: 0.259
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        cpu_ns:
          control_median: 248779000.0
          candidate_median: 239185000.0
          control_p95_over_median: 1.084
          candidate_p95_over_median: 1.129
          change_pct: -5.957
          ci95_low_pct: -7.869
          ci95_high_pct: 0.758
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        user_cpu_ns:
          control_median: 98735500.0
          candidate_median: 87781500.0
          control_p95_over_median: 1.212
          candidate_p95_over_median: 1.158
          change_pct: -15.602
          ci95_low_pct: -19.958
          ci95_high_pct: 0.021
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        system_cpu_ns:
          control_median: 150067000.0
          candidate_median: 156366000.0
          control_p95_over_median: 1.199
          candidate_p95_over_median: 1.152
          change_pct: 7.33
          ci95_low_pct: 0.07
          ci95_high_pct: 15.229
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
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
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 73172388.5
          candidate_median: 67442402.5
          control_p95_over_median: 1.066
          candidate_p95_over_median: 1.052
          change_pct: -7.622
          ci95_low_pct: -10.414
          ci95_high_pct: -5.276
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 69961552.0
          candidate_median: 64419603.5
          control_p95_over_median: 1.07
          candidate_p95_over_median: 1.05
          change_pct: -7.964
          ci95_low_pct: -11.298
          ci95_high_pct: -4.841
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 249281000.0
          candidate_median: 236233500.0
          control_p95_over_median: 1.055
          candidate_p95_over_median: 1.078
          change_pct: -3.306
          ci95_low_pct: -5.91
          ci95_high_pct: -2.104
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 89399000.0
          candidate_median: 80169500.0
          control_p95_over_median: 1.17
          candidate_p95_over_median: 1.185
          change_pct: -6.796
          ci95_low_pct: -17.204
          ci95_high_pct: 0.074
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        system_cpu_ns:
          control_median: 162327000.0
          candidate_median: 161478500.0
          control_p95_over_median: 1.096
          candidate_p95_over_median: 1.156
          change_pct: -1.219
          ci95_low_pct: -4.525
          ci95_high_pct: 2.694
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
    lines_changed: 300
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -7.622
    reason: "quiet 20-pair linux-v6.12 default-tree -7.62% [-10.41%, -5.28%]; --no-controls and node-modules-dense placebos include zero; consumer instructions 436M -> 238M; answers identical"
    commit: e2ef8bcb
    kept: candidate
---
## What was predicted

H183, found by a callgrind of the default tree on `linux-v6.12` at the H169 head
(`217861c1`). With `.gitignore` handling on, the consumer thread executed 436M
instructions, against 103M with it off.
Of the 333M difference, 114M were `memcmp` and about 97M were `Checks::admit`. H171’s
residual pre-checks compared each wildcard rule’s literal prefix and suffix with slice
equality, and searched its inner run with a `memcmp` at every position of the name.
That ran for each residual rule on every entry.

The change computes a folded 32-bit set of the name’s byte classes once per entry.
Each rule keeps the classes of its literal bytes, and the first byte of its prefix and
last byte of its suffix, so most rules are rejected with a mask test and two byte
compares. The survivors compare their literals with inline byte loops.
Every check stays a necessary condition; the glob still decides every answer.

It was pre-registered in the registry (`7e1a874a`) before code:
- **Deciding:** `default-tree` wall −3% with the interval below zero on `linux-v6.12`,
  20 pairs.
- **Placebos:** both arms `--no-controls` on `linux-v6.12`, and `default-tree` on
  `node-modules-dense`, which has no `.gitignore`.
- **Secondary:** consumer instructions down at least 150M.
- **Prediction:** −4% to −8%.

## What was measured

Callgrind, same command and subject: the consumer fell from 436M to 238M instructions
(−198M, −45%), and `memcmp` from matching from 101M to zero.
What `.gitignore` handling costs the consumer fell from 333M to 135M.

Quiet, 20 pairs, no invalid samples.
The first attempt was killed by a container restart and rerun whole.
Control: `20933081` probe (the H169 head engine).
Candidate: `e2ef8bcb`.

| Subject, job | Control | H183 | Change |
| --- | ---: | ---: | --- |
| `linux-v6.12`, `default-tree` (deciding) | 73.2 ms | 67.4 ms | **−7.62% [−10.41%, −5.28%]** |
| `linux-v6.12`, `default-tree --no-controls` (placebo) | 62.1 ms | 62.1 ms | +0.20% [−2.51%, +3.66%] |
| `linux-v6.12`, `aggregate-summary` | 71.8 ms | 67.9 ms | −6.32% [−8.52%, +0.46%] |
| `linux-v6.12`, `aggregate-summary --no-controls` | 55.0 ms | 53.7 ms | −1.70% [−5.08%, +1.17%] |
| `node-modules-dense`, `default-tree` (placebo) | 69.9 ms | 70.3 ms | +1.43% [−2.74%, +3.89%] |

The `node-modules-dense` run is kept beside this record’s evidence.
With H183 the controls-on default tree is 1.086 times its own `--no-controls` arm (67.4
against 62.1 ms).

**Answers.** The product command line matched the H169 head byte for byte in 171
comparisons over the three subjects.
H171’s property test against the linear matcher and the `t3070-wildmatch` table pass
unchanged. New tests cover:
- names lacking or holding a rule’s literal bytes, including escapes, bracket classes,
  `?` and non-UTF-8;
- a 4,000-glob check that every name a glob matches is admitted;
- an exhaustive comparison of the inline byte loops with the slice methods.

`content_cost` is unchanged: the per-rule record stays 16 bytes, because the byte set is
folded to 32 bits and the length fields narrowed with saturation, which can only weaken
a check.

**Supplementary runs.** Beside the primary artifact, gzipped to keep the diff
reviewable: `run-node-modules-dense.json.gz`, the `node-modules-dense` placebo.

## Decision

Accepted.
The default tree on the source tree with 358 `.gitignore` files is 7.6% faster,
as predicted, and both placebos include zero.
The default summary moves the same way but its interval reaches past zero.
About 300 lines, most of them tests.
