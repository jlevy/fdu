---
type: is
id: is-01m2gy094w60t8v29sbbh8c2ez
title: ChildSnapshot.ignored Option and partitions None conflate 'a file' with 'not observed'
kind: task
status: open
priority: 3
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T21:43:36.326Z
updated_at: 2026-09-14T21:43:36.326Z
---
Guideline conformance review of PR #57, finding 3 (https://github.com/jlevy/fdu/pull/57#pullrequestreview-5203155952). Sites: index.rs:698, 704 and 1268 at 30c3895. rust-rules says to replace sentinels with an enum. partitions: None now means both 'this child is a file' and 'the index did not observe control state', and ignored: Option<bool> means unobserved. Consider an explicit enum (Observed{ignored, partitions} / NotObserved) so a caller cannot read one meaning as the other.
