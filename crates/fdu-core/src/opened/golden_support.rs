//! Deterministic transparent-box support for opened-root session goldens.
//!
//! The recorder consumes production requests, responses, commits, and errors. It owns
//! only stable presentation and contract-coverage bookkeeping; it cannot publish facts
//! or manufacture operation results.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::path::{Path, PathBuf};

use crate::{
    ChangeOutcome, ChangePoll, Commit, Coverage, CoverageReason, EffectiveChange, EntryKind, Error,
    Freshness, IndexState, Knowledge, LifecyclePhase, ProjectionResult, ReadProjection,
    ReadRequest, ReadResponse, RefreshResult, RowShape, SessionId, StateTransition,
};

use super::OpenedIndex;

const UPDATE_ENV: &str = "FDU_UPDATE_OPENED_ROOT_GOLDEN";

/// Complete stable trace for one canonical opened-root session.
pub(super) struct SessionTrace {
    name: &'static str,
    aliases: BTreeMap<String, &'static str>,
    sessions: Vec<(String, String)>,
    lines: Vec<String>,
    coverage: ContractCoverage,
    model: SessionModel,
}

impl SessionTrace {
    pub(super) fn new(name: &'static str, root: &Path) -> Self {
        let mut aliases = BTreeMap::new();
        insert_path_alias(&mut aliases, root, "$ROOT");
        if let Ok(canonical) = std::fs::canonicalize(root) {
            insert_path_alias(&mut aliases, &canonical, "$ROOT");
        }
        Self {
            name,
            aliases,
            sessions: Vec::new(),
            lines: vec![format!("scenario: schema=1 name={name}")],
            coverage: ContractCoverage::default(),
            model: SessionModel::new(root),
        }
    }

    pub(super) fn alias_path(&mut self, path: &Path, alias: &'static str) {
        insert_path_alias(&mut self.aliases, path, alias);
        if let Ok(canonical) = std::fs::canonicalize(path) {
            insert_path_alias(&mut self.aliases, &canonical, alias);
        }
    }

    pub(super) fn bind_session(&mut self, session: SessionId) {
        let rendered = session.0.to_string();
        if self.sessions.iter().any(|(known, _)| known == &rendered) {
            return;
        }
        let alias = format!("session-{}", self.sessions.len() + 1);
        self.sessions.push((rendered, alias));
    }

    pub(super) fn record(&mut self, kind: &str, value: &impl Debug) {
        self.lines.push(format!("{kind}: {}", self.normalize(format!("{value:?}"))));
    }

    pub(super) fn record_text(&mut self, kind: &str, value: impl AsRef<str>) {
        self.lines.push(format!("{kind}: {}", self.normalize(value.as_ref().to_owned())));
    }

    pub(super) fn observe_state(&mut self, state: IndexState) {
        self.coverage.observe_state(state);
    }

    pub(super) fn observe_read(&mut self, result: &crate::Result<ReadResponse>) {
        self.coverage.observe_read(result);
    }

    pub(super) fn observe_poll(&mut self, result: &crate::Result<ChangePoll>) {
        self.coverage.observe_poll(result);
    }

    pub(super) fn verify_poll(&mut self, opened: &OpenedIndex, result: &crate::Result<ChangePoll>) {
        if let Ok(poll) = result {
            self.model.observe_poll(opened, poll);
        }
    }

    pub(super) fn observe_refresh(&mut self, result: &crate::Result<RefreshResult>) {
        self.coverage.observe_refresh(result);
    }

    pub(super) fn observe_priority(&mut self, result: &crate::Result<()>) {
        self.coverage.key(if result.is_ok() { "prioritize.ok" } else { "prioritize.error" });
        if let Err(error) = result {
            self.coverage.observe_error(error);
        }
    }

    pub(super) fn observe_close(&mut self, result: &crate::Result<()>) {
        self.coverage.key(if result.is_ok() { "close.ok" } else { "close.error" });
        if let Err(error) = result {
            self.coverage.observe_error(error);
        }
    }

    pub(super) fn coverage(&self) -> &ContractCoverage {
        &self.coverage
    }

    pub(super) fn finish(self) -> String {
        let mut output = self.lines.join("\n");
        output.push('\n');
        output
    }

