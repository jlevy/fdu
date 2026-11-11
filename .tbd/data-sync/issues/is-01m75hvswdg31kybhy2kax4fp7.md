---
type: is
id: is-01m75hvswdg31kybhy2kax4fp7
title: Read a directory's attributes where its own listing opens it (browsable autofs keys; one open per crossing)
kind: bug
status: open
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
hold: null
hold_until: null
created_at: 2026-11-11T14:58:17.101Z
updated_at: 2026-11-11T15:59:03.859Z
started_at: 2026-11-11T14:58:39.058Z
---
From review H4 (remainder) and H8 on PR #209 (https://github.com/jlevy/fdu/pull/209#issuecomment-6792308952). Low / suggestion.

What #209 did: scan::entered_directory_attrs (crates/fdu-core/src/scan.rs) re-reads a
directory the walk descends into from a descriptor it opens, in the PARENT's listing loop,
where the stat put it on another device than its parent or flagged it a kernel automount
point (STATX_ATTR_AUTOMOUNT, now carried out of scan/linux_dents.rs). That fixed G4 and
the kernel-automount part of H4 (tracefs under debugfs, NFS nested exports): two cold
walks agree.

Remaining (H4): a browsable key of an indirect autofs map is a directory on its parent's
(the map's) device and its stat carries no flag (autofs marks the dentry, not the inode),
so it is recorded as the stat found it; the listing then mounts it and the mounted
filesystem's top-level files are counted on the map's row. Two cold runs disagree
(reviewer's h1_repro.py with "browse": first run auto 1 file, auto/alice/sub1 2 files and
"also entered at 1 other path"; second run auto 0, auto/alice 3 files). Volumes no longer
mounts anything because of it (#209's mount-table gate), but the rows differ.

Remaining (H8): the re-read opens the directory a second time in the parent's listing and
closes it; the directory's own listing opens another later. Effects: a directory holding
several unmounted triggers has them mounted one after another by the worker listing it,
where their own listings would mount them in parallel; and an opened root's discovery that
stops at its file budget, or is closed, may have mounted triggers it never lists (from
reading, not a run).

Fix direction: record a directory's device (and the other attributes the G4 rule re-reads)
from the descriptor its own listing opens, at the moment it is listed (linux_dents::Reader
opens it; fstat or statx AT_EMPTY_PATH there), instead of in the parent's loop, or, for the
autofs part alone, fstatfs the entered descriptor and treat every descended child of an
autofs directory as crossed. Deferred from #209 because amending an already emitted
directory entry from its own listing touches every listing route (serial and concurrent
walks, the detached builder, revalidation, serial and wave reconciliation, opened
discovery) and their batching and commit ordering. Done when h1_repro.py browse gives the
same rows on two cold runs and the parent's loop opens nothing.
