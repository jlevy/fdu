---
type: is
id: is-01m2eag6gyvzzt41f6xagdz2ap
title: "PR #51 review PLAN-2: 'watching bypasses it by name' is recorded as a rule"
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2eafpfpe8k5c9z9dhrqvy2y
created_at: 2026-09-13T21:24:17.565Z
updated_at: 2026-09-13T22:08:00.523Z
closed_at: 2026-09-13T22:08:00.522Z
close_reason: "Fixed in b36d5aa on PR #51: the plan says the watch layer honors the same gate as the scan it continues, matching the COMMIT-2 fix (046c9ec)."
resolution: null
duplicate_of: null
---
PR #51 review PLAN-2 (Medium). plan-2026-08-25-fdu-opened-root-inventory-engine.md line 1775 licenses the watch path to ignore the scan policy, which is the mechanism of COMMIT-2. Fix in step with COMMIT-2: the watch path honors the same gate as scans.
