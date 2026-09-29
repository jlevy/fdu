---
type: is
id: is-01m3pk8qatb7mg2mnx10fenwkc
title: "H183: matcher residual pre-checks without per-window memcmp (per-entry byte-set prefilter, inline first/last byte checks)"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
created_at: 2026-09-29T12:47:07.098Z
updated_at: 2026-09-29T12:47:07.098Z
---
Callgrind of the default tree at 217861c1 on linux-v6.12: consumer 436M instructions with .gitignore vs 103M without; memcmp 114M and Checks::admit ~97M (gitignore.rs ~522-535: starts_with/ends_with slice equality and contains() calling memcmp per window, for each residual wildcard rule on every entry). Fix: compute a 256-bit byte-presence set of the name once per entry (shared across sources and rules); precompute each Checks' required-byte set (prefix, suffix, inner run bytes) and reject when missing; compare first/last byte inline before full compare; search the inner run by its first byte with an inline loop. Necessary conditions only, so answers are unchanged; H171's property test vs the linear matcher guards it. Pre-registered in the registry row: default-tree -3% on linux-v6.12 at 20 pairs, placebos --no-controls and node-modules-dense, consumer Ir -150M. Opus implements and reviews.
