---
type: is
id: is-01m2ymchsj7j6ayg8j46kc1v00
title: Code analysis retains an unbounded logical line despite allocation-bounded wording
kind: bug
status: closed
priority: 2
version: 6
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
labels:
  - content
  - memory
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-20T05:24:54.704Z
updated_at: 2026-09-30T05:58:13.039Z
closed_at: 2026-09-30T05:58:13.038Z
close_reason: "Fixed in 539f49079a9b47b2e09ad2fcf32487a28e9c31b4: CodeAccumulator classifies each line in pieces over a 64 KiB window (LINE_WINDOW_BYTES), carrying lexer state plus a per-line LineScan (started-in-comment/string, code seen, rest ignored, C splice, heredoc probe); a site whose lookahead the window edge cuts waits for more bytes; a stalled scan doubles its threshold so a token longer than the window costs one window plus a chunk. Tests (crates/fdu-core/src/content/content_code_metrics.rs): the old whole-line classifier is kept verbatim as a test oracle; piecewise_classification_agrees_with_the_whole_line_classifier_everywhere (15 languages x every piece edge x window sizes 1..64 KiB, CRLF, lone CR, splices, BOM, invalid UTF-8, Unicode whitespace, no trailing newline, tokens longer than the window), every_chunk_boundary_of_a_mixed_source_agrees, a_long_line_costs_the_window_not_the_line, a_sixty_four_mebibyte_line_holds_at_most_the_window_and_a_chunk. Verification: 64 MiB single-line JS file with --analyze code: peak RSS 71 MiB before, 9.98 MiB after (baseline 10.0 MiB), same answer (1 code line). Core suite 998 passed, workspace green, clippy clean."
resolution: null
duplicate_of: null
---
Root correctness review at 7711150f: content/content_code_metrics.rs CodeAccumulator holds line: Vec<u8>, appends every byte until CR/LF, and clear() retains its peak capacity for the rest of the file. A supported minified or generated source file consisting of one very long line therefore consumes memory proportional to that line, potentially the entire file, per analysis worker. The module claims allocation-bounded classification. The unknown-type prefix fix does not bound this analyzer buffer once a supported code language is selected. Preserve exact results: implement incremental lexical state with bounded lookahead rather than truncating or skipping input, and prove chunk-boundary equivalence for every supported syntax. Until that design is implemented, document the longest-line memory limitation alongside exact Markdown buffering. This is distinct from b2qz whole-Markdown parser/source retention.

## Notes

2026-09-20 final release audit: this is a documented operational resource limit rather than an answer-correctness defect: supported code still produces exact metrics, but the active CodeAccumulator buffer grows with the longest logical line per worker and can cause OOM on adversarial minified/generated input. Clearing or shrinking after a newline does not bound the longest line; truncation or size-skipping would violate exactness. A safe fix requires a streaming lexical DFA with bounded lookahead and chunk-boundary equivalence across all 15 supported syntaxes, so keep this bead open as a substantial redesign and retain the longest-line limitation in public/module documentation.

2026-09-30 stability pass (claude/stability-fixes): re-read content_code_metrics.rs at b1376507. CodeAccumulator::push appends every byte of a physical line to line: Vec<u8> and classify_line (about 450 lines, one state machine over the complete line with State carried between lines) decides Code/Comment/Blank from the whole line: heredoc terminators compare the whole line, Ruby =begin/=end test line starts, and the lexer looks ahead for multi-byte openers and closers whose length is not fixed (Rust r###", C++ R"delim(, SQL $tag$, shell and PHP heredoc tags, Ruby %q(). A bounded buffer therefore needs classify_line to accept a line in pieces, report how much it consumed, and carry the unconsumed tail plus the Code/Comment flags across pieces for every one of the 15 syntaxes, with a chunk-boundary equivalence test per syntax; that is the streaming lexical DFA the 2026-09-20 audit described, and it is a redesign rather than a fix. Shrinking the Vec after a long line would bound steady-state retention but not the peak, which is the longest logical line, so it does not answer the bead. Not fixed in this pass; the longest-line limitation stays documented. Unknown-type files no longer feed this buffer past 16 KiB unless they classify as code (fdu-b2qz notes).
