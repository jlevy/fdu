---
type: is
id: is-01m00kvm2k3nar5k070dxfdzqr
title: perfkit reports nothing on macOS and Windows; proc_pidinfo and GetProcessIoCounters fix that
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T17:07:03.378Z
updated_at: 2026-08-14T17:07:03.378Z
---
perfkit's process tier is Linux-only: Snapshot::now() returns available=false everywhere else, so two of three supported platforms get no kernel-side cross-check at all and the three-tier discipline collapses to one tier. Research found unprivileged options on both. On macOS, proc_pidinfo with PROC_PIDTASKINFO fills a proc_taskinfo struct carrying pti_syscalls_unix and pti_syscalls_mach, plus pti_faults, pti_pageins and pti_csw. It needs no privilege for one's own process and is not restricted by SIP, being a process-info query rather than a tracing facility. It gives a total rather than a per-type breakdown, which is still exactly the denominator a cross-check needs: if application counters sum to N and the kernel says N plus M, there are M syscalls unaccounted for. Note the fields are i32 and wrap around two billion. On Windows, GetProcessIoCounters returns read, write and other operation counts with no privilege for one's own process, though the mapping from directory enumeration to OtherOperationCount is undocumented and must be established empirically before anything is claimed from it - the getdents64 finding in fdu-jnuo is the cautionary case. Both are one call and cheap enough for every run. Also worth adding on all platforms: getrusage voluntary and involuntary context switches, which the harness reports but the in-process tier does not, and which detect thread contention.
