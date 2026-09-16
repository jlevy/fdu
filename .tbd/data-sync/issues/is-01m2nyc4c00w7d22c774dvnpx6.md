---
type: is
id: is-01m2nyc4c00w7d22c774dvnpx6
title: Fix Windows installed-wheel text parity and fail-fast smoke execution
kind: bug
status: closed
priority: 0
version: 3
labels:
  - release-blocker
dependencies: []
created_at: 2026-09-16T20:26:16.831Z
updated_at: 2026-09-16T22:07:45.762Z
closed_at: 2026-09-16T22:07:45.733Z
close_reason: "Fixed in PR #75 (merge 4f092ab). PR #76 and post-merge main CI both ran the Windows 3.12/3.14 installed-wheel UTF-8 smoke paths successfully, and Bash fail-fast behavior is live."
resolution: null
duplicate_of: null
---
The release rehearsal run 35145541624 exposed a Windows installed-wheel mismatch between Report.render(TEXT) and the wheel console script. Post-merge CI run 35144882898 contained the same public_smoke.py traceback but was marked green because its PowerShell multiline step continued to later commands and returned success. Diagnose the exact string difference, restore exact cross-surface parity on Windows, and make every wheel smoke step fail immediately when any command fails.

## Notes

Diagnosed release run 35145541624 and post-merge CI run 35144882898: subprocess text=True decoded Rust's UTF-8 pipe output with the Windows locale, yielding unequal Unicode despite identical-looking log bytes. Fix explicitly decodes UTF-8 in both wheel smoke scripts and runs the multiline CI step under bash so the first failed command fails the job. Full make check passed locally, including wheel/sdist smoke and parity.
