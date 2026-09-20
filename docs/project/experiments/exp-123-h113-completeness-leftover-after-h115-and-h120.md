---
title: H113 completeness leftover after H115 and H120
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-123
  title: H113 completeness leftover after H115 and H120
  date: "2026-09-19"
  hypotheses:
    - H113
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
    control: same probe at 01ccc4b8
    candidate: same probe self-comparison
    control_binary:
      name: control
      sha256: fd10aa87a8cd9701b91c2e21c469bf3c4ea066298cece44ba1230fe29495f54b
      size_bytes: 2536992
      args: []
    candidate_binary:
      name: candidate
      sha256: fd10aa87a8cd9701b91c2e21c469bf3c4ea066298cece44ba1230fe29495f54b
      size_bytes: 2536992
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-123-h113-leftover-profile.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 1116535874.5
          candidate_median: 1221305270.5
          control_p95_over_median: 2.977
          candidate_p95_over_median: 3.922
          change_pct: 3.263
          ci95_low_pct: -1.37
          ci95_high_pct: 13.315
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 832242917.0
          candidate_median: 930075208.5
          control_p95_over_median: 3.395
          candidate_p95_over_median: 3.96
          change_pct: 3.851
          ci95_low_pct: -1.788
          ci95_high_pct: 17.702
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 1096107000.0
          candidate_median: 1125776000.0
          control_p95_over_median: 2.186
          candidate_p95_over_median: 2.432
          change_pct: 1.218
          ci95_low_pct: -1.138
          ci95_high_pct: 5.864
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 1005212000.0
          candidate_median: 1021993500.0
          control_p95_over_median: 2.185
          candidate_p95_over_median: 2.426
          change_pct: 1.111
          ci95_low_pct: -0.069
          ci95_high_pct: 5.918
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 95873500.0
          candidate_median: 88813500.0
          control_p95_over_median: 2.084
          candidate_p95_over_median: 2.826
          change_pct: -3.846
          ci95_low_pct: -16.261
          ci95_high_pct: 16.226
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 20428874.5
          candidate_median: 27956458.0
          control_p95_over_median: 45.437
          candidate_p95_over_median: 73.386
          change_pct: 69.727
          ci95_low_pct: -52.306
          ci95_high_pct: 713.944
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 347971584.0
          candidate_median: 347406336.0
          control_p95_over_median: 1.03
          candidate_p95_over_median: 1.004
          change_pct: -0.21
          ci95_low_pct: -0.624
          ci95_high_pct: 0.304
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
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: no engine change; leftover profile only; file-count shortcut not compiled
  verdict:
    decision: accepted
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: 3.263
    reason: completeness walk still 16 percent of content_open after H115+H120; H113 stays open; shortcut not compiled
    commit: 01ccc4b8
---
## What was predicted

H113’s quiet confirmatory still cannot start on this host.
The leftover that justifies it is a second `analysis_candidates` walk in
`open_for_report` after a cache-only sidecar restore, sampled at 12.6% of `content_open`
in exp-109. H115 and H120 changed restore.
H121 named the restore mix (candidates 48%, apply 43%) and did not re-measure that
second walk.

This cell asks whether that walk is still at least 3% of `content-cache-hit` wall after
H115+H120. It does not compile the file-count shortcut.
It is not an uncontrolled H113 confirmatory.

Named before measuring:

- Determination: the completeness walk is still, or is no longer, ≥3% of deciding-scale
  `content-cache-hit` wall / `content_open`.
- If it is still ≥3%, H113 stays open and still needs a quiet host.
  exp-113 remains reserved.
- If it is under 3%, H113 can be retired without a quiet confirmatory.
- Attachment: 12-pair same-binary `content-cache-hit`, `FDU_COUNTERS` unset.
- Attribution: counters-on hits (restore mix) and a 20 s `/usr/bin/sample` (counters and
  oracle off).
- Quiet first for the pair.
  If the start gate fails, label uncontrolled.
  Do not lower the 25% bar.
- Do not compile the shortcut.
  Do not run uncontrolled H113.

