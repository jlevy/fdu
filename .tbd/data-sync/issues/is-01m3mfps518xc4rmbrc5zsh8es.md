---
type: is
id: is-01m3mfps518xc4rmbrc5zsh8es
title: Adopt cargo-semver-checks for patch releases of fdu-core
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m3mcwy5q1j1nab2srxz38ebb
created_at: 2026-09-28T17:06:24.544Z
updated_at: 2026-09-28T17:06:24.544Z
---
tbd guidelines rust-release-rules: 'Run semver checks (cargo-semver-checks) for any library that promises API compatibility.' fdu's 0.x rule promises that a patch release (0.2.0 to 0.2.1) never changes the Rust API incompatibly, but nothing checks it: no cargo-semver-checks in ci.yml, release.yml, or the Makefile. Flowmark runs it as a CI job (flowmark-rs .github/workflows/ci.yml semver-checks, obi1kenobi/cargo-semver-checks-action). Proposal: a release-plan or CI check that compares fdu-core (and fdu's lib, if any) against the last published version when the version bump is a patch; pin the tool per SUPPLY-CHAIN-SECURITY.md. Found in fdu-n2hc.
