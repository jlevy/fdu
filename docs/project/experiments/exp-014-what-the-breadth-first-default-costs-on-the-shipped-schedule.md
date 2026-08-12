---
title: "What the breadth-first default costs, on the shipped scheduler"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-014
  title: "What the breadth-first default costs, on the shipped scheduler"
  date: 2026-08-11
  hypotheses:
    - "H50: exp-012 measured a scheduler that no longer exists and exp-013 compared two breadth-first schedulers, so what the shipped default costs against depth-first is unmeasured"
  subject:
    tree_label: metabrowser
    tree_root_id: dbd79ed9c898f7a2f66530cd95bb61cab88e798375134b86c77ece761de580a9
    tree_engine_digest: c631fbf39d7c7adace225d5c9935aaf991176d05da800abd7a69c56ceb0f3b0e
    tree_entries: 60067
    tree_directories: 7350
    tree_files: 52695
    tree_symlinks: 22
    tree_apparent_bytes: 1085083672
    tree_allocated_bytes: 1230073856
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
    os_cache: warm-steady
  method:
    trials: 20
    warmups: 3
    interleaved: true
    control: "--order depth-first"
    candidate: "--order breadth-first (the shipped default)"
    control_binary:
      name: control
      sha256: 1bc4da85fa40d31db9956da0175acb57faca2fc796a9931b36bf5db284780a07
      size_bytes: 535872
      args:
        - "--order"
        - depth-first
    candidate_binary:
      name: candidate
      sha256: 1bc4da85fa40d31db9956da0175acb57faca2fc796a9931b36bf5db284780a07
      size_bytes: 535872
      args:
        - "--order"
        - breadth-first
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    run_artifact: benchmarks/results/realtree/run-order-current.json
  results:
    - job: cold-scan-index
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 321521042.0
          candidate_median: 323481354.0
          change_pct: 0.499
          ci95_low_pct: -1.393
          ci95_high_pct: 1.977
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        component_ns:
          control_median: 207144500.0
          candidate_median: 204053542.0
          change_pct: 0.002
          ci95_low_pct: -2.899
          ci95_high_pct: 3.791
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        cpu_ns:
          control_median: 1190543500.0
          candidate_median: 1176841000.0
          change_pct: -0.645
          ci95_low_pct: -3.77
          ci95_high_pct: 2.039
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        user_cpu_ns:
          control_median: 244908500.0
          candidate_median: 244956000.0
          change_pct: 0.019
          ci95_low_pct: -0.969
          ci95_high_pct: 1.577
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        system_cpu_ns:
          control_median: 944599000.0
          candidate_median: 932328000.0
          change_pct: -0.684
          ci95_low_pct: -5.225
          ci95_high_pct: 2.775
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unknown
          pairs: 0
        peak_rss_bytes:
          control_median: 34643968.0
          candidate_median: 34078720.0
          change_pct: -1.763
          ci95_low_pct: -2.633
          ci95_high_pct: -0.736
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 20
    - job: cold-scan-producer
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 489368041.0
          candidate_median: 469288291.0
          change_pct: -3.043
          ci95_low_pct: -5.99
          ci95_high_pct: -0.96
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 20
        component_ns:
          control_median: 192025166.5
          candidate_median: 191479854.0
          change_pct: -2.844
          ci95_low_pct: -4.638
          ci95_high_pct: -0.449
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 20
        cpu_ns:
          control_median: 2122030500.0
          candidate_median: 2049799000.0
          change_pct: -3.162
          ci95_low_pct: -4.73
          ci95_high_pct: -1.081
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 20
        user_cpu_ns:
          control_median: 312457000.0
          candidate_median: 314631500.0
          change_pct: 0.394
          ci95_low_pct: -2.19
          ci95_high_pct: 2.524
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        system_cpu_ns:
          control_median: 1812242500.0
          candidate_median: 1733154000.0
          change_pct: -3.782
          ci95_low_pct: -5.912
          ci95_high_pct: -0.829
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          pairs: 20
        blocked_ns:
          control_median: 0.0
          candidate_median: 0.0
          change_pct: 0.0
          ci95_low_pct: null
          ci95_high_pct: null
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unknown
          pairs: 0
        peak_rss_bytes:
          control_median: 34570240.0
          candidate_median: 34136064.0
          change_pct: -1.051
          ci95_low_pct: -1.953
          ci95_high_pct: 0.191
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
    - job: warm-revalidate
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 624373208.0
          candidate_median: 637653916.5
          change_pct: 2.704
          ci95_low_pct: 1.546
          ci95_high_pct: 3.367
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 20
        component_ns:
          control_median: 416569666.5
          candidate_median: 431350875.0
          change_pct: 4.502
          ci95_low_pct: 2.678
          ci95_high_pct: 5.152
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 20
        cpu_ns:
          control_median: 618758000.0
          candidate_median: 632313000.0
          change_pct: 2.478
          ci95_low_pct: 1.178
          ci95_high_pct: 3.167
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 20
        user_cpu_ns:
          control_median: 242260000.0
          candidate_median: 244468500.0
          change_pct: 0.785
          ci95_low_pct: 0.413
          ci95_high_pct: 1.252
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 20
        system_cpu_ns:
          control_median: 375781000.0
          candidate_median: 388401500.0
          change_pct: 3.536
          ci95_low_pct: 1.178
          ci95_high_pct: 4.492
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          pairs: 20
        blocked_ns:
          control_median: 5178041.5
          candidate_median: 6056750.0
          change_pct: 6.336
          ci95_low_pct: -3.995
          ci95_high_pct: 30.701
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
        peak_rss_bytes:
          control_median: 32014336.0
          candidate_median: 32440320.0
          change_pct: 1.113
          ci95_low_pct: -0.127
          ci95_high_pct: 1.671
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          pairs: 20
  reference_tools: []
  complexity:
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: "No code change. Both arms are the same binary under different --order flags, which removes build-to-build variation from a single-digit-percent effect."
  verdict:
    decision: baseline
    primary_job: cold-scan-producer
    primary_metric: wall_ns
    change_pct: -3.043
    reason: "Same binary both arms, 20 interleaved paired trials. Breadth-first is now cheaper where region scheduling reaches: cold-scan-producer wall -3.04% [-5.99%, -0.96%], cold-scan-index peak RSS -1.76% [-2.63%, -0.74%]. warm-revalidate regressed +2.70% [+1.55%, +3.37%] because reconcile walks with the serial take_next and region scheduling never reached it. No code changed; the warm asymmetry is tracked as fdu-v71x"
    commit: null
