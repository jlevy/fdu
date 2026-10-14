# Correctness Runbook: Warm Against Cold on a Real Tree

A standing manual procedure for the claim that caching changes performance and never
semantics. It builds a tree holding every file kind the platform supports, asks the same
requests cold, warm, and cache-only, and compares the answers — while also proving the
cache actually served.

This complements the
[path-independence harness](../../../tests/path_independence/README.md) rather than
duplicating it. The harness replays a synthetic fixture on every commit; this runs
rarely, by hand, over kinds nobody enumerated when the fixture was written.

Two principles govern it.
[Caching Improves Performance, Never Semantics](../architecture/fdu-design-principles.md#caching-improves-performance-never-semantics)
is the property under test.
[Model Every Key Concept Explicitly, in One Place](../architecture/fdu-design-principles.md#model-every-key-concept-explicitly-in-one-place)
is why the oracle is always a cold answer to the request that was asked, never the
warmer’s.

## The Check That Makes the Rest Worth Running

A warm-against-cold comparison cannot fail usefully on its own.
**A cache that never serves scans cold both times and matches.** This is not
hypothetical: with snapshot serving hard-wired to refuse, the path-independence subset
ran 884 cases with zero failures, because a miss simply scans cold and compares equal.

So every case records the mechanism as well as the answer, read from the report’s
`provenance.source` and `provenance.freshness`, or for content from
`provenance.tiers.content.source`, and the expectation differs by request:

| Request | Second run reports | Why |
| --- | --- | --- |
| Metadata only | `cold_scan` | A metadata walk is cheap, so it re-walks by design. Its proof that the snapshot a `--cache on` run left serves is the `--stale-ok` run, which must exit 0, report `cache_only`, and label the answer `stale`. |
| Analysis, named by `--analyze …` or implied by a content view | content tier `revalidated` | The content sidecar is the expensive tier and is the one that must serve. The report-level `warm_revalidate` only says the entries came from a snapshot. |
| `--stale-ok` | `cache_only` | Serves without verifying, and labels the answer stale. |

Which row a request falls in is read from its cold answer’s `request.analyze`, the
analyzer set the engine enabled, and never from its arguments.
`--view code` names no `--analyze` and still enables code analysis, so a script that
looked for `--analyze` in its own argv would hold that request to the metadata row and
never check that its sidecar serves.

A run whose answers all match but whose mechanism column is wrong has proved nothing.
The first time this procedure ran, every invocation was failing on an unknown flag and
the answer comparison still reported zero mismatches; the mechanism column is what
caught it.

**Every clause of that check has to be reachable**, which is not automatic.
An earlier version guarded the cache-only check with `only_rc == 0`, and the engine
either serves `cache_only` or exits 1 — so against a build that never wrote a snapshot,
cache-only exited 1, the check was skipped, and seventeen of the twenty-three cases
printed `ok` against a cache that never served.
Verify each pass by breaking the thing it watches.
Over the refusal-free tree, a wrapper that rewrites `--cache on` to `--cache off` must
make both scripts exit 1. In `warm_cold.py` every case fails: each metadata case reports
`NO-SNAPSHOT`, and each analysis case reports `NOT-WARM(scanned)`, because its `auto`
run scans and then writes a snapshot, which the `--stale-ok` run finds.
`cross_warm.py` holds only pairs whose warmer and ask enable the same analyzer set to
serving, and each of those reports `NOT-WARM(scanned)`. Over the refusal tree, a wrapper
that answers `--stale-ok` with the cold output relabeled `cache_only` must make every
case report `PARTIAL-STORED` and `--refusals-only` exit 1. A partial answer exits 2, so
a check that trusted a zero exit would have called that stored snapshot `withheld`; the
first version of this pass did.
The two wrappers are `tests/correctness/break_no_snapshot.py` and
`tests/correctness/break_partial_stored.py`: name one as `FDU_BIN`, and the real binary
as `FDU_REAL`. A script exits 1 when any one case fails, so read the table as well as
the status: every case must be caught.
`cross_warm.py` prints the number of `pairs held to serving`, which is how many
`NOT-WARM` rows the first break must produce; `make release-stability` checks all three
counts.

## Running It

Before a release, `make release-stability` runs all three passes and both breaks against
the installed candidate, the refusal pass as `nobody` when it runs as root; see the
release process’s [Stability Pass](release-process.md#stability-pass).
By hand, against a build of the checkout:

```shell
make build
python3 tests/correctness/build_tree.py /tmp/fdu-correctness/tree
FDU_BIN=target/debug/fdu python3 tests/correctness/warm_cold.py --refusals-only /tmp/fdu-correctness/tree
python3 tests/correctness/build_tree.py --without-refusals /tmp/fdu-correctness/served
FDU_BIN=target/debug/fdu python3 tests/correctness/warm_cold.py /tmp/fdu-correctness/served
FDU_BIN=target/debug/fdu python3 tests/correctness/cross_warm.py /tmp/fdu-correctness/served
```

**The serving proof needs a tree without refusals.** An unreadable file or unlistable
directory makes every answer partial for an unprivileged user, and a partial scan never
writes the entry tier: a snapshot missing an entry would be served as the tree’s totals
on the next run. So over the full tree the check is the refusal path: every answer is
partial, warm matches cold, and cache-only finds nothing stored (`withheld`).
`--refusals-only` asserts exactly that: a case is withheld only when cache-only exits 1
with nothing on stdout, any answer it does print is compared with cold, and a case that
comes back complete fails, which is what a run whose refusals were not effective looks
like. The same kinds without their refusal bits are complete, so the second run must
serve every case and fails any case that comes back partial.
Every report’s `age_ns` is checked against its own `age_reference_ns` and `mtime_ns`, on
list rows and on tree nodes at every depth, because ages move with the reference instant
and are not comparable across runs, and the reference instant must fall within the
invocation that produced the report.
A tree node that counts no entry, or whose subtree was not listed in full, must have a
null age.
`age_reference_at`, the same instant rendered, is set aside with it; each row’s
`modified_at` is compared like any other field.

Keep the socket path short.
A Unix socket path is limited to about 104 bytes on macOS, so a tree under a long
scratch path reports `socket` absent.

`build_tree.py` prints which kinds it managed to create.
A kind the platform refuses is reported absent rather than skipped silently, because a
run that quietly built fewer kinds looks exactly like a run that passed.
It removes and rebuilds the target directory, chmod-ing its way past the 0o000 entry
first: re-running into an existing tree made every `link`, `symlink`, `mkfifo` and
`mknod` raise `FileExistsError`, which is an `OSError`, so a second run reported six
kinds as “platform refused” and printed a truthful-looking eleven of seventeen.

**No single run can honestly report every kind, and the script says which one you are
in.** Device nodes need `CAP_MKNOD`, so they need root; mode bits are ignored by root,
so as root the unreadable file and unlistable directory exercise nothing and
`permission-denied-effective` reports absent.
Build the tree as root for the device nodes, then run the comparison as an unprivileged
user for the refusal paths.

## When It Runs

Before tagging a release, and after any change to cache identity, serving, or
reconciliation. It is not on a timer and not in `make check`: it needs a built binary, a
constructed tree, privilege on both sides of the root boundary, and minutes rather than
seconds. Record each run per *Recording a Result* below, so that “when did anyone last
actually check this” has an answer.

## What the Tree Holds

Regular files across sizes including empty; a sparse file, where apparent size and
allocated blocks must disagree; hard links, both within one directory and split across
two, so attribution can be wrong in two distinguishable ways; symlinks that are
relative, absolute, to a directory, dangling, self-cyclic and mutually cyclic; FIFOs,
Unix sockets, and character and block device nodes; setuid, setgid and sticky bits; an
unreadable file and an unlistable directory; names carrying spaces, quotes, brackets,
braces, backslashes, colons, commas, tabs, newlines, leading dashes, flag-like prefixes,
non-ASCII and emoji, and a name that is not valid UTF-8 at all; case-colliding names,
which stay distinct here and collapse on a case-insensitive filesystem; a forty-level
chain, a two-hundred-character component, a five-hundred-entry directory, and an empty
one; and nested `.gitignore` files with both an ignored file and an ignored directory.

Two notes on what a run can and cannot establish:

- **Permission denial needs a non-root user.** Root ignores the mode bits, so the
  unreadable file and unlistable directory only exercise the refusal path when the run
  drops privileges.
- **Hard links need a decided policy first.** `fdu-579b` is still open, so “the same
  before and after” is not yet the right oracle for them: both answers could be
  consistently wrong. Pin the attribution rule, then test against it.

## The Cross-Warm Matrix

`cross_warm.py` asks one request after warming with a different one.
This is the shape that produced `fdu-gija`: `--analyze lines` after `--analyze all` once
reported metrics nobody requested and moved `document_words`, and `--analyze lines`
after `--analyze code` read an `Unsupported` record whose line counts had been discarded
at store time. Both passed their own tests, because those tests asserted cache hits
rather than answers.

It compares the full report body and `analysis.analyze` against a cold answer to the
request that was asked, and when the warmer stored the same analyzer set it requires the
ask’s content tier to report `revalidated`: a wider stored set is not yet reused (the
containment deferral in `fdu-7dj6`), so only matching pairs are held to serving.
The field’s documented meaning is what the report *requested*, so serving it from a
wider stored set is itself the defect — a warm `--analyze lines` after `--analyze all`
must report `["lines"]`, never the stored set.

A request’s analyzer set has two sources: what `--analyze` names, and what its content
views imply (`code` implies code, `documents` implies words).
So both sides of the matrix reach analysis both ways, and the set that decides whether a
pair must serve is each side’s own `request.analyze`: the warmer’s from its own report,
the ask’s from its cold answer.
`--analyze lines --view documents` enables lines and words, so it must be served from
the sidecar `--analyze words` or `--view documents` stored, and not from the one
`--analyze lines` stored.
An earlier copy of the script read `--analyze` from argv, and held exactly that pair the
wrong way round: it failed the lines warmer, which correctly read every file again, and
never checked the words warmer.

Check that the oracle discriminates before trusting a green run: the expected
`analysis.analyze` differs per request (`["lines"]`, `["lines","code"]`,
`["lines","code","words"]`), so a build that served the warmer’s set would show.

## Recording a Result

Record the regime, not just the verdict: platform, host (bare metal or virtualized),
filesystem, and cache state decide what a result is evidence about.
A run on one platform is inherited, not proven, on the others — `cfg(target_os = …)`
code in `scan.rs`, `lib.rs`, `platform_tuning.rs` and `counters/process.rs` means a
Linux run validates none of the macOS walk, and `make cross-lint` only proves it
compiles.

The macOS half — `~/Library` under TCC, APFS case-insensitivity and cloning, and iCloud
dataless files that a read can materialize — is tracked separately as `fdu-q098`.

### Last Recorded Run

The 0.5.0 release pass on 2026-10-14 tested release commit
`4a513b00f7d7a508344725a5ebd48e84299f4727` (tree
`bd82aedb40b7e50c1e128c3be791335408abda32`), the commit tagged `v0.5.0` and published.
The candidate was the wheel the pass built from that commit,
`fdu-0.5.0-cp312-abi3-manylinux_2_34_x86_64.whl`, which reported
`fdu 0.5.0-dev+g4a513b00f`. Every pass and both deliberate breaks passed.

**Regime.**

- **Host.** A 4-vCPU Linux x86_64 virtual machine (Firecracker, kernel 6.18.44), not
  bare metal.
- **Filesystem.** ext4, with the trees under a short scratch directory every user can
  reach.
- **Privilege.** The refusal pass ran as `nobody`; the others ran as root, with
  `FDU_TEST_ALLOW_NO_PERMISSION_BITS=1` and `FDU_TEST_ALLOW_NO_NATIVE_WATCH=1` declared.
- **Cache state.** A fresh cache directory for every case.

**Kinds.** The refusal tree held 16 kinds and lacked one: `permission-denied-effective`,
which root cannot experience.
The served tree held all 16 kinds.
There were 26 cases and a cross-warm matrix of 8 warmers by 9 asks.

| Pass | Result |
| --- | --- |
| `--refusals-only`, refusal tree, as `nobody` | 26 of 26 cases partial and withheld; 0 answer mismatches, 0 mechanism failures, 0 stale reference instants; exit status 0 |
| `warm_cold.py`, complete tree | 26 of 26 served `cache_only` and labelled `stale`; 0 answer mismatches, 0 mechanism failures, 0 stale reference instants; exit status 0 |
| `cross_warm.py`, complete tree | 72 of 72 pairs matched the cold answer; 0 violations; exit status 0 |

The deliberate breaks confirmed the checks reject incorrect serving:

- **No snapshot stored.** A wrapper turned `--cache on` into `--cache off`.
  `warm_cold.py` exited 1 and caught all 26 cases: 17 `NO-SNAPSHOT` and 9 `NOT-WARM`.
  `cross_warm.py` exited 1 and caught all 17 same-analyzer pairs as `NOT-WARM`.
- **Partial answer stored.** A wrapper relabeled cold output as `cache_only` for
  `--stale-ok`. All 26 cases reported `PARTIAL-STORED`, and `--refusals-only` exited 1.

Every case is retained in the
[0.5.0 release stability-pass report](../reports/report-2026-10-14-release-0.5.0-stability-pass.md).
`fdu-579b` has not decided hard-link attribution; these checks establish warm/cold
agreement, not that counting policy’s correctness.

### Previous Run: 0.5.0 Candidate on macOS

On 2026-10-12, against the 0.5.0 preparation tree `67e3ca0f` (commit `d08624879`), all
three passes and both breaks passed on a bare-metal M1 Pro, macOS 26.5.2, APFS, as an
unprivileged user, with 26 cases and 72 cross-warm pairs.
The refusal tree held 14 kinds and the served tree 13: `chardev` and `blockdev` need
root, and APFS refuses a name that is not valid UTF-8. The full record is in the
[0.5.0 candidate stability-pass report](../reports/report-2026-10-12-release-0.5.0-stability-pass.md).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
