---
type: is
id: is-01m0nqzynp3vxhfn2rm0c43d8v
title: "Phase 2: the same parity shim over the public Rust library API"
kind: feature
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies: []
parent_id: is-01m0k965p7hx4dy6t0cj29rsae
created_at: 2026-08-22T22:03:22.676Z
updated_at: 2026-08-22T22:03:22.676Z
---
Phase 2 of docs/project/specs/active/plan-2026-08-21-fdu-python-cli-parity.md, which until now existed only as a spec checkbox and so was invisible to tbd ready.

Run the same golden corpus against a shim written on the public Rust library API, with its own deviation file.

Sharper than the Python one: if it cannot be written without reaching into cli.rs, then 'the CLI invents nothing' is false and the library is missing something.

Phase 1 is the argument for doing it. It found seven definitions the CLI had copied from the library -- ALL_VIEWS, view_is_satisfiable, default_view_for, parse_view, view_label, the full-exclusivity message, and the watch-scope guidance -- and five capabilities only the CLI could reach: cache-status rendering, watch record rendering, a watch's live report, the one-shot execution contract, and the omission note. Every one was found by hand, through a second language. A Rust-side shim would have failed on each of them directly.
