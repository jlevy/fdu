# `code-sloc-v1` Engine Decision

**Status:** accepted for implementation\
**Decision:** use the native fdu byte state machine for the first common-language
release; retain Tokei and SCC as pinned semantic and performance comparators.

## Decision

The first production `code-sloc-v1` analyzer will remain dependency-free and consume the
same fdu-owned chunks as `content-basic-v1`. It will claim the 15 required common
languages only: Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, C#, Ruby, PHP,
Swift, Kotlin, shell, and SQL. Unsupported code types remain explicit coverage rather
than inheriting nonblank lines as invented SLOC.

Tokei 14 remains the best Rust reuse candidate when breadth is more important than
binary size and scheduler control.
It does not earn inclusion in the first release.
SCC remains the primary implementation reference and external comparator; Go FFI or a
subprocess does not fit fdu’s Rust-library and embedded-Python boundary.

## Prototypes

The Tokei prototype used commit `fa44e5194060305576514d59b850353643afbfc8`, disabled its
default CLI feature, read each file into an fdu-owned-equivalent buffer, and called
`LanguageType::parse_from_slice()` directly.
The narrow native prototype used the new streaming `CodeAccumulator`, which retains one
logical line and parser state rather than a whole file.
Both read the immutable self-host archive before timing and parsed the same 88
recognized files and 1,478,267 bytes 100 times per sample.

| Prototype | Median parser time per 1.48 MB pass | Stripped spike binary | Clean release build |
| --- | ---: | ---: | ---: |
| Native fdu state machine | 9.46 ms | 312 KiB | 15.5 s; 350 MB peak RSS |
| Tokei 14 per-buffer API | 15.10 ms | 2.3 MiB | 23.1 s; 421 MB peak RSS |

These are decision-spike figures, not release performance claims.
Five warm whole-process samples were used.
The parser interval came from an in-process 100-pass loop after all bytes were loaded;
clean build figures include different dependency graphs and are directional rather than
a controlled Cargo benchmark.

The output totals intentionally differ.
Native reported 31,243 code, 3,381 comment, and 3,600 blank lines; Tokei reported 31,398
code, 3,026 comment, and 3,786 blank lines.
The next semantic-lock bead resolves those dialect differences with adversarial
per-language fixtures and pinned SCC/Tokei output.
Agreement by accident is not an acceptance criterion.

## Why Native Won

- It was about 1.6 times faster in the isolated per-buffer self-host spike and produced
  a much smaller linked artifact.
- It preserves fdu’s fixed-size file-worker ownership.
  Tokei’s per-buffer path can invoke Rayon internally, so placing it inside fdu’s worker
  pool creates nested scheduling and weakens cancellation and oversubscription
  guarantees.
- It adds no crate, build script, generated language table, transitive advisory surface,
  or 14-day supply-chain review obligation.
- Streaming state lets the native analyzer read through EOF with constant parser state.
  Tokei needs a complete buffer for the public adapter.
- Existing file fingerprints, analyzer versions, conditional commits, and content
  sidecars work without another cache or traversal owner.

The tradeoff is breadth.
Tokei recognizes hundreds of languages and has more mature embedded-language rules.
fdu will not imply that the narrow v1 parser matches that coverage.
Later releases can expand the native syntax table or revisit an optional Tokei adapter
behind a separately measured analyzer ID.

## Semantic Commitments

`code-sloc-v1` assigns every physical line in a supported source file to exactly one of
code, comment, or code-blank.
A mixed code/comment line is code.
A line inside a block comment is comment even when its source bytes are whitespace-only.
A line inside a multiline string or docstring is code.
Whitespace-only source lines remain available separately from code-blank lines, because
the two partitions answer different questions.
LF, CRLF, lone CR, mixed endings, and a final unterminated line share one contract.

The implementation must not claim long-tail language support through a generic nonblank
fallback. Generated-file and embedded-language classification stay out of v1 until they
have explicit detection rules and fixtures.

## Revisit Triggers

Reconsider Tokei or another library if any of these becomes true:

- users require broad language coverage faster than fixtures can safely extend the
  native table;
- a per-buffer API removes nested parallelism and materially reduces the dependency and
  artifact surface;
- the native parser cannot match pinned common-language fixtures without becoming a
  general tokenizer; or
- controlled end-to-end evidence reverses the spike’s throughput or memory result.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
