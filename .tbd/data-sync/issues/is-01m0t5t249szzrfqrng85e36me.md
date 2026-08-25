---
type: is
id: is-01m0t5t249szzrfqrng85e36me
title: Hidden-path admission as scope, with an exact-name allowlist
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T15:21:47.400Z
updated_at: 2026-08-25T05:53:25.372Z
closed_at: 2026-08-25T05:53:25.372Z
close_reason: |
  Shipped as `crates/fdu-core/src/admission.rs` plus wiring across all three surfaces.

  Engine: `HiddenPolicy` with `keep_all`/`prune_hidden`/`admits`/`fingerprint`,
  `admission::parse_policy` and `AdmissionError`, `ScanConfig.hidden`,
  `ScanScope.hidden_fingerprint`, and `ScanReport.control_dirs` ->
  `Index::adopt_pruned_control_dirs`. Snapshot format 2 -> 3: the scope record is
  positional, so a field added to it moves every byte after.

  Command line: `--hidden keep|prune` and `--hidden-allow LIST` on the Scope axis, with an
  axis-table row. Python: `ScanOptions(hidden=, hidden_allow=)`. fdu's own default is
  untouched -- a du replacement counts what is there, so pruning is opt-in and fingerprints
  to zero.

  Four things worth recording.

  1. The admission rule and the `dotfile` tag share one predicate on purpose. They are
     distinguished by what they do with an entry, never by which entries they mean: a
     second definition of hidden would make `--hidden prune` and `--not-tag dotfile`
     disagree about one file, and the disagreement would read as a bug in whichever
     surface was consulted second. Windows' FILE_ATTRIBUTE_HIDDEN is deliberately not
     read, for the same reason -- it would make one tree admit different entries on
     different platforms and the parity corpus would carry that forever.

  2. Admission is asked once, by name, before the stat, from all four listing loops the
     engine has. One predicate rather than four copies, because the copies are how a scan
     and a refresh come to disagree about which entries exist.

  3. The snapshot has to record where the pruned control files were. Binding a gitignore
     rule asks the index where the `.gitignore` files are, and pruning is exactly what
     removes them from it. The trap: under `CachePolicy::Auto` a revalidation re-walks and
     re-records them, so the section could be deleted and every assertion still passed. The
     warm-start test only says what it means under `Only`, which cannot touch the tree.

  4. Two copies of one rule is two messages. The CLI validated `--hidden` itself and the
     Python dataclass validated `hidden` again, giving `--hidden` against `hidden` and
     double quotes against single for one mistake; the parity harness recorded the pair as
     a difference between the surfaces. `admission::parse_policy` is the only judge now and
     both print its sentence. The one check left in Python is the bare-string allowlist,
     which is a Python shape mistake with no command-line spelling.

  Six new golden sessions, replayed by the parity harness as six exact matches. No new
  declared deviation.

  Not implemented, and no consumer in the engine asks for it: repository-root detection
  inside a pruned subtree. `.gitignore` is the only control file any enabled rule reads, and
  nothing looks for `.git` itself.
resolution: null
duplicate_of: null
---
Hidden-path admission as scope, split out of the old fdu-mvt3 during the 2026-08-24
restructure so the tag model does not carry it. This is the reconciliation's settled
answer: hidden files PRUNE at scope with an exact-name allowlist. They are not a tag
and not a plane, because the only named consumer wants them absent, and a tag would
still walk the .git, cache, and virtualenv trees it exists to exclude.

Distinct from the dotfile TAG the model ships, and the distinction is the axis test:
the tag filters with both numbers visible; this prunes, and a pruned entry does not
exist in the index at all.

WHAT LANDS: prune hidden components during the walk except a configured exact-name
allowlist; keep governing control files readable without retaining them; fingerprint
the rule and its allowlist into snapshot identity. ScanScope is positional in the
snapshot header, so the new field means a FORMAT_VERSION bump -- deliberate this time,
unlike the leaf-count case where recompute-at-load avoided one.

No dependency on the tag model. P2: metabrowser Phase 2 wants it, nothing in the
critical plane path does.
