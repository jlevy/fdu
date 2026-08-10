---
type: is
id: is-01kzmma7nxajpfh0deqty8gtx7
title: Replace opaque CLI golden corpus with a self-explanatory project fixture
kind: task
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/done/plan-2026-08-09-fdu-cli-golden-tests.md
labels:
  - testing
  - golden
dependencies: []
created_at: 2026-08-10T01:24:09.020Z
updated_at: 2026-08-10T01:39:44.361Z
closed_at: 2026-08-10T01:39:44.360Z
close_reason: The self-explanatory fixture, exact transcripts, documentation, local handoff gate, cross-platform CI, and PR context are complete.
---
Replace the mechanically minimal fixture under tests/golden/fixtures/tree with a small believable project whose contents match their filenames. Preserve end-to-end coverage for extensionless names, case-folded extensions, compound .tar.gz classification, equal-size name tie-breaking, nested roll-ups, render limits, JSON totals, and cache revalidation. Add nearby fixture documentation, regenerate and review every affected golden transcript, run make check, and update PR #1 with the rationale and evidence.

## Notes

Implemented in commit 95d0870. Replaced the opaque 37-byte token corpus with a documented 263-byte Acorn project containing a working Makefile target, meaningful Markdown and Rust files, an exact 18-byte tie pair, and a valid deterministic tar.gz archive. Updated human, JSON, max-depth, by-type, and cache lifecycle goldens without broadening stable values; kept zero mtimes exact. Added LF/binary Git attributes and a narrow dist fixture ignore exception. Local make check and GitHub Actions run 31347426235 both pass, including Windows and all 25 golden scenarios. PR #1 now explains the fixture design and links tryscript issue 49.
