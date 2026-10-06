---
type: is
id: is-01m49mn7db40x5v4cbah1k2a41
title: Fix three doc inconsistencies about peer techniques found by the fast-by-construction audit
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-10-06T22:16:59.562Z
updated_at: 2026-10-06T22:16:59.562Z
---
From the 2026-10-06 audit: (1) fdu-design-principles.md ~:1025-1027 implies dut's lock-free atomic roll-up was implemented; it was not (fdu keeps one consumer; H62 rejected, exp-041). (2) performance-loop.md registry row ~:987 still marks H66 'Queued' though H172 carries it. (3) research-2026-08-06-file-rollup-engine.md read pdu and diskus from documentation only; anything crediting it with a source-level read of them should point to research-2026-09-28-pdu-and-the-linux-peer-gap.md instead.
