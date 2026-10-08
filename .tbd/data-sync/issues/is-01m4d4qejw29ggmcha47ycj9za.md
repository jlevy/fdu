---
type: is
id: is-01m4d4qejw29ggmcha47ycj9za
title: Record the 0.4.0 Linux demo from the release candidate and embed it in README and release notes
kind: task
status: open
priority: 1
version: 2
labels:
  - release
  - docs
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
created_at: 2026-10-08T06:55:32.944Z
updated_at: 2026-10-08T07:01:53.360Z
---
Maintainer decision 2026-10-07: the 0.4.0 demo video is GitHub-hosted and lands before the tag.

1. Build the release candidate from the release-prep PR head (tree-identical to the release commit).
2. Record packages/cli-animate/examples/fdu/linux.yaml with `cli-animate make` (already in 0.4.0 spelling: `fdu . --view code,documents`); update its intro size/file count to the clone. Tree and binary on the internal SSD.
3. Decide fdu-nlgx (block-glyph seams in the bar charts) first: fix, or accept for this take.
4. Upload the MP4 once through GitHub's web editor (no API) to get a user-attachments URL.
5. Embed the URL in README.md and docs/project/release-notes/0.4.0.md in the release-prep PR, so the release page (body derived from the notes; `release-announced` requires body == notes.md) shows it.

Why not a release asset: scripts/release/maintainer.py asset_check fails on any file beyond the eleven the workflow attaches, and the README could only link a download. Why not after publishing: editing the release body breaks the announced check.

Blocked on disk space: ~1.8-3.8 GiB free on the internal SSD; needs a release build plus a 2 GiB kernel clone.

## Notes

Existing take (2026-10-05 23:37 PDT, this Mac): ~/Downloads/fdu-video/linux.{mp4,gif,cast,receipt.json}, copies plus the lossless master in /Volumes/spud-ext1/agent-scratch/fdu-video/. Recorded from the pre-merge fdu-view-implies worktree (now gone), 36 s, 3004x1972, verify ok; tree 0.183 s, cold analysis 7.3 s, cached 0.76 s. It is stale for 0.4.0: it predates 6fa83ad1 (2026-10-06 01:33), so it prints 'note: percentages are shares of document words' where main prints 'shares of code lines (CODE), document words (DOCUMENTS)'. Re-record from the release candidate. The kernel tree is already cloned at /Volumes/spud-ext1/agent-scratch/demo-trees/linux (1.7 GiB, USB drive); linux.yaml asks for the internal SSD.
