# The fdu Cache: Two Layers, and What Verification Costs

How the snapshot cache is structured, what each policy promises, and why the cost of
proving an answer current depends on which question was asked.

This is a design reference, not a tutorial; `fdu --help` is the usage contract.

## Why a Cache Exists at All

A filesystem tells you almost nothing about what changed.
Creating, deleting, or renaming an entry updates the mtime of exactly the directory that
holds it, and **an in-place edit updates no directory at all** — not even the file’s
immediate parent. There are no recursive sizes and no recursive timestamps; the one
attempt in this space, APFS fast directory sizing, was never wired up.

So the hierarchy fdu reports is one the filesystem does not maintain.
The index reconstructs it, and the snapshot is that reconstruction made durable.
The cache is not an optimization bolted onto a walker — it is the only place the
recursive answer lives between runs.

## Layer One: The Core Snapshot

One file per root, under the user cache directory, named by a hash of the canonical root
path so two trees never collide.

It holds entry records from which loading rebuilds per-directory roll-ups, and it is
invalidated wholesale by an engine fingerprint of the crate version, the format version,
and the classification-rules version.
A snapshot written by an incompatible build is not migrated and not repaired — it is
treated as absent. Because the crate version is part of the fingerprint, every release
invalidates every existing snapshot; nothing that must outlive an upgrade belongs in
this cache.

The file also records the scan scope it was built under.
A snapshot whose scope cannot serve the request is a miss as well, and under a
write-permitting policy the next complete indexed scan replaces it.

Three rules keep it honest:

- **Corrupt equals absent, on the load path.** A truncated, foreign, or
  version-mismatched file is never parsed as data: loading treats it as no snapshot at
  all and scans. Status and clearing are the entry points that do look at it, and only
  through the bounded header — enough to say what it is, never enough to serve it.
  A file that is not fdu’s is never deleted.
- **Only complete scans are written.** A snapshot recording a partial view would be
  served as fact on the next run, and an older complete snapshot is better than that.
- **Writes are atomic.** A temporary file and a rename, so an interrupted write leaves
  the previous snapshot intact rather than a half-file for the next run to reject.

### Stale Snapshots and What Clearing Removes

An invalidated snapshot is useless to this build but still occupies disk, and the root
that wrote it may never be scanned again to replace it.
So `--cache-status` and `--cache-clear` identify files by their contents, and sort each
entry in the cache directory into one of four states:

| State | What it is | Cleared? |
| --- | --- | --- |
| `current` | A snapshot this build reads | Yes |
| `stale` | Begins with the snapshot magic, but has an older or newer format version, another engine fingerprint, or a header this build cannot read — a truncated file, or one whose paths another operating system encoded | Yes |
| `leftover` | One of fdu’s own files that is not a snapshot in place: a staging file a killed writer never renamed, or a content sidecar whose snapshot is gone | By `=all`, under the rules below |
| `unrecognized` | Anything else: no magic behind the name, a name the cache gives nothing, or not a regular file (a directory or a symbolic link) | Never |

Recognition does not depend on this build’s format.
Every format fdu has written starts with the same magic, followed by the format version
and the engine fingerprint at fixed offsets, so a snapshot from any release is
recognized as stale, and one from an older format reports its version.
A name alone proves nothing: a listing counts a file as a snapshot only when it has both
the snapshot’s name pattern (sixteen hex digits and `.fdu`) and the magic.

Clearing a snapshot also removes its `.content` sidecar if the sidecar starts with the
sidecar magic. Symbolic links and directories in the cache directory are reported as
unrecognized, never followed or descended into, and never removed.
A file that another process removes during a clear is not an error.
Each file is identified again immediately before removal, so a file replaced after the
listing by something that is not fdu’s survives.

### Leftovers, and the Two Rules That Let a Clear Take Them

Two files here are fdu’s without being a snapshot in place.
A writer killed between staging and rename leaves `.{16 hex}.fdu.tmp.{…}`, which begins
with the snapshot magic; removing a snapshot whose sidecar removal is interrupted leaves
a `{16 hex}.fdu.content` with no snapshot.
Both used to be reported as “not an fdu snapshot”, which told the user to leave fdu’s
own debris alone, and nothing collected either one — the temporary only when the same
root is written again, the sidecar never.

