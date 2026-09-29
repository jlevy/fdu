# Research: Source-Line Counters Compared with fdu’s Code Analysis

**Date:** 2026-09-29

**Author:** fdu project, with Claude Code

**Status:** Complete.
The survey read each tool at source; the accuracy differential and the quiet speed
comparison on Linux v6.12 are measured.

## Overview

The README compares fdu with the disk-usage tools people reach for, and one of its rows
is source code analysis.
No disk-usage peer counts source lines, so that row needed reference columns from the
tools that do. This brief surveys the source-line counters, chooses scc and tokei as the
README’s columns, and measures both against `fdu --analyze=code` on the Linux v6.12
source: how often the three agree file by file, why they disagree where they do, and how
long each takes.

The short answers:

- **fdu’s C counts hold up.** On the kernel’s 59,953 C sources and headers, fdu and scc
  give the same code, comment, and blank counts for every file but 22, and all three
  tools agree on 59,766. The disagreements trace to five constructs, each reproduced
  below in a file of one to five lines: four are defects, one in scc and three in tokei,
  and the fifth is a convention.
  fdu gives the expected count for all four defects.
- **fdu is the slowest of the three on a first run.** With ignore rules off it took 7.92
  s, against scc’s 1.24 s and tokei’s 1.87 s. All three read about 1.46 GB; fdu spends
  6.2 times scc’s CPU doing it.
  Run again, fdu answers from its content cache in 0.55 s, ahead of both.
- **scc and tokei are far broader.** They count 366 and 333 languages against fdu’s 15,
  and on the kernel they counted 47 and 50 languages against fdu’s 6. fdu recognizes
  Assembly, Make, and Perl there but cannot count them.
- **fdu does what they do not:** code tallied per directory, a content cache that reads
  only changed files again, each language’s ignored share, and disk usage from the same
  pass.

## Questions to Answer

1. Which source-line counters are current, and which belong in the README matrix?
2. Where does fdu stand against them on features?
3. Do fdu, scc, and tokei agree on a large real C tree, and where they disagree, which
   is right?
4. How long does each take on that tree, with ignore rules off and on?

## Scope

**Included:**

