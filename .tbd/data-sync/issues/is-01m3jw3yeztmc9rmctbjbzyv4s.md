---
type: is
id: is-01m3jw3yeztmc9rmctbjbzyv4s
title: Make Python checks rebuild stale editable native extensions
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-28T02:04:50.006Z
updated_at: 2026-09-30T02:45:24.044Z
closed_at: 2026-09-30T02:45:24.044Z
close_reason: "Fixed in ba4a09a9 together with its duplicate fdu-ukg6: python-check's pytest line now runs uv with --reinstall-package fdu, so the editable native extension is rebuilt every run and cargo decides staleness (target-owner guards a shared target). tool.uv cache-keys were tried and rejected: uv reads them only from pyproject.toml and, with crates/fdu-py/uv.toml present, warns on every run that they are ignored (scratch-project experiment with uv 0.12.1; keys in uv.toml are not honoured at all). Recipe test in scripts/cargo-target.test.mjs pins the flag. The stale-to-current transition itself needs the coordinator's make check (a native build)."
resolution: null
duplicate_of: null
---
During PR #136 integration into #137, make check reused an ignored python/fdu/_native.abi3.so from before the merge despite current Python wrappers expecting cache_dir. Ten Python tests failed with unexpected keyword argument; the Rust and CLI gates had passed. uv run --directory crates/fdu-py --frozen --group dev --reinstall-package fdu pytest forces a correct native rebuild. Investigate explicit editable-build cache keys or a deterministic native rebuild prerequisite so a reused local environment cannot test stale Rust code. CI fresh environments are not the reproducer. Keep all build outputs on task-specific external targets.
