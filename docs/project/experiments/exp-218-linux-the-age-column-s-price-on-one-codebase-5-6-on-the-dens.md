---
title: "Linux: the age column's price on one codebase, 5.6% on the dense tree, inside noise on the kernel tree"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-218
  title: "Linux: the age column's price on one codebase, 5.6% on the dense tree, inside noise on the kernel tree"
  date: "2026-10-10"
  hypotheses:
    - H193
  subject:
    tree_label: node-modules-dense
    tree_root_id: ffbd3cb6be62b77f7e2e9b5964d1e380e8a3f3b7bb4fda55f462ae4b189841b5
    tree_engine_digest: 62125c60d0dddf9fdb9f89b8c2542c8afd573f3460ccc6f052626099669b7751
    tree_provenance: "npm dependency trees of react-scripts 5.0.1 and gatsby 5.13.7: copy docs/project/experiments/evidence/exp-190/subject-package.json.txt and subject-package-lock.json.txt to package.json and package-lock.json in an empty directory and run npm ci --ignore-scripts (node 22.22.0, npm 10.9.4); rebuilt 2026-10-10 on this host: 79,957 entries, 9,439 directories, 107 symlinks, no .gitignore, the counts exp-197 recorded."
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
    host_memory_bytes: 16874930176
    host_system: Linux 6.18.44-fc-v114
    filesystem: ext4
    host_virtualization: virtualized
    os_cache: warm-steady
  method:
    trials: 100
    warmups: 3
    interleaved: true
    control: "61968412 probe (no-ages arm: 0d6011da with 1b3ac793's scan.rs reversed; measurement only)"
    candidate: "0d6011da probe (the 0.5.0 stack's head)"
    control_binary:
      name: noage
      sha256: 6a447e7db49c3e1c8d7d5e99dbab0d5483cf79188e2bfd2c8b169c71138ee464
      size_bytes: 4266208
      args: []
    candidate_binary:
      name: roots
      sha256: d93d5a4361b6717accb93f52eed531936ca1d1f513555484d2f4c4658086c7a0
      size_bytes: 4266000
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-218/run.json.gz
  results:
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 92045557.5
          candidate_median: 97770694.5
          control_p95_over_median: 1.299
          candidate_p95_over_median: 1.304
          change_pct: 5.627
          ci95_low_pct: 4.073
          ci95_high_pct: 9.274
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 100
        component_ns:
          control_median: 88413956.0
          candidate_median: 94604211.5
          control_p95_over_median: 1.247
          candidate_p95_over_median: 1.297
          change_pct: 6.598
          ci95_low_pct: 3.953
          ci95_high_pct: 9.252
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inferior
          pairs: 100
        cpu_ns:
          control_median: 328035500.0
          candidate_median: 337281000.0
          control_p95_over_median: 1.196
          candidate_p95_over_median: 1.224
          change_pct: 3.28
          ci95_low_pct: 0.565
          ci95_high_pct: 5.615
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 100
        user_cpu_ns:
          control_median: 58656500.0
          candidate_median: 59942500.0
          control_p95_over_median: 1.381
          candidate_p95_over_median: 1.479
          change_pct: 3.798
          ci95_low_pct: -5.779
          ci95_high_pct: 11.549
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 100
        system_cpu_ns:
          control_median: 273860000.0
          candidate_median: 281433000.0
          control_p95_over_median: 1.175
          candidate_p95_over_median: 1.239
          change_pct: 3.861
          ci95_low_pct: 1.748
          ci95_high_pct: 5.937
          significant: false
          passes_acceptance: false
          ci_excludes_zero: true
          direction: regressed
          noninferiority: inconclusive
          pairs: 100
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
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: rejected
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: 5.627
    reason: "not a speed decision: the shipped head against the same code without the directory statx, default-tree +5.63% [+4.07%, +9.27%] on node-modules-dense and +0.42% [-1.28%, +2.97%] on linux-v6.12 at 100 pairs, against +2% to +4% and 0 to +2% predicted; the only system-call difference is statx, +7,887 and +5,181; user-space instructions +0.26%, consumer thread flat; a files-only age would need no call, a definition the maintainer owns"
    commit: 0d6011da
    kept: candidate
---
## What was predicted

H193 states the age column’s Linux price: since `1b3ac793` the folded tree that answers
the default command states every directory and symlink, giving back H185 (exp-197). The
maintainer asked, before any timed sample, whether the tree can tally sizes with ages at
the price of sizes alone, so H193 was amended (`f63af680`, then `8e23f2df` before any
replicate) with a measurement-only no-ages arm: the shipped head with `1b3ac793`’s
`scan.rs` reversed (`61968412`), so the folded tree takes H185’s policy again.
Its directory ages are wrong by construction and it never ships; against the shipped
head it prices the column on one codebase, apart from everything else the stack changed.
Predicted the same as H193: +2% to +4% on `node-modules-dense`, 0 to +2% on
`linux-v6.12`, +3.5% to +5.5% on `linux-balanced-1m`; the price is the walkers’ kernel
time, not the fold’s maximum or the rendered cells.

## What was measured

**Wall** (two-arm cells, quiet regime, 3 warmups and 100 pairs fixed in advance; no
invalid sample; the shipped head `0d6011da`, sha256 `d93d5a43`, against the no-ages arm,
`6a447e7d`):

| Subject | Directories | Shipped head against the no-ages arm |
| --- | ---: | --- |
| `node-modules-dense` (this record) | 9,439 of 79,957 entries | **+5.63% [+4.07%, +9.27%]** |
| `linux-v6.12` | 5,769 of 92,474 | +0.42% [−1.28%, +2.97%] |

On `linux-balanced-1m` the four-arm cell of exp-217 (20 pairs) put the no-ages arm at
−7.95% [−8.90%, −4.30%] against the shipped head, but there the shipped head ran in a
middle slot of every ordinal, which an A/A cell of this session (exp-217) puts at about
+3% on its own; H197’s two-arm cells bound it differently (exp-219).

**System calls** (`strace -f -c` of the probe’s `default-tree`, load-independent): the
only difference between the arms is `statx`, +7,887 on `node-modules-dense` (79,966
against 72,079; predicted +7,886), +5,181 on `linux-v6.12` (92,841 against 87,660;
predicted +5,180), and +116,391 on `linux-balanced-1m` (1,000,010 against 883,619;
predicted about +125k, less because the shipped H185 policy already stated about 8,600
of those directories to prove each listing searchable).
Every other call count is equal: 9,447 `openat`, 18,878 `getdents64`, and 9,447 `close`
on the dense tree, in both arms.

**Instructions** (callgrind, user space only, one run each): `default-tree` on
`node-modules-dense` 213.57M in the no-ages arm against 214.13M at the shipped head
(+0.26%), with the consumer thread at 104.74M against 104.63M. The fold’s maximum and
the age cells cost nothing a user-space count resolves; the price is kernel time.

## Decision

Not a speed decision: the column ships regardless, and this records what it costs.
The cost is one `statx` per directory and per symlink, exactly the counts predicted, and
it is kernel time, so it scales with directories per entry: +5.63% of the default tree
on the directory-dense real tree, about twice the prediction, and not resolved on the
kernel tree, whose interval stays inside the +3% margin.
A files-only age, which the folded tree already maintains as `newest_mtime_ns`, would
need no call at all; counting directories and symlinks is the definition the plan chose
and the maintainer owns.
H197 (exp-219) wins back part of it.
Recorded `rejected` as a speed claim, with the shipped head kept.
