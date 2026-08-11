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

It holds entry records and per-directory roll-up state, and it is invalidated wholesale
by an engine fingerprint: a format version, and a hash of the configuration that would
change what the records mean.
A snapshot written by an incompatible build is not migrated and not repaired — it is
treated as absent.

Three rules keep it honest:

- **Corrupt equals absent.** A truncated, foreign, or version-mismatched file is never
  parsed as data, at any entry point — loading, status, or clearing.
  Anything this build cannot identify is also something it will not delete.
- **Only complete scans are written.** A snapshot recording a partial view would be
  served as fact on the next run, and an older complete snapshot is better than that.
- **Writes are atomic.** A temporary file and a rename, so an interrupted write leaves
  the previous snapshot intact rather than a half-file for the next run to reject.

The snapshot is written on every platform, for every tier of query, including plain stat
roll-ups. Skipping the write for cheap queries is a tempting refinement and the wrong
one: the write costs tens of milliseconds off the hot path, while the stat-tier snapshot
is exactly what the two decisive warm paths consume — on a cloud runner the snapshot is
the *only* possible warm state, because the operating system’s own metadata cache cannot
hold a large tree’s inodes in RAM.

## Layer Two: Derived Data (Reserved)

Content-derived metrics — line counts, word counts, hashes, and future plugin analyzers
— do **not** belong in the core snapshot.
They live in a separate per-analyzer layer keyed by
`(fingerprint, analyzer id, analyzer version)`.

No analyzer ships today.
The shape is fixed now so the content tier can arrive without a format break, and
because the split is load-bearing rather than tidy:

- The core snapshot stays small and fast to open.
  An analyzer’s output can be far larger than the tree’s metadata, and paying for it on
  every open would penalize the common query.
- Per-analyzer invalidation never touches tree truth.
  Bumping a line counter’s version invalidates its own column, not the sizes.
- The layer is loaded lazily, bounded in size, purgeable independently, and accumulates:
  a run that asks for richer roll-ups enriches it, and a run that does not never pays.

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

All views shipped today are stat tier, so verification is the per-entry sweep.
Reducers will declare their tier when the reducer registry lands, at which point a
counts-only query legitimately becomes a per-directory sweep — exactly, with no
staleness label needed, because view selection changes verification cost by integer
factors while staying sound.

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
| `auto` (default) | yes | revalidates | on a complete scan |
| `refresh` | no | full scan | on a complete scan |
| `read-only` | yes | revalidates | never |
| `only` | yes | never | never |
| `off` | no | full scan | never |

`only` is the one tier that can be stale, and it says so: its report carries
`source: cache_only` and `freshness: stale`. It fails outright when no usable snapshot
exists rather than quietly scanning, because a fast path that is sometimes a full walk —
with nothing in the output to say which happened — is worse than no fast path.

Every report carries `source`, `freshness`, `complete`, and `errors` in every format, so
no policy can silently serve old or partial data as current.

## What Is Not Built Yet

- **The block snapshot format.** Today’s format is a flat image whose reader is bounded
  and streaming. Compressed blocks with a tail index would make opening O(1) and let
  directory listings materialize lazily.
- **Journal-scoped revalidation.** On macOS the FSEvents journal can name which
  directories changed since a snapshot was written, turning a warm open into O(changes).
  The snapshot format reserves the resume fields.
- **A durable delta journal**, for `since(clock)` across restarts.
- **Cache retention.** Nothing prunes snapshots for roots never queried again, and
  nothing bounds the derived layer’s total size.
  `--cache-clear` is the only reclaim today.

* * *

*Part of the fdu project documentation.
See [AGENTS.md](../../../AGENTS.md).*
