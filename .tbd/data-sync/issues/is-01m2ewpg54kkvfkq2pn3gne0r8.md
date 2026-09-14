---
type: is
id: is-01m2ewpg54kkvfkq2pn3gne0r8
title: "Stacked merge fails to compile: #48 f00bad5 calls index::path_is_relative_normal, which #51 50e6ca5 removed"
kind: bug
status: open
priority: 1
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T02:42:18.403Z
updated_at: 2026-09-14T02:42:18.403Z
---
Found while waiting for CI on PR #52 at ba83690 (run https://github.com/jlevy/fdu/actions/runs/34799679785). Every Rust-compiling job (15 of 19) failed with one error, and none came from #52's own changes:

    error[E0425]: cannot find function `path_is_relative_normal` in module `crate::index`
      --> crates/fdu-core/src/content/content_cache.rs:235:27

GitHub builds a stacked PR on its base PR's merge ref. This run checked out 470e848 ("Merge ba83690 into 2fff04a"), and 2fff04a is "Merge 66d9f17 into b774ecf", which is #51 on top of #50's merge ref. So #52's CI now compiles #48 + #50 + #51 + #52 together.

Cause: a semantic conflict with no textual conflict.
- #51 `claude/one-shot-commit-cost` 50e6ca5 ("fix: canonicalize public observation paths") deleted `pub(crate) fn path_is_relative_normal` from crates/fdu-core/src/index.rs. It folded the check into `canonical_relative_path`, which now matches components and returns PathEscapesRoot for ParentDir, RootDir, or Prefix.
- #48 `codex/opened-root-inventory-rewrite` f00bad5 ("fix(content): a sidecar record that leaves the root makes the sidecar a miss", fdu-m6n3) later added a call to `crate::index::path_is_relative_normal(&relative_path)` at crates/fdu-core/src/content/content_cache.rs:235.
- A local reproduction merging origin #48 (77e3afa), then #50 (6d37b62), then #51 (66d9f17) auto-merges cleanly and leaves content_cache.rs calling a function index.rs no longer defines.

Impact: #52 is red at ba83690, and #51's CI would also be red if it re-ran against the current #50 merge ref. Its last green run, at 66d9f17, predates f00bad5. #54 will inherit the error.

Fix, during the #50 -> #51 propagation merge: give content_cache.rs a component check equivalent to the removed predicate. Either restore a `pub(crate)` predicate in index.rs that `canonical_relative_path` and content_cache.rs both use, or inline `path.components().all(|c| matches!(c, Component::Normal(_) | Component::CurDir))` at the call site. Keep fdu-m6n3's Windows `\rooted` / `C:relative` and `..` cases rejected; its tests on #48 cover them.
