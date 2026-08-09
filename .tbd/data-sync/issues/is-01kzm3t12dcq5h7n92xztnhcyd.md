---
type: is
id: is-01kzm3t12dcq5h7n92xztnhcyd
title: "Close PR #1 merge gate and publish final senior approval"
kind: task
status: in_progress
priority: 0
version: 17
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels:
  - merge-blocker
  - pr-review
dependencies:
  - type: blocks
    target: is-01kzky7wjz44trprn1ck52pd58
  - type: blocks
    target: is-01kzky86nqp91wq9d3wj2psnwr
  - type: blocks
    target: is-01kzky8bctckj3kk8gwntbg8tn
  - type: blocks
    target: is-01kzkzm62q1vwxbv9hbp39bxxm
  - type: blocks
    target: is-01kzkzmrjbr2ew8wt774r1n26x
  - type: blocks
    target: is-01kzg48zxv9jrjbrfswztx2q36
  - type: blocks
    target: is-01kzg4908hkkpt0pf20602rze8
  - type: blocks
    target: is-01kzg4c75tvbrg6rgh3803nwzj
  - type: blocks
    target: is-01kzg4akhzmh7xgcabnnyc4e9f
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-09T20:35:40.748Z
updated_at: 2026-08-09T22:26:58.915Z
---
Close PR #1 only after the independent supply-chain blocker and the concurrency validation gate are complete. The concurrency gate transitively requires atomic malformed-batch rejection, the guard-free shared-index API, filesystem-I/O-free watch arbitration, bounded fail-safe watcher transport/shutdown, and deterministic cross-thread evidence. Then run the complete local handoff gate, verify the full GitHub Linux/macOS/Windows/MSRV/docs/audit/Python matrix, confirm the branch is clean and tbd is synchronized, update the PR description, and publish a final senior-review comment that supersedes the current hold. The final comment must state the tested ownership, lock, overload, shutdown, snapshot visibility, and Python GIL contracts and identify any explicitly deferred performance-only work. Do not close this bead merely because GitHub reports the branch mergeable.

## Notes

All implementation blockers, including final thread-aware findings fdu-l8vc/fdu-83gl/fdu-ie5z and the live-provenance quota fix fdu-b3qe, are closed. The complete local make check passes: 10 supply-chain tests plus live verification of 66 Cargo/31 npm/2 Python packages/21 action uses/bootstrap pins; fmt and Clippy; 145 all-feature library + 2 CLI + 1 integration + 2 doctests; 25 golden scenarios; rustdoc; 105 core-only + 135 watch-only tests; exact Rust 1.85.0 all-feature compile and 105 core tests; Cargo/npm audits; 2 Python concurrency tests; installed abi3 wheel smoke. tbd doctor and sync are clean. Remaining: commit/push, inspect fresh PR threads, require every new GitHub check, update the PR description, publish the superseding senior approval, then close this bead.
