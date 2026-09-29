---
type: is
id: is-01m3n77fy60sdcdzzk9qmbzg8e
title: "README: add a Comparison to Alternatives matrix (fdu vs du, dust, pdu, dumac, and peers)"
kind: task
status: in_progress
priority: 2
version: 16
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
delegate: claude-code@vm
labels:
  - docs
  - parity
dependencies: []
parent_id: is-01m3mvdz2891yheyemx49gzm6j
hold: null
hold_until: null
created_at: 2026-09-28T23:57:29.414Z
updated_at: 2026-09-29T18:57:44.674Z
started_at: 2026-09-29T17:02:10.729Z
---
Add a "Comparison to Alternatives" matrix to the main README, modeled on https://github.com/jlevy/repren#comparison-to-alternatives: tools as columns, features as rows, each cell a ✅ / ❌ or a few words ("partial: total only", "optional flag"), one intro sentence above, and a short "when to use each" note below that says plainly where a peer is the better choice.

Tools (columns; confirm the final set): fdu, GNU du, ncdu, dust, dua, gdu, pdu, diskus, dumac; consider scc or tokei as the reference for the code-analysis row only, or name them in a footnote, since they are not disk-usage tools.

Feature rows (from the maintainer, 2026-09-28), each orthogonal:
1. Plain usage: one total for a tree.
2. Speed: plain total and the basic tree, with the platform and figure from the published comparisons (macOS and Linux reports) rather than an adjective.
3. Tree breakdowns with flexible pruning: depth, breadth, share floor, row limits.
4. .gitignore support: classify, include, exclude, or show only ignored entries.
5. Source code analysis: languages, SLOC, comments, blanks.
6. Text file analysis: words, pages, paragraphs for Markdown and text.
7. APIs beyond the command line: list them (fdu: Rust fdu-core and the Python package; pdu: Rust library crate; dua: crates; diskus: small Rust library).
8. Watch and streaming support.
9. Cache optimizations for computed numbers (content sidecar, snapshot).
10. Agent-friendly skill.

Requirements:
- Every cell is checked against the tool's source or its current docs, not recalled. The pdu brief (docs/project/research/research-2026-09-28-pdu-and-the-linux-peer-gap.md, section E) already has the pdu parity; the 2026-08-06 survey matrix and the 2026-09-25 peer-agreement report cover the others. Cite versions checked in a footnote.
- The speed row states measured results with their conditions and links the reports. Where fdu is not first (on Linux today: the indexed tree against pdu and diskus, and the default command on source trees with many .gitignore files, about 3x pdu after H162/H163), the cell says so, or the work that closes it (H164, H166, H167) lands first and a fresh quiet matrix supports the claim. The README must not claim fdu is fastest where the evidence does not show it.
- Keep it scannable: at most about 10 rows and 10 columns; details in footnotes.
- Run flowmark on README.md; keep the Why section consistent with the matrix (it lists fifteen surveyed tools).

## Notes

# Matrix draft

## Comparison to Alternatives

There are many disk-usage tools. Here is how fdu compares with the ones people most
often reach for:

