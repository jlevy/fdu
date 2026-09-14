---
type: is
id: is-01m2ebdp32y4vq1m3w2q02qbz5
title: "PR #48 review CLASS-7: the registry parser's TOML subset is unverified against the real registry"
kind: task
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:40:23.777Z
updated_at: 2026-09-14T03:10:18.613Z
closed_at: 2026-09-14T03:10:18.612Z
close_reason: |
  feca876: the registry reader is now a cursor that tracks its line. It accepts a BOM, CRLF, trailing comments, spaces inside [[ kind ]], all four TOML string forms with every TOML 1.0 escape, string arrays over several lines with comments and a trailing comma, and _ separators and exponents. It rejects each of these by name: single-bracket tables, quoted and dotted keys, inline tables, nested arrays, and hexadecimal, octal, or binary integers. The real document is MetaBrowser's src/metabrowser/data/file-rollup-format/recommended-file-types.toml. Its last schema 3 revision (1e1f5f912) gives fingerprint 0x951d41f9a873ef78 before and after the change. A recomputation from tomllib's reading gives the same value, and so does a re-spelling in every accepted form. Not vendored: MetaBrowser is AGPL, and fdu-ekga owns vendoring. MetaBrowser main is schema 4 (fdu-r3j4). The compact [[kind]] reader has the same defect (fdu-ujsa). CI green: run 34801252954.
resolution: null
duplicate_of: null
---
Low. classify/file_rollup_manifest.rs:66-77, 226-243. The hand-written parser rejects valid TOML forms (BOM, inline comments, multi-line arrays, literal strings) and keeps backslash-quote escapes literally in labels; rejection is total and typed, but it was never checked against the actual shared registry document. Fix: add a fixture of the real registry file. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
