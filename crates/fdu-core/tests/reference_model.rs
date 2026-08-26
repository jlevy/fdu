//! Independent state-machine oracle for the retained metadata index.
//!
//! This deliberately does not call index mutation or reducer helpers. The model keeps
//! a canonical path map, recomputes every roll-up from facts, and uses its own logical
//! identities for conditional observations. A failing generated trace prints the seed
//! and every prior action so a discovery can be minimized into a named regression.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use fdu_core::{
    AppliedDelta, ApplyOutcome, ApplyStats, Attrs, Clock, Commit, EffectiveChange, EntryKind,
    ExtTally, Freshness, Impact, ImpactDomain, Index, InvalidateReason, Observation, ObservationOp,
    Op, PathExpectation, PathState, RollUp, StateTransition,
};

const JOURNAL_CAPACITY: usize = 64 * 1024;
/// Contractual dirty-path bound checked independently of the production constant.
const EXPECTED_DIRTY_PATH_LIMIT: usize = 256;
/// Number of transitions exercised for each reproducible generator seed.
const GENERATED_STEPS: usize = 400;
/// State-machine actions from which each generated transition chooses.
const ACTION_COUNT: usize = 10;
const CAPTURE_ACTION: usize = 8;
const APPLY_CAPTURED_ACTION: usize = 9;
/// Linear-congruential constants from the well-known MMIX generator.
const GENERATOR_MULTIPLIER: u64 = 6_364_136_223_846_793_005;
const GENERATOR_INCREMENT: u64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Identity {
    id: u64,
    revision: u64,
    children_revision: u64,
    directory: bool,
}

#[derive(Clone, Debug)]
struct Node {
    kind: EntryKind,
    attrs: Attrs,
    id: u64,
    revision: u64,
    children_revision: u64,
}

