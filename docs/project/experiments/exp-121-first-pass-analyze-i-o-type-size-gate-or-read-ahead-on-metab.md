---
title: First-pass analyze I/O type/size gate or read-ahead on metabrowser
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-121
  title: First-pass analyze I/O type/size gate or read-ahead on metabrowser
  date: "2026-09-19"
  hypotheses:
    - H124
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
    control: HEAD at 45727e1d same probe
    candidate: same probe self-comparison
    control_binary:
      name: control
      sha256: 0be9e2987fa227f1b55ae1dd5e2f7b0a210490a5ad7030ecc5967395583a0f9a
      size_bytes: 2536992
      args: []
    candidate_binary:
      name: candidate
      sha256: 0be9e2987fa227f1b55ae1dd5e2f7b0a210490a5ad7030ecc5967395583a0f9a
      size_bytes: 2536992
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: /tmp/fdu-realtree/results/run-exp-121-h124-first-pass-analyze-io.json
  results:
    - job: content-basic
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 9014396124.5
          candidate_median: 9036454625.0
          control_p95_over_median: 1.692
          candidate_p95_over_median: 1.098
          change_pct: -4.221
          ci95_low_pct: -20.786
          ci95_high_pct: 5.097
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 8312081979.0
          candidate_median: 8382941937.0
          control_p95_over_median: 1.706
          candidate_p95_over_median: 1.085
          change_pct: -4.216
          ci95_low_pct: -20.467
          ci95_high_pct: 5.71
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        cpu_ns:
          control_median: 22723768000.0
          candidate_median: 23450947000.0
          control_p95_over_median: 1.15
          candidate_p95_over_median: 1.107
          change_pct: 3.418
          ci95_low_pct: -0.354
          ci95_high_pct: 8.835
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 5847982500.0
          candidate_median: 5749475000.0
          control_p95_over_median: 1.032
          candidate_p95_over_median: 1.024
          change_pct: -0.913
          ci95_low_pct: -2.319
          ci95_high_pct: -0.049
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 16956420500.0
          candidate_median: 17682038000.0
          control_p95_over_median: 1.185
          candidate_p95_over_median: 1.136
          change_pct: 4.4
          ci95_low_pct: -0.157
          ci95_high_pct: 12.59
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 266526720.0
          candidate_median: 267255808.0
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.003
          change_pct: 0.176
          ci95_low_pct: 0.0
          ci95_high_pct: 0.836
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
          - major_faults does not establish non-regression
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
          major_faults: inconclusive
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
    notes: profile only; no engine change
  verdict:
    decision: rejected
    primary_job: content-basic
    primary_metric: wall_ns
    change_pct: -4.221
    reason: "every admitted open is required for lines; skippable share under 1% wall; read calls already one data chunk per file; no engine change"
    commit: 45727e1d
---
## What was predicted

H118 rejected first-pass insert-then-rebuild (file I/O hid the ancestor walk).
H119 screened walk-overlap (`fdu::scan` 0.13%; `read` 59%, `__open` 17%). H124: a
type/size gate (do not open files that cannot contribute to `content-basic` lines) or
read-ahead on admitted files cuts that wall at least 3%. Not H118. Not H119
walk-overlap. No new `unsafe`.

Named before measuring:

- Accept: `content-basic` wall down at least 3% with the interval below zero on
  deciding-scale metabrowser; content digest identical; worker parallelism retained.
- Control = #92 HEAD `45727e1d` (H115 + H120 in; no engine speed patch on this branch).
- What refutes: interval includes zero, or every admitted open is required for the
  requested metrics (and remaining files are already one-chunk sequential reads, so a
  safe read-ahead cannot clear 3%).
- Inventory first. Implement a smallest engine change only if the mix names skippable
  opens or bytes that can reach the 3% bar.

Subject: frozen APFS clone of `metabrowser-clone` (live checkout has writers).
Experiment id exp-121. exp-113 remains reserved.

## What was measured

Subject: frozen `metabrowser-clone` (146,047 entries / 133,708 files / 11,517
directories, max depth 19). Digest `dc0df263…`. The clone did not mutate.

Path-only classify already skips `ContentFamily::Binary` without opening.
A walk of the clone plus one `FDU_COUNTERS=1` `content-basic` (attribution only):

| Quantity | Count / bytes |
| --- | ---: |
| Regular files | 133,708 |
| Path-binary (skipped, not opened) | 8,022 / 283 MiB |
| Opens | 125,686 |
| Empty non-binary files | 731 (0.58% of opens) |
| Read calls | 249,533 (~1.99 per open) |
| Bytes read | 951,822,681 |
| Report binary / analyzed / invalid UTF-8 | 14,159 / 118,882 / 667 |
| Content digest | `3be19a3e…` |

`.bin` is 2,991 files / 22 MiB and almost all text (4 NUL). Adding it as binary would
change line counts. `.node` is 3 Mach-O files / 20 MiB; prefix NUL already early-exits
after one chunk. No-extension files include 10 magic binaries totaling 458 MiB; those
also early-exit (bytes read 908 MiB vs 1,443 MiB admitted size).

Discovered binary after open: 14,159 − 8,022 = 6,137 (4.9% of opens).
Even skipping those plus every empty file is 5.5% of opens.
Opens are 17% of the H119 sample, so that share is under 1% of wall.

Read calls are one data chunk plus EOF for nearly every file.
Extra multi-chunk data reads are a rounding error on 250k calls.
A larger `READ_CHUNK_BYTES` cannot clear 3%. `F_RDADVISE` / `F_RDAHEAD` is a new
`unsafe` block (person-gated, same class as H119 `openat`).

No engine patch. A 256 KiB `READ_CHUNK_BYTES` edit was reverted in the editor before any
candidate binary was built.

Attachment: 12-pair same-binary `content-basic`, `FDU_COUNTERS` unset.
3 warmups, 12 timed pairs, interleaved, same probe both variants (sha256 `0be9e298…`).

Quiet was attempted.
The start gate refused at 29.6% CPU busy.
The pair ran as **uncontrolled**. Initial busy 68.55%; final 63.33%. Thermal `normal`.
The 25% bar was not lowered.
No RAM disk.

0 invalid samples.
Every timed sample applied 133,708 / analyzed 118,882 / binary 14,159.
Content digest `3be19a3e…`. Engine digest unchanged (`dc0df263…`).

| Arm | Wall median | Component | Peak RSS |
| --- | ---: | ---: | ---: |
| control | 9,014.4 ms | 8,312.1 ms | 254.2 MiB |
| candidate | 9,036.5 ms | 8,382.9 ms | 254.9 MiB |

Same-binary wall −4.22% [−20.79%, +5.10%]. Attachment only.
Interval includes zero.

## What the accept rule said

Every admitted open is required for lines, or the skippable share cannot reach 3% wall.
Safe read-ahead has no 3% target: reads are already one data chunk per file.
H124 is rejected.

## Judgment

Do not add a type/size gate or a larger read chunk from this cell.
Do not add `unsafe` read-ahead.
Do not retry H118 or H119 walk-overlap.

Do not raise the README 200K files/s or 4M cached lines/s from this cell.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
