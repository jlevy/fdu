---
title: "Linux: the pdu track end to end, the default tree 3% and 9% faster and ahead of every pdu mode on both real trees"
softschema:
  contract: fdu.performance:Experiment/v1
  schema: experiment.schema.yaml
  envelope: experiment
  status: enforced
experiment:
  id: exp-201
  title: "Linux: the pdu track end to end, the default tree 3% and 9% faster and ahead of every pdu mode on both real trees"
  date: "2026-09-30"
  hypotheses:
    - H185
    - H186
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
    control: ebc06c78 probe (the final head of the round)
    candidate: "a356d456 probe (the H186 head, the pdu track's shipped stack)"
    control_binary:
      name: final
      sha256: 938370e0cbdc5885888514aa8ec2caa854b151cddab9c83c11712b0251525e82
      size_bytes: 3862104
      args: []
    candidate_binary:
      name: h186
      sha256: 28d57669f1b5cd8bd311157ccdd3006f76df1a3eaf30df915a3d2de6e589af60
      size_bytes: 3863864
      args: []
    toolchain: rustc 1.97.1 (8bab26f4f 2026-07-14)
    build_profile: release
    campaign_stage: exploratory
    confidence_interval: paired-bootstrap-median-95-v1
    stopping_rule: fixed-N-no-optional-stopping-v1
    run_artifact: docs/project/experiments/evidence/exp-201/run.json.gz
  results:
    - job: aggregate-summary
      start_state: cold
      invalid_samples: 0
      metrics:
        wall_ns:
          control_median: 91737450.0
          candidate_median: 84618227.0
          control_p95_over_median: 1.04
          candidate_p95_over_median: 1.078
          change_pct: -6.14
          ci95_low_pct: -8.699
          ci95_high_pct: -3.592
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 88932876.5
          candidate_median: 81996151.5
          control_p95_over_median: 1.036
          candidate_p95_over_median: 1.067
          change_pct: -6.118
          ci95_low_pct: -8.835
          ci95_high_pct: -2.868
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 334498500.0
          candidate_median: 313675500.0
          control_p95_over_median: 1.041
          candidate_p95_over_median: 1.07
          change_pct: -4.591
          ci95_low_pct: -8.373
          ci95_high_pct: -2.404
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        user_cpu_ns:
          control_median: 94306000.0
          candidate_median: 69210000.0
          control_p95_over_median: 1.2
          candidate_p95_over_median: 1.187
          change_pct: -26.575
          ci95_low_pct: -35.037
          ci95_high_pct: -24.653
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        system_cpu_ns:
          control_median: 242291000.0
          candidate_median: 251094000.0
          control_p95_over_median: 1.107
          candidate_p95_over_median: 1.069
          change_pct: 4.218
          ci95_low_pct: -3.606
          ci95_high_pct: 9.051
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
          control_median: 88514417.5
          candidate_median: 85219402.5
          control_p95_over_median: 1.058
          candidate_p95_over_median: 1.141
          change_pct: -3.051
          ci95_low_pct: -5.806
          ci95_high_pct: -1.03
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        component_ns:
          control_median: 85476614.0
          candidate_median: 81991787.5
          control_p95_over_median: 1.057
          candidate_p95_over_median: 1.138
          change_pct: -3.825
          ci95_low_pct: -6.227
          ci95_high_pct: -1.198
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: superior
          pairs: 20
        cpu_ns:
          control_median: 321221500.0
          candidate_median: 309774000.0
          control_p95_over_median: 1.045
          candidate_p95_over_median: 1.102
          change_pct: -2.576
          ci95_low_pct: -5.431
          ci95_high_pct: -1.513
          significant: true
          passes_acceptance: true
          ci_excludes_zero: true
          direction: improved
          noninferiority: noninferior
          pairs: 20
        user_cpu_ns:
          control_median: 65719500.0
          candidate_median: 65712000.0
          control_p95_over_median: 1.278
          candidate_p95_over_median: 1.335
          change_pct: -9.826
          ci95_low_pct: -22.768
          ci95_high_pct: 12.613
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: inconclusive
          pairs: 20
        system_cpu_ns:
          control_median: 262326000.0
          candidate_median: 252539500.0
          control_p95_over_median: 1.072
          candidate_p95_over_median: 1.049
          change_pct: -2.3
          ci95_low_pct: -5.613
          ci95_high_pct: 1.517
          significant: false
          passes_acceptance: false
          ci_excludes_zero: false
          direction: unclear
          noninferiority: noninferior
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
    lines_changed: 0
    new_dependencies: []
    new_unsafe_blocks: 0
    new_failure_modes: []
    notes: ""
  verdict:
    decision: baseline
    primary_job: default-tree
    primary_metric: wall_ns
    change_pct: -3.051
    reason: "Confirms the track's accepted changes in one paired cell per tree against the round's final head, with pdu default, pdu --max-depth 2 and diskus paired with the shipped head in the same run; no decision rests on it."
    commit: a356d456
    kept: neither
