---
type: is
id: is-01m4gy0cxh70ysdbye8wtr2zr5
title: Walk several roots in one walker pool
kind: task
status: open
priority: 3
version: 5
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
hold: null
hold_until: null
created_at: 2026-10-09T18:15:03.856Z
updated_at: 2026-10-10T18:49:23.499Z
started_at: 2026-10-10T18:48:40.276Z
---
A report over several roots runs each root's plan one after another (crates/fdu-core/src/execution.rs, prepare_roots_internal), each with its own walker pool: a thread::scope, pool.initial workers, a calibration window that restarts at every root, and a drain tail, plus per-root planning and index build. PR #192 review C2 (https://github.com/jlevy/fdu/pull/192#issuecomment-6085911373) estimated that for `fdu */` over hundreds of small directories the per-root fixed cost could dominate one walk of their parent. perf_probe now takes a repeated --root (jobs roots-default-tree and roots-summary) so the cost is measurable; the exploratory measurement from #192 is in the notes. If the many-small-roots cell shows per-root overhead above about 10% of the parent walk at k around 100, pre-register a hypothesis for one walker pool across roots: a multi-seed walk with one sink and one .gitignore table per root, keeping every root's retained state and the alias check per root.

## Notes

Measured on #192 (exp-216, H196, macOS, uncontrolled host near load 34 on 10 cores, 12 pairs, probe 3f195a2b): over the cargo registry store's 625 crate directories (36,526 entries), roots-default-tree --child-roots took 1,719.5 ms against 234.5 ms for the store as one root (+570% [+507%, +732%], about 2.4 ms a root; peak RSS 9.1 to 20.8 MB), and roots-summary 762.4 against 201.2 ms (+313% [+137%, +391%], about 0.9 ms a root). Review C2 estimated ~0.3 ms a root; the measured cost is about eight and three times that. User CPU (+209%, +188%) and minor faults (+993%, +1,098%) grow with the roots, pointing at per-root fixed costs (a walker pool of pool.initial threads spawned and calibrated per root, per-root planning, folded index build and release, and on the summary tier over several roots the identified fold that states every directory). Next: profile roots-default-tree --child-roots to attribute, then pre-register a hypothesis for one walker pool across roots (multi-seed walk, one sink and one .gitignore table per root, per-root retained state and alias check kept).

Linux, 2026-10-10 (exp-221, quiet, 20 pairs, probe 0d6011da): a 272-crate copy of this host's cargo registry store (17,089 entries), roots-default-tree --child-roots 162.3 ms against 31.8 ms for the store as one root (+425.6% [+383.1%, +479.0%], about 0.48 ms a root), roots-summary 139.5 against 27.7 ms (+417.9%, about 0.41 ms a root). A fifth (tree) and a half (summary) of macOS's per-root cost, still about five times one walk of the parent: the 10% bar is passed on both platforms. The parent walk keeps four walkers busy (96 ms CPU in 32 ms wall); 272 small walks one after another cannot.
