---
type: is
id: is-01kzys9xn533vn2h4d9b3bfzpp
title: Content-metrics work class and code-counting peers in the tool comparison
kind: task
status: closed
priority: 2
version: 4
delegate: claude-code@vm
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-08-14T00:03:45.957Z
updated_at: 2026-09-29T19:48:56.575Z
started_at: 2026-09-29T19:48:52.044Z
closed_at: 2026-09-29T19:48:56.575Z
close_reason: Code-counting contracts and a line-count work class added to the tool comparison harness and measured on linux-v6.12 (fdu-bj94).
resolution: null
duplicate_of: null
---
Work classes are indexed-tree, indexed-summary, transient-summary, rendered-tree and total-only. There is no content-metrics class and no scc/tokei/cloc ToolContract, though the content research reviewed scc and tokei closely for their LOC conventions. The claim that fdu is fastest at counting lines of code has no measurement path.

## Notes

Done by fdu-bj94 on #162 (commit 5e5b5198): compare_tools.py has a code-by-language work class (and code-by-language-cached) with contracts for fdu --analyze=code, scc, and tokei in two ignore arms, plus a cached fdu contract; a 'measures' field keeps line counts from pairing with disk usage. Measured on linux-v6.12 2026-09-29: fdu is not the fastest cold counter (7.92 s vs scc 1.24 s, tokei 1.87 s, ignore rules off) but is fastest repeated from its content cache (0.55 s). See docs/project/research/research-2026-09-29-sloc-tools-survey.md; the cold gap is fdu-xpwv.
