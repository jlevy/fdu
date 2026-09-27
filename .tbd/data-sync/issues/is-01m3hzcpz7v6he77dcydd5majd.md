---
type: is
id: is-01m3hzcpz7v6he77dcydd5majd
title: Correct shell token context in source-line counting
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m3hz327rqpdxwdh0f5e0hyj5
created_at: 2026-09-27T17:42:48.548Z
updated_at: 2026-09-27T21:17:17.711Z
closed_at: 2026-09-27T21:17:17.711Z
close_reason: "Implemented and reviewed in PR #133, restacked through #136. Every local handoff target passed across the full gate and authoritative Linux parity rerun. All 19 checks passed on each updated PR layer; implementation macOS required one unchanged-code retry, tracked separately as fdu-21ns. Installed clean candidate and bundled skill verified; manual acceptance remainder stays under fdu-kwjc."
resolution: null
duplicate_of: null
---
Full stack review S1 (Medium), owning PR133 at44f106ae/top48248a8. Valid Bash true;# "unterminated followed by a standalone comment and printf ok counts 3 code/0 comments instead of2/1 because # recognition needs whitespace and the quote poisons state. Valid N=2; newline x=$((1<<N)); newline # following similarly counts3/0 instead of2/1 because arithmetic <<N starts heredoc. Source content_code_metrics.rs:443-445,635-637,759-787. bash -n accepted both; retained prior gate binary91c8f93f0.dirty reproduced, source independently traced. Track command-separator comment boundaries and arithmetic-vs-heredoc context; add hand-counted fixtures across chunk splits and preserve existing heredoc tests. No fix applied during review.
