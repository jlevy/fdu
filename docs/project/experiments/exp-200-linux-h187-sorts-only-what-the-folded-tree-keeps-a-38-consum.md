---
title: "Linux: H187 sorts only what the folded tree keeps, a 38% consumer cut with no wall change"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-200
  title: "Linux: H187 sorts only what the folded tree keeps, a 38% consumer cut with no wall change"
  date: "2026-09-30"
  hypotheses:
    - H187
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
    control: a356d456 probe (the H186 head)
    candidate: cfae174e probe (the H187 head)
    control_binary:
      name: control
      sha256: 28d57669f1b5cd8bd311157ccdd3006f76df1a3eaf30df915a3d2de6e589af60
      size_bytes: 3863864
      args: []
    candidate_binary:
      name: candidate
      sha256: 0df1e47c281a72803887a0ae389ba78191a679e86af8b6cd67f81d6318ca868e
      size_bytes: 3888744
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-200/run.json.gz
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 84160590.0
          candidate_median: 84147491.5
          control_p95_over_median: 1.069
          candidate_p95_over_median: 1.102
          change_pct: 0.247
          ci95_low_pct: -2.659
          ci95_high_pct: 2.727
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        component_ns:
          control_median: 81289623.0
          candidate_median: 81192378.0
          control_p95_over_median: 1.071
          candidate_p95_over_median: 1.11
          change_pct: 0.198
          ci95_low_pct: -2.199
          ci95_high_pct: 2.738
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        cpu_ns:
          control_median: 311274000.0
          candidate_median: 312606000.0
          control_p95_over_median: 1.073
          candidate_p95_over_median: 1.053
          change_pct: 1.118
          ci95_low_pct: -0.253
          ci95_high_pct: 2.44
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        user_cpu_ns:
          control_median: 68814500.0
          candidate_median: 66189500.0
          control_p95_over_median: 1.193
          candidate_p95_over_median: 1.207
          change_pct: -1.133
          ci95_low_pct: -16.762
          ci95_high_pct: 16.75
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        system_cpu_ns:
          control_median: 250202500.0
          candidate_median: 247087500.0
          control_p95_over_median: 1.112
          candidate_p95_over_median: 1.103
          change_pct: -0.041
          ci95_low_pct: -2.333
          ci95_high_pct: 3.91
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
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 212981513.0
          candidate_median: 211739859.0
          control_p95_over_median: 1.223
          candidate_p95_over_median: 1.068
          change_pct: -0.773
          ci95_low_pct: -2.153
          ci95_high_pct: 1.47
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        component_ns:
          control_median: 94174000.5
          candidate_median: 92133053.5
          control_p95_over_median: 1.142
          candidate_p95_over_median: 1.136
          change_pct: -1.47
          ci95_low_pct: -4.551
          ci95_high_pct: 1.276
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        cpu_ns:
          control_median: 456339500.0
          candidate_median: 454741500.0
          control_p95_over_median: 1.149
          candidate_p95_over_median: 1.119
          change_pct: -0.351
          ci95_low_pct: -2.6
          ci95_high_pct: 1.288
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        user_cpu_ns:
          control_median: 207538000.0
          candidate_median: 207845500.0
          control_p95_over_median: 1.237
          candidate_p95_over_median: 1.19
          change_pct: -0.872
          ci95_low_pct: -9.184
          ci95_high_pct: 3.819
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        system_cpu_ns:
          control_median: 258256500.0
          candidate_median: 250561000.0
          control_p95_over_median: 1.052
          candidate_p95_over_median: 1.116
          change_pct: -1.647
          ci95_low_pct: -6.206
          ci95_high_pct: 5.826
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        peak_rss_bytes:
          control_median: 30337024.0
          candidate_median: 29816832.0
          control_p95_over_median: 1.04
          candidate_p95_over_median: 1.058
          change_pct: -1.245
          ci95_low_pct: -2.89
          ci95_high_pct: 0.783
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: noninferior
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
          control_median: 84252982.5
          candidate_median: 83006844.0
          control_p95_over_median: 1.154
          candidate_p95_over_median: 1.108
          change_pct: -0.348
          ci95_low_pct: -3.606
          ci95_high_pct: 2.185
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        component_ns:
          control_median: 81236042.5
          candidate_median: 79991315.5
          control_p95_over_median: 1.159
          candidate_p95_over_median: 1.108
          change_pct: -0.446
          ci95_low_pct: -3.793
          ci95_high_pct: 2.654
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        cpu_ns:
          control_median: 309123000.0
          candidate_median: 307186000.0
          control_p95_over_median: 1.055
          candidate_p95_over_median: 1.062
          change_pct: -0.044
          ci95_low_pct: -2.152
          ci95_high_pct: 2.324
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        user_cpu_ns:
          control_median: 64377500.0
          candidate_median: 65560000.0
          control_p95_over_median: 1.434
          candidate_p95_over_median: 1.263
          change_pct: -4.409
          ci95_low_pct: -16.02
          ci95_high_pct: 16.809
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        system_cpu_ns:
          control_median: 239202500.0
          candidate_median: 239322500.0
          control_p95_over_median: 1.113
          candidate_p95_over_median: 1.104
          change_pct: 2.494
          ci95_low_pct: -4.008
          ci95_high_pct: 4.924
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
          - "involuntary_context_switches straddles its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: inconclusive
          major_faults: within-limit
          minor_faults: within-limit
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 210
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: rejected
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -0.348
    reason: "quiet 20-pair default-tree -0.35% [-3.61%, +2.19%] on linux-v6.12 and +0.64% [-2.38%, +3.46%] on node-modules-dense against -3% to -5% and -2% to -4% predicted; replicates agree; placebos at zero; cold-scan-index non-inferior; consumer instructions -20% and -38% with identical answers did not reach wall"
    commit: cfae174e
    kept: control
