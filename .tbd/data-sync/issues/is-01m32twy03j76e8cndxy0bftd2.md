---
type: is
id: is-01m32twy03j76e8cndxy0bftd2
title: "PR #99 review R1: make check as root fails as scattered panics; AGENTS.md does not name the opt-out and nothing detects the host up front"
kind: bug
status: closed
priority: 2
version: 3
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m32h6ea5cf5dd7rzf7ea0jdh
hold: null
hold_until: null
created_at: 2026-09-21T20:35:40.675Z
updated_at: 2026-09-21T20:57:26.670Z
started_at: 2026-09-21T20:36:00.258Z
closed_at: 2026-09-21T20:57:26.670Z
close_reason: "Fixed in bf8f6241 on codex/release-ignore-correctness: a permission-bits Makefile preflight probes a mode-000 file the way the tests do and fails once with one message naming FDU_TEST_ALLOW_NO_PERMISSION_BITS=1; rust-test, lib-only and msrv depend on it. Verified as root: fails with the message, passes with the opt-out; scripts/check-uv-version.test.mjs still passes. AGENTS.md gains a Test Host Preconditions section naming both variables."
resolution: null
duplicate_of: null
---
Medium from https://github.com/jlevy/fdu/pull/99#issuecomment-5764971659. README.md:298-302 documents FDU_TEST_ALLOW_NO_PERMISSION_BITS / FDU_TEST_ALLOW_NO_NATIVE_WATCH; AGENTS.md does not, and nothing detects an unenforced-permission host before the suite runs. Fix: a Makefile preflight that probes mode bits and fails fast with one message (as uv-version does), plus the AGENTS.md note.
