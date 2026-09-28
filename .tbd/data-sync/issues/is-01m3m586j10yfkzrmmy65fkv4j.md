---
type: is
id: is-01m3m586j10yfkzrmmy65fkv4j
title: "gitignore fidelity: honor a case-variant .gitignore on case-insensitive volumes, as git does"
kind: task
status: open
priority: 2
version: 4
labels: []
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
created_at: 2026-09-28T14:03:40.992Z
updated_at: 2026-09-28T20:31:50.804Z
---
fdu accepts a control file only when the listed name is exactly `.gitignore` (control::is_control_file). git opens `<dir>/.gitignore` by path, so on a case-insensitive volume (default macOS APFS, Windows, ext4 casefold) it honors `.GITIGNORE` too: verified 2026-09-28 on APFS, core.ignorecase=true, `git check-ignore -v a.log` reported `.gitignore:1:*.log` from a file named `.GITIGNORE`. So fdu's gitignored share differs from git's on such trees. Decide whether to match git (per-volume case sensitivity; every surface, both the index and the transient summary route from #149, goldens, a differential test). Found by the #149 review, where the transient route's mid-listing probe honored `.GITIGNORE` and the index did not; #149 is being made consistent with the index's exact-name rule first.

## Notes

Design sketch (2026-09-28; pressure-test before building):
- Rule: a listed entry is the directory's control if its name is exactly .gitignore, or if it matches .gitignore case-insensitively and <dir>/.gitignore resolves to the same file (same device and inode, or the Windows file index). Only case-variant names pay the extra stat.
- The transient summary probe's exact-name confirmation (lists_exact_control_name, #149 eeb257c9) then becomes unnecessary: the path lookup is the rule.
- Routes that must agree: the detached index builder; the transient summary fold and its mid-listing probe; the narrowed-population serial walk (--ignored=exclude|only); watch and opened roots (is_control_file on event paths, including a removal that can no longer be stat-ed); and the control table key (one canonical path, so a probe and a listing cannot double-count).
- Tests: route-against-route differential cases on a case-insensitive temp volume (detect at runtime, print a skip otherwise; GitHub's macOS and Windows runners provide one); a case-sensitive negative case with both .gitignore and .GITIGNORE; watch cases; ext4 casefold if a casefold directory can be made without root, else say so.
- Docs: output design, usage guide, gitignore-fidelity notes. The case against: repositories with .GITIGNORE already behave differently in git across platforms; put the design and that trade-off in the PR body.
