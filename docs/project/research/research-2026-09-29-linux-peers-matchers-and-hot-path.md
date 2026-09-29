# Research: Linux Peers, `.gitignore` Matchers, and fdu’s Hot Path

**Date:** 2026-09-29

**Author:** fdu project, with Claude Code

**Status:** Complete for the source reading.
It is the evidence behind
[the overnight plan](../specs/active/plan-2026-09-29-linux-overnight-performance-loop.md)
and its amendments, and it is the source-reading half of the matcher survey
(`fdu-p6vc`). It makes no new timing claim: wall figures are cited from recorded cells
or marked as screens.
Beads: `fdu-fkyf`, `fdu-p6vc`, `fdu-sdul`, `fdu-hb0u`, `fdu-dnfs`, `fdu-sfse`,
`fdu-leja`, `fdu-puk7`, `fdu-ifci`.

## Question

After H162 and H163, fdu’s default command on `linux-v6.12` still takes 2.4 times pdu’s
default (exp-175). Four questions decide what the overnight loop builds:

1. How do the fast Linux walkers work, read from source, and which of their techniques
   transfer to fdu?
2. How do git and four other implementations evaluate `.gitignore`, and is H171 as
   designed both exact and enough?
3. Where does fdu’s default command spend its work on Linux, stage by stage?
4. What model of wall time should the loop’s predictions use?

## Method and Regime

- **Source reading only.** Nothing was built for this brief.
  fdu was read at `e5a71c8a`, the 0.2.1 engine plus documentation; the engine source is
  unchanged at `7c46664b`. Peers and matchers were read at the commits in each table.
- **Evidence labels:**
  - **[src]:** read from source at the cited commit.
  - **[git]:** checked with git 2.43.0 `check-ignore --no-index` in a scratch
    repository, with user and system config off.
  - **[model]:** a Python model of `linux-v6.12`’s rules that counts pattern evaluations
    per entry under each matching scheme.
    It is not a timing or instruction measurement.
    Its verdict totals match git’s over the whole tree.
  - **[ext]:** code not checked out here: Rust std, glibc, gnulib, and the Go and Zig
    standard libraries. The Rust std claims agree with
    [the metadata-walk floor report](../reports/report-2026-08-23-metadata-walk-floor.md),
    which measured them.
- **Figures:** instruction counts are the per-thread callgrind runs of
  [the design study](research-2026-09-29-linux-default-tree-point-solution.md).
  Wall and CPU figures come from the quiet cells exp-174 (Linux 6.18.44-fc-v37) and
  exp-175 (fc-v49), or from `hyperfine` screens, which are labelled.
- **Host:** a 4-vCPU Intel Xeon at 2.1 GHz, a Firecracker guest, Linux 6.18.44-fc-v49,
  ext4 on virtio, warm cache.
- **Subject:** `linux-v6.12`, with 92,474 entries, 5,769 directories, and 358
  `.gitignore` files holding 1,593 rules.
  The rule model walks 92,434 entries in 5,758 directories, excluding `.git`.

## Findings

### 1. How the Fast Linux Peers Work

**Provenance** [src]. Deep dives: dut `68d4ba2` and bfs `37f6c7c` (C). Rust: pdu
`c30e46f` (0.24.0 plus 14 commits), diskus `90196e9`, dust `ae5c770`, dua-cli `d190513`,
jwalk `a5b1ea6`, ripgrep’s `ignore` `3fce3b5` with its users fd `ce97e47` and erdtree
`77199d9`, uutils `f7a9a7d`, walkdir `6fd031c`. Go: gdu `4b179b0`, fastwalk `1d58b22`.
Others: GNU du in coreutils `0e8455a` (gnulib not checked out), duc `c42b038`, ncdu
`1b3d0a6` (Zig), dumac `1ffbe3c` (macOS only).
dua-cli no longer uses jwalk: 2.40.0 replaced it with its own work-stealing walker
(`CHANGELOG.md:907-925`).

**No peer has a cheaper per-entry primitive than fdu on Linux.** Every one issues one
metadata call per entry, or skips it by `d_type` because it is not a du.
The fast ones differ only in how many path components each call resolves, what they
allocate and keep per entry, how the reduction is shaped, and how much concurrency they
run when the cache is cold.

**dut is the parallel floor with a top-N heap.**

- It reads raw `getdents64` into a 1 MiB per-thread buffer and passes each `d_name`
  straight from the kernel buffer to
  `statx(dirfd, name, AT_SYMLINK_NOFOLLOW | AT_NO_AUTOMOUNT, STATX_BASIC_STATS)`
  (`main.c:604`, `main.c:692-718`).
- A file costs no allocation unless it can enter the output heap (`main.c:643-647`).
- Roll-up is bottom-up on the walker threads.
  The last child to finish adds its totals into the parent with atomics
  (`main.c:564-588`).
- Its one waste is opening every directory by full path from `AT_FDCWD` (`main.c:680`),
  as fdu does.
- Its author’s benchmark, which was not reproduced here, puts the win in user CPU: 0.94
  s against pdu’s 2.9 s at similar system time.
  A screen on this host (11 runs, load 2.5) agrees: dut used 12 ms of user CPU against
  fdu’s 84 ms with `.gitignore` off, at 55 ms and 78 ms of wall.

**bfs’s io_uring path moves work to other threads; it does not batch.**