| Feature | fdu | du | ncdu | dust | dua | gdu | pdu | diskus | dumac |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Plain total | ✅ `--view summary` | ✅ `-s` | TUI only | ✅ `-d 0` | ✅ | ✅ `-s` | ✅ `-d 1` | ✅ | ✅ |
| Speed, 1M entries, macOS¹ | **6.4 s** | 56–68 s | 67 s | 11.0 s | 10.4 s | 10.5 s | 9.2 s | 9.3 s | 6.9 s |
| Speed, 1M entries, Linux¹ | SPEED_FDU_LINUX | 2.5 s | 2.7 s | SPEED_DUST | SPEED_DUA | 2.6 s | SPEED_PDU | SPEED_DISKUS | macOS only |
| Tree breakdown and pruning | depth, breadth, share floor, row limit | depth, size floor | TUI browsing | depth, top N, size floor | TUI browsing | depth, top N | depth, share floor | ❌ | ❌ |
| `.gitignore` | ✅ classify; include, exclude, or only ignored | ❌ | ❌ | ❌ | partial: `--ignore-from` | ❌ | ❌ | ❌ | ❌ |
| Source code analysis | ✅ languages, code, comment, and blank lines | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Text analysis | ✅ lines, words, pages | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| APIs beyond the CLI | ✅ Rust, Python | ❌ | JSON export | JSON output | Rust library | JSON export, database | Rust library, JSON | Rust library | ❌ |
| Watch and stream | ✅ `--watch`, JSONL change stream | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Cached results | ✅ snapshot and content cache, revalidated | ❌ | export, not revalidated | ❌ | ❌ | database, not revalidated | ❌ | ❌ | ❌ |
| Agent skill | ✅ `--install-skill` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Platforms | macOS, Linux, Windows | Unix; Windows via MSYS2 | Unix | macOS, Linux, Windows | macOS, Linux, Windows | macOS, Linux, Windows, BSD | macOS, Linux, Windows | macOS, Linux, Windows | macOS |
| Installation | `uvx` with no install; prebuilt wheels (uv, pip); cargo | preinstalled | distro, Homebrew | cargo, Homebrew, scoop, winget, binaries | cargo, Homebrew, scoop, winget, binaries | Homebrew, apt, go, scoop, winget, binaries | cargo, Homebrew, binaries | cargo, Homebrew, binaries | build from source |
| Output formats | color text and tree, paths, long, JSON, JSONL, YAML | text | TUI, JSON | color tree, JSON | color text, TUI | color TUI and text, JSON | tree, JSON | text | text |

