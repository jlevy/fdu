---
type: is
id: is-01m1x444e4rnksjs8v8p37padv
title: Refresh the formal PR stack before final parity validation
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies:
  - type: blocks
    target: is-01m1dtr903vj783j9ajaxfnczf
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-07T05:05:45.411Z
updated_at: 2026-09-07T05:06:31.067Z
---
Inspect live remote heads and formal gh stack53, preserve other work, incorporate the latest PR48 parent fix into PR50 and descendants with safe stack-aware rebase/restack, use lease-guarded pushes for rewritten owned refs, and wait for CI on every pushed stack member. Final parity must use the integrated final branch identity.
