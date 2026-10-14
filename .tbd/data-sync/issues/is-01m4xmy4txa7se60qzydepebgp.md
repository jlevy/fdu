---
type: is
id: is-01m4xmy4txa7se60qzydepebgp
title: Clarify API names and separate lifecycle from materialization costs
kind: task
status: closed
priority: 2
version: 5
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-10-14T16:46:40.476Z
updated_at: 2026-10-14T17:59:33.539Z
started_at: 2026-10-14T16:46:52.679Z
closed_at: 2026-10-14T17:59:33.529Z
close_reason: "Completed in PR #205 at bd7de58de9275919f740145a9d54926bd5f7a6e6: reconciled architecture and API guidance, eight-item tracked roadmap, three focused runtime fixes, and independent Astra review with no findings. All required local make check stages passed after a scoped embedded-Python host setup correction; 68 targeted and 81 full Python tests passed. CI run 38609685963 passed all 20 jobs on this exact commit. Eight future improvement beads remain open; no new performance claims."
resolution: null
duplicate_of: null
---

## Notes

Implemented near-top Report/Index/Progressive tables in README, Python guide, architecture and review. Guidance separates lifecycle from result volume and representation, supports offline progressive use, recommends eligible one-shot summary reduction or reuse of existing handles, and retains Index for analysis/cache/explicit-refresh needs. Native lifecycle/coarse-request policy is prescriptive. Added surface roadmap with priorities, acceptance and eight follow-ups, including cached-index explicit verification (fdu-l9jh). Documented intentional Python stale_ok refresh/watch restriction and required verified reopen. Sol mechanics and Astra technical/adversarial review completed; publication awaits full local gate and final CI.