    pub(super) fn assert_golden(self) {
        let name = self.name;
        let actual = self.finish();
        let path = golden_path(name);
        match std::env::var(UPDATE_ENV) {
            Ok(requested) if requested == name => {
                std::fs::create_dir_all(path.parent().expect("golden parent"))
                    .expect("create opened-root golden directory");
                std::fs::write(&path, actual).expect("write named opened-root golden");
            }
            _ => {
                let expected = std::fs::read_to_string(&path).unwrap_or_else(|error| {
                    panic!(
                        "read opened-root golden {}: {error}; update only this scenario with \
                         `make opened-root-golden-update SCENARIO={name}`",
                        path.display()
                    )
                });
                assert_eq!(actual, expected, "opened-root session golden {name}");
            }
        }
    }

    fn normalize(&self, mut value: String) -> String {
        let mut aliases: Vec<_> = self.aliases.iter().collect();
        aliases.sort_by_key(|(path, _)| std::cmp::Reverse(path.len()));
        for (path, alias) in aliases {
            value = value.replace(path, alias);
        }
        value = value.replace("/private$ROOT", "$ROOT");
        value = normalize_debug_path_separators(value, std::path::MAIN_SEPARATOR);
        value = replace_system_times(value);
        for (session, alias) in &self.sessions {
            value = value.replace(session, alias);
        }
        for (field, replacement) in [
            ("allocated", "[ALLOCATED]"),
            ("mtime_ns", "[TIME]"),
            ("ctime_ns", "[TIME]"),
            ("inode", "[INODE]"),
            ("dev", "[DEVICE]"),
        ] {
            value = replace_integer_field(value, field, replacement);
        }
        value = replace_integer_after(value, "newest_mtime_ns: Some(", "[TIME]");
        value = replace_integer_after(value, "kind: Dir, attrs: Attrs { size: ", "[DIR_SIZE]");
        value
    }
}

fn insert_path_alias(
    aliases: &mut BTreeMap<String, &'static str>,
    path: &Path,
    alias: &'static str,
) {
    let rendered = path.display().to_string();
    aliases.insert(rendered.clone(), alias);
    aliases.insert(debug_string_contents(&rendered), alias);
}

fn debug_string_contents(value: &str) -> String {
    let quoted = format!("{value:?}");
    quoted[1..quoted.len() - 1].to_owned()
}

fn normalize_debug_path_separators(value: String, separator: char) -> String {
    if separator == '\\' { value.replace("\\\\", "/") } else { value }
}

/// `SystemTime`'s derived debug shape is an implementation detail of each platform.
fn replace_system_times(mut source: String) -> String {
    const PREFIX: &str = "SystemTime {";
    const REPLACEMENT: &str = "[SYSTEM_TIME]";
    let mut offset = 0;
    while let Some(found) = source[offset..].find(PREFIX) {
        let start = offset + found;
        let Some(relative_end) = source[start + PREFIX.len()..].find('}') else {
            break;
        };
        let end = start + PREFIX.len() + relative_end + 1;
        source.replace_range(start..end, REPLACEMENT);
        offset = start + REPLACEMENT.len();
    }
    source
}

#[cfg(test)]
mod normalization_tests {
    use super::*;

    #[test]
    fn path_aliases_cover_native_and_debug_escaped_spellings() {
        let path = Path::new(r"\\?\C:\Temp\root");
        let rendered = path.display().to_string();
        let mut aliases = BTreeMap::new();

        insert_path_alias(&mut aliases, path, "$ROOT");

        assert_eq!(aliases.get(&rendered), Some(&"$ROOT"));
        assert_eq!(aliases.get(&debug_string_contents(&rendered)), Some(&"$ROOT"));
    }

    #[test]
    fn windows_debug_path_separators_become_portable_once() {
        let rendered = r#"Commit { path: "target\\leaf.txt" }"#.to_owned();

        assert_eq!(
            normalize_debug_path_separators(rendered, '\\'),
            r#"Commit { path: "target/leaf.txt" }"#
        );
    }

