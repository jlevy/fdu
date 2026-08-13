# `code-sloc-v1` Fixture Comparison

- **Corpus:** `tests/golden/fixtures/code-project`
- **fdu analyzer:** `code-sloc-v1`, version 1
- **Tokei:** 14.0.0
- **SCC:** commit `50ea91a853f94fa581e6d505b85b0aef944bd7b5`

The 15-file adversarial corpus pins the first common-language dialect before any parser
performance work. Every file has seven physical lines.
Counts below are `code / comment / blank` under each tool’s source-line partition.

| Language | fdu | Tokei | SCC | Disposition |
| --- | ---: | ---: | ---: | --- |
| C | 2 / 4 / 1 | 2 / 4 / 1 | 2 / 4 / 1 | Match |
| C# | 3 / 3 / 1 | 3 / 3 / 1 | 3 / 3 / 1 | Match |
| C++ | 3 / 3 / 1 | 3 / 3 / 1 | 3 / 3 / 1 | Match |
| Go | 3 / 4 / 0 | 3 / 4 / 0 | 3 / 4 / 0 | Match |
| Java | 2 / 4 / 1 | 2 / 4 / 1 | 2 / 4 / 1 | Match |
| JavaScript | 3 / 4 / 0 | 3 / 4 / 0 | 3 / 4 / 0 | Match |
| Kotlin | 2 / 4 / 1 | 2 / 4 / 1 | 2 / 4 / 1 | Match |
| PHP | 3 / 4 / 0 | 3 / 4 / 0 | 3 / 4 / 0 | Match |
| Python | 5 / 1 / 1 | 5 / 1 / 1 | 2 / 4 / 1 | Intentional: v1 and Tokei count docstring lines as code; SCC counts them as comments |
| Ruby | 2 / 4 / 1 | 2 / 4 / 1 | 2 / 4 / 1 | Match |
| Rust | 2 / 4 / 1 | 2 / 4 / 1 | 2 / 4 / 1 | Match, including nested comments and raw string |
| Shell | 4 / 2 / 1 | 4 / 2 / 1 | 4 / 2 / 1 | Match, including `word#literal` |
| SQL | 2 / 4 / 1 | 2 / 4 / 1 | 2 / 4 / 1 | Match |
| Swift | 2 / 4 / 1 | 2 / 4 / 1 | 2 / 4 / 1 | Match, including nested comments |
| TypeScript | 2 / 4 / 1 | 2 / 4 / 1 | 2 / 4 / 1 | Match |
| **Total** | **40 / 53 / 12** | **40 / 53 / 12** | **37 / 56 / 12** | Python docstring policy explains the difference |

The CLI golden separately pins fdu’s exact per-language human report and percentages.
The parser unit tests cover chunk boundaries, LF, CRLF, lone CR, mixed endings, final
unterminated lines, invalid UTF-8 admission, and unsupported-language coverage.
This matrix is not a claim that the tools share one universal definition; it makes the
one intentional v1 disagreement reviewable.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
