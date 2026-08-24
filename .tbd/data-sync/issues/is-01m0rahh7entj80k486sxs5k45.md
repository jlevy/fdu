---
type: is
id: is-01m0rahh7entj80k486sxs5k45
title: "Two extension levels: raw logical extension plus canonical suffix matching"
kind: feature
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
child_order_hints:
  - is-01m0rdxwt5fx2h2c0j3wzek4n7
created_at: 2026-08-23T22:06:01.965Z
updated_at: 2026-08-23T23:05:27.832Z
closed_at: 2026-08-23T23:05:27.832Z
close_reason: |-
  Both levels land, and the trap the bead named is defused rather than stepped in.

  logical_ext (renamed from derive_ext) now implements the format's derivation rule
  directly: up to two trailing dotted components, each nonempty, ASCII-alphanumeric and at
  most twelve units, a leading dot treated as part of the basename, and no extension at all
  when the final component is ineligible. The hardcoded .tar fold is gone -- it was a
  special case of the general rule, exactly as derive_ext's own comment had asked.

  TypeRegistry::canonical_ext is the second level: the logical extension when some rule
  claims it, otherwise its trailing component. classify_with looks rules up at the
  canonical level and ext_bucket buckets at it, so release.v2.zip still classifies as
  archive and still lands on the .zip pile. The unknown: type id and the row-level
  extension fields use the logical level, which is what a person reading a listing wants
  and what the format's remaining_types keys are.

  PROPERTY PINNED, exactly as the bead asked. Ran fdu against its fixture
  (release.v2.zip, bundle.umd.min.js, plain.zip, app.js, archive.tar.gz) before and after:
  --view types gives archive 3 / javascript 2 both times, --view extensions gives
  .js 2, .tar.gz 1, .zip 2 both times. Byte-identical. Adopting the logical level moved no
  bucket and no type row.

  Three tests carry it: the_logical_level_is_the_formats_table replays the format's own
  derivation table verbatim, the_canonical_level_falls_back_on_a_component_boundary pins
  the fallback, and an_unclaimed_type_keeps_the_extension_a_person_would_read pins that the
  two levels genuinely differ where they should. The old tar_pairs_fold_into_one_extension
  is gone; it tested the special case the general rule replaced.

  One divergence found and closed while here: the cfg(not(any(unix, windows))) fallback
  still carried the old .tar-fold rule, so a third platform would have derived a different
  extension from the same name. All three platform paths now run one rule over their own
  unit type.

  BLOCKED HALF SPLIT OUT AS fdu-gy3g. The conformance packet cannot usefully be vendored
  yet: its cases are matching-only, so they would pass against a single level and hide the
  gap this work closed. It needs direct basename-to-logical-extension cases from
  metabrowser first.

  make check passes.
resolution: null
duplicate_of: null
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
