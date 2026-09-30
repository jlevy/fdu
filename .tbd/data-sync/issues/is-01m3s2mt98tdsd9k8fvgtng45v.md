---
type: is
id: is-01m3s2mt98tdsd9k8fvgtng45v
title: Make the 64 MiB Markdown exact-render bound liftable by a flag
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-30T11:54:20.839Z
updated_at: 2026-09-30T11:54:20.839Z
---
The words analyzer counts a Markdown file over 64 MiB (MARKDOWN_EXACT_BYTES) as plain text (CoverageReason::TextOnly). Review finding R164-2 on #164: the design principles say every bound is liftable by a flag named where it is stated. 0.3.0 records the bound as the one deliberate exception in fdu-design-principles.md (commit 94656f21) because a liftable bound needs an analyzer-options element in the request's content identity: a new public Basis field threaded through Basis::content, Index::content_identity and its call sites, ContentTierIdentity::for_request, Basis::held_by, the sidecar options_fingerprint, the Python entry points and models, the parity harness, and a CLI flag in the style of --gitignore-budget (size or all), named as a tip in the note. Doing it removes the exception from the principles.
