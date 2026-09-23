---
type: is
id: is-01m32wje089vk2cwd9ds2sms2d
title: Attribute one-shot scan errors per directory so --allow-partial rows are not all incomplete
kind: feature
status: closed
priority: 2
version: 6
spec_path: docs/project/specs/active/plan-2026-09-20-directory-query-formats.md
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
created_at: 2026-09-21T21:04:53.768Z
updated_at: 2026-09-23T08:14:06.843Z
closed_at: 2026-09-23T08:14:06.843Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Follow-up from the PR #96 R1 disposition (https://github.com/jlevy/fdu/pull/96#issuecomment-5765069350). A one-shot scan that finishes with errors records that the walk was partial (set_initial_freshness(false)) but not which directory each error fell in: ScanReport.errors are rendered to strings and never reach the Index, so under --allow-partial every directory row reports complete: false and an unknown age. Plumb the failing paths (ScanReport.errors carry them) into the index or Provenance so only the directories the errors fall in, and their ancestors, are marked incomplete; an opened root already gets this from children_complete. Acceptance: a fixture with one unreadable directory under --allow-partial marks that directory and its ancestors incomplete and leaves siblings complete.

## Notes

Implemented in codex/alpha-cold-partial-state from serving f2848e2d. Cold scan finalization consumes the complete normalized error list (not the bounded retained diagnostic list), leaves failed subtrees and ancestors partial, and marks retained healthy directory listings complete. Unscoped, root, or outside-root errors retain conservative whole-root partial state; descendants of a failed listing never become complete. The successful-scan fast path remains unchanged. Regression with real Unix permission precondition proves blocked child/root incomplete, healthy sibling+nested directory complete and fresh, unknown child absent, and metadata persistence refused; runs both detached and streaming test builders. Additional platform-independent regression guards unknown descendants and unscoped failure fallback. Fresh source/docs rebuild: cold_scan13 and partial19 tests passed; core all-build-features/all-targets Clippy clean. Independent review, forward integration, and full/cross-platform gate pending. Delivery plan docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md.

Publication: independent parent Astra review cleared52c7c70b. Cherry-picked as84de3685 and pushed to codex/alpha-serving-state, PR114, with parent authorization and branch-owner coordination. Final composed directory-list row regression remains pending; full gate/CI and merge still outstanding.

Final public typed-row acceptance: isolated 0c45859a integrated into composed surface as d70fde32. Public open plus query::report regression verifies a real unreadable child and its ancestor have complete=false and unknown age, healthy sibling has complete=true and known age, and both modification-bound directions retain only the healthy sibling. Actual PermissionDenied precondition is required unless the explicit existing permission opt-out is set. Answer agent ran the integrated test: 1 passed; query 112 and writer 38 passed. Stale subtree comments corrected; no production query change was needed because localized directory completeness already composes with complete-coverage fallback. Full gate, CI, and merge remain parent-owned and pending.
