---
type: is
id: is-01m0m3fa9ptd8m5etrb26kfrfy
title: Python API cannot parse the CLI's list grammar (duplicates, empty entries)
kind: feature
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-08-22T06:45:31.573Z
updated_at: 2026-08-22T06:45:31.573Z
---
The parity shim has to accept --view tree,tree and --view tree,,types because the list-level grammar lives in cli.rs, not in the library or the binding.

The CLI rejects both:
  fdu: invalid --view "tree,tree": "tree" appears more than once
  fdu: invalid --view "tree,,types": empty entry in the list

A Python caller building views from a user-supplied string has no way to get those checks, and the shim reimplementing them would be exactly the drift the harness exists to catch, so it deliberately does not.

Expose the list parser (view and analyze) so both surfaces share one grammar.
