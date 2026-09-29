---
title: Linux H159 listing recycle clears 3 percent on a real directory-dense tree
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-190
  title: Linux H159 listing recycle clears 3 percent on a real directory-dense tree
  date: "2026-09-28"
  hypotheses:
    - H159
  subject:
    tree_label: node-modules-dense
    tree_root_id: 1523f67ca22554a156413185f4ddb0b0c267c7971bf0c9ada8d8b00574fa58fd
    tree_engine_digest: d854e8b38414934b745c5d453650645f87e1f324fa64cac6652d7edebca31152
    tree_provenance: "npm dependency trees of react-scripts 5.0.1 and gatsby 5.13.7: copy docs/project/experiments/evidence/exp-190/subject-package.json.txt and subject-package-lock.json.txt to package.json and package-lock.json in an empty directory and run npm ci --ignore-scripts (node 22.22.2, npm 10.9.7); 79,953 entries, 9,439 directories, no .gitignore."
    tree_reconstructible: true
    tree_entries: 79953
    tree_directories: 9439
    tree_files: 70407
    tree_symlinks: 107
    tree_apparent_bytes: 538478644
    tree_allocated_bytes: 757346304
    tree_max_depth: 12
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.10GHz"
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16877547520
    host_system: Linux 6.18.44-fc-v37
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 12
    warmups: 3
    interleaved: true
    control: "eb00edf8 scratch probe: main with H162 and H163"
    candidate: "a15b20f4 probe: H159 with H162 and H163"
    control_binary:
      name: control
      sha256: 9e46354956d0133cce71db81ce2caac4a92b10a88aaae704f677ec71883668e1
      size_bytes: 3716336
      args: []
    candidate_binary:
      name: candidate
      sha256: 058b9e2ab27ab9c4d85a42bac8d2d20766b827696407f3226eaccec6b2918a96
      size_bytes: 3744176
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-190/run.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 197703847.0
          candidate_median: 185546083.0
          control_p95_over_median: 1.129
          candidate_p95_over_median: 1.037
          change_pct: -6.794
          ci95_low_pct: -10.915
          ci95_high_pct: -2.247
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 90149625.0
          candidate_median: 81560357.5
          control_p95_over_median: 1.114
          candidate_p95_over_median: 1.096
          change_pct: -7.221
          ci95_low_pct: -12.441
          ci95_high_pct: -2.871
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 391920500.0
          candidate_median: 375992000.0
          control_p95_over_median: 1.064
          candidate_p95_over_median: 1.061
          change_pct: -4.203
          ci95_low_pct: -7.279
          ci95_high_pct: -2.426
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        user_cpu_ns:
          control_median: 181595000.0
          candidate_median: 180631500.0
          control_p95_over_median: 1.206
          candidate_p95_over_median: 1.121
          change_pct: -3.778
          ci95_low_pct: -9.733
          ci95_high_pct: 2.962
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 212303500.0
          candidate_median: 196079500.0
          control_p95_over_median: 1.068
          candidate_p95_over_median: 1.117
          change_pct: -6.808
          ci95_low_pct: -11.449
          ci95_high_pct: 1.42
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 31381504.0
          candidate_median: 29833216.0
          control_p95_over_median: 1.049
          candidate_p95_over_median: 1.017
          change_pct: -3.84
          ci95_low_pct: -7.596
          ci95_high_pct: -1.097
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: superior
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
          control_median: 106253859.5
          candidate_median: 92897838.5
          control_p95_over_median: 1.112
          candidate_p95_over_median: 1.167
          change_pct: -8.612
          ci95_low_pct: -19.473
          ci95_high_pct: -5.042
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 100974887.0
          candidate_median: 87109703.5
          control_p95_over_median: 1.116
          candidate_p95_over_median: 1.179
          change_pct: -9.763
          ci95_low_pct: -20.125
          ci95_high_pct: -5.127
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 305415000.0
          candidate_median: 292366500.0
          control_p95_over_median: 1.083
          candidate_p95_over_median: 1.139
          change_pct: -4.775
          ci95_low_pct: -10.622
          ci95_high_pct: 1.712
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 92394000.0
          candidate_median: 97101500.0
          control_p95_over_median: 1.297
          candidate_p95_over_median: 1.054
          change_pct: 1.0
          ci95_low_pct: -20.689
          ci95_high_pct: 32.421
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 212545000.0
          candidate_median: 205577500.0
          control_p95_over_median: 1.153
          candidate_p95_over_median: 1.122
          change_pct: -2.213
          ci95_low_pct: -12.756
          ci95_high_pct: 1.559
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 32636928.0
          candidate_median: 30402560.0
          control_p95_over_median: 1.032
          candidate_p95_over_median: 1.024
          change_pct: -6.893
          ci95_low_pct: -8.981
          ci95_high_pct: -1.532
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: superior
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
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -8.612
    reason: "quiet node-modules-dense (8.5 entries per directory) default-tree -8.61% [-19.47%, -5.04%], cold-scan-index -6.79%, peak RSS -6.89%; the per-directory saving the mechanism predicts, absent on sparse linux-v6.12"
    commit: aa58a6b1
    kept: candidate
---
## What was predicted

H159 returns each drained directory listing to the walker that allocated it, so glibc
never frees a walker’s chunk from the index consumer’s thread.
What it saves is paid per directory, not per entry: on `linux-balanced-1m` exp-188
measured 157 ms over 125,001 directories, about 1.25 µs each.
`linux-v6.12`, the subject pre-registered to decide it, has 5,769 directories, where the
same saving is about 7 ms of a 200–570 ms job: below the 3% bar and inside the noise,
which is what exp-188 and exp-189 found.
A fair real deciding subject must therefore be directory-dense.

Pre-registered in `fdu-578e` before any timing, at the maintainer’s request for a
first-principles decision: the same binaries, jobs and rule as exp-189 on a real,
reconstructible, directory-dense tree.
`default-tree` wall down at least 3% with the interval below zero; `cold-scan-index`
expected to move; peak RSS upper bound at most +5%.

## What was measured

Subject `node-modules-dense`: the dependency trees of react-scripts 5.0.1 and gatsby
5.13.7, installed with `npm ci --ignore-scripts` (node 22.22.2, npm 10.9.7): 79,953
entries, 9,439 directories (8.5 entries per directory), 107 symlinks, no `.gitignore`.
Its manifests are committed as
[subject-package.json.txt](evidence/exp-190/subject-package.json.txt) and
[subject-package-lock.json.txt](evidence/exp-190/subject-package-lock.json.txt), renamed
so that no dependency scanner mistakes them for this repository’s own.

Control `eb00edf8` (`main` with H162 and H163), candidate `a15b20f4` (H159 with H162 and
H163). Quiet, 12 interleaved pairs, no invalid sample, tree unchanged.

- `default-tree`: −8.61% [−19.47%, −5.04%] (106.3 → 92.9 ms).
  Accept.
- `cold-scan-index`: −6.79% [−10.91%, −2.25%] (197.7 → 185.5 ms).
- Peak RSS: −6.89% [−8.98%, −1.53%] and −3.84% [−7.60%, −1.10%].

## Decision

Accepted on a real directory-dense subject, as the mechanism predicts.
The recycle pays where directories are many and small (this tree, and the generated one
at −10.63%), and does nothing measurable on the directory-sparse source tree (exp-188,
exp-189), where `cold-scan-index` read +3.22% [+1.21%, +12.87%] once.
Merging [#150](https://github.com/jlevy/fdu/pull/150) remains the maintainer’s action;
this record is the evidence for it.
