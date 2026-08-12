---
type: is
id: is-01kzthrt9421hdj143fgncb5qx
title: fdu --version reports the bare semver with no dev revision
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-08-12T08:35:07.683Z
updated_at: 2026-08-12T08:36:44.003Z
closed_at: 2026-08-12T08:36:44.001Z
close_reason: build.rs embeds the git revision; dev builds print SEMVER-dev+gREV(.dirty), releases keep bare semver
---
A binary built from a checkout printed 'fdu 0.0.1', identical to the published release. Fixed by a build.rs that embeds the git revision: dev builds report SEMVER-dev+gREV (with .dirty when the tree has local edits), builds without git metadata keep the bare semver. Golden and wheel smoke assertions match the revision by pattern while still asserting the semver exactly.
