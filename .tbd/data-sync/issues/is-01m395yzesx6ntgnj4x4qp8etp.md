---
type: is
id: is-01m395yzesx6ntgnj4x4qp8etp
title: "PR #123 review R2: check-environment must read every deployment policy"
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m395wcbdbx8ndh16agfhmrhy
created_at: 2026-09-24T07:44:28.630Z
updated_at: 2026-09-24T07:44:28.630Z
---
scripts/release/publish_gate.py check_environment reads /deployment-branch-policies with the default page size and ignores total_count; a branch rule beyond page 1 passes. Request per_page=100, require total_count == len(branch_policies), and test a non-v tag rule. PR #123 review R2 (Medium).
