---
type: is
id: is-01m0p0j68meqr2mst5pjxpj2e2
title: "PR #42 R21: two live guides still name engine files under crates/fdu/src"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0p06qgww21s4fpdkj2bb6bf
created_at: 2026-08-23T00:33:08.884Z
updated_at: 2026-08-23T00:57:54.167Z
closed_at: 2026-08-23T00:57:54.167Z
close_reason: "Fixed in 4e34ce3; addressed the review on PR #42, verified through make check and make release-rehearse."
---
Beyond the review's R5 and R11: docs/project/guides/integration-runbook.md:134 links crates/fdu/src/execution.rs, and docs/project/guides/platform-tuning.md:75,168 name crates/fdu/src/scan.rs and crates/fdu/src/platform_tuning.rs. All four files are in fdu-core now. Both are live guides AGENTS.md points readers at. Dated material under reports/ and research/ is left alone as historical, per the review's own principle for specs/done.