---
# What the breadth-first default costs, on the shipped scheduler

## Hypothesis

H50: exp-012 answered "what does choosing breadth-first cost?" against an implementation
that no longer exists — a single global FIFO, since replaced by region scheduling
(exp-013). exp-013 in turn compared two *breadth-first* schedulers against each other.
Neither measures the question a reader actually has, which is what the shipped default
costs against the alternative a caller could select.

Predicted: the cold jobs improve or hold, because they are the ones region scheduling
touches. The warm sweep is unaffected either way, because it runs its own walk.

## What was tried

Nothing was changed. One binary, both arms, `--order depth-first` as control and
`--order breadth-first` as candidate, twenty interleaved paired trials per job. Running
both arms from the same binary removes build-to-build variation entirely, which matters
here because the effects are single-digit percentages.

## What the numbers said

| job | scheduler | wall | 95% interval | evidence |
| --- | --- | ---: | --- | --- |
| `cold-scan-producer` | region | −3.04% | [−5.99%, −0.96%] | improved |
| `cold-scan-index` | region | +0.50% | [−1.39%, +1.98%] | unclear |
| `warm-revalidate` | serial FIFO | +2.70% | [+1.55%, +3.37%] | **regressed** |

Peak RSS on `cold-scan-index` is −1.76% [−2.63%, −0.74%], and producer CPU is −3.16%
[−4.73%, −1.08%]. So on the paths region scheduling reaches, breadth-first is now
*cheaper* than depth-first on memory and on producer throughput, having cost memory in
exp-012.

**The warm sweep regressed, and the prediction that it would be unaffected was wrong.**
`reconcile` walks with the serial `take_next`, not with `DirectoryQueue`, so region
scheduling never reached it: breadth-first there is still the front-popping global FIFO
that exp-013 replaced everywhere else, paying the same locality and frontier costs. Wall
+2.70% [+1.55%, +3.37%], component +4.50% [+2.68%, +5.15%], CPU +2.48% [+1.18%, +3.17%].

The regression is small but real, and it survived re-measurement. A first run at 14
trials on a loaded machine put it at +5.96% [+2.33%, +18.20%]; 24 trials gave +2.15%
[−0.19%, +4.43%], whose interval includes zero; 20 clean trials gave the +2.70%
[+1.55%, +3.37%] recorded here. Three runs, one direction, converging as the noise fell.

## What it means

Breadth-first buys orientation: a consumer watching a walk sees top-level totals fill
together instead of one subtree finishing while its siblings read zero. On the warm
sweep, **a one-shot CLI reads none of that** — it prints after reconciliation completes
— so on that path the property is currently bought and thrown away, at 2.7% of the
slowest job in the suite.

Two ways out, neither measured yet: extend region scheduling to the reconcile sweep, or
let the sweep default to depth-first and take breadth-first only from a caller that
reads progressively. The second is closer to the project's own position that traversal
order is a consumer contract rather than an engine setting.

**Reviewed and deliberately not chased.** 2.7% of one job is below the 3% bar this
project applies to changes worth added complexity, and the same effort is worth far more
elsewhere: the adaptive worker pool is estimated at roughly 2x on cold large trees
(`fdu-tt2j`), persisted roll-ups and lazy open turn an 11-second warm load into a first
paint (`fdu-1vd0`), and `open` still accounts for about 28% of cold self-time. Fixing a
2.7% warm regression ahead of any of those would be optimising the wrong thing. It stays
recorded so it is a decision rather than an oversight, and tracked at low priority as
`fdu-v71x`.

## Limitations

One tree, one host. The warm sweep's cost is a locality effect and locality is exactly
what varies most across filesystems and cache states, so the magnitude should not be
quoted as general — only the sign, which three runs agree on.

## Verdict

**BASELINE** (no code changed; recorded as the reference point for what the default costs). The default stays breadth-first: it is cheaper on
both cold jobs and on memory, and the warm cost is 2.7% of one job against an
orientation property the interactive use case needs. The warm-path asymmetry is recorded
rather than smoothed over, and is tracked as its own work item.
