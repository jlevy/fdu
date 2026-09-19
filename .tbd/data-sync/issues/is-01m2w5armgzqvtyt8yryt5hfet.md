---
type: is
id: is-01m2w5armgzqvtyt8yryt5hfet
title: "H113: cache-only completeness should not re-walk analysis_candidates"
kind: task
status: open
priority: 1
version: 1
labels:
  - macos-agenda
  - campaign-2
  - performance
dependencies: []
created_at: 2026-09-19T06:23:18.662Z
updated_at: 2026-09-19T06:23:18.662Z
---
After a cache-only sidecar restore, open_for_report walks analysis_candidates again only to compare hits to len() (lib.rs ~598-602). exp-109 sampled that walk at 12.6% of content_open (~9% of wall). Completeness can use a count already known from restore and still refuse an incomplete sidecar.

Accept: content-cache-hit wall down at least 3% with the interval below zero on deciding-scale metabrowser; content digest identical; incomplete sidecar still refused.

Do this before fdu-jxhk. Do not retry parse-speed (H112).
