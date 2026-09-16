---
type: is
id: is-01m2ngxskwbmrm04xsw56p5v1f
title: Bring --docs, the skill, and in-code docs up to date with main at 16efcd0 (doc-drift audit package 5)
kind: task
status: closed
priority: 3
version: 3
labels:
  - docs
dependencies: []
created_at: 2026-09-16T16:31:15.579Z
updated_at: 2026-09-16T16:55:27.328Z
closed_at: 2026-09-16T16:55:27.327Z
close_reason: "PR #70 (9d4a586, e6ec879, 9a35f5b): all 12 package-5 findings fixed in comments, docstrings, --docs, and SKILL.md; cli-surface golden and parity artifact edited by hand; make check and CI green. The docs view alias is unintended and filed as fdu-ohah."
resolution: null
duplicate_of: null
---
The read-only doc-drift audit at 16efcd0 found 12 findings (6 P2, 6 P3) in work package 5: crates/fdu/src/cli.rs (--docs Scope row), crates/fdu/src/skills/SKILL.md (Scope row), crates/fdu-core/src/execution.rs (prepare_report doc: transient tier needs read_controls off; refusals by either limit), crates/fdu-core/src/snapshot.rs (FORMAT_VERSION 4 writes both limits), crates/fdu-core/src/cache.rs (CacheState::Leftover covers staging files and orphaned sidecars), crates/fdu-core/examples/perf_probe.rs (aggregate tier is fdu --no-gitignore --view summary), crates/fdu-py/src/lib.rs (report_once control_line_limit; open read_controls snapshot scope; render_cache_status scope), crates/fdu-py/python/fdu/_models.py (IgnoredTally both limits; Report.notes carries the refused-.gitignore note). Comments and help text only; the --docs and --skill goldens and the parity artifact change with the text.
