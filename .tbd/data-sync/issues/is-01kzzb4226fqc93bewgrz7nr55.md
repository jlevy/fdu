---
type: is
id: is-01kzzb4226fqc93bewgrz7nr55
title: "Docs: explain the soft-schema experiment record in the performance loop guide"
kind: chore
status: closed
priority: 3
version: 2
labels: []
dependencies: []
created_at: 2026-08-14T05:15:08.230Z
updated_at: 2026-08-14T05:15:13.292Z
closed_at: 2026-08-14T05:15:13.292Z
close_reason: Added 'The record is a soft-schema artifact' to docs/project/guides/performance-loop.md
---
The performance-loop guide documents the protocol but never explains why each experiment is a soft-schema artifact: validated YAML frontmatter for the values the ledger renderer reads, prose body for the reasoning no schema can hold. Add a short section covering the split, the compiled contract, the never-retyped measured half, and the regenerated ledger, so the pattern is reusable beyond performance work.
