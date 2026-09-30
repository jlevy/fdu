---
title: "Linux: every route lists through the native reader, and no stat of a child mounts an autofs trigger"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-196
  title: "Linux: every route lists through the native reader, and no stat of a child mounts an autofs trigger"
  date: "2026-09-29"
  hypotheses:
    - H184
  subject:
    tree_label: linux-v6.12
    tree_root_id: 4ffe9d749fe638cd6c746668f7ae66d88e50225fa6903eabd8c4b9405b1aa245
    tree_engine_digest: e9ca9ca9713695a9fffa9a2937e066f4f584c03c2aa921257dee9db56102b05b
    tree_provenance: "Shallow clone of github.com/torvalds/linux tag v6.12 at adc218676eef25575469234709c2d87185ca223a. Reconstructible: git clone --depth 1 --branch v6.12 https://github.com/torvalds/linux.git. Shape is the published v6.12 source tree plus the clone's .git directory as git left it; no extra workspace install."
    tree_reconstructible: true
    tree_entries: 92474
    tree_directories: 5769
    tree_files: 86643
    tree_symlinks: 62
    tree_apparent_bytes: 1759293209
    tree_allocated_bytes: 1965477888
    tree_max_depth: 14
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.10GHz"
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16876515328
    host_system: Linux 6.18.44-fc-v50
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 20
    warmups: 3
    interleaved: true
    control: "4bc9b738 probe: the #161 layer head"
    candidate: "17874dd6 probe: fdu-d2fn, every route through the native reader"
    control_binary:
      name: control
      sha256: f2a78ef223ffcd666dccfb91bd6aa988a458e665b1f993f6a095720727f1a7ff
      size_bytes: 3862104
      args: []
    candidate_binary:
      name: d2fn
      sha256: b871bccc01da13d54a7a7f15facd633244eb96c6156807bcd4fe09a4942436dd
      size_bytes: 3849576
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-196/run.json.gz
  results:
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 109093857.5
          candidate_median: 110202342.5
          control_p95_over_median: 1.041
          candidate_p95_over_median: 1.058
          change_pct: 0.417
          ci95_low_pct: -2.671
          ci95_high_pct: 4.112
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        component_ns:
          control_median: 106527652.0
          candidate_median: 107656290.5
          control_p95_over_median: 1.042
          candidate_p95_over_median: 1.057
          change_pct: 0.394
          ci95_low_pct: -2.933
          ci95_high_pct: 4.229
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        cpu_ns:
          control_median: 406719000.0
          candidate_median: 406068500.0
          control_p95_over_median: 1.045
          candidate_p95_over_median: 1.075
          change_pct: -0.333
          ci95_low_pct: -2.626
          ci95_high_pct: 4.097
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 60746000.0
          candidate_median: 63342500.0
          control_p95_over_median: 1.225
          candidate_p95_over_median: 1.387
          change_pct: 5.11
          ci95_low_pct: -15.156
          ci95_high_pct: 22.232
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        system_cpu_ns:
          control_median: 346199000.0
          candidate_median: 341620000.0
          control_p95_over_median: 1.091
          candidate_p95_over_median: 1.079
          change_pct: -2.904
          ci95_low_pct: -4.62
          ci95_high_pct: 3.59
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
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: within-limit
        policy_stable: null
        policy_rule: null
    - job: opened-discovery
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 2108284355.0
          candidate_median: 2158716797.0
          control_p95_over_median: 1.031
          candidate_p95_over_median: 1.037
          change_pct: 1.191
          ci95_low_pct: 0.56
          ci95_high_pct: 3.511
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 20
        component_ns:
          control_median: 1602350593.0
          candidate_median: 1616625282.5
          control_p95_over_median: 1.041
          candidate_p95_over_median: 1.035
          change_pct: 0.06
          ci95_low_pct: -0.378
          ci95_high_pct: 1.697
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        cpu_ns:
          control_median: 2213339000.0
          candidate_median: 2266470000.0
          control_p95_over_median: 1.034
          candidate_p95_over_median: 1.036
          change_pct: 1.14
          ci95_low_pct: 0.497
          ci95_high_pct: 3.281
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 20
        user_cpu_ns:
          control_median: 1692084000.0
          candidate_median: 1744648000.0
          control_p95_over_median: 1.061
          candidate_p95_over_median: 1.043
          change_pct: 2.373
          ci95_low_pct: -0.069
          ci95_high_pct: 4.31
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        system_cpu_ns:
          control_median: 515027000.0
          candidate_median: 534339500.0
          control_p95_over_median: 1.149
          candidate_p95_over_median: 1.113
          change_pct: 2.766
          ci95_low_pct: -5.685
          ci95_high_pct: 11.749
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        peak_rss_bytes:
          control_median: 112746496.0
          candidate_median: 112943104.0
          control_p95_over_median: 1.002
          candidate_p95_over_median: 1.002
          change_pct: 0.165
          ci95_low_pct: 0.114
          ci95_high_pct: 0.218
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 20
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
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 554178431.5
          candidate_median: 542376886.5
          control_p95_over_median: 1.033
          candidate_p95_over_median: 1.055
          change_pct: -2.744
          ci95_low_pct: -3.959
          ci95_high_pct: 0.28
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        component_ns:
          control_median: 124119780.5
          candidate_median: 119346798.0
          control_p95_over_median: 1.112
          candidate_p95_over_median: 1.113
          change_pct: -3.561
          ci95_low_pct: -6.978
          ci95_high_pct: 0.761
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        cpu_ns:
          control_median: 856068000.0
          candidate_median: 839949000.0
          control_p95_over_median: 1.067
          candidate_p95_over_median: 1.037
          change_pct: -1.452
          ci95_low_pct: -3.874
          ci95_high_pct: 0.247
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        user_cpu_ns:
          control_median: 510311000.0
          candidate_median: 492980500.0
          control_p95_over_median: 1.051
          candidate_p95_over_median: 1.06
          change_pct: -3.225
          ci95_low_pct: -6.234
          ci95_high_pct: -1.34
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 347510500.0
          candidate_median: 350229500.0
          control_p95_over_median: 1.115
          candidate_p95_over_median: 1.098
          change_pct: -0.635
          ci95_low_pct: -4.079
          ci95_high_pct: 3.646
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        peak_rss_bytes:
          control_median: 42012672.0
          candidate_median: 42487808.0
          control_p95_over_median: 1.003
          candidate_p95_over_median: 1.005
          change_pct: 1.068
          ci95_low_pct: 0.911
          ci95_high_pct: 1.532
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 20
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
    lines_changed: 777
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "the reader's statx wrapper gained a caller by path (stat_path), and one listing iterator now serves the serial walk, revalidation, both reconciliations and opened discovery in place of five read_dir loops; the concurrent walk's own native block is untouched"
  verdict:
    decision: rejected
    primary_job: warm-revalidate
    primary_metric: wall_ns
    change_pct: -2.744
    reason: "not a speed decision: the non-regression screen of a correctness change (fdu-d2fn) that ships regardless, quiet 20 pairs on linux-v6.12: warm-revalidate -2.74% [-3.96%, +0.28%], its reconciliation component -3.56%; opened-discovery wall +1.19% [+0.56%, +3.51%] with its discovery component +0.06% non-inferior; default-tree +0.42% within noise; the serial walk, exp-185's placebo arm, -5.14% [-6.63%, -2.44%] against the control's serial walk; fstat 5,773 -> 4 on every moved route; every tree-entry statx carries AT_NO_AUTOMOUNT on every route"
    commit: 17874dd6
    kept: candidate