- fdu at `ebc06c78`, whose engine is that of
  [#161](https://github.com/jlevy/fdu/pull/161) and
  [#162](https://github.com/jlevy/fdu/pull/162); it reports version 0.2.1-dev.
- scc 4.1.0 from the official `scc_Linux_x86_64.tar.gz`, checked against the release’s
  `checksums.txt`; the binary’s SHA-256 is `4763d743…4ab4022`.
- tokei 15.0.0, built with `cargo +1.97.1 install --locked tokei --version 15.0.0`;
  SHA-256 `0b168d48…f72c271c`. Both releases are older than the 14-day cool-off in
  [SUPPLY-CHAIN-SECURITY.md](../../../SUPPLY-CHAIN-SECURITY.md).
- Source reading, from shallow clones under `attic/` (not committed): tokei 15.0.0, scc
  4.1.0, cloc 2.10, linguist 9.7.0, gocloc 0.7.0, loc 0.4.1, polyglot 0.5.29, ohcount
  4.0.0, pygount 3.2.0, and onefetch 2.28.1.
- The subject: Linux v6.12 (tag `adc21867`), 86,618 regular files, 62 symbolic links,
  and 5,757 directories in 1.6 GB, with 358 `.gitignore` files.
- The host: a 4-vCPU Intel Xeon at 2.1 GHz with 15 GiB of memory, Linux 6.18.44 in a
  Firecracker KVM guest, ext4 on virtio; warm file cache, quiet host.

**Excluded:** macOS, cold caches, and scc with its default complexity count.
sloccount (2004) and tcount, a tree-sitter counter, were not cloned.

## Findings

### The Counters

Every cell was read from the tool’s source at the version shown.

| Feature | fdu 0.2.1 | tokei 15.0.0 | scc 4.1.0 | cloc 2.10 | linguist 9.7.0 |
| --- | --- | --- | --- | --- | --- |
| Languages counted | **15**; recognizes 40 code types and reports the other 25 as unsupported | 333 | 366 | 402 | 836 types, 563 of them programming; measures bytes |
| Code, comment, blank | ✅ | ✅ | ✅ | ✅ | ❌ its `sloc` is non-blank lines |
| Strings and nesting | raw strings, text blocks, heredocs, JavaScript regex; nested comments in Rust, Swift, Kotlin | nested comments in 34 languages | nested comments in 31 languages | regex stripping | — |
| Python docstrings | code | code; configurable | comment | comment; `--docstring-as-code` | — |
| Embedded languages | ❌ | ✅ HTML, Vue, Svelte, Markdown fences, Rust doc comments | ❌ | not checked | — |
| Generated, vendored, minified | flagged, never excluded | ❌ | `--gen`, `--no-gen`, `-z`, `--no-large` | `--no-autogen` | ✅ with `.gitattributes` overrides |
| Duplicate files | ❌ | ❌ | `-d` | unique by default | — |
| `.gitignore` | ✅ with or without `.git`; include, exclude, or only ignored; each language’s ignored share | ✅ only inside a git repository; also `.ignore`, `.tokeignore`, global excludes | ✅ also `.ignore`, `.sccignore`, `info/exclude`, global excludes | only with `--vcs=git` | committed tree only |
| Per file | `--view files --sort code_lines` | `--files` | `--by-file` | `--by-file` | `--breakdown` |
| **Per directory** | **✅** tree or list by `code_lines` | ❌ | ❌; its README says to run it once per directory | ❌ | ❌ |
| Complexity and cost | ❌ | ❌ | complexity in 287 languages, ULOC, DRYness, COCOMO, LOCOMO, git hotspots | ❌ | ❌ |
| Formats | text with color, JSON, JSONL, YAML | text with color, JSON; YAML and CBOR need a feature build | 12, including CSV, cloc YAML, HTML, SQL, OpenMetrics; no terminal color | text, CSV, JSON, YAML, XML, Markdown, SQL | text, JSON |
| Cache | ✅ content sidecar; unchanged files are not reopened | ❌ | ❌ | merges saved reports | incremental between commits |
| Library | Rust and Python | Rust | Go package; MCP server | ❌ | Ruby gem |
| License | MIT | MIT or Apache-2.0 | MIT | GPL-2.0 | MIT |
| Latest release | current | 2026-09-06 | 2026-09-08 | 2026-07-04 | 2026-08-26 |

scc’s `main` branch has 368 languages; the 4.1.0 release has 366.

The smaller counters:

- **gocloc** 0.7.0 (2025-03): 189 languages, no `.gitignore` support.
- **loc** 0.4.1 (2017, last commit 2022): its README points to scc and tokei.
- **polyglot** 0.5.29 (2020): 146 languages, no `.gitignore` support.
- **pygount** 3.2.0 (2026-04): Pygments-based; detects generated files.
- **ohcount** 4.0.0 (2019): GPL-2.0, in maintenance.
- **onefetch** 2.28.1 is not an independent counter; it calls tokei 15.

### Where fdu Stands on Features

**Ahead:**

- The only counter with per-directory code tallies.
- The only one with a persistent cache: a repeated run on the kernel took 0.55 s against
  scc’s 1.24 s, and warm runs were 22–92% faster on the small trees of the
  [paired-cost report](../reports/report-2026-09-26-code-analysis-paired-costs.md).
- Reports the ignored population separately, per language.
- Shows coverage: files in languages it cannot count, and files it cannot classify, are
  listed rather than dropped.
- Counts prose words in the same pass, and the same engine serves the command line,
  Rust, and Python.

**Level:** the line convention.
A line with both code and a comment counts as code in all three.

**Behind:**

- 15 languages against 333–402.
- No embedded languages, complexity estimate, generated or minified exclusion, duplicate
  detection, or CSV, HTML, or SQL output.
- The Python docstring rule is fixed; tokei and cloc let a user change it.
- Speed on a large tree, below.

COCOMO and LOCOMO cost estimates are a non-goal, per the
[codebase analysis brief](research-2026-09-26-codebase-analysis.md).

### Accuracy on Linux v6.12

With every ignore source off, the three tools counted the C family as follows.
fdu has no separate header language: its C row includes headers, and it labels 168
kernel headers C++ (below).

| Tool | Language | Files | Code | Comment | Blank |
| --- | --- | ---: | ---: | ---: | ---: |
| fdu | C, sources and headers | 59,786 | 26,059,137 | 4,229,132 | 4,286,449 |
| fdu | C++ | 175 | 45,208 | 23,676 | 9,502 |
| scc | C | 34,661 | 18,297,094 | 2,752,964 | 3,545,815 |
| scc | C Header | 25,278 | 7,802,626 | 1,499,048 | 749,187 |
| tokei | C | 34,661 | 18,301,279 | 2,751,125 | 3,543,469 |
| tokei | C Header | 25,292 | 7,804,229 | 1,500,959 | 749,686 |

Summed over C and C++ sources and headers, the totals agree to within 0.02%:

| Tool | Files | Code | Comment | Blank |
| --- | ---: | ---: | ---: | ---: |
| fdu | 59,961 | 26,104,345 | 4,252,808 | 4,295,951 |
| scc | 59,962 | 26,104,489 | 4,252,862 | 4,295,871 |
| tokei | 59,962 | 26,107,591 | 4,252,224 | 4,293,407 |

The file counts differ by one for two reasons.
scc and tokei count two `.inl` files as C++ headers, and fdu does not recognize `.inl`.
fdu counts `Documentation/docutils.conf` as C, 6 code lines, because its modeline probe
finds `mode: c` inside `mode: conf-colon` (`fdu-d0gc`).

Whole-tree totals differ by design, because each tool counts only what it recognizes:

| Tool | Languages counted | Files counted | Code | Comment | Blank |
| --- | ---: | ---: | ---: | ---: | ---: |
| fdu | 6 | 61,452 | 26,312,547 | 4,290,517 | 4,343,072 |
| scc | 47 | 81,820 | 29,725,820 | 4,465,118 | 4,906,599 |
| tokei | 50 | 81,894 | 29,615,689 | 4,577,792 | 4,901,942 |

fdu also listed 4,444 files in five languages it recognizes but cannot count: Make
(3,030), Assembly (1,338), Perl (72), CSS (3), and Objective-C (1). scc and tokei count
them, although scc takes the one `.m` file for MATLAB, and they also count Device Tree
(5,329 files), YAML, reStructuredText, and JSON, which fdu does not treat as code.
tokei’s totals include languages embedded in others, such as Markdown in Rust doc
comments.

#### File by File

Joining each tool’s per-file JSON by path over the 59,953 `.c` and `.h` files:

| Pair | Files with equal code, comment, and blank | Code lines apart | Comment lines apart | Blank lines apart |
| --- | ---: | ---: | ---: | ---: |
| fdu and scc | 59,931 | 91 | 1 | 90 |
| fdu and tokei | 59,788 | 6,288 | 3,835 | 2,453 |
| scc and tokei | 59,766 | 6,379 | 3,836 | 2,543 |

All three agree on 59,766 files (99.69%). In every one of the 165 files where fdu and
tokei differ, fdu and scc agree; in every one of the 22 where fdu and scc differ, fdu
and tokei agree. No tool was taken as correct.
Each disagreement was traced to a construct in the file, the construct was reduced to a
file of one to five lines, and all three tools counted it.
The tuples are (code, comment, blank); the last column counts the kernel files each
construct explains and the code lines by which the outlier differs there.

| Construct | Minimal file | fdu | scc | tokei | Kernel files, code lines |
| --- | --- | --- | --- | --- | --- |
| A line holding only a form feed | `int a;`, `\f`, `int b;` | (2, 0, 1) | (3, 0, 0) | (2, 0, 1) | 22, scc +91 |
| A `'"'` character literal | `char c = '"';`, `/* comment */`, blank, `int d;` | (2, 1, 1) | (2, 1, 1) | (4, 0, 0) | 42, tokei +4,672 |
| Code between two block comments | `/* 0 */ int a; /* x */` | (1, 0, 0) | (1, 0, 0) | (0, 1, 0) | 39, tokei −1,426 |
| Code after a multi-line comment closes | `/* a`, ` b */ int x;` | (1, 1, 0) | (1, 1, 0) | (0, 2, 0) | 26, tokei −30 |
| A macro’s line splice after a multi-line comment closes | `#define M(a) \`, `/* \`, `* text \`,` */ \`, `(a)` | (3, 2, 0) | (3, 2, 0) | (2, 3, 0) | 55, tokei −151 |

- **Form feed:** C treats `\f` as whitespace, so the line is blank.
  scc counts it as code, which accounts for all 91 of its code-line differences from
  fdu. Older kernel files such as `drivers/scsi/st.c` use form feeds as page breaks.
- **Quote literal:** tokei opens a string at the `"` inside `'"'` and counts every line
  until the next `"` as code, comments and blanks included.
  This is the defect the
  [codebase analysis brief](research-2026-09-26-codebase-analysis.md) found in Rust;
  tokei 15 has it in C too.
  `net/netfilter/nf_conntrack_sip.c` gains 333 code lines from it.
- **Code between comments:** tokei counts a line that starts with a closed block comment
  and ends with another as a comment, although code sits between them.
  `drivers/atm/idt77252_tables.h`, a table annotated this way, loses 768 of its 774 code
  lines.
- **Code after a close:** tokei counts the line on which a multi-line comment closes as
  a comment, whatever follows the close.
  With code after it, as in `drivers/atm/iphase.c`’s `IF_IADBG_EVENT*/ 0;`, that is a
  defect. With only a macro’s `\` after it, whether the line is code is a convention: fdu
  and scc count it as part of the macro’s text.

Four constructs are defects, one in scc and three in tokei, and fdu gives the expected
count in each; the fifth is a convention on which fdu and scc agree.
Three files, 9 code lines in all, where tokei counts more code than fdu and scc, were
not attributed. The file-level figures are the same with ignore rules on, less the one
file the kernel’s `tags` ignore pattern drops.

#### The Language Split

fdu labels 168 of the kernel’s 25,292 `.h` files C++; scc labels 14, and tokei none.
Five of fdu’s 168 are GCC plugin headers that are C++, and scc agrees on those.
The probe looks for C++ keywords as plain substrings of a header’s first 16 KiB, and 165
of the 168 matched only `namespace `, as in `struct pid_namespace *` or a comment.
That is `fdu-0lo2`, a defect in fdu’s C header probe.
It moves lines between C and C++ and changes no total; the summed C family above is
unaffected.

### Speed on Linux v6.12

fdu is the slowest of the three on a first run and the fastest on a repeated one.
Median wall time over 12 adjacent pairs, each peer against the fdu run beside it, with
the paired change and its 95% interval:

| Arm | fdu | scc | tokei |
| --- | ---: | ---: | ---: |
| Ignore rules off | 7.92 s | 1.24 s, −84% [−84%, −84%] | 1.87 s, −76% [−77%, −76%] |
| Each tool’s own ignore handling | 9.22 s | 1.29 s, −86% [−86%, −86%] | 1.98 s, −78% [−79%, −78%] |
| Ignore rules off, fdu repeated with its cache | 0.55 s | 1.24 s, +127% [+123%, +130%] | 1.86 s, +237% [+227%, +242%] |

CPU time and peak memory, as medians:

| Arm | fdu | scc | tokei |
| --- | ---: | ---: | ---: |
| Ignore rules off | 29.9 s, 126 MiB | 4.8 s, 230 MiB | 7.2 s, 150 MiB |
| Each tool’s own ignore handling | 31.0 s, 157 MiB | 5.0 s, 230 MiB | 7.4 s, 150 MiB |
| Ignore rules off, fdu repeated with its cache | 0.72 s, 142 MiB | 4.8 s, 241 MiB | 7.2 s, 150 MiB |

- **First run:** fdu takes 6.4 times as long as scc and 4.2 times as long as tokei, and
  spends 6.2 and 4.1 times the CPU. The three read about the same data: by its scan
  counters, fdu opened 86,611 files and read 1.48 GB, and scc and tokei read the 81,820
  and 81,894 files they recognize, 1.46 GB each.
  The gap is CPU per byte.
- **Ignore rules:** excluding ignored files adds 1.3 s to fdu, 16%, against 4% for scc
  and 6% for tokei; these are comparisons between cells, not paired.
  In single untimed runs, fdu’s `--ignored=include` took 0.2 s longer than
  `--no-gitignore`, so the cost is in the exclude mode, not in reading the rules.
- **Repeated run:** under the default cache policy, fdu’s first run writes a content
  cache, and later runs revalidate every file and reopen only those that changed.
  Here none had, and each run printed the same table in 0.55 s. Neither peer has a
  cache.
- **Memory:** on a first run with ignore rules off, fdu held the least, 126 MiB against
  230 and 150 MiB. With ignore rules on, tokei’s 150 MiB was 5% below fdu’s 157.
- **Context:** on small macOS trees in 2026-08
  ([performance checkpoint](../reports/report-2026-08-13-code-sloc-performance.md)), fdu
  was close to tokei and about 20% behind scc.
  On a 1.6 GB tree the gap is wider.

### Earlier Accuracy Results

- **2026-08-13, 15 adversarial fixtures:** fdu matched Tokei 14 on all 15 and SCC on 14;
  the difference was the Python docstring rule
  ([fixture comparison](../reports/report-2026-08-13-code-sloc-fixture-comparison.md)).
- **2026-09-26, 30 adversarial fixtures:** fdu 0.1.0 matched the hand-classified answer
  on 14 and Tokei 14 on 21; 8 failed in both
  ([codebase analysis brief](research-2026-09-26-codebase-analysis.md)).
- **Since then:** analyzer v3 closed the three fix beads from that study (`fdu-ov8o`,
  `fdu-lr38`, `fdu-f1m3`) with chunk-split tests for Rust multiline strings and
  lifetimes, C++ raw strings and C line continuations, Java and C# text blocks, C#
  verbatim strings, Ruby `%q` strings and heredocs, shell heredocs and arithmetic, SQL
  dollar quoting, PHP heredocs, and 12 JavaScript regex cases.
  The 30 fixtures have not been re-scored against v3 and the current peers.

## Recommendations

Add scc and tokei as README matrix columns, and name cloc in a footnote:

- **scc** is the fastest counter measured here and in its own benchmarks, the most
  featureful (complexity, cost estimates, generated and duplicate detection, 12 output
  formats, an MCP server), and actively released.
- **tokei** is the Rust ecosystem’s counter and a library: onefetch and crates.io’s line
  counts use it. It handles embedded languages and is widely packaged.
- **cloc** recognizes the most languages but runs on one thread in Perl, so it belongs
  in a footnote.

In the matrix, scc and tokei show “—” in the disk-usage rows, because they count source
lines, not disk usage.
The code-analysis speed row states the arm and the measured figures, fdu last.

## Next Steps

- [ ] Fix the C header probe (`fdu-0lo2`) and add kernel-style header fixtures.
- [ ] Profile the cold code-analysis path on this tree (`fdu-xpwv`): fdu reads the same
  bytes as scc but spends 6.2 times the CPU, and excluding ignored files adds 16%.
- [ ] Match modeline aliases as whole tokens (`fdu-d0gc`).
- [ ] Add the next language tier with fixtures: Assembly, Make, and Perl, all common in
  the kernel, then Lua, PowerShell, and Scala; recognize Kconfig and devicetree (`.dts`,
  `.dtsi`) files, and `.inl` as a C++ header.
- [ ] Show or exclude generated, vendored, and minified files in the code view, and add
  a minified-file heuristic; a one-line minified file costs memory the size of the file.
- [ ] Count code embedded in Markdown fences and in HTML or Vue scripts.
- [ ] Consider an optional branch-point analyzer, as scc does.
- [ ] Consider CSV or cloc YAML output for continuous integration.
- [ ] Re-score the 30 adversarial fixtures with fdu’s current analyzer, scc 4.1.0, and
  tokei 15.0.0.

## Methodology

The survey read each counter’s language table, walker, and command-line definition at
the version given, and checked release dates against the tags.

The measurement follows the survey’s protocol in two arms:

- **Ignore rules off**, on a copy of the tree without `.git`, made with
  `tar --exclude=./.git` so that no file has a second hard link: fdu’s shared-file
  accounting would otherwise do work the others skip.
  The copy matched the clone in every path, type, size, mode, symbolic-link target, and
  byte.
  - `fdu PATH --analyze=code --view=code --no-gitignore --cache=off --color=never --quiet`
  - `scc --no-gitignore --no-ignore --no-scc-ignore --no-gitmodule -c --no-cocomo --no-size PATH`
  - `tokei --no-ignore --hidden PATH`
- **Each tool’s own ignore handling**, on the clone:
  - `fdu PATH --analyze=code --view=code --ignored=exclude --exclude='.git/**' --cache=off --color=never --quiet`
  - `scc -c --no-cocomo --no-size PATH`
  - `tokei --hidden --exclude .git PATH`

Two protocol assumptions did not hold, and neither changes a figure:

- fdu does not walk `.git` in the second arm.
  It treats `.git` as ignored, so `--ignored=exclude` prunes it and `--exclude` repeats
  that; the scan counters show 0 files opened in it.
- Three tracked files, under `tools/testing/selftests/arm64/tags/`, match the kernel’s
  own `tags` ignore pattern, so every tool drops them in the second arm: one C file, one
  Makefile, and one `.gitignore`.

Pre-checks: the clone’s `.git/info/exclude` holds only comments, and the harness runs
each tool with only `LANG`, `LC_ALL`, and `TZ` set, so no global Git configuration or
excludes file applies.

Timing used the tool harness’s new line-count contracts: 3 warm-ups and 12 adjacent,
alternating pairs per peer, each anchored on an fdu run; the `/proc/stat` quiet gate
before and after every sample; peak memory from `wait4`. Each tool printed its text
table, which the harness captured to a file and parsed after the timed window.
Timing JSON would penalize tokei, whose JSON carries every file’s record.
The harness requires one parsable total row, no stderr, and a zero exit per sample, and
one answer per tool across all its samples; it cannot compare totals across tools, which
differ by design.

Accuracy used separate, untimed runs of the same commands with JSON output: per language
(`--format=json`, `-f json`, `-o json`), and per file for the differential (fdu’s files
view sorted in turn by `code_lines`, `comment_lines`, and `code_blank_lines`; scc
`--by-file -f json`; tokei’s per-file records).
fdu’s files view has a `blank_lines` sort too, but that is the line analyzer’s count of
whitespace-only lines, which includes blank lines inside comments; the code analyzer’s
blank count is `code_blank_lines`.

The results are in the evidence directory, gzipped:

- [Ignore rules off](evidence/sloc-tool-comparison-2026-09-29-no-ignore.json.gz),
  [each tool’s own ignore handling](evidence/sloc-tool-comparison-2026-09-29-gitignore.json.gz),
  and
  [fdu repeated with its cache](evidence/sloc-tool-comparison-2026-09-29-cached.json.gz):
  the harness documents, with every sample, the tools’ versions and hashes, the host,
  and the tree fingerprints.
- [Validation](evidence/sloc-validation-2026-09-29.json.gz): per-language totals for
  every tool and arm, every disagreeing file with each tool’s counts, the attribution of
  each to a construct, and the reproductions with their counts.

## References

- [scc 4.1.0](https://github.com/boyter/scc/releases/tag/v4.1.0) (MIT) and
  [tokei 15.0.0](https://crates.io/crates/tokei/15.0.0) (MIT or Apache-2.0)
- [cloc](https://github.com/AlDanial/cloc),
  [linguist](https://github.com/github-linguist/linguist),
  [gocloc](https://github.com/hhatto/gocloc), [loc](https://github.com/cgag/loc),
  [polyglot](https://github.com/vmchale/polyglot),
  [ohcount](https://github.com/blackducksoftware/ohcount),
  [pygount](https://github.com/roskakori/pygount),
  [onefetch](https://github.com/o2sh/onefetch), and
  [tcount](https://github.com/RRethy/tcount)
- [Codebase analysis brief](research-2026-09-26-codebase-analysis.md) and its
  [evidence](evidence/codebase-analysis-2026-09-26.json)
- [Fast file content metrics](research-2026-08-12-fast-file-content-metrics.md)
- [SLOC engine decision](../reports/report-2026-08-13-code-sloc-engine-decision.md),
  [fixture comparison](../reports/report-2026-08-13-code-sloc-fixture-comparison.md),
  and [performance checkpoint](../reports/report-2026-08-13-code-sloc-performance.md)
- [Code analysis paired costs](../reports/report-2026-09-26-code-analysis-paired-costs.md)
- [Content analysis in the usage guide](../../usage.md#analyze-file-contents)
- The counter and classifier:
  [`content_code_metrics.rs`](../../../crates/fdu-core/src/content/content_code_metrics.rs),
  [`file_type_detection.rs`](../../../crates/fdu-core/src/classify/file_type_detection.rs),
  and [`file-types.toml`](../../../crates/fdu-core/rules/file-types.toml)
- [The tool harness](../../../explorations/benchmarks/realtree/compare_tools.py) and
  [its guide](../../../explorations/benchmarks/README.md#source-line-counters)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
