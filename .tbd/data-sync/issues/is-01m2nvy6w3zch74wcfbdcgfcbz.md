---
type: is
id: is-01m2nvy6w3zch74wcfbdcgfcbz
title: "Release rehearsal: make ignored-share byte fixture platform-exact"
kind: bug
status: closed
priority: 1
version: 4
delegate: codex@spud10
labels:
  - release
dependencies: []
parent_id: is-01m01cj7m8tfwapt8575agzmgn
created_at: 2026-09-16T19:43:43.488Z
updated_at: 2026-09-16T20:09:55.964Z
closed_at: 2026-09-16T20:09:55.961Z
close_reason: "Fixed by PR #74 (29472e3): byte-accounting smoke fixtures now use write_bytes, and all 19 CI jobs passed including both Windows wheel smokes."
resolution: null
duplicate_of: null
---
Release rehearsal run 35141677457 reached the Windows installed-wheel public smoke after the nanosecond fix, then failed because check_reports_carry_the_ignored_share writes dist/\n with Path.write_text and hard-codes the Unix byte total 18. Windows text newline translation makes the file one byte larger, so the assertion sees 19. Write byte-exact fixture contents, validate locally and in Windows CI, merge, then rerun release.yml to a green immutable-artifact audit.

## Notes

Fix committed as 610039e and opened as PR #74. The byte-accounting fixture now writes .gitignore and counted files with write_bytes, so the asserted 18-byte total is identical on Windows and Unix. Local installed-wheel public_smoke.py and full make check pass. Awaiting Windows CI, then merge and a fresh release.yml rehearsal.