---
## What was predicted

`fdu-d2fn`, found by the review of #161 (R161-2): H169 phase 1 had put `AT_NO_AUTOMOUNT`
on the parallel walk’s stats alone, so on a tree holding an unmounted autofs trigger
directory the same request answered differently by route and by worker count.
The fix makes every route that lists a directory (the serial and concurrent walks,
revalidation, reconciliation, opened discovery) list through the native reader, stats
the directories it declines and the paths a route verifies by itself (`observe_path`: a
reconciliation’s subtree and its prefixes, the watch’s verification, control lookups) by
path with the same flags, and resolves the walk root alone, through an opened descriptor
(`root_device`), under glibc and musl alike.
musl needed no code: std stats with `fstatat` there, which the kernel treats as passing
the flag.

A correctness change, measured so that it is known not to have regressed the routes it
moved. Pre-registered before any timed sample (`e5724ebc`):

- **Candidate:** `17874dd6`, the fix on the #161 layer head; the control is that head,
  `4bc9b738`.
- **Non-inferiority on `linux-v6.12`, 20 pairs:** `warm-revalidate` and
  `opened-discovery`, predicted −2% to −8% (the reader’s saving on routes that took the
  portable listing); `default-tree`, predicted 0 (one extra `open` of the root per
  scan).
- **The serial walk**, exp-185’s placebo arm
  `aggregate-summary --no-controls --threads 1`, predicted −5% to −10%: it was the
  portable walk and is now the reader.
- **Secondary:** `strace` of every probe route shows every tree-entry `statx` carrying
  `AT_NO_AUTOMOUNT`.
- Nothing is accepted on speed.

## What was measured

