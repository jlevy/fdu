# The fdu Cache: Two Layers, and What Verification Costs

How the snapshot cache is structured, what each policy promises, and why the cost of
proving an answer current depends on which question was asked.

This is a design reference, not a tutorial; `fdu --help` is the usage contract.

Two design principles govern this layer.
[Model Every Key Concept Explicitly, in One Place](../architecture/fdu-design-principles.md#model-every-key-concept-explicitly-in-one-place)
asks that each tier’s identity, its per-item validity rule, and the requests it may
serve be stated once rather than coded into every path that reads or writes it.
[Caching Improves Performance, Never Semantics](../architecture/fdu-design-principles.md#caching-improves-performance-never-semantics)
follows from it: stores (retained indexes, snapshots, and content sidecars) and the
summary-reducer path exist to make answers cheaper, and a cache hit, miss, eviction,
cache policy, or execution path may change only how fast an answer arrives and what its
provenance fields report, never what the answer says.
A store holds one or more tiers, the unit whose identity decides which requests it may
answer: a snapshot holds the entry and `.gitignore` control tiers, and a content sidecar
holds the content tier.
The code does not meet that bar everywhere; [Known Gaps](#known-gaps) lists where, and
[the explicit core models plan](../specs/active/plan-2026-09-17-fdu-explicit-core-models.md)
tracks the work.

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
The scope is the depth, symlink, filesystem-boundary, hidden-entry, and special-object
settings, the type-rules fingerprint, a reducer-set fingerprint that is a constant
today, whether `.gitignore` was observed, and, if it was, the budget and line limit.
A request that returns the index or reconciles it against the tree is served only by a
snapshot taken under exactly its scope.
The file name is keyed by root alone, so alternating a default run with
`--no-gitignore`, another `.gitignore` limit, `--scan-depth`, or `--one-filesystem`
finds no usable snapshot and, under a write-permitting policy, replaces the root’s one
snapshot each time the run retains an index (`fdu-w3l5` tracks keying snapshots by
scope). A `--no-gitignore` summary answered by the transient tier, described below,
retains none, so it replaces nothing.
The one exception is a one-shot `--cache only` report that turns observation off, which
answers from a default snapshot’s all-entry facts and retags the report to its own
scope. `open` with the same options refuses that snapshot.

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
The converse closes the rule.
A name fdu gives one of its own files has already said which magic to expect, so
contents that are not it leave the file unrecognized rather than sending it on to be
identified some other way — otherwise a snapshot image stored under a sidecar’s name
would be read as a snapshot and cleared under a name no snapshot is ever given.

Clearing a snapshot also removes its `.content` sidecar if the sidecar starts with the
sidecar magic. Symbolic links and directories in the cache directory are reported as
unrecognized, never followed or descended into, and never removed.
They are also reported without a byte count: what a filesystem calls a directory’s size
is its own accounting, it differs per platform, and it is not space a clear could
reclaim, so counting it would inflate the figure a status gives for what it leaves
behind. A file that another process removes during a clear is not an error.
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
  should start with. Either alone proves nothing, and a mismatch leaves the file
  unrecognized: the name has already said which magic to expect, so nothing further is
  entitled to a second opinion about it.
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
young to be anyone’s but a running writer’s. A status knows no file’s age — it reads
names and magic, not clocks — so when it lists a staging file it says that one waits
until it is too old to be a running writer’s, rather than promising a count the clear
will then decline and explain.

Machine formats carry the `fdu.cache/1` schema — its own document identity, not a report
schema, because cache status is a fact about the cache directory rather than about a
tree. Every row carries `path`, `bytes`, `content_bytes`, and `state`; a `current` row
adds `root` and `entries`, a `stale` row a `stale_reason` and, for a format mismatch,
the `format_version`, and a `leftover` row its `leftover_kind`. An empty cache directory
is an empty sequence in every machine format, never a null, so one reader works whether
or not anything is cached.

Snapshot persistence is available on every platform, including for metadata queries.
It is not used by every execution plan.
An unfiltered `--view summary` under `--no-gitignore` is answered by the transient tier,
which retains no index and writes no snapshot; the default summary reads `.gitignore` to
report its ignored share, which needs the index, so it does write one.
For indexed reports, a complete scan and a write-permitting cache policy are both
required, and [the policy axis](#the-policy-axis) lists which paths read and write.
A one-shot metadata report skips the read because revalidation stats every entry
regardless, so loading the snapshot is purely additive cost.

## Layer Two: Derived Content Data

Content-derived metrics — line counts, word counts, hashes, and future plugin analyzers
— do **not** belong in the core snapshot.
They live in a separately checksummed sidecar beside the snapshot, `<snapshot>.content`,
recording the engine fingerprint, the entry tier identity the records were analyzed over
(the snapshot’s scope without `.gitignore` observation, which no metric depends on), the
root, the stored analyzer set, the type-rule fingerprint, an options fingerprint, and
ordered analyzer IDs and versions.
It holds one analyzer set per root.
Each sparse file record carries its classification and a fingerprint of size, mtime,
ctime, inode, and device, so a reconciled metadata change rejects only the stale record.
In memory a record also keeps the analyzer set it was produced under, and a save keeps
only records whose set equals the stored one.
The current sidecar format is version 5; an absent, corrupt, oversized, foreign, or
version-mismatched sidecar is a clean analyzer-cache miss and never invalidates the core
snapshot.

Four analyzer dialects ship: streaming physical lines and raw words, common-language
SLOC, normalized plain-text volume, and reader-visible Markdown prose.
Keeping them separate from metadata remains load-bearing rather than tidy:

- The core snapshot stays small and fast to open.
  An analyzer’s output can be far larger than the tree’s metadata, and paying for it on
  every open would penalize the common query.
- Content-sidecar invalidation never touches tree truth.
  A sidecar is usable when its format version, engine fingerprint, entry tier identity,
  and root match, its type-rule fingerprint equals the registry in use, and its stored
  analyzer set contains the requested one (`ContentProvenance::satisfies`). The options
  fingerprint and analyzer versions are recorded but not compared; both are derived from
  the analyzer set today, so a change to an analyzer’s output invalidates stored records
  only through a sidecar format-version bump.
  A narrower sidecar cannot invent a wider result, and a narrower request leaves the
  wider stored set in place.
  Containment is not projection: a report reads the stored records as they are, so a
  narrower request served from a wider sidecar reports the wider set’s metrics and
  label. A mismatch is a clean content-cache miss and never invalidates metadata sizes.
- The layer is loaded only for an explicitly requested analysis profile, bounded before
  allocation, and removed through the same cache lifecycle as its recognized snapshot.
  A metadata-only run never opens it and never pays for content structures.

The payback is largest here.
Re-deriving content over a large repository is minutes; re-deriving only what changed is
seconds, and an unchanged file whose record the sidecar holds is not re-read.

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

A metadata entry is unchanged when every stored attribute matches: size, allocated
bytes, mtime, ctime, inode, and device.
A content record is current when its five-field fingerprint matches: the same attributes
without allocated bytes, because a filesystem can repack a file without changing its
contents.

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
| `auto` (default) | per path, below | full scan or full revalidation | per path, below |
| `refresh` | no | full scan | complete indexed scans |
| `read-only` | per path, below | full scan or full revalidation | never |
| `only` | yes | never, except under `--watch` | never |
| `off` | no | full scan | never |

What a policy reads and writes also depends on the path that answers:

- **One-shot report.** Under `auto` and `read-only` it reads the snapshot only when
  content analysis is requested.
  A metadata-only report is a cold scan, and under `auto` it rewrites the snapshot after
  every complete cold scan; a report with analysis takes the warm path, which writes the
  snapshot only when reconciliation changed something.
  The summary-reducer path retains nothing and writes nothing, so
  `--view summary --no-gitignore` under `auto` never leaves a snapshot.
- **`open` and the first answer of `--watch`.** Both always read a usable snapshot and
  reconcile it. A warm open writes only when reconciliation changed something; a cold
  open writes after a complete scan.
- **Live updates.** Under a write-permitting policy, the command line’s `--watch` saves
  a throttled snapshot whenever the index is fresh.
  Python `Index.refresh` and `Index.watch` never write.
- **Content sidecar.** Written after a complete cold scan with analysis, and after a
  warm open whose analysis applied a record or found a stale one.
  Under `refresh` no sidecar is read, so a narrower analyzer set replaces a wider one.

`only` is the one tier that can be stale, and it says so: its report carries
`source: cache_only` and `freshness: stale`. It fails outright when no usable snapshot
exists rather than quietly scanning, because a fast path that is sometimes a full walk —
with nothing in the output to say which happened — is worse than no fast path.
`--watch --cache only` is accepted: its first answer comes from the snapshot, and it
then verifies and applies live filesystem events.

Every machine-format report carries `source`, `freshness`, `complete`, and `errors`.
Text output names none of them; a partial text run prints its errors as warnings on
standard error.

[`plan_report`](../../../crates/fdu-core/src/execution.rs) and
[`open_for_report`](../../../crates/fdu-core/src/lib.rs) implement these policies.
The transient summary path and snapshot-read bypass mean that `--cache auto` does not
promise a reusable baseline after an arbitrary command.
`--allow-partial` changes exit acceptance; it does not make a partial scan cacheable.

## Future Considerations

### Known Gaps

Each item is a way the present code falls short of
[Caching Improves Performance, Never Semantics](../architecture/fdu-design-principles.md#caching-improves-performance-never-semantics).
[The explicit core models plan](../specs/active/plan-2026-09-17-fdu-explicit-core-models.md)
tracks them.

- **Content containment is not projection.** A narrower request served from a wider
  sidecar answers with the stored records and label: `--analyze lines` after
  `--analyze all` reports different `physical_lines`, `document_words`, share metric,
  and `analysis.analyze` than a cold `--analyze lines`.
- **Mixed records.** A file a narrower request analyzes inside a wider tier gets a
  record under the narrower set.
  Reports aggregate each record by its own set, so totals can match neither cold answer,
  and the save drops those records, so each later run reads the files again.
- **The sidecar’s identity is compared by containment.** A sidecar records its engine
  fingerprint and entry tier and serves only those, but its analyzer versions and
  options are recorded without being compared, and its analyzer set answers any request
  it contains.
- **The one projection exists on one path.** A one-shot cache-only report may answer a
  `.gitignore`-off request from a controls-on snapshot; `open` refuses the same request.
- **Policies mean different things per path.** `read-only` revalidates for `open` but
  behaves as `off` for a one-shot metadata report, and `auto` reads for `open` but not
  for that report. Write rules differ by path as listed above, so whether a later
  `--cache only` succeeds depends on which command ran last.
- **Live provenance and content decay** are session gaps, listed in
  [the engine architecture’s Known Gaps](../architecture/fdu-engine-architecture.md#known-gaps).
- **A type-rules mismatch is silent.** A snapshot taken under another type registry
  parses as absent, so a `--cache only` failure cannot name the registry as the cause.

### Potential Improvements

- **Stores keyed by identity.** One snapshot and one sidecar per root means a request in
  another scope or analyzer set misses and rescans, then replaces the stored one
  (`fdu-w3l5`); `--cache refresh` with a narrower analyzer set likewise replaces a wider
  sidecar. That costs time, never correctness, so keying stores by tier identity is a
  performance improvement rather than a semantic fix.
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
