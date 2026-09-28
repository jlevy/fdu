# Code Analysis Paired Costs, 2026-09-26

The current release build produced the same report content from a fresh scan and a
compatible seeded cache in all 16 measured cells.
In code cells with source inputs, the warm arm applied its sidecar and opened no source
bodies. Metadata-only requests showed no clear warm benefit, which is consistent with
one-shot revalidation still walking the tree.
These are costs of two delivery modes for the *same population*; they are not an
Include-versus-Exclude speedup comparison.

The
[raw samples and counters](../research/evidence/code-analysis-performance-2026-09-26.json)
were collected with the
[paired runner](../../../explorations/benchmarks/spikes/code_analysis_pair.py).
The release binary reported `fdu 0.1.0-dev+g4016fd998.dirty`; its SHA-256 begins
`25b845016bedd1c5`. The host was bare-metal macOS 26.5.2 on arm64, with subjects on an
external APFS volume.
Host pressure was uncontrolled, so the intervals describe this run rather than a
quiet-host acceptance verdict.

## Protocol

The runner generated four fixed subjects: mixed Rust, Python, prose, and ignored build
files (361 files, 102,488 apparent bytes); one 8 MiB Rust source line plus short files
(25 files, 8,395,351 bytes); repeated generated Rust sources (49 files, 5,190,248
bytes); and an ignored-heavy tree (526 files, 1,333,009 bytes).
Each subject had one metadata `include` cell and code-analysis `include`, `exclude`, and
`only` cells.

For each cell, `--cache=off` answered from a fresh fdu scan and `--cache=auto` answered
with a compatible seeded snapshot and analysis sidecar.
Both used the same path, ignored population, analyzer set, and view.
The runner compared the JSON `status` and `reports` before timing and stopped if they
differed. Two warmups established a warm-steady operating-system namespace/cache state.
Twelve adjacent pairs alternated arm order.
Wall time surrounds process start and wait; CPU time, peak RSS, and page faults came
from `wait4`. Counters were collected in separate untimed invocations.
The reported 95% intervals bootstrap the *paired percentage changes* with a fixed seed
and 4,000 resamples.
A negative change means the seeded-cache arm took less wall time.
This is fdu-cache cold versus warm; neither arm represents a controlled-cold OS cache.
Both arms use the same build, so these pairs measure cache costs and do not establish
whether code analysis changed metadata performance relative to an earlier build.

## Paired Results

| Subject | Population and work | Cold median ms | Warm median ms | Warm change, 95% interval | Peak RSS median, cold / warm MB |
| --- | --- | ---: | ---: | ---: | ---: |
| Mixed | include metadata | 6.30 | 6.40 | +1.7% [−0.6%, +2.8%] | 8.7 / 8.9 |
| Mixed | include code | 10.31 | 8.02 | −22.7% [−23.9%, −20.4%] | 9.9 / 9.8 |
| Mixed | exclude code | 11.66 | 8.48 | −28.5% [−29.2%, −24.8%] | 10.2 / 9.8 |
| Mixed | only code | 7.77 | 7.08 | −7.7% [−10.4%, −7.2%] | 9.2 / 9.0 |
| Long line | include metadata | 5.88 | 6.34 | +4.8% [+2.8%, +8.8%] | 8.4 / 8.4 |
| Long line | include code | 84.20 | 6.52 | −92.2% [−93.0%, −92.1%] | 19.5 / 8.8 |
| Long line | exclude code | 83.72 | 6.37 | −92.4% [−92.9%, −91.8%] | 19.6 / 8.8 |
| Long line | only code | 5.75 | 5.91 | +1.1% [−1.2%, +4.0%] | 8.3 / 8.5 |
| Generated | include metadata | 5.93 | 5.99 | +1.4% [−1.4%, +2.8%] | 8.4 / 8.5 |
| Generated | include code | 22.48 | 6.54 | −71.2% [−72.1%, −70.4%] | 9.8 / 9.0 |
| Generated | exclude code | 23.15 | 6.47 | −72.7% [−73.1%, −71.0%] | 10.0 / 9.0 |
| Generated | only code | 5.57 | 5.62 | −0.1% [−0.8%, +2.8%] | 8.5 / 8.7 |
| Ignored-heavy | include metadata | 6.33 | 6.44 | +0.5% [−2.4%, +2.7%] | 8.8 / 9.0 |
| Ignored-heavy | include code | 14.84 | 8.41 | −43.8% [−44.3%, −41.2%] | 10.4 / 10.1 |
| Ignored-heavy | exclude code | 6.48 | 6.01 | −8.1% [−9.0%, −6.2%] | 9.1 / 8.9 |
| Ignored-heavy | only code | 16.81 | 9.31 | −44.7% [−45.3%, −42.8%] | 10.8 / 10.0 |

The long-line cold analysis allocated 19.4 MB cumulatively and reached a 19.5 MB median
peak RSS; the warm arm allocated 0.66 MB and peaked at 8.8 MB. The generated source cell
allocated 4.7 MB cumulatively on a cold code run for 5.2 MB of source, with a 9.8 MB
median peak RSS. These are bounded observations on the fixed subjects, not a general
maximum-memory guarantee.

## Work Counts and Interpretation

| Subject and code population | Cold entries enumerated | Cold source opens | Cold source bytes read | Warm source opens |
| --- | ---: | ---: | ---: | ---: |
| Long line, include | 26 | 25 | 8,395,351 | 0 |
| Generated, include | 51 | 49 | 5,190,248 | 0 |
| Ignored-heavy, include | 538 | 526 | 1,333,009 | 0 |
| Ignored-heavy, exclude | 28 | 26 | 13,009 | 0 |
| Ignored-heavy, only | 538 | 500 | 1,320,000 | 0 |

The ignored-heavy `exclude` request enumerated 28 entries, rather than the 538 that
`include` and `only` needed, and opened only the 25 kept Rust bodies plus the required
control file. `only` traversed the non-ignored ancestors to discover ignored files and
opened their 500 Rust bodies.
These counters support the intended work-avoidance mechanism.
The timing table does not compare those three different answers as if they were
equivalent.

Three metadata-only cells’ intervals cross zero; the long-line metadata cell’s warm arm
was 4.8% slower in this run.
Their cold and warm directory entry counts match, and no metadata improvement is
claimed.
Raw samples include CPU time, page faults, maximum RSS, and complete application
counters for each cell.
The
[correctness evidence](../research/evidence/code-analysis-correctness-2026-09-26.json)
and
[conditional evaluation](../research/evidence/codebase-analysis-conditional-2026-09-26.json)
cover other acceptance questions; this report establishes only the measured cost and
work profile above.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
