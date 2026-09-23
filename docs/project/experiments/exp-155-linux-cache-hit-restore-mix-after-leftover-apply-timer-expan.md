---
title: Linux cache-hit restore mix after leftover apply-timer expansion
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-155
  title: Linux cache-hit restore mix after leftover apply-timer expansion
  date: "2026-09-21"
  hypotheses:
    - H149
  subject:
    tree_label: linux-v6.12
    tree_root_id: d7c0dad8d82c8bb394f459b1dd77c9e8af60d0482cca92199519668546a5147e
    tree_engine_digest: a298a9c2c8f8ee910d22b87093a739993cdf153104590f4638eeee740b239bd0
    tree_provenance: "Shallow clone of github.com/torvalds/linux tag v6.12 at adc218676eef25575469234709c2d87185ca223a. Reconstructible: git clone --depth 1 --branch v6.12 https://github.com/torvalds/linux.git. Shape is the published v6.12 source tree plus the clone's .git directory as git left it; no extra workspace install."
    tree_reconstructible: true
    tree_entries: 92474
    tree_directories: 5769
    tree_files: 86643
    tree_symlinks: 62
    tree_apparent_bytes: 1759293224
    tree_allocated_bytes: 1965477888
    tree_max_depth: 14
    tree_mutated_during_run: false
    host_cpu: Intel(R) Xeon(R) Processor
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16791945216
    host_system: Linux 6.12.94+
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 12
    warmups: 3
    interleaved: true
    control: same leftover-timer release probe both arms
    candidate: same probe leftover restore-mix profile
    control_binary:
      name: control
      sha256: d7b1fc9e6d55ec4feb5d84d8c415033d60f7d7839258dc7a1cc44d9896b7e38a
      size_bytes: 3039592
      args: []
    candidate_binary:
      name: candidate
      sha256: d7b1fc9e6d55ec4feb5d84d8c415033d60f7d7839258dc7a1cc44d9896b7e38a
      size_bytes: 3039592
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-155-h149-linux-cache-hit-timer-mix.json
  results:
    - job: content-cache-hit
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 586808098.5
          candidate_median: 587930805.0
          control_p95_over_median: 1.022
          candidate_p95_over_median: 1.017
          change_pct: 0.039
          ci95_low_pct: -0.381
          ci95_high_pct: 0.593
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 500811074.5
          candidate_median: 502272532.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.018
          change_pct: 0.333
          ci95_low_pct: -0.478
          ci95_high_pct: 0.956
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 586213500.0
          candidate_median: 587372500.0
          control_p95_over_median: 1.023
          candidate_p95_over_median: 1.017
          change_pct: 0.003
          ci95_low_pct: -0.361
          ci95_high_pct: 0.595
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 535088000.0
          candidate_median: 533001500.0
          control_p95_over_median: 1.044
          candidate_p95_over_median: 1.021
          change_pct: -0.919
          ci95_low_pct: -3.204
          ci95_high_pct: 1.013
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 52192000.0
          candidate_median: 51962500.0
          control_p95_over_median: 1.302
          candidate_p95_over_median: 1.389
          change_pct: 0.003
          ci95_low_pct: -10.753
          ci95_high_pct: 26.898
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        blocked_ns:
          control_median: 490859.0
          candidate_median: 460529.5
          control_p95_over_median: 1.308
          candidate_p95_over_median: 1.293
          change_pct: -7.664
          ci95_low_pct: -23.77
          ci95_high_pct: 10.005
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 156243968.0
          candidate_median: 156256256.0
          control_p95_over_median: 1.001
          candidate_p95_over_median: 1.001
          change_pct: 0.001
          ci95_low_pct: -0.059
          ci95_high_pct: 0.063
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
    notes: "same-binary attachment: both arms ran the 065175ee probe, so 0 lines is the attachment's cost, not that commit's; 065175ee (apply-timer expansion plus a dead-branch removal, 35 production lines added in content_cache.rs and index.rs, tests excluded) was never paired against its parent e95167b9, so its wall effect is unmeasured; the timers are Option-gated and off by default"
  verdict:
    decision: accepted
    primary_job: content-cache-hit
    primary_metric: wall_ns
    change_pct: 0.039
    reason: "same leftover identity as H144: apply 60-62% of restore after timer expansion is H116 HashMap now in-bucket; no new compileable cut; do not retry H116"
    commit: "065175ee"
