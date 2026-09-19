---
title: First-pass walk versus opened-discovery I/O on metabrowser
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-126
  title: First-pass walk versus opened-discovery I/O on metabrowser
  date: "2026-09-19"
  hypotheses:
    - H127
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
    candidate: same probe (leftover profile)
    control_binary:
      name: control
      sha256: c86ad8cbeeec5a1cecc2b5b6f128a4913d3fec6e643ee8b569fb693001a3090f
      size_bytes: 2536992
      args: []
    candidate_binary:
      name: candidate
      sha256: c86ad8cbeeec5a1cecc2b5b6f128a4913d3fec6e643ee8b569fb693001a3090f
      size_bytes: 2536992
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-126-h127-first-pass-vs-opened-discovery.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 581151313.0
          candidate_median: 570805687.0
          control_p95_over_median: 1.081
          candidate_p95_over_median: 1.045
          change_pct: -1.879
          ci95_low_pct: -3.357
          ci95_high_pct: -0.805
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 314767104.0
          candidate_median: 305424291.5
          control_p95_over_median: 1.116
          candidate_p95_over_median: 1.082
          change_pct: -2.555
          ci95_low_pct: -5.808
          ci95_high_pct: -2.404
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        cpu_ns:
          control_median: 2071393000.0
          candidate_median: 2024965500.0
          control_p95_over_median: 1.019
          candidate_p95_over_median: 1.031
          change_pct: -1.174
          ci95_low_pct: -1.941
          ci95_high_pct: 2.07
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 398641000.0
          candidate_median: 395430500.0
          control_p95_over_median: 1.008
          candidate_p95_over_median: 1.023
          change_pct: -0.803
          ci95_low_pct: -2.22
          ci95_high_pct: 1.412
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 1671728500.0
          candidate_median: 1630261000.0
          control_p95_over_median: 1.023
          candidate_p95_over_median: 1.04
          change_pct: -1.159
          ci95_low_pct: -2.059
          ci95_high_pct: 2.855
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 40763392.0
          candidate_median: 40640512.0
          control_p95_over_median: 1.01
          candidate_p95_over_median: 1.007
          change_pct: -0.521
          ci95_low_pct: -0.882
          ci95_high_pct: 0.565
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
    - job: opened-discovery
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 3772522771.0
          candidate_median: 4044496499.5
          control_p95_over_median: 1.628
          candidate_p95_over_median: 1.393
          change_pct: -4.966
          ci95_low_pct: -10.239
          ci95_high_pct: 6.386
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 2761184500.0
          candidate_median: 3021877313.5
          control_p95_over_median: 1.775
          candidate_p95_over_median: 1.523
          change_pct: -4.629
          ci95_low_pct: -15.684
          ci95_high_pct: 9.974
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 3860198000.0
          candidate_median: 3909102500.0
          control_p95_over_median: 1.141
          candidate_p95_over_median: 1.096
          change_pct: -1.213
          ci95_low_pct: -3.368
          ci95_high_pct: 1.449
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 2324042000.0
          candidate_median: 2317289500.0
          control_p95_over_median: 1.07
          candidate_p95_over_median: 1.046
          change_pct: -0.425
          ci95_low_pct: -1.868
          ci95_high_pct: 0.576
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 1554138000.0
          candidate_median: 1588762000.0
          control_p95_over_median: 1.233
          candidate_p95_over_median: 1.172
          change_pct: -0.89
          ci95_low_pct: -6.716
          ci95_high_pct: 3.054
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 222642176.0
          candidate_median: 223248384.0
          control_p95_over_median: 1.003
          candidate_p95_over_median: 1.004
          change_pct: 0.448
          ci95_low_pct: -0.161
          ci95_high_pct: 3.911
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
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
    notes: leftover profile only; no engine change
  verdict:
    decision: accepted
    primary_job: opened-discovery
    primary_metric: wall_ns
    change_pct: -4.966
    reason: opened-discovery 8.8x first-pass component; read_dir+fstatat vs getattrlistbulk; journal clones remain; no smallest cut
    commit: 81f9e447
---
H127 pairs first-pass metadata walk I/O (`cold-scan-index`) with opened-root discovery
I/O (`opened-discovery`) on the same frozen deciding-scale `metabrowser-clone`.

Not H113. Not a completeness skip.
Not content on opened-root: an opened root runs no analyzers (`OpenedIndex::basis`). Not
H123 (retained report cheapness).
Not a retry of H104–H106 scanner-shape guesses.

Named before measuring:

- Metric: same-binary 12-pair of both jobs; the determination is the cross-job component
  ratio plus counters and a 20 s sample of each job.
- Subject: frozen APFS clone of `metabrowser-clone` (same tree as H125/H126).
- Accept as determination if the leftover is named: whether opened-discovery residual is
  still journal/control (H110) versus the first-pass walk leftover (H122: directory
  `__open` plus `getattrlistbulk`), and whether any userspace stage is at least 3% and
  not already rejected.