`--cache-clear=all` reclaims them, under rules that make removal safe without asking the
operating system a question it cannot answer:

- A leftover must match both the name fdu gives that file **and** the magic its contents
  should start with. Either alone proves nothing.
- A staging file is removed only once it is older than the age at which the writer’s own
  reaper would collect it (a day), so a clear can never take a file a running writer
  still holds. No liveness check is portable, and pid-based ones are wrong under pid
  reuse.
- A content sidecar is removed only while no snapshot claims it — decided after the
  snapshots this clear removes are gone, so the sidecar of a snapshot that survives
  stays, and a later analyzed scan can still use it.
- Snapshots go first and leftovers second, for exactly that reason.

A root’s own `--cache-clear PATH` reaches only the one path that root’s snapshot
occupies, so leftovers are an `=all` matter.

Status text names the command that reclaims stale snapshots: `fdu --cache-clear PATH`
for one root, and `fdu --cache-clear=all` for the directory, which also removes current
snapshots. Clearing says what it removed and what it left, including a staging file too
young to be anyone’s but a running writer’s.

Machine formats carry the `fdu.cache/1` schema — its own document identity, not a report
schema, because cache status is a fact about the cache directory rather than about a
tree. Every row carries `path`, `bytes`, `content_bytes`, and `state`; a `current` row
adds `root` and `entries`, a `stale` row a `stale_reason` and, for a format mismatch,
the `format_version`, and a `leftover` row its `leftover_kind`.

Snapshot persistence is available on every platform, including for metadata queries.
It is not used by every execution plan: the transient summary path does not retain an
index or write a snapshot.
For indexed reports, a complete scan and a write-permitting cache policy are both
required.
The planner may also skip reading a snapshot when loading it and performing the
full metadata sweep would cost more than scanning directly.

## Layer Two: Derived Content Data

Content-derived metrics — line counts, word counts, hashes, and future plugin analyzers
— do **not** belong in the core snapshot.
They live in a separately checksummed `.content` sidecar keyed by the type-rule
fingerprint, requested profile, semantic options, and ordered analyzer IDs and versions.
Each sparse file record also carries size, mtime, ctime, and inode, so a reconciled
metadata change rejects only the stale record.
The current sidecar format is version 4; an absent, corrupt, oversized, foreign, or
version-mismatched sidecar is a clean analyzer-cache miss and never invalidates the core
snapshot.

Four analyzer dialects ship: streaming physical lines and raw words, common-language
SLOC, normalized plain-text volume, and reader-visible Markdown prose.
Keeping them separate from metadata remains load-bearing rather than tidy:

- The core snapshot stays small and fast to open.
  An analyzer’s output can be far larger than the tree’s metadata, and paying for it on
  every open would penalize the common query.
- Content-sidecar invalidation never touches tree truth.
  The current sidecar is profile-scoped: changing any requested analyzer identity or
  semantic option misses that sidecar as a unit, but never invalidates metadata sizes.
  Separate per-analyzer reuse across profile changes remains future work.
- The layer is loaded only for an explicitly requested analysis profile, bounded before
  allocation, and removed through the same cache lifecycle as its recognized snapshot.
  A metadata-only run never opens it and never pays for content structures.

The payback is largest here.
Re-deriving content over a large repository is minutes; re-deriving only what changed is
seconds, and unchanged files are never re-read.

## Verification Cost Follows the Question

Under `--cache auto`, “revalidate” means the cheapest **sound** verification for the
reducers the requested views actually use.
Different metrics depend on different filesystem state, so they cost different amounts
to prove:

| Tier | Depends on | Cheapest sound verification | Directory fingerprints suffice? |
| --- | --- | --- | --- |
| Name | The namespace only — counts, tree shape, extension tallies by name | One stat per **directory** | Yes, per directory |
| Stat | Per-file inode attributes — sizes, mtimes | One stat per **entry** | No |
| Content | File bytes — line counts, hashes | One stat per entry, then re-read only changed files | No for the sweep; yes for the expensive part |

Three consequences worth stating plainly, because each is a way to be subtly wrong:

