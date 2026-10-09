---
type: is
id: is-01m4fehxx7mx94dqxd8937kkca
title: display_width counts some wide code points as narrow (e.g. U+2705) and some narrow as wide
kind: bug
status: open
priority: 3
version: 1
labels:
  - output
dependencies: []
parent_id: is-01m4f6es9he5sghxbh8df7y6sy
created_at: 2026-10-09T04:25:46.663Z
updated_at: 2026-10-09T04:25:46.663Z
---
Review A A3 on #187 (https://github.com/jlevy/fdu/pull/187#issuecomment-6074080548), predates it: crates/fdu-core/src/report_format.rs display_width (~1486-1514) hard-codes ranges; against Unicode 15.1, 8,267 wide code points fall outside them and 759 narrow ones in U+1F300-1FAFF count as 2. A row labelled with an emoji such as f.✅ misaligns its file count by one column and its stacked continuations sit one column off. Fix: a const table generated from EastAsianWidth.txt, no new dependency.
