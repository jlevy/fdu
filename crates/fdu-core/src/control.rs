//! Bounded, removal-aware control state used to classify retained filesystem facts.
//!
//! A control file is producer input, not something the index discovers by walking the
//! filesystem. The table stores the exact bytes a producer verified and the matcher
//! derived from them. That keeps cold discovery, refresh, observation, and snapshot load
//! on one semantic path and makes deleting the last control file an ordinary state
//! transition rather than a special rebuild.
//!
//! **Which of git's ignore inputs count.** Exactly one: each directory's
//! [`CONTROL_FILE_NAME`] inside the scanned root, governing its own directory and
//! everything below it, with deeper files taking precedence. Nothing else git consults is
//! read. `.git/info/exclude` and `core.excludesFile` are ignored, so a `.DS_Store` excluded
//! only globally lands in the unignored partition. A nested repository is not a boundary:
//! its `.gitignore` files join the outer ones as if the tree were one repository. And
//! unlike git, which never looks inside an ignored directory, the walk reads a
//! `.gitignore` there too; the ignored partition is still right, because git's rule that
//! an excluded parent cannot be re-included is applied when matching, but the file's
//! bytes are retained against the table bound. Matching is case-sensitive regardless of
//! `core.ignorecase`.
//!
//! **Which file is a directory's control.** Whatever a lookup of `<dir>/.gitignore`
//! resolves to, because that is the path git opens. On a case-sensitive directory that is
//! only an entry named exactly `.gitignore`; on a case-insensitive one (APFS and NTFS by
//! default, an ext4 casefold directory) it is the one entry the filesystem folds to that
//! name, so a `.GITIGNORE` governs there as it does for git, and nowhere else. The rules
//! are recorded under the canonical path `<dir>/.gitignore` whatever spelling holds them:
//! every control operation, table key, refusal, and change names that path, as
//! `git check-ignore -v` does, so [`is_control_file`] accepts only that path. A walk pays
//! for this only on a listed name spelled `.gitignore` in another ASCII case, which it
//! resolves with one lookup of the canonical path; the exact name is read by its own path
//! as before, and every other name costs a length comparison. A name some filesystem
//! folds to `.gitignore` through a non-ASCII character is not looked up.

mod gitignore;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use gitignore::Gitignore;

/// Name of the fixed control file understood by the first engine version.
pub const CONTROL_FILE_NAME: &str = ".gitignore";

/// Default retained control-table charge for one index, in bytes.
///
/// The charge includes a fixed amount per directory as well as each distinct source's
/// bytes and matcher, so a hostile tree of empty control files cannot evade it. Four MiB is
/// far above ordinary repositories while remaining small relative to the inventory it
/// governs; a source past it is refused, not an error. Callers set it with
/// [`ControlLimits::budget`], which the command line spells `--gitignore-budget`.
pub const DEFAULT_CONTROL_BUDGET: usize = 4 * 1024 * 1024;

/// Default longest line a control source may hold, in bytes.
///
/// A control file may hold many ordinary rules up to the budget, but a single adversarial
/// rule must not impose unbounded matching work on every entry, so a source with a longer
/// line is refused whole. Callers set it with [`ControlLimits::line_limit`], which the
/// command line spells `--gitignore-line-limit`.
pub const DEFAULT_CONTROL_LINE_LIMIT: usize = 16 * 1024;

/// The two bounds on the control state one index retains, each liftable on its own.
///
/// They bound different things. The budget bounds the memory the whole table retains; the
/// line limit bounds what one pattern costs to match against every entry. So raising the
/// budget admits more files without admitting longer lines, and lifting the line limit
/// admits long lines without retaining more. A source either bound cannot admit is
/// refused, and its [`ControlRefusalReason`] names the one that fired.
///
/// Both are part of the scan scope: they decide which rules apply, so a snapshot taken
/// under other limits never serves a request for these.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ControlLimits {
    /// Bytes of retained charge before further sources are refused, or `None` for no
    /// bound.
    ///
    /// Each directory's key and each distinct source's bytes and matcher are charged, so
    /// identical files count once. It also bounds the read: a control file is read to one
    /// byte past the budget, so `None` reads every control file whole, however large.
    pub budget: Option<usize>,
    /// Longest line in bytes a source may hold before it is refused whole, or `None` for
    /// no bound.
    pub line_limit: Option<usize>,
}

impl Default for ControlLimits {
    /// [`DEFAULT_CONTROL_BUDGET`] and [`DEFAULT_CONTROL_LINE_LIMIT`].
    fn default() -> Self {
        Self { budget: Some(DEFAULT_CONTROL_BUDGET), line_limit: Some(DEFAULT_CONTROL_LINE_LIMIT) }
    }
}

impl ControlLimits {
    /// The limit a refusal for `reason` crossed, or `None` when that limit is unbounded.
    pub const fn limit_for(self, reason: ControlRefusalReason) -> Option<usize> {
        match reason {
            ControlRefusalReason::Budget => self.budget,
            ControlRefusalReason::LineLimit => self.line_limit,
        }
    }
}

/// `budget 4.0 MiB, line limit 16 KiB`, with `all` for an unbounded limit, the word every
/// surface accepts back.
impl std::fmt::Display for ControlLimits {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "budget {}, line limit {}",
            limit_display(self.budget),
            limit_display(self.line_limit)
        )
    }
}

/// One control limit as a person reads it: a size, or `all` when unbounded.
pub(crate) fn limit_display(limit: Option<usize>) -> String {
    limit.map_or_else(
        || "all".to_string(),
        |bytes| crate::report_format::human_bytes(u64::try_from(bytes).unwrap_or(u64::MAX)),
    )
}

/// Conservative retained charge for one key, identity, and matcher shell.
pub(crate) const CONTROL_SOURCE_OVERHEAD: usize = 64;

/// Stable, non-sensitive identity of one retained control source.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ControlIdentity {
    /// Exact source length.
    pub bytes: u64,
    /// Stable FNV-1a digest of the source bytes.
    pub fingerprint: u64,
}

/// One distinct control content, parsed once and shared by every directory holding it.
#[derive(Debug)]
struct SharedContent {
    bytes: Vec<u8>,
    identity: ControlIdentity,
    matcher: Gitignore,
    /// Charge for the exact bytes and the parsed matcher, paid once per distinct content.
    content_cost: usize,
}

/// A distinct content and how many directories of one table hold it.
///
/// The count belongs to the table, not to the `Arc`: a projected clone of a table shares
/// every content with it, so the reference count cannot say which holders one table has.
#[derive(Clone, Debug)]
struct Holding {
    content: Arc<SharedContent>,
    holders: usize,
}

/// Which of the [`ControlLimits`] refused a control source instead of applying its rules.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ControlRefusalReason {
    /// Retaining the source would have taken the table past [`ControlLimits::budget`].
    Budget,
    /// A line of the source is longer than [`ControlLimits::line_limit`].
    LineLimit,
}

impl ControlRefusalReason {
    /// The stable name every structured output and binding uses for this reason, which is
    /// also the name of the limit that fired.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Budget => "budget",
            Self::LineLimit => "line_limit",
        }
    }
}

/// What a control table did with one verified control source.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ControlAdmission {
    /// The rules apply. `changed` is false when the directory already retained exactly
    /// these bytes.
    Retained {
        /// Whether the table's retained sources changed.
        changed: bool,
    },
    /// The rules do not apply, and the directory retains no source. The refusal is
    /// recorded, so the table's coverage names it.
    Refused(ControlRefusalReason),
}

/// What one upsert would do to a control table, decided before anything moves.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Verdict {
    /// The directory already retains exactly these bytes.
    Unchanged,
    /// The source applies, leaving the table charged `retained_cost` bytes.
    Admit { retained_cost: usize },
    /// A limit cannot admit the source.
    Refuse(ControlRefusalReason),
}

/// One control file whose rules an index refused, relative to the index root.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct RefusedControl {
    /// The refused `.gitignore`.
    pub path: PathBuf,
    /// Which limit refused it.
    pub reason: ControlRefusalReason,
}

/// Whether an index's ignore classification applies every control file in its scope.
///
/// Sizes and counts never depend on this: a refused control file costs only the
/// ignored and unignored split. Below a refused file that split is not exact in either
/// direction, because the file may have held negations as well as ignore rules.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ControlCoverage {
    /// The index read no control file, so it classifies nothing as ignored or unignored.
    NotObserved,
    /// The index read every control file in its scope and applied the ones it admitted.
    Observed(ControlObservation),
}

