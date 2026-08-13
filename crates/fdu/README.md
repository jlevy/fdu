# fdu

Fast, incremental file roll-up engine: hierarchical tallies (sizes, counts, recency,
file types) over large directory trees, with a persistent cache and an optional
OS-native watch layer.

Full documentation, design notes, and the tool survey this is built from live in the
repository: <https://github.com/jlevy/fdu>

**Status: pre-release.** The revision-arbitrated observation/commit contract, bounded
parallel walker, applying reconciler, checksummed snapshot and content sidecars, cache
lifecycle, and opt-in metric profiles are tested end to end.
The public release matrix remains open, so the crate is not published yet.

```toml
[dependencies]
fdu = { path = "crates/fdu", default-features = false }
```

The crate is not published yet; the version-based dependency form is a Phase 1 release
step.

Features: `cli` (default, gates the binary and clap), `watch` (gates the notify
dependency).

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
use fdu::content::AnalysisProfile;
use fdu::{OpenConfig, open};
use std::path::Path;

let mut config = OpenConfig::default();
config.analysis.profile = AnalysisProfile::Full;
let (index, report) = open(Path::new("."), &config)?;
let root = index.content_rollup(Path::new("")).expect("analysis requested");
println!("{} lines", root.total.metrics.physical_lines);
assert!(report.analysis.is_some());
# Ok::<(), fdu::Error>(())
```

License: MIT.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
