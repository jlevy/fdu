---
type: is
id: is-01m3k8bzyw79ywfse1dwa77yp0
title: Keep low-space diagnostics from writing snapshots into the diagnosed volume
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T05:38:56.600Z
updated_at: 2026-09-28T05:38:56.600Z
---
Review finding F12: a default one-shot metadata report never reads its snapshot but writes it (35 MB at 451k entries, 114 MB at 1.5M, F_FULLFSYNC) to ~/Library/Caches/fdu on the nearly full Data volume. The disk-pressure profile should use --cache off or an off-volume store (see fdu-vw9r).
