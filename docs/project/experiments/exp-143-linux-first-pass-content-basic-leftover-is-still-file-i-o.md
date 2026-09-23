---
title: Linux first-pass content-basic leftover is still file I/O
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-143
  title: Linux first-pass content-basic leftover is still file I/O
  date: "2026-09-20"
  hypotheses:
    - H142
  subject:
    tree_label: linux-v6.12
    tree_root_id: d7c0dad8d82c8bb394f459b1dd77c9e8af60d0482cca92199519668546a5147e
    tree_engine_digest: a298a9c2c8f8ee910d22b87093a739993cdf153104590f4638eeee740b239bd0
    tree_provenance: "Shallow clone of github.com/torvalds/linux tag v6.12 at adc218676eef25575469234709c2d87185ca223a. Reconstructible: git clone --depth 1 --branch v6.12 https://github.com/torvalds/linux.git. Shape is the published v6.12 source tree plus the clone .git directory as git left it."
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
    control: HEAD release probe both arms
    candidate: same probe leftover profile
    control_binary:
      name: control
      sha256: 84060ec54c559a261960c5e7659020da0f045ef9733e54a0daf72b592e9d7e82
      size_bytes: 3030680
      args: []
    candidate_binary:
      name: candidate
      sha256: 84060ec54c559a261960c5e7659020da0f045ef9733e54a0daf72b592e9d7e82
      size_bytes: 3030680
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-143-h142-linux-content-basic-leftover-uncontrolled.json
  results:
    - job: content-basic
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2165735522.5
          candidate_median: 2169362545.5
          control_p95_over_median: 1.01
          candidate_p95_over_median: 1.005
          change_pct: 0.447
          ci95_low_pct: -0.168
          ci95_high_pct: 0.575
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 1633343787.0
          candidate_median: 1632526783.5
          control_p95_over_median: 1.002
          candidate_p95_over_median: 1.009
          change_pct: 0.089
          ci95_low_pct: -0.324
          ci95_high_pct: 0.654
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 6803740000.0
          candidate_median: 6802674000.0
          control_p95_over_median: 1.002
          candidate_p95_over_median: 1.004
          change_pct: 0.172
          ci95_low_pct: -0.121
          ci95_high_pct: 0.439
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 5967280000.0
          candidate_median: 5974865000.0
          control_p95_over_median: 1.005
          candidate_p95_over_median: 1.021
          change_pct: 0.689
          ci95_low_pct: -0.446
          ci95_high_pct: 1.803
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 841467000.0
          candidate_median: 835643500.0
          control_p95_over_median: 1.064
          candidate_p95_over_median: 1.012
          change_pct: -5.285
          ci95_low_pct: -10.163
          ci95_high_pct: 2.031
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 144881664.0
          candidate_median: 144740352.0
          control_p95_over_median: 1.001
          candidate_p95_over_median: 1.001
          change_pct: -0.034
          ci95_low_pct: -0.165
          ci95_high_pct: 0.027
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
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
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: leftover profile only; no engine change
  verdict:
    decision: accepted
    primary_job: content-basic
    primary_metric: wall_ns
    change_pct: 0.447
    reason: "first-pass leftover is still file I/O (86634 opens, 184057 reads, ~2.12/file); no skippable 3% userspace cut; do not retry H124"
    commit: 937f9445
---
## What was predicted

H142 is the Linux leftover after H124 rejected a type/size gate and a larger read chunk
on first-pass `content-basic`. H118 rejected insert-then-rebuild.
H119 screened walk-overlap.
Darwin H135 (exp-134) found the leftover still file I/O.

Named before measuring:

- Determination: after H124, Linux first-pass leftover is still file I/O, or a userspace
  stage at least 3% Darwin did not see.
- Do not retry type/size or read-ahead.
  Do not retry H118 or H119.
- Attachment: 12-pair same-source `content-basic`, `FDU_COUNTERS` unset.
- Attribution: counters-on hits (`file opens`, `file read calls`, bytes read).
  `perf` / `strace` / `/usr/bin/sample` are not on this image.
- Quiet first. If the gate fails or samples invalidate, label uncontrolled.
  Do not lower the 25% bar.

## What was measured

Quiet start passed (load/core 0.036). The cell did not hold: `content-basic` is a
parallel analyze, and Linux load average remembered that work.
24 of 30 samples invalidated; 0 valid timed pairs.
That incomplete quiet run is not a verdict and was not topped up.
Saved as `run-exp-143-h142-linux-content-basic-leftover.json`.

The claim-grade pair is **uncontrolled**. Same HEAD probe both arms (sha256 `84060ec5…`
/ 3,030,680 bytes). 3 warmups, 12 timed pairs, interleaved.
`FDU_COUNTERS` unset.
Initial load/core 0.265; final 0.729. The 0.25 bar was not lowered.
No RAM disk.

Subject: the same frozen `linux-v6.12` clone as exp-138–140 (92,474 entries / 86,643
files / 5,769 directories).
Fingerprint unchanged.
0 invalid samples on the uncontrolled pair.

Host: 4-core KVM Intel Xeon, 16 GiB, Linux 6.12.94+, ext4, virtualized.
`os_cache: warm-steady`.

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 2,165.7 ms | 1,633.3 ms | 138.2 MiB |
| candidate | 2,169.4 ms | 1,632.5 ms | 138.0 MiB |

Self-comparison wall +0.45% [−0.17%, +0.57%]. Attachment only.

Every timed sample applied 86,643 / analyzed 86,628 / binary 13. Content digest
`06260f6c…` matches exp-138–140. Source `scan`. Cache hits 0.

## What first-pass `content-basic` spends time on

Three `FDU_COUNTERS=1` hits (scan is setup; sidecar timing stays 0):

| Hit | Component ms | File opens | Read calls | Bytes read | Walk µs |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 1,639.6 | 86,634 | 184,057 | 1,476,447,258 | 434,513 |
| 2 | 1,640.9 | 86,634 | 184,057 | 1,476,447,258 | 446,784 |
| 3 | 1,650.5 | 86,634 | 184,057 | 1,476,447,258 | 443,924 |

Opens are 86,634 / 86,643 files (13 path-binary; 2 invalid UTF-8). Reads 184,057 /
86,634 opens (~2.12/file), the same shape as Darwin H124/H135 (~2/file).
Walk stays ~0.44 s against a 1.64 s component.

`perf` and `strace` are not installed, so classify / `commit_record` shares are not a
sampled ground truth here.
The counters already name the leftover: every admitted file is opened and read.
A type/size gate or a larger chunk cannot clear 3% after H124.

## What the determination said

**Same** leftover identity as Darwin H135. First-pass leftover is still file I/O
(`openat` / `read`), not a skippable userspace stage ≥3%.

No engine patch.
Do not retry H118. Do not retry H119. Do not retry H124. Do not invent a
first-pass classify skip.
Do not raise the README 200K files/s or 4M cached lines/s.
