---
type: is
id: is-01m0p0f9q7nap0fkb98yy7k19x
title: "PR #42 R20: fdu-core feature comments still name the deleted cli feature"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0p06qgww21s4fpdkj2bb6bf
created_at: 2026-08-23T00:31:34.119Z
updated_at: 2026-08-23T00:57:54.165Z
closed_at: 2026-08-23T00:57:54.165Z
close_reason: "Fixed in 4e34ce3; addressed the review on PR #42, verified through make check and make release-rehearse."
---
crates/fdu-core/Cargo.toml [features]: the default/watch comments say 'a --features cli build without watch still compiles' and 'scan, index, snapshot, and the CLI are all fully functional'. The cli feature no longer exists and the CLI is no longer in this crate. Same stale-name class as R3, found while fixing it.
