---
type: is
id: is-01m74c9ghthtt4gggx5r9bbqct
title: "PR #209 F15: measure macOS change-time recency share of a home folder"
kind: bug
status: open
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-11T04:01:40.410Z
updated_at: 2026-11-11T07:56:20.255Z
---
Review F15 on jlevy/fdu#209 (https://github.com/jlevy/fdu/pull/209#issuecomment-6788495799): on macOS, measure how much of a home folder is recent by change time. Desktop metadata writers that set extended attributes, new hard links, and tools that restore access times through utimes move a status change, beside the chmod, chown, relabels, and sync clients the docs name. Run fdu ~ --view ages under --time changed and --time modified on a real macOS home folder and record what share of ~ reads under 1d by each clock; that decides how useful the default clock is there. Manual, needs a macOS host; part of the PR's remaining manual macOS checks.

Linux hard-link measurement (review G3 on #209, https://github.com/jlevy/fdu/pull/209#issuecomment-6789444811): also measure the same share on Linux in a tree that holds a hard-linking package store. A link count is inode metadata, so making or removing any link to a file moves the change time of every other name. The reviewer measured with uv 0.12.1 on ext4 (uv hard-links from its cache on Linux): after installing idna and certifi into environment A and taking T=now, 'fdu A --kind file --since @T --view summary' gave 0 files; after installing the same packages into an unrelated environment B, 632 KiB in 26 files (each file's link count rose from 2 to 3); 0 files under --time modified; 26 files again after 'rm -r B'. Repeat on a real developer machine with several uv or pnpm projects sharing one store, running fdu ~/projects --view ages under both clocks, and record the share under 1d by each, so the default-clock decision has a Linux number beside the macOS one. The design question itself is fdu-yl8u.
