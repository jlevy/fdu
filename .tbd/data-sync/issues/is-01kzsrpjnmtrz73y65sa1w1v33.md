---
type: is
id: is-01kzsrpjnmtrz73y65sa1w1v33
title: concurrent_atomic_writes_do_not_share_a_temporary_file flaked once under heavy load
kind: bug
status: open
priority: 3
version: 2
labels: []
dependencies: []
created_at: 2026-08-12T01:16:59.955Z
updated_at: 2026-08-12T01:59:48.521Z
---
Observed once while a benchmark run saturated the machine; passed 4/4 on retry immediately after, and the full suite is green. Pre-existing test in snapshot.rs, untouched by the traversal work. The test spawns concurrent writers and asserts the surviving file holds exactly one writer's payload (all bytes identical), so a failure means either a genuine temp-file collision under contention or an over-strict assertion. Worth reproducing under deliberate load (e.g. run it in a loop with the machine busy) before deciding which. Not reproduced on CI.

## Notes

Inspected the atomic-write path (jlevy asked specifically about randomness). Temp names had ZERO bits: .{file}.tmp.{pid}.{sequence}. That is correct and cannot explain the flake -- the AtomicU64 counter makes two threads of one process unable to collide, and O_CREAT|O_EXCL is what guarantees a single creator, so the name only affects retry probability. The failing test uses 8 threads of ONE process, so a name collision was never possible there. Two real (non-flake) gaps found and fixed in d9838d5: stale temps from killed writers are never reaped, so a recycled pid collides with the corpse every run; and fully predictable names let an attacker pre-create the whole MAX_TEMP_CREATE_ATTEMPTS budget in a hostile directory (denial only -- O_EXCL still blocks symlink attacks). Now mixes 64 bits of per-process RandomState entropy. FLAKE ITSELF UNEXPLAINED: not reproduced in 72 runs (60 release + 12 full-suite debug) under CPU and I/O load. Most likely environmental: the data volume is at 100% capacity with 3.6Gi free, the test writes 8MiB with sync_all, and ENOSPC surfaces through the same expect(). Next step if it recurs: capture the full panic message (the original grep only caught the summary line, so which of the four assertions fired is unknown).
