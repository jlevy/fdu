---
title: "macOS: H194 fixed-buffer instants halve the age column machine-format cost, about 0.12 microseconds a row remains"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-213
  title: "macOS: H194 fixed-buffer instants halve the age column machine-format cost, about 0.12 microseconds a row remains"
  date: "2026-10-09"
  hypotheses:
    - H194
  subject:
    tree_label: rustup
    tree_root_id: 36ce9b22af9a6164721fc2d04580d7da220ffb0de00e0a1c0cac4fd9e9cc21b6
    tree_engine_digest: 4304d9d4071fd4478a0510edf80c4b78594f98b84969d90a1d7230eb0dc94d78
    tree_provenance: "The rustup toolchain store for this machine's installed toolchains (root_id 36ce9b22). Shape depends on which toolchains and targets are installed, so it is not a recipe another machine can follow to the same tree."
    tree_reconstructible: false
    tree_entries: 77355
    tree_directories: 3427
    tree_files: 73928
    tree_symlinks: 0
    tree_apparent_bytes: 3750189949
    tree_allocated_bytes: 3978313728
    tree_max_depth: 17
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
    control: "148ef78e probe: main before the age column (sha256 d2ac70ff, the binary exp-209 to exp-212 used; the run variant notes are empty, so the binding is stated in the record body)"
    candidate: "b2968074 probe, clean tree: exp-212 candidate ae90aef4 plus each machine-output instant written into a fixed stack buffer instead of a String per row (review C7 on #191, fdu-oiuc; sha256 c4e01629; binding stated in the record body)"
    control_binary:
      name: control
      sha256: d2ac70ff129f6c510100a0f58a27677015fec20af2f8d29b732f6e29e2182041
      size_bytes: 3363840
      args: []
    candidate_binary:
      name: candidate
      sha256: c4e0162987460cadcf82dc566df7c3b10119597d3715cdc7cac9d6befd13f53f
      size_bytes: 3380352
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-213/run.json.gz
  results:
    - job: render-json
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 748204437.5
          candidate_median: 760104958.5
          control_p95_over_median: 1.035
          candidate_p95_over_median: 1.022
          change_pct: 1.302
          ci95_low_pct: 0.568
          ci95_high_pct: 1.783
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 128427729.5
          candidate_median: 146512625.0
          control_p95_over_median: 1.004
          candidate_p95_over_median: 1.01
          change_pct: 13.941
          ci95_low_pct: 13.601
          ci95_high_pct: 14.763
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        cpu_ns:
          control_median: 1394779500.0
          candidate_median: 1358590000.0
          control_p95_over_median: 1.072
          candidate_p95_over_median: 1.017
          change_pct: -0.991
          ci95_low_pct: -5.03
          ci95_high_pct: 3.676
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 616449000.0
          candidate_median: 627922500.0
          control_p95_over_median: 1.01
          candidate_p95_over_median: 1.017
          change_pct: 2.37
          ci95_low_pct: 1.887
          ci95_high_pct: 2.889
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 12
        system_cpu_ns:
          control_median: 780488500.0
          candidate_median: 732423000.0
          control_p95_over_median: 1.124
          candidate_p95_over_median: 1.027
          change_pct: -3.087
          ci95_low_pct: -10.381
          ci95_high_pct: 4.787
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 129122304.0
          candidate_median: 128991232.0
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.015
          change_pct: -0.133
          ci95_low_pct: -0.619
          ci95_high_pct: 1.121
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
    - job: render-jsonl
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 724798312.5
          candidate_median: 741922313.0
          control_p95_over_median: 1.06
          candidate_p95_over_median: 1.033
          change_pct: 1.734
          ci95_low_pct: 0.586
          ci95_high_pct: 3.839
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        component_ns:
          control_median: 113695979.5
          candidate_median: 130492708.5
          control_p95_over_median: 1.007
          candidate_p95_over_median: 1.02
          change_pct: 14.557
          ci95_low_pct: 13.709
          ci95_high_pct: 15.644
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        cpu_ns:
          control_median: 1310549500.0
          candidate_median: 1314125000.0
          control_p95_over_median: 1.179
          candidate_p95_over_median: 1.126
          change_pct: 0.122
          ci95_low_pct: -1.403
          ci95_high_pct: 1.215
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
          pairs: 12
        user_cpu_ns:
          control_median: 594587000.0
          candidate_median: 612497500.0
          control_p95_over_median: 1.017
          candidate_p95_over_median: 1.013
          change_pct: 3.031
          ci95_low_pct: 2.498
          ci95_high_pct: 3.568
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 713409000.0
          candidate_median: 701689500.0
          control_p95_over_median: 1.318
          candidate_p95_over_median: 1.226
          change_pct: -1.993
          ci95_low_pct: -4.326
          ci95_high_pct: -0.665
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 12
        peak_rss_bytes:
          control_median: 129490944.0
          candidate_median: 129327104.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.003
          change_pct: -0.113
          ci95_low_pct: -0.511
          ci95_high_pct: 0.114
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
    - job: render-yaml
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 811321937.5
          candidate_median: 826969562.5
          control_p95_over_median: 1.018
          candidate_p95_over_median: 1.013
          change_pct: 2.149
          ci95_low_pct: 0.974
          ci95_high_pct: 2.55
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: noninferior
          pairs: 12
        component_ns:
          control_median: 118681375.0
          candidate_median: 136843124.5
          control_p95_over_median: 1.006
          candidate_p95_over_median: 1.01
          change_pct: 15.578
          ci95_low_pct: 15.106
          ci95_high_pct: 16.445
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 12
        cpu_ns:
          control_median: 1729876000.0
          candidate_median: 1750703500.0
          control_p95_over_median: 1.045
          candidate_p95_over_median: 1.03
          change_pct: 0.349
          ci95_low_pct: -0.745
          ci95_high_pct: 3.01
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        user_cpu_ns:
          control_median: 602451000.0
          candidate_median: 618753000.0
          control_p95_over_median: 1.009
          candidate_p95_over_median: 1.01
          change_pct: 2.929
          ci95_low_pct: 2.44
          ci95_high_pct: 3.429
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 12
        system_cpu_ns:
          control_median: 1128697500.0
          candidate_median: 1131778500.0
          control_p95_over_median: 1.068
          candidate_p95_over_median: 1.046
          change_pct: -1.012
          ci95_low_pct: -2.582
          ci95_high_pct: 3.052
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 12
        peak_rss_bytes:
          control_median: 128737280.0
          candidate_median: 128770048.0
          control_p95_over_median: 1.007
          candidate_p95_over_median: 1.007
          change_pct: -0.204
          ci95_low_pct: -0.368
          ci95_high_pct: 0.388
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
  reference_tools:
    - name: dust
      wall_ns_median: 179290479.0
      argv:
        - "{binary}"
        - "-d"
        - "1"
        - "--no-progress"
        - "{root}"
  complexity:
    lines_changed: 52
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "b2968074: a fixed thirty-byte layout for years 0 to 9999 with the general format! spelling kept as the fallback, in query_values.rs and the one call site (emit_instant) in report_format.rs, plus a 45-line test that holds the two spellings to the same bytes; the measured pair spans all of the #191 engine change against 148ef78e"
  verdict:
    decision: rejected
    primary_job: render-json
    primary_metric: wall_ns
    change_pct: 1.302
    reason: "not a speed decision: the age column ships regardless, and this measures the review C7 fix (b2968074, fdu-oiuc) against the pre-age control: render-json wall +1.30% [+0.57%, +1.78%], component 128.4 to 146.5 ms; render-jsonl wall +1.73% [+0.59%, +3.84%]; render-yaml wall +2.15% [+0.97%, +2.55%], component 118.7 to 136.8 ms; read across runs against exp-212 (+4.27% and +5.98% wall, same control binary) the buffer removes about half the added render cost, and the residual of about 0.12 us a row is the new per-row data itself, accepted as the price of the machine-output timestamps; the Python eager instants are unmeasured"
    commit: b2968074
    kept: candidate
