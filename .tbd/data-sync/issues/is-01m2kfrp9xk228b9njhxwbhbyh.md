---
type: is
id: is-01m2kfrp9xk228b9njhxwbhbyh
title: "PR #63 review PR63-SNAP-1: loader accepts refusal records under an unbounded limit; note renders a 0 B budget"
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m2kfr5a9x4sksa32yq00fh8k
created_at: 2026-09-15T21:32:30.908Z
updated_at: 2026-09-16T06:08:27.598Z
closed_at: 2026-09-16T06:08:27.597Z
close_reason: "e9796df: the loader fails closed on a refusal recorded under a limit no save could have refused by, and the note names a limit without a size rather than a 0 B one and raises only bounded limits. Verified fixed by PR #63 delta review 5218970886 at 9105768."
resolution: null
duplicate_of: null
---
PR #63 review PR63-SNAP-1 at 1fd71a9: snapshot.rs:731-740, query_report.rs:768. No scan writes refusals under an unbounded limit; reject such records on load and never render a 0 B limit.
