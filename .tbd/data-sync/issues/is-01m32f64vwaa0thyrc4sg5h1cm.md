---
type: is
id: is-01m32f64vwaa0thyrc4sg5h1cm
title: Content-sidecar identity encodes the analyzer set three times, masking partial relaxations
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T17:10:59.707Z
updated_at: 2026-09-21T17:10:59.707Z
---
Verified by execution, 2026-09-21.

A containment mutant in `content_cache.rs::parse_header:513` (accept a stored analyzer set that is a superset over the same entry tier, decode under the stored profile, retag each record before `apply_restored_analysis`) is caught — four tests fail. But two earlier variants that kept `options_fingerprint` equality were INERT: 748 tests passed.

Reason: `AnalysisRequest::options_fingerprint` (`content_model.rs:215`) hashes only `profile.bits()`, so the identity encodes the analyzer set three separate times — in `analysis`, in `options_fingerprint`, and in `provenance.analyzers`. A partial relaxation of one field is masked by the other two.

That redundancy is coincidental, not designed. It currently provides defence in depth; it equally means a future change that relaxes all three consistently has no independent check, and that nobody can tell which field is load-bearing.

Related shape: every guard that catches the containment mutant is a hit/miss/applied COUNT — `a_wider_sidecar_is_a_clean_miss_for_a_narrower_request` (`:1002`), `a_different_analyzer_set_replaces_the_sidecar` (`:1049`), `an_analyzer_version_change_invalidates_records` (`:1203`), `lib.rs:1474`. None asserts the metrics a narrower request should produce after a wider sidecar, which is the thing fdu-gija actually got wrong. Add that assertion.
