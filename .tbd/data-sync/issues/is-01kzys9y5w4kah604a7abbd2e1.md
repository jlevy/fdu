---
type: is
id: is-01kzys9y5w4kah604a7abbd2e1
title: A text-bearing corpus for content-analysis benchmarking
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T00:03:46.491Z
updated_at: 2026-08-14T00:03:46.491Z
---
Neither corpus generator emits analyzable text. corpus.py::_create_file writes repeated SHA-256 digest bytes; spikes/gen_tree.py::mkfile writes b'x' * size and sparse-truncates the rest. For the content tier these are actively misleading: sparse regions read as NUL and trip the binary gate, and an xxxx file is one enormous line, pathological for both the LOC partitioner and the prose collectors. Content experiments exp-047 through exp-050 consequently ran at 307 and 2001 entries against 60k/720k/1M for metadata.
