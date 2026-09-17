---
type: is
id: is-01m2qygtb7z3mehzrgzwz3mtx3
title: "Windows revalidation misses a same-size rewrite that keeps mtime: no change time in the validity fingerprint"
kind: bug
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - release
  - cache
dependencies: []
parent_id: is-01m2pmram44dgp78vm6xq4w7k7
created_at: 2026-09-17T15:07:19.250Z
updated_at: 2026-09-17T15:07:19.250Z
---
Found by the path-independence full matrix on windows-latest (PR #79): after a warmer, a same-size rewrite that keeps the file's mtime (mutation samesize_keepmtime) is not detected under --cache auto and read-only, so the warm answer serves the old content's metrics (for example rust code_lines 8 where cold gives 3). The per-item validity fingerprint leaves ctime zero on Windows (crates/fdu-core/src/scan.rs, 'Windows has no ctime in the Unix sense'), so size and mtime are the only change signals. NTFS does record a change time (ChangeTime in FILE_BASIC_INFO); Rust exposes std::os::windows::fs::MetadataExt::change_time, which is None from DirEntry::metadata and needs an opened handle. Decide: read the change time (at a per-file open cost, measured with make perf-compare) or document the limitation. Registry class windows-change-time in tests/path_independence/known-violations.toml clears with this bead.
