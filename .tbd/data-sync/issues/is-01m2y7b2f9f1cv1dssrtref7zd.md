---
type: is
id: is-01m2y7b2f9f1cv1dssrtref7zd
title: Directory roll-up queries for stale environments and build outputs (#93)
kind: epic
status: in_progress
priority: 1
version: 7
delegate: claude-code@spud10
labels: []
dependencies: []
child_order_hints:
  - is-01m2y7by0xkr9zre4es53fwjm0
  - is-01m2y7c1v3etw9wkzz87vgm6ke
  - is-01m2y7c6yzccahx1ntvy2wf1s1
  - is-01m2y7cbprn6w292gjenv0twd7
  - is-01m2y7cf9fsawdtq8p5grnr3nq
hold: null
hold_until: null
created_at: 2026-09-20T01:36:54.755Z
updated_at: 2026-09-20T01:38:30.392Z
started_at: 2026-09-20T01:38:30.390Z
---
Implement https://github.com/jlevy/fdu/issues/93 on a new branch based on PR #92 (perf/campaign-next-2026-09-19), then open a PR against that branch.

Problem: include/exclude select individual entries. Matching .venv currently yields its inode rather than subtree size; matching descendants yields many rows and applies age predicates to individual files. Users need one query over one retained scan to inventory stale environments and build outputs.

Design: add engine-owned ViewSpec::Directories / --view directories. Each matching directory is a whole roll-up root. Include/exclude, kind and ignored-state predicates select roots only; descendant names do not need to match, and excluding a descendant does not subtract it from a selected root. Existing views preserve per-entry selection. Size and modified bounds inspect subtree aggregates. Report all matching roots, including nested matches, without summing overlapping rows. This supports precise inventory; overlapping rows are explicitly documented as non-additive. No pruning at scan time, no cache-key changes, no content reads.

Recency: maximum observed mtime across the directory itself and all retained descendants (including directories and symlinks). This detects fresh installs and directory-entry changes, handles empty directories and pre-epoch timestamps, and does not claim access time or last use. Age is signed whole seconds from the request reference instant; future mtimes yield negative age. Both size metrics sum regular-file bytes with the existing engine semantics; directory inode and symlink bytes are excluded. Counts exclude the matched root. Hard links and shared extents retain existing accounting; size is not a promise of uniquely reclaimable disk space.

Defaults: every match, size descending, path tie-break. Existing sort/reverse/limit compose; bounds are explicit and liftable. Scan-depth constrains known contents and remains in provenance; display depth has the same non-effect on flat listings as files. Partial/cache-only results retain normal completeness and freshness labels. Text displays selected size, actual age and path. Machine rows expose path, apparent and allocated bytes, file/dir counts, newest_mtime_ns, age_seconds and root ignored classification. JSON, JSONL, YAML and Python agree. Full includes the additive view.

Implementation: pure iterative index reader; linear traversal to compute recency including directories (existing maintained newest only describes files), reuse existing subtree byte/count aggregates; do not change index reducers, ownership or snapshot format. Match native paths for one-shot and portable names for opened report projections. Expose typed DirectoryRow and Section::Directories from core, not CLI-only machinery.

Documentation inventory: README use cases; CLI short/long help examples; --docs usage guide; portable --skill; Rust/public API docs; Python README and stubs; machine-format schema guide; architecture view/selection semantics. Show .venv/venv, node_modules and Rust target (Cargo default build output) older than 7d/30d, age+size output, oldest-first sorting, JSON, ignored inventory and repeat queries over a retained index. Explicitly distinguish modification age from last use and byte totals from reclaimable unique space.

Acceptance: deterministic engine tests for nested/empty/fresh descendants, bounds, future/pre-epoch timestamps, ignored/exclude roots, all sizes, sorting/truncation, mixed views and portable names; real one-shot/warm/cache-only and Python parity coverage; reviewed portable goldens; make docs-format and make check; PR against #92 branch, push and CI green. No dependency changes, deletion commands or performance claims.
