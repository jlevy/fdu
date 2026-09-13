---
type: is
id: is-01m2e9nqdh5kzg3hg5jbbrpc26
title: Document durable disk-usage checkpoints and daily delta workflow
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
created_at: 2026-09-13T21:09:50.127Z
updated_at: 2026-09-13T21:31:44.024Z
closed_at: 2026-09-13T21:31:44.018Z
close_reason: "Published docs-only PR #55 at dc27c142d6422952159587bd891d56d75379eeba from an isolated worktree. All 19 CI checks passed; local handoff targets completed with Python 3.12 for the ABI-compatible packaging checks. Research, cache guide, FSEvents plan, campaign link, and immutable checkpoint plan are consistent; implementation follow-ups remain open."
resolution: null
duplicate_of: null
---

## Notes

PR https://github.com/jlevy/fdu/pull/55, commit dc27c142d6422952159587bd891d56d75379eeba, five documentation files only, main base b75bf85a33ed. The original FDU checkout is clean and remains on codex/streaming-performance-parity at afbb2eef01e9. Precommit review corrected cache policy/persistence claims, v3 cursor collision, FullHistory overlap, insufficient-history inference, next-day age-gate conflict, whole-command O(N) costs, engine ownership, and fixed-baseline accounting. No remaining actionable review findings. Validation: all make check targets completed successfully; the first run hit the local default cp314t/abi3 wheel mismatch at python-smoke, and the remaining python-smoke, python-sdist-smoke, parity-check, and release-test targets passed with UV_PYTHON=3.12. Format, whitespace, footers, and 60 local links pass. Follow-ups fdu-uwhl and fdu-8ybz and existing replay issues synchronized. PR is mergeable; awaiting all 19 CI checks before handoff. Build artifacts and temporary MSRV toolchain are on external storage.