- One consumer thread runs the traversal; `min(nproc, 8) − 1` I/O threads each own a
  64-entry ring for `openat`, `statx` and `close` (`ioq.c:807-862`, `ioq.c:1011-1102`).
  `getdents64` stays synchronous (`ioq.c:695`, a TODO).
- `IORING_OP_STATX` always hands off to an io-wq worker [ext], which is why the floor
  report measured ring `statx` 5.9–7.6 times slower warm.
  Without liburing the same threads make plain system calls: the parallelism is the
  pool, not the ring.
- Its transferable part is fd management: an LRU of open directory fds, each directory
  opened relative to its nearest open ancestor, `EMFILE` eviction, a raised
  `RLIMIT_NOFILE`, and a pre-grown fd table (`bftw.c:1110-1185`, `eval.c:1602-1660`).

**pdu does not retain little.** Its default `--max-depth 10` keeps every node above
depth 10, files included, each with an owned `OsString` name, for the whole walk.
The 1% `--min-ratio` cull runs after the walk (`src/tree_builder.rs:50-69`,
`src/app/sub.rs:131-137`). Only the printed tree is small.

**Absolute-path statting is the norm in Rust and Go.** pdu, diskus, dust, jwalk,
`ignore`, erdtree, gdu and fastwalk stat each entry by full path, so the kernel resolves
every component for every entry.
dut, bfs, ncdu, GNU du, uutils, `dua-core` and fdu stat relative to the open directory.
Only bfs, ncdu, GNU du and uutils also open directories relative to a parent.

#### Comparison Matrix

Threads are for a 4-vCPU guest.
“rel” is relative to an open directory fd; “abs” is a full path from `AT_FDCWD`.

| Tool | Directory open | Read buffer | Per-entry metadata | Directories statted | Flags |
| --- | --- | --- | --- | --- | --- |
| dut | abs `open(O_DIRECTORY\|O_NOFOLLOW)` | raw `getdents64`, 1 MiB per thread | `statx` rel, basic stats | once, by the parent | `NOFOLLOW\|AT_NO_AUTOMOUNT` |
| bfs | rel, from the nearest open ancestor; LRU fd cache | `getdents64`, 64 KiB per directory | `statx` rel, lazy or eager | only if needed | `NOFOLLOW\|AT_NO_AUTOMOUNT` |
| pdu | abs | std `read_dir`, 32 KiB [ext] | `symlink_metadata` abs | yes, abs | std |
| diskus | abs | std, 32 KiB | `symlink_metadata` abs | yes, abs | std |
| dust | abs | std, collected | `symlink_metadata` abs | twice (`stat` and `lstat`), abs | std |
| `dua-core` | abs | std, 32 KiB | `DirEntry::metadata()` rel | yes, rel | std |
| gdu | abs | Go `os.ReadDir`, sorted | `Lstat(parent + "/" + name)` abs | `Stat` (following), abs | Go |
| `ignore`, fd | abs | std, 32 KiB | on demand, abs | no | std |
| fastwalk | abs `open` | raw `ReadDirent`, 8 KiB stack | on demand, abs | on demand | Go |
| GNU du | rel via `FTS_CWDFD` [ext] | glibc `readdir` | `fstatat` rel [ext] | yes | `AT_SYMLINK_NOFOLLOW` |
| ncdu | rel, ancestors held open | Zig iterator | `fstatat` rel | once | `AT_SYMLINK_NOFOLLOW` |
| fdu | abs, `fs::read_dir(&abs_dir)` | std, 32 KiB | `DirEntry::metadata()` rel | yes (mtime fingerprint) | std |

| Tool | Per-entry allocation | Retained | Threads | Distribution and reduction | Hard links | `.gitignore` | Teardown |
| --- | --- | --- | --- | --- | --- | --- | --- |
| dut | none for most files | per-thread top-N heap | 4 | global LIFO list; bottom-up atomics on walkers | per-directory tables merged upward, emptied as links complete | no | small heap |
| bfs | arena record, name inline | none (find) | 1 + 3 I/O | one consumer; ring queues to I/O threads | n/a | no | normal |
| pdu | `OsString`, `PathBuf`, task, node | full tree to depth 10 | 4 | rayon recursion; the return value | off; `-H` global map | no | normal drop |
| diskus | `PathBuf`, message | nothing | 12 (3 × cores) | rayon; one message per entry to a receiver | receiver set, `nlink > 1` | no | normal |
| dust | several `PathBuf`s, node | full tree | 4 | rayon scope; mutex per directory, bottom-up | serial post-pass, every node | regex list | normal |
| `dua-core` | `OsString`, `Arc<Path>` | nothing (aggregate) | 4 | LIFO deques and steal; wake one on steal; stat chunks of 4 | map, removed at the last link | user pattern files (gix) | `mem::forget` and `process::exit` |
| gdu | name and object per entry | top level only (stdout) | 8 directories | goroutine per directory, semaphore | inode only | one user file, as regexes | Go GC |
| `ignore` | `PathBuf`, message, `Arc` clone | caller’s | 4 | LIFO deques, batch steal; 1 ms sleep when idle | — | per-directory matchers | fd: jemalloc, `process::exit` |
| ncdu | arena, name inline | full tree | 1 by default | 16-slot LIFO queue | global map, `nlink > 1` | excludes on names, before stat | arena never freed, `exit` |
| fdu | `CString`, two name copies, index slot | full index | 4 walkers + 1 consumer | mutex queue, `notify_all`; one consumer, single writer | none | per-directory, on the consumer | detached release (H156) |

