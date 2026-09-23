# fdu-core

The engine behind [fdu](https://crates.io/crates/fdu): hierarchical tallies (sizes,
counts, recency, file types) over large directory trees, with a persistent cache and an
optional OS-native watch layer.

This crate is the library.
**If you want the command-line tool, install `fdu`** — it carries the command line and
re-exports this whole API, so `cargo add fdu` gives a library caller everything here
under one name.

Full documentation, design notes, and the tool survey this is built from live in the
repository: <https://github.com/jlevy/fdu>

**Status: 0.x.** A new minor release may change the API;
[the release process](https://github.com/jlevy/fdu/blob/main/docs/project/guides/release-process.md)
states the compatibility rules.
The revision-arbitrated observation/commit contract, bounded parallel walker, applying
reconciler, checksummed snapshot and content sidecars, cache lifecycle, and opt-in
content analyzers are tested end to end.

```shell
cargo add fdu-core
```

Published requirements are caret ranges of the reviewed minimum (`libc`,
`pulldown-cmark`). `Cargo.lock` still pins the exact versions this workspace builds;
`cargo install --locked fdu` is what reproduces them.
An exact pin in a published library would make any downstream that needs a newer
compatible release unresolvable.

The crate has no default features.
The `watch` capability is strictly additive; without it, scan, index, and snapshot
remain fully functional.
The `fdu` command and Python package enable it explicitly, while embedding consumers can
leave it out along with its dependency tree.
`.gitignore` handling is always compiled in, because it has no dependency to shed.
Whether a request reads `.gitignore` files is decided at runtime by its
`ScanConfig::read_controls`, which is on by default; a request that turns it off is a
separate snapshot scope.

Content inspection is optional and disabled by default.
`OpenConfig::analysis` enables bounded streaming line, prose, and common-language SLOC
metrics; sparse type, family, and language summaries; and independently versioned
sidecar reuse without changing the metadata snapshot format or cost model for
metadata-only consumers.
`code-sloc-v1` covers Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, C#, Ruby,
PHP, Swift, Kotlin, shell, and SQL without adding a parser dependency.
`text-logical-v1` adds normalized words, paragraphs, and aggregate-derived pages;
`markdown-prose-v1` adds a reader-visible CommonMark projection that excludes code and
destinations. Exact names and extensions stay path-only, while unresolved and ambiguous
paths may use bounded shebang, modeline, literal, or signature probes whose source and
confidence are retained in reports and sidecars.

```rust
use fdu_core::content::AnalysisSet;
use fdu_core::{OpenConfig, open};
use std::path::Path;

let mut config = OpenConfig::default();
config.analysis.profile = AnalysisSet::ALL;
let (index, report) = open(Path::new("."), &config)?;
let lines = index
    .content_rollup(Path::new(""))
    .map_or(0, |root| root.total.lines.metrics.physical_lines);
println!("{} lines", lines);
assert!(report.analysis.is_some());
# Ok::<(), fdu_core::Error>(())
```

License: MIT.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
