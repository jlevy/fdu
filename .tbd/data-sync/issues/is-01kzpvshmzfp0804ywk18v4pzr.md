---
type: is
id: is-01kzpvshmzfp0804ywk18v4pzr
title: Iteratively profile and optimize real-world traversal
kind: epic
status: open
priority: 1
version: 25
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
child_order_hints:
  - is-01kzpvt1me7pzvhsjnpk3yag8s
  - is-01kzpvt1vamkqp8fffnpwhd93v
  - is-01kzpvt22bex8ed6d155y014py
  - is-01kzpvt29hvtsrg1pyrq20awxa
  - is-01kzvqcp0wf2y0fwh6cgq16dxp
  - is-01kzvqcp8pzrxtc0m34fz17ymq
  - is-01kzvqqawcnm795f25dd6dpbbg
  - is-01kzw3t92p7d4512h8vn6ktch1
  - is-01kzwt2czaa7ax4yrzbcsszejj
  - is-01kzwt2d8dapafh6tmf9f92gek
  - is-01kzwt2dggjhccjvvsbnarj4qd
  - is-01kzw3te81j66eehy48rx2djv5
  - is-01kzwxffnkwm7kkmrdpgn5rbsn
  - is-01kzxfvezwjx51r861v2jy2bna
  - is-01kzxpk3n81xyvcrfjr3q60g6v
  - is-01kzxqxeqjqqzhr1m50xb0x2mk
  - is-01kzxrp40f39dhxvkv6cdjn2jw
  - is-01kzxsh1gqs697dy3eat13fw22
  - is-01kzxsmcabr3shfgh9644tbdtg
  - is-01kzxwah348yq9sg1em0cqv2k4
  - is-01kzxws8bayz24vajmdx2jwyf4
  - is-01kzy09g92seeh160bbh3m74nk
  - is-01kzy1w2vbam0mr1z5we4y6fy0
created_at: 2026-08-10T22:13:19.646Z
updated_at: 2026-08-13T17:14:15.274Z
---
Run a measurement-first optimization campaign on an operator-supplied checkout with tens of thousands of files and a large dependency tree, using the local metabrowser checkout as the first subject without persisting personal absolute paths. Measure snapshot-absent and compatible-snapshot behavior separately, keep filesystem-cache state explicit, profile before each change, commit each accepted improvement independently, and retain rejected experiments when gains are small, unstable, or not worth their complexity. This campaign coordinates the existing walker, revalidation, snapshot, and final-report beads rather than weakening their correctness gates.

## Notes

Sequence is baseline/oracle, snapshot-absent profile loop, compatible-snapshot profile loop, then multi-scale decision ledger. Use an operator-supplied large JS checkout without persisting its personal absolute path. Each accepted improvement is one commit with paired evidence; rejected complexity is documented and reverted.
