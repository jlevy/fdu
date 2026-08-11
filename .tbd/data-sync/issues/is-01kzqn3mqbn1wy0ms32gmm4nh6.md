---
type: is
id: is-01kzqn3mqbn1wy0ms32gmm4nh6
title: "P2: --cache-status and --cache-clear lifecycle flags"
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzqn3xvxg58z2dcwjevgd439
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:35:44.874Z
updated_at: 2026-08-11T16:53:35.743Z
closed_at: 2026-08-11T16:53:35.743Z
close_reason: cache module with cache_status/list_caches/clear_cache/clear_all_caches over a new bounded snapshot::read_header, fixing the opaque-hash-file problem; unrecognized files are listed but never deleted, absent dir is empty not an error. CLI --cache-status[=root|all] and --cache-clear[=root|all] run before scan validation, suppress the report, clear-before-status, echo target before acting, render through the format axis. require_equals needed because clap otherwise ate the positional PATH. 4 library tests plus 9 golden blocks.
---
Lifecycle flags on the same grammar, never side effects (Principle 10): --cache-status[=root|all] and --cache-clear[=root|all], both optional-valued and defaulting to root (the resolved PATH), following flowmark's optional-value pattern. They run before scan validation so they need no readable tree, suppress the report, and may be combined in one invocation with clear running first, then status. Output renders through the format axis (--format json works) so agents get cache observability without a second schema style. Clear echoes the cache directory before acting and reports 'Cache cleared.' or 'Cache already empty.' with no prompt and no --force. A report run never deletes anything. Golden coverage modeled on flowmark's cache-behavior tryscript suite, in a scratch XDG_CACHE_HOME: status with and without a cache, status=all listing multiple roots, clear idempotence (twice in a row), clear=all leaving unrecognized files untouched, and the json rendering of each.
