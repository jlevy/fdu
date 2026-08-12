---
type: is
id: is-01kztzfvcgsf3nd7tt5z3mh9fr
title: Reuse macOS bulk directory staging allocations (H54)
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzpvt1vamkqp8fffnpwhd93v
created_at: 2026-08-12T12:34:53.967Z
updated_at: 2026-08-12T12:34:53.967Z
---
Post-exp-026 cold/warm profiles still attribute 7.76-14.90% to allocation. macos_bulk::Reader currently creates and drops one Vec<Entry> per directory (7,350 at 60k; 88,201 at 720k). Retain the staging Vec in each Reader and return a draining iterator so capacity is reused while preserving complete-directory atomic fallback. Pre-registered signal: user CPU/minor faults down and at least 3% wall or component improvement on cold index, producer, or warm revalidate, with exact oracle parity and no material RSS regression; test 60k then confirm 720k if promising.
