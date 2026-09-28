---
type: is
id: is-01m3k8bzyw79ywfse1dwa77yp0
title: Keep low-space diagnostics from writing snapshots into the diagnosed volume
kind: task
status: open
priority: 3
version: 2
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T05:38:56.600Z
updated_at: 2026-09-28T06:10:12.880Z
---
Review finding F12: a default one-shot metadata report never reads its snapshot but writes it (35 MB at 451k entries, 114 MB at 1.5M, F_FULLFSYNC) to ~/Library/Caches/fdu on the nearly full Data volume. The disk-pressure profile should use --cache off or an off-volume store (see fdu-vw9r).

## Notes

2026-09-28: PR #139 (stack 141) resolves the one-shot case: under --cache auto a one-shot metadata report no longer writes a snapshot. Remaining: --watch, opened indexes, and content analysis still write to the default cache dir on the Data volume; the disk-pressure profile needs an off-volume store for those.
