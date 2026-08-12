---
type: is
id: is-01kzvbc3wa6yvdtv12smd9x3j4
title: Standardize repository Markdown formatting on pinned flowmark-rs
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzq1vhvfdyrrhmz3343qh5nr
created_at: 2026-08-12T16:02:34.505Z
updated_at: 2026-08-12T16:15:17.543Z
closed_at: 2026-08-12T16:15:17.542Z
close_reason: Pinned flowmark-rs 0.3.2 in benchmarks/uv.lock, standardized formatting on flowmark --auto ., enforced it in CI, and formatted every supported repository file.
---
Pin the latest native Rust flowmark-rs in the committed benchmark/tooling lockfile. Make repository formatting use flowmark --auto over the whole repository, remove the custom per-file option set, and enforce the same normal form in CI.
