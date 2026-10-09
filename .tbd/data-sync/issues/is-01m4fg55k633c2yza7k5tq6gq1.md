---
type: is
id: is-01m4fg55k633c2yza7k5tq6gq1
title: Files sorted by document_words rank C headers by document words
kind: bug
status: open
priority: 3
version: 1
labels:
  - output
dependencies: []
created_at: 2026-10-09T04:53:45.701Z
updated_at: 2026-10-09T04:53:45.701Z
---
Found by review A on #188: on the Linux v7.3-rc6 tree, 'fdu --view files --sort document_words' lists dcn_3_2_0_sh_mask.h (c, extension) at 2,804,467 document words. A code file should not rank under a documents metric; decide whether the sort restricts to document families or reports the metric as unavailable for code.
