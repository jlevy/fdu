# fdu

Fast, incremental file roll-up engine: hierarchical tallies (sizes, counts, recency,
file types) over large directory trees, with a persistent cache and an optional
OS-native watch layer.

Full documentation, design notes, and the tool survey this is built from live in the
repository: <https://github.com/jlevy/fdu>

**Status: early scaffold.** The revision-arbitrated observation/commit contract,
applying reconciler, bounded payload-checksummed snapshot reader, and cache lifecycle
are tested.
`open()` currently blocks until a full reconciliation completes, and the fast
syscall-level walker is not built, so no latency or throughput claim is made.

```toml
[dependencies]
fdu = { path = "crates/fdu", default-features = false }
```

The crate is not published yet; the version-based dependency form is a Phase 1 release
step.

Features: `cli` (default, gates the binary and clap), `watch` (gates the notify
dependency).

Content inspection is optional and disabled by default.
`OpenConfig::analysis` enables bounded streaming line and prose metrics, sparse
type/family summaries, and independently versioned sidecar reuse without changing the
metadata snapshot format or cost model for metadata-only consumers.

License: MIT.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