---
## What was predicted

H194, registered with this record: review C7 on
[#191](https://github.com/jlevy/fdu/pull/191)’s remedy for the one resolved cost in
[exp-212](exp-212-macos-the-age-column-re-measured-at-the-shipped-head-per-row.md).
Every machine-format row gained `modified_at`, and `format_rfc3339_nanos` built each one
as a new `String` with a seven-field zero-padded `format!`; exp-212 put the new fields
at about 0.23 µs a row in JSON and 0.25 µs in YAML. `b2968074` writes the thirty RFC
3339 bytes into a fixed stack buffer handed to the sink (`with_rfc3339_nanos`), and
keeps the general spelling for a year outside 0 to 9999, which only a corrupt timestamp
reaches; a test holds the two to the same bytes.
`fdu-oiuc` named the remedy and the jobs to re-run (`render-json`, `render-jsonl`,
`render-yaml`) before the change, but no number, so nothing here can be an accept.

## What was measured

One interleaved probe run on the `rustup` toolchain store (77,355 entries, 3,427
directories, depth 17), the same tree as exp-212, 3 warmups and 12 timed trials per
variant, exploratory stage, on an uncontrolled host: the 1-minute load average was 13.0
at the start and 10.3 at the end (15-minute 15.6 and 15.0) over 10 cores, the CPU 32%
and 28% busy. No sample was invalid and the tree was unchanged.

The run’s variant notes are empty, so the binary-to-commit binding is stated here
instead (automating it is `fdu-ce61`). The control is the probe built from `148ef78e`
(sha256 `d2ac70ff`, the binary exp-209 to exp-212 used).
The candidate was built from `b2968074` with a clean tree (sha256 `c4e01629`); its
engine differs from exp-212’s candidate (`ae90aef4`) only by `b2968074`, since
`405ddce7` and `69452cf5` change records and documentation.

Each job streams an unbounded tree and a file list, about 151,000 rows, into a discard
writer after untimed report construction; the component is the render alone.

- `render-json` component 128.4 ms to 146.5 ms, +13.94% [+13.60%, +14.76%]; wall +1.30%
  [+0.57%, +1.78%], non-inferior at +3%. Primary.
- `render-jsonl` component 113.7 ms to 130.5 ms, +14.56% [+13.71%, +15.64%]; wall +1.73%
  [+0.59%, +3.84%], not resolved against the +3% margin.
- `render-yaml` component 118.7 ms to 136.8 ms, +15.58% [+15.11%, +16.45%]; wall +2.15%
  [+0.97%, +2.55%], non-inferior at +3%.
- User CPU +2.37%, +3.03%, and +2.93%; every peak RSS and minor-fault interval includes
  zero.

The control medians match exp-212’s within 0.4 ms (128.8 and 118.4 ms there), so the
fix’s own effect can be read across the two runs, though it is not a paired figure: this
run pairs against the pre-age control, not against `ae90aef4`. The added render
component fell from 34.3 ms to 18.1 ms in JSON (47% of it removed) and from 38.0 ms to
18.2 ms in YAML (52%), and the wall cost from +4.27% and +5.98% in exp-212 to +1.30% and
+2.15% here. What remains is about 0.12 µs a row in JSON and YAML and 0.11 µs in JSON
Lines.

## Decision

Not a speed decision, as exp-212 is not: the age column and its machine-output
timestamps ship regardless, so this records the fix’s effect and the price left after
it, `rejected` as a speed claim with the candidate kept.
The fixed buffer removes about half of what the age column added to a full JSON or YAML
render. The residual, 1% to 2% of the render’s wall and about 0.12 µs a row, is writing
the new data itself: four fields on every tree node (`mtime_ns`, `complete`, `age_ns`,
`modified_at`) and an instant on every row.
It is accepted as the price of the requested machine-output timestamps, which is the
decision `fdu-oiuc` was to take before 0.5.0. Review C7’s third step, a 100k-row Python
list report to decide whether Python’s eager `datetime` derivation should be lazy, was
not measured here. All of it is one loaded macOS host; Linux is unmeasured.
