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
The
[explicit core models plan](../specs/active/plan-2026-09-17-fdu-explicit-core-models.md)
records the implementation and acceptance checks for these contracts.

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

The destination resolves in this order: explicit `--cache-dir` (Python `cache_dir`, Rust
`default_cache_path_in`), `FDU_CACHE_DIR`, `XDG_CACHE_HOME/fdu`, then the native
default. Both macOS and Linux default to `~/.cache/fdu`; Windows uses
`%LOCALAPPDATA%/fdu`. An override names the exact cache directory, with no appended
`fdu` component except for `XDG_CACHE_HOME`. Status and clearing use this same resolver.

Each canonical root maps to a 16-digit hexadecimal key.
Its filesystem snapshot is `<key>.metadata.bin`; derived analysis records are
`<key>.analysis.bin`. The metadata file retains filesystem facts and ignore controls,
while the analysis file retains counts and classification evidence, not source-file
bodies. Both are binary, disposable cache data.
`CachePaths` owns the pair so reading, writing, status, and clearing agree.
An explicit metadata path outside the conventional name shape gets its sidecar by
appending `.derived.bin` to the whole path.
This keeps arbitrary names distinct from each other and from conventional
`.analysis.bin` sidecars.

It holds entry records from which loading rebuilds per-directory roll-ups, and it is
invalidated wholesale by an engine fingerprint of the crate version, the format version,
and the classification-rules version.
A snapshot written by an incompatible build is not migrated and not repaired — it is
treated as absent. Because the crate version is part of the fingerprint, every release
invalidates every existing snapshot; nothing that must outlive an upgrade belongs in
this cache.

The file’s header also records the scan scope it was built under, as the identity of
each tier the snapshot holds, and the start of the pass that last wrote the image.
A snapshot whose scope cannot serve the request is a miss as well, and the next complete
indexed scan that writes replaces it.
The entry tier records depth, symlink, filesystem-boundary, hidden-entry, and
special-object settings, its ignored population, the type-rules fingerprint, and a
reducer-set fingerprint that is a constant today.
`include` retains all entries; `exclude` and `only` admit entries according to
`.gitignore` classification.
For a narrowed population, the entry identity also records a fingerprint of the
governing control limits, since changing those limits can change which entries are
retained. The control tier records whether `.gitignore` was observed and, if it was, the
budget and line limit.
Exact identity serves directly.

An `include` snapshot taken with observation on or off has the same entry identity.
A controls-on `include` snapshot may serve a controls-off `include` request: the loader
validates and discards the stored control section while it builds the index in the
requested blind scope.
A narrowed population requires control observation; it cannot use this projection or a
snapshot of another population.
The file name is keyed by root alone, so alternating a default run with
`--no-gitignore`, another `.gitignore` limit, `--scan-depth`, or `--one-filesystem`
finds no usable snapshot and, in a run that writes, replaces the root’s one snapshot
each time the run retains an index (`fdu-w3l5` tracks keying snapshots by scope).
An unfiltered summary answered by the transient tier, described below, retains none, so
it replaces nothing.
That projection applies to one-shot reports, Rust and Python `open`, cache-only reads,
warm revalidation, and watch startup.
A projected index never replaces the stronger controls-on snapshot, even after a watch
observes changes. A controls-off snapshot cannot serve a controls-on request: a scanning
request walks the tree cold, while `--stale-ok` reports a miss.

The pass start is a lower bound on when the snapshot’s facts were last verified.
A later pass that encodes the same facts keeps the file, stamp included, rather than
rewriting it for the stamp alone, and moves only the file’s modification time, forward
to its own start; reconciliation does not advance the stamp.
Loading still reads the modification time as the observation time of the cached entries.

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
the snapshot’s name pattern (sixteen hex digits and `.metadata.bin`) and the magic.
The converse closes the rule.
A name fdu gives one of its own files has already said which magic to expect, so
contents that are not it leave the file unrecognized rather than sending it on to be
identified some other way — otherwise a snapshot image stored under a sidecar’s name
would be read as a snapshot and cleared under a name no snapshot is ever given.

Clearing a snapshot also removes its matching `.analysis.bin` sidecar if the sidecar
starts with the sidecar magic.
Symbolic links and directories in the cache directory are reported as unrecognized,
never followed or descended into, and never removed.
They are also reported without a byte count: what a filesystem calls a directory’s size
is its own accounting, it differs per platform, and it is not space a clear could
reclaim, so counting it would inflate the figure a status gives for what it leaves
behind. A file that another process removes during a clear is not an error.
Each file is identified again immediately before removal, so a file replaced after the
listing by something that is not fdu’s survives.

