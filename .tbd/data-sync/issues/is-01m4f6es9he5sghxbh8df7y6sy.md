---
type: is
id: is-01m4f6es9he5sghxbh8df7y6sy
title: Fit human reports to narrower terminals
kind: feature
status: open
priority: 2
version: 6
labels:
  - output
  - cli
dependencies: []
child_order_hints:
  - is-01m4f6f1yyff34njegjnnkpdd3
  - is-01m4fcy4xfyg2q85eraewkza5a
  - is-01m4fehxx7mx94dqxd8937kkca
  - is-01m4fehyd3bg51vvpnvdmh42je
  - is-01m4fg4mkdprja7axmjy54bzmf
created_at: 2026-10-09T02:04:15.013Z
updated_at: 2026-10-09T04:53:28.300Z
---
Maintainer, 2026-10-08: fdu would better fit its reports into narrower screens. The 0.4.0 demo at 140 columns showed DOCUMENTS rows of 154 characters wrapping mid-word; stacked grouped rows (see the child task) fix that. Remaining: audit every human view at 100 and 80 columns (tree, flat lists, CODE overview, the perf line at ~200 characters, notes and tips) and decide per view: stack, abbreviate, or adapt to the terminal width without making goldens width-dependent.
