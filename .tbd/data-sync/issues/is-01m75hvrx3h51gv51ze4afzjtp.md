---
type: is
id: is-01m75hvrx3h51gv51ze4afzjtp
title: "Watch verification mounts an expired indirect-map key again: guard its stat with the mount table"
kind: bug
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
hold: null
hold_until: null
created_at: 2026-11-11T14:58:16.098Z
updated_at: 2026-11-11T15:59:02.074Z
started_at: 2026-11-11T14:58:40.801Z
---
From review H3 on PR #209 (https://github.com/jlevy/fdu/pull/209#issuecomment-6792308952). Medium. Exists on main; not introduced by #209.

Problem: a running watch over an indirect autofs map (the usual way /home and /net are
automounted) mounts a key again the moment its mount expires, in any view. When a key
expires, automount(8) removes the key's directory; the backend event names it, and the
watch verifies the name with crates/fdu-core/src/watch.rs verify_paths (around :1137)
through scan::observe_path (crates/fdu-core/src/scan.rs, "observe_path" /
"observe_path_entering"). That stat passes AT_NO_AUTOMOUNT, which covers only a path's
last component and only where it exists: the kernel passes a lookup of a missing name in
an indirect map to the automounter, a stat included (follow_automount in fs/namei.c).

Evidence (the reviewer's probe, Linux, root): fdu DIR --watch --view tree over an
indirect map with default (non-browsable) keys; the expiry alone made 1 mount request and
the key was mounted again; strace showed the verification's statx(..., AT_NO_AUTOMOUNT)
waiting on it. A direct trigger and a browsable key made 0. So a watch holds every key it
found mounted for as long as it runs, and a server that does not answer holds the
verification for as long as the automounter waits.

Fix direction: #209 added crates/fdu-core/src/capacity/mount_table.rs (MountTable: Linux
/proc/self/mountinfo, macOS getfsstat MNT_NOWAIT, read without resolving any path), which
query_volumes uses to decide which paths a capacity call may touch. Verification needs the
same guard before its stat: a path below a mount point whose topmost filesystem is autofs
(an indirect map) and whose name is not itself listed as a mount should be treated as
absent without being looked up, or verified through its parent's listing. Reuse the
reviewer's indirect_watch_probe.py shape for a root-only test in
crates/fdu-core/tests/volumes_automount.rs (its harness now has indirect maps:
Trigger::mount_indirect, mount_key, expire_key). Done: the watch makes 0 requests on expiry.
