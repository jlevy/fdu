---
type: is
id: is-01m77c2rydychbdetqjnc7zpqy
title: Measure large-scan stage costs on a many-directory external volume
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m77c2ng8nt6bt6tr2xmjgbsh
created_at: 2026-11-12T07:55:42.925Z
updated_at: 2026-11-12T07:55:42.925Z
---
#207 §6.1/§6.2. The issue's 36-minute macOS external-APFS run (9.26M files, 3.55M dirs, 5.2 GiB footprint) had two copy processes on the volume and is not a baseline. Nominate a named workload job, add stage timers for selection, serialization, and persistence (counters lack them) and a bulk-reader fallback rate, then paired perf-compare runs including partial coverage and repeated queries; needs a macOS host. Retention work (fdu-uea9, fdu-6o5o, fdu-n48k, fdu-onf3) follows the profile.
