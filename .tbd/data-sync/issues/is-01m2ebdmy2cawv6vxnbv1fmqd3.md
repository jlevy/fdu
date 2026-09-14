---
type: is
id: is-01m2ebdmy2cawv6vxnbv1fmqd3
title: "PR #48 review CLASS-5: gitignore bracket expressions diverge from git"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:40:22.592Z
updated_at: 2026-09-14T03:10:16.797Z
closed_at: 2026-09-14T03:10:16.793Z
close_reason: |
  b8b2c92 + 855d300: class_match now follows wildmatch's loop. It handles negation, a leading ], escapes (including at either end of a range), a range start that is itself a member, and the 12 ASCII-only [:name:] classes with git's own space set. An unterminated expression or an unknown class name drops the line, since git's abort never matches. A / inside a class is a member, not a separator. normalize_glob copies classes unchanged. Remembering the close position of each [: keeps one evaluation linear. The table test has 63 pattern rows recorded from git check-ignore --no-index (git 2.50.1), and on unix it re-asks the host's git for each row. The module doc states the supported forms. 855d300 took a Windows drive-letter path out of the bound test. CI green: run 34801252954. The escaped-slash divergence is filed as fdu-bqan.
resolution: null
duplicate_of: null
---
Low. control/gitignore.rs:226-250. Checked against git check-ignore: [[:alpha:]], [a\-z], and [\]] answer differently; the module doc's 'character classes' overstates. Fix: support [:class:] and escapes inside classes, or document the unsupported forms. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
