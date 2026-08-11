# Cache Performance Across macOS and Linux

env-001 reruns the frozen PR 3 cache comparison on equivalent generated workloads in two
environments. It answers two different questions:

1. Does the candidate improve each benchmark job relative to the corrected control in
   every environment?
2. Is the candidate’s full warm cache path faster than its own cold path in either
   environment?

The first answer is environment-dependent at the CPU gate and incomplete at the Linux
RSS gate. The second is no for latency in both cells, although warm revalidation uses
less CPU.

## Evidence contract

Both runs regenerated the same balanced corpus from seed
`pr3-cache-cross-environment-v1`. The matrix independently recomputed the portable
identity instead of trusting the recorded identifier.

| Field | Value |
| --- | --- |
| Workload | 60,001 entries: 7,501 directories and 52,500 files |
| Portable identity | `797ad1257352770608139731ac9a055f0ff6aa19b547bd77bf4cd289d3d777e7` |
| Semantic digest | `f7ed40e64310fee3aaa7ad3dcbc41e78c9737da6d4cdc00981ea2aa51e1d2049` |
| Corrected control | `c0ddcb9807dacd7190cfc6639104db8fc33be896` |
| Candidate | `bd479aaee90263dea8c7dc5ce9d131368e749568` |
| Schedule | 12 paired trials, 3 warmups, round-robin by ordinal |
| Cache condition | warm-steady OS page cache; unchanged generated tree |
| Correctness | zero invalid oracle samples in either cell |

The matrix also required matching probe-source hashes, build profiles, feature and
variant arguments, complete job contracts, schedule digest, and run group.
Target triples and binary hashes remain cell-local because the architectures differ.

| Cell | Host | Filesystem | Runner grade |
| --- | --- | --- | --- |
| `local-macos-apfs-arm64` | Apple M1 Pro, arm64, 10 logical cores, 32 GiB | APFS | local uncontrolled |
| `github-ubuntu-24.04-x64` | AMD EPYC 7763, x86-64, 2 logical cores, 7.8 GiB | ext4 | shared cloud exploratory |

