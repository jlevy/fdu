---
type: is
id: is-01kzz1krveqzt0ap63a4wjk961
title: Normalize fdu product name casing
kind: task
status: in_progress
priority: 1
version: 4
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
created_at: 2026-08-14T02:28:57.317Z
updated_at: 2026-08-14T02:37:37.889Z
---
Use lowercase fdu consistently for the product and command name, by analogy with du. Preserve uppercase only inside conventional identifiers such as FDU_BUILD_VERSION or environment variables.

## Notes

Normalized every standalone product-name reference from FDU/Fdu to fdu across README, benchmark docs, reports, research, specs, code comments, and detection fixtures. Preserved uppercase conventional identifiers, environment variables, and binary cache magic. Corrected a touched Markdown report's stale 16 MiB analysis-limit statement. Repository-wide casing audit is clean and full make check passes, including 99 golden CLI cases. PR #18 merged during finalization; clean follow-up commit 4bfab2f is in PR #19 with CI running. Global fdu 0.0.1-dev+g4bfab2f is installed.
