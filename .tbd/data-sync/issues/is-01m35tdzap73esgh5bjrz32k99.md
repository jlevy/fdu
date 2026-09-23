---
type: is
id: is-01m35tdzap73esgh5bjrz32k99
title: "Compose PR #97 and #98 without breaking Windows summary-fold validation"
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m35sm4jn2ytfmy91136g50b7
created_at: 2026-09-23T00:25:13.792Z
updated_at: 2026-09-23T00:25:13.792Z
---
Sol merge rehearsal found scan.rs conflict between Unix transient stat skipping and Windows observation. Astra verified production wrapper composition but found integration-only test failure: summary_fold_skips_stat_on_directories_and_symlinks asserts fewer stats everywhere while Windows now correctly performs equal observations. Preserve Unix savings and assert equality on Windows; state Windows bypass in wrapper docs. Scratch patch prepared in alpha review; apply with real merge, review final diff, run make check and cross-lint, then native Windows CI. Not a defect in either standalone head.
