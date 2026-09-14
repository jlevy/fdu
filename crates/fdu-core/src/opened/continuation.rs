//! Bounded handle-local page traversal state.

use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;

use crate::{ContinuationId, EngineVersion, Error, Result, SessionId};

/// Maximum resumable page positions retained by one opened root.
pub(super) const MAX_CONTINUATIONS: usize = 128;
/// First nonzero handle-local continuation ordinal.
const FIRST_CONTINUATION_ORDINAL: u64 = 1;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum ChildPartition {
    Directories,
    Nondirectories,
}

/// Where a breadth-first tree page stopped.
///
/// One frame, not a stack of them. Enumerating level *d* requires walking the directories
/// of level *d-1* in order, which sounds like it needs a frame per level — but the
/// ancestor chain of `parent` already *is* that stack, and it is derivable from the path
/// by splitting it. Storing the path stores the whole position, and advancing to the next
/// directory at this depth costs one child-map lookup per ancestor.
///
/// What must not be stored is the frontier: the set of directories one level up is
/// unbounded in directory width, so it cannot fit a bounded record, and re-deriving it per
/// page would make paging one wide level quadratic in the number of pages.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct ChildPosition {
    /// Native path of the directory whose children were being emitted.
    pub(super) parent: PathBuf,
    /// Levels between the requested root and `parent`; zero is the root itself.
    pub(super) depth: u32,
    pub(super) partition: ChildPartition,
    /// First child name the resumed page should visit within `partition`.
    ///
    /// `None` resumes at the start of the partition. That is not the same as naming the
    /// first child: a page can stop having just arrived at a parent none of whose
    /// children are emitted yet, and there is no name to point at until one is read.
    pub(super) name: Option<String>,
    /// First directory one level below `depth`, noticed while this level was emitted.
    ///
    /// Descending used to search for it: ask every parent at this depth for a directory
    /// child until one answers. That is a scan of the whole level, and on a level of
    /// leaves — the shape of every last level — it scans everything to conclude there is
    /// nothing below.
    ///
    /// The walk already passes every one of those parents while emitting, so the answer
    /// is free if it is noticed in passing. The first directory *row* emitted at the next
    /// level is that level's first directory, because level order there is grouped by
    /// parent order here. `None` at the end of a level means there is no level below.
    ///
    /// It is carried in the cursor because a page can stop mid-level: without it, a
    /// resumed page would notice only what follows where it resumed and take some later
    /// directory for the first.
    pub(super) next_level_first: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub(super) enum ContinuationKind {
    Tree {
        path: PathBuf,
        depth: crate::query::Bound,
        include_ignored: bool,
        next: ChildPosition,
    },
    Flat {
        selection: Box<crate::query::EntrySelection>,
        shape: crate::RowShape,
        /// First complete portable path the resumed page should visit.
        next: crate::PortablePath,
    },
}

#[derive(Clone, Debug)]
pub(super) struct ContinuationRecord {
    pub(super) version: EngineVersion,
    pub(super) kind: ContinuationKind,
}

#[derive(Debug)]
pub(super) struct ContinuationTable {
    next: u64,
    records: BTreeMap<u64, ContinuationRecord>,
    order: VecDeque<u64>,
    /// Set once by shutdown. Reads do not hold the lifecycle lock while they project, so
    /// one that passed the open check can reach this table after shutdown emptied it.
    closed: bool,
}

impl Default for ContinuationTable {
    fn default() -> Self {
        Self {
            next: FIRST_CONTINUATION_ORDINAL,
            records: BTreeMap::new(),
            order: VecDeque::new(),
            closed: false,
        }
    }
}

impl ContinuationTable {
    pub(super) fn insert(
        &mut self,
        session: SessionId,
        record: ContinuationRecord,
    ) -> Result<ContinuationId> {
        if self.closed {
            return Err(Error::OpenedIndexClosed);
        }
        let retained_bytes = record.retained_bytes();
        if retained_bytes > crate::MAX_CONTINUATION_RECORD_BYTES {
            return Err(Error::ContinuationRecordLimit {
                attempted: retained_bytes,
                limit: crate::MAX_CONTINUATION_RECORD_BYTES,
            });
        }
        let ordinal = self.next;
        self.next = self.next.checked_add(1).ok_or(Error::ContinuationIdentityExhausted)?;
        // At or above the bound, not exactly at it: a check for equality stops firing for
        // good the first time anything overshoots, and then the table grows by one record
        // per page with nothing left to bound it.
        while self.records.len() >= MAX_CONTINUATIONS {
            let evicted = self.order.pop_front().expect("a full table has an oldest record");
            self.records.remove(&evicted);
        }
        self.records.insert(ordinal, record);
        self.order.push_back(ordinal);
        Ok(ContinuationId { session, ordinal })
    }

