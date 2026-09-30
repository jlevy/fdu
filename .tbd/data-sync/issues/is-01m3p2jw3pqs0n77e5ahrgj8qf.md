---
type: is
id: is-01m3p2jw3pqs0n77e5ahrgj8qf
title: Match git on a .gitignore UTF-8 BOM and on a NUL byte inside a pattern line
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-29T07:55:33.877Z
updated_at: 2026-09-30T03:48:52.503Z
closed_at: 2026-09-30T03:48:52.503Z
close_reason: "Fixed in bbd8b48e: Gitignore::parse strips a leading UTF-8 BOM (only at file start) and Pattern::parse ends a pattern at the first NUL after the line's trailing CR is stripped and before trailing spaces are trimmed, the order git's C-string read gives. IGNORE_RULES_VERSION is 4 (doc comment says why). SOURCE_EDGE_CASES records 17 BOM/NUL cases the live oracle re-asks git 2.43 about; test a_byte_order_mark_and_nul_bytes_answer_as_git_check_ignore_does failed before the fix. CHANGELOG Fixed entry added. Parse-time only; no per-entry matching cost."
resolution: null
duplicate_of: null
---
Found by the 2026-09-29 matcher source review (fdu-fkyf), git side confirmed by running git 2.43.0: (1) git skips a UTF-8 byte-order mark at the start of a .gitignore; fdu does not, so the first rule of such a file never matches. (2) git truncates a pattern at a NUL byte inside a line; fdu keeps the rest of the line. Fix in Gitignore::parse (crates/fdu-core/src/control/gitignore.rs) with cases in the git check-ignore verdict table. Changing what a rule means may require bumping IGNORE_RULES_VERSION; decide with the maintainer, since the 0.2.2 plan's non-goals say the matcher answers exactly what it answers today. Not a performance item; keep it out of H171's diff so H171 stays answer-identical.
