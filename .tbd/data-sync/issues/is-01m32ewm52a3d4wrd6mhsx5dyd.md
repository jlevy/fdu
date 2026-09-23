---
type: is
id: is-01m32ewm52a3d4wrd6mhsx5dyd
title: The content tier decides reuse at five inline == sites and never touches Serves
kind: bug
status: closed
priority: 0
version: 5
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T17:05:47.681Z
updated_at: 2026-09-23T08:14:06.746Z
closed_at: 2026-09-23T08:14:06.746Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Verified by reading in an adversarial review, 2026-09-21. This is the finding that invalidates the claim that equality-serve makes content reuse correct by construction.

Content reuse is decided by five independent equality comparisons, none of which goes through `stored_state::Serves`:
- `content/content_index.rs:333` (`prepare`)
- `content/content_cache.rs:163` (`load_content_cache`)
- `content/content_cache.rs:513` (`parse_header`)
- `index.rs:3543` (`pending_analysis_candidates`)
- `content/content_index.rs:278` (`holds_record`, record level)
plus `query/query_request.rs:402` (`validate_read`).

#37 (`2aa7da1`), the commit that widened equality to containment and caused fdu-gija, touched `content_cache.rs`, `content_index.rs`, `content_model.rs`, `index.rs` and `query_report.rs` — none of which would have gone through `Serves`. So the enum would NOT have prevented #37 and does not prevent a recurrence today.

Confirmed constructively: a re-widening mutant needed three edits (`lib.rs:773`, `query_request.rs:402`, `query_report.rs:1054`) and never touched `stored_state.rs`.

Fix: route all six content sites through one `serves_content` returning a value-carrying projection, so a record set cannot be consumed without the projection that makes it answer the request.

Until this lands, describe fdu-gija as fixed-by-equality, not as correct by construction.

## Notes

Implementation in codex/alpha-content-admission: ContentTierIdentity::admit is the single borrowed identity/provenance relation, returning ContentAdmission. Its private constructors produce a ContentProjection for tier reads and AdmittedRecord for decoded records; restore cannot accept an unchecked FileAnalysis. Prepare, save/load/header parse, direct commit, pending candidates, read validation, and report metric reads use this relation. Equality-only projection remains deliberate; subset projection is deferred. for_request builds provenance once; per-record admission compares borrowed slices and introduces no allocations or cloning. Regressions cover every identity component, incompatible tiers and unit slots, forged provenance, and exact positive controls. Existing restore timing buckets preserved. Plan consumer/root export coordinated with execution owner. cargo fmt and diff check passed; Rust verification deferred to scheduled integration slot, so bead stays open. Delivery plan: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md.

Validation update: c2cfe480 and status follow-up e9c90ed7 integrated as54c4930a/174cb174. Parent independent Astra review found no findings. Execution owner ran content66, stored-state10, query103; all passed; workspace/all-build-features/all-targets clippy passed after mechanical lint fixes. No full gate claimed; publication pending execution layer. The shared relation is applied to cache records and to TreeStatus/ReportProvenance as well as metric reads.