### Leftovers, and the Two Rules That Let a Clear Take Them

Two files here are fdu’s without being a snapshot in place.
A writer killed between staging and rename leaves `.{16 hex}.metadata.bin.tmp.{…}`,
which begins with the snapshot magic; removing a snapshot whose sidecar removal is
interrupted leaves a `{16 hex}.analysis.bin` with no snapshot.
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

Machine formats carry the `fdu.cache/3` schema — its own document identity, not a report
schema, because cache status is a fact about the cache directory rather than about a
tree. Every row carries `path`, `bytes`, `state`, and `content`; a `current` row adds
`root`, `entries`, and the `identity` of every tier the snapshot holds, a `stale` row a
`stale_reason` and, for a format mismatch, the `format_version`, and a `leftover` row
its `leftover_kind`. A snapshot’s `identity` names its entry tier (the engine
fingerprint, the scope fields, and the type-rules and reducer-set fingerprints) and its
control tier as `ignore_rules`: `null` when no `.gitignore` was read, otherwise the
`limits` it was read under.
The entry `scope` exposes `population` and `control_fingerprint`; the latter is zero for
`include` and binds a narrowed entry tier to its governing control limits.
`content` is `null` when no sidecar with the sidecar magic sits beside the file, and
otherwise its `bytes` and its own `state`: `current`, with its `records` and an
`identity` that adds to the entry tier, which alone carries the type-rules fingerprint,
the analyzer set, the options fingerprint, and the analyzers under the names a report’s
`analysis` object uses, or `stale`, with its `stale_reason` and `format_version`.
Whether the sidecar is grouped with its snapshot and cleared with it is still decided by
its magic, so a stale sidecar is labelled and removed like a current one.
Every fingerprint in either identity is a full 64-bit integer, written as a JSON number,
and a consumer has to parse it exactly: a reader that converts numbers to IEEE doubles —
JavaScript’s `JSON.parse`, jq before 1.7 — rounds values above 2^53, and a rounded
identity field makes two different stores compare equal.
One encoding rule for fingerprints across `fdu.report`, `fdu.stream`, and `fdu.cache` is
decided once, with the answer model’s field schema (`fdu-cggg`), rather than per
document. An empty cache directory is an empty sequence in every machine format, never a
null, so one reader works whether or not anything is cached.

