//! Blocking opened-root polling over the index's one exact commit history.

use std::collections::BTreeSet;
use std::sync::{Condvar, Mutex};
use std::time::Instant;

use super::OpenedIndex;
use crate::{
    ChangeOutcome, ChangePoll, ChangeRequest, EngineVersion, Error, Impact, ImpactDomain,
    IndexState, Result, Since, Work,
};

pub(super) struct JournalWait {
    state: Mutex<WaitState>,
    changed: Condvar,
}

#[derive(Default)]
struct WaitState {
    closed: bool,
}

impl JournalWait {
    pub(super) fn new() -> Self {
        Self { state: Mutex::new(WaitState::default()), changed: Condvar::new() }
    }

    /// Wake every poller after an exact commit has entered the index journal.
    pub(super) fn notify_commit(&self) {
        let guard = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        self.changed.notify_all();
        drop(guard);
    }

    /// Wake every poller permanently during joined owner shutdown.
    pub(super) fn close(&self) {
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        state.closed = true;
        self.changed.notify_all();
    }
}

struct JournalSnapshot {
    version: EngineVersion,
    state: IndexState,
    since: Since,
}

pub(super) fn poll(opened: &OpenedIndex, request: ChangeRequest) -> Result<ChangePoll> {
    let started = Instant::now();
    let mut wait = opened.state.journal.state.lock().map_err(|_| Error::OpenedJournalPoisoned)?;

    loop {
        if wait.closed || opened.state.cancellation.is_cancelled() {
            return Err(Error::OpenedIndexClosed);
        }

        let snapshot = opened.state.index.read_with(|index| {
            let scope = index.scope();
            let version = EngineVersion {
                session: opened.state.session,
                sequence: index.clock(),
                scope: scope.scope_identity(),
                semantics: scope.semantic_identity(),
            };
            validate_cursor(request.after, version)?;
            let since = index.since(request.after.sequence);
            debug_assert_eq!(since.clock, version.sequence);
            debug_assert_eq!(since.state, index.state());
            Ok(JournalSnapshot { version, state: since.state, since })
        })??;

        if opened.state.cancellation.is_cancelled() {
            return Err(Error::OpenedIndexClosed);
        }

        if snapshot.since.truncated {
            return Ok(reset_at(&snapshot));
        }
        if !snapshot.since.commits.is_empty() {
            return Ok(changes_at(snapshot));
        }

        let remaining = request.timeout.saturating_sub(started.elapsed());
        if remaining.is_zero() {
            return Ok(idle_at(request.after, &snapshot));
        }
        #[cfg(test)]
        opened.state.test_controls.reach(super::TestPoint::BeforeJournalWait);
        let (next, _) = opened
            .state
            .journal
            .changed
            .wait_timeout(wait, remaining)
            .map_err(|_| Error::OpenedJournalPoisoned)?;
        wait = next;
    }
}

fn validate_cursor(requested: EngineVersion, current: EngineVersion) -> Result<()> {
    if requested.session != current.session
        || requested.scope != current.scope
        || requested.semantics != current.semantics
        || requested.sequence > current.sequence
    {
        return Err(Error::ChangeCursorUnavailable {
            requested: Box::new(requested),
            current: Box::new(current),
        });
    }
    Ok(())
}

fn changes_at(snapshot: JournalSnapshot) -> ChangePoll {
    let count = u64::try_from(snapshot.since.commits.len()).unwrap_or(u64::MAX);
    let impact = combined_impact(&snapshot.since.commits);
    ChangePoll {
        cursor: snapshot.version,
        version: snapshot.version,
        state: snapshot.state,
        outcome: ChangeOutcome::Changes { commits: snapshot.since.commits, impact },
        work: Work { commits_visited: count, commits_returned: count, ..Work::default() },
    }
}

fn idle_at(after: EngineVersion, snapshot: &JournalSnapshot) -> ChangePoll {
    debug_assert_eq!(after.sequence, snapshot.version.sequence);
    ChangePoll {
        cursor: after,
        version: snapshot.version,
        state: snapshot.state,
        outcome: ChangeOutcome::Idle,
        work: Work::default(),
    }
}

fn reset_at(snapshot: &JournalSnapshot) -> ChangePoll {
    let visited = u64::try_from(snapshot.since.commits.len()).unwrap_or(u64::MAX);
    ChangePoll {
        cursor: snapshot.version,
        version: snapshot.version,
        state: snapshot.state,
        outcome: ChangeOutcome::Reset {
            impact: Impact {
                domains: vec![
                    ImpactDomain::Topology,
                    ImpactDomain::Metadata,
                    ImpactDomain::Classification,
                    ImpactDomain::Aggregates,
                    ImpactDomain::Content,
                    ImpactDomain::State,
                ],
                dirty_paths: Vec::new(),
                all_dirty: true,
            },
        },
        work: Work { commits_visited: visited, ..Work::default() },
    }
}

fn combined_impact(commits: &[crate::Commit]) -> Impact {
    let mut domains = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut all_dirty = false;
    for commit in commits {
        domains.extend(commit.impact.domains.iter().copied());
        if all_dirty || commit.impact.all_dirty {
            all_dirty = true;
            paths.clear();
            continue;
        }
        for path in &commit.impact.dirty_paths {
            paths.insert(path.clone());
            if paths.len() > crate::MAX_DIRTY_PATHS {
                all_dirty = true;
                paths.clear();
                break;
            }
        }
    }
    Impact {
        domains: domains.into_iter().collect(),
        dirty_paths: paths.into_iter().collect(),
        all_dirty,
    }
}
