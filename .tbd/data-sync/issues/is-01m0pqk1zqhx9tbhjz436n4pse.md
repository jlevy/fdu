---
type: is
id: is-01m0pqk1zqhx9tbhjz436n4pse
title: The rendered report is withheld until the snapshot write and index teardown finish
kind: bug
status: open
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - campaign-2
  - macos-agenda
dependencies: []
created_at: 2026-08-23T07:15:34.518Z
updated_at: 2026-08-23T10:06:23.285Z
---
Found in the PR #38 senior review (https://github.com/jlevy/fdu/pull/38#issuecomment-5384769585), deferred out of that PR for the same reason as its sibling: it is a latency change on a path no ledger job measures.

crates/fdu/src/cli.rs:598 writes the rendered report into an 8 KiB BufWriter (cli.rs:1358); cli.rs:603 then joins the snapshot writer -- serialization over every entry, software CRC-32C, temp file, sync_all (F_FULLFSYNC on Apple), rename, a read_dir sweep of the cache directory, and finally the whole index teardown on that thread, since the main thread dropped its Arc at execution.rs:292. The buffer is flushed only at cli.rs:1423, after run() returns. A default depth-2 tree is under 8 KiB, so the user sees nothing until all of it completes, and the footer total counts it.

The comment at cli.rs:595-596 ("Whether output finishes first or the save does, both complete") states the intended overlap; the BufWriter defeats it.

Proposed fix, three separable parts: (1) out.flush() before pending_save.join(), which changes no output bytes; (2) decide whether the last Arc<Index> teardown belongs on the user-visible path at all (mem::forget or ManuallyDrop on the one-shot path, or exit after flush); (3) decide deliberately whether a checksummed, atomically renamed, corrupt-equals-empty cache file needs F_FULLFSYNC -- fdatasync semantics or none is defensible for a cache, and the file already fails closed on a torn write.

Acceptance: measured on the default-path job; part (1) needs no accept-rule verdict since it changes no work, only when the bytes reach the terminal, but it should still be recorded.

## Notes

Part 1 landed (exp-068, H101): flush before join; TTFB -7.54% repeated / -12.47% first run on rustup-toolchains, wall unchanged. Parts 2 and 3 remain: a repeated run still spends ~41 ms after its last byte (encode + compare + index teardown on the writer thread); that is their upper bound on this subject. Tier 3: durability policy, needs a person.
