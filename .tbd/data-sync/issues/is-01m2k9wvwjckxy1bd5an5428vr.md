---
type: is
id: is-01m2k9wvwjckxy1bd5an5428vr
title: Control files under already-ignored directories are read and charged against the control budget
kind: task
status: open
priority: 2
version: 1
labels:
  - stack-followup
  - control-state
  - scale
dependencies: []
created_at: 2026-09-15T19:49:56.240Z
updated_at: 2026-09-15T19:49:56.240Z
---
The cold walk reads and retains every `.gitignore`, including one inside a directory an ancestor's rules already ignore, which git never reads. Classification is still right, because `parent_ignored` short-circuits matching (`crates/fdu-core/src/control.rs` `ControlMatcher::is_ignored`, and `DetachedIndexBuilder::push_directory` keeps `parent_ignored => ignored`), but each such file is charged against the control budget.

Found while closing fdu-1onj, whose notes listed it as residual (4); PR A (branch `claude/control-bounds-degrade`) made the budget degrade and shared identical contents, and did not change which files are read.

Why it matters now: package trees put most of their `.gitignore` files under directories the root ignores (`node_modules/`, vendored checkouts). Over `~/wrk` on 2026-09-15, 4,830 files with 980 distinct contents still charge 4.10 MiB after sharing, past the 4 MiB default, so a default-on scan (fdu-elnn) refuses some and prints a note about rules that could never change any classification.

Direction: skip reading and retaining a control source whose governing directory is already ignored, and re-read it when that directory becomes unignored (a control edit that reclassifies it). The re-read on reclassification is the part that needs design: the streaming path (`Index::reclassify_controlled_subtrees`) would have to request control reads, or the table would retain a dormant record.

Acceptance: a `.gitignore` under an ignored directory costs no budget and produces no refusal; unignoring that directory applies its rules without a rescan of unrelated subtrees; the synthetic 1,105-directory test and the watched-root test in PR A still pass.
