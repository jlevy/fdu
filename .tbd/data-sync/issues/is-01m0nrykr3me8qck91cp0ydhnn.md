---
type: is
id: is-01m0nrykr3me8qck91cp0ydhnn
title: "Spec: the command line on the public API"
kind: epic
status: closed
priority: 1
version: 12
spec_path: docs/project/specs/done/plan-2026-08-22-fdu-cli-on-the-public-api.md
labels: []
dependencies: []
child_order_hints:
  - is-01m0nrz70pggw1yya4tkhh40t3
  - is-01m0nrz7avvrxhxyt3mhgsd63z
  - is-01m0nrz7n0nfyr1v7vd8g8x8av
  - is-01m0nrzvs7hkqmgn3wmxh303zx
  - is-01m0nrzw3n05db3memm9sg4hwh
  - is-01m0nrzwe28fz1esdk2nnxd0eg
  - is-01m0ns0gsj8g96exsc3ndt45bg
  - is-01m0ns0h494z78b7d42fhby8ge
  - is-01m0ns0heyrk7hf6qh7bxzng2p
created_at: 2026-08-22T22:20:07.298Z
updated_at: 2026-08-22T22:55:50.105Z
---
Move the command line into its own crate, depending on fdu the way any other consumer does, so 'the CLI invents nothing' is enforced by the compiler rather than asserted by review.

Replaces Phase 2 of the Python CLI parity plan (fdu-esmm), which proposed a test-only Rust shim run against the golden corpus. That was wrong: the CLI already IS the Rust library's consumer, in the same language against the same API calling the same renderer, so a second one produces an EMPTY deviation file -- which the harness reads as 'the shim never ran', because an empty diff is what a fallthrough produces. It would also be permanent dead weight, a second command line maintained forever to assert it matches the first.

The question it was reaching for is right and is answerable directly. A crate boundary decides it: if the CLI lives in crates/fdu-cli and depends on fdu as an ordinary dependency, crate:: does not resolve and private items are unreachable, so the compiler answers on every build.

The audit says the answer is almost yes. Every path cli.rs reaches for was enumerated; all but three resolve to already-public items and are simply spelled crate:: because the CLI lives inside the crate. The three exceptions are human_bytes, human_count, and prepare_report_with_scan_diagnostics.

The 129 goldens are the parity proof, unchanged. Byte-identical output is the pass condition.

## Notes

Landed. The command line lives in crates/fdu-cli and is built entirely on fdu's public API: cli.rs has zero crate:: paths, the library has no cli module or feature and no clap or anyhow, and make lib-only fails if either returns. All 129 goldens byte-identical, none regenerated.

Spec moved to docs/project/specs/done/plan-2026-08-22-fdu-cli-on-the-public-api.md.

Every child is closed and the spec has no unchecked items -- both checked before closing, because bead state alone does not prove an epic is done.
