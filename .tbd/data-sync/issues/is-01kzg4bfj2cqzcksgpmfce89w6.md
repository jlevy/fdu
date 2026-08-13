---
type: is
id: is-01kzg4bfj2cqzcksgpmfce89w6
title: "Type-rule dialect: declarative rules compiled at build time"
kind: feature
status: closed
priority: 2
version: 7
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
  - type: blocks
    target: is-01kzg4d256qmchmtyvttnpvn4y
  - type: blocks
    target: is-01kzx1awzy1bantebrc5f6dke5
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:28:14.914Z
updated_at: 2026-08-13T08:22:29.163Z
closed_at: 2026-08-13T08:22:29.162Z
close_reason: Added a repository-owned [[kind]] manifest for 64 common file types; the existing build script now validates ids/families/priorities/extensions/exact basenames/shebangs, rejects ambiguity, emits native tables and a fingerprint, and retains git-version embedding. Classification is lossless over OsStr, preserves unknown extensions, and adds bounded 200-byte NUL/shebang detection for unresolved files. Clippy, all-feature tests, rustdoc, docs format, and cargo packaging pass.
---
Today classify.rs does compound-tail extensions only. Goal 6 requires pluggable recognition through a stable interface.

Rules as data in a TOML dialect deliberately compatible with metabrowser's [[kind]] manifests — priorities, a sub-class-of style category hierarchy, predicates for extensions, basenames, folder markers, path globs, and bounded content probes.

Compiled cheapest-first the way scc and tokei do it: extension hash maps, glob sets, and Aho-Corasick magic tables generated AT BUILD TIME, not parsed at runtime. scc generates a Go map literal; tokei renders a template into native match arms. That is what lets the same rule files evaluate at walk speed for millions of files.

Detection cascade ordered by cost, stopping at the first unambiguous answer (linguist, scc, tokei): exact filename -> extension -> shebang -> content heuristic.

Defining this early is the point: plugins must never need two rule languages. Validate the dialect against real metabrowser plugin manifests before freezing it.

## Notes

The 2026-08-09 Rust guideline audit found that the current extension path narrows OsStr through to_str and omits non-Unicode names even when their extension is ASCII. Complete fdu-k8zw first. The compiled dialect must define extension, basename, and glob semantics over native Unix bytes and Windows wide strings without lossy conversion.
