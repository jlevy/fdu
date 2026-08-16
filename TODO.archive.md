# fdu TODO archive

Completed work, kept for the reasoning rather than the checkmark.
Current work is in [TODO.md](TODO.md).

Entries record what was decided and why, because that is the part a later reader needs:
a closed epic’s title says what shipped, and almost never says what was rejected on the
way or what the evidence actually supports.

## Completed epics

| Epic | What it delivered |
| --- | --- |
| `fdu-5rpt` — close the adaptive-worker evidence gap on Apple Silicon/APFS | Falsified the leading explanation for a field report of fdu running slower than `dust`: worker under-scaling was not the cause, and fdu measured faster in every cell the campaign timed. The profile showed 73.7% of samples in kernel/syscall frames against 1.22% in fdu scan code, so parallelism was never the lever. **Production policy deliberately unchanged.** The campaign did not reproduce the symptom, because every measurement it made — and every tool comparison before it — used `--cache off`, which a default invocation does not. That loose end became the cache-layers work. |
| `fdu-3d8c` — polish fdu 0.1.0 packaging and Python API | Release engineering for the non-publishing scope. |
| `fdu-a0w0` — specify and harden the fdu CLI with golden tests | The golden contract suite the CLI’s behaviour is now pinned by. |
| `fdu-6c8n` — CLI UX and zero-install agent skill | The embedded `SKILL.md` surface. |
| `fdu-j5ny` — spec: fast file content metrics | Content analysis and its profile-scoped sidecar. |
| `fdu-vdi9` — close final phase-0 correctness gaps | Branch-wide review remediation. |
| `fdu-jhm7` — clarify and enforce Rust module filenames | Naming convention plus its check. |
| `fdu-p9d5` — copy-edit the performance research white paper | Editorial pass. |

## Completed specs

| Spec | Note |
| --- | --- |
| [CLI golden tests](docs/project/specs/done/plan-2026-08-09-fdu-cli-golden-tests.md) | In `specs/done/`. |
| [file content metrics](docs/project/specs/done/plan-2026-08-12-fdu-file-content-metrics.md) | In `specs/done/`. |
| [rust module filenames](docs/project/specs/done/plan-2026-08-13-rust-module-filenames.md) | In `specs/done/`. |
| [CLI UX and agent skill](docs/project/specs/active/plan-2026-08-09-fdu-cli-ux-and-agent-skill.md) | Complete, but still filed in `active/` because open follow-up epics cite it. |
| [composable CLI surface](docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md) | Implementation complete and merged; follow-ups tracked under `fdu-pxeb` and `fdu-ktyl`. |

## Landed campaigns worth remembering

### The cache cost model, and the field report finally reproduced

Three merged pull requests, August 2026.

The field report — fdu slower than `dust` on macOS — went unreproduced through two
campaigns because everything they measured used `--cache off`. It reproduced on a
494,031-entry tree the moment a *default* invocation was measured: repeat `fdu .` took
4.8 s against `dust`’s 4.4 s, while cold fdu took 3.6–3.9 s. The warm default was slower
than a cold scan of the same view, which the design principles name as a defect rather
than a trade-off.

`FDU_COUNTERS` showed why: a warm run did filesystem work identical to a cold one
(127,915 directory opens, 494,032 stats) *plus* deserialising and reconciling a 38 MB
snapshot, to save a ~50 ms write that an unchanged run skips anyway.
Revalidation stats every entry regardless of what the snapshot holds, so for a metadata
query the snapshot can never pay.
The planner now decides snapshot reads by cost, and repeat runs fell to 3.83 s — level
with `dust` rather than behind it.

Two decisions from that work are worth not re-deriving:

- **`SNAPSHOT_MIN_ENTRIES` stays `None`, and that is a measured decision, not a deferred
  one.** The ext4 data supported a threshold of roughly 250,000 entries.
  APFS measurement did not confirm that number; it removed the premise.
  A no-scan snapshot read costs 0.83 µs/entry against a 2.97 µs/entry walk there — a
  better-than-threefold win where ext4 showed an 18% loss — because deserialisation
  costs about the same on both filesystems while an APFS metadata walk costs roughly
  three and a half times as much.
  The write repays itself about fourfold on first reuse, at any tree size.
  A size gate would have given up real value on exactly the trees it gated.
- **`--cache only` failing after summary-only runs is deliberate.** The compact tier
  retains nothing, so there is nothing to read, and it says so rather than quietly
  scanning. Both halves are pinned in `cli-lifecycle.tryscript.md`.

Consumer-side work landed in the same stack: sharing the index with the snapshot writer
instead of deep-cloning it (−174 ms on macOS, 95% CI [−418, −60]), CRC-32C slicing-by-8,
and skipping unread journal capture.

### Experiment id collision, and what it changed

Two campaigns running in parallel each numbered from the same next-free id and both
claimed `exp-056..059`. Nothing caught it until the branches met, because until then
each side’s artifacts were individually valid — the ledger’s “regenerated from
artifacts, cannot drift” property is real but does not extend to identity.

Resolved by renumbering the later campaign to `exp-060..063` and regenerating.
`make perf-ledger` now fails on a duplicate experiment id.

The obvious stricter rule — one hypothesis, one title — was tried and **rejected by the
data**: `H31` spans twelve experiments under twelve titles, because that is how a claim
is carried through a cumulative run or confirmed on a second platform.
A title-keyed rule would have rejected the committed record and blocked the
cross-platform backfill.
The check warns on one base number wearing several spellings instead, which is the
signature the real collision left.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
