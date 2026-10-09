---
type: is
id: is-01m4d4qejw29ggmcha47ycj9za
title: Record the 0.4.0 Linux demo from the release candidate and embed it in README and release notes
kind: task
status: open
priority: 1
version: 7
labels:
  - release
  - docs
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
created_at: 2026-10-08T06:55:32.944Z
updated_at: 2026-10-09T04:39:01.989Z
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

Final take (2026-10-09 21:31): from d0032b35 (#188, no documentation count), 140x48, --margin 0; tree 0.235 s, cold 10.538 s, cached 0.889 s under host load ~10-18 (three re-takes 10.4/9.6/12.8 s gave nothing better; 0.4.0's diagnostic median cold is 9.5 s). MP4 3,784,222 bytes sha256 31e720b35f96fdc060fa5a33bfa7f62856b3bd8e2160bf39b48724b256a604c5 (kept at ~/fdu-release/media/fdu-demo-0.4.0.mp4 and ~/Downloads/fdu-demo-0.4.0-final/); GIF 1,451,807 bytes, 1234x922, sha256 39a54b05f61af4ea1792c7879d65f0646f49620f00a83a5f4fea9df18a98de8b. Maintainer decision: MP4 release-only (declared by digest in docs/media/fdu-demo.json), GIF in repo.
