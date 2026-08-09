---
type: is
id: is-01kzky6vqxwd47xz3we21s86zq
title: Harden fdu against the Rust engineering quality audit
kind: epic
status: open
priority: 1
version: 23
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - engineering-quality
dependencies: []
parent_id: is-01kzg48ekn4sm0azybr010qgmn
child_order_hints:
  - is-01kzm514w7qntv14b7msxkk1jw
  - is-01kzky75mq2zkhzvgzs9c95cts
  - is-01kzky7fjvk5f7758cav879nhs
  - is-01kzm5e6075c2bbx8b47851v0x
  - is-01kzm5ee1czjcdqn35y96cnyef
  - is-01kzky7pe77x2wqndf6kdwyn6p
  - is-01kzm5eqahbmtm5gwhf6fmejwh
  - is-01kzky7aq9m5j7r8a33tj0tx38
  - is-01kzky7wjz44trprn1ck52pd58
  - is-01kzky86nqp91wq9d3wj2psnwr
  - is-01kzky8bctckj3kk8gwntbg8tn
  - is-01kzky8gxazfdstfgbv3m9fa58
  - is-01kzm9qxbh857h2rym0q39c4te
  - is-01kzm9qxjxbrbcm4nm0mmj21ge
  - is-01kzm9qxshmd1st8ck5rpjjg02
  - is-01kzma00ysmgrv2tree7fpsww7
created_at: 2026-08-09T18:57:49.820Z
updated_at: 2026-08-09T22:23:48.696Z
---
Child hardening epic under fdu-qfz6. The current merge graph is deliberate: fdu-ad45 independently restores executable-input trust; fdu-nlh8 makes batch application atomic before fdu-s7wr seals the guard-free ownership API; fdu-1j0b removes watch filesystem I/O from index locks; and fdu-8jte makes the watcher an I/O-free bounded coalescer with fail-safe overload and shutdown. Those concurrency paths converge at deterministic validation fdu-gd6n, and final approval fdu-sn43 waits on fdu-ad45 plus fdu-gd6n. After approval, pin the toolchain, add model and snapshot failure-state safety nets, and harden stack, native-path, package, and release boundaries before Phase 1 representation and publishing work consumes them.

## Notes

Wave 0 implementation is complete: fdu-ad45, fdu-nlh8, fdu-s7wr, fdu-1j0b, fdu-8jte, and fdu-gd6n are closed. fdu-sn43 is the only active PR #1 gate. Post-merge safety-net and boundary beads remain open by design.
