---
type: is
id: is-01m4xmy4txa7se60qzydepebgp
title: Clarify API names and separate lifecycle from materialization costs
kind: task
status: in_progress
priority: 2
version: 4
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-10-14T16:46:40.476Z
updated_at: 2026-10-14T17:16:04.233Z
started_at: 2026-10-14T16:46:52.679Z
---

## Notes

Implemented near-top Report/Index/Progressive tables in README, Python guide, architecture and review. Guidance separates lifecycle from result volume and representation, supports offline progressive use, recommends eligible one-shot summary reduction or reuse of existing handles, and retains Index for analysis/cache/explicit-refresh needs. Native lifecycle/coarse-request policy is prescriptive. Added surface roadmap with priorities, acceptance and eight follow-ups, including cached-index explicit verification (fdu-l9jh). Documented intentional Python stale_ok refresh/watch restriction and required verified reopen. Sol mechanics and Astra technical/adversarial review completed; publication awaits full local gate and final CI.