/// The control files an observing index applied and refused.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ControlObservation {
    /// The limits the index applied control files under.
    pub limits: ControlLimits,
    /// Control files whose rules apply.
    pub applied: u64,
    /// Accepted rules summed once per governing directory, including repeated sources.
    pub rules: u64,
    /// Control files refused, counted exactly.
    pub refused: u64,
    /// Refused control files in path order, at most [`crate::MAX_RETAINED_ISSUES`] of
    /// them. A list shorter than [`Self::refused`] is truncated.
    pub refusals: Vec<RefusedControl>,
}

impl ControlObservation {
    /// Whether every control file in scope applies, so ignore classification is exact.
    pub const fn is_complete(&self) -> bool {
        self.refused == 0
    }

    /// Whether [`Self::refusals`] names every refused control file.
    pub fn lists_every_refusal(&self) -> bool {
        u64::try_from(self.refusals.len()).is_ok_and(|listed| listed == self.refused)
    }
}

/// Exact `.gitignore` sources and parsed matchers, keyed by governing directory.
///
/// Identical sources are stored and parsed once. A tree of package checkouts repeats a few
/// `.gitignore` files thousands of times, and charging every copy for its bytes and matcher
/// crossed the bound at a fraction of the distinct rules the tree holds (fdu-szkg). Each
/// directory still pays for its own key, so a tree of empty control files cannot evade the
/// bound, and removing the last holder of a content releases the content's charge.
///
/// A source the limits cannot admit is refused, not an error: the table records the
/// refusal and keeps no rules for that directory, and the scan that read it continues
/// (fdu-1onj). A refusal ends when the directory's control file is removed or a later
/// read admits it.
#[derive(Clone, Debug)]
pub struct ControlTable {
    by_directory: BTreeMap<PathBuf, Arc<SharedContent>>,
    /// Distinct contents by identity. A list, because an equal length and FNV-1a digest do
    /// not prove equal bytes, and a collision must never share another source's matcher.
    shared: HashMap<ControlIdentity, Vec<Holding>>,
    /// Every refused source, by governing directory. Kept whole, not bounded like issue
    /// details, so removing a refused file keeps the refused count exact.
    refused: BTreeMap<PathBuf, ControlRefusalReason>,
    limits: ControlLimits,
    source_bytes: usize,
    retained_cost: usize,
}

impl Default for ControlTable {
    fn default() -> Self {
        Self::with_limits(ControlLimits::default())
    }
}

impl ControlTable {
    /// An empty table that refuses sources past either of `limits`.
    pub(crate) fn with_limits(limits: ControlLimits) -> Self {
        Self {
            by_directory: BTreeMap::new(),
            shared: HashMap::new(),
            refused: BTreeMap::new(),
            limits,
            source_bytes: 0,
            retained_cost: 0,
        }
    }

    /// Insert or replace one verified control source.
    ///
    /// `path` names the control file relative to the index root. The source is retained
    /// exactly, while matching state is derived once per distinct content rather than per
    /// directory or per entry. A source the limits cannot admit is refused and recorded,
    /// and a source it replaces is dropped with it: rules no longer on disk must not keep
    /// applying. The line limit is checked first, so a source both limits refuse is
    /// refused for its line.
    ///
    /// # Errors
    ///
    /// [`crate::Error::InvalidControlPath`] when `path` does not name a control file.
    pub fn upsert(&mut self, path: &Path, source: Vec<u8>) -> crate::Result<ControlAdmission> {
        let identity = identity(&source);
        self.upsert_identified(path, source, identity)
    }

    fn upsert_identified(
        &mut self,
        path: &Path,
        source: Vec<u8>,
        identity: ControlIdentity,
    ) -> crate::Result<ControlAdmission> {
        let directory = control_directory(path)?;
        match self.verdict(directory, &source, identity) {
            Verdict::Unchanged => Ok(ControlAdmission::Retained { changed: false }),
            Verdict::Refuse(reason) => {
                crate::counters::bump(|counts| {
                    counts.control_refused = counts.control_refused.saturating_add(1);
                });
                Ok(self.refuse(directory, reason))
            }
            Verdict::Admit { retained_cost } => {
                self.detach(directory);
                self.attach(directory, source, identity);
                self.refused.remove(directory);
                debug_assert_eq!(self.retained_cost, retained_cost);
                Ok(ControlAdmission::Retained { changed: true })
            }
        }
    }

    /// What upserting `source` at `directory` would do, decided before anything moves.
    ///
    /// The decision is a pure function of this table, so a caller can ask whether an
    /// operation would change anything without projecting a copy of the table to find out.
    fn verdict(&self, directory: &Path, source: &[u8], identity: ControlIdentity) -> Verdict {
        if self.by_directory.get(directory).is_some_and(|current| current.bytes == source) {
            return Verdict::Unchanged;
        }
        if self.limits.line_limit.is_some_and(|line_limit| {
            source.split(|byte| *byte == b'\n').any(|line| line.len() > line_limit)
        }) {
            return Verdict::Refuse(ControlRefusalReason::LineLimit);
        }
        let content_charge =
            if self.holding(identity, source).is_some() { 0 } else { content_cost(source) };
        // Saturating: every retained charge is a sum of real allocations, so only a source
        // no budget could admit reaches the ceiling, and an unbounded table never refuses.
        let retained_cost = self
            .retained_cost
            .saturating_sub(self.release_charge(directory))
            .saturating_add(directory_cost(directory))
            .saturating_add(content_charge);
        if self.limits.budget.is_some_and(|budget| retained_cost > budget) {
            return Verdict::Refuse(ControlRefusalReason::Budget);
        }
        Verdict::Admit { retained_cost }
    }

    /// Whether upserting `source` at `path` would leave this table exactly as it is.
    ///
    /// True when the directory already retains those exact bytes, and when it already
    /// records a refusal this source would earn again: refusing an already-refused
    /// directory for the same limit writes the same record. A warm revalidate of a tree
    /// past its budget re-reads every refused file, and this is what tells the index that
    /// those reads change nothing (fdu-hzm5).
    pub(crate) fn upsert_is_inert(&self, path: &Path, source: &[u8]) -> bool {
        let Ok(directory) = control_directory(path) else {
            return false;
        };
        match self.verdict(directory, source, identity(source)) {
            Verdict::Unchanged => true,
            Verdict::Refuse(reason) => self.refused.get(directory) == Some(&reason),
            Verdict::Admit { .. } => false,
        }
    }

    /// Whether removing the control file at `path` would leave this table as it is.
    pub(crate) fn remove_is_inert(&self, path: &Path) -> bool {
        control_directory(path).is_ok_and(|directory| {
            !self.by_directory.contains_key(directory) && !self.refused.contains_key(directory)
        })
    }

    /// Whether any directory at or below `subtree` retains a source or records a refusal.
    ///
    /// What a structural removal of `subtree` would prune, so a batch that removes nothing
    /// the table records leaves it alone.
    pub(crate) fn has_record_at_or_below(&self, subtree: &Path) -> bool {
        has_key_at_or_below(&self.by_directory, subtree)
            || has_key_at_or_below(&self.refused, subtree)
    }

    /// Restore a refusal a snapshot recorded, without the source that was refused.
    pub(crate) fn record_refusal(
        &mut self,
        path: &Path,
        reason: ControlRefusalReason,
    ) -> crate::Result<()> {
        let directory = control_directory(path)?;
        self.refuse(directory, reason);
        Ok(())
    }

    fn refuse(&mut self, directory: &Path, reason: ControlRefusalReason) -> ControlAdmission {
        self.detach(directory);
        self.refused.insert(directory.to_path_buf(), reason);
        ControlAdmission::Refused(reason)
    }

    /// Remove one control source, or the record of its refusal. Missing sources are no-ops.
    ///
    /// # Errors
    ///
    /// [`crate::Error::InvalidControlPath`] when `path` does not name a control file.
    pub fn remove(&mut self, path: &Path) -> crate::Result<bool> {
        let directory = control_directory(path)?;
        let retained = self.detach(directory);
        let refused = self.refused.remove(directory).is_some();
        Ok(retained || refused)
    }

