---
type: is
id: is-01m01ecbhsetn1rmvfn8m26w7e
title: "H88: Separate macOS bulk, fallback, and topology scheduling signals"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - research
  - experiment
  - macos
dependencies:
  - type: blocks
    target: is-01m01cm1sb8xyw9ag3pabb5s3h
  - type: blocks
    target: is-01m01ebsw9cyhe8thve19grn1w
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:50:34.680Z
updated_at: 2026-08-15T00:52:00.851Z
---
Exp-021 calibrated work per entry before getattrlistbulk changed the unit of work to directory opens and bulk batches. Determine whether one selector can serve both the macOS bulk path and portable fallback, or whether the controller needs backend-aware observations. Compare entries, directories, entries-per-directory, bulk-call and fallback counts, completed versus in-flight service, ready frontier width, consumer backlog, and achieved throughput across wide, deep, many-small-directory, mixed-phase, mount/firmlink boundary, and permission fixtures.

Candidate directions include a shared feedback controller whose signals naturally cover both backends, a bulk-path conservative policy plus adaptive portable fallback, and directory/service-based calibration instead of per-entry timing. Reject topology-specific constants and any policy that changes semantic fallback behavior. Feed the measured result into H86 and record every candidate and regime in the ledger.
