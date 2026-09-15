---
type: is
id: is-01m2h7hnm1bh7s4abvp8b8czgt
title: "Release evidence omits fdu-core: inspect_artifacts.py and registry_state.py cover only the fdu crate"
kind: bug
status: in_progress
priority: 1
version: 2
labels:
  - release
dependencies: []
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-15T00:30:23.359Z
updated_at: 2026-09-15T05:36:44.175Z
---
Found by the 2026-09-14 release-readiness audit (scratchpad reviews/release-readiness.md). release.yml uploads both target/package/fdu-core-<v>.crate and fdu-<v>.crate into the release-crate artifact and the evidence job merges every release-* artifact into one directory. scripts/release/inspect_artifacts.py:120-126 (inspect_directory) selects crates by the exact name f"fdu-{version}.crate" and requires exactly one, so fdu-core-<v>.crate is silently ignored: not inspected by inspect_crate, absent from the artifacts list at :140-146, and therefore absent from release-manifest.json and SHA256SUMS. scripts/release/registry_state.py:115-121 (crates_io_state) fetches only crates.io/api/v1/crates/fdu/<v>/download, so fdu-core is never classified missing/identical/conflict. tests/release/test_inspect_artifacts.py writes no fdu-core fixture. docs/project/guides/release-process.md states that fdu-core must be published first and that the publish job must reproduce the validated .crate and compare its SHA-256 with the retained preview before upload; for fdu-core there is no retained preview digest and no registry check, so the invariant cannot be verified for the crate that goes first. Fix: inspect both crates (inspect_crate already accepts fdu-core's layout: Cargo.toml, Cargo.toml.orig, LICENSE, README.md, src/lib.rs), list both in the manifest with kind 'crate' and a package name field, emit both in SHA256SUMS, classify both on crates.io, and add fixtures and a negative test (a directory missing fdu-core must fail). Acceptance: release-manifest.json and SHA256SUMS name both crates; registry-state.json has a crates.io row for fdu-core and for fdu; tests cover both.