    /// Remove every control file, and every refusal, at or below `subtree`.
    pub(crate) fn remove_subtree(&mut self, subtree: &Path) {
        let directories: Vec<PathBuf> = self
            .by_directory
            .keys()
            .filter(|directory| directory.starts_with(subtree))
            .cloned()
            .collect();
        for directory in directories {
            self.detach(&directory);
        }
        self.refused.retain(|directory, _| !directory.starts_with(subtree));
    }

    /// The holding for exactly `source`, when some directory already retains it.
    fn holding(&self, identity: ControlIdentity, source: &[u8]) -> Option<&Holding> {
        self.shared
            .get(&identity)?
            .iter()
            .find(|holding| holding.content.bytes.as_slice() == source)
    }

    /// The charge that dropping `directory`'s current source would release.
    fn release_charge(&self, directory: &Path) -> usize {
        let Some(content) = self.by_directory.get(directory) else {
            return 0;
        };
        let last_holder = self.shared.get(&content.identity).is_some_and(|holdings| {
            holdings
                .iter()
                .any(|holding| Arc::ptr_eq(&holding.content, content) && holding.holders == 1)
        });
        directory_cost(directory).saturating_add(if last_holder { content.content_cost } else { 0 })
    }

    /// Drop `directory`'s source, releasing its content when this was the last holder.
    fn detach(&mut self, directory: &Path) -> bool {
        let Some(content) = self.by_directory.remove(directory) else {
            return false;
        };
        let holdings = self.shared.get_mut(&content.identity).expect("a retained content is held");
        let position = holdings
            .iter()
            .position(|holding| Arc::ptr_eq(&holding.content, &content))
            .expect("a retained content is listed under its own identity");
        holdings[position].holders -= 1;
        if holdings[position].holders == 0 {
            holdings.swap_remove(position);
            self.retained_cost -= content.content_cost;
            if holdings.is_empty() {
                self.shared.remove(&content.identity);
            }
        }
        self.retained_cost -= directory_cost(directory);
        self.source_bytes -= content.bytes.len();
        true
    }

    /// Hold `source` at `directory`, sharing an identical retained content when one exists.
    fn attach(&mut self, directory: &Path, source: Vec<u8>, identity: ControlIdentity) {
        let holdings = self.shared.entry(identity).or_default();
        let content = if let Some(holding) =
            holdings.iter_mut().find(|holding| holding.content.bytes == source)
        {
            holding.holders += 1;
            crate::counters::bump(|counts| {
                counts.control_sources_shared = counts.control_sources_shared.saturating_add(1);
            });
            Arc::clone(&holding.content)
        } else {
            let content_cost = content_cost(&source);
            let matcher = Gitignore::parse(&source);
            let content =
                Arc::new(SharedContent { bytes: source, identity, matcher, content_cost });
            holdings.push(Holding { content: Arc::clone(&content), holders: 1 });
            self.retained_cost += content_cost;
            content
        };
        self.retained_cost += directory_cost(directory);
        self.source_bytes += content.bytes.len();
        self.by_directory.insert(directory.to_path_buf(), content);
    }

    /// Matcher view for one retained path.
    pub fn matcher_for<'a>(&'a self, path: &'a Path) -> ControlMatcher<'a> {
        ControlMatcher { table: self, path }
    }

    /// The controls governing every child of `directory`, resolved once for all of them.
    ///
    /// [`ControlMatcher::is_ignored`] looks each ancestor up in the table for every entry
    /// it classifies. A listing's children share those ancestors, so a caller classifying
    /// a whole listing resolves them here once and matches each child against the chain
    /// (H163). The chain owns its sources, so the table may change while it is held; it
    /// then answers for the table as it was when resolved.
    ///
    /// `directory` must be a normalized relative path, as every walked or event path is:
    /// the chain counts one component per ancestor, which a `..` would break.
    pub(crate) fn chain_for(&self, directory: &Path) -> ControlChain {
        debug_assert!(
            directory.components().all(|component| matches!(component, Component::Normal(_))),
            "control chains are resolved for normalized relative directories: {}",
            directory.display()
        );
        let mut governing = Vec::new();
        if !self.by_directory.is_empty() {
            let depth = gitignore::with_components(directory, None, |components| components.len());
            for (up, ancestor) in directory.ancestors().enumerate() {
                if let Some(source) = self.by_directory.get(ancestor) {
                    governing.push((depth.saturating_sub(up), Arc::clone(source)));
                }
            }
        }
        ControlChain { governing }
    }

    /// The chain for `directory`'s children, derived from `above`, the chain resolved for
    /// its parent's children, instead of resolved from the table again (H175).
    ///
    /// It is [`Self::chain_for`]'s answer whenever no control of a directory above
    /// `directory` has changed since `above` was resolved: `above` then names every control
    /// above `directory`, deepest first, and only `directory`'s own can be new, which goes
    /// first. A parent-first build meets that condition, because each listing applies only
    /// its own directory's control, before any of its children is listed. It costs one
    /// lookup, and an allocation only when `directory` holds a control; a caller that
    /// knows `directory` holds none, because its listing carried no control, can share
    /// `above` without asking.
    pub(crate) fn chain_below(
        &self,
        above: &Arc<ControlChain>,
        directory: &Path,
    ) -> Arc<ControlChain> {
        let Some(source) = self.by_directory.get(directory) else {
            return Arc::clone(above);
        };
        let depth = gitignore::with_components(directory, None, |components| components.len());
        let mut governing = Vec::with_capacity(above.governing.len() + 1);
        governing.push((depth, Arc::clone(source)));
        governing.extend(above.governing.iter().cloned());
        Arc::new(ControlChain { governing })
    }

