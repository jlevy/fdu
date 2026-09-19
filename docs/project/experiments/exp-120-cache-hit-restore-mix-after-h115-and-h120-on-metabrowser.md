---
title: Cache-hit restore mix after H115 and H120 on metabrowser
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-120
  title: Cache-hit restore mix after H115 and H120 on metabrowser
  date: "2026-09-19"
  hypotheses:
    - H121
  subject:
    tree_label: metabrowser-clone
    tree_root_id: 3b5427f76be06cb475a2ea5c609bcd70f5d5f5b8af1280375c6a77b558513f50
    tree_engine_digest: dc0df2630acc6f604f7b76495f8214220d75fc600ac534d8c056f0210185ac5f
    tree_provenance: "An APFS copy-on-write clone of this host's github.com/jlevy/metabrowser checkout, taken 2026-09-19 after concurrent writers mutated the live path during the first pair. Same shape as the live workspace at copy time. Not reconstructible."
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
    control: same probe at ee014340
    candidate: same probe self-comparison
    control_binary:
      name: control
      sha256: 23d9c14e72ddb4ad23e3bf9148ec20e9c9546154c6709d97008676f226fd7db1
      size_bytes: 2536992
      args: []
    candidate_binary:
      name: candidate
      sha256: 23d9c14e72ddb4ad23e3bf9148ec20e9c9546154c6709d97008676f226fd7db1
      size_bytes: 2536992
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-120-h121-cache-hit-reprofile.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1171780958.0
          candidate_median: 1185593479.0
          control_p95_over_median: 1.045
          candidate_p95_over_median: 1.293
          change_pct: 0.742
          ci95_low_pct: -1.243
          ci95_high_pct: 4.792
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 866362563.0
          candidate_median: 859926958.0
          control_p95_over_median: 1.026
          candidate_p95_over_median: 1.043
          change_pct: -0.896
          ci95_low_pct: -2.477
          ci95_high_pct: 0.753
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 1119832500.0
          candidate_median: 1121423500.0
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.019
          change_pct: 0.453
          ci95_low_pct: -0.595
          ci95_high_pct: 1.577
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 1025274500.0
          candidate_median: 1025383500.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.011
          change_pct: -0.125
          ci95_low_pct: -0.497
          ci95_high_pct: 0.552
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 95360000.0
          candidate_median: 92480500.0
          control_p95_over_median: 1.167
          candidate_p95_over_median: 1.272
          change_pct: 6.268
          ci95_low_pct: -6.191
          ci95_high_pct: 10.848
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 55910583.0
          candidate_median: 60092313.0
          control_p95_over_median: 1.601
          candidate_p95_over_median: 6.16
          change_pct: 12.785
          ci95_low_pct: -19.346
          ci95_high_pct: 80.729
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 349470720.0
          candidate_median: 349528064.0
          control_p95_over_median: 1.028
          candidate_p95_over_median: 1.029
          change_pct: 0.087
          ci95_low_pct: -0.408
          ci95_high_pct: 0.48
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
        reasons:
          - "voluntary_context_switches straddles its +50% regression limit"
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
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: profile only; timers already in; no engine change
  verdict:
    decision: accepted
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: 0.742
    reason: "apply 43 percent of restore after H115+H120, candidates 48 percent; no stage at 50 percent; no apply cut"
    commit: ee014340
---
## What was predicted

H115 accepted a restore-only bottom-up roll-up.
H120 accepted streaming sidecar parse-into-apply.
exp-109’s mix (apply 63%, candidates 25%, parse 8.5%) is therefore stale.

H121: after those two accepts, a same-subject `content-cache-hit` re-profile names
whether apply still dominates.

Named before measuring:

- Determination: a named restore stage is at least 50% of restore phase time and at
  least 3% of claim-grade wall.
- Attachment: 12-pair same-binary `content-cache-hit`, `FDU_COUNTERS` unset.
- Timers already in (R1 isolates apply from decode).
  Attribution is counters-on hits.
- If apply no longer dominates, do not start an apply cut (H83 scoped down).
- Not another alloc trim.
  Not a retry of H116.

Subject: deciding-scale metabrowser.
The live checkout was being written (npm / pytest / Cursor agent) and mutated the first
pair. The recorded cell is an APFS copy-on-write clone of that checkout, taken after the
failed live pair.

## What was measured

Subject: frozen `metabrowser-clone` (146,047 entries / 133,708 files / 11,517
directories, max depth 19). Digest `dc0df263…`. The clone did not mutate.

Job: harness `content-cache-hit` after one `content-seed` per variant into isolated
scratch. 3 warmups, 12 timed pairs, interleaved, same probe both variants (sha256
`23d9c14e…`). `FDU_COUNTERS` unset on the claim-grade pair.

Quiet was attempted.
The start gate refused at 100% CPU busy.
The pair ran as **uncontrolled**. Initial busy 100.0%; final 70.86%. The 25% bar was not
lowered. No RAM disk.

0 invalid samples. Every timed sample was `source=content-cache` with 133,708 cache hits
and 0 applied. Content digest `3be19a3e…`.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,171.8 ms | 866.4 ms | 333.3 MiB |
| candidate | 1,185.6 ms | 859.9 ms | 333.3 MiB |

Same-binary wall +0.74% [−1.24%, +4.79%]. Attachment only.

A later `FDU_COUNTERS=1` hit is attribution only.
Three hits, digest identical to the pair.

## What restore spends time on

Median of the four phase timers (R1: apply excludes decode):

| Phase | Median µs | Share of restore |
| --- | ---: | ---: |
| read | 5,525 | 1.6% |
| parse | 25,778 | 7.3% |
| candidates | 166,967 | 47.6% |
| apply | 149,777 | 42.7% |

Apply is 42.7% of restore (about 12.8% of claim-grade wall).
Candidates is the largest slice at 47.6% (about 14.2% of wall).
Neither is at least 50% of restore.
Parse remains about 7%.

## What the accept rule said

A named stage dominates only if it is at least 50% of restore *and* at least 3% of wall.
No stage clears the first clause.
H121 does not name an apply cut.

H116 already rejected dropping the candidate HashMap on wall.
Do not retry that cut because candidates is now the largest slice.

## Judgment

H121 is confirmed as a determination: the post-H115+H120 mix no longer has a dominating
apply stage. H83 is scoped down on apply/install.
Do not start an apply cut from this cell.
Do not retry H116.

H115 remains the standing *speed* best.
H120 remains the standing content-hit RSS best.

Do not raise the README 200K files/s or 4M cached lines/s from this cell.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
