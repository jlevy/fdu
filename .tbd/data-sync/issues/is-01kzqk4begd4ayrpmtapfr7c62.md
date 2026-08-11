---
type: is
id: is-01kzqk4begd4ayrpmtapfr7c62
title: "PR #3 review R13: remove STATX_CHANGE_COOKIE from userspace plans"
kind: bug
status: closed
priority: 2
version: 2
labels:
  - pr-review
dependencies: []
parent_id: is-01kzqk2ct4s2qjv9e2z17fvywr
created_at: 2026-08-11T05:01:10.990Z
updated_at: 2026-08-11T05:37:53.382Z
closed_at: 2026-08-11T05:37:53.381Z
close_reason: Removed STATX_CHANGE_COOKIE from the userspace opportunity set, documented that Linux masks the kernel-internal bit and exposes no statx UAPI field, and linked the pinned kernel implementation and UAPI sources.
---
FDU-PR3-R13. docs/project/research/research-2026-08-10-performance-frontier.md incorrectly treats STATX_CHANGE_COOKIE as Linux userspace UAPI. Correct the research to kernel-internal/future-only status and require exported UAPI plus filesystem evidence before planning against it. Review: https://github.com/jlevy/fdu/pull/3#issuecomment-5249058288.
