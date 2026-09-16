---
type: is
id: is-01m2kfrqzr7avk32202w0ye298
title: "PR #63 review PR63-PERF-1: warm revalidate of an over-budget tree re-upserts refused files and clones the table per batch"
kind: task
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m2kfr5a9x4sksa32yq00fh8k
created_at: 2026-09-15T21:32:32.629Z
updated_at: 2026-09-16T06:08:38.716Z
closed_at: 2026-09-16T06:08:38.713Z
close_reason: "581557c added the control counters and the inert-batch skip, so a warm revalidate of a refused tree no longer clones the table per batch; b989aa9 then restores the cold no-controls lane's cost with a vacant-table fast path. Soundness verified op by op by PR #63 delta review 5218970886; its follow-ups are fdu-88m9 and fdu-jpxt."
resolution: null
duplicate_of: null
---
PR #63 review PR63-PERF-1 (PLAUSIBLE) at 1fd71a9: scan.rs:4844-4854, index.rs:3706. Add control counters (control_reads, control_refused, control_sources_shared) under fdu_core::counters; make a cheap clearly-correct fix with a test if one exists, else file a stack-followup bead.
