---
type: is
id: is-01m0prgyv1eq0g0mzgntn1p4n6
title: "Partitioned tallies surfaces: --tags and --plane, Selection.plane, per-plane values"
kind: feature
status: open
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:31:54.336Z
updated_at: 2026-08-24T08:44:21.159Z
---
CLI scope axis --tags and selection axis --plane; Python Selection(plane=...), per-plane RollUp/Child/TreeNode values and per-entry tag bits so one children() call serves the dual-value listing; default plane 'all' keeps untagged behavior byte-identical. Goldens with a tagged fixture in every format, replayed by the parity harness.

## Notes

SHAPE UNDER REVIEW: see fdu-mvt3's GENERICITY REVIEW note (2026-08-24). The owner raised that gitignore should be one flag among several rather than the feature's name. The proposal decouples tags from planes -- unbounded tags, a small declared promoted subset -- and adds a tier to a tag rule so a content-tier tag cannot silently turn a metadata walk into a content walk. That changes what --tags and --plane accept here: --plane would take a promoted tag's name from a declared set rather than being one-per-enabled-tag. Not applied; awaiting the owner's call. Note mime type is NOT a tag under that proposal -- it is categorical and already served by the interned-key tally maps.
