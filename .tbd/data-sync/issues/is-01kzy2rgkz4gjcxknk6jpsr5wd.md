---
type: is
id: is-01kzy2rgkz4gjcxknk6jpsr5wd
title: "Summary plan: d_type-gated stat elision on the Linux backbone"
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - perf
  - linux
dependencies: []
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-08-13T17:29:46.878Z
updated_at: 2026-08-13T18:11:54.759Z
---
The transient summary needs no directory or symlink attributes: on Linux, getdents64 d_type identifies them without statx (DT_UNKNOWN falls back). Spike measured -1.4% wall [-2.4%, +1.9%] single-threaded on a 6.4%-directory tree - below the gate alone, but it composes with mimalloc (fdu-cckr) and scales with directory share; dir-heavy trees (monorepos, .git object fans) should see 2-4x the effect. Requires the planner to prove the tier (summary-only, cache-off, no dir-attr consumer) exactly as exp-040 does; one_filesystem still forces dir stats. Gate on the ledger protocol; pre-register produced-stat-call count (deterministic, strace-countable) as a mechanism check alongside wall.
