---
type: is
id: is-01m01wbpqrqxahyc9ea3fsecs1
title: Restore GitHub Actions execution for PR checks
kind: task
status: closed
priority: 1
version: 3
labels: []
dependencies: []
created_at: 2026-08-15T04:54:53.431Z
updated_at: 2026-09-17T02:10:45.936Z
closed_at: 2026-09-17T02:10:45.935Z
close_reason: GitHub Actions runs on every PR and push (e.g. CI 35155441744 on 5f2d36d)
resolution: null
duplicate_of: null
---
GitHub Actions run 31865564861 for PR 27 did not start because account payments failed or the spending limit must be increased. After Billing & plans is corrected, rerun CI and confirm every required check completes.

## Notes

PR 27 run 31868608226 on commit 488a9bc reproduced the same account-level billing/spending-limit failure at Supply-chain provenance; every downstream job was skipped. The complete local make check passed.
