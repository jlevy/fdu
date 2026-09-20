---
type: is
id: is-01m2y4f4g34vdbgxf0jcvt8dw3
title: "Linux parallel validation of Darwin #92"
kind: epic
status: closed
priority: 1
version: 13
spec_path: docs/project/specs/active/plan-2026-09-19-linux-parallel-validation.md
labels:
  - linux
  - campaign-2
dependencies: []
child_order_hints:
  - is-01m2y4f4z059xvgn1hhc65fb78
  - is-01m2y4f5czb03hhaskb686x0c0
  - is-01m2y4f5w92kqgdwzzf75bm1yn
  - is-01m2y4f6c3ea0q3ngn99nz7eek
  - is-01m2y762f58jb3kx6nhn0hr31g
  - is-01m2yh3xty3by52m58k26g1c7m
  - is-01m2ykq9k7edrmfct2a96ghxqb
created_at: 2026-09-20T00:46:42.178Z
updated_at: 2026-09-20T05:16:21.417Z
closed_at: 2026-09-20T05:16:21.417Z
close_reason: "Linux parallel validation recorded on #94: H139–H142 same, H111 fail, H143 leftover. Docs marked ready to merge onto #92. Next cut is H144 if minted."
---
Linux host stacked on #92 (26480612). Replicate accepted Darwin engine (H125/H129/H131/H133/H138), profile walk leftover, and run H111 floor gates. Spec: docs/project/specs/active/plan-2026-09-19-linux-parallel-validation.md. Do not push to #91 or the Darwin branch. Do not restart H86. H139-H142 reserved.

## Notes

Rebased onto #92 937f9445 (c441edf6 Cow borrow + restore-evidence). H139-H141 meanings hold: four-view still shares; well-formed cache-hit unchanged. H111 fail / H143 leftover unchanged. Do not remasure unless leftover identity changes.
