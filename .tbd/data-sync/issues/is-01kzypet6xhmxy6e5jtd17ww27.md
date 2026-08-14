---
type: is
id: is-01kzypet6xhmxy6e5jtd17ww27
title: Bound content-analysis candidate scheduling instead of materializing every path
kind: bug
status: open
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
created_at: 2026-08-13T23:14:00.541Z
updated_at: 2026-08-14T00:04:20.143Z
---
Index::analysis_candidates clones every regular-file path into a Vec before work enters the bounded worker channel. Reader concurrency is bounded, but scheduling memory remains O(files). Stream candidates through bounded scheduling while preserving conditional apply semantics.
