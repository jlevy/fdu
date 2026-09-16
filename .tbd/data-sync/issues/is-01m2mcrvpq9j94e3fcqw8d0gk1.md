---
type: is
id: is-01m2mcrvpq9j94e3fcqw8d0gk1
title: "PR #63 delta review PR63D-PERF-1: vacant-table cold lane pays a file_name parse and two BTreeMap probes per op"
kind: task
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2kfr5a9x4sksa32yq00fh8k
created_at: 2026-09-16T05:59:25.142Z
updated_at: 2026-09-16T06:19:21.310Z
closed_at: 2026-09-16T06:19:21.309Z
close_reason: "b989aa9: controls_unchanged_by answers from the op kind alone when the table is vacant (no ControlUpsert is inert against it, nothing structural can drop or prune), and a ControlRemove still asks remove_is_inert so a malformed control path stays non-inert. The populated lane tests the kind before parsing the path. a_batch_that_cannot_change_the_control_table_does_not_copy_it now pins the vacant lane. Unmeasured by design: the estimate is of order 1% of a cold scan, inside perf-compare noise."
resolution: null
duplicate_of: null
---
Delta review 5218970886 at 9105768: index.rs:3777-3795. controls_unchanged_by evaluates is_control_file(path) and has_record_at_or_below(path) for every non-directory Upsert, where the empty-table lane it replaced cost one discriminant check. Reviewer's fix: an exact is_vacant() fast path (on an empty table a ControlUpsert is never inert, and everything else is), plus reordering drops_control to test the kind before parsing the file name. Unmeasured either way.
