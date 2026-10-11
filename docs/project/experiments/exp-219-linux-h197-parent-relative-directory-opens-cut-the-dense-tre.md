---
title: "Linux: H197 parent-relative directory opens cut the dense tree's default tree 3.4% and summary 5.1%"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-219
  title: "Linux: H197 parent-relative directory opens cut the dense tree's default tree 3.4% and summary 5.1%"
  date: "2026-10-10"
  hypotheses:
    - H197
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
    control: "0d6011da probe (the 0.5.0 stack's head)"
    candidate: "d7cbfc71 probe (H197 on 0d6011da, branch claude/h197-relative-opens)"
    control_binary:
      name: roots
      sha256: d93d5a4361b6717accb93f52eed531936ca1d1f513555484d2f4c4658086c7a0
      size_bytes: 4266000
      args: []
    candidate_binary:
      name: h197
      sha256: 67091d30b3725cf26b35fc852dffa9b1ff3171204707bc85d92de3a3e6ca023c
      size_bytes: 4268896
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-219/run.json.gz
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 89968024.0
          candidate_median: 85502776.0
          control_p95_over_median: 1.176
          candidate_p95_over_median: 1.135
          change_pct: -5.062
          ci95_low_pct: -6.096
          ci95_high_pct: -1.758
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 100
        component_ns:
          control_median: 87216374.0
          candidate_median: 82780003.5
          control_p95_over_median: 1.168
          candidate_p95_over_median: 1.137
          change_pct: -5.073
          ci95_low_pct: -6.377
          ci95_high_pct: -3.312
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 100
        cpu_ns:
          control_median: 329178500.0
          candidate_median: 309974500.0
          control_p95_over_median: 1.114
          candidate_p95_over_median: 1.116
          change_pct: -4.938
          ci95_low_pct: -6.515
          ci95_high_pct: -3.015
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 100
        user_cpu_ns:
          control_median: 47007500.0
          candidate_median: 47579500.0
          control_p95_over_median: 1.532
          candidate_p95_over_median: 1.523
          change_pct: -1.641
          ci95_low_pct: -12.575
          ci95_high_pct: 12.936
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 100
        system_cpu_ns:
          control_median: 280401000.0
          candidate_median: 264215500.0
          control_p95_over_median: 1.151
          candidate_p95_over_median: 1.163
          change_pct: -6.4
          ci95_low_pct: -8.008
          ci95_high_pct: -3.565
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
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
    - job: default-tree
      start_state: warm
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 97012619.0
          candidate_median: 94390651.5
          control_p95_over_median: 1.177
          candidate_p95_over_median: 1.123
          change_pct: -3.35
          ci95_low_pct: -6.176
          ci95_high_pct: -0.925
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 100
        component_ns:
          control_median: 93843911.5
          candidate_median: 91307827.5
          control_p95_over_median: 1.167
          candidate_p95_over_median: 1.111
          change_pct: -3.458
          ci95_low_pct: -6.121
          ci95_high_pct: -0.91
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 100
        cpu_ns:
          control_median: 342265500.0
          candidate_median: 330865500.0
          control_p95_over_median: 1.114
          candidate_p95_over_median: 1.099
          change_pct: -3.893
          ci95_low_pct: -5.447
          ci95_high_pct: -2.397
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 100
        user_cpu_ns:
          control_median: 54925000.0
          candidate_median: 59678500.0
          control_p95_over_median: 1.415
          candidate_p95_over_median: 1.376
          change_pct: 7.614
          ci95_low_pct: -2.722
          ci95_high_pct: 17.532
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 100
        system_cpu_ns:
          control_median: 290964000.0
          candidate_median: 270662500.0
          control_p95_over_median: 1.108
          candidate_p95_over_median: 1.118
          change_pct: -6.431
          ci95_low_pct: -9.095
          ci95_high_pct: -3.18
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
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
    lines_changed: 403
    new_dependencies: []
    new_unsafe_blocks: 3
    new_failure_modes:
      - "a relative open lists the child of the directory the parent's listing named, which a tree renamed mid-walk may no longer name at the absolute path"
      - "descriptors held for queued subdirectories, bounded at a quarter of the soft RLIMIT_NOFILE and at most 1,024"
    notes: "Queue items carry an Arc'd parent descriptor; the native reader opens relative to it and falls back to the path on any failure; glibc Linux only."
  verdict:
    decision: accepted
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -3.35
    reason: "default-tree -3.35% [-6.18%, -0.93%] and aggregate-summary -5.06% [-6.10%, -1.76%] on node-modules-dense at 100 pairs, against -0.5% to -1.5% predicted; linux-v6.12 -1.22% and -1.11%, unresolved; linux-balanced-1m -2.34% [-4.47%, -0.94%], under the bar; against the no-ages arm still +4.01% [+1.19%, +6.66%] dense and +1.60% [+0.45%, +2.58%] generated, so ages do not reach the price of sizes alone; answers identical; accepted on the arithmetic, with the complexity judgment, rename semantics included, owed to fdu-tzv3's review; kept arm is the control until a reviewed pull request lands it"
    commit: d7cbfc71
    kept: control
