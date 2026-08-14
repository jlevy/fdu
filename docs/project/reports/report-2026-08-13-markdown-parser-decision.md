# Markdown Projection Parser Decision

**Decision:** ship `pulldown-cmark` 0.13.4 with default features disabled for
`markdown-prose-v1`.

The document profile needs CommonMark-aware reader-visible text, not another raw-source
counter. The projection retains headings, paragraphs, link labels, image alt text, and
table cells while excluding destinations, reference definitions, code, metadata blocks,
footnotes, and hidden HTML syntax.
A native ad hoc scanner would be smaller, but it would recreate parsing edge cases and
make malformed input, nested inline constructs, and reference links unreliable.

## Gate Results

- The parser exposes borrowed streaming events and requires no owned document tree.
- fdu streams every eligible file through EOF and retains Markdown metrics only for
  content admitted as UTF-8.
- Default `pulldown-cmark` features are disabled, so the HTML renderer, CLI, `getopts`,
  and serialization features are absent.
- The locked addition is `pulldown-cmark` plus `memchr` and `unicase`; the repository’s
  release-age, checksum, provenance, license, and advisory gates pass.
- The crate’s Rust 1.71.1 minimum remains below fdu’s Rust 1.85 minimum and does not
  affect Python 3.12 or `abi3-py312` compatibility.

On the same host and empty target directories, the optimized CLI build changed from
24.01 seconds and 479 MB peak compiler RSS to 27.84 seconds and 521 MB. The stripped CLI
changed from 1,482,096 bytes to 1,746,592 bytes, a 17.8% increase.
This is accepted as an explicit opt-in-analysis tradeoff because it replaces a high-risk
Markdown grammar implementation; the artifact increase remains visible in the decision
record rather than being presented as free.
Runtime projection performance is measured separately after semantic goldens freeze.

## Owned Semantics

Pulldown-cmark owns CommonMark parsing, but fdu owns the metric dialect.
fdu’s event reducer decides what is reader-visible, counts paragraph blocks, applies the
same additive logical-word statistics as plain text, derives pages only after query
aggregation, and versions the result as `markdown-prose-v1`. Changing parser options or
projection rules requires a new analyzer version and fixture review.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
