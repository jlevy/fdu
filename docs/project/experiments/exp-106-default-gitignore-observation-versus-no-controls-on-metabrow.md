---
title: Default gitignore observation versus no-controls on metabrowser
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-106
  title: Default gitignore observation versus no-controls on metabrowser
  date: "2026-09-19"
  hypotheses:
    - H107
  subject:
    tree_label: metabrowser-clone
    tree_root_id: a319238d9c29b19d6efb12266d9b77eecbcbc85f3eaf7949da346f79098ca7ba
    tree_engine_digest: 41a1e8457142ad0af625fef87706a64d004eaef68c8132c36f1f4e1c9e4716fc
    tree_provenance: A clone of github.com/jlevy/metabrowser used as this hosts source-checkout subject. The 2026-08 nominated path is gone from disk; this live checkout replaces it. The clone is reproducible; workspace state on top of it is not.
    tree_reconstructible: false
    tree_entries: 145931
    tree_directories: 11512
    tree_files: 133597
    tree_symlinks: 822
    tree_apparent_bytes: 1724995969
    tree_allocated_bytes: 2058641408
    tree_max_depth: 19
    tree_mutated_during_run: false
    host_cpu: Apple M1 Pro
    host_arch: arm64
    host_cores: 10
    host_performance_cores: 8
    host_efficiency_cores: 2
    host_memory_bytes: 34359738368
    host_system: Darwin 25.5.0
    filesystem: apfs
    host_virtualization: bare-metal
    os_cache: warm-steady
  method:
    trials: 12
    warmups: 3
    interleaved: true
    control: same probe with --no-controls
    candidate: same probe shipped default read_controls on
    control_binary:
      name: control
      sha256: d19b1c4e30c7c2296a3d33ec6ab2daa0e2475f1a91e5a6e8656b9d30f0e900e7
      size_bytes: 2487408
      args:
        - "--no-controls"
    candidate_binary:
      name: candidate
      sha256: d19b1c4e30c7c2296a3d33ec6ab2daa0e2475f1a91e5a6e8656b9d30f0e900e7
      size_bytes: 2487408
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-106-h107-gitignore-default-on.json
  results:
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 338603771.0
          candidate_median: 341608854.0
          control_p95_over_median: 1.023
          candidate_p95_over_median: 1.057
          change_pct: 1.639
          ci95_low_pct: -3.998
          ci95_high_pct: 4.372
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 332585708.5
          candidate_median: 335339479.0
          control_p95_over_median: 1.02
          candidate_p95_over_median: 1.054
          change_pct: 1.614
          ci95_low_pct: -3.494
          ci95_high_pct: 4.471
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1923574500.0
          candidate_median: 1946361000.0
          control_p95_over_median: 1.013
          candidate_p95_over_median: 1.048
          change_pct: 1.116
          ci95_low_pct: -1.694
          ci95_high_pct: 6.04
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 128272000.0
          candidate_median: 156950000.0
          control_p95_over_median: 1.089
          candidate_p95_over_median: 1.072
          change_pct: 22.334
          ci95_low_pct: 13.087
          ci95_high_pct: 28.519
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        system_cpu_ns:
          control_median: 1786426500.0
          candidate_median: 1793111000.0
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.053
          change_pct: -0.168
          ci95_low_pct: -3.979
          ci95_high_pct: 5.157
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 59113472.0
          candidate_median: 54992896.0
          control_p95_over_median: 1.015
          candidate_p95_over_median: 1.021
          change_pct: -6.854
          ci95_low_pct: -7.385
          ci95_high_pct: -5.869
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - voluntary_context_switches is missing a paired percent interval
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
          voluntary_context_switches: inconclusive
        policy_stable: null
        policy_rule: null
  reference_tools:
    - name: dust
      wall_ns_median: 406136521.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
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
    change_pct: 1.639
    reason: "wall +1.64% [-4.00%, +4.37%]; user CPU +22% and RSS -6.9% cancelled on the critical path"
    commit: 285a41d2
---
## What was predicted

H107: on a git-heavy deciding subject, `default-tree` with `read_controls` on (the
shipped default) differs from the same binary with `--no-controls` by at least 3% wall
in either direction, because reading `.gitignore` and retaining ignored-share state is
work, and exclusion can also remove work.

Accept rule, named before the run: absolute paired wall median at least 3%, 95% interval
entirely off zero, no invalid samples.
Either direction counts; this is a characterization of the honest control, not a speed
win. Complexity is zero: same binary, extra argv only.

## What was measured

Same `perf_probe` as exp-105 (`285a41d2`). Control arm: `--no-controls`. Candidate arm:
shipped default (controls on).
Subject: the live metabrowser checkout nominated after the 2026-08 corpus path
disappeared, 145,931 entries, 133,597 files, depth 19, dense (sparse ratio 0.84). 12
interleaved pairs, uncontrolled, warm-steady, tree unchanged.

## What happened

`default-tree` wall +1.64% [-4.00%, +4.37%]. REJECT. The interval includes zero and the
median is under the 3% bar.

User CPU on the controls-on arm rose +22.33% [+13.09%, +28.52%]: the control walk is
real. Peak RSS fell -6.85% [-7.38%, -5.87%] and minor faults -6.34%: exclusion removes
retained state. System CPU and wall absorbed both, so the user-visible job did not move.

## Judgment

The wall prediction is refuted on this subject.
A files/s number taken from a `--no-controls` probe on this tree would have been the
same wall as the shipped default, within noise, against a larger retained set.
Do not treat historical no-controls campaign-1 walls as a different speed here; do treat
their tallies as a different answer.
A tree where ignored subtrees dominate the walk (a checkout sitting on `node_modules`
that `.gitignore` drops) could still move wall.
Re-run H107 only on a subject whose ignored share is large enough that exclusion can be
the critical path.
