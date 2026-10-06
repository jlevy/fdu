---
type: is
id: is-01m48ym5fj8g78hpmnfq3at7fp
title: "PR #174 B3: golden-restore-patterns exits 0 for a missing file argument (golden-restore-patterns.mjs:155-160)"
kind: bug
status: closed
priority: 3
version: 4
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m48ykwg1fz7tsw30h6xfhtmd
hold: null
hold_until: null
created_at: 2026-10-06T15:51:56.145Z
updated_at: 2026-10-06T16:32:44.181Z
started_at: 2026-10-06T15:52:01.207Z
closed_at: 2026-10-06T16:32:44.180Z
close_reason: "fixed in ac463a73 (local fix-174-B): exit 2 for missing/outside file, skip only when cat-file -e fails; spawn and scratch-repo tests"
resolution: null
duplicate_of: null
---
Low. scripts/golden-restore-patterns.mjs:155-160 catch{continue} treats every git show failure as new-in-change; a missing file argument prints '0 lines restored' and exits 0. Fix: exit 2 for a named file missing from the working tree; skip only when 'git cat-file -e <base>:<path>' fails; add exit-code tests (--base nope, bare --base, missing file) to scripts/golden-restore-patterns.test.mjs. PR #174, review B: https://github.com/jlevy/fdu/pull/174#issuecomment-6020029670

## Notes

Fixed in ac463a73: exit 2 for missing/outside file; skip only when git cat-file -e fails; spawn tests for 3 exit-2 cases plus scratch-repo test.
