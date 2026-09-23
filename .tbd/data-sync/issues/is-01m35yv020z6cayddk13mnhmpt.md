---
type: is
id: is-01m35yv020z6cayddk13mnhmpt
title: Hash per-analyzer outcomes in content performance digest
kind: bug
status: in_progress
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-23T01:42:14.847Z
updated_at: 2026-09-23T01:43:52.592Z
---
Metric-layer review found that content-summary-v2 retained the old flat coverage inputs. Equal numeric totals with different code or words outcomes produce the same correctness digest. Include requested units and complete per-unit coverage, with regression proving empty analyzed code differs from unsupported code and unrequested code. Parent owns final gate and publication.

## Notes

Implemented in isolated commit 85848059 atop ba3e9564. Digest hashes stable profile bits and all per-unit coverage outcomes (including unsupported encoding); empty-file Rust/Haskell regression establishes equal numeric metrics and prior flat coverage but differing code outcomes, with unrequested profile checked too. cargo fmt and diff check pass; builds intentionally deferred to parent full gate. Independent parent review pending.
