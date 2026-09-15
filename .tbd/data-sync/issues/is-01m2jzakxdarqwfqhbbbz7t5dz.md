---
type: is
id: is-01m2jzakxdarqwfqhbbbz7t5dz
title: "PR #61 review PR61-GUIDE-1: the Linux reproduction fallback must also carry cargo publish"
kind: bug
status: closed
priority: 2
version: 3
labels:
  - release
dependencies: []
parent_id: is-01m2jzadk7w8m1xcsewzwg5wj1
created_at: 2026-09-15T16:45:12.491Z
updated_at: 2026-09-15T17:10:14.345Z
closed_at: 2026-09-15T17:10:14.344Z
close_reason: "38cbe7b: guide says cargo publish cannot upload a prebuilt .crate (1.97.1 takes no archive argument), so the host whose digests matched publishes; a Linux-only match runs all of Publish the Crates there"
resolution: null
duplicate_of: null
---
PR #61 at eb89150, docs/project/guides/release-process.md:215-222. Step 2 of Publish the Crates says to reproduce on Linux x86-64 when a maintainer's cargo package digests differ from the rehearsal's, but not that cargo publish must then run on Linux too. cargo publish repackages (line 109), so a literal reading reproduces on Linux, returns to macOS, and uploads the mismatching bytes. Fix: say the host whose packaging matched is the host that publishes; check whether cargo publish can upload a prebuilt .crate, and prescribe that if it can.
