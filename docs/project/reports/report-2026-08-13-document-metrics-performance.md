# Document Metrics Performance Evidence

## Outcome

The document profile has dedicated plain-text, Markdown, and sidecar-cache evidence
jobs. On this Apple M1 Pro, the frozen 1,800-file plain-text corpus initially completed
in about 85.0 ms, the 2,000-file Markdown corpus in 150.9 ms, and an archived 344-entry
fdu tree in 18.0 ms.
Compatible document-sidecar loads took about 19.3 ms, 20.9 ms, and 6.6 ms respectively.
These figures are exploratory host measurements, not release promises.

The Markdown profile attributed 42.82% of samples to fdu content analysis, 11.32% to
allocation, and 10.14% to kernel work.
`BasicAccumulator::push_text` was the largest named frame at 26.68%; Markdown parsing
itself accounted for the next material content costs.

## Iterations

H67 reserved the known bounded Markdown source size before reading.
The 12-pair result moved wall time by −3.55%, but its [−14.49%, +7.45%] interval crossed
zero and the component was neutral.
Most files already fit the first 64 KiB read.
The change was reverted and is recorded in `exp-042`.

H68 removed an unconditional temporary-vector allocation and copy whenever an input
chunk is already complete UTF-8. An unrelated benchmark saturated the host during the
comparison, so the accepted primary run constrained both variants to one worker and used
32 interleaved pairs.
Markdown wall improved 12.04% [−16.46%, −8.38%], component time improved 13.67%, user
CPU improved 12.24%, and peak RSS improved 9.12%. The default-worker diagnostic
independently showed user CPU down 12.71% and peak RSS down 8.29%, although its wall
interval was unusably noisy.

Plain-text wall time was neutral and peak RSS improved 13.33%. The small self-host tree
was wall-neutral while user CPU improved 17.95% and peak RSS improved 10.63%. Cache-hit
latency was neutral.
All runs preserved the engine and content digests; the exhaustive chunk-boundary tests,
87 tryscript scenarios, and self-host assertions also remained green.
H68 therefore landed as commit `2fef9bf` and is recorded in `exp-043`.

## Interpretation

The cost ladder is now clear: logical plain-text metrics are close to filesystem cost,
reader-visible Markdown adds a real parser pass, and compatible sidecars avoid both.
SCC has no equivalent document-volume mode, so its useful comparison remains the code
SLOC path documented separately.
fdu’s opportunity beyond SCC is to retain these text metrics without charging
metadata-only or cache-hit users for parsing.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
