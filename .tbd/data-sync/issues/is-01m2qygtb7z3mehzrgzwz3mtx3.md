---
type: is
id: is-01m2qygtb7z3mehzrgzwz3mtx3
title: "Windows revalidation misses a same-size rewrite that keeps mtime: no change time in the validity fingerprint"
kind: bug
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - release
  - cache
dependencies:
  - type: blocks
    target: is-01m2yh8kc79nw7bn6k6xw8g3bp
parent_id: is-01m2pmram44dgp78vm6xq4w7k7
hold: null
hold_until: null
created_at: 2026-09-17T15:07:19.250Z
updated_at: 2026-09-20T05:30:29.046Z
started_at: 2026-09-20T05:10:48.766Z
---
Found by the path-independence full matrix on windows-latest (PR #79): after a warmer, a same-size rewrite that keeps the file's mtime (mutation samesize_keepmtime) is not detected under --cache auto and read-only, so the warm answer serves the old content's metrics (for example rust code_lines 8 where cold gives 3). The per-item validity fingerprint leaves ctime zero on Windows (crates/fdu-core/src/scan.rs, 'Windows has no ctime in the Unix sense'), so size and mtime are the only change signals. NTFS does record a change time (ChangeTime in FILE_BASIC_INFO); Rust exposes std::os::windows::fs::MetadataExt::change_time, which is None from DirEntry::metadata and needs an opened handle. Decide: read the change time (at a per-file open cost, measured with make perf-compare) or document the limitation. Registry class windows-change-time in tests/path_independence/known-violations.toml clears with this bead.

## Notes

2026-09-20 review at 937f9445: still reproducible by existing Windows full-matrix registry. Rust MetadataExt change_time, file_index and volume_serial_number remain nightly-only (official std docs), so the Rust 1.85 floor rules out calling them. A fix needs an audited Windows platform boundary using an already locked windows-sys release (0.61.2 is present), or another shared validity mechanism; handle metadata must agree across scan, watch, analysis before/after reads, retained reuse and sidecars. Microsoft FILE_BASIC_INFO/GetFileInformationByHandleEx document ChangeTime. Source checkout prepared at /private/tmp/fdu-correctness-windows (ephemeral review workspace; not a published documentation path). Cross-lint/MSRV and Windows real execution plus measured cost remain required; do not clear known violation without evidence.
