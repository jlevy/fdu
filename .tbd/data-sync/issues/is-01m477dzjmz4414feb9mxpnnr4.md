---
type: is
id: is-01m477dzjmz4414feb9mxpnnr4
title: "Linux demo: intro comment, cached analysis, shorter pause, no wrapping"
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m477dy519nmp2vvdjtg0n3z1
created_at: 2026-10-05T23:47:21.812Z
updated_at: 2026-10-05T23:47:21.812Z
---
packages/cli-animate/examples/fdu/linux.yaml: add a one-line # comment naming the entire Linux source and its size/file count; build fdu's analysis cache in setup so the second command is served from cache; shorten the hold before the second command; widen to 200 columns (font 15) so no line wraps; record from a source tree without .git (the pack file row is noise). Regenerate the video after the output changes land and replace the old one.
