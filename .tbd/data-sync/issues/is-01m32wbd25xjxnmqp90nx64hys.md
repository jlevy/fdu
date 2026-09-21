---
type: is
id: is-01m32wbd25xjxnmqp90nx64hys
title: "PR #98 review S4: release note for the VALIDITY_VERSION bump invalidating every store on every platform"
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m32h6dpd97fr5f8db831dn3y
created_at: 2026-09-21T21:01:03.428Z
updated_at: 2026-09-21T21:01:03.428Z
---
Suggestion from https://github.com/jlevy/fdu/pull/98#issuecomment-5764966314. snapshot.rs:91 and :219 bump VALIDITY_VERSION, which invalidates every existing store on every platform, not just Windows. Deliberate, but it needs a line in the 0.1 release notes / CHANGELOG when the release is assembled.
