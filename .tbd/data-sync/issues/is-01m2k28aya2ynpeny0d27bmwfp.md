---
type: is
id: is-01m2k28aya2ynpeny0d27bmwfp
title: "PR #61 review PR61-REG-1: the crates.io audit hashes the download endpoint's JSON stub, so every crate is conflict"
kind: bug
status: closed
priority: 1
version: 3
labels:
  - release
dependencies: []
parent_id: is-01m2k27z25tt9ygs4c1nchhzez
created_at: 2026-09-15T17:36:23.498Z
updated_at: 2026-09-15T17:41:32.004Z
closed_at: 2026-09-15T17:41:32.004Z
close_reason: "f8064ec: registry_state.py reads /api/v1/crates/{crate}/{version}; 404 is missing, version.checksum decides identical or conflict, a record without a checksum is refused, and non-404 or network failures raise RegistryError naming the URL. Four stub tests (no network) fail on the old code; verified read-only against live crates.io."
resolution: null
duplicate_of: null
---
PR #61, delta review 5213560245. scripts/release/registry_state.py:84-93, 134-135 @ 4db083b. get() sends Accept: application/json, and crates.io's /api/v1/crates/{crate}/{version}/download answers 200 {"url": ...} for any version, existing or not. The audit hashes that stub and reports conflict for every crate: every rehearsal fails at release.yml:218 (exit 2), --require-identical can never pass for crates.io, and the recovery section would send a maintainer to yank an identical fdu-core. The unit test passes only because its fetch stub returns crate bytes for the download URL. Fix: read /api/v1/crates/{crate}/{version}; 404 is missing; compare version.checksum (the .crate SHA-256) with the manifest digest; network errors and non-404 statuses fail hard, never missing.
