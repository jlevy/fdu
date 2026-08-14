# The Performance Iteration Playbook

A reusable method for making a system faster without fooling yourself, and for making
each pass cheaper than the last.

This is written for whoever picks the work up next — a person, or an agent with none of
the context that produced these rules.
Everything here was learned by getting it wrong first, and the failures are named,
because the failure is the part that transfers.

The companion documents are [the performance loop](performance-loop.md), which is fdu’s
specific protocol and hypothesis registry, and
[the design principles](../architecture/fdu-design-principles.md), which decide what a
change is allowed to break.
This one is deliberately domain-neutral: it applies to a filesystem walker, a parser, or
a request handler equally, and the reusable mechanism is factored into the
[`perfkit`](../../../crates/perfkit) crate for that reason.

## The shape of the loop

1. **Instrument first, before optimizing anything.** A loop with good visibility gets
   faster every pass; one without it re-derives the same facts with a profiler every
   time.
2. **Profile before forming a hypothesis.** Intuition about where time goes is reliably
   wrong. Read a caller tree, not a flat profile.
3. **Write the hypothesis down**, including which metric it moves and which regime it
   applies to, before changing code.
4. **Change one thing.**
5. **Measure paired and interleaved** against a control, having first verified both
   produce identical output.
6. **Apply the accept rule**, and drop what fails it.
7. **Record the result, especially the failures**, in a schema-validated experiment.
8. **Re-screen the queue**, because the change you just landed may have eaten the next
   hypothesis’s headroom.

Steps 1 and 7 are what make this a loop rather than a sequence, and they are the two
most often skipped.

## Instrumentation

### The three tiers

Each answers a question the others cannot, and using one alone is the most common route
to a confident wrong conclusion.

| Tier | Source | Cost | Answers |
| --- | --- | --- | --- |
| Application | counters at call sites | ~1 ns/event | *which layer* did the work |
| Process | kernel, sampled per phase | one file read | what the process *really* did |
| External | `strace -c`, `perf`, callgrind | 2×–50× | ground truth, and call sites |

The application tier attributes cost to a layer, which is what tells you where to change
code — but it counts what *your code believes* it did.
The process tier is real kernel data that cannot be fooled and cannot attribute.
The external tier is authoritative about both and far too slow to leave on.

**The discipline is cross-checking the application tier against the process tier**, and
reaching for the external tier only when they disagree or when neither can name a call
site.

### Know what each tier can actually tell you

Assuming coverage is uniform produces confident nonsense.
A worked example that has already misled once here: `/proc/self/io` reports `syscr` and
`syscw`, which read like syscall counts.
They count the read and write families only.
A walk over 17,128 directory entries — every one a `getdents64` or a `statx` — moved
`syscr` by **30**.

So on Linux there is no cheap in-process source for enumeration or stat syscall counts,
and an application counter at the call site is the correct instrument, with `strace -c`
as the periodic ground truth.
Knowing which facts each tier can supply is most of the skill.

### Rules that keep the instrument from distorting the measurement

Instrumentation that changes what it measures is worse than none, because it is
believed.

1. **Thread-local and non-atomic.** A shared atomic in a parallel section measures the
   counter’s contention, not the work.
   Threads fold into the global total when they finish.
2. **Count, do not time, on per-event paths.** A clock read costs an order of magnitude
   more than an integer increment.
   Time whole phases; count events.
3. **Toggle at runtime, not only at build time.** See below.
4. **Measure the overhead and record it** as an experiment, re-run when the
   instrumentation grows.

### Runtime toggles, and when a build flag is right instead

Prefer a runtime switch.
A build flag means two binaries, a rebuild to see anything, and a standing risk that the
measured build and the shipped build differ in ways nobody is tracking.
A relaxed atomic the branch predictor always gets right costs approximately nothing next
to the work being counted.

There is no rule for or against build flags — it is a setting, chosen on evidence:

| Choose | When |
| --- | --- |
| Runtime toggle | the check is cheap next to the work it guards — the usual case |
| Build flag | the instrumentation costs binary size, a dependency, or measurable time |
| Dev build only | it needs something that cannot ship at all |

The way to choose is to measure both.
Three questions, three A/Bs:

- **Recording cost**: instrumented build, on versus off.
- **Idle cost**: instrumented build with recording off, versus a build with no
  instrumentation at all.
  This is the one people forget, and it is the one that justifies leaving the code in.
- **Distortion**: does the instrumented build reach the same *conclusion* about a change
  as the uninstrumented one?
  A 1% overhead that is uniform is harmless; a 1% overhead concentrated in the path
  under test is not.

Keep a way to build without it even when the cost is nil, so the idle-cost question
stays answerable later.

### What to count

Aim at the layers where systems actually spend time, not at whatever is easy to reach:

- **Syscalls**, by kind, at the call site.
  Enumeration, stat, open, read.
