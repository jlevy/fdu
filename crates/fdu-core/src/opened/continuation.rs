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
        selection: crate::query::Selection,
        shape: crate::RowShape,
        /// First complete portable path the resumed page should visit.
        next: String,
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