    pub(super) fn take(
        &mut self,
        session: SessionId,
        continuation: ContinuationId,
    ) -> Result<ContinuationRecord> {
        if continuation.session != session {
            return Err(Error::ContinuationUnavailable);
        }
        let Some(record) = self.records.remove(&continuation.ordinal) else {
            return Err(Error::ContinuationUnavailable);
        };
        self.order.retain(|ordinal| *ordinal != continuation.ordinal);
        Ok(record)
    }

    /// Restore a consumed continuation after a bounded projection returns no page.
    ///
    /// The restored record goes back as the oldest, and eviction takes the oldest first, so
    /// when other pages have filled the table since it was taken, restoring it would evict
    /// exactly this record. It is dropped instead, as an eviction, and the token reads as
    /// unavailable -- never as a record past the table's bound.
    pub(super) fn restore(&mut self, continuation: ContinuationId, record: ContinuationRecord) {
        debug_assert!(!self.records.contains_key(&continuation.ordinal));
        if self.closed || self.records.len() >= MAX_CONTINUATIONS {
            return;
        }
        self.records.insert(continuation.ordinal, record);
        // A repeatedly underfunded token must not become immortal merely because it was
        // retried; keeping it oldest preserves the table's original eviction pressure.
        self.order.push_front(continuation.ordinal);
    }

    /// Drop every record and refuse new ones, as part of shutdown.
    pub(super) fn close(&mut self) {
        self.closed = true;
        self.records.clear();
        self.order.clear();
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        debug_assert_eq!(self.records.len(), self.order.len());
        self.records.len()
    }
}

