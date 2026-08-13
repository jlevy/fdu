---
type: is
id: is-01kzwewqvqrt0sa5h4j4p3bmpa
title: Codify the repository workspace as the canonical 1M-entry live benchmark tree
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - docs
  - benchmark
dependencies: []
parent_id: is-01kzw3t92p7d4512h8vn6ktch1
child_order_hints:
  - is-01kzwjg104rec7p729neyra7bj
created_at: 2026-08-13T02:23:19.414Z
updated_at: 2026-08-13T05:51:08.321Z
closed_at: 2026-08-13T05:51:08.320Z
close_reason: Documented the self-contained repository workspace as the standard million-scale heterogeneous testbed, with frozen-writer workflow, redacted fingerprint, exact count, and generated-corpus distinction.
---
Verify the checkout is genuinely 1M+ entries and document it as the canonical self-contained heterogeneous live benchmark testbed. Require an exact pre/post tree identity, no writes or builds during measurement, a redacted root identifier, and distinction from the deterministic generated corpus. Add a convenient documented workflow for future runs.
