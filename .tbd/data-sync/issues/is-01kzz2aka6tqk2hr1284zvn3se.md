---
type: is
id: is-01kzz2aka6tqk2hr1284zvn3se
title: Supply-chain audit never reads benchmarks/uv.lock
kind: bug
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01kzz29dspd7bsy6jk98mpb9z3
created_at: 2026-08-14T02:41:25.317Z
updated_at: 2026-08-14T02:54:28.666Z
closed_at: 2026-08-14T02:54:28.666Z
close_reason: Generalized check-supply-chain.mjs to a UV_LOCKS list covering crates/fdu-py/uv.lock and benchmarks/uv.lock. The audit now verifies 16 Python packages instead of the fdu-py subset. Reading the benchmark lock for the first time surfaced softschema 0.6.0 and frontmatter-format 0.4.0 inside the 14-day cool-off; both are first-party and both are recorded as time-bounded exceptions expiring 2026-08-19 and 2026-08-16. The durable policy question is fdu-3vum.
---
scripts/check-supply-chain.mjs reads exactly one Python lockfile, crates/fdu-py/uv.lock. main also carries benchmarks/uv.lock with its own benchmarks/uv.toml cool-off policy, and AGENTS.md names 'the benchmark environment' as one of the two places exclude-newer-package exceptions are recorded.

So the whole benchmark Python dependency surface is outside the audit: no 14-day cool-off check, no registry check, no yanked-release check. make check reports supply-chain clean while auditing half the Python dependencies.

PR #4 hit the same gap when it added a second lock and generalized the reader to a list. Port that shape with main's actual second path.
