---
type: is
id: is-01m0pqk1nrzyrd883jtjqf06y7
title: The cold-scan path rewrites an identical snapshot on every run
kind: bug
status: open
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - campaign-2
  - macos-agenda
dependencies: []
created_at: 2026-08-23T07:15:34.199Z
updated_at: 2026-08-23T09:09:08.795Z
---
Found in the PR #38 senior review (https://github.com/jlevy/fdu/pull/38#issuecomment-5384769585), deferred out of that PR because it is an engine behaviour change with no measuring job.

plan_report routes every one-shot metadata query through the cold-scan path (crates/fdu-core/src/execution.rs:152-169), and cold_scan_save_targets returns SaveTargets::all() unconditionally (crates/fdu-core/src/lib.rs:479-500). The "reconciliation mutated nothing so write nothing" rule exists only on the warm-revalidate branch that the read gate no longer takes (lib.rs:370-380). So every default run over an unchanged tree re-serializes and re-writes the whole snapshot through F_FULLFSYNC.

Observed on ~/.rustup (175,190 entries, macOS, exploratory, loaded host): snapshot mtime advances on every run with size identical at 13,925,460 bytes; --cache auto 0.24-0.27 s vs --cache off 0.20 s warm, RSS +20 MiB. At the ~650k field-tree scale that is ~50 MB written per invocation.

Proposed fix: in snapshot::write_atomically (crates/fdu-core/src/snapshot.rs:766) compare the serialized bytes against the existing file before creating the temp file, and return early when identical. Serialization is deterministic (pre-order over name-sorted BTreeMap children), so byte equality is exact and the cache cannot go stale: the bytes are the same bytes. Distinct from fdu-hvs5, which asked whether to persist at all and was correctly rejected on APFS evidence; this keeps every persistence benefit.

Acceptance: a default-path ledger job (see the sibling bead) recording the before/after, plus a test that a second run over an unchanged tree leaves the snapshot mtime untouched.