¹ Median wall time on the same generated 1,000,001-entry tree with warm caches, each
tool doing its own job: fdu builds a reusable index and a ten-row tree; du, dust, dua,
diskus, dumac, and pdu at `-d 1` return a total; gdu renders a tree; ncdu builds its
browsable tree. macOS: an M1 Pro, 2026-09-26/28; Linux: a 4-vCPU virtualized ext4 host,
2026-09-29. See [Speed](#speed) for the conditions and for real source trees, where
reading `.gitignore` files costs fdu time the others do not spend.

Versions checked: fdu 0.2.1, GNU du 9.9, macOS du (file_cmds-479), ncdu 1.19–2.9.2, dust
1.2.4, dua 2.41.1, gdu 5.36.1, pdu 0.24.0, diskus 0.9.0, dumac `1ffbe3c`, each read at
source.

**When to use each.** ncdu, dua, and gdu let you browse a tree interactively and delete
from it, which fdu does not. du is everywhere already. diskus and dumac answer one total
with a tiny binary. fdu is the one to use when you need breakdowns you can prune, a
`.gitignore`-aware answer, content metrics, machine output, a live or cached view, or a
Rust or Python API.


# facts-diskus-dumac-fdu.txt

diskus/dumac/fdu facts received 2026-09-29 (see conversation); key: diskus lib yes, color no, JSON no, platforms mac/linux/win (Windows -b missing), install cargo/brew/nix/release binaries+deb; dumac: macOS only, source build only, no lib, output plain; fdu: formats text|tree|paths|long|json|jsonl|yaml, --color auto|always|never, NO_COLOR/FORCE_COLOR; platforms mac11+ x86/arm, linux glibc2.17+ x86/arm, win x86-64; install uvx zero-install, uv tool, pip, cargo install --locked, no brew; analyzers none|lines|code|words|all; watch --watch --interval jsonl fdu.stream/2; cache auto|on|off stale-ok; skill --install-skill --skill --docs

dust 1.2.4 / dua 2.41.1 / pdu 0.24.0 (received 2026-09-29, source-verified):
1 total: dust -d 0; dua PATH; pdu -d 1 (root only)
3 pruning: dust -d, -n top N lines, -z min size, --collapse, filters; dua none (TUI browse/sort); pdu -d (default 10), -m min-ratio 0.01
4 gitignore: dust none (-X excludes); dua partial (--ignore-from FILE gitignore syntax; TUI marks repo-ignored entries); pdu none
5/6 code/text: none for all (dust -t groups by extension)
7 APIs: dust none (bin only; -j JSON of display tree); dua lib target in dua-cli (dua-core from later tag); pdu lib crate parallel_disk_usage, --json-output/--json-input
8 watch: none (dua TUI live during scan, manual rescan)
9 cache: none (pdu --json-input re-render)
10 agent skill: none (pdu has contributor-only AI instruction templates)
11 platforms: all three for each; dust prebuilt linux gnu/musl many arch, macOS x86_64 only, windows; dua prebuilt linux musl, macOS x86_64+aarch64, windows x86_64/i686/aarch64; pdu prebuilt x86_64 only (linux gnu/musl, macOS, windows)
12 install: dust cargo du-dust, brew, scoop, winget, nix, release tarballs+deb; dua cargo dua-cli, brew, scoop, winget, nix, release binaries; pdu cargo parallel-disk-usage, brew, nix, Arch official, release binaries
13 output: dust bar-chart tree, color on TTY (NO_COLOR, -c off/-C force), -j JSON; dua lines + ANSI color unconditional, TUI with delete; pdu bar-chart tree bottom-up (--top-down), no color, --json-output, -b plain|metric|binary


# facts-du-ncdu-gdu.txt

GNU du 9.9 / BSD du (file_cmds-479) / ncdu 1.19,1.22 / ncdu 2.9.2 / gdu 5.36.1 (received 2026-09-29, source-verified in attic/):
1 total: du -s; ncdu TUI only; gdu -s (non-interactive -n)
2 speed: macOS 1M: GNU du 68.0 s, BSD du 55.6 s, ncdu 2.9.2 67.3 s (single thread), gdu 10.5 s; Linux: GNU du 2.53 s, ncdu 1.19 2.72 s, gdu 2.57 s
3 pruning: du -d, -t size floor; ncdu TUI browse/sort; gdu --depth, --top N
4 gitignore: none for all (glob/regex excludes only)
5/6: none
7 APIs: du none (FreeBSD du --libxo JSON/XML, not macOS); ncdu JSON export/import (2.x also binary export); gdu ncdu-format JSON, SQLite/Badger --db, Go packages undocumented; 5.37 web UI local API
8 watch: none (manual rescan in TUI)
9 cache: ncdu export + -f browse (no revalidation); gdu --db + -r reuse (no revalidation)
10 skill: none (gdu 5.37 AGENTS.md for contributors)
11 platforms: GNU du Linux (+macOS/FreeBSD as gdu via brew/ports; Windows via MSYS2); BSD du macOS/FreeBSD base; ncdu POSIX, no Windows; gdu Linux, macOS, Windows, BSDs, Android
12 install: du preinstalled (brew coreutils g-prefix); ncdu distro packages, brew (2.x built with zig), static Linux binaries; gdu release binaries, brew gdu-go, apt, pacman, nix, snap, conda, winget, scoop, go install, docker
13 output: du plain text no color; ncdu TUI + JSON (color default off in 1.22/2.x); gdu colored TUI, text colored on TTY (--no-color), JSON, DB files


# SLOC survey (fdu-61ez)

## SLOC tools compared with `fdu --analyze code`

I read every cell below from source cloned under `/home/user/fdu/attic/` (shallow clones of tokei, scc, cloc, gocloc, loc, polyglot, ohcount, linguist, pygount, onefetch) unless it is marked otherwise. Release dates come from git tags. I changed nothing in the repo and built nothing. The report runs past 1,200 words because of the README columns you added.

### 1. Comparison

| Feature | fdu 0.2.1 | tokei 15.0.0 | scc 4.1.0 | cloc 2.10 | linguist 9.7.0 |
|---|---|---|---|---|---|
| Languages counted | **15** (40 code types recognised; the other 25 report "unsupported") | 333 | 368 | 402 | 836 types, 563 of them programming; measures bytes |
| Code / comment / blank | ✓ | ✓ | ✓ | ✓ | ✗ (`sloc` means non-blank lines) |
| Strings, raw literals, nesting | ✓ raw strings, text blocks, heredocs, JS regex; nested comments in Rust, Swift, Kotlin | ✓ nested comments in 34 languages | ✓ nested comments in 31 languages | regex stripping | n/a |
| Python docstrings | code | code (a config setting makes them comments) | comment | comment (`--docstring-as-code` flips it) | n/a |
| Embedded languages | ✗ | ✓ HTML/Vue/Svelte, Markdown fences, Rust doc comments | ✗ | not checked | n/a |
| Generated / vendored / minified | flagged only, never excluded | ✗ | `--gen`/`--no-gen`, `-z` minified, `--no-large` | `--no-autogen` | ✓ plus `.gitattributes` overrides |
| Duplicate files | ✗ | ✗ | `-d` | unique files by default | n/a |
| .gitignore | ✓ works without a `.git` directory; include, exclude or only-ignored; reports each language's ignored share | ✓ but **only inside a git repo**; also `.ignore`, `.tokeignore`, global excludes | ✓ also `.ignore`, `.sccignore`, `info/exclude`, global excludes | only via `--vcs=git` | committed tree only |
| Per file | `--view files --sort code_lines` | `--files` | `--by-file` | `--by-file` | `--breakdown` |
| **Per directory** | **✓** tree or list sorted by `code_lines` | ✗ | ✗ (its README says to run it once per directory) | ✗ | ✗ |
| Complexity / cost estimates | ✗ | ✗ | complexity (287 languages), ULOC, DRYness, COCOMO, LOCOMO, git hotspots | ✗ | ✗ |
| Formats | text (colour), JSON, JSONL, YAML | text (colour), JSON; YAML and CBOR only in a `--features all` build; per-file stream | 12 formats including CSV, cloc-yaml, HTML, SQL, OpenMetrics; no terminal colour | text, CSV, JSON, YAML, XML, Markdown, SQL | text, JSON |
| Cache | ✓ content sidecar; unchanged files are not reopened | ✗ | ✗ | merges saved reports only | incremental between commits |
| Library | Rust and Python | Rust | Go package, plus an MCP server | ✗ | Ruby gem |
| License | MIT | MIT or Apache-2.0 | MIT | GPL-2.0 | MIT |
| Last release | current | 2026-09-06 | 2026-09-08 | 2026-07-04 | 2026-08-26 |

The smaller tools:

- **gocloc** 0.7.0 (2025-03): 189 languages, no .gitignore support.
- **loc** 0.4.1 (2017; last commit 2022): its README points people to scc or tokei.
- **polyglot** 0.5.29 (2020): 146 languages, no .gitignore support.
- **pygount** 3.2.0 (2026-04): Pygments-based, detects generated files.
- **ohcount** 4.0.0 (2019): GPL-2.0, maintenance only.
- **sloccount** 2.26 (2004, from memory, not cloned): offers COCOMO.
- **onefetch** 2.28.1 is not an independent counter; it uses tokei 15.
- **tcount** is a newer tree-sitter-based entrant; I did not clone it.

### 2. Where fdu stands

**Ahead**
- Only tool with per-directory code tallies.
- Only one with a persistent cache: warm runs were 22–92% faster in `report-2026-09-26-code-analysis-paired-costs.md`.
- Reports the ignored population separately, per language.
- Shows coverage explicitly: files in languages it cannot count, and files it cannot classify, are listed, not dropped.
- Counts prose words in the same pass.
- The same engine serves the CLI, Rust and Python, and ships an agent skill.

**Level**
- Uses the same line convention as the others: a line with both code and a comment counts as code.
- Speed on small macOS trees: fdu 11.9 ms, scc 9.7 ms, tokei 13.3 ms (233 files). So fdu is close to tokei and about 20% behind scc.

**Behind**
- 15 languages against 333–402.
- No embedded languages, no complexity estimate, no generated/minified exclusion, no duplicate detection, no CSV/HTML/SQL output.
- The docstring rule is fixed; tokei and cloc let you change it.
- COCOMO/LOCOMO are left out on purpose (per the 2026-09-26 research), so treat that as a non-goal.

**Gaps worth beads**
1. **C-header probe bug.** `resolve_c_header` in `crates/fdu-core/src/classify/file_type_detection.rs` looks for the plain substring `namespace ` anywhere in the first 16 KiB. That matches kernel code such as `struct pid_namespace *` and comment text, so some Linux `.h` files get labelled C++. The fix is to require the match at the start of a token or line and skip comments. It changes the language breakdown, not the line counts.
2. **Next language tier, each with fixtures.** Assembly, Perl and Make (all common in Linux), then Lua, PowerShell and Scala. Kconfig and devicetree files (`.dts`/`.dtsi`) are not recognised at all.
3. **Generated/vendored/minified.** Show or exclude them in the code view, and add a minified heuristic. One-line minified files currently cost memory the size of the file.
4. **Embedded code.** Markdown code fences, and scripts inside HTML/Vue.
5. **Optional `branch_points` analyzer**, as scc does it, after the differential in section 3.
6. **CSV or cloc-yaml output**, for CI (low priority).

### 3. Accuracy

- **2026-08-13, 15 adversarial fixtures:** fdu matched Tokei 14 on all 15 and SCC on 14. The one difference is the Python docstring policy.
- **2026-09-26, 30 adversarial fixtures:** fdu 0.1.0 matched the hand-classified answer on 14, Tokei on 21, and 8 failed in both.
- **2026-09-26, 107 real files:** 33 files differed. The largest difference is a Tokei bug: a `'"'` character literal adds roughly 900 lines to `scan.rs`.
- **Since then:** the three fix beads (fdu-ov8o, fdu-lr38, fdu-f1m3) are closed. Analyzer v3 has chunk-split tests for:
  - Rust multiline strings and lifetimes
  - C++ raw strings and C line continuations
  - Java and C# text blocks, C# verbatim strings
  - Ruby `%q` strings and heredocs
  - shell heredocs and arithmetic, SQL dollar quoting, PHP heredocs
  - 12 JS regex cases

**A fresh differential check is warranted:**
- The 14/30 score predates v3.
- Tokei 15 and scc 4.1 have never been measured.
- No large real tree has been compared.

Suggested method:
- Re-run the 30 fixtures stored in `evidence/codebase-analysis-2026-09-26.json` with all three tools.
- On linux-v6.12, compare per-file JSON: `fdu --view files`, `tokei --files -o json`, `scc --by-file -f json`.
- Compare C + C Header totals.
- Settle disagreements with minimal reproductions, not by treating a comparator as correct.

### 4. Quiet speed protocol (linux-v6.12, after the current benchmark)

**Install (both releases clear the 14-day cool-off):**
- `cargo install --locked tokei --version 15.0.0`, with its own target directory.
- scc 4.1.0: download `scc_Linux_x86_64.tar.gz`, check it against `checksums.txt`. Don't use `go install`: the host has Go 1.24.7 and scc needs 1.26.4.

**Setup:**
```
S=$SCRATCH/subjects/linux-v6.12        # tag v6.12 adc21867, has .git
M=$SCRATCH/subjects/linux-v6.12-src
mkdir -p "$M" && tar -C "$S" --exclude=./.git -cf - . | tar -C "$M" -xf -
export HOME=$(mktemp -d) GIT_CONFIG_NOSYSTEM=1
```
- Copy the tree for `$M`; don't use hard links. Every file would then have two links, and fdu's shared-file accounting may add work the other tools don't do.
- Pre-checks:
  - `grep -v '^#' "$S/.git/info/exclude"` should print nothing.
  - `git -C "$S" ls-files -ci --exclude-standard | wc -l` counts tracked files that ignore rules would drop; expect about 0.
  - Record `nproc` and each binary's SHA-256.

**Arm A (main comparison): ignore rules off, hidden files in, identical tree `$M`**
```
fdu "$M" --analyze=code --view=code --no-gitignore --cache=off --color=never --quiet
scc --no-gitignore --no-ignore --no-scc-ignore --no-gitmodule -c --no-cocomo --no-size "$M"
tokei --no-ignore --hidden "$M"
```

**Arm B: each tool's .gitignore handling on, tree `$S`**
```
fdu "$S" --analyze=code --view=code --ignored=exclude --exclude='.git/**' --cache=off --color=never --quiet
scc -c --no-cocomo --no-size "$S"
tokei --hidden --exclude .git "$S"
```
fdu's `--exclude` only filters what is reported, so fdu still walks and reads the roughly 30 files in `.git`; the other two skip it. That difference is small; note it in the results.

**Optional rows, labelled as such:**
- scc with its default complexity on (drop `-c`).
- fdu warm-sidecar hit (`--cache=auto` after a seeding run, with an isolated `XDG_CACHE_HOME`).

**Output and validation:**
- Time text output to `/dev/null`. Tokei's JSON includes every file's record, so timing JSON would penalise it.
- Validate with one untimed JSON run per tool per arm. Record per language: files, code, comment and blank.
- fdu opens every regular file (it counts lines for all text); the others read only files in languages they recognise. Report recognised files and bytes per tool as "work returned".

**Schedule:**
- Preferred: add `fdu-code`, `scc` and `tokei` contracts to `explorations/benchmarks/realtree/compare_tools.py`. That gives 3 warm-ups, 12 adjacent alternating pairs, the `/proc/stat` quiet gate and `wait4` peak memory.
- Fallback: `hyperfine -N -w 3 -r 20`, running the arms alternately.

### README columns: scc and tokei

- **scc** is the speed leader, both in its own benchmarks (Linux kernel on a 32-core VM: `scc -c` 842 ms, tokei 1.42 s) and in fdu's. It is also the most featureful (complexity, cost estimates, detection flags, 12 formats, MCP server) and actively released.
- **tokei** is the Rust-ecosystem standard and a library; onefetch and crates.io's line counts use it. It handles embedded languages and is widely packaged.
- **cloc** has the most languages but is single-threaded Perl, so it belongs in a footnote rather than a column.

| Row | scc | tokei |
|---|---|---|
| Plain total | n/a | n/a |
| Speed | (to be measured) | (to be measured) |
| Tree breakdown / pruning | ✗ per language and `--by-file`; no per-directory totals | ✗ per language and `--files` |
| .gitignore | ✓ honoured, can be switched off; no ignored-only view | ✓ honoured inside git repos; `--no-ignore*` |
| Source code analysis | 368 languages; code/comment/blank; complexity, ULOC, COCOMO/LOCOMO | 333 languages; code/comment/blank; embedded languages |
| Text analysis | ✗ lines only | ✗ lines only |
| APIs beyond CLI | Go package; MCP server | Rust crate |
| Watch / stream | ✗ watch; `csv-stream` | ✗ watch; `--streaming json` |
| Cache | ✗ | ✗ |
| Agent skill | MCP server (`scc --mcp`) | ✗ |
| Platforms | Linux, macOS, Windows, FreeBSD | Linux, macOS, Windows, BSDs |
| Installation | prebuilt release binaries, Homebrew, MacPorts, Scoop, Chocolatey, WinGet, Snap, pkg, `go install`, Docker | `cargo install`, prebuilt release binaries, Homebrew, MacPorts, conda, dnf, nix, winget, Scoop |
| Output formats | tabular, wide, json, json2, csv, csv-stream, cloc-yaml, html, html-table, sql, sql-insert, openmetrics; no colour | coloured table, JSON; YAML/CBOR need a feature build |

### 5. Sources

- **fdu docs:** `/home/user/fdu/docs/project/reports/report-2026-08-13-code-sloc-{engine-decision,fixture-comparison,performance}.md`, `report-2026-09-26-code-analysis-paired-costs.md`, `docs/project/research/research-2026-09-26-codebase-analysis.md` (plus `evidence/codebase-analysis-2026-09-26.json`), `research-2026-08-12-fast-file-content-metrics.md`, `docs/usage.md`.
- **fdu code:** `crates/fdu-core/src/content/content_code_metrics.rs`, `crates/fdu-core/src/classify/file_type_detection.rs`, `crates/fdu-core/rules/file-types.toml`, `crates/fdu-core/src/index.rs` (`for_each_analysis_file`).
- **tokei** (v15.0.0, 2026-09-06): `attic/tokei/languages.json`, `src/utils/fs.rs`, `src/cli.rs`, `src/language/embedding.rs`.
- **scc** (HEAD 2026-09-26, v4.1.0): `attic/scc/languages.json`, `README.md`, `processor/processor.go`, `vendor/github.com/boyter/gocodewalker/file.go`.
- **cloc** (v2.10; `--show-lang` from 2.11 dev): `attic/cloc/cloc`.
- **linguist** (v9.7.0): `attic/linguist/lib/linguist/{languages.yml,vendor.yml,repository.rb}`.
- **Smaller tools:** `attic/gocloc/language.go`, `attic/loc/src/lib.rs`, `attic/polyglot/LANGUAGES.md`, `attic/pygount/pygount/analysis.py`, `attic/onefetch/Cargo.toml` (`tokei = "15.0.0"`).
- **Web:** [tcount](https://github.com/RRethy/tcount).

# Status check 2026-09-29 (Linux overnight round, PR jlevy/fdu#161)

- Not landed: main's README has no "Comparison to Alternatives" section, and no remote branch carries one. This bead's notes are the only copy of the draft (durable on origin/tbd-sync).
- Linux speed row: the Linux overnight round measured the generated 1M-entry tree after H169 (12 pairs, 4-vCPU Firecracker ext4): fdu default tree 1.2 s, pdu default +3% slower, diskus +6% slower, pdu --max-depth 2 -4% faster; peak RSS 58 vs 93 MiB. Rerun a quiet tool cell on the final head (0c8131fd or later) before filling SPEED_* placeholders; the README must claim only what a current cell shows. Real trees at the final head (20 pairs): linux-v6.12 and node-modules-dense level with pdu default and diskus (+1% [-2%, +2%] and +1% [-2%, +4%]).
- .gitignore row, gdu: the draft marks gdu as no support (checked at 5.36.1), but upstream gdu (attic checkout, 2026-09-23) documents an `ignore-from-gitignore` option that reads gitignore-style patterns from one file. Re-check the released version before publishing; likely "partial: patterns from a file", like dua.


# Landed on claude/readme-comparison-matrix (2026-09-29, stacked on #161)

Branch head: 63e01e63. Commits: 9f968651 (measurement), 93bac1cf (README matrix and Speed), d2408cc3 (user docs revision, fdu-y3kb), 63e01e63 (merge of #161's latest branch). Checks: make check stages all passed (docs-format-check failed once on the report, fixed and re-run; path-independence re-run with the gate's smoke venv); post-merge docs-format, perf-test and evidence checks pass. PR body draft is with the coordinator.

- Linux speed row measured on the final head (ebc06c78, fdu-default-tree), linux-balanced-1m, quiet, 12 pairs per peer, 2026-09-29: fdu 1.088 s; pdu default +3.6% [+2.5, +5.2]; pdu --max-depth 2 -2.5% [-4.0, -1.4]; diskus +7.2% [+5.4, +9.9]; dust 1.2.5 +62%; gdu 5.37.0 +158%; GNU du 9.4 +160%; ncdu 1.19 +174%; dua 2.45.0 +234%. Peak RSS fdu 58 MiB, pdu default 93, dust 446, gdu 596. Published as a dated section at the top of report-2026-09-27-fdu-linux-tool-comparison.md, with fdu-linux-tool-comparison-result-2026-09-29-default-tree.json.gz.
- Peers installed at the latest release past the cool-off: dust 1.2.5 and dua 2.45.0 via cargo +1.97.1 install --locked; gdu 5.37.0 via go install (sumdb-verified module, reports version "development"); ncdu 1.19-0.1 from Ubuntu.
- Cells changed from the draft: dua re-read at 2.45.0 (.gitignore partial now includes TUI dimming of ignored entries; aggregate --depth; snapshots and diff; dua-core library). gdu re-read at 5.37.0: .gitignore stays ❌, `-G/--ignore-from-gitignore` exists only on unreleased main (4b179b0), footnoted. gdu plain total `-ns`. pdu cache "JSON, not revalidated" (--json-input), matching ncdu's export cell. Platforms, Installation, and Output formats rows cut to keep 11 rows (platforms in the versions footnote; formats merged into "APIs and machine output"). Code row names 15 languages; footnote gives scc 4.1.0 (366 languages) and tokei 15.0.0 (333).
- README Speed rewritten: Linux first (current engine), macOS labeled as the 0.2.1 engine; tagline and Speed bullet no longer say "fastest". Why corrected ("exactly one persists anything" contradicted the matrix).
- Not done here: the short SLOC survey brief under docs/project/research/ that the parity plan lists for this bead; Cargo.toml/pyproject descriptions and the CLI about string still say "Fastest native du replacement" (maintainer's call; changing them changes --help and its golden).

Stack layer (2026-09-29): branch claude/readme-comparison-matrix, draft PR jlevy/fdu#162, based on #161. Close when #162 merges.
