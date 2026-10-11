---
title: "Linux: several roots pay about 0.5 ms a root, 272 small roots five times their parent"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-221
  title: "Linux: several roots pay about 0.5 ms a root, 272 small roots five times their parent"
  date: "2026-10-10"
  hypotheses:
    - H196
  subject:
    tree_label: cargo-registry-src-linux
    tree_root_id: d5230e4f60929228844d1aa30e0ecce5e6d109e2fe3c675dd71e4c43c2335498
    tree_engine_digest: 236b7d536c429b94f11f2291a010c4d5e78611dba297dacc8c4a80d01a60ae01
    tree_provenance: "A copy (cp -a) of this host's cargo registry source store (~/.cargo/registry/src/index.crates.io-*), taken after building fdu at 148ef78e, 23d11b19 and 0d6011da and installing cargo-deny 0.20.2, pdu 0.24.0 and diskus 0.9.0 with --locked: 272 crate directories and nothing else at its top, 17,089 entries, 2,912 directories, 14,177 files. Approximately reconstructible only: the crate set follows those lockfiles and install order."
    tree_reconstructible: false
    tree_entries: 17089
    tree_directories: 2912
    tree_files: 14177
    tree_symlinks: 0
    tree_apparent_bytes: 254464369
    tree_allocated_bytes: 291057664
    tree_max_depth: 9
    tree_mutated_during_run: false
    host_cpu: "Intel(R) Xeon(R) Processor @ 2.10GHz"
    host_arch: x86_64
    host_cores: 4
    host_performance_cores: 0
    host_efficiency_cores: 0
    host_memory_bytes: 16874930176
    host_system: Linux 6.18.44-fc-v114
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 20
    warmups: 3
    interleaved: true
    control: "0d6011da probe, the store as one root"
    candidate: "0d6011da probe, each of the store's 272 crate directories as a root (--child-roots, same binary)"
    control_binary:
      name: parent
      sha256: d93d5a4361b6717accb93f52eed531936ca1d1f513555484d2f4c4658086c7a0
      size_bytes: 4266000
      args: []
    candidate_binary:
      name: children
      sha256: d93d5a4361b6717accb93f52eed531936ca1d1f513555484d2f4c4658086c7a0
      size_bytes: 4266000
      args:
        - "--child-roots"
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-221/run.json.gz
  results:
    - job: roots-default-tree
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 31839066.0
          candidate_median: 162296098.5
          control_p95_over_median: 1.099
          candidate_p95_over_median: 1.234
          change_pct: 425.546
          ci95_low_pct: 383.067
          ci95_high_pct: 478.991
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 20
        component_ns:
          control_median: 28898883.0
          candidate_median: 158845855.0
          control_p95_over_median: 1.115
          candidate_p95_over_median: 1.234
          change_pct: 466.356
          ci95_low_pct: 426.817
          ci95_high_pct: 530.975
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 20
        cpu_ns:
          control_median: 96424000.0
          candidate_median: 295858000.0
          control_p95_over_median: 1.056
          candidate_p95_over_median: 1.226
          change_pct: 208.397
          ci95_low_pct: 193.653
          ci95_high_pct: 232.621
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 20
        user_cpu_ns:
          control_median: 19406000.0
          candidate_median: 79458500.0
          control_p95_over_median: 1.699
          candidate_p95_over_median: 1.544
          change_pct: 343.6
          ci95_low_pct: 198.776
          ci95_high_pct: 458.345
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 20
        system_cpu_ns:
          control_median: 74083000.0
          candidate_median: 219519500.0
          control_p95_over_median: 1.112
          candidate_p95_over_median: 1.152
          change_pct: 199.922
          ci95_low_pct: 169.318
          ci95_high_pct: 211.971
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "cpu_ns exceeds its +50% regression limit"
          - "system_cpu_ns exceeds its +75% regression limit"
          - peak_rss_bytes is missing a paired percent interval
          - "minor_faults exceeds its +10% regression limit"
          - "voluntary_context_switches exceeds its +50% regression limit"
          - "involuntary_context_switches straddles its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: rejected
          involuntary_context_switches: inconclusive
          major_faults: within-limit
          minor_faults: rejected
          peak_rss_bytes: inconclusive
          system_cpu_ns: rejected
          voluntary_context_switches: rejected
        policy_stable: null
        policy_rule: null
    - job: roots-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 27649568.5
          candidate_median: 139487839.5
          control_p95_over_median: 1.436
          candidate_p95_over_median: 1.35
          change_pct: 417.884
          ci95_low_pct: 377.432
          ci95_high_pct: 464.544
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 20
        component_ns:
          control_median: 25056293.0
          candidate_median: 136252539.0
          control_p95_over_median: 1.402
          candidate_p95_over_median: 1.351
          change_pct: 459.312
          ci95_low_pct: 416.311
          ci95_high_pct: 514.303
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 20
        cpu_ns:
          control_median: 90411500.0
          candidate_median: 258934500.0
          control_p95_over_median: 1.134
          candidate_p95_over_median: 1.217
          change_pct: 199.826
          ci95_low_pct: 183.473
          ci95_high_pct: 213.719
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 20
        user_cpu_ns:
          control_median: 18477000.0
          candidate_median: 66572500.0
          control_p95_over_median: 1.471
          candidate_p95_over_median: 1.41
          change_pct: 260.418
          ci95_low_pct: 218.742
          ci95_high_pct: 353.522
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 20
        system_cpu_ns:
          control_median: 72442000.0
          candidate_median: 201859000.0
          control_p95_over_median: 1.22
          candidate_p95_over_median: 1.289
          change_pct: 187.65
          ci95_low_pct: 145.969
          ci95_high_pct: 215.912
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 20
      qualification:
        campaign_stage: exploratory
        classification: inferior
        confirmable: false
        major_fault_delta_limit: 0.0
        noninferiority_margin_pct: 3.0
        reasons:
          - "cpu_ns exceeds its +50% regression limit"
          - "system_cpu_ns exceeds its +75% regression limit"
          - peak_rss_bytes is missing a paired percent interval
          - "minor_faults exceeds its +10% regression limit"
          - "voluntary_context_switches exceeds its +50% regression limit"
        resource_limits_pct:
          cpu_ns: 50.0
          involuntary_context_switches: 50.0
          minor_faults: 10.0
          peak_rss_bytes: 5.0
          system_cpu_ns: 75.0
          voluntary_context_switches: 50.0
        resources:
          cpu_ns: rejected
          involuntary_context_switches: within-limit
          major_faults: within-limit
          minor_faults: rejected
          peak_rss_bytes: inconclusive
          system_cpu_ns: rejected
          voluntary_context_switches: rejected
        policy_stable: null
        policy_rule: null
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: rejected
    primary_job: roots-default-tree
    primary_metric: wall_ns
    change_pct: 425.546
    reason: "not a speed decision: roots-default-tree 31.8 to 162.3 ms, +425.5% [+383.1%, +479.0%], about 0.48 ms a root, and roots-summary 27.7 to 139.5 ms, +417.9%, about 0.41 ms a root, about a fifth of macOS's exp-216 figure on the tree and half on the summary; far past fdu-ich9's 10% bar on Linux too"
    commit: 0d6011da
    kept: neither
