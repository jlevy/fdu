---
type: is
id: is-01m2p0vd9dkk80npgktbcspqy4
title: Clarify CLI views, analysis, cache reuse, and release usage docs
kind: feature
status: closed
priority: 1
version: 4
labels:
  - release-docs
dependencies: []
created_at: 2026-09-16T21:09:34.635Z
updated_at: 2026-09-16T22:00:54.665Z
closed_at: 2026-09-16T22:00:54.664Z
close_reason: "Merged in PR #76 after full local make check and 19/19 green GitHub checks; CLI views, analysis, cache behavior, offline docs, agent skill, text labels, goldens, and Python parity are updated."
resolution: null
duplicate_of: null
---
Before 0.1.0, reorganize the README and offline help around exact common commands; add a top-level documentation index and focused usage guide; state the default view and sizing behavior; distinguish metadata views from content analysis and explain where the cache helps; correct ignored-selection and analyzer behavior; update stale cache-design prose; and label non-byte percentage denominators in text output. Keep the existing PATH + --view + --analyze CLI model and avoid new aliases or implicit analysis. Update goldens and verify all three surfaces end to end.

## Notes

Implemented the Astra-reviewed design without changing the CLI grammar: README common-use table, docs index and focused usage guide, concise --help examples, rewritten --docs, synchronized agent skill, corrected cache architecture/design prose, and explicit text labels for code-line/document-word percentage denominators. Updated CLI goldens and Python parity recording. Validation: clean full make check, including 161 CLI goldens, all-feature/no-default/watch/MSRV Rust matrices, Python tests/type checks, built wheel and sdist installs, public API smokes, parity replay, and 32 release-contract tests.
