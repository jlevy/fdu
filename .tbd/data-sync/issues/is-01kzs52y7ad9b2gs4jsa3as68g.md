---
type: is
id: is-01kzs52y7ad9b2gs4jsa3as68g
title: Watermark round-trip test for scan_started_at fed back as --modified-since
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
created_at: 2026-08-11T19:34:13.480Z
updated_at: 2026-08-11T19:34:13.480Z
---
The spec's Testing Strategy asks for a watermark round trip: a report's scan_started_at fed back as --modified-since must list exactly the files touched after scan start, including one touched mid-scan. No such test exists. This is the property that makes incremental follow-up queries trustworthy, and the mid-scan case is the one that can silently drop a file - a walker that recorded a directory before a write and stamped scan_started_at after it would miss that write forever. Needs a scan with an injected pause so a file can be touched while the walk is in flight.
