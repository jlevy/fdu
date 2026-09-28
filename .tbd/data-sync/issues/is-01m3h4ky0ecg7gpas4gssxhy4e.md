---
type: is
id: is-01m3h4ky0ecg7gpas4gssxhy4e
title: Re-test H157 (direct file fold, owned names) with the product CLI job as primary
kind: task
status: open
priority: 2
version: 7
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - linux
dependencies:
  - type: blocks
    target: is-01m3kkrj5f6n9b38g6d1w6mrew
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-09-27T09:54:53.582Z
updated_at: 2026-09-28T09:53:56.144Z
---
exp-161 rejected H157 on its pre-registered probe job: cold-scan-index wall -2.22% [-4.04%, +0.04%], component -4.12% [-7.40%, +1.25%]. The same change paired on the product CLI job (fdu --cache off --depth 1 --limit 10, tool harness, 12 pairs, quiet) measured -3.71% [-4.71%, -1.97%], and allocations fell 7.03M -> 4.28M. The change: DetachedIndexBuilder::push_directory folds a file child straight into its parent roll-up through InternedRollUp::add_file, and Index::contribution builds a file's contribution by applying add_file to an empty roll-up, so both paths share one definition; WalkEmission::record_entry takes the listing's owned OsString and record_detached_entry moves it into DetachedChild instead of to_os_string. Pre-register the product job as primary, measure on the balanced 1M tree and one reconstructible real subject (linux-v6.12), and keep only under the accept rule. See docs/project/reports/report-2026-09-27-fdu-linux-tool-comparison.md.

## Notes

## 2026-09-28: not run with H159; left for the Linux round

Considered on the H159 layer (claude/perf-h159-recycle) and deliberately not added:

- It cannot be decided tonight. Its pre-registered job is Linux (the product CLI job on
  linux-balanced-1m and linux-v6.12), and no Linux host is reachable. On macOS it would
  be a screen, and the layer would carry a second unmeasured change.
- It must be measured on top of H159, not beside it. Both target the index consumer's
  allocator cost; exp-161's own reading was that the allocation pattern (cross-thread
  frees), not volume, was the constraint, which is exactly what H159 changes. Two
  hypotheses aimed at one cost divide one budget (H13/H18, H74/fdu-91ts, H89/H86), so
  H157's rerun has H159's verdict as its starting point.
- The exp-161 patch is not in git (the rejected change was reverted without a commit),
  so the rerun reimplements it from exp-161's description: InternedRollUp::add_file used
  by both push_directory and Index::contribution, and an owned OsString through
  WalkEmission::record_entry into DetachedChild. About 95 lines touching the walker
  signature both emissions share.

Pre-registration for the rerun (unchanged from the bead, made explicit):

- Control: the H159 build if H159 is kept on Linux, otherwise 56c506e1 (stack-141 top).
  Candidate: control plus the H157 patch, one commit.
- Primary: product CLI job, the tool harness's `fdu` indexed-tree contract
  (fdu --cache off --color never --depth 1 --limit 10; make perf-compare-tools with
  PERF_TOOL_CONTRACT=fdu, anchor = control CLI, tool = candidate CLI, 12 adjacent pairs,
  3 warmups, quiet), wall.
  Secondary: probe cold-scan-index and default-tree, 12 pairs quiet.
- Subjects: linux-balanced-1m (screening) and reconstructible linux-v6.12 (deciding).
- Accept: median at least 3% faster, 95% interval entirely below zero on the deciding
  subject, zero invalid samples, peak RSS non-inferior; allocation counters recorded.
