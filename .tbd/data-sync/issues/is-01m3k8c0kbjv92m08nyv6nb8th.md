---
type: is
id: is-01m3k8c0kbjv92m08nyv6nb8th
title: Report per-file coverage in the FSEvents spike harness
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T05:38:57.258Z
updated_at: 2026-09-28T05:38:57.258Z
---
Harness audit finding: real_tree.py compare_oracles labels any path whose oracles differ 'concurrent', hiding pre-replay changes that were never nominated, and shallow relists make sibling-refreshed files count as stable-equal. Classify against the baseline row (hidden gap / masked gap / pure concurrency) so trials report coverage, not just oracle agreement.
