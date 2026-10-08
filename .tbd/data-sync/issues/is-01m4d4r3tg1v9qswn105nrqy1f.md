---
type: is
id: is-01m4d4r3tg1v9qswn105nrqy1f
title: Decide the README 'fastest' claim's macOS basis before 0.4.0 (exploratory dumac cell)
kind: task
status: closed
priority: 1
version: 2
labels:
  - release
  - docs
  - performance
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
created_at: 2026-10-08T06:55:54.703Z
updated_at: 2026-10-08T18:08:43.730Z
closed_at: 2026-10-08T18:08:43.728Z
close_reason: "Maintainer decision 2026-10-08: keep the tagline and sentence; label the macOS column and note exploratory/uncontrolled (README, docs/performance-measurements.md). Applied on #185 in d111a98c."
resolution: null
duplicate_of: null
---
README.md:3 "Fastest du replacement" and :9-11 "finished ahead of du and the seven other disk-usage tools measured": reaching seven needs dumac, measured only on macOS, in one uncontrolled session (49-100% CPU busy) on a pre-0.2.0 build (a5c0ab46), whose own report (report-2026-09-26-fdu-live-tool-comparison.md:5) calls itself exploratory; performance-loop.md:142-148 says uncontrolled supports exploration only. README :449-450 and docs/performance-measurements.md:147 say "heavily loaded host" but not exploratory/uncontrolled. The Linux half (exp-202, quiet, 20 pairs) supports a lead over pdu, diskus and the rest, narrowest +9.7% [+1.8, +12.2] over pdu --max-depth 2.

Maintainer decision (the claim was restored deliberately in #183): (a) scope the tagline/sentence to Linux and label the macOS column "exploratory, uncontrolled, pre-0.2.0"; or (b) re-run the macOS peer cell quiet on the 0.4.0 candidate before the release-prep PR; or (c) keep as is with the label added. Ships in the 0.4.0 release commit.
