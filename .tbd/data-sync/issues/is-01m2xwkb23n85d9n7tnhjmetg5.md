---
type: is
id: is-01m2xwkb23n85d9n7tnhjmetg5
title: "H135: first-pass content-basic leftover after H124"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01m2x7xbt1c4we0wffeth7jrd6
created_at: 2026-09-19T22:29:11.362Z
updated_at: 2026-09-19T22:57:01.102Z
closed_at: 2026-09-19T22:57:01.101Z
close_reason: "exp-134: confirmed leftover. first-pass content-basic still file I/O (read 58.87%, open 16.09%). classify_with 2.02%. no skippable >=3% userspace cut. no engine patch. quiet 56.4%. pair uncontrolled."
resolution: null
duplicate_of: null
---
After H124, leftover on deciding-scale first-pass content-basic: whether apply-path classify / apply_analysis / candidate install is >=3% of wall and not already rejected.

Not content-cache-hit. Not H118. Not H119 walk-overlap. Not H124 type/size. Quiet once; if fail, uncontrolled. Do not compile a cut unless inventory names a skippable >=3% mechanism.

exp-134. Frozen metabrowser-clone.
