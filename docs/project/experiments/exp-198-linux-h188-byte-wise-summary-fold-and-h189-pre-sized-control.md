---
title: "Linux: H188 byte-wise summary fold and H189 pre-sized control reads, the default summary 6% faster on linux-v6.12"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-198
  title: "Linux: H188 byte-wise summary fold and H189 pre-sized control reads, the default summary 6% faster on linux-v6.12"
  date: "2026-09-30"
  hypotheses:
    - H188
    - H189
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
    control: c0da65ae probe (the H185 head)
    candidate: "7a3a7058 probe (the H188 part-2 head, with H189)"
    control_binary:
      name: control
      sha256: ad39ff3c78f614c4947b6095e3ead2eac666d0a14f3e4d59fda4af5c06cbfc57
      size_bytes: 3863336
      args: []
    candidate_binary:
      name: candidate
      sha256: 6d966cea2195da057e2eb909583ad29912f3d09505ea9b084887ee4aa7ec99c1
      size_bytes: 3870248
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-198/run.json.gz
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 90336785.0
          candidate_median: 85787192.0
          control_p95_over_median: 1.098
          candidate_p95_over_median: 1.09
          change_pct: -6.152
          ci95_low_pct: -7.943
          ci95_high_pct: -1.801
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 87393763.5
          candidate_median: 83012291.5
          control_p95_over_median: 1.107
          candidate_p95_over_median: 1.086
          change_pct: -6.112
          ci95_low_pct: -7.862
          ci95_high_pct: -1.282
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 324888000.0
          candidate_median: 318128000.0
          control_p95_over_median: 1.084
          candidate_p95_over_median: 1.107
          change_pct: -3.381
          ci95_low_pct: -5.304
          ci95_high_pct: -0.07
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 88440500.0
          candidate_median: 60584000.0
          control_p95_over_median: 1.298
          candidate_p95_over_median: 1.446
          change_pct: -32.665
          ci95_low_pct: -38.788
          ci95_high_pct: -17.148
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 239283000.0
          candidate_median: 255933500.0
          control_p95_over_median: 1.104
          candidate_p95_over_median: 1.126
          change_pct: 7.359
          ci95_low_pct: -0.422
          ci95_high_pct: 13.12
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - peak_rss_bytes is missing a paired percent interval
          - "voluntary_context_switches exceeds its +50% regression limit"
          - "involuntary_context_switches exceeds its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: within-limit
          involuntary_context_switches: rejected
          major_faults: within-limit
          minor_faults: within-limit
          peak_rss_bytes: inconclusive
          system_cpu_ns: within-limit
          voluntary_context_switches: rejected
        policy_stable: null
        policy_rule: null
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 86329361.5
          candidate_median: 87346905.0
          control_p95_over_median: 1.111
          candidate_p95_over_median: 1.061
          change_pct: 0.195
          ci95_low_pct: -1.428
          ci95_high_pct: 3.265
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        component_ns:
          control_median: 83447237.5
          candidate_median: 84567310.5
          control_p95_over_median: 1.106
          candidate_p95_over_median: 1.065
          change_pct: 0.421
          ci95_low_pct: -1.868
          ci95_high_pct: 3.368
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        cpu_ns:
          control_median: 318681000.0
          candidate_median: 316050000.0
          control_p95_over_median: 1.043
          candidate_p95_over_median: 1.051
          change_pct: -0.194
          ci95_low_pct: -1.907
          ci95_high_pct: 1.878
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 20
        user_cpu_ns:
          control_median: 69955000.0
          candidate_median: 68195000.0
          control_p95_over_median: 1.341
          candidate_p95_over_median: 1.226
          change_pct: -8.369
          ci95_low_pct: -22.791
          ci95_high_pct: 19.506
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        system_cpu_ns:
          control_median: 248257000.0
          candidate_median: 252598500.0
          control_p95_over_median: 1.078
          candidate_p95_over_median: 1.074
          change_pct: 1.597
          ci95_low_pct: -4.61
          ci95_high_pct: 6.861
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
  reference_tools: []
  complexity:
    lines_changed: 261
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: accepted
    primary_job: aggregate-summary
    primary_metric: wall_ns
    change_pct: -6.152
    reason: "quiet 20-pair linux-v6.12 aggregate-summary -6.15% [-7.94%, -1.80%], within the -5% to -9% predicted; the --no-controls arms there and both jobs on node-modules-dense include zero; summary consumer instructions 361.7M -> 202.7M; read 1,261 -> 729; answers identical"
    commit: 7a3a7058
    kept: candidate
---
## What was predicted

H188 with H189, from
[the uniformly-faster brief](../research/research-2026-09-29-uniformly-faster-than-pdu.md).

The transient summary’s consumer executed 373M instructions on `linux-v6.12`, more than
the tree route’s 241M, on the route pdu’s `--max-depth 1` is compared with.
180M of it was `std::path`: the fold asked for each op’s `parent()` and `file_name()`,
which parse components, compared the parent with the cached one by `Path` equality,
which walks components (`compare_components`, 94M), and hashed ignored heads by
component. The quiet cells of exp-194 put the controls-on summary at 116.0 ms against
104.4 ms blind, 77 ms of user CPU against 32.