The Linux artifact came from
[GitHub Actions run 31472156291](https://github.com/jlevy/fdu/actions/runs/31472156291).
The hosted workflow is manually dispatched and archives raw trials; reviewed evidence is
committed here so artifact retention is not part of the reproducibility contract.

## Candidate versus corrected control

Changes below are paired median changes.
Negative is better.
The decision requires a significant wall-time improvement of at least
3% and no statistically established CPU or RSS regression above 10%.

| Job | macOS: wall / CPU / RSS | macOS decision | Linux: wall / CPU / RSS | Linux decision |
| --- | ---: | --- | ---: | --- |
| `cold-scan-index` | -44.46% / +125.59% / +2.00% | rejected: CPU | -33.31% / +5.02% / n/a | rejected: RSS unmeasured |
| `cold-scan-producer` | -58.33% / +154.16% / +19.50% | rejected: CPU, RSS | -38.06% / +18.31% / n/a | rejected: CPU; RSS unmeasured |
| `cold-snapshot-save` | -41.61% / +123.94% / -0.58% | rejected: CPU | -31.66% / +4.37% / n/a | rejected: RSS unmeasured |
| `warm-revalidate` | -26.68% / -26.77% / -8.55% | accepted | -41.82% / -41.82% / n/a | rejected: RSS unmeasured |
| `warm-snapshot-load` | -25.89% / -26.15% / -6.31% | accepted | -29.25% / -29.28% / n/a | rejected: RSS unmeasured |

Cold-index CPU and cold-snapshot-save CPU pass on Linux but fail on macOS. The producer
CPU gate fails in both, and warm revalidation and snapshot load improve in wall and CPU
against the control in both environments.
No Linux job is accepted overall because its RSS guardrail is not measured.
The matrix therefore correctly reports `decision_consistent: false`; it never averages
absolute timings across hosts or turns a missing resource signal into a pass.

### Linux peak-RSS limitation

Every one of the Linux run’s 150 samples reported exactly 81,678,336 bytes of peak RSS,
including producer-only and full-index jobs and both implementations.
The corresponding macOS run produced 115 distinct values from 11.0 to 41.3 MiB. A
constant Linux value across structurally different processes is consistent with the
Python launcher’s pre-exec resident high-water mark dominating `wait4.ru_maxrss`; it
cannot distinguish the variants’ memory use.

The matrix detects this run-wide degeneracy, marks Linux RSS `not-measured`, and fails
the overall resource decision closed.
The wall and CPU comparisons remain valid because they vary per child and are collected
independently.
A future Linux claim needs a launcher-independent RSS collector before any
row can be accepted.

The strongest design consequence is that default parallelism creates a real latency/CPU
tradeoff whose CPU cost depends on the environment; the complete Linux resource verdict
remains open. A universal claim remains unsupported.
Thread count, core count, architecture, OS, filesystem, and runner control must be
explicit inputs to later policy evidence.

## Candidate warm path versus candidate cold path

This is a diagnostic comparison of within-cell medians, not an interleaved paired
acceptance test. It decides whether the existing cache read path should be preferred for
these unchanged 60k-entry cells.

| Cell | Cold index wall / CPU | Warm revalidate wall / CPU | Snapshot load wall / CPU |
| --- | ---: | ---: | ---: |
| macOS/APFS | 274.9 ms / 1,111.5 ms | 495.3 ms / 493.1 ms | 170.7 ms / 168.5 ms |
| hosted Linux/ext4 | 315.4 ms / 494.7 ms | 426.8 ms / 426.4 ms | 209.7 ms / 209.5 ms |

Full warm revalidation is about 80% slower in wall time than cold on macOS and 35%
slower on Linux. It uses about 56% less CPU on macOS and 14% less on Linux.
Snapshot load alone is faster than both full paths, so the full filesystem stat
sweep—not snapshot deserialization alone—prevents a one-shot warm latency win.

For a latency-oriented `auto` policy, these observations keep cold scanning as the
fallback in both cells.
They do not make persistence unnecessary.
A snapshot still supports explicit cache-only answers, journal resume, derived-data
reuse, and a long-lived index.
Incremental journal validation or a cheaper verification tier is the route to making the
persisted state a fast trustworthy read rather than an additional full-tree pass.

## Limits and next decision

This experiment establishes a cross-environment difference, not its cause.
The two cells confound core count, CPU architecture, operating system, filesystem,
memory, and runner control.
The Linux runner is shared and exploratory; stable product policy requires a controlled
cell.
Both runs use a generated balanced tree, warm-steady page caches, zero churn, local
storage, and completion latency.
They do not cover cold page caches, first-output latency, real-tree topology, larger
scales, churn, remote storage, Windows, XFS, btrfs, or overlayfs.

The next isolating experiment should run the same corpus at a fixed two-thread producer
count on controlled Mac and Linux hosts, then vary one axis at a time.
Until that is done, describe the finding as an environment-cell or default-concurrency
divergence, never as an APFS-versus-ext4 effect, and do not promote a platform selector
row.

## Durable artifacts

- [Decision matrix](../experiments/evidence/env-001-decision-matrix.json), SHA-256
  `74e2d473e4d8e64646793ac8bb20971f5da318c746060d72a4992d0fd015bac8`
- [macOS/APFS raw run](../experiments/evidence/env-001-macos-apfs-run.json), SHA-256
  `6c6c1c6896e3da3bdfbca048992fe06b2e56455ae7a3619de2298379ea41311d`
- [Linux/ext4 raw run](../experiments/evidence/env-001-linux-ext4-run.json), SHA-256
  `f8ab814dfb25a9e822f5a0f5b3517fc6aff8437b8ef9bda64f96c0a15900b9e7`

* * *

*Part of the fdu project documentation.
See [AGENTS.md](../../../AGENTS.md).*

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
