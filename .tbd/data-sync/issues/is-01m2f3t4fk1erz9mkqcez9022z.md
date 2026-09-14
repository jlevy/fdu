---
type: is
id: is-01m2f3t4fk1erz9mkqcez9022z
title: Refresh and observer passes skip marking directories complete after one transient child error
kind: bug
status: open
priority: 2
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T04:46:37.554Z
updated_at: 2026-09-14T04:46:37.554Z
---
PR #48 delta review DELTA48-ENG-1 (https://github.com/jlevy/fdu/pull/48#pullrequestreview-5194005473). scan.rs:3699-3704, 3738-3740, 812-814, 3448 and index.rs:1938-1973 at d48b8f8. b803b8e (FIX48-1) marks a directory complete only when the whole reconcile pass had no errors, whereas discovery decides per directory. So one transient child-metadata error, such as a file vanishing between read_dir and lstat, keeps every directory that pass listed from being marked complete. A directory first listed by such a pass stays Unknown { Building } forever under a root reporting Complete. The handoff case is safe; refresh and observer passes are not. Fix: record completeness per listed directory, excluding only directories whose own listing or child reads failed. Test with an injected per-child error.
