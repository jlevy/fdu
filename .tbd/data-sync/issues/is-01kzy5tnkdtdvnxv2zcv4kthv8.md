---
type: is
id: is-01kzy5tnkdtdvnxv2zcv4kthv8
title: "PR #8 review observation: document one-shot adaptive calibration"
kind: task
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzy4tve6eej9e0jhxfqmqqmz
created_at: 2026-08-13T18:23:23.244Z
updated_at: 2026-08-13T18:23:39.535Z
closed_at: 2026-08-13T18:23:39.535Z
close_reason: "Fixed: the architecture white paper now states the intentional one-shot calibration boundary and its mixed-latency follow-up condition."
---
Senior review design observation: the fresh-scan service-time calibration intentionally decides once from the initial sample and does not re-evaluate a later slow subtree or mount. State this tradeoff in the architecture report.
