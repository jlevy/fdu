---
type: is
id: is-01kzwxffnkwm7kkmrdpgn5rbsn
title: Compare cache-off FDU summary with dumac
kind: task
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - benchmark
  - dumac
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-13T06:38:13.682Z
updated_at: 2026-08-13T07:36:15.047Z
---
Run a claim-grade adjacent paired comparison of fdu --cache off --view summary against dumac on the canonical million-scale workspace. Distinguish no persisted snapshot from no retained in-memory index, validate output semantics and hard-link accounting, publish the result, and decide whether H59 should include a true transient summary-only library path.

## Notes

Claim-grade exp-040 completed on a frozen 978,339-entry APFS tree (864,914 files, 113,066 dirs; digest 4e615217...). Twelve adjacent pairs plus three warmups, zero drift/invalid samples. Derived rich-summary path vs pre-change indexed summary: paired wall -14.57% when expressed candidate-vs-control (control was +17.047% vs anchor), 95% CI approximately -18.54% to -9.04%; peak RSS 27.7 MiB vs 590.6 MiB, user CPU about 66% lower. Stable report semantic hash identical on all 24 FDU samples. Dumac remained 14.93% faster [5.74%, 19.97%] but computes a narrower hard-link-deduplicated allocated total. Harness is being hardened to invalidate mismatched semantic hashes before publishing.
