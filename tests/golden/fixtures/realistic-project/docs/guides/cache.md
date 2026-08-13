# Cache guide

The cache records the canonical root, scan scope, filesystem fingerprint, and every
entry required to answer a report. A normal open validates that identity before using
the snapshot. A mismatch is never silently accepted as fresh state.

Use cache-off mode when measuring cold traversal. Use cache-only mode when inspecting a
known snapshot without touching the tree. Revalidation compares the live filesystem to
the retained index and writes back only after a complete pass.

Snapshots are replace-on-success artifacts. An interrupted write leaves the last good
snapshot intact, and an incomplete scan is reported rather than promoted.
