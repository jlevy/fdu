---
type: is
id: is-01m32wbbwban5x8k14zwshwr9f
title: "PR #98 review S2: report allocated size on Windows from FILE_STANDARD_INFO"
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m32h6dpd97fr5f8db831dn3y
created_at: 2026-09-21T21:01:02.219Z
updated_at: 2026-09-21T21:01:02.219Z
---
Suggestion from https://github.com/jlevy/fdu/pull/98#issuecomment-5764966314. windows_metadata.rs sets allocated: size; FILE_STANDARD_INFO.AllocationSize is one more query on the handle already open and would make --size allocated meaningful on Windows. Out of scope for the validity fix; needs a Windows host to verify and to measure the added query with S1.
