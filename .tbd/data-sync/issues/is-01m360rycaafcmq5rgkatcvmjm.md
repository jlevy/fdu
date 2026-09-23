---
type: is
id: is-01m360rycaafcmq5rgkatcvmjm
title: Name type-rule mismatch in cache-only snapshot refusal
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels: []
dependencies: []
parent_id: is-01m2pmrbxvnerxyhnjhrswy1ye
created_at: 2026-09-23T02:16:04.745Z
updated_at: 2026-09-23T08:14:06.863Z
closed_at: 2026-09-23T08:14:06.863Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Cache design Known Gaps: a valid snapshot under another TypeRegistry currently reaches ParseError::Invalid before LoadOutcome::Refused, so cache-only says absent. Preserve full structural/checksum validation and root identity, refuse rather than serving foreign rules, and name the type-rule mismatch with a concrete recovery. Regression: write under a custom registry and read cache-only under the compiled registry.

## Notes

Implementation on codex/alpha-type-rules-refusal: a valid foreign registry snapshot becomes a validated Refused identity with root; direct load stays absent; cache-only names type rules after root precedence. Focused fresh fdu-core regression passed 1/1. Awaiting independent review and integration under execution Plan.
