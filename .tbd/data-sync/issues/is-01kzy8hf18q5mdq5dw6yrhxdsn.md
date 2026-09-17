---
type: is
id: is-01kzy8hf18q5mdq5dw6yrhxdsn
title: Real-tree oracle reads stale DirEntry metadata on Windows
kind: bug
status: closed
priority: 1
version: 2
labels:
  - performance
  - testing
dependencies: []
created_at: 2026-08-13T19:10:47.336Z
updated_at: 2026-09-17T02:10:44.772Z
closed_at: 2026-09-17T02:10:44.771Z
close_reason: "Fixed in d597140 (PR #14): the oracle uses os.lstat"
resolution: null
duplicate_of: null
---
benchmarks/realtree/tree.py fingerprints through DirEntry.stat, which Windows serves from directory-enumeration data that NTFS updates lazily. The oracle can therefore record a directory mtime the filesystem no longer reports by the time the probe walks, making test_probe's engine_digest comparison fail intermittently on Windows CI with matching tallies and two disagreeing digests (observed 3x on PR #13 runs, green on the same commits' Linux/macOS jobs). The engine stats freshly for this documented reason, and benchmarks/corpus.py already guards the same hazard behind _DIRENTRY_METADATA_IS_AUTHORITATIVE; the oracle did not. Fix: os.lstat per entry in tree.py::_walk, plus mismatch forensics in test_probe that name the divergent field instead of printing two hashes. Both land in the #8 follow-up PR.
