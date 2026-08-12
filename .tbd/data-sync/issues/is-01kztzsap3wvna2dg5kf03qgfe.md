---
type: is
id: is-01kztzsap3wvna2dg5kf03qgfe
title: Test a larger macOS bulk metadata buffer (H55)
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzpvt1vamkqp8fffnpwhd93v
created_at: 2026-08-12T12:40:04.546Z
updated_at: 2026-08-12T12:44:19.700Z
closed_at: 2026-08-12T12:44:19.699Z
close_reason: "Rejected and reverted in exp-029: 256 KiB was neutral on cold index (-1.80%, CI crossing zero), regressed producer wall/RSS/faults, and left warm unchanged; 64 KiB remains the measured operating point."
---
Post-exp-026 profiles leave getattrlistbulk at 25.93% of cold and 16.61% of warm samples. Test 256 KiB versus the current 64 KiB per Reader to reduce repeated bulk calls in wide directories. Pre-registered signal: at least 3% cold-index or producer wall/component improvement at 60k with system CPU down; warm revalidate may compose. Record RSS explicitly (about 1.1 MiB additional capacity across six cold workers). Confirm at 720k only if the 60k gate or a clearly scale-dependent syscall signal is promising; otherwise revert.
