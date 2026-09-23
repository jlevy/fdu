---
type: is
id: is-01m35t5a4qs9hdez1z1z0p5mcj
title: "PR #96 A96-1: scope machine-format depth exemption to flat lists"
kind: task
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m35sm4jn2ytfmy91136g50b7
created_at: 2026-09-23T00:20:29.973Z
updated_at: 2026-09-23T08:14:06.925Z
closed_at: 2026-09-23T08:14:06.925Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
At c3aeed8a, directory-query plan line 153 says depth has no effect under machine formats. Executed legacy tree JSON with depth=0 still folds children, correctly. Qualify exemption to flat List projections; legacy tree/full sections retain depth. Nonblocking review nit.
