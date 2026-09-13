---
type: is
id: is-01m1dtqbkxhdqfhczrbdctcaxq
title: Canonicalize encoded public observation paths before mutation
kind: bug
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - correctness
  - paths
dependencies:
  - type: blocks
    target: is-01m1dtqkbyrydtq60902w1sgkr
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-01T06:32:53.116Z
updated_at: 2026-09-13T21:52:06.180Z
closed_at: 2026-09-01T08:06:51.371Z
close_reason: Canonical public observation paths are enforced at the ownership boundary with encoded-byte regressions; local and GitHub gates pass.
resolution: null
duplicate_of: null
---
Add encoded-byte or native-unit regressions for interior current-directory components and repeated separators, restore full normalization for public inputs, and reserve any canonical fast lane for a private scanner-owned type with tested invariants.

## Notes

Implemented red-green on codex/streaming-performance-parity. The encoded-output regression failed against the inherited fast lane with dotted/./file.txt retained verbatim. Public normalization now validates and rebuilds in one component pass into a pre-sized PathBuf, canonicalizing current-directory components and repeated native separators without a temporary component vector. Exact commit changes, compatibility operations, and dirty impact paths are checked by encoded bytes; existing root, escape, non-Unicode, and platform cases remain green. Minimal and all-feature fdu-core suites, isolated make check, and macOS/Windows make cross-lint pass. Commit b5d9ba4 is pushed; GitHub Actions run 33484597414 completed with all 19 jobs green.

2026-09-13: PR #51 review 5192254822 finding COMMIT-4 (Low) is this defect, raised against the PR that introduced the fast lane (bf21dd6). Ported down to PR #51 as 50e6ca5 (git cherry-pick -x 3bdfbb2); the code applied unchanged and #52's plan-document edits were left out. 6d2d964 adds the review's own reproduction -- a trailing separator (a/b/) and a trailing `.` -- in the UnknownAncestry error value and the committed change, which the ported test does not reach. Red on 19c0d73 (dotted/./file.txt retained); index suite green after the port; all 19 CI checks green at 6d2d964.
