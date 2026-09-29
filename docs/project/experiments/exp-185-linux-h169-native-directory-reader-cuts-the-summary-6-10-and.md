---
title: "Linux H169 native directory reader cuts the summary 6-10% and the tree 4% on node-modules-dense"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-185
  title: "Linux H169 native directory reader cuts the summary 6-10% and the tree 4% on node-modules-dense"
  date: "2026-09-29"
  hypotheses:
    - H169
  subject:
    tree_label: node-modules-dense
    tree_root_id: db6d49b68990d6b3a741e6289f360187d4317abfdda8be3f70915f799eb7acd4
    tree_engine_digest: 11c4122bec53693180e2e240899b75cdccc707ca9759b9acfadb80a0b19bafac
    tree_provenance: "npm dependency trees of react-scripts 5.0.1 and gatsby 5.13.7: copy docs/project/experiments/evidence/exp-190/subject-package.json.txt and subject-package-lock.json.txt to package.json and package-lock.json in an empty directory and run npm ci --ignore-scripts (node 22.22.2, npm 10.9.7); rebuilt 2026-09-29 on this host: 79,957 entries (exp-190 recorded 79,953), 9,439 directories, no .gitignore."
    tree_reconstructible: true
    tree_entries: 79957
    tree_directories: 9439
    tree_files: 70411
    tree_symlinks: 107
    tree_apparent_bytes: 538992241
    tree_allocated_bytes: 757870592
    tree_max_depth: 12
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.10GHz"
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16876515328
    host_system: Linux 6.18.44-fc-v49
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 20
    warmups: 3
    interleaved: true
    control: "70c2725c probe: H180 head"
    candidate: "20933081 probe: H169 Linux-native reader"
    control_binary:
      name: control-blind
      sha256: 9430e7379bb91acefc51c26a4ee3b1491a976beeb3a4dd9a0320e6223b3cc670
      size_bytes: 3851832
      args:
        - "--no-controls"
    candidate_binary:
      name: h169-blind
      sha256: 1b69d391a91fdb3296d51c97a396d2365731941ab631ef333f03ff0a016882b8
      size_bytes: 3861144
      args:
        - "--no-controls"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-185/run.json
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 70326419.0
          candidate_median: 66693264.5
          control_p95_over_median: 1.137
          candidate_p95_over_median: 1.05
          change_pct: -6.251
          ci95_low_pct: -14.022
          ci95_high_pct: -1.231
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 67287646.0
          candidate_median: 63532617.5
          control_p95_over_median: 1.136
          candidate_p95_over_median: 1.059
          change_pct: -6.909
          ci95_low_pct: -14.473
          ci95_high_pct: -1.486
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 251647500.0
          candidate_median: 234204500.0
          control_p95_over_median: 1.141
          candidate_p95_over_median: 1.083
          change_pct: -4.899
          ci95_low_pct: -12.46
          ci95_high_pct: -1.499
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 65341500.0
          candidate_median: 46111500.0
          control_p95_over_median: 1.162
          candidate_p95_over_median: 1.262
          change_pct: -30.337
          ci95_low_pct: -36.795
          ci95_high_pct: -21.379
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 186536000.0
          candidate_median: 191599000.0
          control_p95_over_median: 1.231
          candidate_p95_over_median: 1.174
          change_pct: 0.971
          ci95_low_pct: -7.55
          ci95_high_pct: 7.832
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - peak_rss_bytes is missing a paired percent interval
          - "minor_faults straddles its +10% regression limit"
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
          minor_faults: inconclusive
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 74159846.5
          candidate_median: 73999729.5
          control_p95_over_median: 1.072
          candidate_p95_over_median: 1.041
          change_pct: -1.673
          ci95_low_pct: -3.637
          ci95_high_pct: 2.057
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        component_ns:
          control_median: 70726596.0
          candidate_median: 70461125.0
          control_p95_over_median: 1.072
          candidate_p95_over_median: 1.045
          change_pct: -1.118
          ci95_low_pct: -4.266
          ci95_high_pct: 2.338
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        cpu_ns:
          control_median: 250632500.0
          candidate_median: 245231000.0
          control_p95_over_median: 1.075
          candidate_p95_over_median: 1.067
          change_pct: -2.255
          ci95_low_pct: -4.572
          ci95_high_pct: 0.161
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        user_cpu_ns:
          control_median: 73380500.0
          candidate_median: 59890500.0
          control_p95_over_median: 1.233
          candidate_p95_over_median: 1.227
          change_pct: -15.119
          ci95_low_pct: -21.145
          ci95_high_pct: -1.117
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 181539000.0
          candidate_median: 186588000.0
          control_p95_over_median: 1.115
          candidate_p95_over_median: 1.14
          change_pct: 4.106
          ci95_low_pct: -4.006
          ci95_high_pct: 8.433
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: inconclusive
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - peak_rss_bytes is missing a paired percent interval
          - "minor_faults straddles its +10% regression limit"
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
          minor_faults: inconclusive
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 1621
    new_dependencies: []
    new_unsafe_blocks: 4
    new_failure_modes: []
    notes: "four unsafe expressions (getdents64 and statx syscalls, the statx availability probe, mem::zeroed statx) behind cfg(all(target_os = linux, target_env = gnu)), each with a SAFETY argument; most lines are tests"
  verdict:
    decision: accepted
    primary_job: aggregate-summary
    primary_metric: wall_ns
    change_pct: -6.251
    reason: "quiet 20-pair node-modules-dense aggregate-summary --no-controls -6.25% [-14.02%, -1.23%], default summary -9.66%, default-tree -4.29% [-8.81%, -0.90%]; serial-portable placebo +0.10% includes zero; fstat 5,773 -> 4; answers identical"
    commit: "20933081"
    kept: candidate
