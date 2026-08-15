---
type: is
id: is-01m01fpdn3qbdv0y2458brk4mw
title: Pin and validate the dust adapter for claim-grade comparisons
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - benchmark
  - validation
dependencies:
  - type: blocks
    target: is-01m01ed61j7yty2bqp0zw8v0xc
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T01:13:33.090Z
updated_at: 2026-08-15T11:06:43.721Z
closed_at: 2026-08-15T11:06:43.721Z
close_reason: Pinned and validated Homebrew dust 1.2.4 by source/formula/bottle/executable hashes, license, target, version, exact argv, allocated-byte and hard-link semantics, plus fail-closed warning/error/timeout/parser/oracle tests.
---
Make dust a claim-grade comparator rather than an identity-only timing command. Pin its source, revision, license, build recipe, version, executable hash, exact argv, minimal environment, and supported semantic contract; define and test parsing or postconditions for root totals, stderr, warnings, nonzero exits, timeouts, permission failures, and incomplete output. Document and either normalize or explicitly exclude semantic differences involving apparent versus allocated size, hard links, symlinks, mounts, and partial traversal.

Acceptance: the adapter consumes the claim-grade provenance manifest from fdu-849g; immutable fixtures prove fdu and dust perform equivalent complete work before a timing sample is valid; error-bearing, semantically unmatched, unpinned, or unparsable samples fail closed with null metrics plus a reason; unit and integration tests cover success and each invalidation class; the release matrix cannot call dust evidence claim-grade until this bead is complete.
