---
type: is
id: is-01m2gy07szcmchpvhan16xd84d
title: Permission-bit tests skip by early return and report ok while asserting nothing
kind: bug
status: closed
priority: 2
version: 4
delegate: codex@spud10
labels:
  - stack-followup
dependencies:
  - type: blocks
    target: is-01m2yh8kc79nw7bn6k6xw8g3bp
hold: null
hold_until: null
created_at: 2026-09-14T21:43:34.964Z
updated_at: 2026-09-27T08:12:58.956Z
started_at: 2026-09-20T05:23:44.145Z
closed_at: 2026-09-27T08:12:58.955Z
close_reason: "Implemented: permission-bits preflight and explicit FDU_TEST_ALLOW_NO_PERMISSION_BITS opt-out in Makefile, test_support.rs, cli_exit.rs and AGENTS.md. Verified against current source and passing 19-job PR #133 CI (36303716655)."
resolution: null
duplicate_of: null
---
Guideline conformance review of PR #56, finding 2 (https://github.com/jlevy/fdu/pull/56#pullrequestreview-5203155693). Sites: scan.rs:7750 and opened.rs:5462 at cfd1335, plus about 7 existing tests. When test_support::permission_bits_are_enforced() is false (for example when running as root, or on a filesystem that ignores mode bits), these tests eprintln 'skipped' and return. libtest captures the output and reports ok. general-testing-rules: never let a skipped selection look like a pass. Fix: an env-gated precondition, for example FDU_TEST_ALLOW_NO_PERMISSION_BITS=1, that panics unless that tier is declared, so CI proves the tests actually ran. rust-testing-rules has no dynamic-skip primitive, so document the mechanism in the testing guide.