Subject: frozen APFS clone of `metabrowser-clone` (live checkout has writers).
Experiment id exp-123. exp-113 remains reserved.
H125 is not minted.

## What was measured

H113 quiet start (this tick) refused at CPU busy **85.17% > 25.0%**. No shortcut pair.
The file-count shortcut was not compiled.
exp-113 unused. A later pre-pair busy check was 43.64%.

H107 hunt remains closed (exp-122). Ignore does not skip descent.

H122 leftover remains closed (exp-122). H125 not minted.

H123 product follow-through: none.
`query::report` / `Index.report()` is already the opened and refresh path.

Subject: frozen `metabrowser-clone` (146,047 entries / 133,708 files / 11,517
directories, max depth 19). Digest `dc0df263…`. The clone did not mutate.

Job: harness `content-cache-hit` after one `content-seed` per variant into isolated
scratch. 3 warmups, 12 timed pairs, interleaved, same probe both variants.
`FDU_COUNTERS` unset on the claim-grade pair.

Quiet was attempted for the H113 shortcut at the start of this tick (85.17% busy).
The leftover-profile pair was labeled **uncontrolled** after a later pre-pair busy check
of 43.64%. Official pair initial busy 18.37% / thermal `fair`; final busy 45.72% /
thermal `normal`. The 25% bar was not lowered.
No RAM disk.

Same release probe both variants (`fd10aa87…`, 2,536,992 bytes).
0 invalid samples. Every timed sample was `source=content-cache` with 133,708 cache hits
and 0 applied. Content digest `3be19a3e…`.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 1,116.5 ms | 832.2 ms | 331.9 MiB |
| candidate | 1,221.3 ms | 930.1 ms | 331.3 MiB |

Self-comparison +3.26% [−1.37%, +13.31%]. Attachment only.
Early pairs spiked (control warmup 9.0 s; candidate #00 7.7 s) on an uncontrolled host.

Seed (attribution, not the pair): 14.6 s component, 118,882 analysed / 133,708 applied,
same content digest.

A later `FDU_COUNTERS=1` hit is attribution only.
Three hits, digest identical.

## What restore and `content_open` spend time on

Median of the four phase timers on the three counters-on hits (R1: apply excludes
decode):

| Phase | Hit 1 µs | Hit 2 µs | Hit 3 µs |
| --- | ---: | ---: | ---: |
| read | 14,316 | 7,970 | 6,275 |
| parse | 62,281 | 43,856 | 34,559 |
| candidates | 386,744 | 252,863 | 224,479 |
| apply | 401,678 | 243,921 | 327,973 |

Host was busy; treat the mix as H121’s claim-grade split (candidates 47.6%, apply
42.7%), not these three noisy hits.

A 20-second `/usr/bin/sample` on the profiling build (`--repeat 25`, counters and oracle
off). Main-thread stacks 13,421; `content_open` 13,357:

| Inclusive node under `content_open` | Samples | Share of `content_open` |
| --- | ---: | ---: |
| `load_content` (lib.rs:775) | 6,891 | 51.6% |
| snapshot load (`open_for_report` lib.rs:562) | 4,037 | 30.2% |
| completeness walk (`open_for_report` lib.rs:601–602) | 2,136 | 16.0% |
| `analysis_candidates` directly under that walk | 2,039 | 15.3% |
| `analysis_candidates` inside `load_content` | 1,872 | 14.0% |

exp-109 sampled the same completeness node at 12.6% of `content_open`. It is still 16.0%
of `content_open` after H115+H120. On H121’s claim-grade split (component 866 ms of
1,172 ms wall) that is about 12% of wall, well above 3%.

## What the determination said

The second `analysis_candidates` walk is still the leftover H113 named.
H113 stays open and still needs a quiet host.
Do not compile the file-count shortcut on an uncontrolled cell.
exp-113 remains reserved.

Do not mint H125. Do not start H111 on this host.
Do not start H107 without an ignore-is-the-walk subject.

No README 200K / 4M change.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
