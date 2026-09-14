---
type: is
id: is-01m2gy0b2zj27j2fb6fzqjvefd
title: "Replace #[allow] with #[expect] and a reason across the workspace"
kind: task
status: open
priority: 3
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T21:43:38.330Z
updated_at: 2026-09-14T21:43:38.330Z
---
Guideline conformance review of PR #57, finding 5 (https://github.com/jlevy/fdu/pull/57#pullrequestreview-5203155952). rust-lint-format-rules: use #[expect(lint, reason = ...)] so a suppression expires once its cause is fixed. The repository has no #[expect] today, and MSRV 1.85 supports it. #57 extended #[allow] on fdu-py/src/lib.rs:1465 and :1530. Convert workspace-wide in one PR, and let clippy report any expectation that goes unfulfilled.
