---
type: is
id: is-01m3jkx8psq29fp5kza3y15kxj
title: Make deep-render stack test exercise the full tree and diagnose Windows failure
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-09-27T23:41:22.509Z
updated_at: 2026-09-27T23:56:47.120Z
---

## Notes

Corrected fixture explicitly disables all display bounds and independently asserts 1,025 nodes/no omissions. Passed macOS64KiB and Windows CI run36359789536 core875passed. Original intermittent failure did not reproduce; phase markers retained without claiming an unproven cause. See tally arithmetic review.