H188 splits each path at its last separator, compares the parent’s bytes, and keys
ignored heads by `OsString`, whose hash and equality are bytes.
Its first build removed 70M of the 373M; the profile of that build put the other 90M in
`ControlTable::chain_for`, inlined into the fold, probing its `BTreeMap<PathBuf>` once
per ancestor of every new parent with component-wise comparisons (234k of them), so the
table now also keeps its sources by the bytes of their directory, and `chain_for` and
`chain_below` probe that index.
H189 rides in the same cell: each `.gitignore` was read in eight `read` calls from 32
bytes doubling, because `read_to_end` behind `take` could not see the length the
metadata already carried; the buffer is reserved from that length within the read limit
and a 64 KiB bound.

Pre-registered in the registry (`bf56ecfa`) before code:

- **Deciding:** `aggregate-summary` wall −3% with the interval below zero on
  `linux-v6.12`, 20 pairs; predicted −5% to −9%.
- **Secondary:** `default-tree` on `linux-v6.12` (H189, predicted −0.5% to −1%);
  consumer instructions −120M or more; `strace -c` `read` 1,261 → about 720.
- **Placebos:** both jobs on `node-modules-dense` (no rules), and the `--no-controls`
  arms on `linux-v6.12`, include zero.
- **Answers:** the summary’s ignored share identical to the index’s under the
  compact-equals-indexed differential; a test holds the byte-wise split and ancestors to
  `Path`’s for every shape a normalized relative path takes.

## What was measured

**System calls** (`strace -c`, load-independent): `read` 1,261 → 729 on `linux-v6.12`
for both routes, two per control file where the mean was 3.5 (most of the tree’s 358
files are under 32 bytes and already took two); the total falls 641 on the tree route.

**Instructions** (callgrind per thread, the stripped release binaries): the summary
route’s consumer on `linux-v6.12` fell from 361.7M (H185 head) to 291.6M with the fold’s
own changes, and to 202.7M with the table’s byte-keyed index, −159M in all against the
−120M predicted; its walkers and the tree route’s consumer are unchanged (132.6M and
239.3M).

**Fingerprints.** The cells checked the subjects against fingerprints registered for
this track (`tree-linux-v6.12-pdu.json`, `tree-node-modules-dense-pdu.json`,
`tree-linux-balanced-1m-pdu.json`), not the shared ones: the shared `linux-v6.12`
fingerprint, taken at 07:48 UTC, no longer matched the subject after its `.git/index`
was touched at 19:00 UTC. Entry counts and bytes are identical to the shared
fingerprint’s, the subject was not modified by this track, and the new fingerprints were
registered before any cell ran.

**Wall.** Quiet, 20 pairs, no invalid samples in either cell.
Control: `c0da65ae` probe (the H185 head).
Candidate: `7a3a7058` probe (the H188 part-2 head, with H189).

| Subject, job | Control | H188 | Change |
| --- | ---: | ---: | --- |
| `linux-v6.12`, `aggregate-summary` (deciding) | 90.3 ms | 85.8 ms | **−6.15% [−7.94%, −1.80%]** |
| `linux-v6.12`, `aggregate-summary --no-controls` (placebo) | 78.9 ms | 76.9 ms | +0.17% [−3.27%, +2.80%] |
| `linux-v6.12`, `default-tree` (H189, secondary) | 86.3 ms | 87.3 ms | +0.20% [−1.43%, +3.27%] |
| `linux-v6.12`, `default-tree --no-controls` (placebo) | 83.2 ms | 84.0 ms | +1.65% [−0.57%, +4.52%] |
| `node-modules-dense`, `aggregate-summary` (placebo) | 77.3 ms | 74.8 ms | −2.07% [−7.17%, +1.19%] |
| `node-modules-dense`, `aggregate-summary --no-controls` (placebo) | 74.5 ms | 75.9 ms | +2.25% [−2.03%, +5.72%] |
| `node-modules-dense`, `default-tree` (placebo) | 82.7 ms | 82.2 ms | −1.70% [−4.74%, +2.97%] |
| `node-modules-dense`, `default-tree --no-controls` (placebo) | 81.7 ms | 83.0 ms | +2.00% [−3.20%, +5.49%] |

With H188 the controls-on summary is 1.116 times its own `--no-controls` arm (85.8
against 76.9 ms), from 1.144 at the H185 head (90.3 against 78.9); on the exp-194
figures the ratio was 1.111. H189’s `read` calls are not visible on wall: 532 fewer
`read` calls of a few hundred bytes each fall below the cell’s resolution, as the
prediction (−0.5% to −1%) said they might.

**Supplementary runs.** Beside the primary artifact, gzipped to keep the diff
reviewable: `run-node-modules-dense.json.gz`, the `node-modules-dense` placebo cell.

## Decision

Accepted. The default summary on the source tree with 358 `.gitignore` files is 6.2%
faster, within the predicted −5% to −9%, and all five placebos include zero: the
`--no-controls` arms on `linux-v6.12`, which neither change touches, and both jobs on
`node-modules-dense`, which has no rules.
H189 rides along as a secondary that wall cannot resolve; its `read` cut is
load-independent and costs nothing.
About 260 lines over the two commits, `a0666bf0` and `7a3a7058`, a quarter of them
tests.
