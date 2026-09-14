---
type: is
id: is-01m2esgp4bz57m5z341p2xgn7r
title: Typed Python Tree cannot request depth or include_ignored, and OpenedOptions has no registry field
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-14T01:46:42.186Z
updated_at: 2026-09-14T01:46:42.186Z
---
From PR #48 review 5192314101, "Prior findings". The two typed-Python gaps in the MetaBrowser notes of 2026-09-01 (https://github.com/jlevy/fdu/pull/48#issuecomment-5497607702) are still present. No finding ID was assigned, and the #48 disposition map does not mention them.

**Gap 1: `Tree` cannot request depth or ignore pruning from Python.** The typed dataclass `fdu.opened.Tree` has only `path` and `page` (`crates/fdu-py/python/fdu/opened.py:421-423` on codex/opened-root-inventory-rewrite), and its wire encoder sends only those. The binding accepts `depth` and `include_ignored` with defaults (`crates/fdu-py/src/opened_binding.rs:287-299` per the notes), but nothing in the typed API reaches them. So a Python caller, including the MetaBrowser adapter, cannot ask for a deeper tree page or an ignore-pruned tree page.

**Gap 2: `OpenedOptions` has no registry field.** Rust `OpenOptions.types` exists (`crates/fdu-core/src/opened.rs:103` per the notes). Python `OpenedOptions` (`opened.py:240-251`) exposes `batch_size`, `follow_symlinks`, `one_filesystem`, `prune_hidden`, `hidden_allow`, `exclude_special`, `max_files`, `observe`, and `journal_capacity`, but no type registry. The MetaBrowser contract supplies its File Rollup registry document per session, so the Python adapter cannot pass it.

**Fix.**
- Add `depth` and `include_ignored` to `Tree`, with the engine's defaults, validation in `__post_init__`, and the encoder.
- Add a registry field to `OpenedOptions` (the registry document or a parsed registry value) and pass it through the binding into `OpenOptions.types`. Map parse failures to the typed error family.
- Cover both in the Python tests and stubs, and in a session golden if one exercises tree depth.

Both block the production MetaBrowser adapter (fdu-2xfp).
