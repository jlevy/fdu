---
type: is
id: is-01m2esgptzqkfqja3qfatsve2q
title: Key snapshots by scan scope so report, open, and --watch snapshots stop evicting each other
kind: feature
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-09-14T01:46:42.910Z
updated_at: 2026-09-14T01:47:10.951Z
---
One cause behind two stack follow-ups: PR #51 review COMMIT-3's cache split (fdu-etfj) and PR #52 review BUILD-3 (fdu-ughl, closed as documented). The fixer's recommended fix is to key snapshots by scan scope.

**Problem.** A tree's default cache path holds one snapshot, but three consumers now want different scan scopes: controls-off one-shot reports (`fdu <dir>`, `fdu.report`), controls-on `open` / `fdu.open`, and `--watch`. Acceptance is exact wherever an index is returned or reconciled (#51 c0729ce, from #52 bcb27ca), so each mismatch cold-scans, and under `CachePolicy::Auto` the cold scan's save overwrites the other scope's snapshot.
- #52 BUILD-3: alternating `fdu DIR` and `fdu --watch DIR` makes each run replace the other's snapshot. This is correct and visible as `ReportSource::ColdScan`, but it changes the cost model. Documented on `CachePolicy::Auto` and in the README (e3b5103, corrected 8640758), with keyed snapshots explicitly not pursued. See `crates/fdu-core/src/lib.rs` `snapshot_scope_serves` and the save path (`lib.rs:334-352, 377-395` at afbb2ee).
- #51 COMMIT-3 follow-up: an `open` right after a report cold-scans, and then displaces the report's snapshot (see fdu-agb6, the decision on the `open` default).

**Fix direction.** Derive the snapshot location, or a per-scope slot within one cache entry, from `ScanScope` identity, at least the ignore-rules fingerprint that separates controls-on from controls-off. Report, open, and watch snapshots then coexist. Keep exact acceptance, and keep the one directional path: a no-scan `--cache only` report may still read a controls-on snapshot.

**Watch for.**
- Cache growth. One tree can now hold up to one snapshot per scope, which interacts with the missing retention policy (fdu-558j).
- Upgrade. The first run after the change finds no keyed snapshot and cold-scans once.

**Acceptance.**
- Alternating `fdu DIR`, `fdu --watch DIR`, and `fdu.open(DIR)` warm-starts each after its own first run.
- The README and `CachePolicy::Auto` wording from e3b5103/8640758 is revised.
- Add the parity case COMMIT-3 asked for: CLI and Python `report` against one cache path, in both orders. #51 covered it only at engine level, because the binding could not be built on that host.

Reviews: https://github.com/jlevy/fdu/pull/51#pullrequestreview-5192254822 and https://github.com/jlevy/fdu/pull/52#pullrequestreview-5192264318
