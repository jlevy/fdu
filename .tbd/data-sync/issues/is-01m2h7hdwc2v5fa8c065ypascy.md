---
type: is
id: is-01m2h7hdwc2v5fa8c065ypascy
title: release.yml crate smoke step cannot resolve fdu-core before it is published
kind: bug
status: open
priority: 1
version: 1
labels:
  - release
dependencies: []
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-15T00:30:15.434Z
updated_at: 2026-09-15T00:30:15.434Z
---
Found by the 2026-09-14 release-readiness audit (scratchpad reviews/release-readiness.md). .github/workflows/release.yml:71-79, step 'Smoke-test extracted crate CLI', runs cargo install --path target/package/fdu-<version> --locked. The extracted package's Cargo.toml carries fdu-core = { version = "0.1.0" } with the path stripped and its Cargo.lock names the crates.io registry source, so resolution goes to crates.io, where fdu-core does not exist until the first publish. Reproduced locally at dda7e6a: FDU_RELEASE_TAG=v0.1.0 cargo package --locked -p fdu-core -p fdu --allow-dirty succeeds, then cargo metadata --manifest-path target/package/fdu-0.1.0/Cargo.toml --locked fails with 'no matching package named fdu-core found; location searched: crates.io index'. The rehearsal that gates the first publish therefore cannot pass before it, and make release-rehearse (Makefile:370-380) does not include this step, so the local rehearsal cannot catch it. The workflow has never been dispatched (gh run list --workflow=release.yml is empty). Verified fix direction: cargo metadata on the extracted package with --config 'patch.crates-io.fdu-core.path="<abs>/target/package/fdu-core-0.1.0"' resolves (exit 0). Options: (a) pass that patch to the cargo install smoke step (drop --locked or regenerate the lock for the smoke only), (b) smoke the workspace-built binary and keep cargo package as the packaging proof. Add the same step to make release-rehearse so the local rehearsal exercises it. Acceptance: a dispatched rehearsal on main passes the crate job with fdu-core absent from crates.io; after the first publish the step still passes.
