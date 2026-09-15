---
type: is
id: is-01m2hsj3b876x988kevmjygg05
title: "PR #60 review PR60-DOCS-4: CHANGELOG removal entry omits that a default-on scope now observes control state"
kind: bug
status: closed
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2h3xgn1f3azaqbcvh9mzxas
created_at: 2026-09-15T05:45:11.782Z
updated_at: 2026-09-15T05:46:14.347Z
closed_at: 2026-09-15T05:46:14.345Z
close_reason: "ebea8eb (PR #60): CHANGELOG removal entry lists the third change (a default-on scope now observes control state: accessors answer, ChildSnapshot carries the ignore bit and partitions, every .gitignore is read, control bounds reachable); scaffold entry says build feature watch and names the CLI as the fdu crate. Reply: https://github.com/jlevy/fdu/pull/60#issuecomment-5675379029. CI 19/19 at ca9264f, CLEAN."
resolution: null
duplicate_of: null
---
Delta review 5204300906 on PR #60 at 3a4d384: CHANGELOG.md:124-129 says a consumer that built without the gitignore build feature 'sees two changes' (control input, fingerprint) but omits the most visible one: with read_controls on (the default) the index now observes control state, the accessors answer, ChildSnapshot carries the ignore bit and partitions, every .gitignore is read, and the control bounds are reachable. Out-of-delta: CHANGELOG.md:31,37 'feature watch' and 'feature cli' (no cli build feature exists).
