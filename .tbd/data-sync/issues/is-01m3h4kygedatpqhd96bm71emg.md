---
type: is
id: is-01m3h4kygedatpqhd96bm71emg
title: "Tool comparison: measure small tools' peak RSS on Linux without the harness floor"
kind: task
status: open
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - linux
dependencies: []
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-09-27T09:54:54.094Z
updated_at: 2026-09-27T09:55:09.337Z
---
On Linux, wait4 ru_maxrss inherits the launching process's high-water mark across execve, so every tool spawned by the ~57 MiB Python harness reports at least that. Since 2026-09-27 the harness withholds a peak at or below its own floor and renders it as a bound (docs/project/reports/report-2026-09-27-fdu-linux-tool-comparison.md), and that report's small-tool peaks came from a separate GNU time pass. Spawn timed children through a minimal launcher, as GNU time does, so the harness measures them directly. The dut adapter is fdu-k5t5.
