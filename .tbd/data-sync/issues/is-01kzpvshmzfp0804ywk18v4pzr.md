---
type: is
id: is-01kzpvshmzfp0804ywk18v4pzr
title: Iteratively profile and optimize real-world traversal
kind: epic
status: open
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
child_order_hints:
  - is-01kzpvt1me7pzvhsjnpk3yag8s
  - is-01kzpvt1vamkqp8fffnpwhd93v
  - is-01kzpvt22bex8ed6d155y014py
  - is-01kzpvt29hvtsrg1pyrq20awxa
created_at: 2026-08-10T22:13:19.646Z
updated_at: 2026-08-10T22:16:05.281Z
---
Run a measurement-first optimization campaign on an operator-supplied checkout with tens of thousands of files and a large dependency tree, using the local metabrowser checkout as the first subject without persisting personal absolute paths. Measure snapshot-absent and compatible-snapshot behavior separately, keep filesystem-cache state explicit, profile before each change, commit each accepted improvement independently, and retain rejected experiments when gains are small, unstable, or not worth their complexity. This campaign coordinates the existing walker, revalidation, snapshot, and final-report beads rather than weakening their correctness gates.

## Notes

Sequence is baseline/oracle, snapshot-absent profile loop, compatible-snapshot profile loop, then multi-scale decision ledger. Use an operator-supplied large JS checkout without persisting its personal absolute path. Each accepted improvement is one commit with paired evidence; rejected complexity is documented and reverted.
