---
type: is
id: is-01m32f50wdffzswdhb754scda6
title: "macOS final phase: ~/Library under TCC, APFS, and iCloud dataless files"
kind: task
status: open
priority: 2
version: 1
labels:
  - macos
dependencies: []
parent_id: is-01m32ez2jg9svawec8b0j9gxvs
created_at: 2026-09-21T17:10:22.860Z
updated_at: 2026-09-21T17:10:22.860Z
---
Deferred to a final test phase on a real Mac; nothing here is reachable from a Linux container. Split out of fdu-y99c so the platform-neutral runbook can run now.

Run in BOTH permission states, because they are different tests: terminal without Full Disk Access, and with it. Under TCC the restricted subtrees (Mail, Messages, Safari, Cookies, Application Support/com.apple.TCC, and the ~1012 sandbox dirs under Containers) return EPERM — not EACCES, not ENOENT. Handled appropriately means a clean refusal, exit 2, warnings on stderr, `complete: false` with the errors named in machine formats; never a crash, never a silently smaller total.

Open thread this closes: fdu-6o5o records a SIGKILL (137) on `fdu ~/Library -d 2 -n 30 --sort size --min-size 300M`, and its last note says the unbounded full-Library scan has still not been retested. Bounded scans passed on 2026-09-18 (0.08s / 23 MiB at --scan-depth=1; 0.23s / 27 MiB at --scan-depth=2, exit 2 with 26 TCC warnings).

Trap when checking warm against cold: the `unverified-subtree` class still holds 128 registered violations on Linux and macOS pending fdu-szll, and TCC denials are how unverified subtrees arise on a real machine. Compare against the registry, not naive equality, or known-open issues get reported as new findings.

macOS-only hazards:
- Do not chase ~/Library/Containers as an fdu performance bug. fdu-6o5o established `du -sh` times out there too; per-container TCC checks make that subtree hostile to every tool.
- iCloud Drive dataless files under ~/Library/Mobile Documents: reading one can trigger a download or return EIO. A metadata walk must not materialize them, and `--analyze` over that path needs deliberate checking — a disk-usage tool that silently downloads gigabytes is both a correctness and a safety problem.
- APFS: case-insensitive but case-preserving by default, so case-colliding names collapse where they stay distinct on Linux; firmlinks into /System/Volumes/Data; snapshots; cloned files, making `allocated` diverge from `apparent` in ways ext4 never shows; resource forks and extended attributes; .DS_Store.
- The macOS walk is separate code: `scan.rs`, `lib.rs`, `platform_tuning.rs` and `counters/process.rs` all carry `cfg(target_os = "macos")`. A Linux run validates none of it, and `make cross-lint` only proves it compiles.

Memory is the open question and wants a slope, not an anecdote: peak RSS at three fixture sizes on a quiet host, both binaries, to establish whether growth is unbounded on ~/Library-shaped trees. Record disk pressure — the original reporter's host was 95-99% full, a confound — and keep TCC-induced slowness out of the attribution.
