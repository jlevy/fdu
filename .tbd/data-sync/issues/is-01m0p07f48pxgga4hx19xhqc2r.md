---
type: is
id: is-01m0p07f48pxgga4hx19xhqc2r
title: "PR #42 R12: architecture doc quotes the dependency guard without --prefix none"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m0p06qgww21s4fpdkj2bb6bf
created_at: 2026-08-23T00:27:17.512Z
updated_at: 2026-08-23T00:57:54.150Z
closed_at: 2026-08-23T00:57:54.150Z
close_reason: "Fixed in 4e34ce3; addressed the review on PR #42, verified through make check and make release-rehearse."
---
docs/project/architecture/fdu-surface-architecture.md:54. Without --prefix none cargo tree prefixes deps, so ^clap can never match and the documented check can never fire.
