---
type: is
id: is-01kzx1bhwb9vp0hvm365w7kaxw
title: "Phase 3c: Lock SLOC semantics with goldens and self-host checks"
kind: task
status: closed
priority: 2
version: 6
spec_path: docs/project/specs/done/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzx1bjgmrwyda3sxf8ev2qhj
parent_id: is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-13T07:45:59.179Z
updated_at: 2026-08-13T12:03:03.607Z
closed_at: 2026-08-13T09:52:25.832Z
close_reason: Added 15-language adversarial fixtures, exact tryscript and self-host semantic locks, SCC/Tokei comparison, line-ending coverage, and passed make check.
---
Add adversarial Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin, shell, and SQL fixtures. Pin code/comment/blank partitions and intentional SCC/Tokei differences in tryscript and unit goldens; extend fdu self-host invariants and run test-golden, content-selfcheck, and make check before parser optimization.