    /// Evaluate complete ignore semantics without relying on retained parent facts.
    ///
    /// The index hot path uses [`ControlMatcher::is_ignored`] with the parent's stored
    /// classification. This standalone form evaluates each directory prefix so callers
    /// and tests receive the same answer even without an index entry in hand.
    pub fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        let components: Vec<_> = path.components().collect();
        let mut current = PathBuf::new();
        let mut parent_ignored = false;
        for (position, component) in components.iter().enumerate() {
            current.push(component.as_os_str());
            if parent_ignored {
                return true;
            }
            let current_is_dir = position + 1 < components.len() || is_dir;
            parent_ignored = self.matcher_for(&current).is_ignored(current_is_dir);
        }
        parent_ignored
    }

    /// Relative subtree whose classification may move when `path` changes.
    pub fn affected_subtree(path: &Path) -> crate::Result<PathBuf> {
        Ok(control_directory(path)?.to_path_buf())
    }

    /// Stable identities changed between two complete table states.
    pub(crate) fn changes_from(
        &self,
        previous: &Self,
    ) -> Vec<(PathBuf, Option<ControlIdentity>, Option<ControlIdentity>)> {
        let directories: BTreeSet<&Path> = previous
            .by_directory
            .keys()
            .chain(self.by_directory.keys())
            .map(PathBuf::as_path)
            .collect();
        directories
            .into_iter()
            .filter_map(|directory| {
                let before = previous.by_directory.get(directory);
                let after = self.by_directory.get(directory);
                let changed = match (before, after) {
                    (Some(before), Some(after)) => {
                        !Arc::ptr_eq(before, after) && before.bytes != after.bytes
                    }
                    (None, None) => false,
                    (Some(_), None) | (None, Some(_)) => true,
                };
                changed.then(|| {
                    (
                        control_path(directory),
                        before.map(|source| source.identity),
                        after.map(|source| source.identity),
                    )
                })
            })
            .collect()
    }

    /// Refusals recorded or lifted between two complete table states.
    pub(crate) fn refusal_changes_from(
        &self,
        previous: &Self,
    ) -> Vec<(PathBuf, Option<ControlRefusalReason>, Option<ControlRefusalReason>)> {
        let directories: BTreeSet<&Path> =
            previous.refused.keys().chain(self.refused.keys()).map(PathBuf::as_path).collect();
        directories
            .into_iter()
            .filter_map(|directory| {
                let before = previous.refused.get(directory).copied();
                let after = self.refused.get(directory).copied();
                (before != after).then(|| (control_path(directory), before, after))
            })
            .collect()
    }

    /// Exact sources in deterministic governing-directory order.
    pub(crate) fn sources(&self) -> impl ExactSizeIterator<Item = (PathBuf, &[u8])> {
        self.by_directory
            .iter()
            .map(|(directory, source)| (control_path(directory), source.bytes.as_slice()))
    }

    /// Every refused control file and its reason, in governing-directory order.
    pub fn refusals(&self) -> impl ExactSizeIterator<Item = RefusedControl> + '_ {
        self.refused.iter().map(|(directory, reason)| RefusedControl {
            path: control_path(directory),
            reason: *reason,
        })
    }

    /// Whether every control source that could govern `path` was admitted.
    ///
    /// A refused source may contain ignore or negation rules, so its descendants have
    /// unknown classification even if the admitted rules currently say otherwise.
    pub(crate) fn classification_known(&self, path: &Path) -> bool {
        !path
            .parent()
            .into_iter()
            .flat_map(Path::ancestors)
            .any(|directory| self.refused.contains_key(directory))
    }

    /// Number of refused control files.
    pub fn refused_len(&self) -> usize {
        self.refused.len()
    }

    /// The limits this table admits sources under.
    pub const fn limits(&self) -> ControlLimits {
        self.limits
    }

    /// This table's coverage, listing at most [`crate::MAX_RETAINED_ISSUES`] refusals.
    pub fn observation(&self) -> ControlObservation {
        ControlObservation {
            limits: self.limits,
            applied: u64::try_from(self.len()).unwrap_or(u64::MAX),
            rules: self
                .by_directory
                .values()
                .fold(0_u64, |total, content| total.saturating_add(content.matcher.rule_count())),
            refused: u64::try_from(self.refused_len()).unwrap_or(u64::MAX),
            refusals: self.refusals().take(crate::MAX_RETAINED_ISSUES).collect(),
        }
    }

    /// Exact retained source bytes across the whole table.
    pub const fn source_bytes(&self) -> usize {
        self.source_bytes
    }

    /// Bounded retained charge for the complete table.
    pub const fn retained_cost(&self) -> usize {
        self.retained_cost
    }

    /// Whether `path` already retains exactly `source`.
    pub fn source_is(&self, path: &Path, source: &[u8]) -> bool {
        control_directory(path)
            .ok()
            .and_then(|directory| self.by_directory.get(directory))
            .is_some_and(|current| current.bytes == source)
    }

    /// Whether the table holds a record at `path`: a retained source or a refusal.
    ///
    /// Reconciliation asks this to decide whether a control file missing from a listing
    /// needs a removal, and a refused file that disappears must lift its refusal.
    pub(crate) fn contains(&self, path: &Path) -> bool {
        control_directory(path).ok().is_some_and(|directory| {
            self.by_directory.contains_key(directory) || self.refused.contains_key(directory)
        })
    }

    /// Number of retained control files.
    pub fn len(&self) -> usize {
        self.by_directory.len()
    }

    /// Whether no control file is retained. A table may still record refusals.
    pub fn is_empty(&self) -> bool {
        self.by_directory.is_empty()
    }

    /// Whether the table records nothing at all: no retained source and no refusal.
    pub(crate) fn is_vacant(&self) -> bool {
        self.by_directory.is_empty() && self.refused.is_empty()
    }
}

/// A path-bound view over the controls that may govern it.
pub struct ControlMatcher<'a> {
    table: &'a ControlTable,
    path: &'a Path,
}

impl ControlMatcher<'_> {
    /// Decide this path assuming its retained parent is not ignored.
    ///
    /// A caller with retained facts already knows the parent's effective classification,
    /// so it can stop immediately when that parent is ignored. Otherwise every control
    /// directory on this path is active and the deepest matching opinion wins.
    pub fn is_ignored(&self, is_dir: bool) -> bool {
        // The empty table answers without collecting the ancestor list. Every entry of
        // a scan that observes no control state asks this question exactly once, so the
        // allocation below would be the last per-entry cost of a machinery the scan
        // opted out of (fdu-etfj).
        if self.table.by_directory.is_empty() {
            return false;
        }
        // A deeper control source overrides every matching ancestor. Walk from the
        // immediate directory toward the root and return the first opinion instead of
        // allocating and reversing an ancestor vector for every classified entry.
        for directory in self.path.parent().into_iter().flat_map(Path::ancestors) {
            let Some(source) = self.table.by_directory.get(directory) else {
                continue;
            };
            let relative = self.path.strip_prefix(directory).unwrap_or(self.path);
            if let Some(ignored) = source.matcher.matches(relative, is_dir) {
                return ignored;
            }
        }
        false
    }
}

/// The controls that govern one directory's children, deepest first, with how many of the
/// directory's path components lead to each one's own directory.
#[derive(Clone, Debug, Default)]
pub(crate) struct ControlChain {
    governing: Vec<(usize, Arc<SharedContent>)>,
}

impl ControlChain {
    /// Whether no control governs the directory, so none of its children is ignored.
    pub(crate) fn is_empty(&self) -> bool {
        self.governing.is_empty()
    }

    /// Whether two chains name the same controls, as the same retained contents, at the
    /// same depths and in the same order.
    pub(crate) fn same_as(&self, other: &Self) -> bool {
        self.governing.len() == other.governing.len()
            && self
                .governing
                .iter()
                .zip(&other.governing)
                .all(|(left, right)| left.0 == right.0 && Arc::ptr_eq(&left.1, &right.1))
    }

    /// Decide the child `name` of `directory`, the directory this chain was resolved for,
    /// assuming that directory is not ignored.
    #[cfg(test)]
    pub(crate) fn is_ignored(&self, directory: &Path, name: &[u8], is_dir: bool) -> bool {
        with_directory_components(directory, |components| {
            self.is_ignored_within(components, name, is_dir)
        })
    }

    /// Decide the child `name` of the directory this chain was resolved for, given that
    /// directory's normal components, assuming that directory is not ignored.
    ///
    /// The answer is [`ControlMatcher::is_ignored`]'s for `directory/name`: the deepest
    /// control with an opinion wins, each matching the path relative to its own directory.
    /// A caller classifying a whole listing splits the directory once, with
    /// [`with_directory_components`], and the name is hashed once here for every control
    /// that governs it (H171).
    pub(crate) fn is_ignored_within(&self, directory: &[&[u8]], name: &[u8], is_dir: bool) -> bool {
        if self.governing.is_empty() {
            return false;
        }
        let name = gitignore::Name::new(name);
        let mut tally = gitignore::Tally::default();
        let ignored = self
            .governing
            .iter()
            .find_map(|(leading, source)| {
                let relative = directory.get(*leading..).unwrap_or_default();
                source.matcher.decide(relative, &name, is_dir, &mut tally)
            })
            .unwrap_or(false);
        tally.record();
        ignored
    }
}

/// Call `each` with the normal components of `directory`, split once for every child of
/// one listing, without a heap allocation for a path of ordinary depth.
pub(crate) fn with_directory_components<R>(
    directory: &Path,
    each: impl FnOnce(&[&[u8]]) -> R,
) -> R {
    gitignore::with_components(directory, None, each)
}

/// A directory's normal components, copied once, for a caller that keeps them across
/// calls and classifies each child of the directory as it arrives.
///
/// [`Path::components`] parses the whole path each time it is asked, which costs a
/// deep entry more than matching its name does once the rules are indexed (H171).
#[derive(Debug)]
pub(crate) struct SplitDirectory {
    bytes: Vec<u8>,
    /// Where each component ends in `bytes`.
    ends: Vec<usize>,
}

impl SplitDirectory {
    pub(crate) fn new(directory: &Path) -> Self {
        with_directory_components(directory, |components| {
            let mut split = Self {
                bytes: Vec::with_capacity(components.iter().map(|component| component.len()).sum()),
                ends: Vec::with_capacity(components.len()),
            };
            for component in components {
                split.bytes.extend_from_slice(component);
                split.ends.push(split.bytes.len());
            }
            split
        })
    }

    /// Call `each` with the components, without a heap allocation for a path of ordinary
    /// depth.
    pub(crate) fn with_components<R>(&self, each: impl FnOnce(&[&[u8]]) -> R) -> R {
        let components = self.ends.iter().scan(0, |start, &end| {
            let component = &self.bytes[*start..end];
            *start = end;
            Some(component)
        });
        gitignore::with_collected(components, each)
    }
}

