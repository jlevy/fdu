---
type: is
id: is-01m0pt9he483bx4et2eykcdp1j
title: Runtime-supplied type registry, parsed and indexed in Rust
kind: feature
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T08:02:48.388Z
updated_at: 2026-08-23T08:02:48.388Z
---
Today build.rs compiles the rules into OUT_DIR and the module states runtime never parses TOML, so a consumer wanting an unknown type or a different grouping must rebuild the crate or reclassify in its own language. Add a registry as a typed engine value built from the same [[kind]] dialect: parse, validate, index, group, and fingerprint all in Rust, accepted by OpenConfig, ScanOptions/AnalysisOptions, and the CLI scope axis. The compiled registry stays the default and the fast path — no configuration, no startup parse, CLI behavior unchanged. type_rules_fingerprint must read the active registry instead of a const fn; the invalidation plumbing already exists (carried in scan scope and content_model, already compared), so a rule change invalidates snapshot and sidecar by the mechanism the design principles already require for a bucketing change. SEQUENCING: lands after PR #38, converting its LazyLock<HashMap<&'static str, &'static GeneratedRule>> statics into a per-registry index — same index shape, same win, different lifetime — and generalizing its indexed_rule_tiers_agree_with_the_scan_they_replaced tie-break test into a property over any registry. Validation is tested for what it rejects: duplicate ids, unknown group, tie-break ambiguity, fingerprint collision with the default.
