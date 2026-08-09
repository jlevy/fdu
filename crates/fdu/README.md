# fdu

Fast, incremental file roll-up engine: hierarchical tallies (sizes, counts, recency,
file types) over large directory trees, with a persistent cache and an optional
OS-native watch layer.

Full documentation, design notes, and the tool survey this is built from live in the
repository: <https://github.com/jlevy/fdu>

**Status: early scaffold.** The architecture and cache lifecycle are in place and
tested; the fast syscall-level walker is not yet, so no performance claim is made for
this release.

```toml
[dependencies]
fdu = { version = "0.0.1", default-features = false }
```

Features: `cli` (default, gates the binary and clap), `watch` (gates the notify
dependency).

License: MIT.