impl Node {
    fn identity(&self) -> Identity {
        Identity {
            id: self.id,
            revision: self.revision,
            children_revision: self.children_revision,
            directory: self.kind.is_dir(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ModelExpectation {
    state: PathState,
    entry: Option<Identity>,
    absence_guard: Option<Identity>,
}

#[derive(Clone, Copy, Debug)]
enum ModelCondition {
    Any,
    State(ModelExpectation),
}

#[derive(Clone, Debug)]
struct ModelOp {
    op: Op,
    condition: ModelCondition,
}

#[derive(Clone, Debug)]
struct Model {
    nodes: BTreeMap<PathBuf, Node>,
    next_id: u64,
    clock: Clock,
    journal: VecDeque<Commit>,
    journal_cost: usize,
    journal_floor: Clock,
    pending_invalidations: Vec<(PathBuf, InvalidateReason)>,
    freshness_epoch: u64,
    freshness: BTreeMap<PathBuf, (Freshness, u64)>,
}

impl Model {
    fn new() -> Self {
        let mut nodes = BTreeMap::new();
        nodes.insert(
            PathBuf::new(),
            Node {
                kind: EntryKind::Dir,
                attrs: Attrs::default(),
                id: 0,
                revision: 0,
                children_revision: 0,
            },
        );
        Self {
            nodes,
            next_id: 1,
            clock: Clock::ZERO,
            journal: VecDeque::new(),
            journal_cost: 0,
            journal_floor: Clock::ZERO,
            pending_invalidations: Vec::new(),
            freshness_epoch: 0,
            freshness: BTreeMap::new(),
        }
    }

    fn capture(&self, path: &Path) -> ModelExpectation {
        let entry = self.nodes.get(path).map(Node::identity);
        ModelExpectation {
            state: self.path_state(path),
            entry,
            absence_guard: entry.is_none().then(|| self.absence_guard(path)).flatten(),
        }
    }

    fn path_state(&self, path: &Path) -> PathState {
        self.nodes.get(path).map_or(PathState::Absent, |node| PathState::Present {
            kind: node.kind,
            attrs: node.attrs,
        })
    }

    fn absence_guard(&self, path: &Path) -> Option<Identity> {
        let mut current = PathBuf::new();
        let mut identity = self.nodes.get(Path::new("")).map(Node::identity);
        let mut components = path.components().peekable();
        while let Some(component) = components.next() {
            if components.peek().is_none() {
                break;
            }
            current.push(component.as_os_str());
            let Some(node) = self.nodes.get(&current) else {
                break;
            };
            identity = Some(node.identity());
        }
        identity
    }

    fn expectation_matches(&self, op: &Op, expected: ModelExpectation) -> bool {
        if self.path_state(op.path()) != expected.state {
            return false;
        }
        let require_structure = match (op, expected.state) {
            (Op::Remove { .. }, _) => true,
            (Op::Upsert { kind, .. }, PathState::Present { kind: old, .. }) => *kind != old,
            (Op::Upsert { .. } | Op::InvalidateSubtree { .. }, _) => false,
        };
        if !same_target(
            self.nodes.get(op.path()).map(Node::identity),
            expected.entry,
            require_structure,
        ) {
            return false;
        }
        expected.absence_guard.is_none_or(|expected_guard| {
            self.absence_guard(op.path())
                .is_some_and(|current| same_absence_guard(current, expected_guard))
        })
    }

    fn apply(&mut self, ops: &[ModelOp]) -> ApplyOutcome {
        let accepted: Vec<bool> = ops
            .iter()
            .map(|observed| match observed.condition {
                ModelCondition::Any => true,
                ModelCondition::State(expected) => self.expectation_matches(&observed.op, expected),
            })
            .collect();
        let observed = u64::try_from(ops.len()).expect("model operation count");
        let mut stats = ApplyStats::default();
        let mut changes = Vec::new();
        let mut transitions = Vec::new();
        for (observed, accepted) in ops.iter().zip(accepted) {
            if !accepted {
                stats.stale += 1;
                continue;
            }
            match &observed.op {
                Op::Upsert { path, kind, attrs } => {
                    self.upsert(path, *kind, *attrs, &mut stats, &mut changes);
                }
                Op::Remove { path } => {
                    self.remove(path, &mut stats, &mut changes);
                }
                Op::InvalidateSubtree { path, reason } => {
                    let previous = self.freshness_at(path);
                    self.pending_invalidations.push((path.clone(), *reason));
                    self.mark_unfresh(path, Freshness::Stale);
                    let current = self.freshness_at(path);
                    stats.invalidated += 1;
                    changes
                        .push(EffectiveChange::Invalidated { path: path.clone(), reason: *reason });
                    if previous != current {
                        transitions.push(StateTransition::Freshness {
                            path: path.clone(),
                            previous,
                            current,
                        });
                    }
                }
            }
        }
        if changes.is_empty() && transitions.is_empty() {
            return ApplyOutcome { stats, commit: None, applied: None };
        }
        self.clock = self.clock.checked_next().expect("model clock");
        let commit = Commit {
            clock: self.clock,
            impact: model_impact(&changes, &transitions),
            changes,
            state: transitions,
            work: model_work(observed, stats),
        };
        self.retain(commit.clone());
        let applied = commit.applied_delta();
        ApplyOutcome { stats, commit: Some(commit), applied }
    }

    fn upsert(
        &mut self,
        path: &Path,
        kind: EntryKind,
        attrs: Attrs,
        stats: &mut ApplyStats,
        changes: &mut Vec<EffectiveChange>,
    ) -> bool {
        if path.as_os_str().is_empty() {
            let root = self.nodes.get_mut(Path::new("")).expect("model root");
            if root.attrs == attrs {
                stats.unchanged += 1;
                return false;
            }
            let previous = root.attrs;
            root.attrs = attrs;
            root.revision += 1;
            stats.updated += 1;
            changes.push(EffectiveChange::Updated {
                path: PathBuf::new(),
                kind: EntryKind::Dir,
                previous,
                current: attrs,
            });
            return true;
        }
        let parent = path.parent().unwrap_or(Path::new(""));
        assert!(
            self.nodes.get(parent).is_some_and(|node| node.kind.is_dir()),
            "reference producer must establish exact directory ancestry before {}",
            path.display()
        );
        if let Some(node) = self.nodes.get_mut(path) {
            if node.kind == kind && node.attrs == attrs {
                stats.unchanged += 1;
                return false;
            }
            if node.kind == kind {
                let previous = node.attrs;
                node.attrs = attrs;
                node.revision += 1;
                stats.updated += 1;
                changes.push(EffectiveChange::Updated {
                    path: path.to_path_buf(),
                    kind,
                    previous,
                    current: attrs,
                });
                return true;
            }
            self.remove(path, stats, changes);
        }
        self.insert(path, kind, attrs);
        stats.inserted += 1;
        changes.push(EffectiveChange::Inserted { path: path.to_path_buf(), kind, attrs });
        true
    }

    fn insert(&mut self, path: &Path, kind: EntryKind, attrs: Attrs) {
        let parent = path.parent().unwrap_or(Path::new(""));
        self.bump_children(parent);
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(
            path.to_path_buf(),
            Node { kind, attrs, id, revision: 0, children_revision: 0 },
        );
    }

    fn remove(
        &mut self,
        path: &Path,
        stats: &mut ApplyStats,
        changes: &mut Vec<EffectiveChange>,
    ) -> bool {
        if path.as_os_str().is_empty() || !self.nodes.contains_key(path) {
            stats.unchanged += 1;
            return false;
        }
        let parent = path.parent().unwrap_or(Path::new("")).to_path_buf();
        let removed = self.removal_order(path);
        stats.removed += u64::try_from(removed.len()).expect("model entry count");
        for candidate in removed {
            let node = self.nodes.get(&candidate).expect("model removal node");
            changes.push(EffectiveChange::Removed {
                path: candidate.clone(),
                kind: node.kind,
                attrs: node.attrs,
            });
            self.nodes.remove(&candidate);
        }
        self.bump_children(&parent);
        true
    }

    fn removal_order(&self, path: &Path) -> Vec<PathBuf> {
        let mut result = Vec::new();
        let mut queue = VecDeque::from([path.to_path_buf()]);
        while let Some(current) = queue.pop_front() {
            result.push(current.clone());
            queue.extend(
                self.nodes
                    .keys()
                    .filter(|candidate| {
                        candidate.as_path() != current
                            && candidate.parent() == Some(current.as_path())
                    })
                    .cloned(),
            );
        }
        result
    }

    fn bump_children(&mut self, path: &Path) {
        self.nodes.get_mut(path).expect("model parent").children_revision += 1;
    }

    fn mark_unfresh(&mut self, path: &Path, freshness: Freshness) {
        self.freshness_epoch += 1;
        self.freshness.insert(path.to_path_buf(), (freshness, self.freshness_epoch));
    }

    fn begin_reconcile(&mut self, path: &Path) -> u64 {
        let previous = self.freshness_at(path);
        self.mark_unfresh(path, Freshness::Reconciling);
        let started_at = self.freshness_epoch;
        let current = self.freshness_at(path);
        if previous != current {
            self.commit_state(vec![StateTransition::Freshness {
                path: path.to_path_buf(),
                previous,
                current,
            }]);
        }
        started_at
    }

    fn finish_reconcile(&mut self, path: &Path, started_at: u64, complete: bool) {
        let previous = self.freshness_at(path);
        self.freshness
            .retain(|marked, (_, epoch)| !marked.starts_with(path) || *epoch > started_at);
        let mut state = Vec::new();
        if complete {
            state.push(StateTransition::Verified { path: path.to_path_buf() });
        } else {
            self.mark_unfresh(path, Freshness::Partial);
        }
        let current = self.freshness_at(path);
        if previous != current {
            state.push(StateTransition::Freshness { path: path.to_path_buf(), previous, current });
        }
        if !state.is_empty() {
            self.commit_state(state);
        }
    }

    fn commit_state(&mut self, state: Vec<StateTransition>) {
        self.clock = self.clock.checked_next().expect("model state clock");
        let commit = Commit {
            clock: self.clock,
            changes: Vec::new(),
            impact: model_impact(&[], &state),
            state,
            work: fdu_core::Work::default(),
        };
        self.retain(commit);
    }

    fn freshness_at(&self, path: &Path) -> Freshness {
        self.freshness
            .iter()
            .filter(|(marked, _)| path.starts_with(marked) || marked.starts_with(path))
            .map(|(_, (state, _))| *state)
            .max_by_key(|state| freshness_rank(*state))
            .unwrap_or(Freshness::Fresh)
    }

    fn rollup(&self, path: &Path) -> Option<RollUp> {
        self.nodes.get(path)?.kind.is_dir().then(|| {
            let mut result = RollUp::default();
            let mut newest = None;
            for (candidate, node) in &self.nodes {
                if candidate == path || !candidate.starts_with(path) {
                    continue;
                }
                match node.kind {
                    EntryKind::Dir => result.dirs += 1,
                    EntryKind::File => {
                        result.files += 1;
                        result.bytes += node.attrs.size;
                        result.allocated += node.attrs.allocated;
                        newest =
                            Some(newest.map_or(node.attrs.mtime_ns, |old: i64| {
                                old.max(node.attrs.mtime_ns)
                            }));
                        let ext = extension_bucket(candidate.file_name().expect("non-root file"));
                        let tally = result.by_ext.entry(ext).or_insert_with(ExtTally::default);
                        tally.files += 1;
                        tally.bytes += node.attrs.size;
                        tally.allocated += node.attrs.allocated;
                    }
                    EntryKind::Symlink | EntryKind::Other => {}
                }
            }
            result.newest_mtime_ns = newest.unwrap_or(0);
            result
        })
    }

    fn children(&self, path: &Path) -> Option<Vec<OsString>> {
        self.nodes.get(path)?.kind.is_dir().then(|| {
            self.nodes
                .keys()
                .filter(|candidate| {
                    !candidate.as_os_str().is_empty() && candidate.parent() == Some(path)
                })
                .map(|candidate| candidate.file_name().expect("child").to_os_string())
                .collect()
        })
    }

    fn has_known_ancestry(&self, path: &Path) -> bool {
        path.parent()
            .unwrap_or(Path::new(""))
            .ancestors()
            .all(|ancestor| self.nodes.get(ancestor).is_some_and(|node| node.kind.is_dir()))
    }

    fn retain(&mut self, commit: Commit) {
        let cost = commit.retained_cost();
        if cost > JOURNAL_CAPACITY {
            self.journal.clear();
            self.journal_cost = 0;
            self.journal_floor = commit.clock;
            return;
        }
        while self.journal_cost + cost > JOURNAL_CAPACITY {
            let dropped = self.journal.pop_front().expect("over-capacity model journal");
            self.journal_cost -= dropped.retained_cost();
            self.journal_floor = dropped.clock;
        }
        self.journal_cost += cost;
        self.journal.push_back(commit);
    }

    fn since(&self, clock: Clock) -> (Vec<Commit>, Vec<AppliedDelta>, bool) {
        let commits: Vec<Commit> =
            self.journal.iter().filter(|commit| commit.clock > clock).cloned().collect();
        let deltas = commits.iter().filter_map(Commit::applied_delta).collect();
        (commits, deltas, clock < self.journal_floor)
    }

    fn take_pending_invalidations(&mut self) -> Vec<(PathBuf, InvalidateReason)> {
        std::mem::take(&mut self.pending_invalidations)
    }
}

fn model_impact(changes: &[EffectiveChange], state: &[StateTransition]) -> Impact {
    let mut domains = BTreeSet::new();
    let mut dirty_paths = BTreeSet::new();
    let mut all_dirty = false;
    for change in changes {
        match change {
            EffectiveChange::Inserted { .. } | EffectiveChange::Removed { .. } => {
                domains.extend([
                    ImpactDomain::Topology,
                    ImpactDomain::Metadata,
                    ImpactDomain::Classification,
                    ImpactDomain::Aggregates,
                    ImpactDomain::Content,
                ]);
            }
            EffectiveChange::Updated { .. } => {
                domains.extend([
                    ImpactDomain::Metadata,
                    ImpactDomain::Aggregates,
                    ImpactDomain::Content,
                ]);
            }
            EffectiveChange::Invalidated { .. } => {
                domains.insert(ImpactDomain::State);
            }
        }
        model_dirty(change.path(), &mut dirty_paths, &mut all_dirty, EXPECTED_DIRTY_PATH_LIMIT);
    }
    for transition in state {
        domains.insert(ImpactDomain::State);
        model_dirty(transition.path(), &mut dirty_paths, &mut all_dirty, EXPECTED_DIRTY_PATH_LIMIT);
    }
    Impact {
        domains: domains.into_iter().collect(),
        dirty_paths: if all_dirty { Vec::new() } else { dirty_paths.into_iter().collect() },
        all_dirty,
    }
}

fn model_work(observations: u64, stats: ApplyStats) -> fdu_core::Work {
    fdu_core::Work { observations, unchanged: stats.unchanged, stale: stats.stale }
}

fn model_dirty(
    path: &Path,
    dirty_paths: &mut BTreeSet<PathBuf>,
    all_dirty: &mut bool,
    limit: usize,
) {
    if *all_dirty {
        return;
    }
    for ancestor in path.ancestors() {
        dirty_paths.insert(ancestor.to_path_buf());
        if dirty_paths.len() > limit {
            dirty_paths.clear();
            *all_dirty = true;
            return;
        }
    }
}

fn same_target(current: Option<Identity>, expected: Option<Identity>, structure: bool) -> bool {
    match (current, expected) {
        (Some(current), Some(expected)) => {
            current.id == expected.id
                && current.revision == expected.revision
                && (!structure || current.children_revision == expected.children_revision)
        }
        (None, None) => true,
        (Some(_), None) | (None, Some(_)) => false,
    }
}

fn same_absence_guard(current: Identity, expected: Identity) -> bool {
    current.id == expected.id
        && current.children_revision == expected.children_revision
        && (expected.directory || current.revision == expected.revision)
}

fn freshness_rank(freshness: Freshness) -> u8 {
    match freshness {
        Freshness::Fresh => 0,
        Freshness::Reconciling => 1,
        Freshness::Stale => 2,
        Freshness::Partial => 3,
    }
}

fn extension_bucket(name: &OsStr) -> String {
    // The model derives this from name bytes independently of the production
    // classifier. Lossy conversion is sufficient here because generated extensions
    // are ASCII and an invalid native stem must not erase an otherwise valid suffix.
    let name = name.to_string_lossy();
    let searchable = name.strip_prefix('.').unwrap_or(&name);
    let Some(dot) = searchable.rfind('.') else {
        return "(none)".into();
    };
    let (stem, extension) = searchable.split_at(dot);
    if extension.len() <= 1 {
        return "(none)".into();
    }
    if let Some(inner_dot) = stem.rfind('.') {
        if stem[inner_dot..].eq_ignore_ascii_case(".tar") {
            return format!(".tar{}", extension.to_ascii_lowercase());
        }
    }
    extension.to_ascii_lowercase()
}

fn attrs(value: u64) -> Attrs {
    let signed = i64::try_from(value % 41).expect("bounded") - 20;
    Attrs {
        size: value % 4096,
        allocated: (value % 9) * 512,
        mtime_ns: signed,
        ctime_ns: signed.wrapping_mul(3),
        inode: value.wrapping_mul(31),
        dev: value % 3,
    }
}

fn assert_equivalent(index: &mut Index, model: &mut Model, seed: u64, trace: &[String]) {
    let context = || format!("seed={seed:#018x}\n{}", trace.join("\n"));
    assert_eq!(index.clock(), model.clock, "clock mismatch\n{}", context());
    assert_eq!(
        usize::try_from(index.len()).expect("model-sized index"),
        model.nodes.len(),
        "entry count mismatch\n{}",
        context()
    );
    assert_eq!(
        index.freshness(),
        model.freshness_at(Path::new("")),
        "freshness mismatch\n{}",
        context()
    );

    for (path, node) in &model.nodes {
        assert_eq!(index.kind(path), Some(node.kind), "kind at {}\n{}", path.display(), context());
        assert_eq!(
            index.attrs(path),
            Some(&node.attrs),
            "attrs at {}\n{}",
            path.display(),
            context()
        );
        assert_eq!(
            index.rollup(path),
            model.rollup(path),
            "roll-up at {}\n{}",
            path.display(),
            context()
        );
        let actual_children = index
            .children(path)
            .map(|children| children.map(|(name, _)| name.to_os_string()).collect::<Vec<_>>());
        assert_eq!(
            actual_children,
            model.children(path),
            "children at {}\n{}",
            path.display(),
            context()
        );
        assert_eq!(
            index.freshness_at(path),
            model.freshness_at(path),
            "freshness at {}\n{}",
            path.display(),
            context()
        );
    }

    let mut actual_paths = Vec::new();
    let mut frontier = vec![PathBuf::new()];
    while let Some(path) = frontier.pop() {
        actual_paths.push(path.clone());
        if let Some(children) = index.children(&path) {
            for (name, _) in children.rev() {
                frontier.push(path.join(name));
            }
        }
    }
    assert_eq!(
        actual_paths.into_iter().collect::<BTreeSet<_>>(),
        model.nodes.keys().cloned().collect::<BTreeSet<_>>(),
        "path set mismatch\n{}",
        context()
    );

    let actual_since = index.since(Clock::ZERO);
    let (model_commits, model_deltas, model_truncated) = model.since(Clock::ZERO);
    assert_eq!(actual_since.commits, model_commits, "commit journal mismatch\n{}", context());
    assert_eq!(actual_since.deltas, model_deltas, "journal mismatch\n{}", context());
    assert_eq!(actual_since.truncated, model_truncated, "journal floor mismatch\n{}", context());
    assert_eq!(
        index.take_pending_invalidations(),
        model.take_pending_invalidations(),
        "pending invalidations mismatch\n{}",
        context()
    );
}

#[derive(Clone, Copy)]
struct Generator(u64);

impl Generator {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(GENERATOR_MULTIPLIER).wrapping_add(GENERATOR_INCREMENT);
        self.0
    }

    fn choose(&mut self, length: usize) -> usize {
        usize::try_from(self.next() % u64::try_from(length).expect("choice length"))
            .expect("choice")
    }
}

#[derive(Clone)]
struct Pending {
    op: Op,
    engine: PathExpectation,
    model: ModelExpectation,
}

fn parent_first_upsert(model: &Model, op: Op, generator: &mut Generator) -> Vec<Op> {
    let Op::Upsert { path, kind, attrs: entry_attrs } = op else {
        return vec![op];
    };
    let mut operations = Vec::new();
    let mut established = BTreeSet::new();
    let mut current = PathBuf::new();
    let mut components = path.components().peekable();
    while let Some(component) = components.next() {
        if components.peek().is_none() {
            break;
        }
        current.push(component.as_os_str());
        let known = established.contains(&current)
            || model.nodes.get(&current).is_some_and(|node| node.kind.is_dir());
        if !known {
            operations.push(Op::Upsert {
                path: current.clone(),
                kind: EntryKind::Dir,
                attrs: attrs(generator.next()),
            });
            established.insert(current.clone());
        }
    }
    operations.push(Op::Upsert { path, kind, attrs: entry_attrs });
    operations
}

#[test]
fn fixed_seed_operation_sequences_match_the_independent_model_after_every_step() {
    const SEEDS: [u64; 6] = [
        0x18f4_9a22_d71c_6501,
        0x2b71_0fd3_a484_11ce,
        0x5849_6a4f_028d_94b7,
        0x8873_931a_bf44_62a9,
        0xc139_a641_75de_0403,
        0xf40a_d227_09b2_777d,
    ];
    let paths = [
        "alpha.txt",
        "beta.RS",
        "src",
        "src/lib.rs",
        "src/deep/item.bin",
        "src/deep/other",
        "docs",
        "docs/guide.md",
        "docs/api/index.html",
        "build/out.o",
        "swap/child.txt",
        "missing/late.json",
    ];

    for seed in SEEDS {
        let mut generator = Generator(seed);
        let mut index = Index::new("/model-root");
        let mut model = Model::new();
        let mut pending = Vec::<Pending>::new();
        let mut trace = Vec::new();

        for step in 0..GENERATED_STEPS {
            let path = PathBuf::from(paths[generator.choose(paths.len())]);
            let action = generator.choose(ACTION_COUNT);
            if action == CAPTURE_ACTION {
                let op = match generator.choose(3) {
                    0 => Op::Remove { path: path.clone() },
                    1 => Op::InvalidateSubtree {
                        path: path.clone(),
                        reason: InvalidateReason::Requested,
                    },
                    _ if model.has_known_ancestry(&path) => {
                        let kind =
                            [EntryKind::File, EntryKind::Dir, EntryKind::Symlink, EntryKind::Other]
                                [generator.choose(4)];
                        Op::Upsert { path: path.clone(), kind, attrs: attrs(generator.next()) }
                    }
                    _ => Op::Remove { path: path.clone() },
                };
                pending.push(Pending {
                    engine: index.expectation(&path),
                    model: model.capture(&path),
                    op: op.clone(),
                });
                trace.push(format!("{step:03} capture {op:?}"));
                assert_equivalent(&mut index, &mut model, seed, &trace);
                continue;
            }

            let (observation, model_ops, label) = if action == APPLY_CAPTURED_ACTION
                && !pending.is_empty()
            {
                let captured = pending.remove(generator.choose(pending.len()));
                (
                    Observation::from_ops(vec![ObservationOp::if_state(
                        captured.op.clone(),
                        captured.engine,
                    )]),
                    vec![ModelOp {
                        op: captured.op.clone(),
                        condition: ModelCondition::State(captured.model),
                    }],
                    format!("conditional {:?}", captured.op),
                )
            } else {
                let ops = match action {
                    0..=4 => {
                        let kind =
                            [EntryKind::File, EntryKind::Dir, EntryKind::Symlink, EntryKind::Other]
                                [generator.choose(4)];
                        let op =
                            Op::Upsert { path: path.clone(), kind, attrs: attrs(generator.next()) };
                        parent_first_upsert(&model, op, &mut generator)
                    }
                    5 => vec![Op::Remove { path: path.clone() }],
                    6 => vec![Op::InvalidateSubtree {
                        path: path.clone(),
                        reason: [
                            InvalidateReason::WatchOverflow,
                            InvalidateReason::UnpairedRename,
                            InvalidateReason::WatchSetupRace,
                            InvalidateReason::PeriodicSweep,
                            InvalidateReason::VerificationFailed,
                            InvalidateReason::WatchContention,
                            InvalidateReason::Requested,
                        ][generator.choose(7)],
                    }],
                    7 => model.nodes.get(&path).map_or_else(
                        || vec![Op::Remove { path: path.clone() }],
                        |node| {
                            vec![Op::Upsert {
                                path: path.clone(),
                                kind: node.kind,
                                attrs: node.attrs,
                            }]
                        },
                    ),
                    _ => parent_first_upsert(
                        &model,
                        Op::Upsert {
                            path: PathBuf::from("ordered/child/value.txt"),
                            kind: EntryKind::File,
                            attrs: attrs(generator.next()),
                        },
                        &mut generator,
                    ),
                };
                (
                    Observation::new(ops.clone()),
                    ops.iter()
                        .cloned()
                        .map(|op| ModelOp { op, condition: ModelCondition::Any })
                        .collect(),
                    format!("unconditional {ops:?}"),
                )
            };
            trace.push(format!("{step:03} {label}"));
            let actual = index.apply(&observation).expect("generated paths are valid");
            let expected = model.apply(&model_ops);
            assert_eq!(
                actual,
                expected,
                "outcome mismatch\nseed={seed:#018x}\n{}",
                trace.join("\n")
            );
            assert_equivalent(&mut index, &mut model, seed, &trace);
        }
    }
}

#[test]
fn reconciliation_state_only_commits_match_the_independent_model() {
    let dir = tempfile::tempdir().expect("temporary root");
    std::fs::write(dir.path().join("file.txt"), b"stable").expect("fixture file");
    let config = fdu_core::ScanConfig::default();
    let (mut index, _) = fdu_core::scan::scan_into_index(dir.path(), &config).expect("baseline");
    let before = index.clock();
    let mut model = Model::new();
    let started_at = model.begin_reconcile(Path::new(""));
    model.finish_reconcile(Path::new(""), started_at, true);

    let mut observed_commits = Vec::new();
    let report = fdu_core::scan::reconcile(&mut index, &config, &mut |commit| {
        observed_commits.push(commit.clone());
    })
    .expect("unchanged reconciliation");

    assert!(report.is_complete());
    let actual = index.since(before);
    let (expected_commits, expected_deltas, expected_truncated) = model.since(Clock::ZERO);
    assert_eq!(observed_commits, expected_commits);
    assert_eq!(actual.commits, expected_commits);
    assert_eq!(actual.deltas, expected_deltas);
    assert_eq!(actual.truncated, expected_truncated);
}

#[test]
fn delayed_observations_reject_present_and_absent_aba() {
    let mut index = Index::new("/model-root");
    let mut model = Model::new();
    let initial = Op::Upsert { path: "file.txt".into(), kind: EntryKind::File, attrs: attrs(7) };
    let initial_model = ModelOp { op: initial.clone(), condition: ModelCondition::Any };
    index.apply(&Observation::new(vec![initial])).expect("initial");
    model.apply(&[initial_model]);

    let delayed_path = Path::new("file.txt");
    let engine_expected = index.expectation(delayed_path);
    let model_expected = model.capture(delayed_path);
    let absent_path = Path::new("later.txt");
    let engine_absent = index.expectation(absent_path);
    let model_absent = model.capture(absent_path);

    for op in [
        Op::Remove { path: delayed_path.into() },
        Op::Upsert { path: delayed_path.into(), kind: EntryKind::File, attrs: attrs(7) },
        Op::Upsert { path: absent_path.into(), kind: EntryKind::File, attrs: attrs(9) },
        Op::Remove { path: absent_path.into() },
    ] {
        index.apply(&Observation::new(vec![op.clone()])).expect("ABA mutation");
        model.apply(&[ModelOp { op, condition: ModelCondition::Any }]);
    }

    let present_update =
        Op::Upsert { path: delayed_path.into(), kind: EntryKind::File, attrs: attrs(70) };
    let absent_update =
        Op::Upsert { path: absent_path.into(), kind: EntryKind::File, attrs: attrs(90) };
    let actual = index
        .apply(&Observation::from_ops(vec![
            ObservationOp::if_state(present_update.clone(), engine_expected),
            ObservationOp::if_state(absent_update.clone(), engine_absent),
        ]))
        .expect("delayed observation");
    let expected = model.apply(&[
        ModelOp { op: present_update, condition: ModelCondition::State(model_expected) },
        ModelOp { op: absent_update, condition: ModelCondition::State(model_absent) },
    ]);

    assert_eq!(actual, expected);
    assert_eq!(actual.stats.stale, 2);
    assert!(actual.applied().is_none());
    assert_equivalent(&mut index, &mut model, 0xaba, &["named ABA regression".into()]);
}

#[test]
fn lowering_a_nested_max_repairs_ancestors_above_an_already_correct_parent() {
    // Minimized from seed 0x18f49a22d71c6501. The leaf's single-child parent becomes
    // correct during differential re-merge, while the root still has another file to
    // compare. An early-exit repair walk used to leave the old leaf maximum at root.
    let mut index = Index::new("/model-root");
    let mut model = Model::new();
    let initial = vec![
        Op::Upsert { path: "nested".into(), kind: EntryKind::Dir, attrs: attrs(1) },
        Op::Upsert { path: "nested/only".into(), kind: EntryKind::Dir, attrs: attrs(2) },
        Op::Upsert {
            path: "nested/only/value.txt".into(),
            kind: EntryKind::File,
            attrs: attrs(38),
        },
        Op::Upsert { path: "sibling.txt".into(), kind: EntryKind::File, attrs: attrs(35) },
    ];
    index.apply(&Observation::new(initial.clone())).expect("initial tree");
    model.apply(
        &initial
            .into_iter()
            .map(|op| ModelOp { op, condition: ModelCondition::Any })
            .collect::<Vec<_>>(),
    );
    let lower =
        Op::Upsert { path: "nested/only/value.txt".into(), kind: EntryKind::File, attrs: attrs(1) };
    index.apply(&Observation::new(vec![lower.clone()])).expect("lower nested max");
    model.apply(&[ModelOp { op: lower, condition: ModelCondition::Any }]);

    assert_equivalent(
        &mut index,
        &mut model,
        0x18f4_9a22_d71c_6501,
        &["minimized nested newest-mtime regression".into()],
    );
}

#[test]
fn maximum_recomputation_excludes_symlinks_and_special_objects() {
    // Minimized from the same generated trace. Incremental contributions exclude
    // non-files, but the old full repair path accidentally read every non-directory's
    // mtime and could promote a symlink or special object to the directory maximum.
    let mut index = Index::new("/model-root");
    let mut model = Model::new();
    let initial = vec![
        Op::Upsert { path: "kept.txt".into(), kind: EntryKind::File, attrs: attrs(28) },
        Op::Upsert { path: "link".into(), kind: EntryKind::Symlink, attrs: attrs(37) },
        Op::Upsert { path: "lowered.txt".into(), kind: EntryKind::File, attrs: attrs(38) },
    ];
    index.apply(&Observation::new(initial.clone())).expect("initial tree");
    model.apply(
        &initial
            .into_iter()
            .map(|op| ModelOp { op, condition: ModelCondition::Any })
            .collect::<Vec<_>>(),
    );
    let lower = Op::Upsert { path: "lowered.txt".into(), kind: EntryKind::File, attrs: attrs(1) };
    index.apply(&Observation::new(vec![lower.clone()])).expect("lower file max");
    model.apply(&[ModelOp { op: lower, condition: ModelCondition::Any }]);

    assert_equivalent(
        &mut index,
        &mut model,
        0x18f4_9a22_d71c_6501,
        &["minimized non-file newest-mtime regression".into()],
    );
}

#[test]
fn invalid_observation_is_atomic_and_does_not_advance_the_model() {
    let mut index = Index::new("/model-root");
    let mut model = Model::new();
    let before = index.clone();
    let error = index
        .apply(&Observation::new(vec![
            Op::Upsert {
                path: "would-have-applied.txt".into(),
                kind: EntryKind::File,
                attrs: attrs(1),
            },
            Op::Remove { path: "../escape".into() },
        ]))
        .expect_err("invalid path must refuse the complete observation");

    assert!(matches!(error, fdu_core::Error::PathEscapesRoot(_)));
    assert_eq!(index.clock(), before.clock());
    assert_eq!(index.total(), before.total());
    assert_eq!(index.len(), before.len());
    assert_equivalent(&mut index, &mut model, 0xbad, &["invalid batch".into()]);
}

#[test]
fn bounded_journal_reports_loss_at_the_same_clock_as_the_model() {
    let mut index = Index::new("/model-root");
    let mut model = Model::new();
    for value in 0..=JOURNAL_CAPACITY {
        let op = Op::Upsert {
            path: "changing.txt".into(),
            kind: EntryKind::File,
            attrs: attrs(u64::try_from(value).expect("bounded")),
        };
        index.apply(&Observation::new(vec![op.clone()])).expect("journal mutation");
        model.apply(&[ModelOp { op, condition: ModelCondition::Any }]);
    }

    let actual = index.since(Clock::ZERO);
    let (expected_commits, expected_deltas, expected_truncated) = model.since(Clock::ZERO);
    assert_eq!(actual.truncated, expected_truncated);
    assert_eq!(actual.commits, expected_commits);
    assert_eq!(actual.deltas, expected_deltas);
    assert!(actual.truncated);
    assert!(!actual.deltas.is_empty());
}

#[cfg(unix)]
#[test]
fn non_unicode_names_remain_distinct_in_the_model_and_index() {
    use std::os::unix::ffi::OsStringExt;

    let first = PathBuf::from(OsString::from_vec(vec![b'n', 0x80, b'.', b'R', b'S']));
    let second = PathBuf::from(OsString::from_vec(vec![b'n', 0x81, b'.', b'R', b'S']));
    let ops = vec![
        Op::Upsert { path: first, kind: EntryKind::File, attrs: attrs(10) },
        Op::Upsert { path: second, kind: EntryKind::File, attrs: attrs(20) },
    ];
    let mut index = Index::new("/model-root");
    let mut model = Model::new();
    let actual = index.apply(&Observation::new(ops.clone())).expect("native names");
    let expected = model.apply(
        &ops.into_iter()
            .map(|op| ModelOp { op, condition: ModelCondition::Any })
            .collect::<Vec<_>>(),
    );

    assert_eq!(actual, expected);
    assert_equivalent(&mut index, &mut model, 0x80_81, &["non-Unicode names".into()]);
}
