---
type: is
id: is-01m395yzesx6ntgnj4x4qp8etp
title: "PR #123 review R2: check-environment must read every deployment policy"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m395wcbdbx8ndh16agfhmrhy
created_at: 2026-09-24T07:44:28.630Z
updated_at: 2026-09-24T07:50:43.439Z
closed_at: 2026-09-24T07:50:43.436Z
close_reason: "Fixed in 15e61d0e: per_page=100 and total_count check in check_environment; non-v tag and under-read listing tests."
resolution: null
duplicate_of: null
---
scripts/release/publish_gate.py check_environment reads /deployment-branch-policies with the default page size and ignores total_count; a branch rule beyond page 1 passes. Request per_page=100, require total_count == len(branch_policies), and test a non-v tag rule. PR #123 review R2 (Medium).
