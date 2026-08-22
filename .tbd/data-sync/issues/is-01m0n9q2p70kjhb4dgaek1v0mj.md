---
type: is
id: is-01m0n9q2p70kjhb4dgaek1v0mj
title: Move the tree depth default out of the CLI into the library
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m0n9pjsahc4wk6ek37syjct4
created_at: 2026-08-22T17:53:51.814Z
updated_at: 2026-08-22T18:10:33.666Z
closed_at: 2026-08-22T18:10:33.665Z
close_reason: |-
  Selection.depth is now Option<Bound>, on the same terms limit and sort already used, and ViewSpec::default_depth supplies 2 for the tree and unbounded elsewhere. The CLI no longer declares default_value = "2"; it states the default in help text instead, as [tree default: 2], because it is the view's default and not the flag's.

  Verified: the CLI renders the same tree as before, and the Python surface now renders it identically where it previously emitted an extra directory level. A regression test asserts both halves -- that the CLI parses to None, and that the library turns None into 2 for the tree and All for files.
---
The CLI declares default_value = "2" for --depth at cli.rs:355. Selection.depth is a plain Bound whose Default is unbounded, so a Python caller who leaves depth unset gets a deeper tree than the identical CLI invocation.

Confirmed against the same tree: the CLI renders 9 rows, the Python surface 8 -- Python emits a fourth directory level the CLI stops before. It is a silent difference in the most common view there is.

Selection.limit and Selection.sort already solved exactly this, and their doc comments say why: 'a bound that suits a per-directory tree is not the bound that suits a complete enumeration'. depth should follow the same shape -- Option<Bound>, None meaning the view applies its own default -- so the CLI stops declaring it and every surface inherits it.

This is Principle 7 (the CLI invents nothing) and the same root cause as fdu-ggux and fdu-gw5b.
