---
type: is
id: is-01m32tww1snf1n9w7dnqkv98s5
title: "PR #98 review R2: locked/denied-file fallback std provided is gone, so fdu C:\\ drops pagefile and hiberfil"
kind: bug
status: closed
priority: 1
version: 3
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m32h6dpd97fr5f8db831dn3y
hold: null
hold_until: null
created_at: 2026-09-21T20:35:38.681Z
updated_at: 2026-09-21T20:57:25.677Z
started_at: 2026-09-21T20:36:00.243Z
closed_at: 2026-09-21T20:57:25.677Z
close_reason: "Fixed in 650b6b08: the handle opens with access 0 as std's metadata does, and on ERROR_SHARING_VIOLATION or ERROR_ACCESS_DENIED observation answers from the listing metadata the caller already holds (observe_dir_entry passes || entry.metadata(); observe/attrs_from pass the caller's Metadata) with kind, size and write time, and ctime/inode/dev zero. should_descend already treats dev == 0 as within-filesystem, so a fallback directory is not skipped. Tests: error-code classification, listing observation of a real file/dir, ordinary-file open with a fallback closure that panics if reached; passed in Windows CI. Not verified: an actual hiberfil.sys/System Volume Information scan on a Windows host."
resolution: null
duplicate_of: null
---
Blocker from https://github.com/jlevy/fdu/pull/98#issuecomment-5764966314. windows_metadata.rs:29-33 opens with FILE_READ_ATTRIBUTES and propagates any open error; std's metadata opens with access 0 and falls back to FindFirstFileExW on ERROR_SHARING_VIOLATION / ERROR_ACCESS_DENIED (hiberfil.sys, pagefile.sys, System Volume Information). Fix: open with access 0; on those two codes derive kind/size/mtime from the enumeration metadata already in hand (observe_dir_entry, scan.rs:5378-5394) with ctime/inode/dev zero, never an error.
