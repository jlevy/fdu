---
type: is
id: is-01kztzsap3wvna2dg5kf03qgfe
title: Test a larger macOS bulk metadata buffer (H55)
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzpvt1vamkqp8fffnpwhd93v
created_at: 2026-08-12T12:40:04.546Z
updated_at: 2026-08-12T12:40:04.546Z
---
Post-exp-026 profiles leave getattrlistbulk at 25.93% of cold and 16.61% of warm samples. Test 256 KiB versus the current 64 KiB per Reader to reduce repeated bulk calls in wide directories. Pre-registered signal: at least 3% cold-index or producer wall/component improvement at 60k with system CPU down; warm revalidate may compose. Record RSS explicitly (about 1.1 MiB additional capacity across six cold workers). Confirm at 720k only if the 60k gate or a clearly scale-dependent syscall signal is promising; otherwise revert.
