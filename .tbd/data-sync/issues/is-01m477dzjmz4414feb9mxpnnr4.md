---
type: is
id: is-01m477dzjmz4414feb9mxpnnr4
title: "Linux demo: intro comment, cached analysis, shorter pause, no wrapping"
kind: task
status: closed
priority: 2
version: 4
labels: []
dependencies: []
parent_id: is-01m477dy519nmp2vvdjtg0n3z1
created_at: 2026-10-05T23:47:21.812Z
updated_at: 2026-10-06T06:26:46.584Z
closed_at: 2026-10-06T06:26:46.583Z
close_reason: "Video regenerated: three commented commands, cached run shown, scrolling, recorded from the internal SSD"
resolution: null
duplicate_of: null
---
packages/cli-animate/examples/fdu/linux.yaml: add a one-line # comment naming the entire Linux source and its size/file count; build fdu's analysis cache in setup so the second command is served from cache; shorten the hold before the second command; widen to 200 columns (font 15) so no line wraps; record from a source tree without .git (the pack file row is noise). Regenerate the video after the output changes land and replace the old one.

## Notes

Maintainer, 2026-10-05 (supersedes the earlier step list): three commands run from inside the Linux source (cwd = the tree, so the root is '.'), each preceded by its # comment exactly:
  # Full tally of usage                                            -> fdu .
  # Full code and document analysis (first run, no cache, slowest)  -> fdu . --view code,documents   (hidden before: fdu . --cache-clear)
  # Full code and document analysis (subsequent runs, fast)         -> fdu . --view code,documents   (served from the cache the previous step built)
Keep the earlier one-line top comment naming the entire Linux source with its size and file count. Fit each output to the terminal (200 cols, 15 px; add --limit only if needed); short holds; no .git in the tree. Delete the old videos, regenerate, show the new one.
