---
type: is
id: is-01m48emwnsdwf25hz13v4brjky
title: v0.3.0 warm revalidation is slower than its own cold scan on the Linux v6.12 tree
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-10-06T11:12:42.672Z
updated_at: 2026-10-06T11:12:42.672Z
---
Index cell macos-warm-metadata-warm-revalidate (PR #176 driver run, 2026-10-06): v0.3.0 whole-process revalidate 478 ms vs cold-open-save 303 ms and default tree 164 ms on K (internal SSD, uncontrolled host). The probe's own revalidation timer is ~124 ms; the rest is process start, snapshot load, and the probe's oracle digest. Design principles: a warm path that loses to a cold scan is a defect. Separate the oracle's cost (time without --no-oracle where supported), profile snapshot load, and decide whether the CLI's cached path loses too.
