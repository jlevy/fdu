---
type: is
id: is-01m4mp5e6ym95w00jftzt5crt3
title: "Parity scripts: spell the age-cell format once"
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-10-11T05:14:58.141Z
updated_at: 2026-10-11T05:14:58.141Z
---
Review F on PR #191 (F3 suggestion): scripts/parity-classes.mjs:85 (AGE_TOKEN) and :176-179 (the classifier's [AGE] pattern) spell the same age format twice, and the goldens already disagree on it: cli-human's pattern accepts a 'w' unit the renderer never prints (the ladder is s, m, h, d, mo, y). Derive both from one definition and align the golden pattern with the renderer's ladder.