---
## What was predicted

H149 is the Linux cache-hit restore mix after leftover `fdu-2pct` moved apply start to
`candidates.remove`. H144 (exp-144) named the leftover under the narrower apply-only
bucket that arrived with `e667b739` (HashMap remove and fingerprint outside apply):
apply ~80–84 ms, parse and candidates ~27 ms, read ~9 ms; no new ≥3% userspace cut.
H121 said not to judge another apply/install cut until the four restore rows can sum to
the load.

Named before measuring:

- Job: same-binary 12-pair `content-cache-hit` on reconstructible `linux-v6.12`.
- Phase split from counters-on hits (`content_sidecar_{read,parse,candidates,apply}_us`)
  with apply including HashMap remove and fingerprint compare.
- Determination: a named restore stage is or is not ≥50% of restore and ≥3% of
  claim-grade wall, or the leftover is still the identity H144 recorded.
- Do not retry H116, H125, H129, H131, or H133.
- Quiet first. If the gate fails or samples invalidate, label uncontrolled.
  Do not lower the 25% bar.

## What was measured

Quiet held: initial load/core 0.059, final 0.102, 0 of 30 samples invalidated.
`FDU_COUNTERS` unset on the pair.
3 warmups, 12 timed pairs, interleaved.
Same leftover-timer release probe both arms (sha256 `d7b1fc9e…`, commit `065175ee`). No
RAM disk.

Subject: the same frozen `linux-v6.12` clone as exp-138 through exp-154 (92,474 entries
/ 86,643 files / 5,769 directories).
Fingerprint and engine digest unchanged.
Content digest `06260f6c…`. 86,643 cache hits / 0 applied.
Every timed sample was `source=content-cache`.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 586.8 ms | 500.8 ms | 149.0 MiB |
| candidate | 587.9 ms | 502.3 ms | 149.0 MiB |

Self-comparison wall +0.04% [−0.38%, +0.59%]. Attachment only.

Three `FDU_COUNTERS=1` hits after the pair (apply starts at `candidates.remove`):

| Phase | Hit 1 µs | Hit 2 µs | Hit 3 µs |
| --- | ---: | ---: | ---: |
| read | 9,253 | 9,188 | 8,396 |
| parse | 26,180 | 26,098 | 25,680 |
| candidates | 26,424 | 26,129 | 24,179 |
| apply | 99,181 | 97,791 | 88,981 |

Apply is 60–62% of the four restore rows (89–99 ms).
That is ≥50% of restore and about 15–17% of the 587 ms wall.
Parse and candidates stay ~24–26 ms each (~4% of wall).
Read is ~8–9 ms, under 3% of wall.
Versus H144, apply grew about 10–15 ms and the other three rows did not; that is the
previously unbucketed HashMap remove plus fingerprint.
This is unpaired attribution across two sessions and two binaries, not a measured delta.
438,021 roll-up merges.
File opens 0. Walk 0.

`perf` and `/usr/bin/sample` are not on this image, so there is no `content_open` symbol
split. The phase timers are the leftover-expanded instrument.

## What the determination said

**Same** leftover identity as H144 / Darwin H134. Apply now clears the H121
restore-share bar because remove and fingerprint sit in that bucket.
The newly attributed work is H116’s HashMap path, rejected on wall.
Remaining cost is already-landed restore work (H115 apply rebuild, H120 streaming
parse-into-apply, H129 classify skip, H125 completeness) and already-rejected stages
(H116, parse-speed).
No new compileable userspace cut.

No further engine patch.
Do not retry H116. Do not retry H125/H129/H131/H133. Do not raise the README 200K
files/s or 4M cached lines/s. H83 remains format (H78/H92), not another apply/install
increment from this mix.