- **Allocation** — count, reallocation count, and bytes.
  Routinely the largest line in a systems profile and the least visible without a
  profiler; here it was ~35% of a cold scan’s engine work.
- **Work the code chose to do**: cache hits versus misses, retries, fallbacks, items per
  batch. These are what distinguish a change that removed work from one that moved it.
- **Ratios per unit of work**, not just totals.
  “Allocations per entry” transfers between trees of different sizes; “6.9 million
  allocations” does not.

### A counter that reads zero is worse than no counter

A page of zeroes invites the conclusion that the work did not happen.
This failed twice in one sitting here: once instrumenting a serial path while the
parallel path — the one actually used — went untouched, and once when a lint fix hoisted
a call out of a `match` scrutinee and took the counter with it.
Both compiled. Both passed every other test.
Both reported zero.

The guard is a test that asserts **equality against the system’s own totals**, not
non-zero, and that covers every path the work can take.
An earlier version of that test here covered one of two walkers and passed with the
other’s counter deleted.
Verify the guard by deleting a counter and watching it fail; a test you have not seen
fail is not yet evidence.

Where a number genuinely cannot be obtained, leave the counter out and say why.
Absent is honest; pinned to zero is a lie with a plausible face.

## Measuring

**Verify identical output before timing anything.** A faster wrong answer is not a
result. Compare across every mode and view, ignoring only genuinely nondeterministic
fields — and check the control against *itself* first, so you learn which fields those
are rather than assuming.

**Pair and interleave.** Run A, B, A, B, so that drift in machine state hits both arms.
Never compare a run from this morning against one from this afternoon.

**Run both orderings** when a result is surprising.
If A:B and B:A disagree on the sign, the effect is position bias or noise.

**Have an accept rule, written down, applied without negotiation.** Here it is: median
at least 3% better, the whole 95% interval on the right side of zero, and the complexity
worth it. A change that misses the bar is dropped even when the mechanism is real —
*especially* then, because a real mechanism is exactly what makes a small number feel
worth keeping.

**Predict the metric the rule scores.** A prediction of “15%” that describes a component
while the rule scores wall time is not a near miss; it is a category error.
It happened here, in exp-051: the component moved 16.6%, the wall 7.35%, and the
prediction was right about a number nobody was grading.

**Expect a surprising result to be noise until it survives.** A 25-pair run here
reported a 3.42% regression with an interval clear of zero.
It was mechanically plausible — the code path really was reached.
At 45 pairs it vanished; both orderings disagreed; a third harness put it at +0.54%
spanning zero. Three measurements to kill one plausible number.

## Recording

Record **every** experiment, and the rejections most of all.
Negative results are the most reusable thing a performance campaign produces: they stop
the next person, or the next agent, re-running a dead end.
Of the experiments recorded here, more were refuted than confirmed, and the refutations
are what make the queue trustworthy.

Use a **schema-validated format** with a machine-checkable envelope.
Prose rots; a schema keeps every entry answering the same questions:

- The hypothesis, and which tier or regime it applies to
- Control and candidate, precisely enough to rebuild both
- The primary job and the primary metric — the one the accept rule scores
- Median, interval, trial count
- The decision, and the reason in a sentence someone can disagree with
- Lines changed, new dependencies, new unsafe
- Host, platform, and cache state

**Record the regime, not just the number.** A constant measured on one platform is
*inherited*, not proven, on the others.
Say which.

**Write down what the prediction got wrong**, not only whether it was met.
“Right about the component, wrong about the wall” is a reusable lesson; “missed” is not.

## Things that will bite you

**Two hypotheses competing for one cost.** Whichever lands first captures the win and
the second measures noise.
This happened twice here.
Re-screen the queue after every landing rather than working down the list.

**The harness in the profile.** A probe’s own verification digest measured 38.8% of one
profile. Subtract the harness before quoting any percentage, or you will attribute its
cost to the system.

**Flat profiles.** They attribute cost to a function, not to a reason.
`malloc` at the top tells you nothing you can act on; the caller tree tells you which
layer is allocating.
Read both, and trust the tree.

**Defensive code you cannot test.** A guard for a hazard that cannot occur reads as
evidence the hazard exists.
If you cannot write a test that fails without it, delete it and leave the reasoning as a
comment — or better, as a test that pins the reasoning.

**A test that cannot fail.** Verify guards by mutation: break the thing, watch the test
go red, put it back.
Applies with double force to caching and instrumentation, where a silent wrong answer
looks exactly like a right one.

## Why this is worth the ceremony

The instrumentation in this repository paid for itself inside one session.
A parent-memo hit rate that had taken a callgrind run to establish became a number
printed by an ordinary build.
An assumption the campaign had carried for weeks — that allocation was concentrated in
the index consumer — was inverted by two counters in the time it took to run two probe
jobs.

That is the compounding return: each pass leaves the next one cheaper, and the loop gets
better at answering questions rather than merely faster at asking them.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
