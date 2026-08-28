---
type: is
id: is-01m12sw58r03v2pvpy8s0pt99c
title: "Port the scripted-event path guard fix from PR #47 ledger row 5ace86c"
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies: []
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-27T23:46:23.120Z
updated_at: 2026-08-28T00:20:39.241Z
closed_at: 2026-08-28T00:20:39.229Z
close_reason: null
resolution: null
duplicate_of: null
---
PR #47 commit 5ace86c replaced a path guard in crates/fdu-core/src/watch/scripted_events.rs that did not guard. The plan's PR #47 implementation ledger row for 5ace86c says: 'Port each Windows-only correction with its test when the affected code moves; do not cherry-pick the accumulated diff.'

The code has moved. scripted_events.rs exists in neither main nor the #48 merge base, so the file originated in PR #47 (9460231) and the opened-root rewrite extracted it, taking the pre-fix version.

Current state on codex/opened-root-inventory-rewrite:
- scripted_events.rs asks relative.is_absolute(), which answers false for a Windows rooted path with no drive prefix, so the guard does not guard there.
- '..' slips past the same check on EVERY platform, Unix included, and is joined onto the watch root.
- crate::index::path_is_representable is still private (plain fn at index.rs:3611); the fix makes it pub(crate) so there is one definition rather than two that agree until one is edited.
- The '..' test cases 5ace86c added are absent (zero occurrences).

Fix: replace the is_absolute check with !crate::index::path_is_representable(relative), promote that function to pub(crate), update the refusal message to say the path escapes the watch root, and port the '..' test cases.

The other two corrections in 5ace86c need no action here: the symlink 'nt' guard is already present in #48, and JSON_SEP addressed a golden #48 does not have.

See the close note on https://github.com/jlevy/fdu/pull/47