    #[test]
    fn system_time_debug_shapes_share_one_closed_token() {
        for rendered in [
            "generated_at: SystemTime { tv_sec: 0, tv_nsec: 0 }",
            "generated_at: SystemTime { intervals: 116444736000000000 }",
        ] {
            assert_eq!(replace_system_times(rendered.to_owned()), "generated_at: [SYSTEM_TIME]");
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct ModelFact {
    kind: EntryKind,
    size: u64,
}

/// Recomputing oracle that owns no engine reducers, mutation helpers, or retained rows.
struct SessionModel {
    root: PathBuf,
    facts: BTreeMap<PathBuf, ModelFact>,
    freshness: BTreeMap<PathBuf, Freshness>,
    state: IndexState,
    clock: crate::Clock,
}

impl SessionModel {
    fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            facts: BTreeMap::new(),
            freshness: BTreeMap::new(),
            state: IndexState::default(),
            clock: crate::Clock::ZERO,
        }
    }

    fn observe_poll(&mut self, opened: &OpenedIndex, poll: &ChangePoll) {
        match &poll.outcome {
            ChangeOutcome::Changes { commits, .. } => {
                for commit in commits {
                    assert!(commit.clock > self.clock, "session commits must advance exactly once");
                    self.apply_commit(commit);
                    self.clock = commit.clock;
                }
                assert_eq!(self.clock, poll.version.sequence, "poll cursor skips a commit");
                assert_eq!(self.state, poll.state, "modeled and published state diverged");
            }
            ChangeOutcome::Idle => {
                assert_eq!(self.clock, poll.version.sequence, "idle poll advanced the model clock");
                assert_eq!(self.state, poll.state, "idle poll changed modeled state");
            }
            ChangeOutcome::Reset { .. } => {
                self.facts = scan_fixture(&self.root);
                self.freshness.clear();
                self.freshness.insert(PathBuf::new(), poll.state.freshness);
                self.clock = poll.version.sequence;
                self.state = poll.state;
            }
        }
        self.assert_public_projection(opened);
    }

    fn apply_commit(&mut self, commit: &Commit) {
        for change in &commit.changes {
            match change {
                EffectiveChange::Inserted { path, kind, attrs } => {
                    assert!(
                        self.facts
                            .insert(path.clone(), ModelFact { kind: *kind, size: attrs.size })
                            .is_none(),
                        "insert replaced an existing modeled fact at {}",
                        path.display()
                    );
                }
                EffectiveChange::Updated { path, kind, previous, current } => {
                    assert_eq!(
                        self.facts.get(path),
                        Some(&ModelFact { kind: *kind, size: previous.size }),
                        "update did not describe the modeled previous fact at {}",
                        path.display()
                    );
                    self.facts.insert(path.clone(), ModelFact { kind: *kind, size: current.size });
                }
                EffectiveChange::Removed { path, kind, attrs } => {
                    assert_eq!(
                        self.facts.remove(path),
                        Some(ModelFact { kind: *kind, size: attrs.size }),
                        "remove did not describe the modeled fact at {}",
                        path.display()
                    );
                }
                EffectiveChange::ControlUpdated { .. }
                | EffectiveChange::Reclassified { .. }
                | EffectiveChange::Invalidated { .. } => {}
            }
        }
        for transition in &commit.state {
            match transition {
                StateTransition::Freshness { path, previous, current } => {
                    let modeled = self.freshness.get(path).copied().unwrap_or(Freshness::Fresh);
                    assert_eq!(
                        modeled,
                        *previous,
                        "freshness transition skipped for {}",
                        path.display()
                    );
                    self.freshness.insert(path.clone(), *current);
                }
                StateTransition::IndexState { previous, current } => {
                    assert_eq!(self.state, *previous, "index-state transition skipped");
                    self.state = *current;
                }
                StateTransition::Verified { .. } | StateTransition::DirectoryComplete { .. } => {}
            }
        }
    }

    fn assert_public_projection(&self, opened: &OpenedIndex) {
        let response = opened
            .read(ReadRequest {
                projections: vec![
                    ReadProjection::Flat {
                        selection: crate::query::Selection::default(),
                        shape: RowShape::Full,
                        page: crate::PageRequest {
                            limit: crate::MAX_PAGE_ROWS,
                            max_work: crate::MAX_PAGE_WORK,
                        },
                    },
                    ReadProjection::RollUp { path: PathBuf::new() },
                ],
                expected: None,
            })
            .expect("model comparison read");
        let ProjectionResult::Flat(page) = &response.results[0] else {
            panic!("model comparison did not return a flat page");
        };
        assert!(page.next.is_none(), "canonical fixture exceeded one bounded model page");
        let actual: BTreeMap<_, _> = page
            .rows
            .iter()
            .map(|row| (row.path.clone(), ModelFact { kind: row.kind, size: row.attrs.size }))
            .collect();
        assert_eq!(actual, self.facts, "modeled facts and coherent flat projection diverged");

        let ProjectionResult::RollUp(Knowledge::Present(rollup)) = &response.results[1] else {
            panic!("model comparison did not return the root roll-up");
        };
        let files = self.facts.values().filter(|fact| fact.kind == EntryKind::File).count() as u64;
        let dirs = self.facts.values().filter(|fact| fact.kind == EntryKind::Dir).count() as u64;
        let bytes = self
            .facts
            .values()
            .filter(|fact| fact.kind == EntryKind::File)
            .map(|fact| fact.size)
            .sum::<u64>();
        assert_eq!((rollup.all.files, rollup.all.dirs, rollup.all.bytes), (files, dirs, bytes));
    }
}

fn scan_fixture(root: &Path) -> BTreeMap<PathBuf, ModelFact> {
    fn visit(root: &Path, relative: &Path, output: &mut BTreeMap<PathBuf, ModelFact>) {
        let directory = root.join(relative);
        let mut entries: Vec<_> = std::fs::read_dir(&directory)
            .expect("scan model directory")
            .map(|entry| entry.expect("scan model entry"))
            .collect();
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            let path = relative.join(entry.file_name());
            let metadata = std::fs::symlink_metadata(entry.path()).expect("scan model metadata");
            let kind = if metadata.file_type().is_dir() {
                EntryKind::Dir
            } else if metadata.file_type().is_file() {
                EntryKind::File
            } else if metadata.file_type().is_symlink() {
                EntryKind::Symlink
            } else {
                EntryKind::Other
            };
            output.insert(path.clone(), ModelFact { kind, size: metadata.len() });
            if kind == EntryKind::Dir {
                visit(root, &path, output);
            }
        }
    }

