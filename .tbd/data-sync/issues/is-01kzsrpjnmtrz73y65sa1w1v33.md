---
type: is
id: is-01kzsrpjnmtrz73y65sa1w1v33
title: concurrent_atomic_writes_do_not_share_a_temporary_file flaked once under heavy load
kind: bug
status: closed
priority: 3
version: 5
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-08-12T01:16:59.955Z
updated_at: 2026-09-30T04:06:20.316Z
closed_at: 2026-09-30T04:06:20.315Z
close_reason: "Not reproducible: snapshot::tests::concurrent_atomic_writes_do_not_share_a_temporary_file passed 200 of 200 runs in a loop over the test binary (--exact, --test-threads 1) while a cargo clippy/test build ran on the same host, on top of the 72 runs in the notes, and 4 of 4 in the full suite runs of this pass. The naming hardening (d9838d5) and the reaper (f97ea59) rule out the two collision mechanisms; the four assertions are individually diagnosable (the writer expect prints the io error and path, the leftovers assertion lists names). The remaining hypothesis is the notes' ENOSPC on a full volume, which is a host condition, not a race in the code or the test's synchronisation. Reopen with the panic text if it recurs."
resolution: null
duplicate_of: null
---
Observed once while a benchmark run saturated the machine; passed 4/4 on retry immediately after, and the full suite is green. Pre-existing test in snapshot.rs, untouched by the traversal work. The test spawns concurrent writers and asserts the surviving file holds exactly one writer's payload (all bytes identical), so a failure means either a genuine temp-file collision under contention or an over-strict assertion. Worth reproducing under deliberate load (e.g. run it in a loop with the machine busy) before deciding which. Not reproduced on CI.

## Notes

Naming inspected and hardened (d9838d5), litter closed (f97ea59). Answer to 'should killed-writer files ever collide': no, and they now cannot -- 64 bits of per-process RandomState entropy in .{file}.tmp.{pid}.{entropy}.{seq}, verified as 8 distinct values across 8 processes. But entropy alone made abandoned temporaries PERMANENT (no future process regenerates the name), so a reaper now removes temporaries older than 24h on each successful write, matched on the .{name}.tmp. prefix. Both reaper tests mutation-checked. REMAINING: the original flake is still unexplained -- not reproduced in 72 runs; leading hypothesis is ENOSPC (data volume at 100%, 3.6Gi free; test writes 8MiB with sync_all). If it recurs, capture the full panic to learn which of the four assertions fired.
