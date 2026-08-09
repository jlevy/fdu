---
type: is
id: is-01kzj8wf0jzt8asjpae7eynvp1
title: "PR #1 review C3: Relist only newly created watched directories"
kind: bug
status: closed
priority: 2
version: 2
labels:
  - pr-review
dependencies: []
parent_id: is-01kzj8v9cxyrx4z87g2gcw4z46
created_at: 2026-08-09T03:25:54.577Z
updated_at: 2026-08-09T03:55:26.695Z
closed_at: 2026-08-09T03:55:26.694Z
close_reason: Watcher coalescing now preserves create intent separately from generic verification; only directories with a coalesced create trigger WatchSetupRace relisting. Unit and backend watch tests pass.
---
PR #1 Cursor thread C3: https://github.com/jlevy/fdu/pull/1#discussion_r3742309829. File: crates/fdu/src/watch.rs. Preserve create-directory intent through event coalescing; routine metadata events on existing directories must not emit WatchSetupRace invalidations.