Nobody walks with `O_PATH`, uses `AT_STATX_DONT_SYNC` per entry, or uses
`posix_fadvise`, `madvise` or readahead for metadata.
Only the `ignore` crate discovers per-directory `.gitignore` files.
It builds matchers only for listings that contain one (`dir.rs:284-338`) and matches on
walker threads before queueing work.

#### Techniques That Could Make fdu Faster

Ranked by expected value for fdu, given the floor report’s settled items.

| # | Technique | Peers (source) | Stage | For fdu |
| --- | --- | --- | --- | --- |
| T1 | Consume the `getdents64` buffer in place; no per-entry name allocation | dut `main.c:692-718`; bfs `dir.c:257-336`; fastwalk `fastwalk_unix.go:37-75`; ncdu `scan.zig:189-199` | enumeration, allocation | H169 phase 1 (Q6); removes glibc’s `fstat` and `DIR` buffer as side effects |
| T2 | Names, not paths: build a path only when something needs it | bfs `bftw.c:196-258`, `:1552-1591`; ncdu `model.zig:65-212` | allocation, consumer | H177 (name arena); H86 family |
| T3 | Per-thread arenas never freed; exit without teardown | ncdu `mem_sink.zig:15-18`; dua `src/main.rs:290-315`; fd `src/exit_codes.rs:42` | teardown | command line only, after output and any snapshot write; measure the drop first |
| T4 | Walker-side bottom-up roll-up | dut `main.c:564-588`; dust `dir_walker.rs:415-452`; GNU du `du.c:651-705` | reduction | breaks the single writer; H174 keeps the writer and moves only listing-local work |
| T5 | Hard-link tables that shrink as links are found | dut `main.c:415-437`, `:516-560`; dua `src/inodefilter.rs:149-166` | reduction | only when fdu adds unique bytes; key by `(dev, ino)` |
| T6 | Cold: more workers than cores | diskus `walk.rs:169-175`; gdu `parallel.go:13` | parallelism | H76 (`fdu-tk1b`), bare metal |
| T7 | Open relative to an ancestor fd, with an fd budget | bfs `bftw.c:1110-1185`; ncdu; uutils | enumeration | dropped from H169 phase 1: the fd budget is the breadth-first frontier |
| T8 | Raise `RLIMIT_NOFILE`; pre-grow the fd table | bfs `eval.c:1602-1660` | enumeration | only with T7, and never from the library |
| T9 | Stat fan-out inside a wide directory | `dua-core` `lib.rs:67`, `:140`, `:1241-1264` | parallelism | H58 (`fdu-r9he`) |
| T10 | Publish subdirectories before statting files | fastwalk `fastwalk.go:205-240`; dua `lib.rs:1423-1436` | parallelism | mostly a cold-cache effect |
| T11 | Spin, then park; wake one worker per steal | bfs `ioq.c:278-345`; dua `lib.rs:903-925` | parallelism | H166; `ignore` sleeps 1 ms instead (`walk.rs:1975-1990`) |
| T12 | Match ignore rules on names, not paths | ncdu `exclude.zig:123-219`; `ignore` `dir.rs:284-338` | consumer | H171’s literal map |
| T13 | A custom allocator in the command line | fd `src/main.rs:41-57` (jemalloc) | allocation | the maintainer’s decision (H74) |
| T14 | Keep per-entry shared atomics off the hot path | anti-patterns: dust `dir_walker.rs:393-397`, diskus `walk.rs:108` | consumer | fdu’s counters are thread-local and off by default |

#### Techniques Not to Adopt

| Technique | Why not |
| --- | --- |
| io_uring `statx` and `openat` (bfs) | settled warm by the floor report; bfs’s design shows why |
| Bigger `getdents64` buffers (dut, bfs) | fewer calls only above about 1,000 entries per directory; a side effect of T1 |
| Eliding the terminating `getdents64` | nobody does it safely; bfs only moves it to an I/O thread (`dir.c:223-233`) |
| Skipping `statx` by `d_type` | a du reports sizes and fingerprints directory mtimes; H72 already takes what is safe |
| Top-N or top-level-only retention (dut, gdu) | changes what the index holds; H172 is the exact form, with K from the share threshold |
| Hard-link shortcuts (gdu, dust, dumac) | inode-only keys, or deduplicating every inode regardless of `nlink`, trade exactness |
| Narrow masks, `AT_STATX_DONT_SYNC` | the cost is VFS lookup, not fields (H25, H71) |
| `O_PATH` directory fds | cannot be passed to `getdents64` |

### 2. How `.gitignore` Is Matched

Revisions read [src]: git v2.56.0 (`a018953`), ripgrep `ignore` 0.4.33 and `globset`
0.4.20 (`3fce3b5`), gitoxide `8738598` (gix-glob 0.28.0, gix-ignore 0.23.0), libgit2
1.9.0 (`0551dfd`) and jj `d6270d1` (0.45.1). The gitoxide and jj checkouts are same-day
heads, so a benchmark must pin published crates that clear the 14-day cool-off.
Sapling and Mercurial were not checked out.

#### 2.1 What fdu Does Today

- `matches_components` is `iter().filter(..).map(..).next_back()`
  (`control/gitignore.rs:125-131`). `Filter::next_back` is `rfind`, so fdu already scans
  in reverse and stops at the first match, as git does.
- The early exit does not help on this tree.
  99.24% of entries match no rule
  [git: 91,731 no opinion, 702 negated and 5 ignored, of 92,438 paths], so the scan
  touches 110.23 of 110.92 governing patterns per entry [model].
