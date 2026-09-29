---
type: is
id: is-01m3nvebxp8jtjk6jwcfvq79g0
title: Survey the fastest .gitignore matchers from source and benchmark them against git check-ignore (before H171)
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies:
  - type: blocks
    target: is-01m3nbsak6hftpnr1rx5x90d1w
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
created_at: 2026-09-29T05:50:46.198Z
updated_at: 2026-09-29T05:50:46.807Z
---
Read from source (clone into attic/ at release tags): ripgrep crates/globset (Literal, BasenameLiteral, Extension, Prefix/Suffix via Aho-Corasick, RequiredExtension, RegexSet) and crates/ignore (how the winning match is chosen, per-directory stack, Candidate allocation); git dir.c (last_matching_pattern_from_list reverse scan with early exit, PATTERN_FLAG_NODIR/ENDSWITH/MUSTBEDIR/NEGATIVE, nowildcardlen literal-prefix precheck, exclude stack) and wildmatch.c; gitoxide gix-ignore/gix-glob/gix-dir; Sapling eden/scm/lib/pathmatcher TreeMatcher (matches_directory Yes/No/Maybe pruning); Mercurial rust/hg-core matchers.rs and filepatterns.rs; jj lib/src/gitignore.rs; libgit2 ignore.c; regex-automata multi-pattern lazy DFA with PatternSet. Build a throwaway bench outside the repo (crates pinned, >=14 days old) over linux-v6.12's entries and its 358 .gitignore files: instructions per entry (callgrind) for fdu's current matcher, the H171 prototype (diff in fdu-sdul notes), H171 plus anchored grouping, an H173 live-set sketch, ignore::gitignore::Gitignore, gix-ignore; every engine must agree with git check-ignore on every path. Adopt git's t3070-wildmatch and t0008-ignores cases as a cfg(test) conformance table and report fdu matcher bugs. Check glob_matches' worst case on patterns like *a*a*a*a*b. Output: recommended algorithm and changes to the H171/H173 rows. A first attempt in the 0.2.1 session was stopped before reporting; nothing from it is recorded.
