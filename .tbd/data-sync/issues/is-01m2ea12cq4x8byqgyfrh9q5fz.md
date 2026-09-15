---
type: is
id: is-01m2ea12cq4x8byqgyfrh9q5fz
title: Implement durable disk-usage checkpoints and repeatable net comparisons
kind: feature
status: open
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
created_at: 2026-09-13T21:16:01.814Z
updated_at: 2026-09-15T16:09:22.146Z
---
Implement delivery slice 2 in fdu-core with CLI and Python parity: immutable durable checkpoint identity distinct from live clocks, explicit baseline capture and advancement, signed allocated/apparent/count deltas computed before ranking, and repeated A-to-B reads without refresh. Begin with complete scanned roots; preserve pinned baselines across refresh, failure, and cache eviction. Define compatible scope, unknown coverage, rename attribution, own-store exclusion, and concurrency/publication semantics before persistence. Integrate with existing block-format and durable-journal work for later bounded access.

## Notes

2026-09-14 (PR #55 review, 4727de0): slice 2 scope grew; the checkpoint plan now specifies it.
- Checkpoint store separate from the snapshot cache, under the user data directory, with its own format version. Each checkpoint records ScopeIdentity, SemanticIdentity, classification version, and accounting version. Reason: engine_fingerprint mixes the crate version (crates/fdu-core/src/snapshot.rs:173-186 at dda7e6a), so a cache-keyed checkpoint would be discarded on every release.
- Immutable checkpoint ids with movable labels and pins. Comparisons record the resolved ids and refuse a removed checkpoint rather than substituting one.
- Three measures: apparent bytes, per-path allocated bytes, and unique allocated bytes (once per (dev, inode) within one checkpoint). Unique needs a retained link count in fdu-core; Attrs has none (crates/fdu-core/src/engine_contract.rs:81-95 at dda7e6a). Durable hard-link attribution depends on fdu-579b.
- Denied subtrees become typed gaps; a capture with gaps publishes a gap-marked partial checkpoint.
- Open questions for the user are listed at the end of the plan: default measure, label reuse, released-format policy, partial-checkpoint default.

2026-09-15 (PR #55 delta review 5204152578, 55ce4a3): more slice 2 requirements now in the checkpoint plan.
- Volume identity is the filesystem volume UUID (macOS ATTR_VOL_UUID), never st_dev and not the FSEvents database UUID. Where none is observed (Linux, Windows where dev and inode are 0, network and FUSE volumes) record not observed; such a comparison skips only the volume check and is marked volume-unverified. Two observed UUIDs that differ refuse. Rustdoc fix: fdu-4qtk.
- The link count is a new pub field on Attrs (crates/fdu-core/src/engine_contract.rs:81-95 at dda7e6a; not #[non_exhaustive]; re-exported at lib.rs:98-99). Library APIs: DO NOT MAINTAIN, since fdu-core is unreleased. The snapshot record is fixed-width (snapshot.rs:236-241), so the same change bumps FORMAT_VERSION.
- Where (dev, inode) or a link count is unavailable (Windows today), record unique allocated bytes as not observed in the accounting version; rank by per-path allocated under that name and refuse an explicit unique request.
- Attribution is by first in-scope path in bytewise path order. Expected-delta tests now include an out-of-scope hard-link source and renaming one link of a multi-link file.
- Checkpoint format retirement is per user store: a reader refuses both newer and retired older formats with id, version, and the release range that reads them.

2026-09-15 (PR #55 delta review 5205945198; ba70cc0, e0aad48, a2e1eba): more slice 2 requirements now in the checkpoint plan.
- Partially observed classification. fdu-1onj records the user's decisions for 0.1.0: a crossed control budget is an exit-0 coverage note with exact sizes, the budget is mixed into ignore_rules_fingerprint, the per-line guard is raised by the same setting, and snapshot FORMAT_VERSION is bumped to carry refused rules. Each checkpoint records its control budget and every refused control source, read from the index. Byte and count deltas stay exact. If either checkpoint refused a source, ignored/unignored deltas are marked partial at every directory at or below a source refused in either checkpoint and at its ancestors, with the sources named; equal refused sets do not lift the marker. A checkpoint that retains only a count of refused sources marks every classification delta partial. Different budgets differ in SemanticIdentity, so classification is not comparable. A crossed budget is not a gap: the checkpoint is complete. Before slice 2 fixes its API, confirm on main that the index, including one loaded from a snapshot, carries a typed record of its budget and refused sources. Tests: the classification cases in the plan's correctness list.
- Checkpoint format retirement happens after a documented support window (releases since the last release that wrote the format), with no re-encoding duty. Compaction may re-encode, but nothing relies on it; a checkpoint still in a retired format, such as a pinned one compaction never copied, is refused with the releases that read it.
- Renaming one link of a multi-link file: the expected-delta test covers all four outcomes (attributed link renamed and still first, or no longer first; another link renamed and still later, or now first).
