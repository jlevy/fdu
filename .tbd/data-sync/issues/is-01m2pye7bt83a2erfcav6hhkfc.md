---
type: is
id: is-01m2pye7bt83a2erfcav6hhkfc
title: "P1.4.6: Per-tier content provenance (TierProvenance.content)"
kind: task
status: closed
priority: 0
version: 6
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - core-models
dependencies: []
parent_id: is-01m2pmra8yqrcxg27kc6ezg9vd
hold: null
hold_until: null
created_at: 2026-09-17T05:46:39.865Z
updated_at: 2026-09-23T08:14:06.552Z
started_at: 2026-09-20T04:40:17.369Z
closed_at: 2026-09-23T08:14:06.552Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 4: Provenance and Tree Status", commit 6. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `TierProvenance { entries: TierState, content: Option<TierState> }` with `TierState { source, freshness, observed_at_ns }`, computed for the content tier from its store identity.

**Tests**

- Content freshness is stale under cache-only.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

## Notes

Confirmed integration defect at 9e96e850: analyze a file, mutate it, metadata-reconcile, then report. Content profile remains but invalidated record is absent; report still says complete=true and content freshness Fresh. ContentIndex::invalidate does not dirty content state and status only recognizes explicit failures. Public regression source: /private/tmp/fdu-state-review.0etO0q/crates/fdu-core/tests/review_state_proof.rs. Missing eligible records must prevent fresh/complete content claims until reanalysis; preserve per-unit nonoperational coverage distinctions.

2026-09-20 implementation on codex/release-state-transitions: TreeStatus and ReportProvenance detect pending eligible content from current regular files versus retained coverage records. Mutation and addition now report incomplete/Partial until analysis; pure deletion stays complete; metadata-only status ignores the held content gap; pending work creates no fake I/O Issue. Existing nonoperational per-file coverage records remain unchanged.
