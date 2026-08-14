---
type: is
id: is-01kzzbrr0nx14vvdj39kebzn78
title: "PR #20 R2: remove invalid blocked_ns from parallel experiment results"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01kzzanfm0vcgrcdmjwr90rcja
created_at: 2026-08-14T05:26:26.068Z
updated_at: 2026-08-14T05:38:24.076Z
closed_at: 2026-08-14T05:38:24.075Z
close_reason: Removed 78 invalid blocked_ns metric blocks from historically parallel jobs, retained serial/cache-hit measurements, validated all 51 softschema artifacts, and regenerated the experiment ledger.
---
Audit exp-000 through exp-050 result jobs, remove blocked_ns only where the measured process was parallel, preserve valid serial/cache-hit metrics, validate every softschema artifact, and regenerate the experiment ledger.