/// Whether any key of `directories` is `subtree` or lies below it.
///
/// One lookup rather than a scan: [`Path`] orders component by component, so every
/// descendant of `subtree` sorts immediately after it and before any other key, and the
/// first key at or after `subtree` decides.
fn has_key_at_or_below<V>(directories: &BTreeMap<PathBuf, V>, subtree: &Path) -> bool {
    directories
        .range::<Path, _>((std::ops::Bound::Included(subtree), std::ops::Bound::Unbounded))
        .next()
        .is_some_and(|(directory, _)| directory.starts_with(subtree))
}

/// Whether a relative path names the fixed control file: the canonical path every control
/// operation, and the table, name a directory's rules by.
///
/// A directory's rules may be held by a `.GITIGNORE` on a case-insensitive volume, but
/// they are still recorded under `<dir>/.gitignore` (see the module documentation), so a
/// control operation naming any other spelling is malformed.
pub fn is_control_file(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name == CONTROL_FILE_NAME)
}

/// How a listed name relates to its directory's control file.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ControlSpelling {
    /// Exactly [`CONTROL_FILE_NAME`]: the entry a lookup of the directory's control opens
    /// on every filesystem, read through its own path.
    Exact,
    /// [`CONTROL_FILE_NAME`] in another ASCII case, such as `.GITIGNORE`: the directory's
    /// control only where the filesystem resolves `.gitignore` to it, which a lookup of the
    /// canonical path decides.
    Variant,
}

/// Whether `name` spells the control file name, and how.
///
/// Every listed entry asks this, so it is a length test and, for the rare ten-byte name,
/// an ASCII case-insensitive comparison: no allocation and no system call.
pub(crate) fn control_spelling(name: &std::ffi::OsStr) -> Option<ControlSpelling> {
    let bytes = name.as_encoded_bytes();
    let exact = CONTROL_FILE_NAME.as_bytes();
    if bytes.len() != exact.len() {
        None
    } else if bytes == exact {
        Some(ControlSpelling::Exact)
    } else if bytes.eq_ignore_ascii_case(exact) {
        Some(ControlSpelling::Variant)
    } else {
        None
    }
}

/// The spelling of the control name a relative path's last component uses, if any.
pub(crate) fn path_control_spelling(path: &Path) -> Option<ControlSpelling> {
    path.file_name().and_then(control_spelling)
}

/// The canonical control path of the directory `path` sits in: `<parent>/.gitignore`.
pub(crate) fn sibling_control_path(path: &Path) -> PathBuf {
    control_path(path.parent().unwrap_or_else(|| Path::new("")))
}

/// The control file a walk error under `root` names, relative to it, when there is one,
/// by its canonical path.
///
/// A control file the walk could not read leaves the rules it holds unknown, so the
/// ignored split it governs cannot be verified. That includes a listed case variant whose
/// own metadata could not be read: on a case-insensitive volume it may hold the rules, and
/// nothing looked the canonical path up. The index and the transient summary both ask this
/// of the same normalized walk errors, so they withhold the same shares.
pub(crate) fn unreadable_control(root: &Path, error: &crate::Error) -> Option<PathBuf> {
    let crate::Error::Io { .. } = error else {
        return None;
    };
    crate::Issue::from_error_under(root, error).path.and_then(|path| governing_control(&path))
}

/// The canonical control path of the directory whose control an entry at `path` may hold,
/// when its name spells the control name in any case.
pub(crate) fn governing_control(path: &Path) -> Option<PathBuf> {
    path_control_spelling(path).map(|_| sibling_control_path(path))
}

fn control_directory(path: &Path) -> crate::Result<&Path> {
    if !is_control_file(path) {
        return Err(crate::Error::InvalidControlPath(path.to_path_buf()));
    }
    Ok(path.parent().unwrap_or_else(|| Path::new("")))
}

fn control_path(directory: &Path) -> PathBuf {
    directory.join(CONTROL_FILE_NAME)
}

fn identity(bytes: &[u8]) -> ControlIdentity {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x100_0000_01b3;

    let mut fingerprint = FNV_OFFSET_BASIS;
    for byte in bytes {
        fingerprint ^= u64::from(*byte);
        fingerprint = fingerprint.wrapping_mul(FNV_PRIME);
    }
    ControlIdentity { bytes: u64::try_from(bytes.len()).unwrap_or(u64::MAX), fingerprint }
}

/// Charge for one directory's key and identity, whatever content it holds.
fn directory_cost(directory: &Path) -> usize {
    CONTROL_SOURCE_OVERHEAD.saturating_add(directory.as_os_str().as_encoded_bytes().len())
}

/// Charge for one distinct content: its exact bytes and parsed glob bytes, plus the
/// matcher's per-pattern and per-segment shells.
fn content_cost(source: &[u8]) -> usize {
    let (newlines, segment_shells) = source.iter().fold((0usize, 0usize), |counts, byte| {
        (counts.0 + usize::from(*byte == b'\n'), counts.1 + usize::from(*byte == b'/'))
    });
    let pattern_shells = newlines.saturating_add(1);
    source
        .len()
        .saturating_mul(2)
        .saturating_add(pattern_shells.saturating_mul(64))
        .saturating_add(segment_shells.saturating_mul(24))
}

/// Charge for one source whose content no other directory holds.
#[cfg(test)]
fn retained_source_cost(directory: &Path, source: &[u8]) -> usize {
    directory_cost(directory).saturating_add(content_cost(source))
}

#[cfg(test)]
pub(crate) fn source_at_test_limit() -> Vec<u8> {
    let mut source = Vec::new();
    loop {
        let previous_len = source.len();
        source.extend(std::iter::repeat_n(b'a', DEFAULT_CONTROL_LINE_LIMIT));
        source.push(b'\n');
        if retained_source_cost(Path::new(""), &source) > DEFAULT_CONTROL_BUDGET {
            source.truncate(previous_len);
            break;
        }
    }
    let remaining = DEFAULT_CONTROL_BUDGET - retained_source_cost(Path::new(""), &source);
    source.extend(std::iter::repeat_n(b'a', (remaining / 2).min(DEFAULT_CONTROL_LINE_LIMIT)));
    assert_eq!(retained_source_cost(Path::new(""), &source), DEFAULT_CONTROL_BUDGET);
    source
}

