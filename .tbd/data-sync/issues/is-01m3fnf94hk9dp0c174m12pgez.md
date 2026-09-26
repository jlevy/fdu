---
type: is
id: is-01m3fnf94hk9dp0c174m12pgez
title: Recognize JavaScript regex literals in code SLOC
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-26T20:10:58.064Z
updated_at: 2026-09-26T20:10:58.064Z
---
Research fdu-4il8 minimal valid JS input const re = /[/*]/; followed by const answer = 42; is 2 code lines but fdu 0.1.0 reports 1 code/1 comment: it enters block-comment state inside regex. Tokei14 shares this defect, demonstrating comparator agreement is not sufficient. Add JS/TS regex-versus-division cases and state recovery fixtures with hand-classified expectations; assess a narrow lexical fix and version analyzer cache semantics.
