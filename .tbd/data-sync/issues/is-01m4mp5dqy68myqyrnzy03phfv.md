---
type: is
id: is-01m4mp5dqy68myqyrnzy03phfv
title: "linux_dents tests: settle allocations with test_support::settle_allocations, not a system-wide sync"
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-10-11T05:14:57.661Z
updated_at: 2026-10-11T05:14:57.661Z
---
Review F on PR #191 (F1 Low, F2 suggestion; https://github.com/jlevy/fdu/pull/191#issuecomment-6105659434). F1: crates/fdu-core/src/scan/linux_dents.rs:1098-1102, the fdu-ekmb fix for stale_bytes_in_the_buffer_never_reach_a_listing, runs the coreutils 'sync' command; the crate already has test_support::settle_allocations(root) (fdu-tq70, test_support.rs:78-105) for this exact ext4 delayed-allocation race, and the same module calls it at :1640. Use it. F2: the sibling test entries_match_the_portable_metadata_contract_byte_for_byte (linux_dents.rs:1348) compares allocated bytes across listings of five freshly written files without settling them; settle there too. Deferred from the 0.5.0 stack to avoid a merge-up through #192/#194 for a test-only change.
