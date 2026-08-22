---
type: is
id: is-01m0nqzynp3vxhfn2rm0c43d8v
title: "Phase 2: the same parity shim over the public Rust library API"
kind: feature
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies: []
parent_id: is-01m0k965p7hx4dy6t0cj29rsae
created_at: 2026-08-22T22:03:22.676Z
updated_at: 2026-08-22T22:21:29.836Z
---
Phase 2 of docs/project/specs/active/plan-2026-08-21-fdu-python-cli-parity.md, which until now existed only as a spec checkbox and so was invisible to tbd ready.

Run the same golden corpus against a shim written on the public Rust library API, with its own deviation file.

Sharper than the Python one: if it cannot be written without reaching into cli.rs, then 'the CLI invents nothing' is false and the library is missing something.

Phase 1 is the argument for doing it. It found seven definitions the CLI had copied from the library -- ALL_VIEWS, view_is_satisfiable, default_view_for, parse_view, view_label, the full-exclusivity message, and the watch-scope guidance -- and five capabilities only the CLI could reach: cache-status rendering, watch record rendering, a watch's live report, the one-shot execution contract, and the omission note. Every one was found by hand, through a second language. A Rust-side shim would have failed on each of them directly.

## Notes

Superseded by fdu-s74c, which reframes this work rather than doing it.

This bead proposed a test-only Rust shim over the public library API, run against the golden corpus with its own deviation file. It does not survive scrutiny: the command line already IS the Rust library's consumer, in the same language against the same API calling the same renderer, so a second one produces an EMPTY deviation file -- which the parity harness reads as 'the shim never ran', because an empty diff is exactly what a fallthrough produces. The design did not fit what it was pointed at. It would also be permanent dead weight: a second command line, maintained forever, whose only output asserts it matches the first.

The question it was reaching for -- can everything the CLI does be done through the public API? -- is right, and is decided by a crate boundary rather than sampled by a test. See fdu-s74c and docs/project/specs/active/plan-2026-08-22-fdu-cli-on-the-public-api.md.

The audit that motivated the reframe: every path cli.rs reaches for was enumerated, and all but three are already public. The CLI simply spells them crate:: because it lives inside the crate.