- **Name-tier pruning is per directory, never per subtree.** A deep namespace change
  bumps only its immediate parent, so every directory in the subtree still needs its
  fingerprint checked.
  Fingerprints do not compose upward.
- **Size roll-ups are exactly what directory fingerprints cannot protect.** The most
  common change in a working tree — editing a file in place — changes its size and no
  directory’s mtime.
- **Within a tier, extra attributes are free.** One stat returns size, mtime, ctime, and
  inode together, so a sizes-only view costs the same as sizes-plus-timestamps.
  The jumps are at tier boundaries, not proportional to how many metrics a view carries.

Metadata views shipped today are stat tier, so verification is the per-entry sweep.
An explicitly requested content profile performs that same sweep, restores every
fingerprint-compatible sidecar record, and opens only files whose requested metrics are
absent or stale. A future reducer registry can let a counts-only query become a
per-directory sweep — exactly, with no staleness label needed — because view selection
changes verification cost by integer factors while staying sound.

## Fingerprints, and the Race They Have to Survive

An entry is unchanged when size, mtime, ctime, and inode all match.

mtime alone is not enough: it is settable by userspace, and some tools restore it after
modifying a file. ctime is kernel-controlled and catches that.
Inode catches replace-by-rename.

The subtle case is the **racily clean** one: a file modified within the same timestamp
tick as the snapshot’s capture is indistinguishable from one that was not touched.
Any future revalidation shortcut must keep treating a fingerprint whose mtime equals the
capture instant as suspect rather than clean, and the snapshot has to record the
filesystem’s timestamp granularity it was captured under.
Git and borg both close this window explicitly; it is not hypothetical.

Filesystems that do not supply stable inodes — some FUSE mounts and network filesystems
— make an inode-bearing fingerprint report false changes rather than false matches.
That is the safe direction, and the per-filesystem policy layer is where any relaxation
belongs.

## The Policy Axis

| Policy | Reads snapshot | Touches filesystem | Writes snapshot |
| --- | --- | --- | --- |
| `auto` (default) | when the execution plan benefits | full scan or full revalidation | complete indexed scans |
| `refresh` | no | full scan | complete indexed scans |
| `read-only` | when the execution plan benefits | full scan or full revalidation | never |
| `only` | yes | never | never |
| `off` | no | full scan | never |

`only` is the one tier that can be stale, and it says so: its report carries
`source: cache_only` and `freshness: stale`. It fails outright when no usable snapshot
exists rather than quietly scanning, because a fast path that is sometimes a full walk —
with nothing in the output to say which happened — is worse than no fast path.

Every report carries `source`, `freshness`, `complete`, and `errors` in every format, so
no policy can silently serve old or partial data as current.

[`plan_report`](../../../crates/fdu-core/src/execution.rs) and
[`open_for_report`](../../../crates/fdu-core/src/lib.rs) implement these policies.
The transient summary path and snapshot-read bypass mean that `--cache auto` does not
promise a reusable baseline after an arbitrary command.
`--allow-partial` changes exit acceptance; it does not make a partial scan cacheable.

## What Is Not Built Yet

- **The block snapshot format.** Today’s flat image is read and rebuilt in full.
  Persisted aggregates and indexed blocks could make bounded summary queries avoid
  materializing every entry; that cost must be measured with validation and persistence
  included.
- **FSEvents history replay.** On macOS, the persistent FSEvents history can name which
  scopes need fresh observations.
  The current snapshot does not store a replay cursor, and reducing filesystem work
  alone would still leave full-image load/save costs.
- **Durable checkpoints and comparison.** The cache replaces the latest inventory for a
  root; it does not retain a user-selected baseline.
  The index journal behind `since(clock)` is process local and starts empty on load.
  The
  [disk-usage checkpoint plan](../specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md)
  specifies immutable checkpoints in a store separate from this cache, repeatable
  deltas, history-replay refresh, and checkpoint-aware retention.
- **Cache retention.** Nothing prunes snapshots for roots never queried again, and
  nothing bounds the derived layer’s total size.
  `--cache-clear` is the only reclaim today, and no scope removes stale snapshots while
  keeping current ones: `--cache-clear=all` removes both.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