- Same H125 probe both arms (`c86ad8cb…`). `FDU_COUNTERS` unset on the pair.
- Quiet first. If the start gate fails, label uncontrolled.
  Do not lower the 25% bar.
- Do not compile an engine cut unless a smallest named mechanism clears 3%. Progressive
  discovery must keep correct roll-ups at each commit; H115’s end-of-restore rebuild
  does not transfer.

H113 quiet was already skipped this campaign.
This cell’s official quiet check refused at CPU busy **53.86% > 25.0%** (load/core
1.382). File-count not compiled.

## What was measured

Same release probe both variants (sha256 `c86ad8cb…`, 2,536,992 bytes).
Jobs: harness `cold-scan-index` and `opened-discovery`. 3 warmups, 12 timed pairs,
interleaved. `FDU_COUNTERS` unset.

The pair ran as **uncontrolled**. Official quiet check 53.86% busy.
Harness initial busy 35.11%; final 55.87%. Thermal `normal`. The 25% bar was not
lowered. No RAM disk.

Digest `dc0df263…`. 0 invalid samples.
Tree fingerprint unchanged.

| Job | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| `cold-scan-index` | 581.2 ms | 314.8 ms | 38.9 MiB |
| `opened-discovery` | 3,772.5 ms | 2,761.2 ms | 212.3 MiB |

Opened-discovery component is about **8.8×** first-pass (2,761 / 315). Wall about
**6.5×** (3,773 / 581). Peak RSS about **5.5×** (212 / 39). Same-binary pairs on each
job are noise (scan wall −1.88% [−3.36%, −0.81%]; opened wall −4.97% [−10.24%, +6.39%]).

Counters-on attribution (same binary, not the verdict; first of three hits):

| Job | Component | Opens | Enum calls | Enum / open | Walk | Finish | Journal retained / cloned | Roll-up merges | Control reads |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `scan-index` | 319.6 ms | 11,517 | 22,480 | 1.952 | 316.9 ms | 2.4 ms | 0 / 0 | 157,562 | 52 |
| `opened-discovery` | 2,667.0 ms | 11,517 | 0 | — | — | — | 11,524 / 11,524 | 1,122,499 | 52 |

Same directory opens and 146,046 entries enumerated.
First-pass uses `macos_bulk` (1.952 `getattrlistbulk` per directory including EOF;
file-heavy vs exp-122’s 1.403 on directory-heavy frameworks).
Opened discovery lists with `std::fs::read_dir` in `discover_directory` (`opened.rs`),
so `dir_enumeration_calls` stays 0. It then `fstatat`s. It commits once per directory
(11,524 retained commits, 11,524 clones, 26 all-dirty, 241,310 dirty paths).
Roll-up merges stay at the pre-H115 1.12M shape because each progressive commit must
publish correct totals.
Control reads are 52 on both jobs.

20 s `/usr/bin/sample` on the release probe (symbols stripped; kernel names remain):

`scan-index` (`--repeat 80`, 81,964 thread-root samples):

| Symbol | Share |
| --- | ---: |
| `__open` | 47.0% |
| `getattrlistbulk` | 32.6% |
| `semaphore_wait_trap` | 6.5% |

No `fstatat`. No `getdirentries64`. Same leftover as H122, higher bulk multiplicity.

`opened-discovery` (4,325 samples; main thread + `fdu-discovery`):

| Symbol | Share of process |
| --- | ---: |
| `__psynch_cvwait` (main waiting on discovery) | 37.1% |
| `fstatat` | 8.3% |
| `__open_nocancel` (`opendir`) | 6.8% |
| `__getdirentries64` | 3.3% |
| `getattrlistbulk` | 0.0% |

Listing I/O (`fstatat` + `opendir` + `getdirentries64`) is 18.4% of process samples and
about 43% of the discovery thread.
No userspace symbol is named at ≥3% (release probe is stripped).
Allocator pieces are each <2%.

## What the determination said

Opened-root discovery is not the first-pass walk plus a cheap journal.
On this file-heavy tree it is about 9× the first-pass component, 5× the RSS, and a
different listing backend (`read_dir` + `fstatat` versus `getattrlistbulk`).

The leftover that H110 already owns is still there: one journal clone per directory
commit. The first-pass leftover (H122) is still `__open` plus `getattrlistbulk`, now at
1.952 calls/dir on this tree.

A `macos_bulk` port of `discover_directory` is a serving-path backend swap, not a
smallest patch. Applying H115’s end-of-restore rebuild to progressive commits would
change mid-discovery roll-ups.
Do not compile either.
Do not retry H104–H106.

Do not raise the README 200K files/s or 4M cached lines/s.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
