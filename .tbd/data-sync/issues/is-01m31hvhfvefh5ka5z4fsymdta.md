---
type: is
id: is-01m31hvhfvefh5ka5z4fsymdta
title: "Linux stabilization session: gate, PR review, and stack readiness (2026-09-21)"
kind: epic
status: open
priority: 0
version: 18
labels: []
dependencies: []
parent_id: is-01m31hc3p7wv76jeq5dhgv3bd8
child_order_hints:
  - is-01m31hw3vmet0hpbpnt98dnrfk
  - is-01m31hw4f865xyaj8kvtz2n88c
  - is-01m31jnypzjgvz7qsrhe2qhpn4
  - is-01m31jnzma7mmnt1m13zrk94md
  - is-01m32dphqr330wwjakmf2jrwm8
  - is-01m32dpjd0e1s1fsshtqxjg6fs
  - is-01m32ewkja67k8pzqate1hdmz5
  - is-01m32ewm52a3d4wrd6mhsx5dyd
  - is-01m32ewmqkv1v71f5pjtc3djmx
  - is-01m32exka5myg890p63v94yenw
  - is-01m32ez2jg9svawec8b0j9gxvs
  - is-01m32f63r0ba1v7fkcjjwf2f1x
  - is-01m32f649ymyw0htpyrs3b1rhw
  - is-01m32f64vwaa0thyrc4sg5h1cm
  - is-01m32f6qyw26btzdt7de1m4hrs
  - is-01m32h5emeefrv7vx5yrx70fga
  - is-01m32jqdbwhh12ekwymn8h6r1y
created_at: 2026-09-21T08:38:23.483Z
updated_at: 2026-09-21T18:12:51.196Z
---
Work carried out from a Linux host with no nested `.claude/worktrees/` checkout — the first host able to run the handoff gate since `fdu-vjf2` was found.

Scope:
- Fix `fdu-vjf2` so a local gate exists again.
- Run `make check` and `make cross-lint` on Linux, which no host has done for the open stack.
- Review every open PR (#94, #96, #97, #98, #99, #103, #104, #105) — the survey found zero reviews on all eight.
- Resolve #105's failing `Test (ubuntu-latest)`.
- Leave the stack in a reviewed, green, ready-to-merge state.

Host: Linux 6.18.44, x86_64, virtualized container. Toolchain bootstrapped to the pinned versions (uv 0.12.1, cargo-deny 0.20.2, MSRV 1.85.0, darwin+windows lint targets) — the image shipped uv 0.8.17, which cannot parse the relative `exclude-newer` and fails as several unrelated errors.
