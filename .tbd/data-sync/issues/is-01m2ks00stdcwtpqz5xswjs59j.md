---
type: is
id: is-01m2ks00stdcwtpqz5xswjs59j
title: "PR #67 review PR67-1: cache-status machine output carries no schema string"
kind: bug
status: closed
priority: 2
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2krzt6endqrw5gq2phas5g6
created_at: 2026-09-16T00:13:48.217Z
updated_at: 2026-09-16T03:49:47.170Z
closed_at: 2026-09-16T03:49:47.169Z
close_reason: "f39b701: cache status carries the fdu.cache/1 schema in JSON (first field), JSONL (its own envelope line) and YAML (first line), mirroring how fdu.report/N is carried; CACHE_SCHEMA constant with a promise test, the surface test now checks every fdu.<family>/<version> string in --docs and --skill, goldens, SKILL.md, README, cache-design.md and the CHANGELOG updated."
resolution: null
duplicate_of: null
---
crates/fdu-core/src/report_format.rs:1486-1493@816fcf7; docs/project/architecture/fdu-design-principles.md:450. render_cache_status emits {"caches": [...]} with no schema key in JSON, JSONL or YAML, so the recognized -> state change and the new absent row are a breaking shape change with no version a consumer can key on. DECISION (user): fix it. Add fdu.cache/1 to cache-status JSON, JSONL and YAML, carried exactly the way fdu.report/N is carried in each format. Carry it in Python's model too if Python exposes the status rows. Update goldens, parity recording if it changes, --docs and skill text, and this PR's CHANGELOG entry. fdu.report/* needs no bump.