---
## What was predicted

H197, registered before any timed sample of it (`b6792651`, with its cells fixed at 100
pairs in `8e23f2df`), with the maintainer’s question as its bar: can the default tree
tally sizes with ages at the price of sizes alone?

Since `1b3ac793` the default tree states every directory and symlink once, so that a
row’s age can count their times (H193). Each directory was also opened by its absolute
path, which makes the kernel resolve every component of the path again for every
directory: about seven components each on `node-modules-dense`. H197 is H169’s third
phase: a walker that lists a directory keeps the listing’s descriptor while the
subdirectories it queued wait, and each opens relative to it (`openat(parent, name)`).
The descriptor closes when the last of them has opened; at most a quarter of the soft
`RLIMIT_NOFILE` (at most 1,024) are kept at once, and a relative open that fails is
retried by path. The call count does not change; the path walk inside each open does.

Predicted: against the shipped head −0.5% to −1.5% on `node-modules-dense` and about −1%
on the generated tree, short of the accept rule; against the no-ages arm +1.5% to +3%,
so ages still not at the price of sizes alone; `aggregate-summary`, whose walk opens
every directory the same way, −0.5% to −1.5%.

## What was measured

Six two-arm cells, quiet regime (each started only after one-second CPU busy had stayed
under 15% and load per core under 0.25 for ten seconds; the harness’s per-sample gate
held CPU busy under 25% around every sample), 3 warmups and 100 pairs each, fixed in
advance, exploratory stage.
No sample was invalid and no tree changed.
Binaries, each tied to its commit and copied out of its worktree: the no-ages arm
(`61968412`, the shipped head with `1b3ac793`’s `scan.rs` reversed, sha256 `6a447e7d`),
the shipped head (`0d6011da`, `d93d5a43`), and H197 (`d7cbfc71`, `67091d30`, which is
`4df4e590` plus tests and a raw-pointer lint fix).

| Subject | Job | H197 against the shipped head | H197 against the no-ages arm |
| --- | --- | --- | --- |
| `node-modules-dense` | `default-tree` | **−3.35% [−6.18%, −0.93%]** | +4.01% [+1.19%, +6.66%] |
| `node-modules-dense` | `aggregate-summary` | **−5.06% [−6.10%, −1.76%]** | — |
| `linux-v6.12` | `default-tree` | −1.22% [−4.39%, +1.05%] | +2.52% [−1.64%, +6.02%] |
| `linux-v6.12` | `aggregate-summary` | −1.11% [−3.02%, +0.70%] | — |
| `linux-balanced-1m` | `default-tree` | −2.34% [−4.47%, −0.94%] | +1.60% [+0.45%, +2.58%] |

**Answers.** On all three subjects the H197 command line’s folded default tree equals
the full index and 0.4.0’s sizes on every row (`--full`, or depth 4 at every breadth on
the generated tree): 79,850, 92,412 and 8,675 rows.
Under `strace` every directory open but the root’s is relative
(`openat(4, "eslint", …)`), and the probe’s tallies equal the shipped head’s. 182 scan
tests pass, two new reader tests among them (a relative read equals a read by path under
every policy; a relative open that fails reads by path), and clippy is clean.

## Decision

On `node-modules-dense` H197 clears the accept rule on both jobs it reaches, −3.35% on
the default tree and −5.06% on the summary, more than predicted; on the kernel tree,
with half the directories per entry, neither interval resolves, and on the generated
tree the saving is resolved but under the 3% bar.
By the medians that is about 0.2 to 0.3 µs of wall per directory on all three trees, as
a per-directory mechanism predicts, and it pays on every route the concurrent walk
serves.

They do not answer the maintainer’s question in the affirmative.
Against the no-ages arm H197 is still +4.01% [+1.19%, +6.66%] on the dense tree and
+1.60% [+0.45%, +2.58%] on the generated one: the age column’s price is one metadata
call per directory and per symlink, and no call the walk already makes returns a
directory’s time.
Ages cost the same as sizes alone only if something else removes a call
per directory, or if the age counts files alone, which the folded tree already maintains
for nothing; the second is a definition the maintainer owns.

Accepted on `node-modules-dense`, on the accept rule’s arithmetic: the median is at
least 3% faster, the interval lies below zero, and no sample was invalid.
The rule’s fourth criterion, that the complexity is worth it, is a judgment this record
does not make. It is owed to the review that lands the change (`fdu-tzv3`), over the
complexity block above: 403 lines, three new `unsafe` blocks, descriptors held for
queued subdirectories, and the rename case.
On a tree renamed during the walk a relative open lists the child of the directory whose
listing named it, where the absolute path may by then name another directory or none, so
the registration’s “never decides an answer” holds only for a tree that is not renamed
mid-walk. The kept arm is recorded as the control, because the change is not in the
product: it is on `claude/h197-relative-opens`, outside the 0.5.0 stack, and ships only
if a reviewed pull request lands it.
