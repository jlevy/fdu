---
type: is
id: is-01m0nbv330xhagsyc2ehtm7hha
title: Relabeling a diagnostic by string replace corrupts the user's own token
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m0n9pjsahc4wk6ek37syjct4
created_at: 2026-08-22T18:31:00.447Z
updated_at: 2026-08-22T18:53:16.267Z
closed_at: 2026-08-22T18:53:16.266Z
close_reason: Label is a parameter now; nothing rewrites the message afterwards. Regression test covers the tokens that used to be corrupted.
---
The library's analyze grammar formats its message as 'invalid analyze ...', and the CLI turns that into its own flag spelling with a blind substring replace:

  AnalysisSet::parse(&self.analyze).map_err(|m| anyhow!(m.replace("analyze", "--analyze")))

The replace hits every occurrence, including the one inside the value the user typed:

  $ fdu --analyze analyzer /tmp
  fdu: invalid --analyze "--analyzer": expected one of none, lines, code, words, all

  $ fdu --analyze reanalyze /tmp
  fdu: invalid --analyze "re--analyze": ...

So the diagnostic misquotes the input it is complaining about, which is the one thing an invalid-value message has to get right.

Fix by passing the label into the parser rather than rewriting its output afterwards. That also gives the view list grammar somewhere to live: --view's duplicate and empty-entry checks are still in cli.rs (fdu-jozr), and moving them needs the same label parameter.
