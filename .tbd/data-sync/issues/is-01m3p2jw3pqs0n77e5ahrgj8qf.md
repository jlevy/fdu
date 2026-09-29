---
type: is
id: is-01m3p2jw3pqs0n77e5ahrgj8qf
title: Match git on a .gitignore UTF-8 BOM and on a NUL byte inside a pattern line
kind: bug
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-29T07:55:33.877Z
updated_at: 2026-09-29T07:55:33.877Z
---
Found by the 2026-09-29 matcher source review (fdu-fkyf), git side confirmed by running git 2.43.0: (1) git skips a UTF-8 byte-order mark at the start of a .gitignore; fdu does not, so the first rule of such a file never matches. (2) git truncates a pattern at a NUL byte inside a line; fdu keeps the rest of the line. Fix in Gitignore::parse (crates/fdu-core/src/control/gitignore.rs) with cases in the git check-ignore verdict table. Changing what a rule means may require bumping IGNORE_RULES_VERSION; decide with the maintainer, since the 0.2.2 plan's non-goals say the matcher answers exactly what it answers today. Not a performance item; keep it out of H171's diff so H171 stays answer-identical.
