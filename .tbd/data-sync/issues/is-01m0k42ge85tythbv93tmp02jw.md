---
type: is
id: is-01m0k42ge85tythbv93tmp02jw
title: Classify .pyc, .map, and .pack instead of leaving them unknown
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0k41ks4s0nxzfxj3v141nx8
created_at: 2026-08-21T21:36:46.023Z
updated_at: 2026-08-21T21:57:19.668Z
closed_at: 2026-08-21T21:57:19.667Z
close_reason: Landed; make check green (24 suites, 129 goldens).
---
Common file types fall through to `unknown`, which pushes real volume into an unhelpful
bucket. On a 180k-file tree `unknown` was the largest family at 43%, and the types view
showed:

    95 MiB  unknown:.map    171 files
    90 MiB  unknown:.pyc   4752 files
    77 MiB  unknown:.pack    22 files

- `.pyc` -- Python bytecode, a known binary format, and text analyzers must not open it
- `.map` -- a JSON source map; data, and machine-generated, so arguably `generated`
- `.pack` -- a git packfile; binary

Add these to the classification table and audit the extension list for other common
fall-throughs of the same kind. The `unknown:` prefix is doing its job -- it names what it
could not resolve rather than guessing -- so this is about the table being short, not about
the mechanism.
