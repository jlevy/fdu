---
type: is
id: is-01kzz2bak4xbxszt2na2yhmeq0
title: FSEvents plan documents a stream shape Apple does not recommend for persistent replay
kind: task
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01kzz29dspd7bsy6jk98mpb9z3
created_at: 2026-08-14T02:41:49.156Z
updated_at: 2026-08-14T03:01:49.948Z
closed_at: 2026-08-14T03:01:49.947Z
close_reason: "Corrected the still-active FSEvents plan on the three points PR #4 identified: replay is a device-relative stream (FSEventStreamCreateRelativeToDevice, with the FFI table updated to declare it) rather than a per-path FSEventStreamCreate; the design requests FileEvents so soundness comes from normalizing item events into parent relists and subtree invalidations rather than from an assumed directory granularity; and the unsupported 'size-independent tens of milliseconds' claim is retracted to a hypothesis. Also fixed the boundary semantics: the fence must be taken before the scan and persisted only as far as applied, because capturing the current event ID at save time silently drops changes made during the walk."
---
docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md is still active and unimplemented, so it is the document someone will build from. PR #4 corrected three things in it that main still has wrong:

1. It plans replay via FSEventStreamCreate(sinceWhen:). For software that persists a cursor across launches Apple recommends a device-relative stream, FSEventStreamCreateRelativeToDevice: event IDs share a system-wide sequence but the stream, its UUID, and its retained history are per-volume. The plan also needs the initial fence, the HistoryDone boundary, and persistence of the applied boundary rather than the observed one.

2. It asserts FSEvents callbacks are directory-granular and builds the subtree-skipping soundness argument on that. The design requests FileEvents, so callbacks name items. Soundness has to come from normalizing item events into parent relists and subtree invalidations, with ambiguous flags falling back to a full sweep, plus periodic full sweeps because Apple documents the event list as advisory.

3. It states replay turns 60k-entry revalidation into 'size-independent tens of milliseconds'. No experiment supports that; it is a hypothesis until the corrected state machine is implemented and measured.

Port PR #4's corrections to the plan text only. No engine code is in scope.
