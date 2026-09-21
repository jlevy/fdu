---
type: is
id: is-01m32twxbcecbbxabqwq19dz44
title: "PR #98 review R4: MSRV job runs on ubuntu only and cannot see the Windows-only module"
kind: bug
status: in_progress
priority: 2
version: 2
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m32h6dpd97fr5f8db831dn3y
hold: null
hold_until: null
created_at: 2026-09-21T20:35:40.012Z
updated_at: 2026-09-21T20:36:00.253Z
started_at: 2026-09-21T20:36:00.253Z
---
Medium from https://github.com/jlevy/fdu/pull/98#issuecomment-5764966314. .github/workflows/ci.yml:162-176 MSRV job checks ubuntu only; windows_metadata.rs is MSRV-checked by nothing required. Fix: add --target x86_64-pc-windows-msvc to the MSRV job.
