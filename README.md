# fdu

**Fast, incremental file roll-up engine** — `fd` + `du`, read as “fast du”.

fdu answers, for *every* directory in a tree at once: how big is it, how many files does
it hold, what changed most recently, and what kinds of files live in it.
One walk, many metrics, cached between runs.

> **Status: early scaffold.** The architecture, the delta contract, and the cache
> lifecycle are in place and tested end to end.
> The fast walker is not — the current one is a portable `read_dir` + `symlink_metadata`
> implementation, and **no performance claim should be made for this crate until the
> syscall layer lands and the benchmark gate passes**. See
> [docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md](docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md).

## Why

Of a dozen surveyed tools in this space (`du`, `ncdu`, `dust`, `dua`, `gdu`, `dut`,
`duc`, `fsearch`, `bfs`, `fd`, `scc`, `tokei`), exactly one persists anything, exactly
one carries multiple metrics per pass, **none** does per-directory type tallies, and
**none** does mtime-based incremental revalidation.
The combination is unoccupied ground, and it is what a live file browser actually needs.

The full survey, with the techniques worth adapting and their sources, is in
[docs/project/research/research-2026-08-06-file-rollup-engine.md](docs/project/research/research-2026-08-06-file-rollup-engine.md).

## Install

```shell
cargo install fdu          # CLI
```

```shell
uv add fdu                 # Python module (not yet published)
```

## Use it

```shell
fdu                        # summarize the current directory
fdu -d 3 ~/src             # three levels deep
fdu --by-type ~/Downloads  # break down by file extension
fdu --json .               # stable, versioned JSON for agents and scripts
```

```text
/workspace/fdu/crates  12 files, 4 dirs, 156 KiB
   140 KiB  █████████░    90%  fdu/
   136 KiB  █████████░    87%    src/
   4.0 KiB  ░░░░░░░░░░     3%    Cargo.toml
```

`--help` is the complete source of truth.
JSON output carries a `schema` field (`fdu.tree/1`) that is versioned with the tool, so
an agent can tell what it is parsing.

## As a Rust library

```toml
[dependencies]
fdu = { version = "0.0.1", default-features = false }
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

## As a Python module

```python
import fdu_py

index = fdu_py.open("/path/to/tree")
print(index.total())          # {'files': ..., 'bytes': ..., 'by_extension': {...}}
print(index.children("src"))  # one call returns every child with its roll-up

mark = index.clock
index.refresh()               # reconcile against the filesystem
print(index.since(mark))      # exactly what changed, or truncated=True if you fell behind
```

Every method is bulk: it returns a whole structured result in one call.
A million small zero-copy calls lose comfortably to one large call.

## How it works

Three artifacts and one contract:

| Artifact | What it is |
| --- | --- |
| **Index** | In-memory parent-pointer tree; every directory carries pre-computed roll-ups |
| **Snapshot** | That index, serialized, invalidated wholesale by an engine fingerprint |
| **Delta** | A typed, clocked change — the *only* way the index or cache is ever modified |

Everything else produces or consumes deltas.
A cold scan is a large batch of upserts; a revalidation sweep is the diff between
snapshot and reality; the watch layer is verified, coalesced filesystem events.
The index knows `apply(Delta)` and nothing about where changes came from, so a batch
run, a synthetic test, and a live watcher are indistinguishable to it.

Freshness is a ladder, not a set of alternatives: the snapshot answers instantly, the
revalidation sweep guarantees correctness at open, and the watcher keeps the gap between
them near zero while the process lives.
**Correctness never rests on the watcher** — a missed event costs staleness until the
next open, never a wrong answer that persists.

Two invariants are non-negotiable, because a cache that lies is worse than no cache:

- Fingerprints are **size + mtime + ctime + inode**, not mtime alone.
  mtime is user-settable and some applications roll it back after writing; ctime is
  kernel-controlled. Borg and restic both learned this the hard way.
- A corrupt or unrecognized snapshot is treated as **absent, never as data**. Failing
  closed costs a rescan; failing open silently corrupts every answer built on it.

## Development

```shell
make build      # debug build, all features
make test       # test suite
make check      # fmt, clippy, tests, docs — the handoff gate
make fix        # apply formatting
```

## License

MIT. See [LICENSE](LICENSE).

Designs adapted from GPL-licensed tools (`dut`’s atomic-refcount roll-up, `fsearch`’s
record layout) are clean reimplementations written from the descriptions in the research
doc, not transliterated from their source.
