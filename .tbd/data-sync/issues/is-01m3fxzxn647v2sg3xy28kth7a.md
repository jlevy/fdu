---
type: is
id: is-01m3fxzxn647v2sg3xy28kth7a
title: Prune excluded ignored subtrees under the unified population policy
kind: feature
status: open
priority: 2
version: 2
spec_path: docs/project/research/research-2026-09-26-codebase-analysis.md
labels: []
dependencies: []
created_at: 2026-09-26T22:39:51.973Z
updated_at: 2026-09-26T23:09:18.570Z
---
Implement and verify traversal pruning for --ignored=exclude as part of the unified ignored-population design in fdu-gdg0. No separate scan-ignored flag. Follow effective ancestor and negation rules, do not prune unknown classifications, distinguish control reads from content reads, and mark pruned totals unavailable. Exercise rule changes and all-to-excluded/excluded-to-all cache transitions with enumeration counters. The unified exclusion option must not ship as a report-only filter.
