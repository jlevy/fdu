---
type: is
id: is-01m2khchfcavrw5gcz7nyev3j3
title: "Default --view summary retains the full index to classify ignored entries: peak RSS 13 MiB to 128 MiB"
kind: task
status: open
priority: 1
version: 4
labels:
  - stack-followup
  - scale
dependencies: []
created_at: 2026-09-15T22:00:49.899Z
updated_at: 2026-09-17T02:10:33.442Z
---
Found by the PR B speed gate (branch claude/gitignore-default-on, d95d729), as decision Q7 on fdu-elnn anticipated: the transient summary tier keeps no control table, so with `.gitignore` observed by default an unfiltered `fdu --view summary PATH` falls closed to `RetainedState::FullIndex` (crates/fdu-core/src/execution.rs `plan_report`, `summary_is_sufficient` requires `!config.scan.read_controls`).

Measured 2026-09-15 on an M1 Pro, macOS/APFS, release builds of origin/main f047dab and the branch, `--cache off --color never --view summary`, 15 interleaved pairs, a loaded host (load average 20-28 from Spotlight and another agent):
- ~/.rustup/toolchains (70,143 files, 0 .gitignore): wall pair ratio 1.036 (95% CI 1.017-1.053); peak RSS 13.1 -> 27.9 MiB.
- metabrowser-release-perf checkout (548,251 files, 290 .gitignore): wall pair ratio 1.022 (CI 0.900-1.071); peak RSS 13.2 -> 127.9 MiB.

Wall time passes the 10% gate; memory does not have a gate, and this is the contract where fdu decisively beat dust. `--no-gitignore --view summary` keeps the 13 MiB tier.

Direction (plan section 5 and Q7): a streaming classifier in the summary reducer: controls emitted before their directory's entries, the ignore decision per entry from the governing sources, and a top-most-ignored-directory set so descendants of an ignored directory are counted without a table walk. It must report the same `SummaryRow.ignored` as the index tier, pinned by the existing compact-versus-indexed equality test extended to the share.

Acceptance: default `--view summary` peak RSS within the transient tier's order of magnitude on both subjects, identical totals and ignored share, recorded with `make perf-record`.

## Notes

2026-09-15, review of PR #65. Verdict on whether this must be fixed before 0.1.0: no. Wall time passed the gate on both subjects (1.02-1.04); the RSS is the index's, which the default `fdu PATH` has always paid on the same tree, so the summary view loses a special standing rather than regressing below the default; 0.1.0 is the first release, so nobody holds a summary-RSS baseline; and `--no-gitignore --view summary` keeps the aggregate-only path, documented in README, --help, SKILL and the ledger. At a few hundred bytes per entry a multi-million-entry home directory costs the summary several hundred MiB: a resource cost with no correctness hazard and no new crash class.

One thing to decide here rather than later, raised by the same review: restoring the transient reducer with a streaming classifier would flip `--cache only --view summary` a second time. PR B made the default summary retain the full index, which is also what lets it save a snapshot; a later streaming classifier would stop it saving one again, so the same command changes behaviour twice across releases. Either accept that flip and note it in the CHANGELOG when it lands, or keep the snapshot write on the streaming path so only memory changes.

2026-09-16, MEASUREMENT: quote a range and the mechanism, never a point. The figure this bead was filed with, "13 MiB to 128 MiB", is one sample of a number that moves. Four paired runs of `fdu --cache off --color never --view summary` on the same control-rich checkout, across the PR's own heads, measured the branch's peak RSS at 128, 101, 68 and 68 MiB, against 13.2, 13.0, 12.4 and 13.9 MiB for the same command before the change. The base is stable because the aggregate-only tier retains nothing per entry; the branch's is not, because it is the index's, and an index's peak depends on how the allocator grew its arenas for that tree on that run.

So the release note and any other text should say: the default `--view summary` now retains the index, so its peak RSS is the index's rather than the reducer's -- on the order of 10 MiB before and 70 to 130 MiB after on a 300k-entry control-rich checkout, scaling with retained entries rather than with a constant. `--no-gitignore --view summary` keeps the aggregate-only tier and its roughly 13 MiB. Do not quote a single multiplier: the same command on the same tree gave between 5x and 10x across four runs.
