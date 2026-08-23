---
type: is
id: is-01m0pt93kk5pytsjrb0v5wrweq
title: "A group level: browsing taxonomy as its own axis"
kind: feature
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0pt9he483bx4et2eykcdp1j
  - type: blocks
    target: is-01m0ptezmtmkn04mh1f1rwgdxb
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T08:02:34.226Z
updated_at: 2026-08-23T08:19:18.571Z
---
ContentFamily is a fixed five-value enum answering an analysis question (which analyzer may open this). Every image, video, PDF, and archive therefore carries family='binary', so --view families over a photo directory is one row reading 'binary 100%'. Metabrowser's reference registry answers the browsing question with six groups (archives, code, data, docs, media, other) over 126 families. Add a group level to the rule dialect and to maintained roll-up state — its own axis, not a reinterpretation of the analysis families, which keep their present meaning — plus a groups view. Group breakdowns are pre-computed reads on the reducer path, never recomputed per request.
