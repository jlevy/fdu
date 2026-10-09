---
type: is
id: is-01m4gk0vyq21d943eskery7ne5
title: Record each perf variant's source commit and dirty flag automatically
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-10-09T15:03:04.918Z
updated_at: 2026-10-09T15:03:04.918Z
---
Review C4 on #191 (https://github.com/jlevy/fdu/pull/191#issuecomment-6082950211): exp-209 to exp-211 recorded a candidate commit in prose only, and exp-210/211's binary was built 44 minutes before its named commit existed; variants.*.notes were empty, so nothing tied a sha256 to source. exp-212 tied its two binaries to commits by hand through the NAME=PATH:NOTES variant syntax of benchmarks.realtree measure. Make it automatic: when perf-compare builds the candidate (perf-probe-release), write the HEAD commit and a dirty-tree flag into the candidate variant's notes (and refuse or warn on a dirty tree), and have perf-record copy a variant's commit into method.candidate/control so the record cannot name a commit the binary was not built from. Files: Makefile perf-compare, explorations/benchmarks/realtree/__main__.py (_variant), measure.py (Variant.notes), record.py.
