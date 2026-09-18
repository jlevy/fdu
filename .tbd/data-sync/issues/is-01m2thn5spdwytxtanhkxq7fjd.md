---
type: is
id: is-01m2thn5spdwytxtanhkxq7fjd
title: Crate smoke does not pin the .cargo_vcs_info.json skip
kind: bug
status: in_progress
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-09-18T15:20:13.877Z
updated_at: 2026-09-18T17:50:42.204Z
---
PR #87 R1 removed FDU_RELEASE_TAG from the packaged-crate install in scripts/release/smoke_crate.py so the version derivation is no longer masked, and added a '-dev+g' assertion. Verified by experiment that this still does not pin the branch it names: the extracted crate is installed under TMPDIR, which is outside any git repository, so 'git rev-parse' fails and emit_version falls to 'None => semver' (crates/fdu/build.rs:59-85). Deleting .cargo_vcs_info.json from the packaged crate and running the same 'cargo install' still reports bare 'fdu 0.1.0'. Deleting the skip at crates/fdu/build.rs:54-58 would therefore leave 'make release-rehearse' green. The real crates.io case, where a consumer extracts into a directory inside their own git repo, is the case the skip protects and is still untested. Fix would be to run the smoke install inside a throwaway git repo with its own HEAD, so the git fallback would stamp a revision unless the skip fires. Severity Low; not a merge blocker.