- Every pattern runs `glob_matches`, a single-backtrack loop.
  There is no literal, ends-with or prefix fast path; bracket classes are re-parsed for
  every byte tried; a basename rule tests directory-only after the glob
  (`control/gitignore.rs:217-219`).
- `glob_matches` cannot go exponential.
  Only the last `*` is ever resumed, so the worst case is O(|pattern| × |name|), bounded
  by the 16 KiB line limit.
- Precedence is git’s: the deepest governing source with an opinion wins, and an ignored
  parent short-circuits its children.

#### 2.2 git’s Algorithm

**Parse** (`dir.c`: `add_patterns_from_buffer` :1230, `parse_path_pattern` :702):

- A leading UTF-8 BOM is skipped.
  A line starting with `#` is skipped before trimming.
  One `\r` is removed, then unescaped trailing spaces; tabs are kept.
- Patterns are C strings, so an embedded NUL truncates the line.
- Flags: `NEGATIVE` for a leading `!`; `MUSTBEDIR` for a trailing `/`; `NODIR` when no
  `/` remains; `ENDSWITH` when the pattern is `*` followed by no glob-special character;
  and `nowildcardlen`, the literal prefix up to the first `*?[\`.

**Per entry** (`last_matching_pattern_from_list` :1428):

1. Scan the list in reverse; the first match wins.
2. Check `MUSTBEDIR` first.
   A symlink is not a directory.
3. A `NODIR` pattern goes to `match_basename` (:1333): a literal compares length, then
   bytes; an `ENDSWITH` pattern compares length, then the tail; anything else runs
   `wildmatch`.
4. Any other pattern goes to `match_pathname` (:1357): the base directory prefix, the
   literal-prefix check, an immediate accept for a literal, then `wildmatch` with
   `WM_PATHNAME`.

**Precedence.** `prep_exclude` (:1654) keeps one list per directory level.
A directory is judged by the lists above it, never by its own `.gitignore`; an excluded
directory short-circuits every descendant, and git does not descend into it.
Command-line patterns come first, then the per-directory files deepest first, then
`info/exclude` and `core.excludesFile`.

`wildmatch` is recursive backtracking with the `WM_ABORT_ALL` and `WM_ABORT_TO_STARSTAR`
cut-offs, which keep it polynomial.
On this tree a non-matching entry costs git about 111 flag checks and length compares.
git has no hashing and no bucketing.

#### 2.3 Other Implementations

|  | fdu now | git | ripgrep | gitoxide, jj | libgit2 |
| --- | --- | --- | --- | --- | --- |
| Order within a file | reverse, early exit | reverse, early exit | all hits, highest index | reverse, early exit | reverse, early exit |
| Literal fast path | no | length and `memcmp` | hash (basename, full path) | equality | no |
| `*tail` fast path | no | `ENDSWITH` | extension hash, `*.ext` only | `ENDS_WITH` | no |
| Literal-prefix check | no | `nowildcardlen` | via the regex set | `first_wildcard_pos` | no |
| Directory-only checked first | anchored rules only | yes | after matching | yes | yes |
| Multi-pattern automaton | no | no | lazy DFA for the rest | no | no |
| Per-directory work | chain per listing (H163) | list stack | a node per directory | stack, matched-directory memo | file stack |
| BOM skipped | **no** | yes | yes | yes | not checked |

- **ripgrep** (`ignore/src/gitignore.rs`, `globset/src/lib.rs`) translates each line to
  a glob and sorts globs into literal, basename-literal, extension, prefix, suffix,
  required-extension and regex strategies.
  `Gitignore::matched_stripped` collects every matching index, then takes the highest
  that satisfies directory-only: H171’s rule, independently.
  Read from source, it diverges from git by trimming all trailing whitespace, reading an
  unclosed `[abc` as a literal, making `foo\/` directory-only, and dropping the rest of
  a file at a non-UTF-8 line.
- **gitoxide** (`gix-glob`, `gix-ignore`) ports git’s flags, including `ENDS_WITH` and
  `first_wildcard_pos`, scans in reverse with early exit, and caps `wildmatch` recursion
  at 64. **jj** wraps one `gix_ignore::Search` per `.gitignore` and inherits it.
- **libgit2** sends every rule to `wildmatch` with no literal path.
  From reading only, `parse_ignore_file` drops a negation with no wildcard unless it
  negates an earlier rule in the same file (`ignore.c` `does_negate_rule` :103). The
  lesson for fdu: never prune a negation using one file’s context, because precedence
  crosses files.

#### 2.4 The Rule Model on `linux-v6.12`

The tree has 358 sources, 1,593 rules and 332 distinct contents [model]:

| Class, as git sees it | Rules | Examples |
| --- | ---: | --- |
| Basename literal | 1,118 | `Makefile`-style names |
| Basename `*.ext` | 88 | `*.o`, `*.ko` |
| Basename `*literal`, dot-less tail (git `ENDSWITH`) | 15 | `*`, `*~`, `*_64`, `*uuid` |
| Other basename wildcard | 80 | `.*`, `vmlinux*`, `*.py[cod]`, `*.o.*`, `\#*#` |
| Anchored, all-literal segments | 268 | `/vmlinux`, `/include/config/` |
| Anchored with a wildcard | 23 | `/arch/*/include/generated/`, `/[gmnq]conf` |
| `**` | 1 | `/**` in `tools/testing/selftests/kvm` |

The root `.gitignore` has 106 rules and governs every entry, so 106 of the 110.92
governing rules per entry are the root’s. Those are 37 `*.ext`, 31 anchored literal, 26
literal, 10 other wildcard, 1 anchored wildcard and 1 `*~`.

| Scheme | Patterns touched per entry [model] |
| --- | ---: |
| Governing rules (1.70 sources per entry) | 110.92 |
| Today: reverse scan with early exit | 110.23 |
| H171 prototype: residual after literal and `*.ext` maps | 44.76 |
| H171 with git’s ends-with bucket and anchored rules grouped by segment count | 10.97 |
| The same plus a per-directory live set (H173-lite) | 10.51 |
| Full globs left after literal-prefix and suffix checks | 4.11 |
| Full globs left after a required-literal-substring check | about 0 |

- The last row comes from four survivors, `*.c.[012]*.*`, `*.o.*`, `*.asn1.[ch]` and
  `*.tab.[ch]`, whose required literals occur in essentially no name.
- Entries decided by each kind: 400 by literals, 247 by `*.ext`, 41 by other basename
  wildcards, 14 by `**`, and 1 by an anchored literal.
  The kernel tree is therefore weak evidence for any bucket: a property test and a
  synthetic built-output overlay are needed as well.
- Cheap byte filters are weak here.
  A root literal’s first byte passes 43% of names, its length 80%, both together 35%; a
  suffix’s last byte passes 93%. Hashing wins for literals; prefix, suffix and
  required-substring checks win for wildcards.
- Derived, not measured: the matcher is 81% of 2,067M consumer instructions, about 18k
  instructions per entry or 150 per pattern.
  The prototype’s residual is about 4k per entry.

#### 2.5 H171 Is Exact

H171 answers with the pattern at the highest matching index across a literal map, a
suffix map and a residual list.
Checked rule by rule [src]:

| Concern | Verdict | Why |
| --- | --- | --- |
| Last match wins | exact | Within one file, the last matching line is the highest matching index |
| Negation | exact | The pattern at the highest index decides, and its `ignored` flag is the answer |
| Directory-only | exact | `admits_kind(is_dir)` applies to bucket hits; the residual’s `matches` checks it |
| Middle `/` anchors | exact | Any `/`, escaped or inside `[...]`, makes the rule `Fixed` or `Spanning` and sends it to the residual |
| `*.ext` key | exact | If the tail is `A.B` with no `.` in `B`, a name ending in `A.B` has its last `.` exactly there |
| Leading `**/`, trailing `/**`, escapes, `[...]` | exact | All go to the residual; `**/seg` could be rewritten as a basename rule |
| Rejected lines | exact | Indices are assigned after rejection; only relative order matters |

#### 2.6 The Revised Design

What the prototype leaves is performance.
Its 44.76 residual touches are about 33 anchored rules, each failing on a segment-count
compare after several calls, and about 11 basename wildcards, each running the full glob
at 100–150 instructions.
Per entry it also makes 2 SipHash probes per source and initializes a 32-slot component
array.

The revision, adopted as Q2 in the overnight plan:

- **(a)** The suffix bucket becomes git’s ends-with form: every `*tail` with no special
  character. Tails with a `.` are keyed after their last `.`, dot-less tails go in a
  short `ends_with` list, and `*` alone is one match-all index.
- **(b)** Grouping anchored rules by segment count becomes required.
  With (a) it takes the residual from 44.76 to 10.97 touches per entry.
- **(c)** Residual rules carry precomputed cheap checks — directory-only, minimum
  length, literal prefix, literal suffix, a required literal — and are scanned in
  descending index order, stopping below the best bucket hit.
- **(d)** Literal map values are `(any, files)` maximum indices, stored as indices into
  the pattern arena, not copied keys.
  Copied keys cost about 70–90 bytes per rule that `content_cost` does not charge.
- **(e)** An in-crate FNV-1a hash, not SipHash; ideally the name is hashed once per
  entry and every governing source probed with it.
- **(f)** The per-entry call takes `(name, is_dir)`, with the listing’s directory split
  held in the chain.
- **(g)** A pre-registered extra arm: git-style compiled forms with no buckets,
  estimated at 1.3–1.8k instructions per entry, so the maps must earn their complexity.
- **(h)** The linear matcher stays under `cfg(test)` as the property-test oracle, beside
  git’s `t3070-wildmatch` rows and a differential against `git check-ignore`.

Estimated cost for the revision: 0.4–0.7k instructions per entry against about 18k
today. H175, deriving each listing’s chain from its parent’s instead of
`ControlTable::chain_for`, rides in the same cell (about 104M consumer instructions on
this tree).

#### 2.7 H173 Is Demoted

Carrying a per-directory live set of anchored and `**` rules down the walk is correct in
principle, but it saves 0.46 pattern touches per entry on top of the revision (10.97 to
10.51). The tree has one `**` rule, which never dies, and 96.9% of entries (89,543) see
no live anchored rule once rules are grouped by segment count.
Carrying state would also mean a live set per pending directory in two builders.
H173 is re-scoped as a stateless per-listing subset computed in `chain_for`, justified
only by a subject heavy with `**` or deep anchored rules.

#### 2.8 Conformance Checklist

Condensed from 29 items.
Pattern-level cases compare against `Gitignore::matches_components`, not the table,
because parent exclusion changes rows such as `foo/*` against `foo/bba/arr`.

| Area | What must hold | git tests |
| --- | --- | --- |
| Line syntax | `#` at column 0 only; `\#` and `\!` literal; unescaped trailing spaces trimmed, tabs kept; one `\r` stripped; a final line without a newline is a rule; a lone `!` matches nothing; **BOM skipped**; **NUL truncates** | t0008 whitespace and backslash cases |
| Anchoring | no `/` left means basename at any depth; any `/` anchors, including `\/` and `[a/b]`; `\/x`, `//x` and `a//b` match nothing | t0008 “exact prefix matching” |
| Directory-only | trailing `/` means directories, not symlinks to them; `foo\/` matches nothing; `!foo/` does not match a file | t0008 “ignored sub-directory” |
| Wildcards | in anchored rules `*`, `?` and `[...]` never match `/`; `**` is special only as a whole segment; `x/**` excludes `x`; `**\/` needs one directory; brackets, ranges, POSIX classes, escapes; byte-wise matching; bounded worst case | t3070 blocks: basic, slash-matching, character class, malformed, recursion |
| Precedence | last match in a file wins; a deeper file wins; an excluded directory cannot be re-included; `data/**`, `!data/**/`, `!data/**/*.txt` | t0008 “nested include of negated pattern”, “directories and `**` matches” |
| Case folding | only if ever implemented: ASCII only, and `[A]` matches neither case under `core.ignorecase` [git] | t3070 case-sensitivity columns |

Documented scope differences, not bugs: `info/exclude` and `core.excludesFile` are not
read; nested repositories are not boundaries; a `.gitignore` inside an ignored directory
is read, without changing any verdict; fdu’s budget refusal differs from git’s 100 MB
warning; fdu is always case-sensitive.

#### 2.9 Two Divergences From git

Both are filed as `fdu-ifci` and kept out of H171, because they change answers and
`IGNORE_RULES_VERSION`.

1. **A UTF-8 BOM is not skipped.** `Gitignore::parse` splits the raw bytes, so the first
   rule of a BOM-prefixed file never matches.
   git skips the BOM [git], and so do ripgrep and gitoxide [src].
2. **An embedded NUL does not truncate the line.** git’s `bar\0baz` rule ignores `bar`
   [git]; fdu keeps the NUL, so the rule is dead.
   git’s order is: strip `\r`, truncate at the first NUL, then trim spaces.

Everything else checked agreed with git: brackets, escapes, `**`, whitespace and `\r`.

### 3. fdu’s Linux Hot Path

`fdu PATH` and `fdu --view tree PATH` take one route.
Both resolve to a tree view (`query_report.rs:245-253`, `:516-523`) and plan
`RetainedState::FullIndex` (`execution.rs:372-469`). Under `--cache auto` the one-shot
reads and writes no snapshot (H108, H160). Citations are at `e5a71c8a`.

| Stage | What fdu does | Cost | Registry |
| --- | --- | --- | --- |
| Setup | The root is canonicalized three times (`cli.rs:806`, `lib.rs:652`, `scan.rs:4822`); a cache path is resolved that the plan never uses | microseconds | new |
| Threads | 4 walkers and the calling thread as the one consumer (`scan.rs:2575-2592`); `PORTABLE` values inherited from macOS (`platform_tuning.rs:149-158`); the adaptive unlock never fires warm | 5 runnable threads on 4 vCPUs | H84, H165 |
| Enumeration | `fs::read_dir(root.join(rel_dir))` (`scan.rs:3463`, `:3512`): glibc `opendir` on an absolute path, its `fstat`, a 32 KiB `DIR` buffer, at least two `getdents64`, `close`; std adds an `Arc` and a path copy | at least 5 system calls and 4 allocations per directory | H169 |
| Names | std’s `DirEntry` owns a `CString`; `file_name()` (`scan.rs:3535`) and `to_os_string()` (`:3660`) copy it twice | 3 name allocations per entry on the walker | H157, H177 |
| Metadata | `DirEntry::metadata()` (`scan.rs:1276-1278`): `statx(dirfd, name, AT_SYMLINK_NOFOLLOW \| AT_STATX_SYNC_AS_STAT)` [ext]; the index route stats directories too | 1 `statx` per entry; each directory described twice | H72, H169, H179 |
| Handoff | one `Mutex` and `Condvar` queue, claims of up to 4 directories (`scan.rs:4475-4503`), `notify_all` on every `extend` (`:4515`), one unbounded `mpsc` message per chunk (`:2752`) | about 4.5k `futex` on this tree | H166, H158 |
| Index build | `DetachedIndexBuilder::push_directory` (`index.rs:1625-1722`): a SipHash `HashMap<PathBuf>` remove, the control upsert, a name sort and dedup, `chain_for`, then the per-child steps below | 2.6k instructions and 7 allocations per entry without `.gitignore` | H167, H168, H172, H176 |
| Classification | `ControlChain::is_ignored` (`control.rs:765-778`) re-splits the directory for every child and scans every governing pattern | about 111 patterns × 140 instructions per entry: 1.68G of 2.07G | H171, H175 |
| After the walk | `finish()` reverses over the whole arena with two `BTreeMap` merges per directory (`index.rs:1725-1747`); `set_initial_freshness` makes a second full pass (`index.rs:2923-2941`) | 5–10 ms tail on this tree | folded into H172 |
| Report | `expand` over admitted directories; a `PathBuf` and `String` per child (`query_report.rs:3098`, `:3144-3147`), below-share rows cloned to be summed (`:3040`), two `String` clones per tie in the sort | small here; grows with wide directories | new |
| Teardown | an index of 64Ki entries or more drops on a detached thread (`lib.rs:314-326`); otherwise inline, and always inline under `FDU_COUNTERS=1` | outside the answer | H156 |

**Per child on the consumer** (`index.rs:1687-1720`):

| Step | Code | Cost |
| --- | --- | --- |
| Extension (files) | `intern_ext(&ext_bucket(&name))` (`index.rs:1690-1691`, `classify.rs:979-1045`) | a lowercased `String`, a `BTreeMap<String>` lookup, a free |
| Classification | `chain.is_ignored(path, name, is_dir)` (`index.rs:1693`) | the dominant cost with `.gitignore` present |
| Directory path | `path.join(&name)` (`index.rs:1696`) | an allocation per directory |
| Slot | `Entry::new_detached` → `Index::alloc` (`index.rs:4988-5006`) | a slot of about 136 bytes, arena grown by reallocation; directories add a `Box<DirectoryEntry>` of about 176 bytes |
| Contribution | `contribution(child_id)` (`index.rs:5239-5270`) | two one-entry `BTreeMap` nodes of about 328 bytes for a file |
| Merge | `rollup_mut().merge(&direct)` (`index.rs:1715`) | two partition merges into the parent’s maps, then two frees |
| Directory map | `directory_ids.insert(child_path, child_id)` (`index.rs:1717`) | SipHash, occasional growth |

Only the merge into the parent depends on anything outside the listing.

**Inefficiencies this pass found that the registry did not name:**

1. A basename rule runs its glob before the cheap directory-only rejection
   (`control/gitignore.rs:217-219`). Subsumed by H171.
2. `chain_for` probes a `BTreeMap<PathBuf>` once per ancestor with component-wise
   ordering for every listing (`control.rs:527-546`), although the parent’s chain is
   already known. Now H175.
3. Two full arena passes after the walk, and the freshness pass sits outside
   `detached_finish_us`, so no counter sees it.
   Folded into H172’s tier.
4. A walker count below the core count was never measured for the consumer-bound
   default. Now the Q3 screen, and H178 as the self-adjusting form.
5. `statx` without `AT_NO_AUTOMOUNT` (section 4).
6. With counters on, each bump copies a 424-byte `Cell<Counts>` in and out, including on
   every allocation (`counters.rs:859-887`), so counter-run walls are not comparable
   with counters-off runs.
7. Micro items, none expected to clear 3% alone: the summary route’s per-entry path
   clone (`scan.rs:3797`; H51’s mechanism, refuted on macOS, so a Linux attempt must say
   why cross-thread frees differ); its copy of each `.gitignore`’s bytes
   (`execution.rs:724`); an unsized `read_to_end` after an `fstat` that gave the length
   (`scan.rs:4121-4123`); the report rows above; ancestor walks over empty refusal maps
   (`control.rs:640-646`, `index.rs:3886-3923`); and three canonicalizations of the
   root.

### 4. `statx` Triggers Automounts That `fstatat` Does Not

Bead `fdu-puk7`. Checked in the kernel at v6.12 (`adc21867`):

- `getname_statx_lookup_flags` sets `LOOKUP_AUTOMOUNT` unless the caller passes
  `AT_NO_AUTOMOUNT` (`fs/stat.c:240`).
- `vfs_fstatat`, behind `fstatat`, `stat` and `lstat`, always adds `AT_NO_AUTOMOUNT`
  (`fs/stat.c:328`).

So `lstat` and `fstatat` never trigger an automount on a terminal component, and a bare
`statx` does. Rust std’s `DirEntry::metadata()` and `symlink_metadata` call `statx` with
`AT_SYMLINK_NOFOLLOW | AT_STATX_SYNC_AS_STAT` and no `AT_NO_AUTOMOUNT` [ext]. dut
(`main.c:604`) and bfs (`stat.c:62-74`) pass the flag; GNU du and ncdu use `fstatat`.

The consequence: a walk that meets an autofs trigger directory (`/net`, `/misc`, a
systemd automount unit) mounts it just by statting it, possibly a slow or hanging
network mount, even where the walk would not descend into it, as under
`--one-filesystem`. Walkers that use `fstatat` (GNU du, ncdu) or pass the flag (dut,
bfs) mount only what they open to list.
This is inferred from the two code paths; no automount was exercised here.

The fix rides with H169 phase 1 (Q6): the Linux reader calls
`statx(dirfd, d_name, AT_SYMLINK_NOFOLLOW | AT_NO_AUTOMOUNT, STATX_BASIC_STATS)`. It
changes no answer on a host without automounts, only whether a mount fires, and is
recorded in H169’s row as a correctness side effect.
The portable fallback and the root’s `symlink_metadata` still go through std.

### 5. Two Regimes of Wall Time

The plan’s first model, fixed cost plus the larger of the parallel walk and the serial
consumer, omits CPU saturation.
The Fable review’s correction rests on exp-174 (quiet, the 0.2.1 engine).
Its medians are over all 15 samples per arm, so they differ slightly from the ledger’s
12-pair medians (211.3 ms and 80.6 ms for the tree):

| Arm, `linux-v6.12` | Wall | User CPU | Kernel CPU | CPU total | CPU per wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| `default-tree`, controls on | 215.3 ms | 237.8 ms | 186.7 ms | 424.5 ms | 1.97 |
| `default-tree`, `--no-controls` | 81.2 ms | 73.6 ms | 194.0 ms | 267.6 ms | 3.30 |
| `aggregate-summary`, controls on | 164.9 ms | 224.9 ms | 177.4 ms | 402.3 ms | 2.44 |
| `aggregate-summary`, `--no-controls` | 71.2 ms | 68.4 ms | 175.0 ms | 243.4 ms | 3.42 |

The controls-on tree leaves two of four vCPUs idle: the consumer runs alone for the
second half of the run.
The blind tree keeps 3.3 cores busy, and its consumer, about 25 ms of CPU, costs about
10 ms of wall because it runs in the idle fraction of a core.
So:

```
wall ≈ max(T_consumer_serial, (CPU_walkers + CPU_consumer + CPU_kernel) / P_eff) + tail
P_eff ≈ 3.3–3.4 on this guest; tail ≈ 5–10 ms (finish, freshness, report, exit)
```

The review’s predictions for the queue, on this host’s numbers (not results):

| Step | Total CPU | Predicted wall, default tree | Regime |
| --- | ---: | ---: | --- |
| Today | 425 ms | 215 ms (measured) | consumer-bound |
| H171 prototype | about 340 ms | 100–115 ms | still consumer-bound |
| H171 revised, with H175 | about 300 ms | 87–95 ms | crosses to CPU-bound |
| plus H172 with H176 | 250–265 ms | 76–82 ms | CPU-bound |
| plus H169 phase 1 | 215–225 ms | 66–70 ms | CPU-bound |
| plus H177 | 205–215 ms | 62–66 ms | about pdu’s default |

- Beating pdu needs total CPU at or under about 215 ms.
  After the consumer stops being critical, only removing CPU moves wall; moving it does
  not.
- **H174** (a walker-side listing digest) moves consumer work to walkers, so it is
  conditional: it runs only if the consumer is still busy for more than 80% of the walk,
  and its accept rule adds total instructions non-increasing.
- **A `cores − 1` walker count** wins only before H171 lands; after it, three walkers
  need about 268/3 = 89 ms for the walk and lose.
- **Parity is a walker-CPU problem.** The walkers spend about 65 ms of user CPU where
  pdu spends 53 and dut 12 (screen); H169’s in-place names and H177’s name arena target
  it.
- **Instructions do not convert to time at one rate.** 2,067M consumer instructions in
  about 205 ms is an IPC of 4.8 at 2.1 GHz, plausible only for a tight byte loop.
  Predictions are stated in CPU milliseconds; instructions are the secondary.

## Recommendations

These are adopted in
[the overnight plan’s amendments](../specs/active/plan-2026-09-29-linux-overnight-performance-loop.md#amendments-after-the-plan-review).

1. **Q2: H171 revised, with H175.** Changes (a)–(h) above, with two predictions: at most
   1.15 times the `--no-controls` arm for the revision, 1.45 for the prototype.
2. **Demote H173** to a stateless per-listing subset, run only on a `**`-heavy subject.
3. **Q4: H172 with H176 and the fused post-walk passes.**
4. **Q6: H169 phase 1**, with `AT_NO_AUTOMOUNT`, closing `fdu-puk7` on Linux.
   Parent-relative opens wait for an fd budget; fd-derived directory attributes are
   H179.
5. **The next night:** H177 after H169, and H178 as the self-adjusting walker count.
6. **Fix `fdu-ifci` separately,** with recorded git verdicts: it changes answers.
7. **The matcher benchmark half** uses published crates past the cool-off, on the clean
   tree, the tree with virtual build outputs, a many-rules synthetic, and adversarial 16
   KiB lines; every engine must agree with `git check-ignore --no-index` on every path.
8. **No `PORTABLE` change from one 4-vCPU guest.** A cap of three also changes 8-core
   hosts, and `cores − 1` makes 2-core hosts serial.
9. **Do not adopt** io_uring, bigger buffers, terminating-call elision, top-N retention,
   hard-link shortcuts, or an allocator in `fdu-core`.

## References

- [Overnight plan](../specs/active/plan-2026-09-29-linux-overnight-performance-loop.md)
  and [0.2.2 plan](../specs/active/plan-2026-09-29-linux-parity-0.2.2.md)
- [Design study](research-2026-09-29-linux-default-tree-point-solution.md) and
  [pdu brief](research-2026-09-28-pdu-and-the-linux-peer-gap.md)
- [Metadata-walk floor report](../reports/report-2026-08-23-metadata-walk-floor.md)
- [Hypothesis registry](../guides/performance-loop.md#hypotheses): H51, H58, H71, H72,
  H74, H76, H84, H156–H179
- [Experiment ledger](../reports/report-2026-08-10-fdu-performance-experiments.md):
  exp-173, exp-174, exp-175
- Peers at the commits in section 1: dut, bfs, pdu 0.24.0 at `c30e46f`, diskus, dust,
  dua-cli, jwalk, gdu, ripgrep `ignore`, fd, erdtree, fastwalk, GNU coreutils, ncdu,
  uutils, walkdir, duc, dumac
- Matchers at the commits in section 2: git (`dir.c`, `wildmatch.c`,
  `t/t0008-ignores.sh`, `t/t3070-wildmatch.sh`), ripgrep `globset` and `ignore`,
  gitoxide `gix-glob` and `gix-ignore`, libgit2 `attr_file.c` and `ignore.c`, jj
  `lib/src/gitignore.rs`
- Linux v6.12 (`adc21867`), `fs/stat.c`

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
