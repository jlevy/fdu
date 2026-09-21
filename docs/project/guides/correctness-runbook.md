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

So every case records the mechanism as well as the answer, and the expectation differs
by request:

| Request | Second run reports | Why |
| --- | --- | --- |
| Metadata only | `cold_scan` | A metadata walk is cheap, so it re-walks by design. Its proof that the snapshot serves is the cache-only run, which must exit 0, report `cache_only`, and label the answer `stale`. |
| `--analyze …` | `warm_revalidate` | The content sidecar is the expensive tier and is the one that must serve. |
| `--cache only` | `cache_only` | Serves without verifying, and labels the answer stale. |

A run whose answers all match but whose mechanism column is wrong has proved nothing.
The first time this procedure ran, every invocation was failing on an unknown flag and
the answer comparison still reported zero mismatches; the mechanism column is what
caught it.

**Every clause of that check has to be reachable**, which is not automatic.
An earlier version guarded the cache-only check with `only_rc == 0`, and the engine
either serves `cache_only` or exits 1 — so against a build that never wrote a snapshot,
cache-only exited 1, the check was skipped, and seventeen of the twenty-three cases
printed `ok` against a cache that never served.
Verify the check by breaking the thing it watches: run the comparison against a wrapper
that rewrites `--cache auto` to `--cache off` and confirm every case reports
`NO-SNAPSHOT` and the script exits 1.

## Running It

```shell
make build
python3 tests/correctness/build_tree.py /tmp/fdu-correctness/tree
FDU_BIN=target/debug/fdu python3 tests/correctness/warm_cold.py /tmp/fdu-correctness/tree
FDU_BIN=target/debug/fdu python3 tests/correctness/cross_warm.py /tmp/fdu-correctness/tree
```

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
request that was asked.
The field’s documented meaning is what the report *requested*, so serving it from a
wider stored set is itself the defect — a warm `--analyze lines` after `--analyze all`
must report `["lines"]`, never the stored set.

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

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
