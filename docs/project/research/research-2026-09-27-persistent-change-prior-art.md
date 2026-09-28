# Persistent Filesystem Changes: Prior Art and Implementation Choices

**Date:** 2026-09-27

**Status:** Prior-art review complete; real-root acceptance remains in progress.

## Scope and Questions

Can fdu base a cross-process, cached disk-growth workflow on existing demonstrations?
Which modern Rust and native APIs preserve the necessary history and failure signals on
macOS and Linux?
This review distinguishes vendor-described production behavior, reviewed
source, and fdu’s local experiments.
It does not benchmark or independently validate third-party products.
Sources are linked alongside the claims they support.

This supplements the
[performance-frontier research](research-2026-08-10-performance-frontier.md) and the
[FSEvents design](../specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md).
It does not enable a new fdu capability.

## What Has Already Been Demonstrated?

The pattern is established: retain an inventory, ask a change source which scopes need
attention, observe those scopes, and fall back to a full scan when continuity is lost.
Our earlier claim that fdu would pioneer this in production was too broad.
The unresolved question is whether our implementation meets fdu’s accounting,
verification, and whole-command latency requirements.

| Evidence | What it establishes | What it does not establish |
| --- | --- | --- |
| August fdu scratch spike, about 60,000 entries | A deep append generated a parent-directory event despite unchanged directory mtimes; dispatch delivery and historical replay worked | Source and exact flags were not retained; no reproducible large-tree refresh result |
| [September fdu probe](../../../explorations/fsevents-replay/README.md) | 48 recorded short-gap cross-process micro-fixture replays matched an independent metadata oracle on two APFS volumes | Hour/day retention, live large-tree correctness, engine parity, or end-to-end savings |
| September 27 real-root and next-day extension | Quiet-root agreement; zero stable mismatches on a live 442k-entry root; 26-hour fixture events still delivered | Large root widened to a full scan with concurrent paths; all sixteen next-day attempts timed out before historical completion |
| Carbon Copy Cloner Quick Update | A shipping macOS backup workflow narrows enumeration using FSEvents history from the previous successful task | Its private cursor, completion, and reconciliation implementation; fdu speed or correctness |
| SuperDuper Turbo Smart Update | Another shipping macOS backup workflow uses FSEvents to reduce visits between compatible jobs | A reusable implementation or a completeness proof |
| Watchman and Git fsmonitor source review | Resident monitoring, invalidation, and recovery mechanics; reviewed restart paths recrawl | Independent one-shot processes resuming an fdu inventory without a resident observer |
| Robinhood with Lustre ChangeLog | Scan-to-database followed by persistent changelog consumption is established at large scale | A portable journal for ordinary Linux desktop filesystems |

