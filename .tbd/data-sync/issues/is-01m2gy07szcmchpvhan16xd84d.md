---
type: is
id: is-01m2gy07szcmchpvhan16xd84d
title: Permission-bit tests skip by early return and report ok while asserting nothing
kind: bug
status: in_progress
priority: 2
version: 2
delegate: codex@spud10
labels:
  - stack-followup
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-14T21:43:34.964Z
updated_at: 2026-09-20T05:23:44.145Z
started_at: 2026-09-20T05:23:44.145Z
---
Guideline conformance review of PR #56, finding 2 (https://github.com/jlevy/fdu/pull/56#pullrequestreview-5203155693). Sites: scan.rs:7750 and opened.rs:5462 at cfd1335, plus about 7 existing tests. When test_support::permission_bits_are_enforced() is false (for example when running as root, or on a filesystem that ignores mode bits), these tests eprintln 'skipped' and return. libtest captures the output and reports ok. general-testing-rules: never let a skipped selection look like a pass. Fix: an env-gated precondition, for example FDU_TEST_ALLOW_NO_PERMISSION_BITS=1, that panics unless that tier is declared, so CI proves the tests actually ran. rust-testing-rules has no dynamic-skip primitive, so document the mechanism in the testing guide.
