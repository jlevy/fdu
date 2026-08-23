---
type: is
id: is-01m0prhd1jfqr4gvtaq96hbwd7
title: Classification identity in listings; registry identity in Python
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:32:08.882Z
updated_at: 2026-08-23T21:14:56.883Z
closed_at: 2026-08-23T21:14:56.883Z
close_reason: ChildSnapshot and files-view rows carry the active registry's verdict (type id, family, resolved group id, detection source, confidence, origin flags) plus the logical extension, as metadata-only fields; filled after a view's bound so a bounded preset classifies only what it emits. Registry identity reads from Python through fdu.TypeRegistry (fingerprint, rule_count, extension_count, filename_count, type_ids, classify), which fdu-ctp5 introduced. Goldened in JSONL on both surfaces, with the directory case pinned as null rather than a sentinel.
resolution: null
duplicate_of: null
---
children() and files-view rows carry the compiled registry's verdict (type id, family, logical extension) as metadata-only fields; registry schema version, revision, and fingerprint readable from Python. Lets a client drop its own classifier while keeping its wire models; the shared-taxonomy contract (fdu-v4lc) makes the verdicts compatible by construction.

## Notes

Extends ChildSnapshot — the same struct fdu-gav9 already touches to carry provenance — so the two should land in that order to avoid touching it twice.