The internal observations are recorded in the FSEvents plan’s
[August findings](../specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md#phase-0-spike-findings-2026-08-10-run-on-this-host)
and the linked probe records.
They are mechanism evidence, not accepted production performance experiments.

### macOS Backup Products Are the Closest Workflow Precedent

[CCC’s current documentation](https://support.bombich.com/hc/en-us/articles/20686481162647-Advanced-Settings)
describes limiting source enumeration to folders changed since the previous successful
task. It supports locally attached APFS/HFS+ sources and falls back to an audit for
changed scope/filter settings, unavailable history, or an old successful run.
It also documents periodic audits and a cloud-only-content limitation.
These are useful policy precedents, not evidence for copying CCC’s time thresholds or
assuming its private replay algorithm is correct for fdu.

[SuperDuper’s Turbo Tips](https://www.shirt-pocket.com/blog/turbo_tips) independently
describes FSEvents-guided visits, job-identity invalidation, and full scans when history
is unavailable. Neither vendor document proves the exact no-client-process interval or
barrier protocol our spike tests; both establish that history-assisted scoped
enumeration is a production technique.

Apple explicitly documents
[persistent event IDs and volume identity](https://developer.apple.com/library/archive/documentation/Darwin/Conceptual/FSEvents_ProgGuide/UsingtheFSEventsFramework/UsingtheFSEventsFramework.html).
fdu need not remain running for the operating system to retain history.
Availability is conditional, and the log nominates observations rather than supplying
past file sizes or a complete accounting ledger.

### Watchman Supplies Both a Model and a Warning

Use Watchman’s clocks and fresh-instance handling as API precedent: an invalid cursor
must cause explicit recrawl/invalidation, not an empty successful diff.
[Jujutsu’s optional Watchman integration](https://docs.jj-vcs.dev/latest/config/#filesystem-monitor)
shows a modern Rust tool using a resident service to avoid repeated working-copy scans.
That is a different deployment model from one-shot native replay.

Watchman documents a critical
[macOS synchronization limitation](https://facebook.github.io/watchman/docs/cookies#limitation-macos-fsevents):
under load it has received earlier changes after both its cookie notification and
`FSEventStreamFlushSync` return.
A settle interval is a heuristic, not a completeness proof.
`HistoryDone` concerns historical delivery, not a linearizable current-state barrier.
The spike must preserve this distinction even when its oracle matches.
Do not insert cookie files into user roots for a read-only diagnostic.

## Rust and Native API Choices

Keep the existing portable live watcher and a dedicated history adapter separate.
The source review of notify 8.2.0 and 9.0.0-rc.4 found no public replay cursor or
historical-completion contract; debounce and rename pairing do not add one.
Preserve `need_rescan()` and fail closed on watcher gaps.

The [notify release notes](https://github.com/notify-rs/notify/releases) matter to the
binding decision: the 9.0 RC line moves to objc2, removes the journal-purge call from
the older macOS shutdown path, and includes path/panic and Linux watcher fixes.
It also raises the Rust floor to 1.88, above fdu’s declared 1.85; this research does not
authorize an upgrade or a supply-chain exception.

[fsevent-sys 5.2.0](https://docs.rs/fsevent-sys/5.2.0/fsevent_sys/) is deprecated in
favor of `objc2-core-services`. Re-evaluate the earlier plan to extend locked
`fsevent-sys 4.1.0` before Rust integration.
Prefer maintained generated bindings if their audited dependency, MSRV, and
build-feature costs fit; retaining the locked binding plus narrow declarations is a
measured fallback, not a settled best practice.
The standalone C probe uses the installed SDK and changes no Rust dependency.

For Spotlight, generated
[`MDQuery`](https://docs.rs/objc2-core-services/latest/x86_64-apple-darwin/objc2_core_services/struct.MDQuery.html)
and
[`NSMetadataQuery`](https://docs.rs/objc2-foundation/latest/objc2_foundation/struct.NSMetadataQuery.html)
bindings exist.
[Core Spotlight](https://developer.apple.com/documentation/corespotlight)
indexes app-provided content and is not the whole-filesystem query API. Binding
availability does not fix selective indexing, latency, or deleted-path coverage.
Spotlight remains an optional discovery hint, never the disk-usage oracle.

## Linux: Same Workflow, Different Continuity Source

[inotify](https://man7.org/linux/man-pages/man7/inotify.7.html) is a live,
descriptor-owned queue: closing the instance removes its watches.
Recursive coverage needs watch setup and reconciliation, and queue overflow invalidates
continuity. [fanotify](https://man7.org/linux/man-pages/man7/fanotify.7.html) offers
broader marks with permission and filesystem constraints, but does not provide a durable
offline replay cursor either.
Neither makes an ordinary ext4/XFS checkpoint refresh incremental through a period when
no observer was running.

A resident observer with its own durable log is one option; a gap requires a scan.
Without that observer, retained baselines still support the same comparison workflow
using fdu’s fast full scan.
Optional btrfs/ZFS snapshot differences or Lustre ChangeLog adapters can have separate
capabilities and gates; avoid the blanket claim that Linux can never support persistent
changes.
[Robinhood’s setup](https://github.com/cea-hpc/robinhood) explicitly starts with
a scan to initialize its database and then consumes registered Lustre changelogs.
Its [authors’ paper](https://arxiv.org/abs/1505.01448) is further architectural prior
art.

Desktop indexes are not substitutes for complete enumeration:
[GNOME LocalSearch](https://gnome.pages.gitlab.gnome.org/localsearch/indexed-data.html)
and [KDE Baloo](https://api.kde.org/baloo-indexerconfig.html) have configurable coverage
and exclusions. The portable contract must report coverage independently of how it
obtained candidate paths.

## What to Validate Next

The
[September 27 observations](../../../explorations/fsevents-replay/README.md#real-roots-and-next-day-replay-2026-09-27)
make two immediate priorities concrete: implement relisting without unnecessary
recursive scope inflation, and establish a bounded historical-completion protocol.
The real-root spike does not yet validate fast daily refresh.

Use arbitrary real roots, with metadata-only reads and state outside the measured root.
Save a pre-scan device fence, exit, then refresh in a fresh process after natural
activity. Persist the candidate independently of the full-scan oracle.
Report stable mismatches separately from concurrent mutations, actual entries visited,
scope widening, and load/replay/reconcile/roll-up/save costs.
A full-root fallback that matches the oracle proves correctness of that observation, not
incremental savings.

Retain the original baseline for repeat tests until safe cursor advancement is
established. Short-gap success cannot substitute for hour/day, reboot, retention-loss,
permission, and hard-link tests.
Python probe timings are not shipped Rust engine benchmarks.
These gates remain in the FSEvents plan; folder suggestions and workflow defaults belong
in the brief skill described by the
[checkpoint plan](../specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
