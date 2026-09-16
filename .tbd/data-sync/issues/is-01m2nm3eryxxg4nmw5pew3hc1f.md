---
type: is
id: is-01m2nm3eryxxg4nmw5pew3hc1f
title: "PR #69 review 69-3: four guides end with the old trailer instead of the standard doc footer"
kind: bug
status: closed
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2nm36wpq4crkj85j40kmnn0
created_at: 2026-09-16T17:26:46.813Z
updated_at: 2026-09-16T17:37:31.214Z
closed_at: 2026-09-16T17:37:31.213Z
close_reason: "e3ac5fd (PR #69): the standard common-doc-guidelines footer replaces the old trailer in integration-runbook, performance-loop, performance-loop-runbook and platform-tuning."
resolution: null
duplicate_of: null
---
docs/project/guides/integration-runbook.md, performance-loop.md, performance-loop-runbook.md and platform-tuning.md (file tails @fe5cb70, PR #69; pre-existing at 16efcd0) end with the older "Part of the fdu project documentation. See AGENTS.md." trailer rather than the common-doc-guidelines footer. Replace it.
