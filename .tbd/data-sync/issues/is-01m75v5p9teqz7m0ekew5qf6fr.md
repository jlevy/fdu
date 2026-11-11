---
type: is
id: is-01m75v5p9teqz7m0ekew5qf6fr
title: "PR #209 I1: a mount below a device boundary that is no mount point reads capacity unknown on a fresh walk"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m75v586ca3vv6vremva5h5er
hold: null
hold_until: null
created_at: 2026-11-11T17:40:58.298Z
updated_at: 2026-11-11T18:08:49.802Z
started_at: 2026-11-11T17:41:24.753Z
closed_at: 2026-11-11T18:08:49.802Z
close_reason: "Fixed in 8172907f: every listed entry is looked up in the mount table on its own; failed and NOT_REACHED removed. Tests: a_filesystem_no_longer_mounted_where_the_walk_entered_it_is_never_asked (updated) and a_mount_below_a_boundary_that_is_no_mount_point_is_asked (new), both fail before the fix. btrfs reproduction reads the tmpfs below the subvolume; covered-from-above probe makes 0 calls naming the covered mount; volumes_automount 7 passed, 0 skipped; review H probes 0 requests after expiry."
resolution: null
duplicate_of: null
---
Medium. crates/fdu-core/src/query/query_volumes.rs:365, :372-373, :386-388 (failed, NOT_REACHED); stated at docs/project/architecture/fdu-design-principles.md:456, docs/usage.md:496, CHANGELOG.md:114. An entry that fails the mount-table gate is added to failed, and no entry below it is looked up, so a tmpfs below a btrfs subvolume (and, by reading, every /Volumes disk under a macOS firmlink from /) reads unknown. Fix (reviewer's first option, coordinator decision): look every listed entry up in the table and drop failed and NOT_REACHED; safety holds from the table lookup itself (visible mount at exactly the path, topmost not autofs, stat confirms the recorded device). Review: https://github.com/jlevy/fdu/pull/209#issuecomment-6802796047 (PR #209).
