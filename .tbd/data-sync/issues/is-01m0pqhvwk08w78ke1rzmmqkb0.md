---
type: is
id: is-01m0pqhvwk08w78ke1rzmmqkb0
title: "PR #38 review R11: regime fields are empty on every Linux artifact"
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m0pqh0yf7etx8dywann7tnx3
created_at: 2026-08-23T07:14:55.486Z
updated_at: 2026-08-23T07:34:39.952Z
closed_at: 2026-08-23T07:34:39.951Z
close_reason: "Fixed: host_facts gains Linux cpu_model, memory_bytes and filesystem; virtualization detected on both platforms and mapped into the artifact; the filesystem reported is now the subject root's rather than /'s. Ten tests over the pure parsers."
---
measure.host_facts() populates cpu_model, memory_bytes, P/E cores and filesystem only under darwin, and nothing populates host_virtualization on any platform: 0 of 66 artifacts carry it, so the ledger regime table prints unrecorded for all 66. The loop guide says all three axes belong in every recorded result. Also _darwin_filesystem reads the filesystem of / rather than of the subject root.
