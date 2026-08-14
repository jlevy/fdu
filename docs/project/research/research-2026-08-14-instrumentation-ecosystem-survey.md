# Research: What the Rust Ecosystem Already Solves, and What It Does Not

**Date:** 2026-08-14

**Author:** fdu project, with Claude Code research assistance

**Status:** Complete

## Overview

`perfkit` was written before checking whether the ecosystem had already solved the
problem, which is the wrong order.
This survey asked three questions after the fact: should that crate exist, what
profiling tooling is missing, and can syscalls actually be counted properly.

The first answer is yes, for structural reasons rather than preference.
The other two found four real gaps and two dead ends worth not re-walking.

## Should perfkit exist

Yes. Every mainstream crate surveyed — `tracing`, the `metrics` family, `fastrace`,
`opentelemetry`, `prometheus` — is built for **observability**: telemetry exported to a
collector, scraper, or log aggregator, at request or operation granularity.
That is a different problem from **paired A/B measurement inside an optimization loop**,
and the mismatch is not stylistic.
It shows up as cost.

|  | Per event | 13M events/run | Dependencies |
| --- | ---: | ---: | ---: |
| `perfkit` thread-local `Cell` | 1–2 ns | ~20 ms | 0 |
| `metrics` + a recorder | ~20–100 ns | 260 ms – 1.3 s | 20+ |
| `tracing`, span enabled | ~1,000–4,000 ns | prohibitive | 10+ |

For a tool whose entire runtime is a few hundred milliseconds, the middle row is not
instrumentation, it is the measurement.
The `tracing` figure is the least certain of the three — it is derived from `fastrace`’s
comparative benchmarks rather than from `tracing`’s own, which are not published for the
enabled case — but the order of magnitude is not in doubt, and `tracing` is designed for
spans around units of work, not events fired once per directory entry.

Two further gaps confirm the split.
Nothing surveyed counts allocations: `tracking-allocator`, the closest thing, last
released in July 2022. And nothing provides a process tier — `/proc/self/io` and
`/proc/self/stat` snapshot differencing — at all.

**This is not a general argument against the ecosystem.** If fdu ever needs
request-level telemetry exported somewhere, `tracing` is the right answer and this
survey does not argue otherwise.
The claim is narrower: hot-path counters in an optimization loop are a use case none of
these crates targets.

## What is missing, and now filed

**A CI regression gate — `fdu-slgp`.** `iai-callgrind` (v0.19.4, since renamed Gungraun)
counts user-space instructions deterministically, with no run-to-run variance, and can
drive an external binary through `Command::new`. The repository’s standing reason for
having no gate — a timing gate on a shared runner measures the runner — is true of wall
clock and false of instruction counts.

It must be advertised as partial, or it will be trusted for what it cannot see: Valgrind
does not instrument kernel code, and 29–62% of this workload’s time is system CPU. It
would catch index-build, allocator, snapshot-loader and content regressions, and it
would have missed `fdu-jnuo` — a pure syscall-count finding — entirely.
Linux only; Valgrind does not run on Apple Silicon.

**Cross-platform profiling — `fdu-c65j`.** `samply` profiles on Linux, macOS and Windows
from one command, viewed in the Firefox Profiler.
This closes `fdu-fz3j`: two of three supported platforms currently cannot take the first
step of the loop. It does not replace callgrind, which gives deterministic counts and
exact caller trees where samply gives statistical samples; they answer different
questions.

**Allocation attribution — `fdu-zr3a`.** `dhat-rs` attributes allocations to call sites
on all three platforms, which is exactly what `fdu-zgxd` needs — counters localized
eleven reallocations per entry to a layer and cannot name the site.
Its testing mode also supports “this path performs exactly N allocations” assertions,
which would turn the per-entry figures from an observation into a guard.

**perfkit’s process tier is Linux-only — `fdu-3b7v`.** Elsewhere `Snapshot::now()`
reports nothing, so the three-tier cross-check collapses to one tier.
macOS has `proc_pidinfo` with `PROC_PIDTASKINFO`: unprivileged, unaffected by SIP
because it is a process-info query rather than a tracing facility, giving a total
syscall count rather than a per-type breakdown — which is still the denominator a
cross-check needs. Windows has `GetProcessIoCounters`, whose mapping from directory
enumeration to `OtherOperationCount` is undocumented and must be established empirically
before anything is claimed from it.

## Dead ends, recorded so nobody re-walks them

**seccomp-BPF cannot count syscalls.** It is the obvious idea — a filter already runs on
every syscall entry, at about 26 ns with the constant-action bitmap — and it does not
work. Classic BPF has no writable state, so a filter can decide but cannot tally.
The return codes that would let a supervisor count (`SECCOMP_RET_USER_NOTIF`,
`SECCOMP_RET_TRACE`) block the caller until the supervisor answers, which defeats the
purpose.

**`perf_event_open` on syscall tracepoints and eBPF both need privilege.** Tracepoint
access requires `CAP_PERFMON` or `perf_event_paranoid = -1`; eBPF program loading
requires `CAP_BPF`. Containers and CI runners have neither.
Hardware counters on one’s own process do work unprivileged at the default paranoid
level, but those count cycles and instructions, not syscalls by type.

The consequence is worth stating plainly: **on Linux there is no unprivileged,
in-process, per-type syscall count.** Application counters at the call site, validated
periodically against `strace -c`, are not a compromise — they are the only instrument
that works everywhere without privilege.
The three-tier structure in
[the playbook](../guides/performance-instrumentation-playbook.md) is a consequence of
that fact rather than a preference.

## What this changes about the playbook

Nothing structural, which is itself the finding.
The tier model, the cross-check discipline, and the rule that application counters are
primary all survive contact with the ecosystem.
What changes is the priority list: a partial CI gate is now reachable, and the process
tier’s Linux-only limitation is a hole to close rather than an accepted platform
difference.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
