---
type: is
id: is-01kzsrpjnmtrz73y65sa1w1v33
title: concurrent_atomic_writes_do_not_share_a_temporary_file flaked once under heavy load
kind: bug
status: open
priority: 3
version: 3
labels: []
dependencies: []
created_at: 2026-08-12T01:16:59.955Z
updated_at: 2026-08-12T02:14:50.088Z
---
Observed once while a benchmark run saturated the machine; passed 4/4 on retry immediately after, and the full suite is green. Pre-existing test in snapshot.rs, untouched by the traversal work. The test spawns concurrent writers and asserts the surviving file holds exactly one writer's payload (all bytes identical), so a failure means either a genuine temp-file collision under contention or an over-strict assertion. Worth reproducing under deliberate load (e.g. run it in a loop with the machine busy) before deciding which. Not reproduced on CI.

## Notes

Naming inspected and hardened (d9838d5), litter closed (f97ea59). Answer to 'should killed-writer files ever collide': no, and they now cannot -- 64 bits of per-process RandomState entropy in .{file}.tmp.{pid}.{entropy}.{seq}, verified as 8 distinct values across 8 processes. But entropy alone made abandoned temporaries PERMANENT (no future process regenerates the name), so a reaper now removes temporaries older than 24h on each successful write, matched on the .{name}.tmp. prefix. Both reaper tests mutation-checked. REMAINING: the original flake is still unexplained -- not reproduced in 72 runs; leading hypothesis is ENOSPC (data volume at 100%, 3.6Gi free; test writes 8MiB with sync_all). If it recurs, capture the full panic to learn which of the four assertions fired.
