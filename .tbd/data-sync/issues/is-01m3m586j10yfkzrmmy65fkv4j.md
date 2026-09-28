---
type: is
id: is-01m3m586j10yfkzrmmy65fkv4j
title: "gitignore fidelity: honor a case-variant .gitignore on case-insensitive volumes, as git does"
kind: task
status: open
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
created_at: 2026-09-28T14:03:40.992Z
updated_at: 2026-09-28T16:17:24.914Z
---
fdu accepts a control file only when the listed name is exactly `.gitignore` (control::is_control_file). git opens `<dir>/.gitignore` by path, so on a case-insensitive volume (default macOS APFS, Windows, ext4 casefold) it honors `.GITIGNORE` too: verified 2026-09-28 on APFS, core.ignorecase=true, `git check-ignore -v a.log` reported `.gitignore:1:*.log` from a file named `.GITIGNORE`. So fdu's gitignored share differs from git's on such trees. Decide whether to match git (per-volume case sensitivity; every surface, both the index and the transient summary route from #149, goldens, a differential test). Found by the #149 review, where the transient route's mid-listing probe honored `.GITIGNORE` and the index did not; #149 is being made consistent with the index's exact-name rule first.

## Notes

Also pre-existing: the narrowed-population serial walk (--ignored=exclude|only) resolves its read_directory_control by path, so on a case-insensitive volume it honors .GITIGNORE while the index does not. #149's transient-summary probe was fixed in eeb257c9 to confirm the exact listed name (lists_exact_control_name) before reading; whichever rule is chosen here must apply to every route (index, transient summary probe, serial walk).
