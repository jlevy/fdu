# fdu Performance Evidence Report

**No performance claim is established by this report.** It summarizes one validated raw
result set; release claims require the complete dedicated-host matrix and review gates
in the active performance plan.

## Run Identity

- Evidence status: reproducible input record (no claim implied)
- Run id: `77777777777777777777777777777777`
- Started: `2026-08-09T12:00:00.000000Z`
- Finished: `2026-08-09T12:00:01.000000Z`
- Source revision: `8888888888888888888888888888888888888888`
- Result schema: `fdu-performance-result-v1`
- Result hash: `35333b5dfa0a13f02447b79dd40f882a9da1b90abbea2e1bb7bfdb18cf7365de`
- Host class: `fixture-host-v1`
- Platform: `FixtureOS aarch64`
- Kernel: `fixture-kernel`
- Logical CPUs: `8`
- Memory bytes: `16000000000`
- Filesystem: `apfs`
- Harness identity: `7556981e77baff4b7a9943ebf3cdcdccdbbc4681647d90fa1dd98f7364ef69d3`
- Python: `cpython 3.13.5`

## Executables

| Adapter | Identity SHA-256 | Components |
| --- | --- | --- |
| fixture | `bf98b979025d9381a734b3e6bd6d24a279ac74c0bf528f7fbc7841929a47efc6` | fdu@111111111111 |

## Scenario Statistics

Only valid timed samples contribute. Warmups and invalid trials remain in the raw-trial
table below.

| Scenario | Valid | Invalid | Median | MAD | P95 | Min | Max | Mean | Stddev | CV | Entries/s |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fixture/contract/baseline | 3 | 1 | 20.000 ms | 10.000 ms | n/a (requires at least 20 valid trials) | 10.000 ms | 30.000 ms | 20.000 ms | 8.165 ms | 40.82% | 5,050.0 |

## Review Triggers

- fixture/contract/baseline: 1 timed trial(s) were invalid.
- fixture/contract/baseline: coefficient of variation is 40.82%, above the 10%
  investigation threshold.
- fixture/contract/baseline: 3 valid timed trial(s) are below the release-headline
  minimum of 10.

## Raw Trials

| Ordinal | Kind | Scenario | Snapshot | FS cache | Valid | First output | Wall | Exit | Stdout bytes | Stdout SHA-256 | Reason |
| ---: | --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | --- | --- |
| 0 | timed | fixture/contract/baseline | absent | uncontrolled | yes | 5.000 ms | 10.000 ms | 0 | 100 | `6666666666666666666666666666666666666666666666666666666666666666` | — |
| 1 | timed | fixture/contract/baseline | absent | uncontrolled | yes | 10.000 ms | 20.000 ms | 0 | 100 | `6666666666666666666666666666666666666666666666666666666666666666` | — |
| 2 | timed | fixture/contract/baseline | absent | uncontrolled | yes | 15.000 ms | 30.000 ms | 0 | 100 | `6666666666666666666666666666666666666666666666666666666666666666` | — |
| 3 | timed | fixture/contract/baseline | absent | uncontrolled | no | 500.000 ms | 1000.000 ms | 0 | 100 | `6666666666666666666666666666666666666666666666666666666666666666` | synthetic invalid sample |

## Reproduction Contract

- Invocation ordering: `sha256-round-robin-v1`
- Order seed: `synthetic-order`
- Snapshot state and filesystem-cache state are recorded per raw trial.
- Exact commands use tokenized argument arrays; personal absolute paths are not
  persisted.
- Invalid samples are retained and are never included in the statistics.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