Quiet, 20 pairs, no invalid samples, against a baseline fingerprint registered the same
evening: the subject’s `.git/index` had been touched at 19:00 UTC, after the morning’s
fingerprint, and the first attempt at this cell was invalidated whole on that drift, for
control and candidate alike; entry counts and bytes were unchanged.
Control: `4bc9b738` probe (the #161 layer head).

| Job | Control | `fdu-d2fn` | Change | Non-inferior (3%) |
| --- | ---: | ---: | --- | --- |
| `warm-revalidate` wall | 554.2 ms | 542.4 ms | −2.74% [−3.96%, +0.28%] | yes |
| `warm-revalidate` component (the reconciliation) | 124.1 ms | 119.3 ms | −3.56% [−6.98%, +0.76%] | yes |
| `opened-discovery` wall | 2108.3 ms | 2158.7 ms | +1.19% [+0.56%, +3.51%] | inconclusive |
| `opened-discovery` component (the discovery) | 1602.4 ms | 1616.6 ms | +0.06% [−0.38%, +1.70%] | yes |
| `default-tree` wall | 109.1 ms | 110.2 ms | +0.42% [−2.67%, +4.11%] | inconclusive |
| `aggregate-summary --no-controls --threads 1`, the serial walk (second cell) | 344.9 ms | 328.4 ms | **−5.14% [−6.63%, −2.44%]** | yes; clears the accept rule |
| `aggregate-summary`, the concurrent walk, unchanged (second cell) | 108.9 ms | 107.0 ms | −0.04% [−4.84%, +1.40%] | yes |

The reconciliation itself gained 3.6% and its user CPU fell 3.23% [−6.23%, −1.34%]: the
reader’s saving on a route that took the portable listing, as predicted.
Discovery’s own component is flat; the +1.19% on its wall lies outside the discovery
loop, in the journal drain and the independent validation that the job times after it,
and no metric of that job passes the acceptance test in either direction.
The default tree, whose walk this change does not touch, moved within noise.
Peak RSS rose 0.17% and 1.07% on the two moved routes (the reader’s 64 KiB record buffer
per worker), within its 5% limit.

The serial walk’s rows are the second cell, `run-serial.json.gz` beside this record: the
same four arms exp-185 ran, quiet, 20 pairs, no invalid samples.
Its user CPU fell 26.9% [−40.2%, −7.1%]: the walk that was exp-185’s portable placebo is
now the reader.

`strace -c` of the probe on `linux-v6.12`, control against candidate: `revalidate`
`fstat` 5,773 → 4, `getdents64` 11,540 → 11,538, `statx` 92,902 → 92,907, `openat` 6,183
→ 6,187; `opened-discovery` `fstat` 5,773 → 4, `getdents64` 11,540 → 11,538, `statx`
92,833 → 92,834; the serial summary `fstat` 5,773 → 4, `statx` 86,648 → 86,649. The
`fstat` is glibc’s per-`opendir` call, gone with the portable listing; the extra `statx`
and `openat` are the root’s descriptor (`root_device`). On a small tree holding every
kind of entry, `strace -f -e trace=statx,newfstatat,fstatat64` of every probe route
(`summary` and `scan-index` at one and four workers,
`summary --no-controls --threads 1`, `scan-producer` at one and four workers,
`default-tree`, `opened-discovery`, `revalidate`) shows every `statx` of a tree entry
carrying `AT_NO_AUTOMOUNT`, 11 to 34 per route, and the calls without it being the walk
root itself, descriptor stats with `AT_EMPTY_PATH`, std’s `available_parallelism` cgroup
probes, and the snapshot file.
The control’s `opened-discovery` and `revalidate` made 20 and 41 child stats without the
flag. Under the crate’s walk-hook test, which forces the portable path over four random
trees, 93,419 child stats carried the flag and none of a tree entry lacked it.
glibc std issues no `fstatat`.

**Answers.** The routes cell’s tallies oracle passed on every sample, and the 884 core
tests, the golden corpus and the Python parity gate pass.
On glibc the `linux_dents` tests compare the reader and the crate’s portable listing
with std’s own answer over random trees of every name shape and kind, the path stat with
`symlink_metadata` over every kind (file, directory, symlink, dangling symlink, FIFO,
socket, hard link, a missing path, a non-UTF-8 name), and the serial walk against a walk
hook that forces the portable path.

## Decision

Recorded as a non-regression screen; the change ships on correctness (`fdu-d2fn`), not
on speed, so nothing here is an accept.
The reconciliation route is faster and the others are within noise or under the 3%
margin; the one interval above zero, `opened-discovery` wall, has its discovery
component non-inferior and is worth a profile of the drain and validation that follow
the walk before it is called a cost of this change.
About 700 lines, most of them tests; no new `unsafe` expression (the reader’s `statx`
wrapper gained a caller by path), no new dependency.
