---
type: is
id: is-01kztk38w55wpkx0gra5p1510g
title: Move producer paths into cold-scan observations
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/guides/performance-loop.md
labels: []
dependencies: []
created_at: 2026-08-12T08:58:18.880Z
updated_at: 2026-08-12T09:01:26.192Z
closed_at: 2026-08-12T09:01:26.189Z
close_reason: "Experiment exp-016 completed and was rejected: no wall/CPU improvement, with about 4% RSS and minor-fault regressions. Candidate reverted and result recorded."
---
Test H48: the portable walker currently clones every relative PathBuf into an Upsert even though non-directory entries can transfer ownership. Move paths into observations, cloning only directory paths retained by the frontier; measure cold-scan-producer and cold-scan-index with the paired real-tree loop, record the result, and retain only if it clears the project bar.