---
## What was predicted

H187, from
[the uniformly-faster brief](../research/research-2026-09-29-uniformly-faster-than-pdu.md).

With `.gitignore` off, the folded tree’s consumer still ran 1,100–1,500 instructions per
entry building a tree pdu’s worker builds in fewer: a name sort of every listing (28M on
`linux-v6.12`) although the tier keeps only directories, symlinks, other kinds and K
files; two empty extension maps built, cloned and dropped per folded file (21M); and a
directory map keyed by `PathBuf`, hashed and compared component by component (19M).

The change hashes each listed name once into an open-addressed table of positions, which
finds any repeated name (the last observation wins, as the sorted dedup kept it, and a
superseded name stays in the listing for the walker to free), sorts only the kinds the
tier keeps, folds each file into its directory’s roll-ups through a scalar add that
states exactly what merging a one-file contribution without an extension does, and keys
the directory map by the path’s bytes.
The full index keeps its sorted dedup.

Pre-registered in the registry (`bf56ecfa`) before code, as a structural composite:

- **Deciding:** `default-tree` wall −3% with the interval below zero on
  `node-modules-dense` and `linux-v6.12`, 20 pairs; predicted −2% to −4% and −3% to −5%.
- **Secondary:** consumer instructions −35% or more; `cold-scan-index` non-inferior.
- **Placebo:** `aggregate-summary --no-controls` includes zero.
- **Answers:** identical under the parallel-equivalence and transient-versus-indexed
  differentials, including repeated names in a listing.

## What was measured

**Instructions** (callgrind per thread, the stripped release binaries): the consumer
thread fell from 228.5M (H186 head) to 182.2M on `linux-v6.12` (−20%; −24% from the H185
head’s 240.6M) and from 103.1M to 64.2M on `node-modules-dense` (−38%; −47% from
120.7M). The remaining 182M on the kernel tree is mostly `.gitignore` classification
(111M), H190’s target.

**Answers.** 129 index and folded-tree tests pass, the 212 goldens pass, and the product
command line matched the H186 head byte for byte in 54 comparisons.

