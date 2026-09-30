---
type: is
id: is-01m0pqk1zqhx9tbhjz436n4pse
title: The rendered report is withheld until the snapshot write and index teardown finish
kind: bug
status: closed
priority: 2
version: 7
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-08-23T07:15:34.518Z
updated_at: 2026-09-30T03:51:07.900Z
closed_at: 2026-09-30T03:51:07.899Z
close_reason: "Withholding was not deliberate and is already gone; what remains is deliberate. Part 1 (flush the rendered report before joining the snapshot writer) landed as exp-068/H101; cli.rs run() writes and flushes the report, then joins pending_save. Part 2 (index teardown off the user-visible path) landed as exp-160: lib.rs release_index moves the last drop of an index of 64k+ entries to a detached thread that nothing joins, so a process that exits first lets the OS reclaim it; the engine architecture's one-shot serving section documents it. The join of the snapshot writer before run() returns is deliberate for correctness: a snapshot lost at exit costs the next run the cold scan this one already did, and the comment at cli.rs 'Joined before returning, and before the render error is raised' says so; the report has already reached the terminal by then, so it is not withheld. Part 3, whether a checksummed, atomically renamed, fail-closed cache file needs F_FULLFSYNC, is a durability policy for a person and is now fdu-30c5. No latency change was made, so no exp cell is needed; the golden corpus covers that the report is complete and identical."
resolution: null
duplicate_of: null
---
Found in the PR #38 senior review (https://github.com/jlevy/fdu/pull/38#issuecomment-5384769585), deferred out of that PR for the same reason as its sibling: it is a latency change on a path no ledger job measures.

crates/fdu/src/cli.rs:598 writes the rendered report into an 8 KiB BufWriter (cli.rs:1358); cli.rs:603 then joins the snapshot writer -- serialization over every entry, software CRC-32C, temp file, sync_all (F_FULLFSYNC on Apple), rename, a read_dir sweep of the cache directory, and finally the whole index teardown on that thread, since the main thread dropped its Arc at execution.rs:292. The buffer is flushed only at cli.rs:1423, after run() returns. A default depth-2 tree is under 8 KiB, so the user sees nothing until all of it completes, and the footer total counts it.

The comment at cli.rs:595-596 ("Whether output finishes first or the save does, both complete") states the intended overlap; the BufWriter defeats it.

Proposed fix, three separable parts: (1) out.flush() before pending_save.join(), which changes no output bytes; (2) decide whether the last Arc<Index> teardown belongs on the user-visible path at all (mem::forget or ManuallyDrop on the one-shot path, or exit after flush); (3) decide deliberately whether a checksummed, atomically renamed, corrupt-equals-empty cache file needs F_FULLFSYNC -- fdatasync semantics or none is defensible for a cache, and the file already fails closed on a torn write.

Acceptance: measured on the default-path job; part (1) needs no accept-rule verdict since it changes no work, only when the bytes reach the terminal, but it should still be recorded.

## Notes

Part 1 landed (exp-068, H101): flush before join; TTFB -7.54% repeated / -12.47% first run on rustup-toolchains, wall unchanged. Parts 2 and 3 remain: a repeated run still spends ~41 ms after its last byte (encode + compare + index teardown on the writer thread); that is their upper bound on this subject. Tier 3: durability policy, needs a person.