    let mut facts = BTreeMap::new();
    visit(root, Path::new(""), &mut facts);
    facts
}

fn golden_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden/opened-root")
        .join(format!("{name}.golden"))
}

/// Replace one named debug integer with a closed normalization token.
fn replace_integer_field(source: String, field: &str, replacement: &str) -> String {
    replace_integer_after(source, &format!("{field}: "), replacement)
}

fn replace_integer_after(mut source: String, needle: &str, replacement: &str) -> String {
    let mut offset = 0;
    while let Some(found) = source[offset..].find(needle) {
        let start = offset + found + needle.len();
        let bytes = source.as_bytes();
        let mut end = start;
        if bytes.get(end) == Some(&b'-') {
            end += 1;
        }
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        if end == start || (end == start + 1 && bytes.get(start) == Some(&b'-')) {
            offset = start;
            continue;
        }
        source.replace_range(start..end, replacement);
        offset = start + replacement.len();
    }
    source
}

/// Coverage keys derived only from production values observed by the session runner.
#[derive(Default)]
pub(super) struct ContractCoverage {
    keys: BTreeSet<&'static str>,
}

impl ContractCoverage {
    pub(super) fn merge(&mut self, other: &Self) {
        self.keys.extend(other.keys.iter().copied());
    }

    pub(super) fn assert_required(&self, required: &[&'static str]) {
        let missing: Vec<_> =
            required.iter().copied().filter(|key| !self.keys.contains(key)).collect();
        assert!(missing.is_empty(), "opened-root contract outcomes not observed: {missing:#?}");
    }

    fn key(&mut self, key: &'static str) {
        self.keys.insert(key);
    }

    fn observe_state(&mut self, state: IndexState) {
        self.key(match state.phase {
            LifecyclePhase::Discovering => "phase.discovering",
            LifecyclePhase::Reconciling => "phase.reconciling",
            LifecyclePhase::Ready => "phase.ready",
            LifecyclePhase::Watching => "phase.watching",
            LifecyclePhase::Stopped => "phase.stopped",
            LifecyclePhase::Failed => "phase.failed",
        });
        match state.coverage {
            Coverage::Complete => self.key("coverage.complete"),
            Coverage::Partial(reason) => {
                self.key("coverage.partial");
                self.key(match reason {
                    CoverageReason::Building => "coverage_reason.building",
                    CoverageReason::Budget => "coverage_reason.budget",
                    CoverageReason::Cancelled => "coverage_reason.cancelled",
                    CoverageReason::Inaccessible => "coverage_reason.inaccessible",
                    CoverageReason::Failed => "coverage_reason.failed",
                });
            }
        }
        self.key(match state.freshness {
            Freshness::Fresh => "freshness.fresh",
            Freshness::Reconciling => "freshness.reconciling",
            Freshness::Stale => "freshness.stale",
            Freshness::Partial => "freshness.partial",
        });
    }

    fn observe_read(&mut self, result: &crate::Result<ReadResponse>) {
        self.key("operation.read");
        match result {
            Ok(response) => {
                self.key("read.ok");
                self.observe_state(response.state);
                for projection in &response.results {
                    match projection {
                        ProjectionResult::Lookup(value) => {
                            self.key("projection.lookup");
                            self.observe_knowledge(value);
                        }
                        ProjectionResult::RollUp(value) => {
                            self.key("projection.rollup");
                            self.observe_knowledge(value);
                        }
                        ProjectionResult::Tree(value) => {
                            self.key("projection.tree");
                            self.observe_knowledge(value);
                        }
                        ProjectionResult::Flat(_) => self.key("projection.flat"),
                        ProjectionResult::Aggregate(_) => self.key("projection.aggregate"),
                        ProjectionResult::Report(_) => self.key("projection.report"),
                        ProjectionResult::Diagnostics(_) => self.key("projection.diagnostics"),
                        ProjectionResult::Limit(_) => self.key("projection.limit"),
                    }
                }
            }
            Err(error) => {
                self.key("read.error");
                self.observe_error(error);
            }
        }
    }

    fn observe_knowledge<T>(&mut self, knowledge: &Knowledge<T>) {
        self.key(match knowledge {
            Knowledge::Present(_) => "knowledge.present",
            Knowledge::Absent => "knowledge.absent",
            Knowledge::Unknown { .. } => "knowledge.unknown",
        });
    }

    fn observe_poll(&mut self, result: &crate::Result<ChangePoll>) {
        self.key("operation.changes");
        match result {
            Ok(poll) => {
                self.observe_state(poll.state);
                match &poll.outcome {
                    ChangeOutcome::Changes { commits, .. } => {
                        self.key("changes.commits");
                        for commit in commits {
                            self.observe_commit(commit);
                        }
                    }
                    ChangeOutcome::Idle => self.key("changes.idle"),
                    ChangeOutcome::Reset { .. } => self.key("changes.reset"),
                }
            }
            Err(error) => {
                self.key("changes.error");
                self.observe_error(error);
            }
        }
    }

    fn observe_refresh(&mut self, result: &crate::Result<RefreshResult>) {
        self.key("operation.refresh");
        match result {
            Ok(receipt) => {
                self.key("refresh.ok");
                self.observe_state(receipt.state);
                if !receipt.rejected.is_empty() {
                    self.key("refresh.rejected");
                }
            }
            Err(error) => {
                self.key("refresh.error");
                self.observe_error(error);
            }
        }
    }

    fn observe_commit(&mut self, commit: &Commit) {
        for change in &commit.changes {
            self.key(match change {
                EffectiveChange::Inserted { .. } => "change.inserted",
                EffectiveChange::Updated { .. } => "change.updated",
                EffectiveChange::Removed { .. } => "change.removed",
                EffectiveChange::ControlUpdated { .. } => "change.control_updated",
                EffectiveChange::Reclassified { .. } => "change.reclassified",
                EffectiveChange::Invalidated { .. } => "change.invalidated",
            });
        }
        for transition in &commit.state {
            match transition {
                StateTransition::Freshness { previous, current, .. } => {
                    self.key("transition.freshness");
                    self.observe_freshness(*previous);
                    self.observe_freshness(*current);
                }
                StateTransition::Verified { .. } => self.key("transition.verified"),
                StateTransition::DirectoryComplete { .. } => {
                    self.key("transition.directory_complete");
                }
                StateTransition::IndexState { previous, current } => {
                    self.key("transition.index_state");
                    self.observe_state(*previous);
                    self.observe_state(*current);
                }
            }
        }
    }

    fn observe_freshness(&mut self, freshness: Freshness) {
        self.key(match freshness {
            Freshness::Fresh => "freshness.fresh",
            Freshness::Reconciling => "freshness.reconciling",
            Freshness::Stale => "freshness.stale",
            Freshness::Partial => "freshness.partial",
        });
    }

    fn observe_error(&mut self, error: &Error) {
        self.key(match error {
            Error::OpenedIndexClosed => "error.closed",
            Error::OpenedIndexStopped => "error.stopped",
            Error::PriorityPathLimit { .. } => "error.priority_limit",
            Error::RefreshPathLimit { .. } => "error.refresh_limit",
            Error::ReadProjectionLimit { .. } => "error.read_limit",
            Error::VersionUnavailable { .. } => "error.version_unavailable",
            Error::ContinuationUnavailable => "error.continuation_unavailable",
            Error::ContinuationRecordLimit { .. } => "error.continuation_record_limit",
            Error::ContinuationStale { .. } => "error.continuation_stale",
            Error::ChangeCursorUnavailable { .. } => "error.change_cursor_unavailable",
            Error::OpenedWorkerPanicked { .. } => "error.worker_panicked",
            Error::OpenedWorkerFailed { .. } => "error.worker_failed",
            _ => "error.other",
        });
    }
}
