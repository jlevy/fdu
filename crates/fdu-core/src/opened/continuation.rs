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

#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct ChildPosition {
    pub(super) partition: ChildPartition,
    /// First child name the resumed page should visit.
    pub(super) name: String,
}

#[derive(Clone, Debug)]
pub(super) enum ContinuationKind {
    Tree {
        path: PathBuf,
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
}

impl Default for ContinuationTable {
    fn default() -> Self {
        Self { next: FIRST_CONTINUATION_ORDINAL, records: BTreeMap::new(), order: VecDeque::new() }
    }
}

impl ContinuationTable {
    pub(super) fn insert(
        &mut self,
        session: SessionId,
        record: ContinuationRecord,
    ) -> Result<ContinuationId> {
        let retained_bytes = record.retained_bytes();
        if retained_bytes > crate::MAX_CONTINUATION_RECORD_BYTES {
            return Err(Error::ContinuationRecordLimit {
                attempted: retained_bytes,
                limit: crate::MAX_CONTINUATION_RECORD_BYTES,
            });
        }
        let ordinal = self.next;
        self.next = self.next.checked_add(1).ok_or(Error::ContinuationIdentityExhausted)?;
        if self.records.len() == MAX_CONTINUATIONS {
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
    pub(super) fn restore(&mut self, continuation: ContinuationId, record: ContinuationRecord) {
        debug_assert!(!self.records.contains_key(&continuation.ordinal));
        self.records.insert(continuation.ordinal, record);
        // A repeatedly underfunded token must not become immortal merely because it was
        // retried; keeping it oldest preserves the table's original eviction pressure.
        self.order.push_front(continuation.ordinal);
    }

    pub(super) fn clear(&mut self) {
        self.records.clear();
        self.order.clear();
    }
}

impl ContinuationRecord {
    fn retained_bytes(&self) -> usize {
        let kind = match &self.kind {
            ContinuationKind::Tree { path, next } => {
                path.as_os_str().as_encoded_bytes().len().saturating_add(next.name.len())
            }
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
                        next: ChildPosition {
                            partition: ChildPartition::Directories,
                            name: "next".to_owned(),
                        },
                    },
                },
            )
            .expect("retained record");
        table.clear();
        assert!(matches!(table.take(session, retained), Err(Error::ContinuationUnavailable)));
    }
}
