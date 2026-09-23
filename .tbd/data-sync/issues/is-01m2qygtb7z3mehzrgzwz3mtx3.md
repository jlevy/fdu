---
type: is
id: is-01m2qygtb7z3mehzrgzwz3mtx3
title: "Windows revalidation misses a same-size rewrite that keeps mtime: no change time in the validity fingerprint"
kind: bug
status: closed
priority: 1
version: 10
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
updated_at: 2026-09-23T08:14:06.898Z
started_at: 2026-09-20T05:10:48.766Z
closed_at: 2026-09-23T08:14:06.898Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Found by the path-independence full matrix on windows-latest (PR #79): after a warmer, a same-size rewrite that keeps the file's mtime (mutation samesize_keepmtime) is not detected under --cache auto and read-only, so the warm answer serves the old content's metrics (for example rust code_lines 8 where cold gives 3). The per-item validity fingerprint leaves ctime zero on Windows (crates/fdu-core/src/scan.rs, 'Windows has no ctime in the Unix sense'), so size and mtime are the only change signals. NTFS does record a change time (ChangeTime in FILE_BASIC_INFO); Rust exposes std::os::windows::fs::MetadataExt::change_time, which is None from DirEntry::metadata and needs an opened handle. Decide: read the change time (at a per-file open cost, measured with make perf-compare) or document the limitation. Registry class windows-change-time in tests/path_independence/known-violations.toml clears with this bead.

## Notes

Draft PR #98 at fe06b11b is under native CI. Root found an introduced Unix allocation regression: five scanner DirEntry call sites eagerly build item.path() although Unix metadata observation does not need a path. Linux detached allocation guard caught 14863 allocations / 2080 added entries against ceiling 14560. Assigned platform-gated lazy path observation fix; do not raise the guard. Native Windows rewrite/replacement evidence and combined fallible-observation cleanup still pending.

Revision 3accd7be removes duplicate Windows metadata observation and unnecessary Unix path allocation. Native Windows CI passes the new same-size/preserved-mtime rewrite and path-identity replacement tests, the directory allocation guard, and 721 core tests. Linux/macOS tests pass too. The remaining Windows job failure is the expected four registered same-size mutation cases now conforming. Full Linux/macOS/Windows path-independence run 35493483402 is in progress to regenerate the registry from all judged artifacts. Keep open until that evidence is reviewed and integrated.

Full three-platform matrix run35493483402 at3accd7be recorded15939cases each on macOS/Linux and14049onWindows. All20formerwindows-change-time cases now conform; registry regenerated from all3judged artifacts, retiring only those20entries and the now-empty class. No new difference classifications. Full local makecheck running before evidence commit.

2026-09-20 final PR98 performance-harness failure was an oracle drift, not an engine regression: the independent Python engine digest still zeroed Windows ctime/inode/dev after the engine began using ChangeTime/file index/volume serial. Commit 8c69d830 adds an independent non-following Win32 handle observer (double-query, fail closed), uses it for corpus and real-tree digests, and adds native same-size/restored-mtime plus reparse no-follow tests. Local corpus/realtree suites pass 119 tests with the two Windows-native checks queued for CI; PR98 branch pushed for validation.
