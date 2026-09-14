---
type: is
id: is-01m2esgptzqkfqja3qfatsve2q
title: Key snapshots by scan scope so report, open, and --watch snapshots stop evicting each other
kind: feature
status: open
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-09-14T01:46:42.910Z
updated_at: 2026-09-14T20:54:29.559Z
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

## Notes

2026-09-14 (fix wave, PR #51 2237a70): the command-line half is cured without keying. `fdu --watch` now scans with read_controls: false (crates/fdu/src/cli.rs:536-552@2237a70) because no CLI view reads control state (fdu-1onj), so its snapshot carries the one-shot scope. A watch after a report that saved its snapshot now takes WarmRevalidate, and so does an analyzing report after a watch; a plain metadata `fdu PATH` still never reads under Auto, by the planner's cost rule rather than by scope. Pinned by crates/fdu/tests/watch_controls.rs a_watch_and_a_one_shot_report_start_warm_from_each_others_snapshot (red before the change: cold_scan after a watch).

What remains: open / fdu.open versus report still keep different scopes at one cache path and replace each other (pending fdu-agb6, or keying here); the CLI-and-Python report parity case in both orders is still owed. The README and CachePolicy::Auto wording from e3b5103/8640758 exists only on #52 (README.md:190-197 and crates/fdu-core/src/lib.rs:156-165 at ba83690) and becomes wrong when #51 propagates: it says `fdu --watch PATH` observes control state and that a watch after `fdu PATH` starts cold. The propagation pass must rewrite both to say that `fdu PATH` and `fdu --watch PATH` share the controls-off scope and warm-start from each other, and that only a default open / fdu.open keeps the controls-on scope. Not revised on #51 because that text is not there.

2026-09-14 DECISION (user, supersedes the default-off decision recorded earlier the same day): .gitignore information is built into the tool and the library, and is rolled up by default on every surface: CLI reports, --watch, library open, fdu.open/fdu.scan/fdu.report, and opened roots. Each request can turn it off (--no-gitignore on the CLI, read_controls=False in the library and Python). The typed 'not observed' answer from #57 stays, for requests that opt out. The CLI shows split totals, for example '1.2 GB (340 MB ignored)', plus --exclude-ignored and --only-ignored filters. Prerequisites before the default flips: fdu-1onj (the control budget degrades to partial instead of aborting), fdu-okne (a liftable bound named in the error), fdu-szkg (charges deduplicated by fingerprint), and a speed check against main with controls on. Consequence for this bead: with every surface observing by default, report, open and watch share one snapshot scope again, so the cache split disappears for defaults. Only an explicit opt-out produces a second scope.
