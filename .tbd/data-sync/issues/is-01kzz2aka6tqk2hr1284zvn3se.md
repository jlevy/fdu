---
type: is
id: is-01kzz2aka6tqk2hr1284zvn3se
title: Supply-chain audit never reads benchmarks/uv.lock
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01kzz29dspd7bsy6jk98mpb9z3
created_at: 2026-08-14T02:41:25.317Z
updated_at: 2026-08-14T02:41:25.317Z
---
scripts/check-supply-chain.mjs reads exactly one Python lockfile, crates/fdu-py/uv.lock. main also carries benchmarks/uv.lock with its own benchmarks/uv.toml cool-off policy, and AGENTS.md names 'the benchmark environment' as one of the two places exclude-newer-package exceptions are recorded.

So the whole benchmark Python dependency surface is outside the audit: no 14-day cool-off check, no registry check, no yanked-release check. make check reports supply-chain clean while auditing half the Python dependencies.

PR #4 hit the same gap when it added a second lock and generalized the reader to a list. Port that shape with main's actual second path.
