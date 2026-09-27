---
type: is
id: is-01m3gspe14ztewpqs5grdvzc2s
title: Audit hard-link allocation accounting and scope a follow-up
kind: task
status: closed
priority: 2
version: 5
spec_path: docs/project/specs/done/plan-2026-09-26-directory-inventory-accounting.md
labels: []
dependencies: []
created_at: 2026-09-27T06:44:01.176Z
updated_at: 2026-09-27T08:18:19.526Z
closed_at: 2026-09-27T07:10:17.895Z
close_reason: "Assessment and verified examples delivered in stacked PR #135; issue #93 confirmed implemented by #117, per-path hard-link accounting documented with measured evidence, future unique allocation retains fdu-579b/fdu-8ybz ownership. Full make check and all 19 CI jobs pass."
resolution: null
duplicate_of: null
---
User requested careful analysis of allocated-byte reporting with hard-linked files across uv environments and a decision on current coverage versus a future capability. Distinguish pathname totals, unique file identity, clones/reflinks, and reclaimable bytes; record current evidence and scoped recommendations.

## Notes

Audit complete: per-path allocation confirmed with direct hard-link fixture; nested root union distinguished from inode deduplication. Existing fdu-579b and fdu-8ybz retain future design ownership. Assessment and usage documentation added in stacked documentation layer; awaiting final CI.
