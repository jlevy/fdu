---
title: Post-H128 first-run default-tree leftover
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-135
  title: Post-H128 first-run default-tree leftover
  date: "2026-09-19"
  hypotheses:
    - H136
  subject:
    tree_label: metabrowser-clone
    tree_root_id: 3b5427f76be06cb475a2ea5c609bcd70f5d5f5b8af1280375c6a77b558513f50
    tree_engine_digest: dc0df2630acc6f604f7b76495f8214220d75fc600ac534d8c056f0210185ac5f
    tree_provenance: "An APFS copy-on-write clone of this host's github.com/jlevy/metabrowser checkout, taken 2026-09-19 after concurrent writers mutated the live path. Same shape as the live workspace at copy time. Not reconstructible."
    tree_reconstructible: false
    tree_entries: 146047
    tree_directories: 11517
    tree_files: 133708
    tree_symlinks: 822
    tree_apparent_bytes: 1726062376
    tree_allocated_bytes: 2060058624
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
    control: HEAD release probe at 2aa3b7ee
    candidate: same probe (leftover profile)
    control_binary:
      name: control
      sha256: 8765aa6f198cacf94027411ea9151125b58af2ac4458c9f45b782ec1bfc71124
      size_bytes: 2553504
      args: []
    candidate_binary:
      name: candidate
      sha256: 8765aa6f198cacf94027411ea9151125b58af2ac4458c9f45b782ec1bfc71124
      size_bytes: 2553504
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-135-h136-default-tree-first-leftover.json
  results:
    - job: default-tree-first
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 480951833.5
          candidate_median: 453571916.5
          control_p95_over_median: 1.482
          candidate_p95_over_median: 2.505
          change_pct: 0.182
          ci95_low_pct: -15.018
          ci95_high_pct: 38.621
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 472441708.0
          candidate_median: 444906042.0
          control_p95_over_median: 1.474
          candidate_p95_over_median: 2.534
          change_pct: -3.047
          ci95_low_pct: -15.014
          ci95_high_pct: 36.101
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1655608000.0
          candidate_median: 1606621000.0
          control_p95_over_median: 1.064
          candidate_p95_over_median: 1.085
          change_pct: -1.851
          ci95_low_pct: -9.225
          ci95_high_pct: 1.414
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 162282000.0
          candidate_median: 166549000.0
          control_p95_over_median: 1.055
          candidate_p95_over_median: 1.023
          change_pct: 1.771
          ci95_low_pct: -1.046
          ci95_high_pct: 4.134
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 1489263500.0
          candidate_median: 1435536000.0
          control_p95_over_median: 1.072
          candidate_p95_over_median: 1.098
          change_pct: -2.507
          ci95_low_pct: -10.113
          ci95_high_pct: 1.663
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 56852480.0
          candidate_median: 57049088.0
          control_p95_over_median: 1.046
          candidate_p95_over_median: 1.057
          change_pct: 0.346
          ci95_low_pct: -1.884
          ci95_high_pct: 2.55
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
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
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: leftover profile only; no engine change
  verdict:
    decision: accepted
    primary_job: default-tree-first
    primary_metric: wall_ns
    change_pct: 0.182
    reason: "first-run leftover after H128 is still the walk (83-88%); isolated snapshot save 45ms (~11-16% of first-run) is >=3% but not skippable (H100/H78/H92/fsync); no engine patch"
    commit: 2aa3b7ee
---
## What was predicted

H128 confirmed the second `fdu PATH` is still a cold walk (92.9% of component,
`snapshot_written` false).
H122 holds on file-heavy metabrowser.
H100 already skips an identical rewrite.
H135 closed the first-pass analyze leftover hunt.

This cell is a leftover profile on first-run `default-tree-first`, not a cache-hit skip
and not a snapshot load on `fdu PATH`.

Named before measuring:

- Determination: after H128, snapshot encode/write/render is or is not a stage ≥3% of
  first-run wall that is not already rejected (H100, H101, H78/H92, `fdu-n75m`).
- If no skippable ≥3% mechanism appears, do not compile a cut in this cell.
- Attachment: 12-pair same-source `default-tree-first`, `FDU_COUNTERS` unset.
- Attribution: three counters-on first-run hits plus three isolated `snapshot-save` hits
  (scan outside the save timer).

Subject: frozen APFS clone of `metabrowser-clone`. Experiment id exp-135. H136. No
engine change.

Quiet start this tick refused at CPU busy 75.4% > 25.0%. Tried once; skipped.
Uncontrolled. Do not lower the 25% bar.

## What was measured

Subject: frozen `metabrowser-clone` (146,047 entries / 133,708 files / 11,517
directories). Digest `dc0df263…`. The clone did not mutate.

Job: harness `default-tree-first` (no snapshot present; cache policy Auto; tree render;
save joined). 3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset on the pair.

Labeled **uncontrolled**. Official quiet check 75.4% busy.
Pair initial 46.59%; final 88.11%. Thermal `normal`. The 25% bar was not lowered.
No RAM disk.

Same copied HEAD release probe both variants (`8765aa6f…` / 2,553,504 bytes).
0 invalid samples. Self-comparison only.
Every timed sample had `snapshot_written` true, `source=scan`, snapshot 10,549,343
bytes.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 481.0 ms | 472.4 ms | 54.2 MiB |
| candidate | 453.6 ms | 444.9 ms | 54.4 MiB |

Wall +0.18% [−15.02%, +38.62%]. Attachment only.
Interval includes zero.

## What first-run `default-tree-first` spends time on

Three `FDU_COUNTERS=1` first-run hits (snapshot deleted before each):

| Hit | Component ms | Walk ms | Walk / component | Written |
| ---: | ---: | ---: | ---: | --- |
| 1 | 421.3 | 372.3 | 88.4% | true |
| 2 | 320.4 | 272.7 | 85.1% | true |
| 3 | 289.5 | 241.9 | 83.5% | true |

Opens 11,517; `getattrlistbulk` 22,480 (1.952 / dir including EOF). Finish 2–4 ms.

Three isolated `snapshot-save` hits (scan is setup; timer is encode + `publish` only):
45.3 ms, 43.7 ms, 46.7 ms.
Median 45.3 ms of a 10.5 MiB image.
That is 10.8–15.7% of the matching first-run component hits.

Render plus join residue after walk and that save is a few milliseconds.

No new 20 s sample of the walk: H127 already sampled it (`__open` 47%, `getattrlistbulk`
33%). The new fact is the first-run write share.

## What the determination said

First-run leftover after H128 is still the walk (83–88% of component).
Snapshot encode/write is a real ≥3% slice of first-run wall (~45 ms / ~11–16%) and is
not skippable: the image does not exist yet, so H100’s identical-rewrite skip does not
apply. Speeding encode is H78/H92 (format).
Dropping `F_FULLFSYNC` is `fdu-n75m` (person-gated).
No unused `path_of` in `save` (`name_of` only).
No engine patch in this cell.

Do not retry H100. Do not load a snapshot on `fdu PATH`. Do not mint a Darwin walk cut.
Do not mint a snapshot-format cut.
Do not invent another cache-hit skip.

Do not raise the README 200K files/s or 4M cached lines/s.
