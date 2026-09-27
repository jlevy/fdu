---
type: is
id: is-01m3g53gx1d4p4bd1hd8qhqye5
title: Use the shared Unix cache location and expose cache destination overrides
kind: feature
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-09-26-code-analysis-presentation.md
labels: []
dependencies: []
created_at: 2026-09-27T00:44:10.016Z
updated_at: 2026-09-27T00:44:10.016Z
---
User approved ~/.cache/fdu on macOS. Implement typed core destination, FDU_CACHE_DIR exact override, XDG_CACHE_HOME/fdu, shared Unix default, preserved Windows native default, visible resolved destination and explicit safe cleanup of recognized caches at the old macOS location. Automatic retention remains fdu-558j.
