---
type: is
id: is-01m00v044dfkgm0e4xqwnphsp2
title: "PR #22 review R1: restore fdu packaging and two-crate boundary"
kind: bug
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m00tzk6myk9ba0110gv86kdz
created_at: 2026-08-14T19:11:50.924Z
updated_at: 2026-08-14T19:24:17.072Z
closed_at: 2026-08-14T19:24:17.071Z
close_reason: Fixed by removing the third workspace crate, integrating instrumentation under fdu::counters, removing the path dependency, and restoring cargo package --locked -p fdu --allow-dirty --no-verify (48 files packaged successfully).
---
Blocker. PR #22 review R1. crates/fdu/Cargo.toml:71. fdu package fails because path dependency perfkit has no version; separate crate conflicts with documented two-crate boundary. Fold instrumentation into fdu or make an explicit publishable-crate decision.

## Notes

Choosing the review's primary fix: fold reusable instrumentation into fdu::counters, restore the documented two-crate workspace, and validate cargo package.
