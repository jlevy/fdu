---
type: is
id: is-01m3fznvkp0zx3rhf2vg3wrsv5
title: Support narrower content measurement scope alongside broader metadata
kind: feature
status: closed
priority: 2
version: 2
spec_path: docs/project/research/research-2026-09-26-codebase-analysis.md
labels: []
dependencies: []
created_at: 2026-09-26T23:09:19.349Z
updated_at: 2026-09-26T23:11:16.485Z
closed_at: 2026-09-26T23:11:16.484Z
close_reason: "Superseded during design review: no new analyze-scope qualifier. Existing analyze and view controls compose with the single ignored-population option; mixed populations use ordinary separate requests. This closes the proposed extra interface, not an implemented capability."
resolution: null
duplicate_of: null
---
Implement the proposed analyze-scope=selected|non-ignored|ignored qualifier with a shared engine measurement-population model. Selected follows the main ignored population. Narrower scopes retain metadata reports for both admitted populations while requesting content metrics only for one. Reject scopes that name an excluded class or require disabled classification. Test all eight nonempty population/measurement combinations, unknown and unrequested coverage, ignore-rule changes, and warm/cold parity. Keep analyzer set and population together so separate scoped engine requests compose.
