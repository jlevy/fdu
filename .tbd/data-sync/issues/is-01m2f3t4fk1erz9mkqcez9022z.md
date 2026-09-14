---
type: is
id: is-01m2f3t4fk1erz9mkqcez9022z
title: Refresh and observer passes skip marking directories complete after one transient child error
kind: bug
status: closed
priority: 2
version: 2
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T04:46:37.554Z
updated_at: 2026-09-14T15:33:33.390Z
closed_at: 2026-09-14T15:33:33.389Z
close_reason: "33853ce: the walk names a directory only when no error was pushed while processing that directory; ReconcileReport::take_recordable_completeness withholds all of them when a commit lost a race or was refused; finish_reconcile records them whether or not the pass completed, deciding before its own Partial mark. Test injects one child metadata error under a complete directory while a new directory is listed for the first time; it is recorded complete and answers Absent below."
resolution: null
duplicate_of: null
---
PR #48 delta review DELTA48-ENG-1 (https://github.com/jlevy/fdu/pull/48#pullrequestreview-5194005473). scan.rs:3699-3704, 3738-3740, 812-814, 3448 and index.rs:1938-1973 at d48b8f8. b803b8e (FIX48-1) marks a directory complete only when the whole reconcile pass had no errors, whereas discovery decides per directory. So one transient child-metadata error, such as a file vanishing between read_dir and lstat, keeps every directory that pass listed from being marked complete. A directory first listed by such a pass stays Unknown { Building } forever under a root reporting Complete. The handoff case is safe; refresh and observer passes are not. Fix: record completeness per listed directory, excluding only directories whose own listing or child reads failed. Test with an injected per-child error.