impl ContinuationRecord {
    fn retained_bytes(&self) -> usize {
        let kind = match &self.kind {
            ContinuationKind::Tree { path, next, .. } => path
                .as_os_str()
                .as_encoded_bytes()
                .len()
                .saturating_add(next.parent.as_os_str().as_encoded_bytes().len())
                .saturating_add(next.name.as_deref().map_or(0, str::len))
                .saturating_add(
                    next.next_level_first
                        .as_ref()
                        .map_or(0, |path| path.as_os_str().as_encoded_bytes().len()),
                ),
            ContinuationKind::Flat { selection, next, .. } => {
                selection.retained_heap_bytes().saturating_add(next.retained_heap_bytes())
            }
        };
        std::mem::size_of::<Self>().saturating_add(kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(session: SessionId) -> EngineVersion {
        EngineVersion {
            session,
            sequence: crate::Clock::ZERO,
            scope: crate::ScopeIdentity {
                max_depth: None,
                follow_symlinks: false,
                one_filesystem: false,
                hidden_fingerprint: 0,
                exclude_special: false,
            },
            semantics: crate::SemanticIdentity {
                ignore_rules_fingerprint: 0,
                type_rules_fingerprint: 0,
                reducers_fingerprint: 0,
            },
        }
    }

    #[test]
    fn oversized_record_is_rejected_before_identity_or_eviction_changes() {
        let session = SessionId::from_opaque(1).expect("nonzero session");
        let mut table = ContinuationTable::default();
        let first = table
            .insert(
                session,
                ContinuationRecord {
                    version: version(session),
                    kind: ContinuationKind::Flat {
                        selection: Box::new(crate::query::EntrySelection::default()),
                        shape: crate::RowShape::Compact,
                        next: crate::PortablePath::new("first".to_owned()),
                    },
                },
            )
            .expect("small record");

        let error = table
            .insert(
                session,
                ContinuationRecord {
                    version: version(session),
                    kind: ContinuationKind::Flat {
                        selection: Box::new(crate::query::EntrySelection::default()),
                        shape: crate::RowShape::Compact,
                        next: crate::PortablePath::new(
                            "x".repeat(crate::MAX_CONTINUATION_RECORD_BYTES),
                        ),
                    },
                },
            )
            .expect_err("oversized record");
        assert!(matches!(
            error,
            Error::ContinuationRecordLimit { attempted, limit }
                if attempted > limit && limit == crate::MAX_CONTINUATION_RECORD_BYTES
        ));

        let expanded =
            crate::query::Pattern::parse(&"{a,b}".repeat(10)).expect("bounded pattern expansion");
        let query_error = table
            .insert(
                session,
                ContinuationRecord {
                    version: version(session),
                    kind: ContinuationKind::Flat {
                        selection: Box::new(crate::query::EntrySelection {
                            query: crate::query::Selection {
                                include: vec![expanded],
                                ..crate::query::Selection::default()
                            },
                            ..crate::query::EntrySelection::default()
                        }),
                        shape: crate::RowShape::Compact,
                        next: crate::PortablePath::new("next".to_owned()),
                    },
                },
            )
            .expect_err("expanded query record");
        assert!(matches!(query_error, Error::ContinuationRecordLimit { .. }));
        assert_eq!(table.records.len(), 1);
        assert_eq!(table.next, first.ordinal + 1);
        table.take(session, first).expect("existing record was not evicted");

        let retained = table
            .insert(
                session,
                ContinuationRecord {
                    version: version(session),
                    kind: ContinuationKind::Tree {
                        path: PathBuf::new(),
                        depth: crate::query::Bound::Limit(1),
                        include_ignored: true,
                        next: ChildPosition {
                            parent: PathBuf::new(),
                            depth: 0,
                            partition: ChildPartition::Directories,
                            name: Some("next".to_owned()),
                            next_level_first: None,
                        },
                    },
                },
            )
            .expect("retained record");
        table.close();
        assert!(matches!(table.take(session, retained), Err(Error::ContinuationUnavailable)));
    }

    fn flat_record(session: SessionId, next: &str) -> ContinuationRecord {
        ContinuationRecord {
            version: version(session),
            kind: ContinuationKind::Flat {
                selection: Box::new(crate::query::EntrySelection::default()),
                shape: crate::RowShape::Compact,
                next: crate::PortablePath::new(next.to_owned()),
            },
        }
    }

    /// Take, insert, restore: the interleaving of two pages that once overshot the bound.
    ///
    /// One page takes its record from a full table, another page's insert fills the table
    /// again without evicting, and the first page then runs out of budget and restores what
    /// it took. Eviction fired only at exactly the bound, so after that overshoot it never
    /// fired again and the table grew by one record per page.
    #[test]
    fn the_table_never_exceeds_its_bound_across_take_insert_and_restore() {
        let session = SessionId::from_opaque(1).expect("nonzero session");
        let mut table = ContinuationTable::default();
        let ids = (0..MAX_CONTINUATIONS)
            .map(|position| {
                table.insert(session, flat_record(session, &format!("r{position}"))).expect("fill")
            })
            .collect::<Vec<_>>();
        assert_eq!(table.len(), MAX_CONTINUATIONS);

        let taken_id = ids[MAX_CONTINUATIONS / 2];
        let taken = table.take(session, taken_id).expect("take");
        table.insert(session, flat_record(session, "other page")).expect("insert while taken");
        table.restore(taken_id, taken);
        assert_eq!(table.len(), MAX_CONTINUATIONS, "a restore may not overshoot the bound");
        for position in 0..16 {
            table.insert(session, flat_record(session, &format!("later{position}"))).expect("page");
            assert_eq!(table.len(), MAX_CONTINUATIONS);
        }
        // A restored record is the oldest by design, so it is the one a full table gives up.
        assert!(matches!(table.take(session, taken_id), Err(Error::ContinuationUnavailable)));
    }

    #[test]
    fn a_restored_record_is_retryable_while_the_table_has_room() {
        let session = SessionId::from_opaque(1).expect("nonzero session");
        let mut table = ContinuationTable::default();
        let id = table.insert(session, flat_record(session, "next")).expect("insert");
        let record = table.take(session, id).expect("take");
        table.restore(id, record);
        assert_eq!(table.len(), 1);
        table.take(session, id).expect("restored record is retryable");
    }

    /// A read no longer holds the lifecycle lock while it projects, so a page can finish
    /// after shutdown emptied the table. Its record must not outlive the root.
    #[test]
    fn a_closed_table_refuses_records_that_race_shutdown() {
        let session = SessionId::from_opaque(1).expect("nonzero session");
        let mut table = ContinuationTable::default();
        let id = table.insert(session, flat_record(session, "before close")).expect("insert");
        let record = table.take(session, id).expect("take");
        table.close();

        assert!(matches!(
            table.insert(session, flat_record(session, "after close")),
            Err(Error::OpenedIndexClosed)
        ));
        table.restore(id, record);
        assert_eq!(table.len(), 0);
        assert!(matches!(table.take(session, id), Err(Error::ContinuationUnavailable)));
    }
}
