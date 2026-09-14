---
type: is
id: is-01m2grj142r85szxdb4q4q6yqt
title: Decide what ignore facts an opened root reports in a build without the gitignore feature
kind: task
status: closed
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T20:08:26.497Z
updated_at: 2026-09-14T21:53:11.130Z
closed_at: 2026-09-14T21:53:11.128Z
close_reason: "Moot: 077318c (PR #60, fdu-x7yb) removed the gitignore build feature, so no build lacks ignore handling and every opened root observes control state. bb26446 reverted the crate-private accessor from 30c3895; the opened roll-up uses the checked partition_rollup_summary again."
resolution: canceled
duplicate_of: null
---
Found while finishing PR #57's review fixes (2026-09-14). The opened root sets read_controls: true, but a build without the gitignore feature reads no ignore rules, so ScanScope::observes_controls() is false (ignore_rules_fingerprint 0). After 53dc59a, public ignore accessors refuse on such an index (is_ignored, controls, partition_total, partition_rollup_summary). The opened roll-up projection keeps answering through a crate-private accessor, with the unignored partition equal to the whole subtree; see the follow-up commit on claude/contract-decisions. Without that, every featureless opened roll-up failed the whole read. Opened pages (EntryValue.ignored) likewise report false. Under the user's fdu-agb6 rule (never silently answer 'not ignored'), the consistent option is to make PartitionRollUpSummary.unignored and EntryValue.ignored Option in every build, None when no rules were observed, mirroring ChildSnapshot in 53dc59a. That is a further Rust and Python API change for a build shape MetaBrowser does not use. Decide whether a featureless build's 'no rules exist' counts as observed-empty, as now, or as not-observed.