#[cfg(test)]
impl ControlTable {
    /// Recompute every charge and holder count from the directories alone.
    fn assert_consistent(&self) {
        let mut distinct: Vec<&Arc<SharedContent>> = Vec::new();
        let mut directory_charges = 0;
        let mut source_bytes = 0;
        for (directory, content) in &self.by_directory {
            directory_charges += directory_cost(directory);
            source_bytes += content.bytes.len();
            if !distinct.iter().any(|seen| Arc::ptr_eq(seen, content)) {
                distinct.push(content);
            }
        }
        let content_charges: usize = distinct.iter().map(|content| content.content_cost).sum();
        assert_eq!(self.retained_cost, directory_charges + content_charges, "retained cost");
        assert_eq!(self.source_bytes, source_bytes, "source bytes");
        let holdings: usize = self.shared.values().map(Vec::len).sum();
        assert_eq!(holdings, distinct.len(), "one holding per distinct content");
        for (identity, holdings) in &self.shared {
            assert!(!holdings.is_empty(), "no empty identity list survives");
            for holding in holdings {
                assert_eq!(holding.content.identity, *identity);
                let holders = self
                    .by_directory
                    .values()
                    .filter(|content| Arc::ptr_eq(content, &holding.content))
                    .count();
                assert_eq!(holding.holders, holders, "holder count");
            }
            for (index, left) in holdings.iter().enumerate() {
                for right in &holdings[index + 1..] {
                    assert_ne!(left.content.bytes, right.content.bytes, "equal bytes are shared");
                }
            }
        }
        assert!(
            self.refused.keys().all(|directory| !self.by_directory.contains_key(directory)),
            "a refused directory retains no source"
        );
        assert!(
            self.limits.budget.is_none_or(|budget| self.retained_cost <= budget),
            "within budget"
        );
        assert!(
            self.limits.line_limit.is_none_or(|line_limit| {
                distinct.iter().all(|content| {
                    content.bytes.split(|byte| *byte == b'\n').all(|line| line.len() <= line_limit)
                })
            }),
            "within the line limit"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic `SplitMix64`, so a failing sequence replays from its printed seed.
    struct SplitMix(u64);

    impl SplitMix {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut value = self.0;
            value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            value ^ (value >> 31)
        }

        fn below(&mut self, bound: usize) -> usize {
            usize::try_from(self.next() % u64::try_from(bound).expect("small bound")).expect("fits")
        }
    }

    /// Every directory's last operation decides whether it holds a record, whatever the
    /// limits decided: an upsert leaves exactly one of a source or a refusal, and a removal
    /// leaves neither. Charges and holder counts are recomputed after every step, under
    /// each combination of a bounded or unbounded budget and line limit.
    #[test]
    fn rule_totals_count_accepted_patterns_per_governing_location() {
        let mut table = ControlTable::default();
        let source = b"# comment\n\n*.log\n!important.log\n*.log\n[bad\n";
        table.upsert(Path::new(".gitignore"), source.to_vec()).expect("root control");
        table.upsert(Path::new("nested/.gitignore"), source.to_vec()).expect("nested control");
        assert_eq!(table.observation().applied, 2);
        assert_eq!(table.observation().rules, 6);
        table
            .upsert(Path::new("nested/.gitignore"), b"# empty\n".to_vec())
            .expect("replacement control");
        assert_eq!(table.observation().applied, 2);
        assert_eq!(table.observation().rules, 3);
    }

    #[test]
    fn charges_and_refusals_stay_exact_through_random_upserts_and_removals() {
        const DIRECTORIES: [&str; 6] = ["", "a", "a/b", "b", "b/c/d", "c"];
        let long_line = [vec![b'x'; DEFAULT_CONTROL_LINE_LIMIT + 1], b"\n".to_vec()].concat();
        let large = b"pattern/\n".repeat(40);
        let contents: [&[u8]; 6] =
            [b"*.log\n", b"target/\n", b"!keep\n*.tmp\n", b"", &large, &long_line];
        // Room for a few small sources and one large one, so the budget refuses often.
        let budget = 2 * retained_source_cost(Path::new("b/c/d"), &large);
        for seed in 0..64 {
            let mut random = SplitMix(seed);
            let limits = ControlLimits {
                budget: (seed % 2 == 0).then_some(budget),
                line_limit: (seed % 4 < 2).then_some(DEFAULT_CONTROL_LINE_LIMIT),
            };
            let mut table = ControlTable::with_limits(limits);
            let mut holds_record: BTreeMap<&Path, bool> = BTreeMap::new();
            for step in 0..300 {
                let directory = Path::new(DIRECTORIES[random.below(DIRECTORIES.len())]);
                let path = directory.join(CONTROL_FILE_NAME);
                match random.below(8) {
                    0 => {
                        table.remove_subtree(directory);
                        for (held, record) in &mut holds_record {
                            if held.starts_with(directory) {
                                *record = false;
                            }
                        }
                    }
                    1 | 2 => {
                        table.remove(&path).expect("control path");
                        holds_record.insert(directory, false);
                    }
                    _ => {
                        let content = contents[random.below(contents.len())].to_vec();
                        let admission = table.upsert(&path, content.clone()).expect("control path");
                        if content == long_line && limits.line_limit.is_some() {
                            assert_eq!(
                                admission,
                                ControlAdmission::Refused(ControlRefusalReason::LineLimit)
                            );
                        }
                        if limits.budget.is_none() && limits.line_limit.is_none() {
                            assert!(
                                matches!(admission, ControlAdmission::Retained { .. }),
                                "an unbounded table refuses nothing"
                            );
                        }
                        holds_record.insert(directory, true);
                    }
                }
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    table.assert_consistent();
                    for (directory, record) in &holds_record {
                        let path = directory.join(CONTROL_FILE_NAME);
                        assert_eq!(table.contains(&path), *record, "{}", path.display());
                    }
                    let records = holds_record.values().filter(|record| **record).count();
                    assert_eq!(table.len() + table.refused_len(), records);
                }))
                .unwrap_or_else(|_| panic!("seed {seed}, step {step}: inconsistent table"));
            }
            for directory in DIRECTORIES {
                table.remove(&Path::new(directory).join(CONTROL_FILE_NAME)).expect("control path");
            }
            assert_eq!(table.retained_cost(), 0, "seed {seed}");
            assert_eq!(table.source_bytes(), 0, "seed {seed}");
            assert!(table.shared.is_empty(), "seed {seed}");
            assert!(table.is_vacant(), "seed {seed}");
        }
    }

    #[test]
    fn a_resolved_chain_answers_as_the_per_entry_matcher_does() {
        let mut table = ControlTable::default();
        for (path, source) in [
            (".gitignore", &b"*.log\n/build/\n!keep.log\nsub/*.tmp\n"[..]),
            ("a/.gitignore", b"!*.log\n*.o\n/deep/**\n"),
            ("a/b/.gitignore", b"*.log\n!x.o\n"),
            ("c/.gitignore", b"# comment only\n"),
        ] {
            table.upsert(Path::new(path), source.to_vec()).expect("fixture control");
        }
        let directories =
            ["", "a", "a/b", "a/b/c", "a/deep", "a/deep/er", "build", "c", "c/sub", "sub", "x/y"];
        let names = ["x.log", "keep.log", "x.o", "y.o", "build", "deep", "t.tmp", "plain"];
        for directory in directories {
            let chain = table.chain_for(Path::new(directory));
            for name in names {
                for is_dir in [false, true] {
                    let path = Path::new(directory).join(name);
                    assert_eq!(
                        chain.is_ignored(Path::new(directory), name.as_bytes(), is_dir),
                        table.matcher_for(&path).is_ignored(is_dir),
                        "{} (dir {is_dir})",
                        path.display()
                    );
                }
            }
        }
        assert!(!ControlTable::default().chain_for(Path::new("a")).is_ignored(
            Path::new("a"),
            b"x.log",
            false
        ));
    }

    /// A chain derived parent-first, each directory adding only its own control, names the
    /// same controls as the chain the table resolves, whatever each directory's control
    /// did: applied, shared with another directory, refused for the budget or for a line,
    /// removed, or never listed (H175).
    #[test]
    fn chains_derived_parent_first_are_the_ones_the_table_resolves() {
        let long_line = [vec![b'x'; DEFAULT_CONTROL_LINE_LIMIT + 1], b"\n".to_vec()].concat();
        let large = b"pattern/\n".repeat(40);
        let contents: [&[u8]; 6] =
            [b"*.log\n", b"!keep\n*.tmp\n", b"", b"/a/x.log\n!*.log\n", &large, &long_line];
        let budget = 3 * retained_source_cost(Path::new("a/b/c"), &large);
        for seed in 0..200 {
            let mut random = SplitMix(seed);
            let mut table = ControlTable::with_limits(ControlLimits {
                budget: Some(budget),
                ..ControlLimits::default()
            });
            let mut listings = std::collections::VecDeque::from([(
                PathBuf::new(),
                Arc::<ControlChain>::default(),
            )]);
            while let Some((directory, above)) = listings.pop_front() {
                let control = directory.join(CONTROL_FILE_NAME);
                let listed = match random.below(4) {
                    0 | 1 => {
                        let content = contents[random.below(contents.len())].to_vec();
                        table.upsert(&control, content).expect("control path");
                        true
                    }
                    2 => {
                        table.remove(&control).expect("control path");
                        true
                    }
                    _ => false,
                };
                let chain = if listed { table.chain_below(&above, &directory) } else { above };
                let resolved = table.chain_for(&directory);
                assert!(chain.same_as(&resolved), "seed {seed}: {}", directory.display());
                for name in ["x.log", "keep", "x.tmp", "pattern", "a"] {
                    for is_dir in [false, true] {
                        assert_eq!(
                            chain.is_ignored(&directory, name.as_bytes(), is_dir),
                            resolved.is_ignored(&directory, name.as_bytes(), is_dir),
                            "seed {seed}: {}/{name}",
                            directory.display()
                        );
                    }
                }
                if directory.components().count() < 4 {
                    for name in ["a", "b", "c"] {
                        if random.below(3) > 0 {
                            listings.push_back((directory.join(name), Arc::clone(&chain)));
                        }
                    }
                }
            }
            table.assert_consistent();
        }
    }

    #[test]
    fn a_chain_agrees_past_the_inline_buffers_and_beside_unrelated_controls() {
        let mut table = ControlTable::default();
        table.upsert(Path::new("z/.gitignore"), b"*.log\n".to_vec()).expect("unrelated");
        let unrelated = table.chain_for(Path::new("a/b"));
        assert!(!unrelated.is_ignored(Path::new("a/b"), b"x.log", false));
        assert!(!table.matcher_for(Path::new("a/b/x.log")).is_ignored(false));

        // A control 33 directories down, and directories of 31 to 34 components, cross the
        // 32-component inline buffer both in the chain's key and in the matched path.
        let deep: PathBuf = (0..33).map(|at| format!("d{at}")).collect();
        table.upsert(Path::new(".gitignore"), b"**/x.log\n/d0/**/y.log\n".to_vec()).expect("root");
        table.upsert(&deep.join(".gitignore"), b"!x.log\n*.tmp\n".to_vec()).expect("deep");
        for depth in [31usize, 32, 33, 34] {
            let directory: PathBuf = (0..depth).map(|at| format!("d{at}")).collect();
            let chain = table.chain_for(&directory);
            for name in ["x.log", "y.log", "z.tmp", "plain"] {
                let path = directory.join(name);
                assert_eq!(
                    chain.is_ignored(&directory, name.as_bytes(), false),
                    table.matcher_for(&path).is_ignored(false),
                    "{name} at depth {depth}"
                );
            }
        }
    }

    /// The matching counters see each lookup, each lookup that found a rule, and each rule
    /// whose glob had to run, on the thread that classified.
    #[test]
    fn matching_counts_lookups_hits_and_rules_tested() {
        let _serial = crate::counters::test_serial();
        crate::counters::enable(true);
        crate::counters::test_thread_reset();
        let mut table = ControlTable::default();
        table
            .upsert(Path::new(".gitignore"), b"*.o\nMakefile\n/build\n*.c.[01]*\n".to_vec())
            .expect("root control");
        let chain = table.chain_for(Path::new(""));
        // A name lookup, and an extension lookup that finds `*.o`; `/build` and the
        // wildcard rule are ruled out by their length and literal checks.
        assert!(chain.is_ignored_within(&[], b"main.o", false));
        // Two lookups that find nothing, and the wildcard rule tested in full.
        assert!(chain.is_ignored_within(&[], b"a.c.0x", false));
        let counts = crate::counters::test_thread_snapshot();
        crate::counters::enable(false);
        assert_eq!(
            (counts.ignore_bucket_probes, counts.ignore_bucket_hits, counts.ignore_patterns_tested),
            (4, 1, 1)
        );
    }

    #[test]
    fn a_fingerprint_collision_never_shares_a_matcher() {
        let collision = ControlIdentity { bytes: 6, fingerprint: 7 };
        let mut table = ControlTable::default();
        table
            .upsert_identified(Path::new("a/.gitignore"), b"*.log\n".to_vec(), collision)
            .expect("first");
        table
            .upsert_identified(Path::new("b/.gitignore"), b"*.tmp\n".to_vec(), collision)
            .expect("second");
        table.assert_consistent();

        assert_eq!(table.shared[&collision].len(), 2);
        assert!(table.is_ignored(Path::new("a/x.log"), false));
        assert!(!table.is_ignored(Path::new("a/x.tmp"), false));
        assert!(table.is_ignored(Path::new("b/x.tmp"), false));
        assert!(!table.is_ignored(Path::new("b/x.log"), false));
        assert_eq!(
            table.retained_cost(),
            retained_source_cost(Path::new("a"), b"*.log\n")
                + retained_source_cost(Path::new("b"), b"*.tmp\n")
        );

        table.remove(Path::new("a/.gitignore")).expect("remove");
        table.assert_consistent();
        assert!(table.is_ignored(Path::new("b/x.tmp"), false));
    }

    const CHANGED: ControlAdmission = ControlAdmission::Retained { changed: true };
    const UNCHANGED: ControlAdmission = ControlAdmission::Retained { changed: false };
    const OVER_BUDGET: ControlAdmission = ControlAdmission::Refused(ControlRefusalReason::Budget);
    const OVER_LINE_LIMIT: ControlAdmission =
        ControlAdmission::Refused(ControlRefusalReason::LineLimit);

    /// A table under `budget` and the default line limit.
    fn budgeted(budget: Option<usize>) -> ControlTable {
        ControlTable::with_limits(ControlLimits { budget, ..ControlLimits::default() })
    }

    #[test]
    fn replacing_the_last_holder_releases_its_content_for_the_bound() {
        let mut table = ControlTable::default();
        let first = source_at_test_limit();
        assert_eq!(
            table.upsert(Path::new(".gitignore"), first.clone()).expect("control path"),
            CHANGED
        );
        // The same content elsewhere costs only a key, which still crosses a full table.
        assert_eq!(table.upsert(Path::new("copy/.gitignore"), first).expect("path"), OVER_BUDGET);
        assert_eq!(
            table.upsert(Path::new(".gitignore"), b"small\n".to_vec()).expect("control path"),
            CHANGED
        );
        table.assert_consistent();
        assert_eq!(table.retained_cost(), retained_source_cost(Path::new(""), b"small\n"));
    }

    #[test]
    fn creation_edit_and_last_removal_are_exact() {
        let mut table = ControlTable::default();
        assert_eq!(
            table.upsert(Path::new(".gitignore"), b"*.log\n".to_vec()).expect("control path"),
            CHANGED
        );
        let original = table.clone();
        assert_eq!(
            table.upsert(Path::new(".gitignore"), b"*.log\n".to_vec()).expect("control path"),
            UNCHANGED
        );
        assert_eq!(
            table.upsert(Path::new(".gitignore"), b"*.tmp\n".to_vec()).expect("control path"),
            CHANGED
        );
        assert_eq!(table.changes_from(&original).len(), 1);
        assert!(table.remove(Path::new(".gitignore")).expect("remove"));
        assert!(table.is_empty());
        assert_eq!(table.source_bytes(), 0);
        assert!(!table.remove(Path::new(".gitignore")).expect("missing is a no-op"));
    }

    /// The budget admits a table charged exactly to it and refuses one byte more.
    #[test]
    fn the_budget_admits_its_own_size_and_refuses_one_byte_over_it() {
        let source = b"*.log\n".to_vec();
        let exact = retained_source_cost(Path::new("a"), &source);
        let mut at_budget = budgeted(Some(exact));
        assert_eq!(
            at_budget.upsert(Path::new("a/.gitignore"), source.clone()).expect("control path"),
            CHANGED
        );
        assert_eq!(at_budget.retained_cost(), exact);

        let mut under_budget = budgeted(Some(exact - 1));
        assert_eq!(
            under_budget.upsert(Path::new("a/.gitignore"), source).expect("control path"),
            OVER_BUDGET
        );
        assert_eq!(under_budget.retained_cost(), 0);
        assert!(under_budget.contains(Path::new("a/.gitignore")));
        assert_eq!(
            under_budget.observation(),
            ControlObservation {
                limits: ControlLimits {
                    budget: Some(exact - 1),
                    line_limit: Some(DEFAULT_CONTROL_LINE_LIMIT),
                },
                applied: 0,
                rules: 0,
                refused: 1,
                refusals: vec![RefusedControl {
                    path: PathBuf::from("a/.gitignore"),
                    reason: ControlRefusalReason::Budget,
                }],
            }
        );
    }

    #[test]
    fn a_refused_source_drops_the_rules_it_replaces_and_freed_budget_admits_it_later() {
        let mut table = ControlTable::default();
        let first = source_at_test_limit();
        assert_eq!(
            table.upsert(Path::new(".gitignore"), first.clone()).expect("control path"),
            CHANGED
        );
        assert_eq!(
            table
                .upsert(Path::new("nested/.gitignore"), b"*.log\n".to_vec())
                .expect("control path"),
            OVER_BUDGET
        );
        let before = table.clone();

        // Replacing the root's source with one that no longer fits drops the old rules
        // rather than keeping rules that are no longer on disk.
        let mut grown = first;
        grown.push(b'\n');
        assert_eq!(
            table.upsert(Path::new(".gitignore"), grown).expect("control path"),
            OVER_BUDGET
        );
        assert!(table.is_empty());
        assert_eq!(table.changes_from(&before).len(), 1);
        assert_eq!(
            table.refusal_changes_from(&before),
            vec![(PathBuf::from(".gitignore"), None, Some(ControlRefusalReason::Budget))]
        );

        // With the budget free, reading the nested file again admits it and lifts its refusal.
        assert_eq!(
            table
                .upsert(Path::new("nested/.gitignore"), b"*.log\n".to_vec())
                .expect("control path"),
            CHANGED
        );
        assert_eq!(table.refused_len(), 1);
        // Removing a refused file lifts its refusal too.
        assert!(table.remove(Path::new(".gitignore")).expect("remove refused"));
        assert_eq!(table.refused_len(), 0);
        table.assert_consistent();
    }

    #[test]
    fn identical_sources_share_one_content_charge() {
        let source = b"target/\n*.log\nnode_modules/\n".to_vec();
        let mut table = ControlTable::default();
        table.upsert(Path::new("a/.gitignore"), source.clone()).expect("first holder");
        let one = table.retained_cost();
        table.upsert(Path::new("bb/.gitignore"), source.clone()).expect("second holder");

        // The second directory pays for its key and nothing for content it shares.
        assert_eq!(table.retained_cost() - one, CONTROL_SOURCE_OVERHEAD + "bb".len());
        assert_eq!(table.source_bytes(), 2 * source.len());
        assert!(table.is_ignored(Path::new("a/debug.log"), false));
        assert!(table.is_ignored(Path::new("bb/debug.log"), false));
    }

    /// One line at the limit applies; one byte longer refuses the whole source, before any
    /// parsing, so a hostile rule costs no matching work.
    #[test]
    fn the_line_limit_admits_its_own_length_and_refuses_one_byte_over_it() {
        let mut table = ControlTable::default();
        let at_limit = vec![b'a'; DEFAULT_CONTROL_LINE_LIMIT];
        assert_eq!(
            table.upsert(Path::new("a/.gitignore"), at_limit).expect("control path"),
            CHANGED
        );
        let over = [b"*.log\n".as_slice(), &vec![b'a'; DEFAULT_CONTROL_LINE_LIMIT + 1]].concat();
        assert_eq!(
            table.upsert(Path::new("b/.gitignore"), over).expect("control path"),
            OVER_LINE_LIMIT
        );
        assert!(!table.is_ignored(Path::new("b/debug.log"), false), "no rule of it applies");
        assert_eq!(
            table.refusals().collect::<Vec<_>>(),
            vec![RefusedControl {
                path: PathBuf::from("b/.gitignore"),
                reason: ControlRefusalReason::LineLimit,
            }]
        );
    }

    /// Raising or lifting one limit never moves the other: a larger budget still refuses a
    /// long line, and no line limit still refuses a table past its budget.
    #[test]
    fn the_budget_and_the_line_limit_lift_independently() {
        let long = [b"*.log\n".as_slice(), &vec![b'a'; DEFAULT_CONTROL_LINE_LIMIT + 1]].concat();
        let large = source_at_test_limit();

        let mut no_budget = budgeted(None);
        assert_eq!(
            no_budget.upsert(Path::new("big/.gitignore"), large.clone()).expect("path"),
            CHANGED
        );
        assert_eq!(
            no_budget.upsert(Path::new("c/.gitignore"), b"*.tmp\n".to_vec()).expect("path"),
            CHANGED
        );
        assert!(no_budget.retained_cost() > DEFAULT_CONTROL_BUDGET);
        assert_eq!(
            no_budget.upsert(Path::new(".gitignore"), long.clone()).expect("path"),
            OVER_LINE_LIMIT
        );

        let mut raised_budget = budgeted(Some(16 * DEFAULT_CONTROL_BUDGET));
        assert_eq!(
            raised_budget.upsert(Path::new(".gitignore"), long.clone()).expect("path"),
            OVER_LINE_LIMIT
        );

        let mut no_line_limit = ControlTable::with_limits(ControlLimits {
            line_limit: None,
            ..ControlLimits::default()
        });
        assert_eq!(no_line_limit.upsert(Path::new(".gitignore"), long).expect("path"), CHANGED);
        assert!(no_line_limit.is_ignored(Path::new("debug.log"), false));
        assert_eq!(
            no_line_limit.upsert(Path::new("big/.gitignore"), large).expect("path"),
            OVER_BUDGET
        );
        no_line_limit.assert_consistent();
    }

    #[test]
    fn a_listing_of_refusals_is_bounded_and_says_when_it_is_truncated() {
        let mut table = budgeted(Some(0));
        let refused = crate::MAX_RETAINED_ISSUES + 1;
        for directory in 0..refused {
            let path = PathBuf::from(format!("d{directory:03}/.gitignore"));
            assert_eq!(table.upsert(&path, b"*\n".to_vec()).expect("control path"), OVER_BUDGET);
        }
        let observation = table.observation();
        assert_eq!(observation.refused, u64::try_from(refused).expect("small"));
        assert_eq!(observation.refusals.len(), crate::MAX_RETAINED_ISSUES);
        assert_eq!(observation.refusals[0].path, Path::new("d000/.gitignore"));
        assert!(!observation.lists_every_refusal());
        assert!(!observation.is_complete());
    }

    #[test]
    fn control_identity_uses_standard_fnv1a_vectors() {
        assert_eq!(identity(b"").fingerprint, 0xcbf2_9ce4_8422_2325);
        assert_eq!(identity(b"a").fingerprint, 0xaf63_dc4c_8601_ec8c);
        assert_eq!(identity(b"foobar").fingerprint, 0x8594_4171_f739_67e8);
    }

    #[test]
    fn nested_negation_and_control_removal_change_the_governed_subtree() {
        let mut table = ControlTable::default();
        table.upsert(Path::new(".gitignore"), b"*.log\n".to_vec()).expect("root");
        table.upsert(Path::new("docs/.gitignore"), b"!keep.log\n".to_vec()).expect("nested");

        assert!(table.is_ignored(Path::new("debug.log"), false));
        assert!(table.is_ignored(Path::new("docs/other.log"), false));
        assert!(!table.is_ignored(Path::new("docs/keep.log"), false));
        assert_eq!(
            ControlTable::affected_subtree(Path::new("docs/.gitignore")).expect("scope"),
            Path::new("docs")
        );

        table.remove(Path::new("docs/.gitignore")).expect("remove nested");
        assert!(table.is_ignored(Path::new("docs/keep.log"), false));
    }

    #[test]
    fn ignored_parent_cannot_be_reincluded_from_inside_it() {
        let mut table = ControlTable::default();
        table.upsert(Path::new(".gitignore"), b"vendor/\n".to_vec()).expect("root");
        table
            .upsert(Path::new("vendor/.gitignore"), b"!keep.txt\n".to_vec())
            .expect("retained but inactive nested control");

        assert!(table.is_ignored(Path::new("vendor"), true));
        assert!(table.is_ignored(Path::new("vendor/keep.txt"), false));
    }

    /// Only `.gitignore` spelled in some ASCII case is a spelling of the control name; the
    /// canonical path every spelling is recorded under is its directory's `.gitignore`, and
    /// only that path is a valid control path (fdu-0w1b).
    #[test]
    fn a_control_name_is_spelled_in_any_ascii_case_and_recorded_by_one_path() {
        use std::ffi::OsStr;

        assert_eq!(control_spelling(OsStr::new(".gitignore")), Some(ControlSpelling::Exact));
        for variant in [".GITIGNORE", ".GitIgnore", ".gitIGNORE", ".gitignorE"] {
            assert_eq!(control_spelling(OsStr::new(variant)), Some(ControlSpelling::Variant));
        }
        // A non-ASCII letter some filesystem folds to `i` is not looked up, and neither is a
        // name of another length or with other characters.
        for other in [".gıtıgnore", ".gitignor", ".gitignore~", "gitignore.", ".gitignor3", ""] {
            assert_eq!(control_spelling(OsStr::new(other)), None, "{other:?}");
        }
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt as _;
            assert_eq!(control_spelling(OsStr::from_bytes(b".gitignor\xff")), None);
        }

        assert_eq!(
            governing_control(Path::new("a/.GITIGNORE")),
            Some(PathBuf::from("a/.gitignore"))
        );
        assert_eq!(governing_control(Path::new(".gitignore")), Some(PathBuf::from(".gitignore")));
        assert_eq!(governing_control(Path::new("a/README")), None);
        assert!(is_control_file(Path::new("a/.gitignore")));
        assert!(!is_control_file(Path::new("a/.GITIGNORE")), "operations name one path");
        let mut table = ControlTable::default();
        assert!(table.upsert(Path::new("a/.GITIGNORE"), b"*.log\n".to_vec()).is_err());
    }
}
