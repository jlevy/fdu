---
type: is
id: is-01m0rahh7entj80k486sxs5k45
title: "Two extension levels: raw logical extension plus canonical suffix matching"
kind: feature
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T22:06:01.965Z
updated_at: 2026-08-23T22:06:01.965Z
---
Split out of fdu-ctp5, which closed carrying the runtime registry only. The registry landed; this did not, and it is the half with a trap in it.

VERIFIED BY RUNNING fdu (fixture: release.v2.zip, bundle.umd.min.js, plain.zip, app.js, archive.tar.gz):
  --view types        -> archive 3 files, javascript 2 files
  --view extensions   -> .js 2, .tar.gz 1, .zip 2
So release.v2.zip already classifies as archive under .zip, and bundle.umd.min.js as javascript under .js. Those ARE the canonical answers File Rollup Format wants. fdu is not deriving the wrong thing -- it has one of the format's two levels, and the one it has is the canonical one.

DO NOT simply change derive_ext to return the raw value. classify_path_with_prefix looks rules up by EXACT key in RULES_BY_EXTENSION with no suffix fallback, so key v2.zip misses every rule and the archive becomes unknown:.v2.zip; ext_bucket wraps the same function, so the .zip rollup bucket splits at the same time. One edit, two regressions, in exactly the names the change is for.

BUILD THE PAIR:
  1. Raw logical extension -- up to two eligible trailing components per the format's rule, eligibility living in the rule dialect rather than a hand-maintained list (derive_ext's own comment asks for this). Exposed on entries and in the projections that want it: navigation tallies, literal filters, recent and catalog rows, unknown remaining_types keys.
  2. Canonical suffix matching -- a raw extension matching no rule falls back to its trailing component for BOTH rule lookup and the rollup bucket.

PROPERTY TO PIN: adopting the raw level moves no existing bucket and no existing type row. The fixture above is the regression test.

Also here: vendor the File Rollup conformance packet at a reviewed metabrowser revision, verify its manifest and hashes locally in CI (no network fetch, no sibling checkout), and run it against fdu's classifier. The packet needs direct basename-to-logical-extension cases added first -- today's matching-only cases pass against fdu's single level and hide the gap entirely. Ask for at least one name whose raw extension is also canonical (archive.tar.gz) so the fixture distinguishes correct fallback from never deriving a raw value.
