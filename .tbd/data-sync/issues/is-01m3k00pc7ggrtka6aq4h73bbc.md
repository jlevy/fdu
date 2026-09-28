---
type: is
id: is-01m3k00pc7ggrtka6aq4h73bbc
title: Investigate stable FSEvents misses and active-writer coverage before history refresh
kind: bug
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md
labels: []
dependencies: []
created_at: 2026-09-28T03:12:57.733Z
updated_at: 2026-09-28T03:18:21.256Z
---
September 27 shallow spike: controlled 20k tree exact with 712 observations, but live 454775-entry root had 1 then 2 stable misses despite HistoryDone, no degradation, flags 144. First missed append of 23948 bytes persisted on same-cursor repeat; targeted lsof found read/write descriptor. Determine actual omission mechanism using aged controlled open-writer tests, test generic active-writer metadata supplement with permissions/coverage/cost limits if warranted, and preserve full-scan fallback. Do not hardcode application paths, drop overlap, or call journal-only results fully verified. Evidence and continuation gate documented in replay plan and probe README.

## Notes

Confirmed a controlled failure, not merely a hypothesis: final open_writer.py source reproduced on two untouched aged synthetic leaves (one parent independent). With actual pre-append device-time fence and flags144, append+fsync while descriptor stayed open: HistoryDone,0events,0overlap,1stable mismatch. Close and replay SAME cursor:1fresh file event,exact101-entry oracle match. Both diagnostic stages valid with identity match and zero observation/degradation errors. Fresh-file control is overlap-masked. Track generic active-writer observation and honest coverage/fallback; does not prove exact cause of every live-root miss. See continuation JSON and active replay plan.
