---
type: is
id: is-01m0pt9he483bx4et2eykcdp1j
title: Runtime-supplied type registry, parsed and indexed in Rust
kind: feature
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T08:02:48.388Z
updated_at: 2026-08-23T21:33:30.291Z
closed_at: 2026-08-23T20:16:26.671Z
close_reason: "TypeRegistry is a typed engine value: parsed, validated, indexed and fingerprinted in Rust from the same [[kind]] dialect, accepted by OpenConfig/ScanConfig, the CLI's --type-rules on the Scope axis, and ScanOptions.type_rules in Python. build.rs include!s the crate's own parser so build time and run time read one implementation, proven by a migration test. type_rule_fingerprint reads the active registry; its three existing consumers already compare it, so a rule change invalidates snapshot and sidecar with no new plumbing (asserted end to end). PR #38's tie-break test is now a property over any registry; validation is tested for what it rejects. Compiled default stays the default and the fast path, holding Cow::Borrowed over the rendered statics."
duplicate_of: null
resolution: null
---
Today build.rs compiles the rules into OUT_DIR and the module states runtime never parses TOML, so a consumer wanting an unknown type or a different grouping must rebuild the crate or reclassify in its own language. Add a registry as a typed engine value built from the same [[kind]] dialect: parse, validate, index, group, and fingerprint all in Rust, accepted by OpenConfig, ScanOptions/AnalysisOptions, and the CLI scope axis. The compiled registry stays the default and the fast path — no configuration, no startup parse, CLI behavior unchanged. type_rules_fingerprint must read the active registry instead of a const fn; the invalidation plumbing already exists (carried in scan scope and content_model, already compared), so a rule change invalidates snapshot and sidecar by the mechanism the design principles already require for a bucketing change. SEQUENCING: lands after PR #38, converting its LazyLock<HashMap<&'static str, &'static GeneratedRule>> statics into a per-registry index — same index shape, same win, different lifetime — and generalizing its indexed_rule_tiers_agree_with_the_scan_they_replaced tie-break test into a property over any registry. Validation is tested for what it rejects: duplicate ids, unknown group, tie-break ambiguity, fingerprint collision with the default.

## Notes

SCOPE GROWS, and one earlier framing was wrong. Verified by running fdu: release.v2.zip already classifies as archive and buckets as .zip, and bundle.umd.min.js as javascript under .js -- which is what File Rollup Format wants canonically. The format has TWO levels: a raw logical extension (up to two eligible trailing components) AND a canonical suffix match that drives rule lookup and roll-up bucketing. fdu has only the canonical one. Do NOT simply change derive_ext to return the raw value: classify_path_with_prefix looks rules up by exact key in RULES_BY_EXTENSION with no suffix fallback, so key v2.zip misses every rule and the archive becomes unknown:.v2.zip, while ext_bucket (same function) splits the .zip bucket at the same time. Build the pair instead -- raw level exposed on entries and in navigation/literal-filter/recent/catalog projections, plus canonical suffix matching -- with a test pinning that no existing bucket or type row moves. Also: registry arrives as an immutable packet at open with identity echoed back and disagreement failing the open; the conformance packet is vendored at a reviewed metabrowser revision with local hash verification, and needs direct basename-to-logical-extension cases added before it can serve as the oracle.
