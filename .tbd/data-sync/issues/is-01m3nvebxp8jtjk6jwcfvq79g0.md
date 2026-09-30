---
type: is
id: is-01m3nvebxp8jtjk6jwcfvq79g0
title: Survey the fastest .gitignore matchers from source and benchmark them against git check-ignore (before H171)
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
delegate: claude-code@vm
labels: []
dependencies:
  - type: blocks
    target: is-01m3nbsak6hftpnr1rx5x90d1w
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
hold: null
hold_until: null
created_at: 2026-09-29T05:50:46.198Z
updated_at: 2026-09-29T22:00:34.284Z
started_at: 2026-09-29T07:27:42.256Z
closed_at: 2026-09-29T22:00:34.284Z
close_reason: "Superseded. The survey's source half ran (research-2026-09-29-linux-peers-matchers-and-hot-path.md); its benchmark half was overtaken by measurement of the matcher it would have chosen: H171 (exp-178, accepted, -29.6% default tree) and H183 (exp-193) were measured directly, full glob evaluations fell from about 110 to 0.0019 per entry, and answers agree with git check-ignore on every path of linux-v6.12. Further matching work, if any, is ranked under fdu-faqa."
resolution: null
duplicate_of: null
---
Read from source (clone into attic/ at release tags): ripgrep crates/globset (Literal, BasenameLiteral, Extension, Prefix/Suffix via Aho-Corasick, RequiredExtension, RegexSet) and crates/ignore (how the winning match is chosen, per-directory stack, Candidate allocation); git dir.c (last_matching_pattern_from_list reverse scan with early exit, PATTERN_FLAG_NODIR/ENDSWITH/MUSTBEDIR/NEGATIVE, nowildcardlen literal-prefix precheck, exclude stack) and wildmatch.c; gitoxide gix-ignore/gix-glob/gix-dir; Sapling eden/scm/lib/pathmatcher TreeMatcher (matches_directory Yes/No/Maybe pruning); Mercurial rust/hg-core matchers.rs and filepatterns.rs; jj lib/src/gitignore.rs; libgit2 ignore.c; regex-automata multi-pattern lazy DFA with PatternSet. Build a throwaway bench outside the repo (crates pinned, >=14 days old) over linux-v6.12's entries and its 358 .gitignore files: instructions per entry (callgrind) for fdu's current matcher, the H171 prototype (diff in fdu-sdul notes), H171 plus anchored grouping, an H173 live-set sketch, ignore::gitignore::Gitignore, gix-ignore; every engine must agree with git check-ignore on every path. Adopt git's t3070-wildmatch and t0008-ignores cases as a cfg(test) conformance table and report fdu matcher bugs. Check glob_matches' worst case on patterns like *a*a*a*a*b. Output: recommended algorithm and changes to the H171/H173 rows. A first attempt in the 0.2.1 session was stopped before reporting; nothing from it is recorded.

## Notes

# Source-reading half (2026-09-29, session on host `vm`, taken over from claude-code@spud10 with the maintainer's agreement)

Full report kept in the session scratchpad as review-matchers.md; findings that bear on H171/H173:

- fdu already does git's reverse scan with early exit (`filter().map().next_back()`), but 99.24% of linux-v6.12 entries match no rule, so the scan saves 0.6% (110.23 of 110.92 patterns touched per entry).
- git 2.43.0 over all 92,438 paths: 91,731 no opinion, 702 negated, 5 ignored; a Python rule model reproduces the bead's counts (358 sources, 1,593 rules, ~111 governing rules, 1.70 sources per entry). 106 of the ~111 governing rules are the root .gitignore.
- H171 is exact: last-match-wins == highest matching index (ripgrep's Gitignore::matched_stripped uses the same rule). Middle-slash anchoring, leading `**/`, trailing `/**`, escapes, `[...]`, names ending in `.` all check out.
- As prototyped, H171 still touches ~44.76 residual patterns per entry (~4k instructions): the root's 32 anchored rules and ~11 wildcard basenames (`.*`, `*~`, `*.o.*`).
- Two cheap changes cut that to ~10.97: git's general ends-with form (`*tail`, any literal tail) instead of only `*.ext`; anchored grouping by segment count made required. Literal prefix/suffix/min-length pre-checks leave ~4.1 full globs per entry; a required-substring check leaves ~0 on this tree.
- H173's live set saves only 0.46 touches per entry on top of that (one `**` rule; 96.9% of entries see no live anchored rule). Re-scope as a stateless step in chain_for, justified only by a `**`-heavy subject.
- Also: store indices (not copied keys) so content_cost stays honest (~70-90 B per rule otherwise uncharged); hash the name once with an in-crate FNV; scan the residual descending and stop below the best bucket hit; rewrite `**/seg` as a basename rule; take (name, is_dir) per entry.
- Estimated per-entry cost: 0.4-0.7k instructions for the recommended design vs ~18k today (derived, not measured). Add a no-bucket arm (per-pattern compiled forms with git-style pre-checks, est. 1.3-1.8k) to show the maps earn their complexity.
- Divergences from git (git side confirmed): UTF-8 BOM not skipped; NUL inside a line not truncated -> fdu-ifci.
- Linux alone is weak evidence for buckets (400 entries decided by literals, 247 by *.ext): the property test and a synthetic built-output overlay are needed.
- Conformance checklist of 29 items referencing t0008-ignores.sh and t3070-wildmatch.sh.

# Benchmark half: still to do

Instructions per entry in the classify loop for fdu linear, H171 prototype, H171-revised, no-bucket compiled arm, ignore (published crate >=14 days old), gix-ignore; data sets: clean linux-v6.12, linux + virtual build outputs, a many-rules tree, adversarial 16 KiB lines; every path checked against `git check-ignore --no-index` with GIT_DIR in scratch.
