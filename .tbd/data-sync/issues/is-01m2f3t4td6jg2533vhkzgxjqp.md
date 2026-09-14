---
type: is
id: is-01m2f3t4td6jg2533vhkzgxjqp
title: Shared watch keeps a file deleted mid-walk as a phantom entry with permanent Partial freshness
kind: bug
status: open
priority: 2
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T04:46:37.900Z
updated_at: 2026-09-14T04:46:37.900Z
---
PR #48 delta review DELTA48-ENG-2 (https://github.com/jlevy/fdu/pull/48#pullrequestreview-5194005473). scan.rs:939-946, 3695-3704, 3721-3730; index.rs:1972-1973; watch.rs:416-417 at d48b8f8. baf6c00 (fdu-yq1v) made the shared watch target, used by fdu --watch and Python Index.watch() through watch_session.rs:127, settle a scan error instead of re-queuing it. A known file deleted between read_dir and lstat now stays as a phantom entry with permanent Partial freshness and no retry; before baf6c00 the next event healed it. The opened root has had the same behaviour since f276cb5, so baf6c00 extends the problem rather than introducing it. The file's own deletion event may still heal it on a live watcher; confirm that. Suggested fix covering both targets: on NotFound for a name read_dir just returned, push a conditional Remove instead of an error. Test: delete a child between listing and stat using the existing test hooks.
