---
type: is
id: is-01m3nbs9kfe7ygc6jx23j1byzt
title: "0.3.0: beat pdu and diskus on Linux for the default tree and .gitignore handling"
kind: epic
status: open
priority: 1
version: 4
labels: []
dependencies: []
child_order_hints:
  - is-01m3nbsak6hftpnr1rx5x90d1w
  - is-01m3nbsbrbr5v9y6r3x8pxgmr5
created_at: 2026-09-29T01:17:07.054Z
updated_at: 2026-09-29T01:17:09.258Z
---
Point solution from the 2026-09-29 design study (brief in this epic's notes). Gap (b), the default command on a real repo, is 81% consumer-thread .gitignore classification: a linear scan of ~111 governing patterns per entry, although 1,118 of the 1,593 linux-v6.12 rules are literal names or *.suffix. Fix: H171, bucketed matching. Gap (a), the tree view on the generated 1M tree, is the one-shot index build that no reader reuses. Fix: H172, an exact transient tree tier. H164 (fdu-emqf) follows on the ~0.5G residual. Predicted quiet ranking on linux-v6.12: default tree 211 -> ~85-95 ms (pdu 70, diskus 74.5); balanced tree 1.25 -> ~1.03-1.10 s (pdu 1.02). Ids H171/H172 come from the unused fdu-k1n8 block (H171-H179).

## Notes

# Verdict

**Point solution for 0.3.0: H171, bucketed `.gitignore` matching** (~150 lines in `gitignore.rs` + `control.rs`), prototyped and measured: consumer instructions on the `linux-v6.12` default tree **2,067M → 761M (−63%)**, summary 1,918M → 660M, answers byte-identical (0 non-timing differences over 1,629,566 JSON leaves; 93 control tests incl. the `git check-ignore` verdict table pass). **Second half of the pair, H172, an exact transient tree tier** (directories + the K largest files), closes gap (a): proxy cuts the balanced consumer 523M → 195M and RSS 66 → 20 MB; wall effect at 1M is a prediction (level with pdu), not confirmed by my 200k screen under load.

Why fdu loses today:
- **(b)** Classification is 81% of the consumer's instructions (1.68G of 2.07G) on the one thread that cannot scale: ~111 governing patterns per entry × ~140 instructions per `glob_matches` = 19.6k instr/entry. 63 of the root file's 106 rules and 1,118 of all 1,593 are literal names or `*.suffix`, answerable by a lookup. This is *removable overhead*, not work pdu skips.
- **(a)** The index build: 2.6k consumer instr/entry (10× a walker's per-thread work), 7 allocs/entry, 66 MB/200k → 8× the page faults, a fifth runnable thread on 4 vCPUs. Work pdu skips by design; after H159 no cross-thread frees remain, and no one-shot reader reads it.

# Evidence

Consumer-thread instructions, callgrind on a profiling build of e889694c (load-independent):

| Job, linux-v6.12 | Consumer | Walker (each) |
|---|---:|---:|
| default tree | 2,067M | 30M |
| tree `--no-gitignore` | 257M | 19–32M |
| summary, controls on | 1,918M | 55–64M |
| default tree, **H171 prototype** | 761M | 24–35M |
| summary, H171 prototype | 660M | 43–49M |
| default tree, H171 + H172 proxy | 588M | 24–27M |

Attribution (default tree, control): `ControlChain::is_ignored` 1,684M incl. (control.rs:765), `glob_matches` 1,423M (gitignore.rs:364; 1,258M self), `chain_for` 104M (control.rs:527), per-child path split in `with_components` ~76M (gitignore.rs:42, called at control.rs:769). Index build alone = the 257M: `derive_ext`/`intern_ext` 66M, child sort 34M, `contribution`+`merge` 52M (index.rs:5252, 1713–1715), malloc/free 48M.

Rule mix (358 files, 1,593 rules): 1,301 basename (1,118 literal, 88 `*.suffix`), 291 anchored, 1 `**`; mean 110.9 governing rules and 1.70 sources per entry. Root file: 37 `*.suffix`, 32 anchored, 26 literal (9 negated), 12 other globs.

| Balanced proxy (200k entries, recipe shape) | Consumer Ir | Allocs | Minor faults | RSS |
|---|---:|---:|---:|---:|
| default tree | 523M | 1.40M | 15.0k | 66 MB |
| summary `--no-gitignore` | 12M | 876k | 1.8k | 10 MB |
| tree, H172 proxy | 195M | — | — | 19.5 MB |

Tree consumer self: malloc/free ~100M, `InternedRollUp::merge` 83M, `derive_ext` 70M, `contribution` 55M, `ExtTally` insert 43M, sort ~50M, `HashMap<PathBuf>` ~56M (H167), `intern_ext` 32M. Proxy remainder: path hashing ~36M, sort 17M.

Screens (rusage, shared host; load average in brackets):
- linux-v6.12 [2–3]: fdu default tree 207–218 ms (user 0.17–0.21), `--no-gitignore` 91–180, summary 194–216, pdu 66–99, diskus 87–109. **Reference ceiling:** `rg --files` (ignore on, 86,615 files) 157–196 ms wall / 235–330 ms user vs `--no-ignore` 69–75 / 75 user; `-j1` 233–236 vs 73–76. ripgrep's compiled globset costs ~160 ms CPU here, 2× fdu's, and stays 2.2–2.7× its own no-ignore wall in parallel: a compiled matcher is not the lever.
- H171 vs control [7, tests compiling]: tree 281–316 → 194–222 (−27–31%); summary 231–378 → 194–225; `--no-gitignore` 157–201 both.
- H172 proxy on H171 [4]: linux tree 146–187 → 119–158, `--no-gitignore` 117–131 → 103–119, pdu 101–119; balanced tree 0.29–0.34 → 0.29–0.34 (unresolved), summary 0.25–0.29, pdu 0.23–0.25.

# Design

**H171 mechanism.** At `Gitignore::parse` (gitignore.rs:104) bucket each pattern: Basename with no metachar → `literal_names: HashMap<name, Vec<index>>`; Basename `*.suffix` with no other metachar → `suffixes` keyed by the text after the pattern's last `.`, each confirmed by `ends_with`; everything else → `residual`. `matches_components` (gitignore.rs:125) probes both maps with the entry's name and extension, scans the residual, and answers `patterns[max matching index].ignored` — last-match-wins, negations and `directory_only` preserved exactly. Add `ControlChain::is_ignored_within(dir_components, name, is_dir)` and split the listing's directory once in `push_directory` (index.rs:1684–1695) and per cached parent in `SummaryControls::classify` (execution.rs:797–801). Snapshot format unchanged (matchers are rebuilt from source bytes at control.rs:500). Optional in the same change: bucket anchored (`Fixed`) patterns by segment count (~80M more). Diff saved at scratchpad `aea5-prototype-h171-h172proxy.diff`.

**H172 mechanism.** `RetainedState::Tree` (execution.rs:25, chosen at :439–447 beside `summary_is_sufficient` :414) when route is OneShot, no index policy, no analysis, views == [Tree], unfiltered, population Include, min-share > 0, K = ⌈100/pct⌉ ≤ 65,536. `push_directory` allocates entries for directories only; each file is classified, folded into its parent's `all`/`unignored` roll-up scalars (no entry, no ext tally), and offered to a global bounded min-heap of the K largest by the request's size metric; unretained files add to the parent's `folded` tally (entries, files, bytes, allocated, ignored); their names stay in the listing so the walker frees them (scan.rs:3260, H159). `expand` adds `folded` to the Share omission at query_report.rs:3044 via `Index::folded_children(id)`, `None` on a full index. Exactness: `ShareThreshold::admits` (query_selection.rs:206–218) is false at total 0 and needs part×100 ≥ pct×whole, so at most ⌊100/pct⌋ files are ever rows whatever the sort key, and K+1 files tied at the boundary would exceed the total. Licensed by the architecture: "One-shot `report()` may use the derived-report plan when the request proves that retained state has no consumer" and execution.rs:21–23.

**Answers change:** nothing, on either.

**Hypothesis rows.**

| Id | Claim | Accept |
|---|---|---|
| H171 | After H162–H163, 81% of the default tree's consumer instructions on linux-v6.12 are a linear scan of ~111 governing patterns per entry, though most rules are literal names or `*.suffix`; indexing each source by name and suffix and taking the highest matching index answers them in two lookups. | `default-tree` (deciding) and `aggregate-summary`, controls on, linux-v6.12: wall −3% with the 95% interval below zero; consumer Ir −50%; placebos both arms `--no-controls` on linux-v6.12 and `default-tree` on balanced-1M include zero; peak RSS non-inferior; route differential identical. |
| H172 | A one-shot tree reads only directory roll-ups and at most ⌈100/min-share⌉ files, yet `fdu .` retains every entry (2.6k instr and 7 allocations per entry, 322 MiB at 1M). A `Tree` tier keeps directories plus the K largest files and folds the rest into roll-ups and omission tallies. | `default-tree` on balanced-1M (deciding): wall −3%, interval below zero, and peak RSS −50%; `default-tree` on linux-v6.12 non-inferior; placebos `aggregate-summary --no-controls` and `cold-scan-index` include zero; goldens and parity unchanged. |

**Tests.** H171: keep the linear matcher under `cfg(test)` and property-test random rule sets (literal, suffix, anchored, negated, dir-only, escaped, `[..]`) × random names against it; the existing verdict table. H172: mirror `compact_summary_equals_the_indexed_summary_under_every_control_case` (execution.rs:2350) as `transient_tree_equals_the_indexed_tree_under_every_bound_case` over workers 1/2/4/8 × both orders × min-share {1%, 0.5%, 10%, 0.01%} × size metric × sort keys × limit/breadth/depth × ties at K × zero-size trees × ignored files at the boundary × refused controls; `tests/parallel_equivalence.rs`; goldens; parity (CI); path-independence subset.

**Predicted ranking.** (b) quiet scale: consumer 2,067M ≈ 200 ms → 761M ≈ 60–75 ms, below the 81 ms walk floor, so default tree 211 → ~85–95 ms, summary 167 → ~75–85; with H172 ~80–90. pdu 70, diskus 74.5: fdu default +15–25% vs pdu, ≈ diskus; `--no-gitignore` level. Closing the rest needs H164 on the 0.5G residual. (a) harness scale: tree 1.25 adds 0.64 CPU-s over the 0.94 summary; H172 removes ~63% of consumer instructions and ~70% of faults ≈ 0.4 CPU-s → 1.03–1.10 s, level with pdu 1.02 and diskus 1.04 within the interval; ahead only with H167 or H166 on top.

**Residual risks.** H171: bucket semantics for `*.`, names ending in `.`, escapes (excluded by metachar test), non-UTF-8 bytes; two maps per distinct source (small; measure RSS). H172: a new model element (`folded`) and tier; every non-tree reader of a pruned index must be unreachable and asserted; heap ties; tiny thresholds; the proxy could not resolve wall at 200k under load. Both: one 4-vCPU virtualized regime.

# Rejected

- **H164** (classify on walkers): moves 1.8G instructions onto walkers doing 30M each; after H171 the residual is 0.5G and H164 is the natural follow-up, not the first move; also needs a public `Op::Upsert` field for the summary route and walker-side budget admission.
- **H166/H170**: target walker parking; the tree route's binding cost is the consumer (523M vs 50M per walker); H170 is summary-only.
- **H167/H168**: 56M and ~100M of a 523M consumer (H168 moot for folded files under H172).
- **Compiled globset**: ripgrep's costs 160 ms CPU on this tree, 2× fdu's linear matcher.
- **Pruning ignored subtrees**: default population is Include; nothing to prune on the default path.
- **Allocator**: −12–17% screened but a C dependency and +RSS (H74); H172 removes the allocations instead.

# Open questions

1. H172 carrier: a pruned `Index` with a `folded` tally on directory entries (~400–500 lines, `expand` +10) or `TreeNode`s built outside the index (`expand` abstracted, larger)?
2. Ship H171 alone in 0.3.0 and H172 in 0.3.x/0.4.0, given its size?
3. K ceiling and whether `--sort name/mtime/count` keep the tier (the proof says eligibility is size-only).
4. Add linux-v6.12 default (controls on) as a product job in the tool comparison, and state pdu's depth.
5. Register H164 as the sequenced follow-up to H171.

Worktree restored to e889694c with no changes; build directory, proxy tree and binaries deleted; scripts and the prototype diff remain in the scratchpad (`aea5-*`).
