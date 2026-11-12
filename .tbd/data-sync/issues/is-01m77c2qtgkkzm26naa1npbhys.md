---
type: is
id: is-01m77c2qtgkkzm26naa1npbhys
title: "Reuse one scan across CLI questions: bless --cache on then --stale-ok, or design a query bundle"
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m77c2ng8nt6bt6tr2xmjgbsh
created_at: 2026-11-12T07:55:41.776Z
updated_at: 2026-11-12T07:55:41.776Z
---
#207 §2.1/§2.2. Today one --cache on run plus --stale-ok answers independent kinds and windows without a walk (complete scans only, labelled stale); Python fdu.scan answers several windows from one walk. A bundle would be new grammar against One Scan, Many Views. Maintainer decision; the partial-scan half depends on partial persistence (fdu-8ybz plan, open question 4).
