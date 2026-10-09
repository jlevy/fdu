---
type: is
id: is-01m4fn9ntt9xy415w7bb35xy86
title: release-demo can run while the announce job is live
kind: task
status: open
priority: 2
version: 1
labels:
  - release
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
created_at: 2026-10-09T06:23:36.275Z
updated_at: 2026-10-09T06:23:36.275Z
---
Review B B1 on #189 (https://github.com/jlevy/fdu/pull/189#issuecomment-6075575482): nothing stops make release-demo after approval, risking two drafts or an in-flight upload on a release that then becomes immutable. Say in step 8 never to run step 7 while Announce runs, document the duplicate-draft recovery, and optionally refuse while a publishing run on the tag is in progress.
