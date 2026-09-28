---
title: Linux H161 ignore-aware transient summary clears wall; RSS bar met only without gitignore
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-187
  title: Linux H161 ignore-aware transient summary clears wall; RSS bar met only without gitignore
  date: "2026-09-28"
  hypotheses:
    - H161
  subject:
    tree_label: linux-v6.12
    tree_root_id: 14549a49743a72c3c1aadb09f34f8a097211652c22d533110f3c18b91c518d71
    tree_engine_digest: c7a4d447d9bf3ab36d55c385a4bbe3ed367963aca0fc6e8e5671ec3e5ad8b124
    tree_provenance: "Shallow clone of github.com/torvalds/linux tag v6.12 at adc218676eef25575469234709c2d87185ca223a. Reconstructible: git clone --depth 1 --branch v6.12 https://github.com/torvalds/linux.git. Shape is the published v6.12 source tree plus the clone's .git directory as git left it; no extra workspace install."
    tree_reconstructible: true
    tree_entries: 92474
    tree_directories: 5769
    tree_files: 86643
    tree_symlinks: 62
    tree_apparent_bytes: 1759236097
    tree_allocated_bytes: 1965420544
    tree_max_depth: 14
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
    control: "a5c0ab46 probe: the default summary falls closed to the full index"
    candidate: "0d73ed54 probe (main with 149): the transient summary classifies entries"
    control_binary:
      name: control
      sha256: 07f0a0323659f51fb85276860f2520486201d0710d958da8be4adfce742d365f
      size_bytes: 3687248
      args: []
    candidate_binary:
      name: candidate
      sha256: 2d8684e546f5eb7754e3503b1be74749310880dc96369e6e984c1dbbd157b3d6
      size_bytes: 3707464
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-187/run.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 557585763.5
          candidate_median: 512276362.5
          control_p95_over_median: 1.104
          candidate_p95_over_median: 1.072
          change_pct: -6.926
          ci95_low_pct: -11.288
          ci95_high_pct: -0.169
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        component_ns:
          control_median: 551082279.5
          candidate_median: 506524419.5
          control_p95_over_median: 1.108
          candidate_p95_over_median: 1.076
          change_pct: -7.005
          ci95_low_pct: -11.009
          ci95_high_pct: -0.067
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        cpu_ns:
          control_median: 755272500.0
          candidate_median: 728513000.0
          control_p95_over_median: 1.095
          candidate_p95_over_median: 1.057
          change_pct: -0.756
          ci95_low_pct: -6.146
          ci95_high_pct: 2.485
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 572617000.0
          candidate_median: 562463500.0
          control_p95_over_median: 1.05
          candidate_p95_over_median: 1.063
          change_pct: -0.988
          ci95_low_pct: -3.595
          ci95_high_pct: 5.086
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 192630500.0
          candidate_median: 175483000.0
          control_p95_over_median: 1.08
          candidate_p95_over_median: 1.072
          change_pct: -9.095
          ci95_low_pct: -18.282
          ci95_high_pct: -5.783
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 12
        peak_rss_bytes:
          control_median: 37498880.0
          candidate_median: 28905472.0
          control_p95_over_median: 1.01
          candidate_p95_over_median: 1.023
          change_pct: -22.86
          ci95_low_pct: -23.726
          ci95_high_pct: -21.921
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
          - "voluntary_context_switches straddles its +50% regression limit"
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
          peak_rss_bytes: within-limit
          system_cpu_ns: within-limit
          voluntary_context_switches: inconclusive
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
    primary_job: aggregate-summary
    primary_metric: wall_ns
    change_pct: -6.926
    reason: "quiet linux-v6.12 wall -6.93% [-11.29%, -0.17%], placebos include zero; peak RSS -22.86% misses the 50% bar there, balanced-1m -19.27% wall and -97% RSS"
    commit: 0d73ed54
    kept: candidate
---
## What was predicted

H161, pre-registered in its registry row on 2026-09-28: the default `--view summary`
retained the full index only to classify the ignored share, so classifying each entry in
the transient reducer answers the same question without retaining an index.
macOS accepted it on peak RSS (exp-170, exp-171), and
[#149](https://github.com/jlevy/fdu/pull/149) merged on that evidence.
The Linux cell decides wall: `aggregate-summary`, bare (controls on), quiet, on
reconstructible `linux-v6.12` with `linux-balanced-1m` screening; wall down at least 3%
with the interval below zero and peak RSS down at least 50%; placebos with both arms
`--no-controls`, and `default-tree`.

## What was measured

Control: the release probe at `a5c0ab46`, the engine #149 was stacked on.
Candidate: the probe at `main` `0d73ed54`, which carries #149 and its review fixes
(`eeb257c9`, the exact-name probe confirmation).
Quiet cells, 12 interleaved pairs, no invalid sample, tree unchanged.

`linux-v6.12`, four variants (control, candidate, and each with `--no-controls`):

- `aggregate-summary` wall: −6.93% [−11.29%, −0.17%] (557.6 → 512.3 ms).
  Accept.
- Peak RSS: −22.86% [−23.73%, −21.92%] (35.8 → 27.6 MiB). Short of the 50% bar.
- Placebo, both arms `--no-controls`: +0.95% [−8.52%, +4.21%].
- `default-tree` placebo: +0.85% [−8.62%, +10.94%], peak RSS −0.15%
  ([evidence/exp-187/default-tree-placebo-run.json](evidence/exp-187/default-tree-placebo-run.json)).

`linux-balanced-1m`, screening, no `.gitignore` files
([evidence/exp-187/balanced-run.json](evidence/exp-187/balanced-run.json)):

- `aggregate-summary` wall: −19.27% [−22.68%, −17.34%] (1,562.6 → 1,265.2 ms).
- Placebo, both arms `--no-controls`: +0.56% [−0.67%, +3.83%].
- The harness withholds peak RSS values below its Linux floor, which is inherited from
  the Python launcher across `execve`. A small C launcher (fork, exec, `wait4`, median
  of five) measured 314.2 MiB for the control, 8.6 MiB for the candidate, and 8.5 MiB
  for either arm with `--no-controls`: −97%. On `linux-v6.12` the same launcher agreed
  with the harness: 35.7 and 27.6 MiB, and 7.6 MiB without controls.

With controls on, both arms spent about 0.5 s of user CPU classifying 92,474 entries on
one thread, 7.5 times the `--no-controls` walk; that cost is what H162 and H163 then
removed.

## Decision

Accepted on wall, as the Linux cell decides.
The RSS bar is met on the tree without `.gitignore` (−97%) and missed on the source tree
(−22.86%), where classification itself holds about 20 MiB more than a walk without it
(27.6 against 7.6 MiB). That residue is not explained by the ignored-subtree heads,
which on this tree are few; what holds it is an open item for the maintainer, alongside
whether the RSS criterion was meant to bind on Linux or only on macOS.
