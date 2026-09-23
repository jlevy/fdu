---
type: is
id: is-01m360rycaafcmq5rgkatcvmjm
title: Name type-rule mismatch in cache-only snapshot refusal
kind: bug
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels: []
dependencies: []
parent_id: is-01m2pmrbxvnerxyhnjhrswy1ye
created_at: 2026-09-23T02:16:04.745Z
updated_at: 2026-09-23T02:16:04.745Z
---
Cache design Known Gaps: a valid snapshot under another TypeRegistry currently reaches ParseError::Invalid before LoadOutcome::Refused, so cache-only says absent. Preserve full structural/checksum validation and root identity, refuse rather than serving foreign rules, and name the type-rule mismatch with a concrete recovery. Regression: write under a custom registry and read cache-only under the compiled registry.