---
## What was predicted

The pdu track end to end, from
[the uniformly-faster brief](../research/research-2026-09-29-uniformly-faster-than-pdu.md):
the round’s final head (`ebc06c78`, the engine this branch is based on) against the
track’s shipped head (`a356d456`: H185, H188 with H189, and H186; H187 was rejected in
exp-200 and reverted), in one paired cell per real tree, with the peer standing (pdu
0.24.0 default, pdu `--max-depth 2`, diskus 0.9.0) in the same session so the goal the
track was opened for, “uniformly faster than pdu”, is answered on the same host and
hour.

No decision rests on this record: each change carried its own verdict in its own cell
(exp-197, exp-198, exp-199). What it settles is the size of the stacked effect, in
particular H186’s, whose deciding comparison (−4.91%) and replicate (−0.93%) disagreed
on the dense tree, and whether the kernel tree, where H185 and H186 were each not
resolvable alone, shows their sum.

- **Expected, stacked:** `default-tree` on `node-modules-dense` −6% to −10% (H185 and
  H186); on `linux-v6.12` −2% to −5% (each below resolution alone); `aggregate-summary`
  on `linux-v6.12` −5% to −9% (H188), on `node-modules-dense` unchanged.
- **Standing:** fdu’s default command against pdu’s default, pdu `--max-depth 2` and
  diskus on both trees, each competitor paired with the shipped head in one interleaved
  run, with the final head’s product binary as a fourth competitor so the same run
  carries the track’s own delta on the product command line.

## What was measured

**Fingerprints.** The cells checked the subjects against fingerprints registered for
this track (`tree-linux-v6.12-pdu.json`, `tree-node-modules-dense-pdu.json`), not the
shared ones: the shared `linux-v6.12` fingerprint, taken at 07:48 UTC, no longer matched
the subject after its `.git/index` was touched at 19:00 UTC. Entry counts and bytes are
identical to the shared fingerprint’s, and the subject was not modified by this track.
The tool comparisons wrote their own pre-run fingerprints beside the shared tool
results.

**Wall, the stacked probes.** Quiet, 20 pairs, no invalid samples.
Control: `ebc06c78` probe (the final head of the round).
Candidate: `a356d456` probe (the H186 head, the track’s shipped stack).

| Subject, job | Final head | Shipped head | Change |
| --- | ---: | ---: | --- |
| `linux-v6.12`, `default-tree` | 88.5 ms | 85.2 ms | **−3.05% [−5.81%, −1.03%]** |
| `linux-v6.12`, `default-tree --no-controls` | 85.8 ms | 81.3 ms | **−4.74% [−7.64%, −2.35%]** |
| `linux-v6.12`, `aggregate-summary` | 91.7 ms | 84.6 ms | **−6.14% [−8.70%, −3.59%]** |
| `linux-v6.12`, `aggregate-summary --no-controls` (placebo) | 77.5 ms | 76.3 ms | −0.89% [−6.55%, +3.08%] |
| `node-modules-dense`, `default-tree` | 86.0 ms | 79.5 ms | **−8.94% [−12.29%, −5.04%]** |
| `node-modules-dense`, `default-tree --no-controls` | 85.4 ms | 81.8 ms | **−4.05% [−9.38%, −1.69%]** |
| `node-modules-dense`, `aggregate-summary` (placebo) | 75.9 ms | 76.9 ms | +0.62% [−3.22%, +2.73%] |
| `node-modules-dense`, `aggregate-summary --no-controls` (placebo) | 77.5 ms | 77.9 ms | −0.77% [−2.48%, +4.49%] |

The primary artifact is the `linux-v6.12` probe cell.
The stacked default tree is faster on both real trees: on `linux-v6.12` −3.05%, the sum
of H185 and H186, each not resolvable alone, within the −2% to −5% expected, with the
`--no-controls` replicate at −4.74%; on `node-modules-dense` −8.94%, within the −6% to
−10% expected, with the replicate at −4.05%. The two comparisons on the dense tree
bracket the stacked effect as they bracketed H186’s alone (exp-199): both intervals lie
below −1.5%, and the larger, controls-on figure is the one the default command takes.
The default summary is −6.14% on `linux-v6.12` (H188) and unchanged on the dense tree,
which has no rules; the `--no-controls` summary, which no change touches, is at zero on
both. The stacked summary’s qualification block reads `inferior` on context switches for
the reason exp-198 records: its voluntary and involuntary switches rose from 144 and 36
to 310 and 153 (medians of 20) as the consumer stopped being the longest thread, against
about 760 and 620 on the blind arms, and the switch gate was not pre-registered for
H188.

