---
type: is
id: is-01m2h3xw5vzbmm4843r3vgp8zd
title: "PR #60 review PR60-DOCS-2: AGENTS.md Terminology overstates watch and the per-request default; CI names keep the old probe split"
kind: bug
status: in_progress
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2h3xgn1f3azaqbcvh9mzxas
created_at: 2026-09-14T23:27:08.986Z
updated_at: 2026-09-14T23:27:31.340Z
---
PR #60 at df43bdd: AGENTS.md:83 says watch is the only build feature, but crates/fdu-py/Cargo.toml:19-24 has extension-module. AGENTS.md:84 'turned off per request' must match the code after #57 (default on for library open; one-shot reports and --watch off pending fdu-elnn). .github/workflows/ci.yml:100-103 step names distinguish 'minimal-library' from 'repository-only' probes, and :114 job name 'Test library feature boundaries' uses plain 'feature'.
