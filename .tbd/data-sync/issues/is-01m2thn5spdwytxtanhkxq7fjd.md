---
type: is
id: is-01m2thn5spdwytxtanhkxq7fjd
title: Crate smoke does not pin the .cargo_vcs_info.json skip
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
created_at: 2026-09-18T15:20:13.877Z
updated_at: 2026-09-18T18:11:35.040Z
closed_at: 2026-09-18T18:11:35.038Z
close_reason: "Crate smoke now installs the packaged crate inside a throwaway git checkout (one empty commit), so the .cargo_vcs_info.json skip is the only thing that can report bare semver. Reproduced the vacuity first: with the skip deleted from crates/fdu/build.rs the old smoke still exited 0 reporting 'fdu 0.1.0', and deleting the sidecar from the extracted crate also still reported bare semver because TMPDIR is outside any repository. Red-green with the fix: skip deleted -> smoke fails on 'fdu 0.1.0-dev+g0d44a81ac'; skip restored -> passes in a checkout at a050ae46e. GIT_* and FDU_RELEASE_TAG are scrubbed for git and the install so an inherited GIT_DIR cannot redirect either; the commit carries its own identity and skips hooks and signing. Unit tests pin the checkout, that its revision is not this repository's, and the scrub under a hostile ambient GIT_DIR. Commit b58a7856 on PR #87; CI green."
resolution: null
duplicate_of: null
---
PR #87 R1 removed FDU_RELEASE_TAG from the packaged-crate install in scripts/release/smoke_crate.py so the version derivation is no longer masked, and added a '-dev+g' assertion. Verified by experiment that this still does not pin the branch it names: the extracted crate is installed under TMPDIR, which is outside any git repository, so 'git rev-parse' fails and emit_version falls to 'None => semver' (crates/fdu/build.rs:59-85). Deleting .cargo_vcs_info.json from the packaged crate and running the same 'cargo install' still reports bare 'fdu 0.1.0'. Deleting the skip at crates/fdu/build.rs:54-58 would therefore leave 'make release-rehearse' green. The real crates.io case, where a consumer extracts into a directory inside their own git repo, is the case the skip protects and is still untested. Fix would be to run the smoke install inside a throwaway git repo with its own HEAD, so the git fallback would stamp a revision unless the skip fires. Severity Low; not a merge blocker.
