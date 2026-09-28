---
type: is
id: is-01m3jw3yeztmc9rmctbjbzyv4s
title: Make Python checks rebuild stale editable native extensions
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-28T02:04:50.006Z
updated_at: 2026-09-28T02:04:50.006Z
---
During PR #136 integration into #137, make check reused an ignored python/fdu/_native.abi3.so from before the merge despite current Python wrappers expecting cache_dir. Ten Python tests failed with unexpected keyword argument; the Rust and CLI gates had passed. uv run --directory crates/fdu-py --frozen --group dev --reinstall-package fdu pytest forces a correct native rebuild. Investigate explicit editable-build cache keys or a deterministic native rebuild prerequisite so a reused local environment cannot test stale Rust code. CI fresh environments are not the reproducer. Keep all build outputs on task-specific external targets.
