---
type: is
id: is-01m77c2pr8ffjsedez859fhh1v
title: Say when --cache on wrote no snapshot because the scan was partial
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m77c2ng8nt6bt6tr2xmjgbsh
created_at: 2026-11-12T07:55:40.680Z
updated_at: 2026-11-12T07:55:40.680Z
---
#207 §2.3/§3.2. Over a partly unreadable tree, --cache on --allow-partial exits 0, writes no snapshot, and says nothing; the next --stale-ok then fails recommending a --cache on run that cannot help. The engine knows (RunFacts::entries_verified in lib.rs). Give the fact a public home (a note in text and a provenance field in machine output) and make the --stale-ok miss message name the partial-scan cause.