---
## What was predicted

H169 phase 1, the Linux-native directory reader, specified by a Fable design review and
revised after a Fable soundness review
([plan amendment 7](../specs/active/plan-2026-09-29-linux-overnight-performance-loop.md);
research brief §4). On Linux with glibc, each walker opens a directory by absolute path
with `O_DIRECTORY | O_NOFOLLOW` and reads `getdents64` into a reused, always-initialized
buffer. It uses the names in place, with no per-entry `CString` or `DirEntry`, and no
glibc `opendir` `fstat`. It stats each entry with
`statx(dirfd, name, AT_SYMLINK_NOFOLLOW | AT_NO_AUTOMOUNT | AT_STATX_SYNC_AS_STAT, STATX_BASIC_STATS)`
through `libc::syscall`, since the wheels target glibc 2.17. Any open or `getdents64`
failure declines the directory before a child is published, and the portable path
re-reads it. The boundary has four `unsafe` expressions with written safety arguments.
The `statx` availability latch mirrors std’s probe.
The three public diagnostics fields of the first draft are deferred to 0.3.0 as a public
API change (`fdu-q7hf`).

Pre-registered before any timed sample (`a9e963bf`):
- **Candidate:** `20933081`, the reader merged on the H180 head; the control is the H180
  head.
- **Deciding:** `aggregate-summary --no-controls` on `node-modules-dense` and
  `linux-v6.12`, 20 pairs, predicted −8% to −12%.
- **Co-secondaries:** `default-tree` on both, predicted −6% to −8%.
- **Placebo:** `aggregate-summary --no-controls --threads 1`, which stays on the
  portable serial walk.

## What was measured

Quiet, 20 pairs, no invalid samples.
Control: `70c2725c` probe (the H180 engine).

| Job | Control | H169 | Change |
| --- | ---: | ---: | --- |
| `aggregate-summary --no-controls` (deciding) | 70.3 ms | 66.7 ms | **−6.25% [−14.02%, −1.23%]** |
| `aggregate-summary` | 71.9 ms | 66.3 ms | −9.66% [−13.96%, −5.37%] |
| `default-tree` (co-secondary) | 74.6 ms | 70.8 ms | −4.29% [−8.81%, −0.90%] |
| `default-tree --no-controls` | 74.2 ms | 74.0 ms | −1.67% [−3.64%, +2.06%] |
| `aggregate-summary --no-controls --threads 1` (placebo) | 187.6 ms | 185.5 ms | +0.10% [−6.42%, +3.31%] |

`strace -c` of the default command on `linux-v6.12`: `fstat` 5,773 → 4, `getdents64`
11,540 → 11,538, `statx` and the opened paths unchanged.
Every per-entry `statx` now passes `AT_NO_AUTOMOUNT`, which closes `fdu-puk7` on this
path.

**Answers.** The product command line matched the H180 head byte for byte in 171
comparisons over the three subjects.
The walk-level tests compare the native reader with the portable path.
They cover chunk boundaries, a garbage-prefilled buffer, `DT_UNKNOWN` and `DT_WHT`
records, a directory removed while being read, and a mode-`0o400` directory, which was
run as a non-root user.
`make cross-lint` passes on macOS, Windows, i686 glibc and x86_64 musl, and the MSRV
check passes.

## Decision

Accepted on the deciding job, with the placebo at zero.
The default summary gains 9.7%. The default tree gains 4.3% here, below the −6% to −8%
prediction: the tree route stats every entry, directories included, so the saving is a
smaller share of its work.
About 1,600 lines, most of them tests; four `unsafe` expressions behind
`cfg(all(target_os = "linux", target_env = "gnu"))`.
