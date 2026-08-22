---
type: is
id: is-01m0m26p3jtm947t9grxnsrne8
title: Python View enum declaration order does not match Rust ViewSpec::ALL
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-08-22T06:23:20.177Z
updated_at: 2026-08-22T06:53:41.580Z
closed_at: 2026-08-22T06:53:41.579Z
close_reason: |-
  Fixed, with one correction to the bead's premise: the parity assertion in public_smoke already compared sequences (contract['views'] == [v.value for v in fdu.View]), so ordering was in scope all along. It passed anyway because contract() hard-coded its OWN copy of the view list in the same wrong order -- two copies of one mistake agreeing with each other.

  contract() now derives from ViewSpec::ALL plus 'full', so the list cannot drift and a new view needs no edit in lib.rs. The Python enum is reordered to match, with a docstring saying why the order is load-bearing.

  The remaining instance of the same pattern is the hand-copied 'full' diagnostic, tracked as fdu-gw5b.
---
The parity shim surfaced this on its first run.

Rust  ViewSpec::ALL: summary, tree, families, types, extensions, languages, documents, largest, recent, files
Python fdu.View:     tree, extensions, types, families, languages, documents, largest, recent, files, summary

Same members, different order. The existing StrEnum parity assertion in public_smoke compares membership as a set, so it has never noticed.

Order is observable: anything that iterates fdu.View -- an expectation clause built from the enum, a help string, a docs table, a caller iterating views in 'the documented order' -- gets a sequence the CLI never produces. The Rust order is deliberate (summary first, then the roll-up ladder, files last); the Python order looks like whatever the file was typed in.

Two fixes, both wanted: reorder the Python enum to match, and strengthen the parity assertion to compare sequences rather than sets so the next reorder is caught by the mechanism that is supposed to catch it.
