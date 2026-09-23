---
type: is
id: is-01m35tdzap73esgh5bjrz32k99
title: "Compose PR #97 and #98 without breaking Windows summary-fold validation"
kind: task
status: closed
priority: 1
version: 4
labels: []
dependencies: []
parent_id: is-01m35sm4jn2ytfmy91136g50b7
created_at: 2026-09-23T00:25:13.792Z
updated_at: 2026-09-23T16:24:23.939Z
closed_at: 2026-09-23T16:24:23.937Z
close_reason: "Composed for real on 2026-09-23: #97 merged main (0b8aca22) with the reviewed execution.rs/scan.rs resolution (Windows equal-observation test split by cfg), merged to main in d69c705b; tree verified identical to the rehearsed composition that passed make check and cross-lint; native Windows CI green."
resolution: null
duplicate_of: null
---
Sol merge rehearsal found scan.rs conflict between Unix transient stat skipping and Windows observation. Astra verified production wrapper composition but found integration-only test failure: summary_fold_skips_stat_on_directories_and_symlinks asserts fewer stats everywhere while Windows now correctly performs equal observations. Preserve Unix savings and assert equality on Windows; state Windows bypass in wrapper docs. Scratch patch prepared in alpha review; apply with real merge, review final diff, run make check and cross-lint, then native Windows CI. Not a defect in either standalone head.

## Notes

Astra confirmed the scratch production composition preserves both contracts. Sol final c5b759ee scopes equal stat counts to cfg(windows), fewer counts to cfg(not(windows)), saving2 on Unix and saving1 on other non-Windows non-Unix targets. Rustfmt and diff checks pass. No Rust build/native Windows execution of combined tree yet. integration-scan-final.patch saved with audit artifacts; real merge and CI still required.

2026-09-22 read-only current-stack rehearsal: PR117 local 8edd9b21 merges cleanly with PR94 03b48aa3, but PR97 7643f9a1 and PR105 245395c0 each conflict in exactly execution.rs and scan.rs. Preserve Basis/Plan request routing and scan error normalization while composing transient fold, recycling, and d_type stat skip; Windows equal-observation test remains required. Exact paths/SHAs in /tmp/fdu-alpha-stack-PnTm5M/reconciliation-audit.md. No real merge or new runtime change yet.
