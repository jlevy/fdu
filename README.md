# fdu

**Fast, incremental file roll-up engine** — `fd` and `du`, read as “fast du”.

fdu answers, for *every* directory in a tree at once: how big is it, how many files does
it hold, what changed most recently, and what kinds of files live in it.
One walk, many metrics, cached between runs.

> **Status: pre-release.** The observation/commit contract, bounded in-process change
> feed, cache lifecycle, applying reconciler, CLI, and Python wheel are tested end to
> end, and the measured-improvement loop described below is running.
> The syscall layer is not built yet: the walker is a bounded parallel pool over
> portable `read_dir` and `symlink_metadata`, so **no release-grade performance claim
> should be made for this crate until that layer lands and the benchmark gate against
> other tools passes**. See
> [the Phase 1 plan](docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md).

## Why

Of a dozen surveyed tools in this space ([du](https://www.gnu.org/software/coreutils/),
[ncdu](https://dev.yorhel.nl/ncdu), [dust](https://github.com/bootandy/dust),
[dua](https://github.com/Byron/dua-cli), [gdu](https://github.com/dundee/gdu),
[dut](https://codeberg.org/201984/dut), [duc](https://github.com/zevv/duc),
[fsearch](https://github.com/cboxdoerfer/fsearch),
[bfs](https://github.com/tavianator/bfs), [fd](https://github.com/sharkdp/fd),
[scc](https://github.com/boyter/scc), [tokei](https://github.com/XAMPPRocky/tokei)),
exactly one persists anything, exactly one carries multiple metrics per pass, **none**
does per-directory type tallies, and **none** does mtime-based incremental revalidation.
The combination is unoccupied ground, and it is what a live file browser actually needs.

The full survey, with the techniques worth adapting and their sources, is in
[the file roll-up engine research](docs/project/research/research-2026-08-06-file-rollup-engine.md).

## Speed and the Cache

fdu can build an answer from a full scan, a revalidated snapshot, or an explicitly stale
snapshot-only read, and it labels which path produced the result. A full stat-tier
answer still has to enumerate and stat every entry unless a trustworthy change source
narrows the work: an in-place file edit changes no directory’s mtime, not even its
parent’s. A directory fingerprint can prove membership, not child bytes.

The evidence is narrower than the design space:

| Case | What is known now |
| --- | --- |
| Warm APFS, roughly 60k entries, current stat-tier snapshot | Snapshot load plus full revalidation is slower than a cold scan in the measured experiments. |
| Other filesystems, Windows, remote storage, cold metadata caches, and larger scales | Not yet measured; no benefit or loss is claimed. |
| Snapshot-only read | Avoids tree I/O but is explicitly stale, so it serves a different contract rather than winning an exact comparison. |
| macOS journal-scoped revalidation | Designed, not built; Apple documents persistent history, but the cross-restart correctness and performance spike is still required. |
| Cached content analyzers such as line counts | Designed, not built; fingerprints could avoid rereading unchanged content, but no timing claim exists yet. |

The cache is therefore not necessary for a correct one-shot stat roll-up, and it should
not be read automatically where evidence says it loses. It remains useful infrastructure
for cache-only answers, future applied journal boundaries, and future derived metrics.
The proposed `auto` policy selects a cached read only for a measured
platform/filesystem/storage/scale cell; unknown cells use the cold path. Explicit
refresh, read-only, cache-only, and off policies keep the decision under caller control.
See the
[composable CLI plan](docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md)
and the
[FSEvents-scoped revalidation plan](docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md).

Speed changes here are decided by paired, interleaved measurement against a real tree,
with an independent oracle verifying that faster output is still identical output.
Every accepted and rejected experiment is recorded in
[the experiment ledger](docs/project/reports/report-2026-08-10-fdu-performance-experiments.md);
the protocol is [the performance loop](docs/project/guides/performance-loop.md).
The cost model, the platform levers that change constants by integer factors, and the
ranked backlog are in
[the performance frontier research](docs/project/research/research-2026-08-10-performance-frontier.md),
which draws on source review of bfs, dut,
[pdu](https://github.com/KSXGitHub/parallel-disk-usage),
[diskus](https://github.com/sharkdp/diskus), and
[jwalk](https://github.com/jessegrosjean/jwalk), plus
[dumac](https://healeycodes.com/maybe-the-fastest-disk-usage-program-on-macos)’s
measurements of the macOS bulk-attribute path.

## Build Locally

```shell
cargo install --path crates/fdu
fdu --help
```

```shell
make python-smoke  # build the wheel; test its module, console script, and local uvx path
```

Publishing is Phase 1 work.
`cargo install fdu` and `uvx --from fdu==<version> fdu` are future commands; neither
package should be presented as available from crates.io or PyPI yet.

## Use It

```shell
fdu                        # summarize the current directory
fdu -d 3 ~/src             # three levels deep
fdu --by-type ~/Downloads  # break down by file extension
fdu --json .               # stable, versioned JSON for agents and scripts
fdu --skill                # print the self-contained agent skill
```

```text
/workspace/fdu/crates  12 files, 4 dirs, 156 KiB
   140 KiB  █████████░    90%  fdu/
   136 KiB  █████████░    87%    src/
   4.0 KiB  ░░░░░░░░░░     3%    Cargo.toml
```

`--help` is the complete source of truth.
Human output uses restrained semantic color when its destination is a terminal;
`--color auto|always|never`, `NO_COLOR`, and `FORCE_COLOR` make the policy explicit.
Primary results go to stdout, while warnings and errors go to stderr.
JSON and skill output never contain ANSI styling.
JSON output carries a `schema` field (`fdu.tree/2`) that is versioned with the tool,
plus freshness and per-path error details.
Scan completeness, scan scope, and rendered-tree truncation are separate fields.
Invalid-Unicode paths retain their display string and add a lossless, platform-tagged
raw identity.
Exit status 2 means partial results; pass `--allow-partial` to accept those
as success. Exit status 1 means the command failed.

The next iteration of this surface — composable views, selection filters,
modified-file candidate queries, cache policies, and a `tail -f`-style watch mode, all as
orthogonal flags over one grammar — is designed in
[the composable CLI and query surface plan](docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md).

## As a Rust Library

```toml
[dependencies]
fdu = { path = "crates/fdu", default-features = false }
```

`default-features = false` skips the CLI’s dependency tree.
Add `features = ["watch"]` for the OS-native watch layer.

```rust
use fdu::{OpenConfig, open};
use std::path::Path;

let (index, report) = open(Path::new("."), &OpenConfig::default())?;
let total = index.total();
println!("{} files, {} bytes", total.files, total.bytes);

// Per-directory roll-ups are pre-computed: this is a field read, not a traversal.
if let Some(src) = index.rollup(Path::new("src")) {
    println!("src/: {} files, newest {}", src.files, src.newest_mtime_ns);
}
# Ok::<(), fdu::Error>(())
```

## As a Python Module

```python
import fdu_py

index = fdu_py.open("/path/to/tree")
print(index.complete, index.freshness, index.errors)
print(index.total())          # {'files': ..., 'bytes': ..., 'by_extension': {...}}
print(index.children("src"))  # one call returns every child with its roll-up

mark = index.clock
index.refresh()               # reconcile against the filesystem
print(index.since(mark))      # exactly what changed, or truncated=True if you fell behind
```

Every method is bulk: it returns a whole structured result in one call.
A million small zero-copy calls lose comfortably to one large call.
The same wheel also installs an `fdu` console script backed by the native Rust CLI. Once
a release is published, that makes an exact version directly runnable as
`uvx --from fdu==<version> fdu`; the local wheel and `uvx` path are already exercised by
`make python-smoke` without implying that a public release exists.

## How It Works

Three artifacts and one contract:

| Artifact | What it is |
| --- | --- |
| **Index** | In-memory parent-pointer tree; every directory carries pre-computed roll-ups |
| **Snapshot** | A complete index baseline, keyed by canonical root, semantic scan scope, format, and engine version |
| **Observation** | Verified producer input, optionally conditional on the indexed path state |
| **AppliedDelta** | A clocked batch of effective committed changes for the bounded change feed |

Everything else produces observations or consumes applied deltas.
A cold scan establishes a historyless baseline; a reconciliation sweep conditionally
applies its diff while it walks; the watch layer coalesces event hints and verifies them
by stat.
The index alone arbitrates observations, removes no-ops, advances the clock, and
mints `AppliedDelta`.

Today, `open()` is deliberately blocking: it loads a usable snapshot and completes a
filesystem reconciliation before returning.
It never serves the snapshot as fresh before that pass, and it never replaces a complete
snapshot with a partial result.
`IndexHandle` and the reconciliation APIs support readers between applied batches with
explicit `Fresh`, `Reconciling`, `Stale`, and `Partial` state, but applications must opt
into that serving model.
The optional watcher is an adapter and driver; `open()` and the Python API do not start
it automatically, and the Python wheel does not compile the watch dependency.
Its applying driver re-verifies queued samples at a clock-stable commit boundary and
currently accepts only an unbounded, cross-filesystem scope; bounded-depth and
one-filesystem event filtering is tracked as watch-hardening work and those
configurations fail explicitly rather than indexing excluded paths.
A watch sample is valid at its filesystem `stat` point; the process does not pretend it
can freeze external filesystem mutation until the in-memory commit.
Backend events that arrive during or after verification remain queued for the next
batch, while reported loss or ambiguity invalidates and reconciles the affected scope.
The logical-clock check prevents an older sample from overwriting a newer in-memory
commit; it is not a filesystem transaction.

Two invariants are non-negotiable, because a cache that lies is worse than no cache:

- Content-reuse fingerprints are **size, mtime, ctime, and inode**, not mtime alone.
  mtime is user-settable and some applications roll it back after writing; ctime is
  kernel-controlled. All observed stat fields are still compared when updating stored
  state, so allocated-byte or device changes cannot leave query results stale.
- A corrupt or unrecognized snapshot is treated as **absent, never as data**. Failing
  closed costs a rescan; failing open silently corrupts every answer built on it.
  The bootstrap format verifies its payload checksum before parsing records, and Unix
  cache files are created owner-only because they contain a filesystem inventory.
- Conditional observations carry generation and revision guards.
  Present-state ABA, parent replacement, and absent create/remove races are rejected at
  one batch boundary without making changes in unrelated subtrees conflict.
- Cold scans and every warm mutation path enforce the same semantic scope.
  Depth zero is root-only, and subtree reconciliation refuses paths below
  depth/filesystem boundaries or through symlink ancestors.

## Development

```shell
npm ci             # install the exact development-only golden-test toolchain
make supply-chain  # verify release age, provenance, exact pins, and CI trust controls
make build         # debug build, all features
make test          # Rust tests plus the end-to-end CLI golden contract
make test-golden   # build and compare only the CLI sessions
make check         # tests, audits, docs, and installed-wheel smoke — the handoff gate
make fix           # apply formatting
```

The golden sessions are executable Markdown under `tests/golden/`, run by
[tryscript](https://github.com/jlevy/tryscript).
After an intentional CLI output change, run `make golden-update`; it regenerates
affected blocks and immediately reruns comparison.
Review the Markdown diff before committing.
The scenario design and the small set of permitted dynamic patterns are documented in
[the completed CLI golden-test plan](docs/project/specs/done/plan-2026-08-09-fdu-cli-golden-tests.md).

Performance work has its own targets (`make perf-baseline`, `perf-profile`,
`perf-compare`, `perf-ledger`), deliberately outside `make check` — a timing gate on a
shared CI runner measures the runner.
Follow [the performance loop](docs/project/guides/performance-loop.md) before changing
anything for speed.

Read [the supply-chain policy](SUPPLY-CHAIN-SECURITY.md) before changing a dependency,
toolchain, CI action, or bootstrap download.
[AGENTS.md](AGENTS.md) carries the conventions worth not rediscovering.

## License

MIT. See [LICENSE](LICENSE).

Designs adapted from GPL-licensed tools ([dut](https://codeberg.org/201984/dut)’s
atomic-refcount roll-up, [fsearch](https://github.com/cboxdoerfer/fsearch)’s record
layout) are clean reimplementations written from the descriptions in the research doc,
not transliterated from their source.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
