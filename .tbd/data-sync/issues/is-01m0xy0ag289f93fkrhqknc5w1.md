---
type: is
id: is-01m0xy0ag289f93fkrhqknc5w1
title: "Address the PR #48 senior design review (R1-R15)"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T02:22:21.698Z
updated_at: 2026-08-26T02:22:57.461Z
---
Published senior engineering review of PR #48 (design plan + PR #47 readiness report) at head 51d9b47, as a formal GitHub review on https://github.com/jlevy/fdu/pull/48

5 High:
- R1 depth-bounded watching. crates/fdu-core/src/scan.rs:268-274 rejects watching any scope with max_depth or one_filesystem, and fdu-design-principles.md states the rule as a Serving Model invariant. Every MetaBrowser root carries max_depth=20 as scope (contract.py:86-133). The plan supersedes fdu-7sou in a bead-table line with no design section and no doc amendment. Preferred fix: make depth selection, not scope.
- R2 "path order" is undefined. fdu is BTreeMap<OsString> pre-order (index.rs:231); MetaBrowser catalog is lexicographic full-path (python_inventory.py:1548); MetaBrowser directory rows are dirs-first-then-name (python_inventory.py:412). Three orders, none named, and Phase 4 requires envelopes to agree.
- R3 non-UTF-8 and Windows path identity has no stated semantics for unrepresentable entries against the three-valued absent/unknown rule.
- R4 exact filtered totals versus the deterministic work bound. total_matching is product data (recent.py:47 -> server.py:1841 -> app.js:3657); the Python provider maintains sorted mtime arrays for it; no fdu equivalent is named and no behavior is defined when the aggregate exhausts its budget.
- R5 the changes() bridge maps a blocking pull onto an AsyncIterator via asyncio.to_thread, which is neither bounded nor cancellable.

7 Medium: R6 PR body says non-cloneable, plan says cloneable, and close() under clones is undefined. R7 freshness/phase vocabularies do not map onto the contract. R8 "keep as scope decisions" names three fields that do not exist. R9 progressive-results spec still owns the same concepts. R10 TODO.md not updated with the spec or fdu-snej. R11 Phase 1 is several phases. R12 the one-long-lived-PR decision reproduces the shape the report diagnosed.

3 Low: R13 fdu-9tdm spec_path dangles onto a PR #47-only file. R14 verification note. R15 continuation-ID lifetime belongs in the joint contract.

Four items were checked and confirmed benign; they are listed in the review so they are not "fixed": at_version, deferred per-value provenance, remaining_rows removal, and the all/unignored partition.

Work these with: tbd shortcut address-pr-review
