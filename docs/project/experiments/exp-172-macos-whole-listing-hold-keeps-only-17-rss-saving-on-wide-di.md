---
title: "macOS whole-listing hold keeps only 17% RSS saving on wide directories"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-172
  title: "macOS whole-listing hold keeps only 17% RSS saving on wide directories"
  date: "2026-09-28"
  hypotheses:
    - H161
  subject:
    tree_label: rustup-toolchains
    tree_root_id: 36ce9b22af9a6164721fc2d04580d7da220ffb0de00e0a1c0cac4fd9e9cc21b6
    tree_engine_digest: b2e3920a70095ca9ea73098ff48319125083a027b1dd8921458931a508f28ec7
    tree_provenance: "The rustup toolchain store for this machine's installed toolchains (root_id 36ce9b22). Shape depends on which toolchains and targets are installed, so it is not a recipe another machine can follow to the same tree."
    tree_reconstructible: false
    tree_entries: 77159
    tree_directories: 3420
    tree_files: 73739
    tree_symlinks: 0
    tree_apparent_bytes: 3212541127
    tree_allocated_bytes: 3440263168
    tree_max_depth: 17
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
    control: "a5c0ab46 probe: the default summary falls closed to the full index"
    candidate: "cbeb9e57 probe: first cut, each listing held whole until it ends"
    control_binary:
      name: control
      sha256: 02e8f3805362af40ee4b2fc83cb1518d039a1dac947280dfb254f6bfac7bf9c3
      size_bytes: 3148912
      args: []
    candidate_binary:
      name: candidate
      sha256: 6b5c0e583bf1201d01e3cab7263d00e9de89ef7cec03cbee5abcab19171585b2
      size_bytes: 3165424
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-172/run.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 119419770.5
          candidate_median: 122416437.5
          control_p95_over_median: 1.087
          candidate_p95_over_median: 1.068
          change_pct: 2.917
          ci95_low_pct: -9.905
          ci95_high_pct: 14.515
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 113823771.5
          candidate_median: 117092374.5
          control_p95_over_median: 1.089
          candidate_p95_over_median: 1.069
          change_pct: 3.284
          ci95_low_pct: -10.517
          ci95_high_pct: 15.167
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 625217500.0
          candidate_median: 600185000.0
          control_p95_over_median: 1.045
          candidate_p95_over_median: 1.027
          change_pct: -3.803
          ci95_low_pct: -10.706
          ci95_high_pct: 2.516
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 48990500.0
          candidate_median: 37063500.0
          control_p95_over_median: 1.111
          candidate_p95_over_median: 1.045
          change_pct: -24.386
          ci95_low_pct: -28.961
          ci95_high_pct: -22.182
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        system_cpu_ns:
          control_median: 573559500.0
          candidate_median: 563045500.0
          control_p95_over_median: 1.055
          candidate_p95_over_median: 1.028
          change_pct: -2.305
          ci95_low_pct: -9.612
          ci95_high_pct: 5.153
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 26877952.0
          candidate_median: 22298624.0
          control_p95_over_median: 1.047
          candidate_p95_over_median: 1.068
          change_pct: -17.072
          ci95_low_pct: -21.272
          ci95_high_pct: -12.091
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
  reference_tools: []
  complexity:
    lines_changed: 900
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: First cut of H161; its scan.rs emission held each listing until it ended.
  verdict:
    decision: superseded
    primary_job: aggregate-summary
    primary_metric: peak_rss_bytes
    change_pct: -17.072
    reason: "Peak RSS -17.07% [-21.27%, -12.09%] (25 to 21 MiB, against 10 MiB without controls) fails the pre-registered 50% bar: holding whole listings sizes batches by the widest listing. Superseded by 060bbfe6, which sends at batch_size and probes the directory control when a batch fills first (exp-171)."
    commit: cbeb9e57
    kept: neither
---
## What was predicted

The first cut of H161 (`cbeb9e57`) kept each directory’s control ahead of its entries by
holding a worker’s whole listing until it ended, moving the control to the front, and
sending batches only at chunk boundaries.
It was measured under the registration in
[exp-170](exp-170-macos-ignore-aware-transient-summary-cuts-default-summary-pe.md):
primary peak RSS of `aggregate-summary` (bare) down at least 50% with the interval below
zero.

## What was measured

The same four-variant design as exp-170, on the nominated package cache (77,159 entries,
listings of up to about 10,000 entries): `a5c0ab46` and `cbeb9e57`, each bare and with
`--no-controls`, 3 warmups and 12 timed trials, `uncontrolled` after the quiet attempt
invalidated samples.
No sample was invalid and the tree was unchanged.

- Peak RSS: −17.07% [−21.27%, −12.09%] (25 MiB to 21 MiB), against 10 MiB for either
  build with `--no-controls`. Fails the 50% bar.
- Wall: +2.92% [−9.90%, +14.52%]; inconclusive at the +3% margin.
- User CPU: −24.39% [−28.96%, −22.18%].

On the source checkout the same cut measured −64.81% [−66.17%, −63.42%], because its
listings are short.
Holding a whole listing makes a batch as large as the largest listing
in a chunk, and each worker keeps the recycled vectors at that capacity, so a tree of
wide directories pays most of what the index cost.

## Decision

Superseded before shipping by `060bbfe6`: a grouping emission now sends at `batch_size`
like every other stream, and when a batch fills before a listing’s `.gitignore` is
listed it reads that file directly and lets the read stand for the listing.
A three-run screen put the revision at 10–11 MiB on this tree, and
[exp-171](exp-171-macos-ignore-aware-transient-summary-cuts-peak-rss-58-on-a-t.md)
measured it at −57.86%. Neither arm of this experiment is in the product.