Snapshot persistence is available on every platform, including for metadata queries.
It is not used by every execution plan.
Under the default `auto` policy a one-shot metadata report neither reads nor writes a
snapshot: revalidation stats every entry regardless, so loading one is purely additive
cost, and no later one-shot report reads what it would write.
`--cache on` writes after every complete indexed scan, and an unfiltered
`--view summary`, which the transient tier otherwise answers without an index, builds
the index it writes.
[The policy axis](#the-policy-axis) lists which paths read and write.

## Layer Two: Derived Content Data

Content-derived metrics — line counts, word counts, hashes, and future plugin analyzers
— do **not** belong in the core snapshot.
They live in a separately checksummed analysis file beside the snapshot,
`<key>.analysis.bin`, recording the engine fingerprint, the entry tier identity the
records were analyzed over (the snapshot’s scope, type-rule fingerprint, and reducer
set), the stored analyzer set, an options fingerprint, ordered analyzer IDs and
versions, and the root.
The type-rule fingerprint is recorded once, in the entry tier.
It holds one analyzer set per root.
Each sparse file record carries its classification and a fingerprint of size, mtime,
ctime, inode, and device, so a reconciled metadata change rejects only the stale record.
In memory the content tier holds records of exactly one content tier identity: preparing
it for another identity clears it, and a record produced under another is refused.
The current analysis format is version 8; an absent, corrupt, oversized, foreign, or
version-mismatched sidecar is a clean analyzer-cache miss and never invalidates the core
snapshot.

Four analyzer dialects ship: streaming physical lines and raw words, common-language
SLOC, normalized plain-text volume, and reader-visible Markdown prose.
Keeping them separate from metadata remains load-bearing rather than tidy:

- The core snapshot stays small and fast to open.
  An analyzer’s output can be far larger than the tree’s metadata, and paying for it on
  every open would penalize the common query.
- Content-sidecar invalidation never touches tree truth.
  A sidecar is usable when its format version and root match and its content tier
  identity equals the requested one: the engine fingerprint, the entry tier (whose
  type-rule fingerprint is the registry in use), the analyzer set, the options
  fingerprint, and each analyzer’s ID and version.
  Equality, not containment: a wider sidecar holds metrics a narrower request did not
  ask for, so it misses, and the narrower run reads every file and replaces it.
  A mismatch is a clean content-cache miss and never invalidates metadata sizes.
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

`--cache` takes three values, and `auto` means something different per kind of request,
because whether a store pays depends on whether a later request reads it.
The
[cache economics brief](../research/research-2026-09-27-cache-economics-and-default-plans.md)
has the measurements behind each row.

| Request | `auto` (default) | `on` | `off` |
| --- | --- | --- | --- |
| One-shot metadata report | no read, no write | no read; writes after every complete scan, building an index for a summary that would retain none | no read, no write |
| One-shot report with content analysis | reads and revalidates the snapshot and sidecar; writes both | same as `auto` | no read, no write |
| `open`, the first answer of `--watch`, and a refresh | reads and revalidates; writes | same as `auto` | no read, no write |

`on` changes only what is written, not what is read: loading a snapshot that a full
revalidation then re-stats costs more than a cold walk, and caching never changes an
answer, so there is nothing to gain by forcing a read.

`--stale-ok` (Python `stale_ok=True`, Rust
[`Delivery::stale_ok`](../../../crates/fdu-core/src/query/query_request.rs)) is a
separate choice from the policy, because it changes what the answer promises rather than
what the run stores: it reads the snapshot, never touches the filesystem, and never
writes. It is refused with `--cache off`, which reads nothing, with `--watch`, and by a
refresh.

What each path writes, in more detail:

- **One-shot report.** A metadata-only report under `auto` scans cold and leaves the
  store as it found it.
  A report with analysis takes the warm path, which writes the snapshot only when
  reconciliation changed something.
  The metadata snapshot is written beside the sidecar because the two are one pair in
  the cache lifecycle: status names a sidecar without its snapshot a leftover, and
  `--cache-clear=all` removes it.
- **`open` and the first answer of `--watch`.** Both load a usable snapshot and
  reconcile it; `off` starts cold.
  A retained `open` with `stale_ok` loads without revalidation; a watch refuses it.
  A warm open writes when reconciliation changes something, and a cold open writes after
  a complete scan.
- **Live updates.** Unless the policy is `off`, both command-line and Python watches use
  the engine session’s throttled persistence.
  Python `Index.refresh` applies the same write policy after reconciliation.
  A failed watch save reports the error and remains pending for a later attempt.
- **Content sidecar.** Written after a cold scan with analysis, and after a warm open
  whose analysis applied a record or found a stale one.
  Each tier has its own write rule.
  A partial pass never replaces the metadata snapshot.
  It may write a content sidecar only beside an existing snapshot of the same entry
  identity; the sidecar includes only reusable records whose files this pass verified.
  Failed or unverified records remain misses and are retried on a later request.
  A partial pass under another entry identity preserves the existing snapshot and its
  sidecar. A run with another analyzer set misses the stored sidecar, because the stored
  one is another identity, and replaces it.

A stale answer is the one tier that can be stale, and it says so: its report carries
`source: cache_only` and `freshness: stale`. It fails outright when no usable snapshot
exists rather than quietly scanning, because a fast path that is sometimes a full walk —
with nothing in the output to say which happened — is worse than no fast path.
The failure names `--cache on` as the way to leave one.
`--watch --stale-ok` is refused: starting observation cannot verify the interval between
snapshot capture and observation registration.

Earlier releases spelled the policy `auto`, `refresh`, `read-only`, `only`, or `off`.
`only` is now `--stale-ok`; `refresh`, a cold scan that rewrote the snapshot, and
`read-only`, a read that never wrote, were removed, and each is refused with its
replacement named.

Every machine-format report carries `status` with `complete`, `coverage`, `errors`, and
`errors_omitted`, plus `provenance` with source, freshness, timestamps, and per-tier
truth. Text output names none of them; a partial text run prints its errors as warnings
on standard error.

[`Plan`](../../../crates/fdu-core/src/execution.rs) derives loading, verification, and
write authorization (`Plan::persists`) for every route.
The [executors](../../../crates/fdu-core/src/lib.rs) perform the authorized work;
[`Session`](../../../crates/fdu-core/src/watch_session.rs) owns watch throttling.
`--cache auto` does not promise a reusable baseline after an arbitrary command;
`--cache on` does, after any complete one.
`--allow-partial` changes exit acceptance; it does not make a partial scan’s snapshot
cacheable.

## Future Considerations

### Potential Improvements

- **Stores keyed by identity.** One snapshot and one sidecar per root means a request in
  another scope or analyzer set misses and rescans, then replaces the stored one
  (`fdu-w3l5`), so alternating analyzer sets re-reads every file each time.
  That costs time, never correctness, so keying stores by tier identity, or serving a
  narrower set from a wider one through a projection proven to reproduce the cold
  answer, is a performance improvement rather than a semantic fix.
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