---
## What was predicted

H196 on Linux, as `fdu-ich9` asked (H196’s registration is macOS’s, exp-216): a report
over several roots runs each root’s plan one after another, so each root pays a plan’s
fixed costs before it reads an entry.
exp-216 measured about 2.4 ms a root on the default tree and 0.9 ms on the summary over
625 crate directories on a loaded macOS host.
This cell asks the same question on the Linux host of the 0.5.0 release measurement
(`fdu-088k`), with no new prediction: the recorded macOS figures are the reference.

## What was measured

One interleaved run, quiet regime (the cell started at a 1-minute load average of 0.27
over 4 vCPUs, and the harness’s per-sample gate held CPU busy under 25% around every
timed sample), 3 warmups and 20 pairs per configuration, exploratory stage, on a frozen
copy of this host’s cargo registry source store (`cargo-registry-src-linux`: 272 crate
directories and nothing else at its top, 17,089 entries, 2,912 directories, 14,177
files, copied with `cp -a` from `~/.cargo/registry/src/index.crates.io-*` after the
release builds populated it).
No sample was invalid and the tree was unchanged.

One binary, the probe built from `0d6011da` (sha256 `d93d5a43`), in two configurations
bound by the variant notes: the store as one root (control) and each of its 272 crate
directories as a root (`--child-roots`, candidate), both through the several-roots door.

| Job | One root | 272 roots | Change [95% interval] | Per root |
| --- | ---: | ---: | --- | ---: |
| `roots-default-tree` | 31.8 ms | 162.3 ms | +425.5% [+383.1%, +479.0%] | about 0.48 ms |
| `roots-summary` | 27.7 ms | 139.5 ms | +417.9% [+377.4%, +464.5%] | about 0.41 ms |

CPU grows less than wall: total CPU +208% and +200%, system CPU +200% and +188%, user
CPU +344% and +260%, minor faults +120% and +61%. The parent walk keeps four walkers
busy (about 96 ms of CPU in 32 ms of wall); 272 small walks one after another cannot,
which is the shape a fixed per-root cost of pool start-up, calibration, and drain
predicts.

## Decision

Not a speed decision: several roots ship regardless.
On this Linux host a root costs about 0.5 ms on the default tree and 0.4 ms on the
summary, about a fifth of the macOS figure on the tree and half on the summary, and 272
small roots still take about five times one walk of their parent.
That is far past `fdu-ich9`’s 10% bar, so one walker pool across roots keeps its claim
to a hypothesis on both platforms.
Recorded `rejected` as a speed claim, keeping neither configuration, as exp-216 is.
