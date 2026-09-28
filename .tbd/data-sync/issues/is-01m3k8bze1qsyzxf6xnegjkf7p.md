---
type: is
id: is-01m3k8bze1qsyzxf6xnegjkf7p
title: Resolve firmlinks in cache and checkpoint root identity
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T05:38:56.064Z
updated_at: 2026-09-28T05:38:56.064Z
---
Review finding: canonicalize() leaves /Users/... and /System/Volumes/Data/Users/... as two cache keys for one tree (lib.rs root hash). Use ATTR_CMNEXT_NOFIRMLINKPATH plus volume UUID. FSEvents device-relative filters must be nofirmlinkpath minus mount point; the System/Volumes/Data spelling matched nothing.
