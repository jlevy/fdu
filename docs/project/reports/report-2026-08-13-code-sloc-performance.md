# Common-Language SLOC Performance Checkpoint

This checkpoint measures the frozen `code-sloc-v1` implementation after its exact
semantic fixtures were committed.
It is engineering evidence for the next optimization, not a universal speed ranking.

## Fdu Component Profile

The symbol-bearing probe repeated analysis of an immutable archive containing 233 files
and 3,175,738 apparent bytes.
Attribution placed 32.73% of samples in fdu content analysis, 14.10% in filesystem
operations, 12.25% in kernel or syscall work, and 8.76% in scanning.
The largest named analyzer frames were `BasicAccumulator::push_text` at 8.74% and
`CodeAccumulator::finish_line` at 5.42%.

The corresponding release probe measured a median end-to-end `code-sloc` time of 20.2 ms
and a median analyzer component of 9.2 ms across 12 valid trials.
The compatible content-sidecar path took 10.1 ms end to end and 2.1 ms in its open
component. The cold result was intentionally partial because two recognized code files
use languages outside the v1 parser set; the semantic digest still covered every metric
slot and matched across variants.

## SCC and Tokei Context

Hyperfine ran each complete CLI 20 times after three warmups with output redirected to
`/dev/null`. SCC 3.7.0 disabled complexity and COCOMO; Tokei 14.0.0 used JSON; fdu
disabled its cache and rendered the language summary as JSON.

| Corpus | fdu | SCC | Tokei |
| --- | ---: | ---: | ---: |
| Immutable self-host, 233 files and 3.18 MB | 11.9 ± 0.4 ms | 9.7 ± 0.5 ms | 13.3 ± 0.9 ms |
| Generated common-language tree, 7,500 files and 0.72 MB | 108.9 ± 4.2 ms | 90.9 ± 1.6 ms | 111.2 ± 17.5 ms |

These figures include each tool’s own walk, ignore handling, classification, analysis,
aggregation, and JSON serialization, so they are useful product-scale context rather
than parser-only throughput.
On these two macOS warm-filesystem corpora, fdu is close to Tokei and approximately
20–23% behind SCC. Fdu additionally builds its reusable metadata/content index and
exposes exact coverage and cache provenance; SCC and Tokei have broader language
grammars.

## First Optimization Verdict

H66 skipped prose-only word, paragraph, and logical-word counters while reading code.
The candidate preserved every semantic oracle and reduced user CPU by 4.67%, but its
12-pair end-to-end interval crossed zero and its +1.50% paired wall result did not clear
the 3% acceptance bar.
Basic and cache-hit negative controls were also neutral, so the implementation was
reverted and recorded as exp-041.

The evidence says the native parser itself is not the dominant end-to-end gap on these
small files. Future SLOC optimization should begin with file-open and scheduling
attribution or use a larger byte-heavy corpus before changing parser mechanics.
The reusable SLOC cold and cache-hit jobs remain in the harness.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
