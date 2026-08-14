---
type: is
id: is-01m00kvknz3anmdbkpfbr8hhhp
title: Evaluate iai-callgrind for the CI performance gate the project has never had
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T17:07:02.974Z
updated_at: 2026-08-14T17:07:02.974Z
---
The repository states plainly that timing gates are not in CI because a timing gate on a shared runner measures the runner. That is correct about wall clock and not about instruction counts, which are deterministic: iai-callgrind (v0.19.4, July 2026, since renamed Gungraun) counts user-space instructions under callgrind with essentially zero run-to-run variance, and it can benchmark an external binary through Command::new, so it can target perf_probe rather than needing library-level harness code. That would give the project its first automated regression gate. The caveat is decisive for scoping and must be written into whatever lands: valgrind does not instrument kernel code, so syscall and I/O work is invisible - and on this workload that is 29 to 62 percent of the time in system CPU. A gate would therefore catch index-build, allocator, snapshot-loader and content-analysis regressions and would silently miss a syscall-count or I/O-pattern regression, which is exactly the class fdu-jnuo just found. So it is a partial gate that must be advertised as partial, or it will be trusted for what it cannot see. Practical constraints: valgrind imposes 4 to 20 times slowdown, so a small generated fixture of 1k to 5k entries is the workable size, and it is Linux-only because valgrind does not run on Apple Silicon. Also evaluate CodSpeed, which is free for open-source and gives PR-level regression reports, with the same kernel-blindness. Open question worth settling before adopting: whether an instruction-count regression under valgrind's thread serialization reliably predicts a wall-time regression in a parallel walk.
