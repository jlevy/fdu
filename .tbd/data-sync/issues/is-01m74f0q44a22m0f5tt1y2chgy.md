---
type: is
id: is-01m74f0q44a22m0f5tt1y2chgy
title: Count every entry a window decided on a stand-in change time, and put the count in machine output
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-11T04:49:17.956Z
updated_at: 2026-11-11T04:49:17.956Z
---
Deferred from review F8 on jlevy/fdu#209 (https://github.com/jlevy/fdu/pull/209#issuecomment-6786513339). Windows-only in practice: Linux and macOS always observe a status change, so only an entry a Windows walker listed but could not open lacks one (`Attrs::change_time_ns` is `None` and `Attrs::times` substitutes the modification time on the change clock).

Today the note `note: N files have no observed status change, so their modification time stands for their change time` counts only regular files that a time window judged (`Walked::timed_by_modification` in `crates/fdu-core/src/query/query_report.rs`, `walk`) or that `ages` banded. Two cases stay silent:

1. Directories decided on a maximum that holds a stand-in. Under `--kind dir --before 30d` (the staleness recipe), the rest of the selection admits no file, so no file is counted, and a directory called stale on a lower bound gets no note. The members' count is reported only beside `ages`. Fix: count a member whenever its stand-in entered a decision, including a directory's maximum (the subtree measurement would need to carry how many unobserved files fed each maximum, or a flag), and say so in the note's wording if it counts more than files. Non-file entries (symlinks, other) judged by a window on a stand-in are likewise not counted; PR #209 made their list rows report the status as unobserved (`FileRow::status_observed`, `changed_ns: null`) but left the note to files.
2. Machine output. The count exists only in the note, on stderr. A JSON consumer of `ages`, `summary`, or `tree` cannot see it, where selection coverage got envelope fields `selected` and `scanned` so a parser need not read a note. Fix: put the count on the envelope (for example `timed_by_modification`) or on the `ages` section, in `report_format.rs`'s field walk, the Python models, and `docs/machine-output.md`, under a schema version that has not shipped (`fdu.report/12` until its release, otherwise a bump).

Done when both cases are counted, the count is in machine output, the goldens and parity are updated, and tests pin a directory decided on a stand-in and the envelope field.
