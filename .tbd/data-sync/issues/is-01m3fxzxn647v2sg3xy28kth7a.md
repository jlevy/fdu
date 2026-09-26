---
type: is
id: is-01m3fxzxn647v2sg3xy28kth7a
title: Add explicit ignored-path traversal scope with unavailable omitted totals
kind: feature
status: open
priority: 2
version: 1
spec_path: docs/project/research/research-2026-09-26-codebase-analysis.md
labels: []
dependencies: []
created_at: 2026-09-26T22:39:51.973Z
updated_at: 2026-09-26T22:39:51.973Z
---
Implement the later --scan-ignored=include|exclude proposal after selected-content analysis. Exclude prunes effective ignored paths and cannot claim descendant file/byte/content totals. Resolve omitted ignored selection to exclude; reject explicit include/only or no-gitignore conflicts. Keep admission and report ignore rules identical; record scope in cache identity, support correct rule-change invalidation, and test with directory-open counters. Owner permits replacing alpha interfaces without compatibility aliases.
