---
type: is
id: is-01m48y1944fcj5vknqbkc6ptkc
title: "Index: time v0.1.0+ with --no-gitignore to separate .gitignore cost from speed"
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-10-06T15:41:37.275Z
updated_at: 2026-10-06T15:41:37.275Z
---
The unified score rises from 1.19x (exp101) to 2.47x (v0.1.0) slower than v0.3.0 because v0.1.0 reads .gitignore by default and earlier builds do not, so builds before and after 0.1.0 do different work on the Linux source tree (358 .gitignore files). Scale, on a tree with no .gitignore, is flat across the boundary. Add history cells timing v0.1.0 and later with --no-gitignore against the pre-0.1.0 builds, as a like-for-like line, so the chart separates the feature's cost from speed changes. Needs a versioned revision of index-suite.json (new or redefined components; digest refusal applies). Measure on the internal SSD, per the performance loop; about an hour of machine time.