**Fingerprints.** The cells checked the subjects against fingerprints registered for
this track (`tree-linux-v6.12-pdu.json`, `tree-node-modules-dense-pdu.json`,
`tree-linux-balanced-1m-pdu.json`), not the shared ones: the shared `linux-v6.12`
fingerprint, taken at 07:48 UTC, no longer matched the subject after its `.git/index`
was touched at 19:00 UTC. Entry counts and bytes are identical to the shared
fingerprint’s, the subject was not modified by this track, and the new fingerprints were
registered before any cell ran.

**Wall.** Quiet, 20 pairs, no invalid samples in either cell.
Control: `a356d456` probe (the H186 head).
Candidate: `cfae174e` probe (the H187 head).

| Subject, job | Control | H187 | Change |
| --- | ---: | ---: | --- |
| `linux-v6.12`, `default-tree` (deciding) | 84.3 ms | 83.0 ms | −0.35% [−3.61%, +2.19%] |
| `linux-v6.12`, `default-tree --no-controls` | 81.1 ms | 80.1 ms | −0.65% [−2.02%, +1.31%] |
| `linux-v6.12`, `aggregate-summary` | 84.2 ms | 84.1 ms | +0.25% [−2.66%, +2.73%] |
| `linux-v6.12`, `aggregate-summary --no-controls` (placebo) | 75.4 ms | 76.7 ms | −0.21% [−3.80%, +3.31%] |
| `linux-v6.12`, `cold-scan-index` (secondary, non-inferior) | 213.0 ms | 211.7 ms | −0.77% [−2.15%, +1.47%] |
| `node-modules-dense`, `default-tree` (deciding) | 78.8 ms | 80.3 ms | +0.64% [−2.38%, +3.46%] |
| `node-modules-dense`, `default-tree --no-controls` | 80.6 ms | 78.3 ms | −1.62% [−3.41%, +1.15%] |
| `node-modules-dense`, `aggregate-summary` | 76.0 ms | 74.3 ms | −3.69% [−6.08%, +3.25%] |
| `node-modules-dense`, `aggregate-summary --no-controls` (placebo) | 76.5 ms | 75.5 ms | −0.88% [−4.45%, +1.73%] |
| `node-modules-dense`, `cold-scan-index` (secondary, non-inferior) | 202.4 ms | 206.6 ms | +1.63% [−3.67%, +4.48%] |

The primary artifact is the `linux-v6.12` cell.
Every comparison on both trees includes zero, the deciding ones and their
`--no-controls` replicates alike, and the full index is non-inferior.
The consumer executed 20% and 38% fewer instructions and the answers did not change, and
none of it reached wall: on four vCPUs the tree route’s consumer is not on the critical
path (it was busy 60% and 30% of the walk in the brief’s profile, the H174 gate’s
finding), so its user-space savings are slack, where H188’s were not because the summary
consumer was the longest thread.
The attribution’s wall model, every CPU millisecond on any thread costing a quarter of a
millisecond of wall, holds for the walkers’ kernel time and the serial tail, not for a
consumer with slack.

**Supplementary runs.** Beside the primary artifact, gzipped to keep the diff
reviewable: `run-node-modules-dense.json.gz`, the `node-modules-dense` cell.

## Decision

Rejected. No wall change on either real tree at 20 pairs (−0.35% [−3.61%, +2.19%] and
+0.64% [−2.38%, +3.46%]) against −3% to −5% and −2% to −4% predicted, with the
replicates agreeing, the placebos at zero and the full index non-inferior.
The instruction cut is real and the answers are identical, but 178 lines of consumer
structure that buy no wall are complexity without a return; the change is reverted from
the track’s branch and not merged.
What it teaches is recorded in the brief: after H185 and H186, the tree route’s consumer
has slack on four vCPUs, and the next wall comes from the walkers’ kernel time or the
serial tail, not from consumer instructions.
