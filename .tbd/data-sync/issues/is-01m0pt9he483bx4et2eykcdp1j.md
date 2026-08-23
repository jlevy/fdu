---
type: is
id: is-01m0pt9he483bx4et2eykcdp1j
title: Runtime-supplied type registry, parsed and indexed in Rust
kind: feature
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T08:02:48.388Z
updated_at: 2026-08-23T17:01:29.535Z
---
Today build.rs compiles the rules into OUT_DIR and the module states runtime never parses TOML, so a consumer wanting an unknown type or a different grouping must rebuild the crate or reclassify in its own language. Add a registry as a typed engine value built from the same [[kind]] dialect: parse, validate, index, group, and fingerprint all in Rust, accepted by OpenConfig, ScanOptions/AnalysisOptions, and the CLI scope axis. The compiled registry stays the default and the fast path — no configuration, no startup parse, CLI behavior unchanged. type_rules_fingerprint must read the active registry instead of a const fn; the invalidation plumbing already exists (carried in scan scope and content_model, already compared), so a rule change invalidates snapshot and sidecar by the mechanism the design principles already require for a bucketing change. SEQUENCING: lands after PR #38, converting its LazyLock<HashMap<&'static str, &'static GeneratedRule>> statics into a per-registry index — same index shape, same win, different lifetime — and generalizing its indexed_rule_tiers_agree_with_the_scan_they_replaced tie-break test into a property over any registry. Validation is tested for what it rejects: duplicate ids, unknown group, tie-break ambiguity, fingerprint collision with the default.

## Notes

IMPLEMENTATION MAP. Today: build.rs:217 renders TYPE_RULE_FINGERPRINT and the rule table into OUT_DIR, included at classify.rs:167; RULES_BY_FILENAME/RULES_BY_EXTENSION (classify.rs:181-184) index it into LazyLock statics over &'static data. THE REGISTRY BECOMES A VALUE: GeneratedRule's &'static str fields become owned; the two statics become fields on a TypeRegistry built by the same index_rules (classify.rs:186) — same algorithm, same tie-break, different lifetime; classify_path_with_prefix (classify.rs:268) takes &TypeRegistry with a default-registry wrapper keeping every current call site working; type_rule_fingerprint() (classify.rs:204) stops being a const fn over a compiled constant and reads the active registry. ITS THREE CONSUMERS ALREADY COMPARE IT so invalidation needs no new plumbing: scan.rs:203, content_model.rs:245, and the freshness comparison at content_model.rs:263. The compiled registry stays the default and the fast path — no file to find, no startup parse, CLI behaviour unchanged. TESTS: the differential is the important one and generalizes a test PR #38 already wrote — indexed_rule_tiers_agree_with_the_scan_they_replaced pins max_by_key's last-wins tie-break over every key the table declares; that becomes a property over ANY registry, plus a migration assertion that the runtime-parsed default classifies byte-identically to the compiled one over the same key set. Validation tested for what it REJECTS: duplicate ids, unknown group, tie-break ambiguity, a fingerprint colliding with the default. OPEN: owned strings cost something against &'static; the measurement is a loop job on PR #38's own subject and decides whether the compiled default keeps a specialized path.
