---
title: Post-H125 cache-hit leftover after restore-count completeness
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-125
  title: Post-H125 cache-hit leftover after restore-count completeness
  date: "2026-09-19"
  hypotheses:
    - H126
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
    control: H125 release probe at be8d4d69
    candidate: same-source rebuild of HEAD
    control_binary:
      name: control
      sha256: c86ad8cbeeec5a1cecc2b5b6f128a4913d3fec6e643ee8b569fb693001a3090f
      size_bytes: 2536992
      args: []
    candidate_binary:
      name: candidate
      sha256: 33286ed51072a5095ee8d34d1095c090b124fb2748a0840d1af91a8873a25c81
      size_bytes: 2536992
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-125-h126-post-h125-leftover-profile.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1064856979.5
          candidate_median: 1038846187.0
          control_p95_over_median: 1.191
          candidate_p95_over_median: 1.027
          change_pct: -0.309
          ci95_low_pct: -6.152
          ci95_high_pct: 0.381
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 748014229.0
          candidate_median: 738964562.5
          control_p95_over_median: 1.091
          candidate_p95_over_median: 1.042
          change_pct: -0.264
          ci95_low_pct: -3.942
          ci95_high_pct: 0.94
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 1006191500.0
          candidate_median: 999105000.0
          control_p95_over_median: 1.035
          candidate_p95_over_median: 1.009
          change_pct: -0.15
          ci95_low_pct: -3.371
          ci95_high_pct: 0.628
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 917736000.0
          candidate_median: 911935500.0
          control_p95_over_median: 1.029
          candidate_p95_over_median: 1.004
          change_pct: -0.405
          ci95_low_pct: -1.673
          ci95_high_pct: -0.087
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 85651500.0
          candidate_median: 87936500.0
          control_p95_over_median: 1.264
          candidate_p95_over_median: 1.078
          change_pct: 0.642
          ci95_low_pct: -14.241
          ci95_high_pct: 8.558
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 51981479.0
          candidate_median: 40409520.5
          control_p95_over_median: 4.241
          candidate_p95_over_median: 1.552
          change_pct: -18.896
          ci95_low_pct: -45.959
          ci95_high_pct: 2.965
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 348610560.0
          candidate_median: 358047744.0
          control_p95_over_median: 1.032
          candidate_p95_over_median: 1.003
          change_pct: 2.665
          ci95_low_pct: -0.037
          ci95_high_pct: 2.809
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
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: no engine change; leftover profile only
  verdict:
    decision: accepted
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: -0.309
    reason: completeness walk gone after H125; first candidates walk remains; no new cut
    commit: 25f423fd
---
## What was predicted

H125 removed the second `analysis_candidates` walk from cache-only completeness.
H121’s restore mix (candidates 48%, apply 43%) is now stale as a picture of
`content_open`, because that walk sat *after* `load_content`.

This cell is a leftover profile, not a cut.

Named before measuring:

- Determination: the `open_for_report` completeness walk is under 3% of `content_open`
  (or absent), and the named remaining leftover is or is not a userspace stage ≥3% that
  is not already rejected.
- If the completeness node is gone and no new ≥3% userspace cut appears, do not mint an
  engine patch. H116 still owns the first candidates walk.
- Attachment: 12-pair same-source `content-cache-hit`, `FDU_COUNTERS` unset.
- Attribution: counters-on hits and a 20 s `/usr/bin/sample`.

Subject: frozen APFS clone of `metabrowser-clone`. Experiment id exp-125. H126. No
engine change.

## What was measured

H113 quiet this earlier tick refused at 45.48%. H125 accepted.
File-count not compiled.

Subject: frozen `metabrowser-clone` (146,047 entries / 133,708 files / 11,517
directories). Digest `dc0df263…`. The clone did not mutate.

Job: harness `content-cache-hit` after one `content-seed` per variant.
3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset on the pair.

Labeled **uncontrolled**. Initial busy 39.98%; final 41.18%. The 25% bar was not
lowered. No RAM disk.

Control is the H125 release probe (`c86ad8cb…`). Candidate is a same-source rebuild of
HEAD (`33286ed5…`). 0 invalid samples.
Self-comparison only.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,064.9 ms | 748.0 ms | 332.5 MiB |
| candidate | 1,038.8 ms | 739.0 ms | 341.5 MiB |

Wall −0.31% [−6.15%, +0.38%]. Attachment only.

Every timed sample was `source=content-cache` with 133,708 cache hits and 0 applied.
Content digest `3be19a3e…`.

Seed (attribution): 9.95 s component, 118,882 analysed / 133,708 applied, same digest.

## What restore and `content_open` spend time on

Three `FDU_COUNTERS=1` hits after H125 (apply excludes decode):

| Phase | Hit 1 µs | Hit 2 µs | Hit 3 µs |
| --- | ---: | ---: | ---: |
| read | 5,526 | 5,511 | 5,873 |
| parse | 26,152 | 25,795 | 25,159 |
| candidates | 167,857 | 165,879 | 165,526 |
| apply | 149,175 | 139,097 | 146,979 |

Restore mix is unchanged from H121: candidates ~48%, apply ~43%, parse ~7.5%.

A 20-second `/usr/bin/sample` on the profiling build (`--repeat 25`, counters and oracle
off). Main-thread `content_open` 15,296 samples:

| Inclusive node under `content_open` | Samples | Share of `content_open` |
| --- | ---: | ---: |
| `load_content` (lib.rs:773) | 9,707 | 63.5% |
| snapshot load (`open_for_report` lib.rs:562) | 5,570 | 36.4% |
| completeness compare (`open_for_report` lib.rs:594) | 1 | 0.007% |
| `analysis_candidates` (first restore walk only) | 2,400 | 15.7% |

exp-123 sampled the completeness walk at 16.0% of `content_open`. It is gone.

## What the determination said

The second walk is gone.
The remaining leftover is the first candidates walk inside restore (H116, rejected on
wall) plus apply. Snapshot load rose as a *share* because completeness left.
No new userspace cut ≥3%. No engine patch.

Do not retry H116. Do not mint a snapshot-parse cut (H78/H92).

Do not raise the README 200K files/s or 4M cached lines/s.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
