---
type: is
id: is-01m2pj0f459s8ad1efzyn2qmbq
title: "Answer model: one typed value per document, serialized by every writer"
kind: epic
status: open
priority: 0
version: 10
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - output
  - design
  - release
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
child_order_hints:
  - is-01m2pj0fmpbg0w8xj83dkyt9mh
  - is-01m2pj0g6c5fswmdcbbzjhx0rx
  - is-01m2pj0gs0cyxp178kz5kktpbb
  - is-01m2pj0hc166t091a4k04kks9t
  - is-01m2pj0hxv9y019pxwhnvpgkx8
  - is-01m0k512k9a6dq2k51fbfe5xn4
  - is-01m2pmrmvr6x2kz662jksackef
created_at: 2026-09-17T02:09:26.148Z
updated_at: 2026-09-17T02:57:35.954Z
---
fdu's JSON and YAML are hand-written field by field in report_format.rs, so the two formats diverged
under one schema id and YAML scalar quoting failed real parsers (fdu-c2ml). Python already wraps the
Rust renderer, which is the right boundary. Goal (maintainer direction 2026-09-17): rely on highly
conformant YAML in Rust, reusing a standard approach or, if none is adequate, a reusable YAML utility
used consistently by every YAML site and usable in future projects; a single ordered value model so JSON
and YAML cannot diverge structurally; conformance proven against real YAML 1.1 and 1.2 parsers.
