---
type: is
id: is-01m2khchfcavrw5gcz7nyev3j3
title: "Default --view summary retains the full index to classify ignored entries: peak RSS 13 MiB to 128 MiB"
kind: task
status: open
priority: 1
version: 1
labels:
  - stack-followup
  - release
  - scale
dependencies: []
created_at: 2026-09-15T22:00:49.899Z
updated_at: 2026-09-15T22:00:49.899Z
---
Found by the PR B speed gate (branch claude/gitignore-default-on, d95d729), as decision Q7 on fdu-elnn anticipated: the transient summary tier keeps no control table, so with `.gitignore` observed by default an unfiltered `fdu --view summary PATH` falls closed to `RetainedState::FullIndex` (crates/fdu-core/src/execution.rs `plan_report`, `summary_is_sufficient` requires `!config.scan.read_controls`).

Measured 2026-09-15 on an M1 Pro, macOS/APFS, release builds of origin/main f047dab and the branch, `--cache off --color never --view summary`, 15 interleaved pairs, a loaded host (load average 20-28 from Spotlight and another agent):
- ~/.rustup/toolchains (70,143 files, 0 .gitignore): wall pair ratio 1.036 (95% CI 1.017-1.053); peak RSS 13.1 -> 27.9 MiB.
- metabrowser-release-perf checkout (548,251 files, 290 .gitignore): wall pair ratio 1.022 (CI 0.900-1.071); peak RSS 13.2 -> 127.9 MiB.

Wall time passes the 10% gate; memory does not have a gate, and this is the contract where fdu decisively beat dust. `--no-gitignore --view summary` keeps the 13 MiB tier.

Direction (plan section 5 and Q7): a streaming classifier in the summary reducer: controls emitted before their directory's entries, the ignore decision per entry from the governing sources, and a top-most-ignored-directory set so descendants of an ignored directory are counted without a table walk. It must report the same `SummaryRow.ignored` as the index tier, pinned by the existing compact-versus-indexed equality test extended to the share.

Acceptance: default `--view summary` peak RSS within the transient tier's order of magnitude on both subjects, identical totals and ignored share, recorded with `make perf-record`.
