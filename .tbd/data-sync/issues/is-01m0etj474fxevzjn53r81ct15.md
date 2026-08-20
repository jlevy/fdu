---
type: is
id: is-01m0etj474fxevzjn53r81ct15
title: Watch text repaints run together with no boundary
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m0erhq35tpxzjecxn3p9jzx2
created_at: 2026-08-20T05:33:34.306Z
updated_at: 2026-08-20T06:20:51.015Z
closed_at: 2026-08-20T06:20:51.013Z
close_reason: "Fixed in d785300: each text watch repaint is preceded by a gray rule carrying the render instant. Drawn only for text (machine repaints are already framed by their own envelope) and only for repaints, so the opening report is unchanged. Goldened end to end via a new watch-repaint-capture helper; deleting the rule turns that golden red."
---
Found during the end-to-end round on macOS.

`fdu --watch --interval 1s --view tree,types,summary --color never TREE` in text format renders each repaint straight after the previous one, so the last block of repaint N and the first block of repaint N+1 are adjacent with no separator:

    4.0 KiB  1 file, 0 directories        <- end of repaint 1
    8.0 KiB  ██████████   100%  . (2 files)  <- start of repaint 2

The all-caps view headers added in fdu-dzhm make this much better for multi-view watch runs, since each repaint now begins with a visible TREE header. A single-view watch run still has no repaint boundary at all.

Worth deciding whether a watch repaint should be separated from the previous one (a blank line, or a timestamped rule). Watch has no performance footer, so there is no existing marker to lean on.
