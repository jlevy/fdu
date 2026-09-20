---
type: is
id: is-01m2yvd6fn85kxwhercabbzhg9
title: "H148: PGO screen on linux-v6.12 cold-scan-index and warm-revalidate"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-20-linux-pgo-screen.md
labels: []
dependencies: []
parent_id: is-01m2yvd1dvv6z596vdz5tt2swg
created_at: 2026-09-20T07:27:35.925Z
updated_at: 2026-09-20T07:39:02.734Z
closed_at: 2026-09-20T07:39:02.734Z
close_reason: "H148 accepted (exp-154): quiet linux-v6.12 cold-scan-index -8.35% and warm-revalidate -8.15%. Cargo.toml unchanged. fdu-pdne stays open for pipeline adoption."
---
Standing bead fdu-pdne. Same source as #97 HEAD. Control is fat-LTO release probe. Candidate is profile-use after training scan-index, revalidate, summary, and summary --no-controls on reconstructible linux-v6.12. Accept only if BOTH jobs clear 3% wall with intervals below zero and RSS no worse. A miss on either job rejects. Do not change Cargo.toml unless both clear. Do not reuse H93. First experiment exp-154.
