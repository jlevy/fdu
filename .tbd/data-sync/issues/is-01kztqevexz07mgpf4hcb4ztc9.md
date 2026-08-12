---
type: is
id: is-01kztqevexz07mgpf4hcb4ztc9
title: Spike macOS getattrlistbulk scan accelerator (H26)
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzpvt1vamkqp8fffnpwhd93v
created_at: 2026-08-12T10:14:32.668Z
updated_at: 2026-08-12T11:29:18.990Z
closed_at: 2026-08-12T11:29:18.989Z
close_reason: "Accepted as exp-022: fail-closed macOS getattrlistbulk backend with portable equivalence tests and small/720k paired evidence; final large cold-index wall -30.13%, producer wall -41.60%."
---
The post-adaptive profile attributes 67.2% of cold-scan-index samples to kernel/syscall work: open 29.7%, fstatat 20.1%, and getdirentries 9.3%, while index code is 1.35%. Test the existing H3/H26 target-specific macOS directory backend: obtain names and fingerprint metadata in bulk with getattrlistbulk, preserving the portable backend as the correctness reference and fallback. Pre-registered signal: cold-scan-producer and cold-scan-index wall and system CPU; require exact oracle parity, at least 3% paired wall improvement, and a narrow audited unsafe boundary. libc 0.2.189 is already locked and passed the repository provenance, age, license, and advisory gates; adding it directly still requires manifest review and full supply-chain validation.