**The standing, product command lines.** Each competitor paired with the shipped head
(`fdu-h186`, the `fdu` product binary at `a356d456`) in one interleaved run per tree, 20
pairs, 3 warm-ups; positive means the competitor took longer than the adjacent fdu run.
pdu 0.24.0 (`--silent-errors`, and `--max-depth 2` for the row named `pdu`), diskus
0.9.0, and the final head’s own product binary as a fourth competitor.

| Subject | Shipped `fdu` | `fdu` final head | pdu default | pdu `--max-depth 2` | diskus |
| --- | ---: | ---: | ---: | ---: | ---: |
| `linux-v6.12` | 0.084 s | 0.088 s, +4% [+2%, +7%] | 0.094 s, **+13% [+10%, +15%]** | 0.088 s, **+3% [+1%, +8%]** | 0.094 s, **+12% [+8%, +16%]** |
| `node-modules-dense` | 0.079 s | 0.087 s, +12% [+7%, +13%] | 0.092 s, **+15% [+13%, +20%]** | 0.086 s, **+10% [+4%, +13%]** | 0.090 s, **+11% [+10%, +15%]** |

**What the track adds and what the regime adds.** In this run, on `linux-v6.12`, fdu’s
default leads pdu default by 13%, diskus by 12% and pdu `--max-depth 2` by 3%; the
release head alone led pdu default by about 6% in the same run, against level in exp-194
(+1% [−2%, +2%]), so the track adds about 4% and the rest is the regime.
The release head’s own change against each peer is derived from the artifact: the
harness pairs every competitor with the anchor, not with each other, but two
competitors’ timed samples at the same ordinal ran within one rotation of the anchor of
each other, so they are paired at equal ordinals as the harness pairs variants, with its
own deterministic bootstrap of the median.
On `linux-v6.12` the final head against pdu default is +6.2% [+4.4%, +10.3%] (pdu
slower), against pdu `--max-depth 2` −1.5% [−2.8%, +1.9%], level as in exp-194, and
against diskus +5.7% [+1.0%, +7.9%]; normalizing each sample by its adjacent anchor run
first gives +9.6% [+4.0%, +12.6%], +0.7% [−6.1%, +4.2%] and +6.5% [+2.3%, +10.3%]. The
lead over the depth-2 mode, +3% [+1%, +8%], is therefore the track’s alone.
On `node-modules-dense`, in this run, fdu’s default leads pdu default by 15%, pdu
`--max-depth 2` by 10% and diskus by 11%; the release head alone led pdu default by
about 6% (+6.2% [+2.7%, +9.1%]) in the same run, against level in exp-194, was level
with diskus (+1.2% [−0.8%, +5.8%], as in exp-194), and was 2.4% behind pdu
`--max-depth 2` (−2.4% [−4.7%, −0.0%]) where exp-194 had it 7% behind; the track adds
+12% [+7%, +13%] and the rest is the regime.
Anchor-normalized: +5.6% [+0.8%, +12.0%], −1.0% [−6.5%, +4.0%] and +1.0% [−1.9%, +2.9%].
The regime’s part is specific to pdu’s default mode, about six points on both trees,
while diskus is where exp-194 left it; the method and its output are kept beside the
evidence as `derived-pairs.txt`.

**Answers.** The shipped head’s product command line matched the final head’s byte for
byte in the 54-comparison answer diff at each layer (exp-197 to exp-199), and the tool
comparison found 0 semantic mismatches and 0 summary-oracle mismatches on both trees.

## Decision

Baseline: this confirms the track’s accepted changes in one paired cell per tree against
the round’s final head, and no decision rests on it.
The shipped default command is 3.1% faster on `linux-v6.12` and 8.9% faster on
`node-modules-dense` than the final head’s, and the default summary 6.1% faster on
`linux-v6.12`. In this run fdu’s default leads pdu default by 13% and 15%, diskus by 12%
and 11%, and pdu `--max-depth 2` by 3% [+1%, +8%] and 10% [+4%, +13%], on the kernel and
the dense tree; about six points of the lead over pdu’s default mode are tonight’s
regime (the release head alone led it by 6% where exp-194 had them level), and the
track’s own contribution is the release-head row, +4% [+2%, +7%] and +12% [+7%, +13%].
On this host, fdu’s default command is ahead of every pdu mode measured on both real
trees; on the kernel tree the lead over the depth-2 mode is at the edge of what 20 pairs
resolve.
