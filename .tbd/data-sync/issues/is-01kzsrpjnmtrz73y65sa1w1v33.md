---
type: is
id: is-01kzsrpjnmtrz73y65sa1w1v33
title: concurrent_atomic_writes_do_not_share_a_temporary_file flaked once under heavy load
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-12T01:16:59.955Z
updated_at: 2026-08-12T01:16:59.955Z
---
Observed once while a benchmark run saturated the machine; passed 4/4 on retry immediately after, and the full suite is green. Pre-existing test in snapshot.rs, untouched by the traversal work. The test spawns concurrent writers and asserts the surviving file holds exactly one writer's payload (all bytes identical), so a failure means either a genuine temp-file collision under contention or an over-strict assertion. Worth reproducing under deliberate load (e.g. run it in a loop with the machine busy) before deciding which. Not reproduced on CI.
