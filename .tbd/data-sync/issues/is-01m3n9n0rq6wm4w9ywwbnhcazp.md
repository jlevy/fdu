---
type: is
id: is-01m3n9n0rq6wm4w9ywwbnhcazp
title: "Differential and quiet speed run: fdu --analyze code vs tokei 15.0.0 and scc 4.1.0 on linux-v6.12"
kind: task
status: open
priority: 2
version: 2
labels: []
dependencies:
  - type: blocks
    target: is-01m3n77fy60sdcdzzk9qmbzg8e
created_at: 2026-09-29T00:39:49.782Z
updated_at: 2026-09-29T00:39:59.275Z
---
Needed for the README matrix scc/tokei columns (fdu-dbn9) and to re-score accuracy after analyzer v3 (the 14/30 fixture score predates v3; tokei 15 and scc 4.1 never measured). Protocol from the SLOC survey (fdu-61ez; saved with the matrix draft): Arm A ignore rules off on an identical copied tree, Arm B each tool's .gitignore handling on; text output to /dev/null; one untimed JSON validation run per tool per arm; add fdu-code/scc/tokei contracts to explorations/benchmarks/realtree/compare_tools.py (3 warm-ups, 12 alternating pairs, quiet gate, wait4 RSS). Settle per-file disagreements with minimal reproductions. Install tokei via cargo --locked --version 15.0.0; scc from the release tarball verified against checksums.txt (host Go is too old).
