---
type: is
id: is-01kzzbrr9wd6q42pfmchffs9v6
title: "PR #20 R3: make benchmark headline selection fail closed"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01kzzanfm0vcgrcdmjwr90rcja
created_at: 2026-08-14T05:26:26.363Z
updated_at: 2026-08-14T05:38:24.322Z
closed_at: 2026-08-14T05:38:24.321Z
close_reason: record.py now raises for missing primary jobs and metrics while preserving null only for an existing job with no comparisons; added red-green regression coverage and all 83 benchmark tests pass.
---
record.py currently returns a null headline for a missing primary job or metric. Raise descriptive errors for absent names, preserve null only for an existing baseline job with no comparisons, and add regression tests.
