---
type: is
id: is-01kzyfaf99b5edke31z5s1aapt
title: "H77: macOS searchfs catalog enumeration to break the per-directory open floor"
kind: task
status: open
priority: 3
version: 1
labels:
  - performance
dependencies: []
created_at: 2026-08-13T21:09:18.248Z
updated_at: 2026-08-13T21:09:18.248Z
---
Exp-045/046 profiles put about 95% of both FDU's and dumac's worker samples in synchronous open plus getattrlistbulk, and the 901,963-entry subject has 110,369 directories, so both tools pay at least one open and one bulk call per directory. searchfs(2) queries a volume's catalog directly instead of opening each directory, which is the same physics NTFS MFT scanners exploit, and no surveyed tool uses it. It would need parent-id tree reconstruction, subtree scoping, an audit of permission semantics (a catalog query can surface entries a directory walk would not), non-UTF-8 name handling, and probe-and-fallback to the bulk reader like every other accelerator. High design risk and possibly unusable for scoped scans, but it is the only identified idea that removes per-directory work rather than shaving it. Screen with a standalone prototype against the existing tree before touching production code.
