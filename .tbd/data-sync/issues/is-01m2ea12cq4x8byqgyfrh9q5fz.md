---
type: is
id: is-01m2ea12cq4x8byqgyfrh9q5fz
title: Implement durable disk-usage checkpoints and repeatable net comparisons
kind: feature
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
created_at: 2026-09-13T21:16:01.814Z
updated_at: 2026-09-15T00:31:52.966Z
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
