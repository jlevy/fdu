---
type: is
id: is-01m0w5fbs0n1xv9rxrmrp79mda
title: Exclude special filesystem objects at the MetaBrowser provider boundary
kind: bug
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47
    at: 2026-08-25T09:54:29.343Z
  - kind: pr
    url: https://github.com/jlevy/metabrowser/pull/74
    at: 2026-08-25T09:54:29.344Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5017522830
    at: 2026-08-25T09:57:38.318Z
labels:
  - pr47-review
  - metabrowser
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-25T09:54:25.695Z
updated_at: 2026-08-25T09:57:38.319Z
---
Exact-head adoption finding at FDU d19b0ce versus MetaBrowser 0577bb1. MetaBrowser exposes a closed EntryType algebra of file, directory, and symlink and requires every provider to exclude sockets, FIFOs, devices, and other special objects rather than reclassify them (arch-inventory-provider.md:188-193; contract.py:366-369). Its Python walker excludes them before retention and refresh removes them. FDU retains EntryKind::Other in the authoritative index (scan.rs:4309-4319), and the reference embedder listing maps every non-directory row through the same file-shaped Row path without checking kind (browser_provider.py:204-219). A thin adapter cannot repair native rollups, continuation remainders, change invalidations, and max_files semantics after the fact if Other remains in projections as an ordinary leaf. Define the engine-native MetaBrowser projection/admission rule: special objects may remain valid for FDU CLI semantics, but the opened provider view must exclude them consistently from listing/tree/catalog pages, rollups/remainders, refresh/watch output, diagnostics, and agreement fixtures without a Python mirror or second filter index. Acceptance: boot plus create/delete/replace scenarios for FIFO/socket where supported; three-kind output only; exact conservation and paging; no special object counted as a regular file; both providers agree from one recorded observation stream.
