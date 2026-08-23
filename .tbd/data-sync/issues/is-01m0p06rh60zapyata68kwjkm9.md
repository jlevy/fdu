---
type: is
id: is-01m0p06rh60zapyata68kwjkm9
title: "PR #42 R2: cargo package -p fdu fails; release rehearsal and release.yml broken"
kind: bug
status: closed
priority: 0
version: 2
labels: []
dependencies: []
parent_id: is-01m0p06qgww21s4fpdkj2bb6bf
created_at: 2026-08-23T00:26:54.373Z
updated_at: 2026-08-23T00:57:54.118Z
closed_at: 2026-08-23T00:57:54.118Z
close_reason: "Fixed in 4e34ce3; addressed the review on PR #42, verified through make check and make release-rehearse."
---
Makefile:306-307 and .github/workflows/release.yml:63. fdu depends on fdu-core which is not on crates.io, so packaging alone fails. Needs one invocation naming both.
