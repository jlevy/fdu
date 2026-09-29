---
type: is
id: is-01m3n9n06twrrj78fndn1mt2zv
title: "Code analysis: show or exclude generated, vendored and minified files; add a minified heuristic"
kind: feature
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-29T00:39:49.210Z
updated_at: 2026-09-29T00:39:49.210Z
---
fdu flags generated/vendored files but never excludes them from the code view; scc has --gen/--no-gen, -z minified, --no-large; cloc has --no-autogen; linguist honours .gitattributes. One-line minified files also cost memory proportional to file size. From the SLOC survey (fdu-61ez).
