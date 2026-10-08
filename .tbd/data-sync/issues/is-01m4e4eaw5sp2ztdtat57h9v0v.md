---
type: is
id: is-01m4e4eaw5sp2ztdtat57h9v0v
title: "Placebo cell: time v0.3.0 against itself on the macOS index to explain the exp201 -> 0.3.0 step"
kind: task
status: open
priority: 2
version: 1
labels:
  - performance
  - macos
dependencies: []
created_at: 2026-10-08T16:09:48.676Z
updated_at: 2026-10-08T16:09:48.676Z
---
Review A on #184 (What Next): in the committed macOS history cells, the pdu-track milestone exp201 (a356d456) is 10.0% [4.2, 12.4] slower than v0.3.0 on default-tree and scores 1.0546 [1.0409, 1.0684] on the full index, though the release added no change expected to speed the walk (exp-202's record: expected to cost nothing). v0.3.0 is the anchor in every cell, so an anchor/ordering effect is possible. Run one cell with v0.3.0 against a byte-identical copy of itself (same driver, same rounds) before the quiet re-time (fdu-bkj2); if the placebo shows a similar offset, the index's adjacent-to-anchor steps need a correction.
