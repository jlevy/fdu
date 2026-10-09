//! The request model: what determines an answer, the grammars its values are written in,
//! and the typed refusals that name a bad request in the caller's own vocabulary.
//!
//! A [`Request`] is the [`Basis`] a retained index or opened root holds for its lifetime --
//! root, scope, and content -- plus what each read supplies: the [`Query`] and `now`, the
//! instant relative time windows resolve against. [`Delivery`] is how the caller asks for
//! it to be carried out, and never changes what the answer says.
//!
//! A refusal is a value, not a sentence. Each surface renders it through its
//! [`AxisNames`], so the rule and its wording are stated once here while the command line
//! names flags and the library and the Python API name fields. The grammars used to live
//! in both front ends, each with its own copy of every spelling and every message, which
//! is how one request came to mean two things depending on the door it came through.

use std::fmt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::CachePolicy;
use crate::content::AnalysisSet;
use crate::control::{ControlLimits, DEFAULT_CONTROL_BUDGET, DEFAULT_CONTROL_LINE_LIMIT};
use crate::engine_contract::EntryKind;
use crate::query::query_glob::Pattern;
use crate::query::query_report::{AxisNames, Query, ViewList, ViewSpec};
use crate::query::query_selection::{
    Bound, IgnoredEntries, Selection, ShareThreshold, SizeMetric, SortKey,
};
use crate::query::query_values::{
    parse_control_budget, parse_control_line_limit, parse_size, parse_when, system_time_to_nanos,
};
use crate::scan::ScanConfig;

/// Semantic filesystem scope, independent of scheduling and batching.
#[derive(Clone, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct Scope {
    /// Maximum retained depth.
    pub max_depth: Option<usize>,
    /// Whether directory symlinks are followed.
    pub follow_symlinks: bool,
    /// Whether traversal stays on one filesystem.
    pub one_filesystem: bool,
    /// Hidden-component admission.
    pub hidden: Option<std::sync::Arc<crate::admission::HiddenPolicy>>,
    /// Whether special filesystem objects are excluded.
    pub exclude_special: bool,
    /// Classification rules.
    pub types: Option<std::sync::Arc<crate::classify::TypeRegistry>>,
    /// Whether gitignore control files are observed.
    pub read_controls: bool,
    /// Ignored population retained by this basis.
    pub population: IgnoredEntries,
    /// Control admission limits.
    pub control_limits: ControlLimits,
}
impl Default for Scope {
    fn default() -> Self {
        ScanConfig::default().into()
    }
}
impl From<ScanConfig> for Scope {
    fn from(scan: ScanConfig) -> Self {
        Self {
            max_depth: scan.max_depth,
            follow_symlinks: scan.follow_symlinks,
            one_filesystem: scan.one_filesystem,
            hidden: scan.hidden,
            exclude_special: scan.exclude_special,
            types: scan.types,
            read_controls: scan.read_controls,
            population: scan.population,
            control_limits: scan.control_limits,
        }
    }
}
impl Scope {
    /// Derive the scanner's operational configuration from this scope and a delivery.
    pub fn scan_config(&self, delivery: &Delivery) -> ScanConfig {
        ScanConfig {
            max_depth: self.max_depth,
            follow_symlinks: self.follow_symlinks,
            one_filesystem: self.one_filesystem,
            hidden: self.hidden.clone(),
            exclude_special: self.exclude_special,
            types: self.types.clone(),
            read_controls: self.read_controls,
            population: self.population,
            control_limits: self.control_limits,
            threads: delivery.workers.scan,
            batch_size: delivery.batch_size,
            order: delivery.order,
            progress: None,
        }
    }
    fn identity_config(&self) -> ScanConfig {
        ScanConfig {
            max_depth: self.max_depth,
            follow_symlinks: self.follow_symlinks,
            one_filesystem: self.one_filesystem,
            hidden: self.hidden.clone(),
            exclude_special: self.exclude_special,
            types: self.types.clone(),
            read_controls: self.read_controls,
            population: self.population,
            control_limits: self.control_limits,
            ..ScanConfig::default()
        }
    }
    /// Semantic identity observed by the scanner.
    pub fn scope(&self) -> crate::ScanScope {
        self.identity_config().scope()
    }
    /// Identity of the persisted metadata tiers.
    pub fn snapshot_identity(&self) -> crate::SnapshotIdentity {
        self.identity_config().snapshot_identity()
    }
    pub(crate) fn unsupported_axis(&self) -> Option<ScopeAxis> {
        self.identity_config().unsupported_axis()
    }
}

/// Operational worker limits. Zero analysis workers selects available parallelism.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Workers {
    /// Directory-reading workers; absent selects the engine's bounded automatic pool.
    pub scan: Option<usize>,
    /// Content-reader workers.
    pub analysis: usize,
}

/// What a retained index or an opened root holds for its lifetime.
///
/// Everything here shapes the stored state itself, so a read can only be answered by a
/// holder of the same basis: see [`Request::validate_read`].
#[derive(Clone, Debug)]
pub struct Basis {
    /// The directory the answer is about.
    pub root: PathBuf,
    /// What the scan observes and retains.
    ///
    /// Scheduling is supplied separately by [`Delivery`].
    pub scope: Scope,
    /// The analyzers whose results the answer may report.
    pub content: AnalysisSet,
}

impl Basis {
    /// The basis a retained index holds, as the index itself can still state it.
    ///
    /// Scope comes back from [`ScanScope`](crate::ScanScope) and the control tier rather
    /// than from the `ScanConfig` that made it: what a read validates against is what the
    /// index observed, and the fields a config keeps beyond that -- threads, batch size,
    /// order -- are delivery, which no answer depends on.
    ///
    /// The control tier carries both halves of what the scan observed, so both are read
    /// from it: whether any rule was read, and the limits the ones that were read were
    /// admitted under. An index that observed nothing applied no limits, and the table's
    /// are what a scope that reads no rule would have been taken under; nothing in a basis
    /// that observes no control state depends on them.
    pub fn held_by(index: &crate::Index) -> Self {
        let scope = index.scope();
        let controls = index.control_identity();
        Self {
            root: index.root_path().to_path_buf(),
            scope: Scope {
                max_depth: scope.max_depth,
                follow_symlinks: scope.follow_symlinks,
                one_filesystem: scope.one_filesystem,
                exclude_special: scope.exclude_special,
                read_controls: controls.is_observed(),
                population: scope.population,
                control_limits: match controls {
                    crate::ControlTierIdentity::Observed { limits } => limits,
                    crate::ControlTierIdentity::NotObserved => Self::UNOBSERVED_LIMITS,
                },
                ..Scope::default()
            },
            content: index.content_set(),
        }
    }

    /// The basis a spec names, without the read half it also carries.
    ///
    /// What a holder is fixed with for its lifetime: root, scope, and analyzers. A caller
    /// that opens an index and then reads it many times parses these once, and each read
    /// hands them back to [`Request::read`]; building a whole request and discarding its
    /// query was the shape that made the basis look like the read's to overwrite.
    ///
    /// # Errors
    ///
    /// [`RequestError`] for a value no grammar accepts, named as `axes` spells its axis.
    pub fn build(spec: &RequestSpec<'_>, axes: &'static AxisNames) -> Result<Self, RequestError> {
        let content = parse_content(spec, axes)?;
        let mut scope = parse_scope(spec, axes)?;
        scope.population = parse_population(spec.read.ignored, axes)?;
        Ok(Self { root: spec.root.to_path_buf(), scope, content })
    }

    /// The limits a basis records when its scan observed no control state at all.
    ///
    /// The defaults table's, because they are what the scope would have been taken under
    /// had it read a rule; no rule this model states reads them when `read_controls` is
    /// off, so this is a placeholder named rather than a value implied.
    const UNOBSERVED_LIMITS: ControlLimits = Request::DEFAULTS.control_limits;
}

/// One root of a report: the path as its caller named it, and the directory it is.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamedRoot {
    /// The path as the caller gave it, normalized by its components.
    ///
    /// `PathBuf::from_iter(path.components())`: repeated and trailing separators go, and so
    /// does a `.` anywhere but at the start, while `/` stays `/` and `C:\` stays `C:\`
    /// rather than becoming an empty string or a drive-relative `C:`. It is what text
    /// prints before a root-relative path (`docs/guide.md`), as `find docs src` does, and
    /// what a machine format serializes as `label`, beside `label_raw` when it is not
    /// UTF-8.
    pub label: PathBuf,
    /// The canonical directory, which is what the root's index names
    /// ([`Index::root_path`](crate::Index::root_path)).
    pub path: PathBuf,
}

/// The roots of one report: one or more directories, in the caller's order, none equal to
/// or inside another.
///
/// A report over several roots is the sum of the reports over each, so a path counted under
/// two roots would count twice, and fdu counts each path once. Collapsing the inner root
/// instead would guess at intent and change what `.gitignore` applies, since rules are read
/// only inside the scanned root. So [`Self::resolve`] refuses an overlap, naming both
/// labels, and refusing is the reversible choice: accepting overlap later adds an answer,
/// where changing what a total means after release would not.
///
/// Built only by [`Self::resolve`], which validates every root before anything is scanned,
/// so a value of this type is always a valid, disjoint, non-empty list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Roots {
    roots: Vec<NamedRoot>,
}

impl Roots {
    /// Validate and name the roots of one report, in the caller's order.
    ///
    /// In order: an empty list is refused ([`RequestError::NoRoots`]); then each root, in
    /// argument order, must exist and be a directory, failing with the error one root has
    /// always failed with -- `Error::Io` at the path as given when it cannot be resolved,
    /// and at the canonical path, as the scanner words it, when it is not a directory; then
    /// no two may be the same directory ([`RequestError::RootsRepeated`]) or one inside the
    /// other ([`RequestError::RootsOverlap`]).
    ///
    /// The overlap check compares identities as well as paths. Canonical paths miss
    /// aliases: on macOS `/Users` and `/System/Volumes/Data/Users` are one directory
    /// through a firmlink, and a Linux bind mount behaves the same way. So on Unix each
    /// root's device and inode are compared against every other root's and against the
    /// chain of its ancestors, and everywhere canonical paths are compared component by
    /// component. Elsewhere only the path comparison runs, so an alias such as a `subst`
    /// drive goes undetected there. The check is conservative: `/` and `/mnt/usb` overlap
    /// even under one-filesystem, where the walk would not have descended into the second.
    ///
    /// This reads each root's metadata and its ancestors', and nothing else: it validates
    /// the request's input, as a scan would on reaching each root, before any work is done.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidRequest`](crate::Error::InvalidRequest) for an empty list and for
    /// overlapping roots; [`Error::Io`](crate::Error::Io) for a root that cannot be
    /// resolved or is not a directory.
    pub fn resolve<P: AsRef<Path>>(paths: &[P]) -> crate::Result<Self> {
        if paths.is_empty() {
            return Err(crate::Error::InvalidRequest(RequestError::NoRoots));
        }
        let mut roots = Vec::with_capacity(paths.len());
        let mut facts = Vec::with_capacity(paths.len());
        for given in paths {
            let given = given.as_ref();
            let canonical = given.canonicalize().map_err(|error| crate::Error::io(given, error))?;
            let metadata = std::fs::metadata(&canonical)
                .map_err(|error| crate::Error::io(&canonical, error))?;
            if !metadata.is_dir() {
                // The scanner's own words, so a file root fails as it always has.
                return Err(crate::Error::io(
                    &canonical,
                    std::io::Error::new(
                        std::io::ErrorKind::NotADirectory,
                        "scan root is not a directory",
                    ),
                ));
            }
            facts.push(RootFacts::of(&canonical, &metadata));
            roots.push(NamedRoot { label: given.components().collect(), path: canonical });
        }
        if let Some(overlap) = first_overlap(&facts) {
            let (outer, inner) = (&roots[overlap.outer], &roots[overlap.inner]);
            return Err(crate::Error::InvalidRequest(if overlap.same {
                RequestError::RootsRepeated {
                    first: outer.label.clone(),
                    second: inner.label.clone(),
                }
            } else {
                RequestError::RootsOverlap {
                    inner: inner.label.clone(),
                    outer: outer.label.clone(),
                }
            }));
        }
        Ok(Self { roots })
    }

    /// Roots as given, unvalidated, for a test whose indexes name directories no
    /// filesystem holds.
    #[cfg(test)]
    pub(crate) fn named(roots: Vec<NamedRoot>) -> Self {
        Self { roots }
    }

    /// Every root, in the caller's order.
    pub fn as_slice(&self) -> &[NamedRoot] {
        &self.roots
    }

    /// Every root, in the caller's order.
    pub fn iter(&self) -> std::slice::Iter<'_, NamedRoot> {
        self.roots.iter()
    }

    /// The first root the caller named; the only one of a single-root report.
    pub fn first(&self) -> &NamedRoot {
        &self.roots[0]
    }

    /// Whether there is more than one root, which is when a report's shape changes: rows
    /// gain a root index, the tree a total row, and text a label before each path.
    pub fn is_several(&self) -> bool {
        self.roots.len() > 1
    }
}

/// One request over one or more roots: the roots, and the request every root shares.
///
/// The rest of a request -- scope, analyzers, selection, views, and `now` -- belongs to the
/// whole report, so it is built once, from a spec naming any one root, and each root's
/// request is that one with its basis root replaced. Each therefore has exactly the
/// identity one root's request has, its snapshot, content sidecar, and cache policy
/// included, so one root answered from its cache and another walked cold can sit in one
/// report; and every root's ages are measured from one instant.
#[derive(Clone, Debug)]
pub struct RootsRequest {
    roots: Roots,
    request: Request,
}

impl RootsRequest {
    /// A request over `roots`, sharing everything but the basis root with `request`.
    ///
    /// The shared request's basis root becomes the first root's label, so that its own
    /// root is never a directory outside the report.
    pub fn new(roots: Roots, mut request: Request) -> Self {
        request.basis.root.clone_from(&roots.first().label);
        Self { roots, request }
    }

    /// The report's roots.
    pub fn roots(&self) -> &Roots {
        &self.roots
    }

    /// The request every root shares, rooted at the first root.
    pub fn request(&self) -> &Request {
        &self.request
    }

    /// Each root beside its own request: the shared one, rooted at the root's label.
    ///
    /// The label rather than the canonical path, so a root that vanished between
    /// validation and its walk fails in the spelling its caller used, as one root does;
    /// every route canonicalizes the root before it reads anything.
    pub(crate) fn per_root(&self) -> impl Iterator<Item = (&NamedRoot, Request)> + '_ {
        self.roots.iter().map(|root| {
            let mut request = self.request.clone();
            request.basis.root.clone_from(&root.label);
            (root, request)
        })
    }
}

impl<'a> IntoIterator for &'a Roots {
    type Item = &'a NamedRoot;
    type IntoIter = std::slice::Iter<'a, NamedRoot>;

    fn into_iter(self) -> Self::IntoIter {
        self.roots.iter()
    }
}

/// What the overlap check knows of one root: its canonical path, and on Unix the device
/// and inode of the root and of each of its ancestors.
#[derive(Clone, Debug, Default)]
struct RootFacts {
    canonical: PathBuf,
    /// The root's own identity, where the platform has one.
    identity: Option<(u64, u64)>,
    /// The identities of its ancestors, nearest first, skipping any that cannot be read.
    ancestors: Vec<(u64, u64)>,
}

impl RootFacts {
    #[cfg(unix)]
    fn of(canonical: &Path, metadata: &std::fs::Metadata) -> Self {
        use std::os::unix::fs::MetadataExt;
        Self {
            canonical: canonical.to_path_buf(),
            identity: Some((metadata.dev(), metadata.ino())),
            ancestors: canonical
                .ancestors()
                .skip(1)
                .filter_map(|ancestor| std::fs::metadata(ancestor).ok())
                .map(|metadata| (metadata.dev(), metadata.ino()))
                .collect(),
        }
    }

    #[cfg(not(unix))]
    fn of(canonical: &Path, _metadata: &std::fs::Metadata) -> Self {
        Self { canonical: canonical.to_path_buf(), identity: None, ancestors: Vec::new() }
    }

    /// Whether this root is the same directory as `other`.
    fn same_as(&self, other: &Self) -> bool {
        self.canonical == other.canonical
            || self.identity.is_some_and(|identity| other.identity == Some(identity))
    }

    /// Whether this root lies inside `outer`, by path or, through an alias, by identity.
    fn inside(&self, outer: &Self) -> bool {
        // `starts_with` compares whole components, so `src-old` is not inside `src`.
        self.canonical.starts_with(&outer.canonical)
            || outer.identity.is_some_and(|identity| self.ancestors.contains(&identity))
    }
}

/// The first pair of roots, in argument order, that overlap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Overlap {
    /// The containing root, or the earlier one of two that are the same directory.
    outer: usize,
    /// The contained root, or the later one of two that are the same directory.
    inner: usize,
    /// Whether the two are one directory rather than one inside the other.
    same: bool,
}

/// The first overlapping pair in argument order, or `None` when the roots are disjoint.
///
/// A pure function of the facts, so a test can state aliases no host it runs on has.
fn first_overlap(facts: &[RootFacts]) -> Option<Overlap> {
    for (a, first) in facts.iter().enumerate() {
        for (b, second) in facts.iter().enumerate().skip(a + 1) {
            if first.same_as(second) {
                return Some(Overlap { outer: a, inner: b, same: true });
            }
            if second.inside(first) {
                return Some(Overlap { outer: a, inner: b, same: false });
            }
            if first.inside(second) {
                return Some(Overlap { outer: b, inner: a, same: false });
            }
        }
    }
    None
}

/// How a request is carried out, which never changes what its answer says.
///
/// No `Default`, deliberately. Every field here is a decision its caller has already made,
/// and the cache policy is the one that decides whether an answer touches the filesystem
/// and whether it leaves a trace; a default one reads `cache: Auto` whatever the caller
/// asked for, which is exactly how the watch session came to validate against a cache
/// policy nobody had chosen and `WatchCacheOnly` became unreachable inside the engine
/// (fdu-i18y). A caller that wants the ordinary delivery names it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delivery {
    /// How the snapshot cache may be used.
    pub cache: CachePolicy,
    /// Answer from the snapshot alone, without touching the tree.
    ///
    /// The one delivery whose answer can be stale, and it says so: the report's
    /// provenance is `cache_only` and its freshness `stale`. It fails when no usable
    /// snapshot exists rather than quietly scanning, because a fast path that is sometimes
    /// a full walk, with nothing in the output to say which happened, is worse than none.
    /// It writes nothing, and it is refused with [`CachePolicy::Off`], with a watch, and
    /// by a refresh, none of which can take an unverified snapshot as their answer.
    pub stale_ok: bool,
    /// Where the snapshot for this root lives, or `None` for no cache at all.
    pub cache_path: Option<PathBuf>,
    /// Whether a partial answer is accepted as a success.
    pub accept_partial: bool,
    /// Whether the answer repeats as a watch, and how.
    pub watch: Option<WatchDelivery>,
    /// Directory and content reader workers.
    pub workers: Workers,
    /// Maximum operations in one scanner batch.
    pub batch_size: usize,
    /// Directory traversal scheduling.
    pub order: crate::ScanOrder,
}

impl Delivery {
    /// Ordinary execution settings with an explicitly chosen cache policy and location.
    pub fn new(cache: CachePolicy, cache_path: Option<PathBuf>) -> Self {
        Self {
            cache,
            stale_ok: false,
            cache_path,
            accept_partial: false,
            watch: None,
            workers: Workers::default(),
            batch_size: ScanConfig::default().batch_size,
            order: crate::ScanOrder::default(),
        }
    }

    /// Ordinary execution settings that answer from the snapshot at `cache_path` alone.
    pub fn stale_ok(cache_path: Option<PathBuf>) -> Self {
        Self { stale_ok: true, ..Self::new(CachePolicy::Auto, cache_path) }
    }

    /// Representative deliveries for checking policy independently of route.
    /// Worker counts and cache location are fixed; every cache, stale-answer,
    /// partial-answer, and watch choice is represented.
    pub fn enumerate() -> impl Iterator<Item = Self> {
        [
            (CachePolicy::Auto, false),
            (CachePolicy::On, false),
            (CachePolicy::Off, false),
            (CachePolicy::Auto, true),
            (CachePolicy::On, true),
            (CachePolicy::Off, true),
        ]
        .into_iter()
        .flat_map(|(cache, stale_ok)| {
            [false, true].into_iter().flat_map(move |accept_partial| {
                [None, Some(WatchDelivery::default())].into_iter().map(move |watch| Self {
                    cache,
                    stale_ok,
                    cache_path: Some(PathBuf::from("cache.fdu")),
                    accept_partial,
                    watch,
                    workers: Workers { analysis: 1, ..Workers::default() },
                    batch_size: ScanConfig::default().batch_size,
                    order: crate::ScanOrder::default(),
                })
            })
        })
    }
}

/// How a watch repeats its answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WatchDelivery {
    /// The longest a repaint waits for changes; change detection itself is event-driven.
    pub interval: Duration,
}

impl Default for WatchDelivery {
    fn default() -> Self {
        Self { interval: Self::DEFAULT_INTERVAL }
    }
}
impl WatchDelivery {
    /// Shared default repaint cadence for every surface.
    pub const DEFAULT_INTERVAL: Duration = Duration::from_secs(2);
}

/// Everything that determines an answer.
///
/// The three public fields are the answer's whole input. A private fourth records how the
/// basis's analyzers were chosen, which no answer reads and only a refusal does, so a
/// request outside this crate is made with [`Self::new`], [`Self::read`], or
/// [`Self::build`] rather than a struct literal.
#[derive(Clone, Debug)]
pub struct Request {
    /// Root, scope, and content: what a holder of stored state must match.
    pub basis: Basis,
    /// Selection, views, and view options.
    pub query: Query,
    /// The instant relative time windows were resolved against, and the reference every
    /// age in the answer is measured from.
    ///
    /// The windows are resolved when the request is built, so [`Selection::modified`] is
    /// absolute and a watch that builds its request once at start never slides its window.
    /// After that this field is only the age reference: a watch session reads each repaint
    /// with a copy whose `now` is that repaint's instant, so ages stay current over a long
    /// session while the question it answers stays the one it was built with.
    pub now: SystemTime,
    /// How the basis's analyzers were chosen.
    ///
    /// Private, because it is provenance rather than input: the answer is the same whether
    /// a caller named `code` as a view or as an analyzer, and a public field could be set
    /// out of step with `basis.content`, so that a refusal named a view that caused
    /// nothing. Fixed by the constructor that read the caller's axes.
    origin: Origin,
}

/// How a request's analyzers were chosen: what its caller named, and which views implied
/// the rest.
///
/// What a refusal needs to name what the caller wrote, and to offer a remedy the caller's
/// route accepts: a request built from an analyzer axis can add an analyzer, and a basis
/// supplied whole cannot be told to.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Origin {
    /// The caller supplied the basis whole ([`Request::new`], [`Request::read`]), so nothing
    /// was named or implied.
    Supplied,
    /// [`Request::build`] read the caller's analyzer and view axes.
    Built {
        /// What the analyzer axis named; `none` when it named nothing.
        named: AnalysisSet,
        /// The named views that imply an analyzer, in the caller's order.
        implied_by: Vec<ViewSpec>,
    },
}

/// What fixed the analyzers a read is answered from, which decides the remedy a refusal
/// can offer for analysis the read needs and the basis lacks.
///
/// Each route accepts a different remedy. A request that builds its own basis takes
/// another analyzer; a retained index answers only what it was opened with, so the
/// remedy is to open it again; an opened root runs no analyzer at all; and a basis a
/// caller supplied whole is the caller's to widen. A refusal that named one route's remedy
/// on another sent the caller looking for a parameter that does not exist.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BasisHolder {
    /// [`Request::build`]: the request enables what its analyzer axis names.
    Built,
    /// [`Request::new`]: a basis supplied whole, which nothing holds.
    Supplied,
    /// A retained index, which holds the analyzers it was opened with.
    Index,
    /// An opened root, which runs no analyzer.
    OpenedRoot,
}

impl BasisHolder {
    /// What the caller can do about analysis `needed` that the basis, holding `held`, lacks.
    ///
    /// Appended to a sentence that already names what is needed, so each form starts with
    /// its own punctuation.
    fn remedy(self, needed: AnalysisSet, held: AnalysisSet, axes: &AxisNames) -> String {
        let analyze = axes.analyze;
        match self {
            Self::Built => format!(": add {analyze} {}", needed.request_label()),
            Self::Supplied => format!("; its basis holds {analyze} {}", held.request_label()),
            Self::Index => {
                format!("; this index was opened with {analyze} {}", held.request_label())
            }
            Self::OpenedRoot => format!(
                ", which an opened root never runs; use a one-shot report, or an index opened \
                 with {analyze} {}",
                needed.request_label()
            ),
        }
    }
}

/// A request as a caller wrote it: raw values, before any grammar has read them.
///
/// Surface-neutral, so the command line and the Python API fill one shape and
/// [`Request::build`] parses both identically. A value a surface already holds typed, such
/// as `--scan-depth`, reaches this through its `Display`, so a disagreement between that
/// type and the grammar here shows up as a golden difference rather than a silent one.
/// `None` and an empty list mean the caller named nothing, and [`Request::DEFAULTS`]
/// decides; the switches whose only default is off are plain booleans.
#[derive(Clone, Copy, Debug)]
pub struct RequestSpec<'a> {
    /// The directory the answer is about.
    pub root: &'a Path,
    /// Retention depth: a whole number.
    pub scan_depth: Option<&'a str>,
    /// Stay on the root's filesystem.
    pub one_filesystem: bool,
    /// Observe `.gitignore`.
    pub read_controls: Option<bool>,
    /// The `.gitignore` budget: a size or `all`.
    pub control_budget: Option<&'a str>,
    /// The longest `.gitignore` line: a size or `all`.
    pub control_line_limit: Option<&'a str>,
    /// Analyzers: a comma list of `none`, `lines`, `code`, `words`, or `all`.
    pub analyze: Option<&'a str>,
    /// What this read asks of the basis the axes above describe.
    pub read: ReadSpec<'a>,
}

/// What one read supplies, as its caller wrote it: the [`Query`] half of a request.
///
/// Split from the basis half rather than flattened beside it, because a holder of stored
/// state fixes root, scope, and analyzers once and then answers many reads: a read names
/// only this, and [`Request::read`] is what takes the two together. A field declared in
/// both halves is a field one of them could silently drop, which is why this is the one
/// declaration and [`RequestSpec`] contains it.
#[derive(Clone, Copy, Debug)]
pub struct ReadSpec<'a> {
    /// Views: a comma list, or `full`.
    pub views: Option<&'a str>,
    /// Presentation format; omitted means automatic human output.
    pub format: Option<&'a str>,
    /// Logical words per document page: a positive integer.
    pub words_per_page: Option<&'a str>,
    /// Patterns an entry must match one of.
    pub include: &'a [String],
    /// Patterns that exclude an entry.
    pub exclude: &'a [String],
    /// Smallest size, as `512`, `10M`, or `1.5GiB`.
    pub min_size: Option<&'a str>,
    /// Inclusive lower bound on modification time, as `2h` or a timestamp.
    pub modified_since: Option<&'a str>,
    /// Exclusive upper bound on modification time.
    pub modified_before: Option<&'a str>,
    /// Entry kinds: a comma list of `file`, `dir`, `symlink`, or `other`.
    pub kinds: Option<&'a str>,
    /// Selection by ignored state: `include`, `exclude`, or `only`.
    pub ignored: Option<&'a str>,
    /// Rendered tree depth: a whole number or `all`.
    pub depth: Option<&'a str>,
    /// Minimum displayed contribution to the selected root, as a percentage.
    pub min_share: Option<&'a str>,
    /// Maximum immediate children shown per directory: a whole number or `all`.
    pub breadth: Option<&'a str>,
    /// Rows per view: a whole number or `all`.
    pub limit: Option<&'a str>,
    /// Ordering key: `size`, `count`, `mtime`, or `name`.
    pub sort: Option<&'a str>,
    /// Reverse the ordering.
    pub reverse: bool,
    /// Size metric: `allocated` or `apparent`.
    pub size: Option<&'a str>,
}

impl ReadSpec<'_> {
    /// A read that names nothing, so every axis takes its default.
    pub const fn new() -> Self {
        Self {
            views: None,
            format: None,
            words_per_page: None,
            include: &[],
            exclude: &[],
            min_size: None,
            modified_since: None,
            modified_before: None,
            kinds: None,
            ignored: None,
            depth: None,
            min_share: None,
            breadth: None,
            limit: None,
            sort: None,
            reverse: false,
            size: None,
        }
    }
}

impl Default for ReadSpec<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> RequestSpec<'a> {
    /// A spec that names only its root, so every other axis takes its default.
    pub const fn new(root: &'a Path) -> Self {
        Self {
            root,
            scan_depth: None,
            one_filesystem: false,
            read_controls: None,
            control_budget: None,
            control_line_limit: None,
            analyze: None,
            read: ReadSpec::new(),
        }
    }
}

/// Every default a request takes when its caller names nothing, stated once.
///
/// | Axis | Default |
/// | --- | --- |
/// | Size | allocated |
/// | Views of a report | [`Self::report_view`]: [`ViewSpec::default_for`] the content |
/// | Views of a watch | [`Self::report_view`] of no content, which is `tree` |
/// | Words per page | 250 |
/// | Content | no analyzer |
/// | `.gitignore` | observed, under the default budget and line limit |
///
/// Each answers a question: "how much disk does this use" is allocated bytes, as `du`
/// reports them; a request that pays to read files displays what it read; and a tree is
/// what "what is big here" looks like. Surfaces take these rather than declaring their own,
/// because a default declared twice drifts: size was apparent in Rust and allocated
/// everywhere else, and `words_per_page` was written out in three places.
///
/// A watch has no view default of its own, and no field here for one. It is derived rather
/// than declared because it is not a separate decision: [`RequestError::WatchContent`]
/// refuses a watch with any analyzer, named or implied by a view, so the content a watch
/// serves is always none and its view is the report default for none. A field would have
/// restated `tree` beside the rule that makes it true, which is the shape a default drifts
/// out of.
///
/// Content is `none` beyond what the views imply: [`Request::build`] adds the analyzers a
/// content view shows ([`ViewSpec::implies`]), so `none` names no analyzer of its own rather
/// than forbidding one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestDefaults {
    /// The size metric.
    pub size: SizeMetric,
    /// Logical words per derived document page.
    pub words_per_page: u64,
    /// The analyzers a request enables.
    pub content: AnalysisSet,
    /// Whether a scan observes `.gitignore`.
    pub read_controls: bool,
    /// The limits `.gitignore` files are applied under.
    pub control_limits: ControlLimits,
}

impl RequestDefaults {
    /// The view a report displays when its caller named none: the one that shows what
    /// `content` read, or the tree when it read nothing.
    pub const fn report_view(self, content: AnalysisSet) -> ViewSpec {
        ViewSpec::default_for(content)
    }
}

impl Request {
    /// The defaults table.
    pub const DEFAULTS: RequestDefaults = RequestDefaults {
        size: SizeMetric::Allocated,
        words_per_page: 250,
        content: AnalysisSet::NONE,
        read_controls: true,
        control_limits: ControlLimits {
            budget: Some(DEFAULT_CONTROL_BUDGET),
            line_limit: Some(DEFAULT_CONTROL_LINE_LIMIT),
        },
    };

    /// One read of what `basis` holds, from a query its caller already has typed.
    ///
    /// The composition six call sites wrote out by hand, each of them a holder's basis and
    /// one read's query and instant. A literal is not wrong, but a name is where the rule
    /// can be stated: `basis` is the holder's, never the read's, and `now` is this read's,
    /// fixed so a watch that repaints does not slide its own window.
    ///
    /// No validation, because the query is already typed and its holder is the one that
    /// knows which rule applies -- [`Self::validate_read`] for a retained index,
    /// [`Self::validate`] for a request that is its own basis. [`Self::read`] is the
    /// constructor that parses and validates in one step.
    ///
    /// Nothing is implied: the basis is the caller's, whole. A refusal says what the basis
    /// holds ([`BasisHolder::Supplied`]) rather than naming an analyzer to add, because no
    /// analyzer axis built it.
    pub const fn new(basis: Basis, query: Query, now: SystemTime) -> Self {
        Self { basis, query, now, origin: Origin::Supplied }
    }

    /// The named views whose analyzers [`Self::build`] added to the basis, in the caller's
    /// order; empty for a basis supplied whole.
    ///
    /// Provenance, not part of the answer: the basis's content already includes what they
    /// imply. It is what a watch refusal names ([`RequestError::WatchContent`]), since the
    /// caller wrote a view rather than an analyzer.
    pub fn implied_by(&self) -> &[ViewSpec] {
        match &self.origin {
            Origin::Built { implied_by, .. } => implied_by.as_slice(),
            Origin::Supplied => &[],
        }
    }

    /// Which rule fixed this request's analyzers, for a refusal of its own basis.
    const fn holder(&self) -> BasisHolder {
        match self.origin {
            Origin::Built { .. } => BasisHolder::Built,
            Origin::Supplied => BasisHolder::Supplied,
        }
    }

    /// One read of what `basis` holds, written in the value grammars.
    ///
    /// The constructor every read site wants: a holder supplies the basis it was opened
    /// with, and the caller supplies only what this read asks. The view default comes from
    /// the analyzers the basis already holds, so a typed set never has to be spelled back
    /// into the grammar to find out what a request that read files displays, and no caller
    /// builds a request with a throw-away basis and overwrites it afterwards.
    ///
    /// A held basis is never widened: a view that implies an analyzer the holder was not
    /// opened with is refused ([`RequestError::ViewNeedsAnalyzer`]) rather than read for,
    /// because a read cannot add to what the holder paid for.
    ///
    /// # Errors
    ///
    /// [`RequestError`] for a value no grammar accepts, and for a read the basis cannot
    /// answer: [`Self::validate`]'s rules, which here are the holder's own. A refusal for
    /// missing analysis names what the index was opened with ([`BasisHolder::Index`]),
    /// since that is the holder this constructor reads for.
    pub fn read(
        basis: Basis,
        spec: &ReadSpec<'_>,
        now: SystemTime,
        axes: &'static AxisNames,
    ) -> Result<Self, RequestError> {
        Self::read_held(basis, spec, now, axes, BasisHolder::Index)
    }

    /// One read of an opened root, written in the value grammars.
    ///
    /// [`Self::read`] over [`OpenedIndex::basis`](crate::OpenedIndex::basis), the one
    /// statement of what every opened root holds, except that a refusal for missing
    /// analysis says an opened root runs no analyzer ([`BasisHolder::OpenedRoot`]) rather
    /// than naming an analyzer to open it with, which its constructor does not take.
    ///
    /// # Errors
    ///
    /// [`RequestError`] as [`Self::read`] returns it.
    pub fn read_opened(
        spec: &ReadSpec<'_>,
        now: SystemTime,
        axes: &'static AxisNames,
    ) -> Result<Self, RequestError> {
        Self::read_held(crate::OpenedIndex::basis(), spec, now, axes, BasisHolder::OpenedRoot)
    }

    /// The read both held constructors share, refused in `holder`'s words.
    fn read_held(
        basis: Basis,
        spec: &ReadSpec<'_>,
        now: SystemTime,
        axes: &'static AxisNames,
        holder: BasisHolder,
    ) -> Result<Self, RequestError> {
        let views = parse_views(spec.views, axes)?;
        let mut query = build_query(basis.content, views, spec, now, axes)?;
        if spec.ignored.is_none() {
            query.selection.ignored = basis.scope.population;
        }
        let request = Self::new(basis, query, now);
        request.validate_against(&request.basis, holder)?;
        Ok(request)
    }

    /// Parse a spec into a request, resolving relative time windows against `now`.
    ///
    /// Refusals name each axis as `axes` spells it, and so do the report diagnostics of the
    /// query built here. Every value is parsed before any is checked against another, in
    /// the order the command line reads its flags: content, views, selection, the page
    /// denominator, then scope. Rules that relate axes are [`Self::validate`]'s.
    ///
    /// This request builds its own basis, so a view that shows analysis requests it: the
    /// basis enables the analyzers `analyze` names together with the ones the named views
    /// imply ([`ViewSpec::implies`]), and `full` and the default view are resolved against
    /// that union. `--view code` and `--analyze code` therefore build one basis, and share
    /// one sidecar and one answer. Only a view with no metadata meaning implies anything,
    /// so no metadata report ever turns into a read of file bodies.
    pub fn build(
        spec: &RequestSpec<'_>,
        now: SystemTime,
        axes: &'static AxisNames,
    ) -> Result<Self, RequestError> {
        // The three steps in the order the command line reads its flags, which is the order
        // a refusal names when two axes are both wrong: content, then everything this read
        // supplies, then scope. `Basis::build` runs the first and the third together, for
        // the callers that fix a basis and read it many times; it implies nothing, because
        // a holder's analyzers are what it was explicitly opened with.
        let named = parse_content(spec, axes)?;
        let views = parse_views(spec.read.views, axes)?;
        let implied_by = views.implying();
        let content = named.union(views.implies());
        let query = build_query(content, views, &spec.read, now, axes)?;
        let mut scope = parse_scope(spec, axes)?;
        scope.population = query.selection.ignored;
        Ok(Self {
            origin: Origin::Built { named, implied_by },
            ..Self::new(Basis { root: spec.root.to_path_buf(), scope, content }, query, now)
        })
    }

    /// Refuse a request no holder of its own basis could answer.
    ///
    /// In order: more views than one report carries, a view its content cannot answer, and
    /// a selection by ignored state its scope does not observe. A refusal for missing
    /// analysis offers the remedy of the constructor that made the request: the analyzer to
    /// add for one [`Self::build`] made, and what the basis holds for one supplied whole.
    pub fn validate(&self) -> Result<(), RequestError> {
        self.validate_against(&self.basis, self.holder())
    }

    /// Refuse a read that `held`, the basis of a retained index, cannot answer.
    ///
    /// Content must be equal: an index built with other analyzers holds other metrics, and
    /// serving a narrower request from a wider store is a projection this model does not
    /// define. The remaining rules are [`Self::validate`]'s, applied to what `held`
    /// observed rather than to what the request says it would have, and a refusal for
    /// missing analysis names what the index was opened with. Scope equality is not
    /// checked here; `ScanConfig` owns it.
    pub fn validate_read(&self, held: &Basis) -> Result<(), RequestError> {
        let entries = held.scope.snapshot_identity().entries;
        let wanted = crate::ContentTierIdentity::for_request(entries, self.basis.content);
        let stored = crate::ContentTierIdentity::for_request(entries, held.content);
        if wanted.admit(&stored).is_none() {
            return Err(RequestError::ContentMismatch {
                held: held.content,
                requested: self.basis.content,
            });
        }
        self.validate_against(held, BasisHolder::Index)
    }

    /// Refuse a read an opened root cannot answer: [`Self::validate`]'s rules over the
    /// opened root's basis, whose refusal for missing analysis says that an opened root
    /// runs no analyzer rather than naming an option its constructor does not have.
    pub(crate) fn validate_opened(&self) -> Result<(), RequestError> {
        self.validate_against(&self.basis, BasisHolder::OpenedRoot)
    }

    /// Refuse a delivery that cannot carry this request out.
    ///
    /// Everything a watch cannot do, in one place, because a watch is the one delivery that
    /// changes which requests can be answered at all:
    ///
    /// - A narrowed scan scope ([`RequestError::WatchScope`]): a watcher cannot filter its
    ///   backend's events against a boundary the scan drew. Selection still works, because
    ///   it filters the retained index rather than the scan.
    /// - Content analysis ([`RequestError::WatchContent`]): nothing re-reads a file the
    ///   watch sees change, so a session would go on reporting the metrics it started with
    ///   as fresh. Named or implied alike, and the refusal names every axis that enabled it
    ///   -- the analyzers named and the views that implied the rest ([`Self::implied_by`])
    ///   -- since those are what the caller wrote, and dropping only one would not help.
    /// - A snapshot nothing verified ([`RequestError::WatchCacheOnly`]): the window between
    ///   the snapshot and the session's start is never observed, so the first answer would
    ///   describe a tree that may have moved and every later one would build on it.
    ///
    /// Each was a guard on one surface, which is why a library caller and a Python caller
    /// could ask for what the command line refuses.
    ///
    /// One refusal applies to every route: a stale answer comes from the snapshot, which
    /// [`CachePolicy::Off`] never reads ([`RequestError::StaleOkCacheOff`]).
    pub fn validate_delivery(&self, delivery: &Delivery) -> Result<(), RequestError> {
        if delivery.stale_ok && delivery.cache == CachePolicy::Off {
            return Err(RequestError::StaleOkCacheOff);
        }
        if delivery.watch.is_none() {
            return Ok(());
        }
        if self.basis.scope.max_depth.is_some() || self.basis.scope.one_filesystem {
            return Err(RequestError::WatchScope);
        }
        if self.basis.content.is_enabled() {
            return Err(match &self.origin {
                Origin::Built { named, implied_by } => {
                    RequestError::WatchContent { named: *named, views: implied_by.clone() }
                }
                // A basis supplied whole was not named on any axis; its analyzers are what
                // the caller chose, so the refusal speaks of them as named.
                Origin::Supplied => {
                    RequestError::WatchContent { named: self.basis.content, views: Vec::new() }
                }
            });
        }
        if delivery.stale_ok {
            return Err(RequestError::WatchCacheOnly);
        }
        Ok(())
    }

    /// [`Self::validate`]'s rules against `basis`, which `holder` fixed: the holder only
    /// words the remedy of a refusal for missing analysis, and decides nothing.
    fn validate_against(&self, basis: &Basis, holder: BasisHolder) -> Result<(), RequestError> {
        // Capability first, and against the scope the request names rather than against
        // whatever a holder retained: a scope this build cannot honour has no answer at any
        // delivery, so the refusal must not wait for a scan that a cache-only read never
        // runs, nor for a snapshot that a cold read never loads.
        if let Some(axis) = self.basis.scope.unsupported_axis() {
            return Err(RequestError::ScopeUnsupported { axis, reason: axis.reason() });
        }
        let views = self.query.views.len().saturating_add(self.query.omitted_views.len());
        if views > crate::MAX_REPORT_VIEWS {
            return Err(RequestError::ViewLimit {
                attempted: views,
                limit: crate::MAX_REPORT_VIEWS,
            });
        }
        let format = self.query.format;
        if matches!(
            format,
            crate::report_format::Format::Tree
                | crate::report_format::Format::Paths
                | crate::report_format::Format::Long
        ) {
            let compatible = self.query.views.len() == 1
                && self.query.views.iter().all(|view| {
                    matches!(view, ViewSpec::List | ViewSpec::Tree | ViewSpec::Files)
                        || (format != crate::report_format::Format::Tree
                            && matches!(view, ViewSpec::Largest | ViewSpec::Recent))
                });
            if !compatible {
                return Err(invalid(
                    self.query.axes.format,
                    format.label(),
                    "requires a single list view; use text or a machine format for aggregate/mixed views (largest/recent support paths and long)",
                ));
            }
        }
        check_views(&self.query.views, basis.content, holder)?;
        if let Some(SortKey::Metric(name)) = self.query.selection.sort {
            let metric =
                crate::content::METRICS.iter().find(|metric| metric.name == name).ok_or_else(
                    || invalid(self.query.axes.sort, name, "expected a registered numeric metric"),
                )?;
            if self.query.views.contains(&ViewSpec::Extensions) {
                return Err(invalid(
                    self.query.axes.sort,
                    name,
                    "extensions cannot sort by content metrics; use size, count, or name, or select files or another metric-capable view",
                ));
            }
            // A sort orders rows; it never says what the report is about, so unlike a
            // content view it implies nothing and is refused on every basis.
            if !basis.content.contains(metric.owner) {
                return Err(RequestError::SortNeedsAnalyzer {
                    metric: metric.name,
                    analyzer: metric.owner,
                    held: basis.content,
                    holder,
                });
            }
        }
        let hierarchy = self.query.views.iter().any(|view| self.query.tree_for(*view));
        // Neutral bounds compose across views, including the CLI --full shorthand.
        // A finite hierarchy bound on flat output would promise filtering it cannot do.
        if matches!(self.query.selection.depth, Some(Bound::Limit(_))) && !hierarchy {
            return Err(invalid(self.query.axes.depth, "", "requires a hierarchical view"));
        }
        if matches!(self.query.selection.breadth, Some(Bound::Limit(_))) && !hierarchy {
            return Err(invalid(self.query.axes.breadth, "", "requires a hierarchical view"));
        }
        let additive = self.query.views.iter().any(|view| {
            self.query.tree_for(*view)
                || matches!(
                    view,
                    ViewSpec::Extensions
                        | ViewSpec::Types
                        | ViewSpec::Families
                        | ViewSpec::Languages
                        | ViewSpec::Documents
                        | ViewSpec::Code
                )
        });
        if self.query.selection.min_share.as_ref().is_some_and(|share| !share.admits(0, 1))
            && !additive
        {
            return Err(invalid(self.query.axes.min_share, "", "requires an additive view"));
        }
        check_observation(basis.scope.population, basis.scope.read_controls)?;
        check_observation(self.query.selection.ignored, basis.scope.read_controls)?;
        if basis.scope.population != IgnoredEntries::Include
            && self.query.selection.ignored != basis.scope.population
        {
            return Err(RequestError::PopulationMismatch {
                held: basis.scope.population,
                requested: self.query.selection.ignored,
            });
        }
        Ok(())
    }
}

/// The analyzers a spec names, or the table's default.
fn parse_content(
    spec: &RequestSpec<'_>,
    axes: &'static AxisNames,
) -> Result<AnalysisSet, RequestError> {
    spec.analyze.map_or(Ok(Request::DEFAULTS.content), |value| {
        AnalysisSet::parse_rejecting(value).map_err(|rejection| rejection.on(axes.analyze))
    })
}

fn parse_population(
    value: Option<&str>,
    axes: &'static AxisNames,
) -> Result<IgnoredEntries, RequestError> {
    value.map_or(Ok(IgnoredEntries::Include), |value| {
        IgnoredEntries::parse(value)
            .map_err(|expected| Rejection::new(value, expected).on(axes.ignored))
    })
}

/// The scan scope a spec names, with the table's defaults for what it leaves out.
///
/// The delivery fields a `ScanConfig` still carries -- threads, batch size, order -- are
/// its own defaults: no answer depends on them, and they move into `Delivery` with
/// `Workers`.
fn parse_scope(spec: &RequestSpec<'_>, axes: &'static AxisNames) -> Result<Scope, RequestError> {
    let limits = Request::DEFAULTS.control_limits;
    Ok(Scope {
        max_depth: spec
            .scan_depth
            .map(|value| {
                value
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| invalid(axes.scan_depth, value, "expected a whole number"))
            })
            .transpose()?,
        one_filesystem: spec.one_filesystem,
        read_controls: spec.read_controls.unwrap_or(Request::DEFAULTS.read_controls),
        control_limits: ControlLimits {
            budget: spec.control_budget.map_or(Ok(limits.budget), |value| {
                parse_control_budget(value)
                    .map_err(|error| named_refusal(error, axes.control_budget))
            })?,
            line_limit: spec.control_line_limit.map_or(Ok(limits.line_limit), |value| {
                parse_control_line_limit(value)
                    .map_err(|error| named_refusal(error, axes.control_line_limit))
            })?,
        },
        ..Scope::default()
    })
}

/// The view axis a read names, in its grammar, before any analyzer resolves it.
///
/// An analyzer's name in the view axis is refused with the view that shows it, because it
/// is the one mistake the two vocabularies invite.
fn parse_views(spec: Option<&str>, axes: &'static AxisNames) -> Result<ViewList, RequestError> {
    ViewList::parse(spec).map_err(|rejection| {
        let suggestion = match rejection.value().to_ascii_lowercase().as_str() {
            "lines" => Some(("lines", ViewSpec::Families)),
            "words" => Some(("words", ViewSpec::Documents)),
            _ => None,
        };
        match suggestion {
            Some((analyzer, suggested_view)) => RequestError::AnalyzerNamedAsView {
                value: rejection.value().to_string(),
                analyzer,
                suggested_view,
            },
            None => rejection.on(axes.view),
        }
    })
}

/// The query one read supplies, with its views resolved against the analyzers its basis
/// holds.
///
/// `content` decides `full` and the view default and nothing else here: a request that
/// paid to read files displays what it read. Every value is parsed before any is checked
/// against another, in one order for every surface -- views ([`parse_views`], which each
/// caller runs first), selection, then the page denominator.
fn build_query(
    content: AnalysisSet,
    views: ViewList,
    spec: &ReadSpec<'_>,
    now: SystemTime,
    axes: &'static AxisNames,
) -> Result<Query, RequestError> {
    let (views, omitted_views) = views.resolve(content);

    let mut selection = Selection {
        depth: spec.depth.map(|value| parse_bound(value, axes.depth)).transpose()?,
        min_share: spec
            .min_share
            .map(|value| {
                ShareThreshold::parse(value).ok_or_else(|| {
                    invalid(axes.min_share, value, "expected a percentage from 0% through 100%")
                })
            })
            .transpose()?,
        breadth: spec.breadth.map(|value| parse_bound(value, axes.breadth)).transpose()?,
        limit: spec.limit.map(|value| parse_bound(value, axes.limit)).transpose()?,
        reverse: spec.reverse,
        size: spec
            .size
            .map_or(Ok(Request::DEFAULTS.size), |value| parse_size_metric(value, axes.size))?,
        ..Selection::default()
    };
    for pattern in spec.include {
        selection.include.push(Pattern::parse(pattern).map_err(grammar_refusal)?);
    }
    for pattern in spec.exclude {
        selection.exclude.push(Pattern::parse(pattern).map_err(grammar_refusal)?);
    }
    if let Some(value) = spec.min_size {
        selection.min_size = Some(parse_size(value).map_err(grammar_refusal)?);
    }
    if let Some(value) = spec.modified_since {
        let when = parse_when(value, now).map_err(grammar_refusal)?;
        selection.modified.since = Some(bound_nanos(value, when, axes.modified_since)?);
    }
    if let Some(value) = spec.modified_before {
        let when = parse_when(value, now).map_err(grammar_refusal)?;
        selection.modified.before = Some(bound_nanos(value, when, axes.modified_before)?);
    }
    if let Some(value) = spec.kinds {
        selection.kinds = parse_kinds(value, axes.kind)?;
    }
    if let Some(value) = spec.sort {
        selection.sort = Some(parse_sort(value, axes.sort)?);
    }
    selection.ignored = parse_population(spec.ignored, axes)?;
    let words_per_page =
        spec.words_per_page.map_or(Ok(Request::DEFAULTS.words_per_page), |value| {
            value
                .trim()
                .parse::<u64>()
                .ok()
                .filter(|words| *words > 0)
                .ok_or_else(|| invalid(axes.words_per_page, value, "expected a positive integer"))
        })?;

    let format = spec.format.map_or(Ok(crate::report_format::Format::Text), |value| {
        crate::report_format::Format::parse(value).ok_or_else(|| {
            invalid(
                axes.format,
                value,
                format!("expected one of {}", crate::report_format::Format::ALL.join(", ")),
            )
        })
    })?;
    Ok(Query { selection, views, format, omitted_views, axes, words_per_page })
}

/// A scan-scope axis a build may be unable to honour at all.
///
/// Not every axis a scan config carries: only the two whose support is a property of the
/// build rather than of the tree, so asking for one is a request no delivery can carry out
/// and no stored state can rescue. Which of them this build refuses is stated once, by
/// `ScanConfig::unsupported_axis`, and asked there by [`Request::validate`] for every route
/// and by the scan config's own validation for the engine-internal callers that never build
/// a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeAxis {
    /// Walking into what a symbolic link points at.
    FollowSymlinks,
    /// Keeping the walk on the root's own filesystem.
    OneFilesystem,
}

impl ScopeAxis {
    /// Why the axis has no supported semantics, in the library's field names.
    ///
    /// One sentence per axis, and the only one: the engine-internal callers that report
    /// [`Error::UnsupportedScanConfig`](crate::Error::UnsupportedScanConfig) print it
    /// verbatim, and [`RequestError::message`] prints it with the axis renamed, so no
    /// surface can drift from the rule by rewording its own copy.
    pub const fn reason(self) -> &'static str {
        match self {
            Self::FollowSymlinks => {
                "follow_symlinks requires cycle, root-boundary, and filesystem-boundary semantics"
            }
            Self::OneFilesystem => "one_filesystem requires platform device identity",
        }
    }

    /// How `axes` names the axis.
    const fn named(self, axes: &AxisNames) -> &'static str {
        match self {
            Self::FollowSymlinks => axes.follow_symlinks,
            Self::OneFilesystem => axes.one_filesystem,
        }
    }
}

/// Why a request cannot be answered as asked.
///
/// Typed so a caller can match the refusal it can act on, and rendered by
/// [`Self::message`] in the vocabulary of the surface the request came through.
///
/// Non-exhaustive, so a later refusal is an additive change: a match outside this crate
/// names the refusals it acts on and renders the rest with [`Self::message`].
/// [`Self::AnalyzerNamedAsView`], [`Self::ViewNeedsAnalyzer`], [`Self::SortNeedsAnalyzer`],
/// and [`Self::WatchContent`] are non-exhaustive too, so a field one of them gains later
/// is additive as well: a match outside this crate reads them with `..`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RequestError {
    /// A value did not match its axis's grammar.
    InvalidValue {
        /// The axis, as the requesting surface names it.
        axis: &'static str,
        /// The rejected value, as the grammar quotes it.
        value: String,
        /// What the grammar accepts instead.
        expected: String,
    },
    /// An analyzer name was supplied where a report view was expected.
    #[non_exhaustive]
    AnalyzerNamedAsView {
        /// The rejected token in the caller's spelling.
        value: String,
        /// The analyzer the token names.
        analyzer: &'static str,
        /// The view that shows that analyzer's results.
        suggested_view: ViewSpec,
    },
    /// A view shows analysis the basis does not hold.
    ///
    /// Only a basis the request did not build can raise this -- a retained index, an opened
    /// root, or one supplied whole -- because [`Request::build`] enables what its views
    /// imply, and a read never widens what a holder was opened with.
    #[non_exhaustive]
    ViewNeedsAnalyzer {
        /// The view the read named.
        view: ViewSpec,
        /// The analyzers the basis holds.
        held: AnalysisSet,
        /// What fixed those analyzers, which decides the remedy the refusal names.
        holder: BasisHolder,
    },
    /// A sort by a content metric whose analyzer the request did not enable.
    ///
    /// Unlike a content view, a sort implies nothing: it orders rows and never says what a
    /// report is about, so the analyzer must be named.
    #[non_exhaustive]
    SortNeedsAnalyzer {
        /// The metric, as the sort axis spells it.
        metric: &'static str,
        /// The analyzers that measure it.
        analyzer: AnalysisSet,
        /// The analyzers the basis holds.
        held: AnalysisSet,
        /// What fixed those analyzers, which decides the remedy the refusal names.
        holder: BasisHolder,
    },
    /// A selection by ignored state over a scan that observes no `.gitignore`.
    IgnoredWithoutObservation(IgnoredEntries),
    /// A retained narrow population cannot answer a read of another population.
    PopulationMismatch {
        /// Population retained by the holder.
        held: IgnoredEntries,
        /// Population this read selected.
        requested: IgnoredEntries,
    },
    /// A scan scope this build cannot honour, whatever the delivery.
    ScopeUnsupported {
        /// Which axis, so each surface names it in its own words.
        axis: ScopeAxis,
        /// The rule, one sentence, in the library's field names.
        reason: &'static str,
    },
    /// A read asks for another analyzer set than the retained index holds.
    ContentMismatch {
        /// The analyzers the index was built with.
        held: AnalysisSet,
        /// The analyzers the read asks for.
        requested: AnalysisSet,
    },
    /// A watch was asked to narrow its scan scope.
    WatchScope,
    /// A watch was asked to keep content analysis current.
    ///
    /// Names every axis that enabled analysis, so a caller who drops one is not refused
    /// again for the other.
    #[non_exhaustive]
    WatchContent {
        /// The analyzers the caller named, or that a basis supplied whole holds; `none`
        /// when every analyzer was implied by a view.
        named: AnalysisSet,
        /// The views that implied analysis, in the caller's order; empty when none did.
        views: Vec<ViewSpec>,
    },
    /// A watch was asked to start from a snapshot nothing verifies.
    WatchCacheOnly,
    /// A stale answer was asked of a delivery that never reads the snapshot.
    StaleOkCacheOff,
    /// An operation names another root than the retained index it would mutate.
    RootMismatch {
        /// Root held by the index.
        held: PathBuf,
        /// Root requested by the caller.
        requested: PathBuf,
    },
    /// A route cannot honor the requested execution policy.
    DeliveryUnsupported {
        /// The lifecycle that refuses it.
        route: &'static str,
        /// The unsupported policy combination.
        reason: &'static str,
    },
    /// A read names more views than one report may carry.
    ViewLimit {
        /// Views and omitted views the request carries.
        attempted: usize,
        /// The most one report accepts.
        limit: usize,
    },
    /// A report was asked about no root at all.
    NoRoots,
    /// One root lies inside another, so its paths would count twice ([`Roots`]).
    RootsOverlap {
        /// The contained root, by its label.
        inner: PathBuf,
        /// The containing root, by its label.
        outer: PathBuf,
    },
    /// Two roots are the same directory, by path or through an alias ([`Roots`]).
    RootsRepeated {
        /// The earlier of the two, by its label.
        first: PathBuf,
        /// The later of the two, by its label.
        second: PathBuf,
    },
}

impl RequestError {
    /// The refusal in the vocabulary of the surface `axes` describes.
    ///
    /// [`Self::InvalidValue`] already carries its axis, named by the surface that parsed
    /// it, so `axes` names only the knobs the other refusals point at.
    pub fn message(&self, axes: &AxisNames) -> String {
        match self {
            Self::InvalidValue { axis, value, expected } => invalid_message(axis, value, expected),
            // A view that implies its analyzer is the whole answer; one that does not needs
            // the analyzer named beside it.
            Self::AnalyzerNamedAsView { value, analyzer, suggested_view } => {
                let shown = if suggested_view.implies().is_enabled() {
                    String::new()
                } else {
                    format!(", with {} {analyzer}", axes.analyze)
                };
                format!(
                    "invalid {} {value:?}: {analyzer} is an analyzer; its view is {}{shown}",
                    axes.view,
                    suggested_view.label()
                )
            }
            Self::ViewNeedsAnalyzer { view, held, holder } => format!(
                "{} {} needs {} analysis{}",
                axes.view,
                view.label(),
                view.implies().named().join(" and "),
                holder.remedy(view.implies(), *held, axes)
            ),
            Self::SortNeedsAnalyzer { metric, analyzer, held, holder } => format!(
                "{} {metric} needs {} analysis{}",
                axes.sort,
                analyzer.named().join(" and "),
                holder.remedy(*analyzer, *held, axes)
            ),
            Self::IgnoredWithoutObservation(ignored) => format!(
                "{} needs .gitignore classification, and {} turned it off; drop one of them",
                match ignored {
                    IgnoredEntries::Exclude => axes.exclude_ignored,
                    IgnoredEntries::Only => axes.only_ignored,
                    IgnoredEntries::Include => axes.ignored,
                },
                axes.read_controls
            ),
            Self::PopulationMismatch { held, requested } => format!(
                "{}={} cannot be read from retained {}={} scope",
                axes.ignored,
                requested.label(),
                axes.ignored,
                held.label()
            ),
            // The sentence the engine has always printed, with only the axis renamed: a
            // Python caller reads the words they wrote, and the command line names its
            // flag. The kind stays in front of it, so this refusal is the same sentence
            // whichever door raised it.
            Self::ScopeUnsupported { axis, reason } => format!(
                "unsupported scan configuration: {}",
                reason.replacen(axis.named(&AxisNames::FIELDS), axis.named(axes), 1)
            ),
            Self::ContentMismatch { held, requested } => format!(
                "{analyze} {requested} cannot be answered by an index built with {analyze} \
                 {held}; open the root again with {analyze} {requested}",
                analyze = axes.analyze,
                requested = analysis_label(*requested),
                held = analysis_label(*held),
            ),
            Self::WatchScope => watch_scope_message(axes),
            Self::RootMismatch { held, requested } => {
                format!(
                    "requested root {} does not match retained root {}",
                    requested.display(),
                    held.display()
                )
            }
            Self::DeliveryUnsupported { route, reason } => format!("{route}: {reason}"),
            Self::WatchContent { views, .. } if views.is_empty() => format!(
                "{} is not yet supported with {}; use a one-shot report",
                axes.analyze, axes.watch
            ),
            Self::WatchContent { named, views } => watch_content_message(*named, views, axes),
            Self::WatchCacheOnly => format!(
                "{watch} cannot start from a {stale_ok} answer: nothing verifies what changed \
                 between the snapshot and the start of the watch; drop {stale_ok}",
                watch = axes.watch,
                stale_ok = axes.stale_ok,
            ),
            Self::StaleOkCacheOff => format!(
                "{stale_ok} answers from the snapshot, which {cache} off never reads; drop one \
                 of them",
                stale_ok = axes.stale_ok,
                cache = axes.cache,
            ),
            Self::ViewLimit { attempted, limit } => {
                format!("report request contains {attempted} views or omissions; limit is {limit}")
            }
            Self::NoRoots => "a report needs at least one root".to_owned(),
            Self::RootsOverlap { inner, outer } => {
                format!("{} is inside {}; name one or the other", inner.display(), outer.display())
            }
            Self::RootsRepeated { first, second } if first == second => {
                format!("{} is named twice; name it once", first.display())
            }
            Self::RootsRepeated { first, second } => format!(
                "{} is the same directory as {}; name one or the other",
                second.display(),
                first.display()
            ),
        }
    }
}

/// The library's vocabulary, as [`AxisNames::default`] is: a refusal rendered without
/// naming a surface belongs to a library caller, not to the command line.
impl fmt::Display for RequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message(&AxisNames::FIELDS))
    }
}

impl std::error::Error for RequestError {}

/// A watch refused for analysis some view implied, naming each axis that enabled it.
///
/// The analyzers are listed in the order the caller wrote them: the named ones first, then
/// each view's in the views' order, so `--view documents,code` reads as documents needing
/// words and code needing code rather than pairing documents with code.
fn watch_content_message(named: AnalysisSet, views: &[ViewSpec], axes: &AxisNames) -> String {
    let mut analyzers = named.named();
    for name in views.iter().flat_map(|view| view.implies().named()) {
        if !analyzers.contains(&name) {
            analyzers.push(name);
        }
    }
    let views = views.iter().map(|view| view.label()).collect::<Vec<_>>().join(",");
    let subject = if named.is_enabled() {
        format!("{} {} and {} {views} need", axes.analyze, named.request_label(), axes.view)
    } else {
        format!("{} {views} needs", axes.view)
    };
    format!(
        "{subject} {} analysis, which {} cannot keep current; use a one-shot report",
        analyzers.join(" and "),
        axes.watch
    )
}

/// An analyzer set as its axis spells it back: `none`, or the analyzers joined by commas.
fn analysis_label(set: AnalysisSet) -> String {
    if set.is_enabled() { set.labels().join(",") } else { AnalysisSet::NONE_LABEL.to_string() }
}

/// The watch-scope rule, with each knob named as `axes` names it.
///
/// One pass over whole words of [`crate::scan::WATCH_SCOPE_GUIDANCE`], which is written
/// in the library's field names, never re-scanning a replacement. A sequential replace
/// does re-scan: `max_depth` becomes `--scan-depth`, and then `depth` matches inside it,
/// giving `--scan---depth` (fdu-7j6z). The command line carried this substitution
/// itself until the rule moved here.
fn watch_scope_message(axes: &AxisNames) -> String {
    let fields = &AxisNames::FIELDS;
    let vocabulary = [
        (fields.scan_depth, axes.scan_depth),
        (fields.one_filesystem, axes.one_filesystem),
        (fields.modified_since, axes.modified_since),
        (fields.depth, axes.depth),
        (fields.include, axes.include),
    ];
    let is_word = |character: char| character.is_ascii_alphanumeric() || character == '_';
    crate::scan::WATCH_SCOPE_GUIDANCE
        .split_inclusive(|character: char| !is_word(character))
        .map(|piece| {
            let end = piece.find(|character: char| !is_word(character)).unwrap_or(piece.len());
            let (word, tail) = piece.split_at(end);
            match vocabulary.iter().find(|(field, _)| *field == word) {
                Some((_, name)) => format!("{name}{tail}"),
                None => piece.to_string(),
            }
        })
        .collect()
}

/// Refuse a view the basis's analyzers cannot answer.
///
/// One containment test, against the table [`ViewSpec::implies`] states, so which views
/// need content is decided once, where a new view must decide it. A request that built its
/// own basis enabled what its views imply and always passes; only a basis it did not build,
/// which a read never widens, can fail it, and `holder` words the refusal for that route.
fn check_views(
    views: &[ViewSpec],
    content: AnalysisSet,
    holder: BasisHolder,
) -> Result<(), RequestError> {
    match views.iter().find(|view| !content.contains(view.implies())) {
        Some(view) => Err(RequestError::ViewNeedsAnalyzer { view: *view, held: content, holder }),
        None => Ok(()),
    }
}

/// Refuse a selection by ignored state when the scan observes no control state.
pub(crate) fn check_observation(
    ignored: IgnoredEntries,
    observes_controls: bool,
) -> Result<(), RequestError> {
    match ignored {
        IgnoredEntries::Include => Ok(()),
        IgnoredEntries::Exclude | IgnoredEntries::Only if observes_controls => Ok(()),
        IgnoredEntries::Exclude | IgnoredEntries::Only => {
            Err(RequestError::IgnoredWithoutObservation(ignored))
        }
    }
}

/// A value a grammar refused, before any surface has named the axis.
///
/// The list grammars that predate this model ([`ViewSpec::resolve`] and
/// [`AnalysisSet::parse_labeled`]) take a free-form label and return a sentence; they
/// produce this instead, so the request model can put a typed axis on it and they can go on
/// rendering the identical sentence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Rejection {
    value: String,
    expected: String,
}

impl Rejection {
    pub(crate) fn new(value: impl Into<String>, expected: impl Into<String>) -> Self {
        Self { value: value.into(), expected: expected.into() }
    }

    pub(crate) fn value(&self) -> &str {
        &self.value
    }

    /// The refusal, on the axis a surface named.
    pub(crate) fn on(self, axis: &'static str) -> RequestError {
        RequestError::InvalidValue { axis, value: self.value, expected: self.expected }
    }

    /// The refusal's sentence, for a caller holding only a label.
    pub(crate) fn labeled(&self, label: &str) -> String {
        invalid_message(label, &self.value, &self.expected)
    }
}

fn invalid_message(axis: &str, value: &str, expected: &str) -> String {
    format!("invalid {axis} {value:?}: {expected}")
}

fn invalid(
    axis: &'static str,
    value: impl Into<String>,
    expected: impl Into<String>,
) -> RequestError {
    Rejection::new(value, expected).on(axis)
}

/// An engine value-grammar error, which already names its grammar: `invalid size "10X"`
/// and `invalid pattern` read the same on every surface.
fn grammar_refusal(error: crate::Error) -> RequestError {
    match error {
        crate::Error::InvalidValue { kind, value, hint } => invalid(kind, value, hint),
        other => invalid("value", String::new(), other.to_string()),
    }
}

/// An engine value-grammar error on a knob each surface names, as the `.gitignore` limits
/// are: `--gitignore-budget` and `control_budget`, never `control budget`.
fn named_refusal(error: crate::Error, axis: &'static str) -> RequestError {
    match error {
        crate::Error::InvalidValue { value, hint, .. } => invalid(axis, value, hint),
        other => invalid(axis, String::new(), other.to_string()),
    }
}

/// Parse one entry kind: `file`, `dir`, `symlink`, or `other`.
///
/// `axis` names the knob as the calling surface spells it: `--kind` or `kind`.
pub fn parse_kind(value: &str, axis: &'static str) -> Result<EntryKind, RequestError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "file" => Ok(EntryKind::File),
        "dir" => Ok(EntryKind::Dir),
        "symlink" => Ok(EntryKind::Symlink),
        "other" => Ok(EntryKind::Other),
        other => Err(invalid(axis, other, "expected one of file, dir, symlink, other")),
    }
}

/// Parse a comma-separated list of entry kinds.
///
/// Closed vocabularies are comma lists and open pattern values are repeatable, because
/// glob brace syntax (`*.{rs,toml}`) contains commas and would be shredded by a split.
/// An empty entry and a repeated kind are errors rather than silent no-ops, as they are
/// for views: repeating a value is far more likely to be a typo than an intention.
pub fn parse_kinds(list: &str, axis: &'static str) -> Result<Vec<EntryKind>, RequestError> {
    let mut kinds = Vec::new();
    for token in list.split(',') {
        let token = token.trim();
        if token.is_empty() {
            return Err(invalid(axis, list, "empty entry in the list"));
        }
        let kind = parse_kind(token, axis)?;
        if kinds.contains(&kind) {
            return Err(invalid(axis, list, format!("{token:?} appears more than once")));
        }
        kinds.push(kind);
    }
    Ok(kinds)
}

/// Parse a bound that accepts `all` for unbounded, as `--depth` and `--limit` are written.
pub fn parse_bound(value: &str, axis: &'static str) -> Result<Bound, RequestError> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("all") {
        return Ok(Bound::All);
    }
    value
        .parse::<usize>()
        .map(Bound::Limit)
        .map_err(|_| invalid(axis, value, "expected a whole number or `all`"))
}

/// Parse a metadata ordering key or a registered numeric content metric.
pub fn parse_sort(value: &str, axis: &'static str) -> Result<SortKey, RequestError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "size" => Ok(SortKey::Size),
        "count" => Ok(SortKey::Count),
        "mtime" => Ok(SortKey::Mtime),
        "name" => Ok(SortKey::Name),
        other => crate::content::METRICS.iter().find(|metric| metric.name == other).map_or_else(
            || {
                Err(invalid(
                    axis,
                    other,
                    "expected size, count, mtime, name, or a registered numeric metric",
                ))
            },
            |metric| Ok(SortKey::Metric(metric.name)),
        ),
    }
}

/// Parse a size metric: `allocated` or `apparent`.
pub fn parse_size_metric(value: &str, axis: &'static str) -> Result<SizeMetric, RequestError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "allocated" => Ok(SizeMetric::Allocated),
        "apparent" => Ok(SizeMetric::Apparent),
        other => Err(invalid(axis, other, "expected allocated or apparent")),
    }
}

/// Convert a parsed time bound to index nanoseconds, or refuse the value.
///
/// [`system_time_to_nanos`] returns `None` for an instant outside the range the index can
/// represent (roughly 1677-2262). Storing that `None` would leave the bound unset, so the
/// query would run with no time filter at all while the caller believed one was active --
/// a silently wrong answer, which is worse than a refused value.
pub fn bound_nanos(value: &str, when: SystemTime, axis: &'static str) -> Result<i64, RequestError> {
    system_time_to_nanos(when).ok_or_else(|| {
        invalid(
            axis,
            value,
            "that time is outside the range fdu can represent (about 1677 to 2262)",
        )
    })
}

/// Parse a cache policy: `auto`, `on`, or `off`.
///
/// The three values that earlier releases also accepted are refused with the replacement,
/// since each still appears in scripts and a bare list of values would not say where
/// `only` went.
pub fn parse_cache_policy(value: &str, axis: &'static str) -> Result<CachePolicy, RequestError> {
    let stale_ok = if axis == AxisNames::FLAGS.cache {
        AxisNames::FLAGS.stale_ok
    } else {
        AxisNames::FIELDS.stale_ok
    };
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Ok(CachePolicy::Auto),
        "on" => Ok(CachePolicy::On),
        "off" => Ok(CachePolicy::Off),
        "only" => Err(invalid(
            axis,
            "only",
            format!("answering from the snapshot alone is now {stale_ok}"),
        )),
        "refresh" => Err(invalid(
            axis,
            "refresh",
            "removed; use on, which also writes after a one-shot report",
        )),
        "read-only" => Err(invalid(
            axis,
            "read-only",
            "removed; auto no longer writes after a one-shot metadata report, and off reads nothing",
        )),
        other => Err(invalid(axis, other, "expected one of auto, on, off")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_grammar_names_its_axis_as_the_surface_spells_it() {
        let flags = &AxisNames::FLAGS;
        let fields = &AxisNames::FIELDS;
        let cases: [(&str, RequestError, RequestError); 6] = [
            (
                "kind",
                parse_kind(" Socket ", flags.kind).expect_err("unknown kind"),
                parse_kind(" Socket ", fields.kind).expect_err("unknown kind"),
            ),
            (
                "bound",
                parse_bound("two", flags.depth).expect_err("not a number"),
                parse_bound("two", fields.depth).expect_err("not a number"),
            ),
            (
                "sort",
                parse_sort("Newest", flags.sort).expect_err("unknown key"),
                parse_sort("Newest", fields.sort).expect_err("unknown key"),
            ),
            (
                "size",
                parse_size_metric("logical", flags.size).expect_err("unknown metric"),
                parse_size_metric("logical", fields.size).expect_err("unknown metric"),
            ),
            (
                "time",
                bound_nanos("2300-01-01T00:00:00Z", far_future(), flags.modified_since)
                    .expect_err("unrepresentable"),
                bound_nanos("2300-01-01T00:00:00Z", far_future(), fields.modified_since)
                    .expect_err("unrepresentable"),
            ),
            (
                "cache",
                parse_cache_policy("readonly", flags.cache).expect_err("unreleased alias"),
                parse_cache_policy("readonly", fields.cache).expect_err("unreleased alias"),
            ),
        ];
        let expected = [
            (
                "invalid --kind \"socket\": expected one of file, dir, symlink, other",
                "invalid kind \"socket\": expected one of file, dir, symlink, other",
            ),
            (
                "invalid --depth \"two\": expected a whole number or `all`",
                "invalid depth \"two\": expected a whole number or `all`",
            ),
            (
                "invalid --sort \"newest\": expected size, count, mtime, name, or a registered numeric metric",
                "invalid sort \"newest\": expected size, count, mtime, name, or a registered numeric metric",
            ),
            (
                "invalid --size \"logical\": expected allocated or apparent",
                "invalid size \"logical\": expected allocated or apparent",
            ),
            (
                "invalid --modified-since \"2300-01-01T00:00:00Z\": that time is outside the \
                 range fdu can represent (about 1677 to 2262)",
                "invalid modified_since \"2300-01-01T00:00:00Z\": that time is outside the range \
                 fdu can represent (about 1677 to 2262)",
            ),
            (
                "invalid --cache \"readonly\": expected one of auto, on, off",
                "invalid cache policy \"readonly\": expected one of auto, on, off",
            ),
        ];
        for ((grammar, flag, field), (flag_text, field_text)) in cases.into_iter().zip(expected) {
            assert_eq!(flag.message(&AxisNames::FLAGS), flag_text, "{grammar}");
            assert_eq!(field.message(&AxisNames::FIELDS), field_text, "{grammar}");
            // The axis travels with the value, so rendering never re-names it.
            assert_eq!(flag.message(&AxisNames::FIELDS), flag_text, "{grammar}");
        }
    }

    fn far_future() -> SystemTime {
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(10_000_000_000)
    }

    #[test]
    fn the_grammars_accept_their_whole_vocabulary() {
        let axis = AxisNames::FIELDS.kind;
        for (spelling, kind) in [
            ("file", EntryKind::File),
            ("DIR", EntryKind::Dir),
            (" symlink", EntryKind::Symlink),
            ("other", EntryKind::Other),
        ] {
            assert_eq!(parse_kind(spelling, axis), Ok(kind));
        }
        assert_eq!(parse_kinds("file, dir", axis), Ok(vec![EntryKind::File, EntryKind::Dir]));
        assert_eq!(parse_bound(" ALL ", axis), Ok(Bound::All));
        assert_eq!(parse_bound("0", axis), Ok(Bound::Limit(0)));
        for (spelling, key) in [
            ("size", SortKey::Size),
            ("count", SortKey::Count),
            ("MTIME", SortKey::Mtime),
            ("name", SortKey::Name),
            ("CODE_LINES", SortKey::Metric("code_lines")),
        ] {
            assert_eq!(parse_sort(spelling, axis), Ok(key));
        }
        assert_eq!(parse_size_metric("Allocated", axis), Ok(SizeMetric::Allocated));
        assert_eq!(parse_size_metric("apparent", axis), Ok(SizeMetric::Apparent));
        for (spelling, policy) in
            [("auto", CachePolicy::Auto), ("ON", CachePolicy::On), ("off", CachePolicy::Off)]
        {
            assert_eq!(parse_cache_policy(spelling, axis), Ok(policy));
        }
        // The retired values name their replacement, in each surface's own words.
        for (spelling, flags, fields) in [
            (
                "only",
                "invalid --cache \"only\": answering from the snapshot alone is now --stale-ok",
                "invalid cache policy \"only\": answering from the snapshot alone is now stale_ok",
            ),
            (
                "refresh",
                "invalid --cache \"refresh\": removed; use on, which also writes after a one-shot \
                 report",
                "invalid cache policy \"refresh\": removed; use on, which also writes after a \
                 one-shot report",
            ),
            (
                "read-only",
                "invalid --cache \"read-only\": removed; auto no longer writes after a one-shot \
                 metadata report, and off reads nothing",
                "invalid cache policy \"read-only\": removed; auto no longer writes after a \
                 one-shot metadata report, and off reads nothing",
            ),
        ] {
            let message = |axis| {
                parse_cache_policy(spelling, axis).expect_err("retired").message(&AxisNames::FLAGS)
            };
            assert_eq!(message(AxisNames::FLAGS.cache), flags);
            assert_eq!(message(AxisNames::FIELDS.cache), fields);
        }
        let epoch = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(2);
        assert_eq!(bound_nanos("@2", epoch, axis), Ok(2_000_000_000));
    }

    #[test]
    fn a_kind_list_refuses_empty_and_repeated_entries() {
        let axis = AxisNames::FLAGS.kind;
        assert_eq!(
            parse_kinds("file,,dir", axis).map_err(|error| error.message(&AxisNames::FLAGS)),
            Err("invalid --kind \"file,,dir\": empty entry in the list".to_string())
        );
        assert_eq!(
            parse_kinds("file, FILE", axis).map_err(|error| error.message(&AxisNames::FLAGS)),
            Err("invalid --kind \"file, FILE\": \"FILE\" appears more than once".to_string())
        );
    }

    /// Every refusal, in both vocabularies, quoted whole: the wording is the contract the
    /// goldens and the parity harness hold each surface to.
    #[test]
    fn every_refusal_renders_in_flag_and_field_wording() {
        let cases = [
            (
                RequestError::ViewNeedsAnalyzer {
                    view: ViewSpec::Code,
                    held: AnalysisSet::NONE,
                    holder: BasisHolder::Index,
                },
                "--view code needs code analysis; this index was opened with --analyze none",
                "view code needs code analysis; this index was opened with analyze none",
            ),
            (
                RequestError::ViewNeedsAnalyzer {
                    view: ViewSpec::Documents,
                    held: AnalysisSet::CODE_ONLY,
                    holder: BasisHolder::Index,
                },
                "--view documents needs words analysis; this index was opened with --analyze code",
                "view documents needs words analysis; this index was opened with analyze code",
            ),
            // Each holder names the remedy its route accepts: an opened root has no analyzer
            // option, and a basis supplied whole was opened by nothing.
            (
                RequestError::ViewNeedsAnalyzer {
                    view: ViewSpec::Documents,
                    held: AnalysisSet::NONE,
                    holder: BasisHolder::OpenedRoot,
                },
                "--view documents needs words analysis, which an opened root never runs; use a \
                 one-shot report, or an index opened with --analyze words",
                "view documents needs words analysis, which an opened root never runs; use a \
                 one-shot report, or an index opened with analyze words",
            ),
            (
                RequestError::ViewNeedsAnalyzer {
                    view: ViewSpec::Code,
                    held: AnalysisSet::WORDS_ONLY,
                    holder: BasisHolder::Supplied,
                },
                "--view code needs code analysis; its basis holds --analyze words",
                "view code needs code analysis; its basis holds analyze words",
            ),
            (
                RequestError::SortNeedsAnalyzer {
                    metric: "code_lines",
                    analyzer: AnalysisSet::CODE_ONLY,
                    held: AnalysisSet::NONE,
                    holder: BasisHolder::Built,
                },
                "--sort code_lines needs code analysis: add --analyze code",
                "sort code_lines needs code analysis: add analyze code",
            ),
            (
                RequestError::SortNeedsAnalyzer {
                    metric: "code_lines",
                    analyzer: AnalysisSet::CODE_ONLY,
                    held: AnalysisSet::LINES_ONLY,
                    holder: BasisHolder::Index,
                },
                "--sort code_lines needs code analysis; this index was opened with --analyze \
                 lines",
                "sort code_lines needs code analysis; this index was opened with analyze lines",
            ),
            (
                RequestError::SortNeedsAnalyzer {
                    metric: "document_words",
                    analyzer: AnalysisSet::WORDS_ONLY,
                    held: AnalysisSet::NONE,
                    holder: BasisHolder::OpenedRoot,
                },
                "--sort document_words needs words analysis, which an opened root never runs; \
                 use a one-shot report, or an index opened with --analyze words",
                "sort document_words needs words analysis, which an opened root never runs; use \
                 a one-shot report, or an index opened with analyze words",
            ),
            (
                RequestError::SortNeedsAnalyzer {
                    metric: "code_lines",
                    analyzer: AnalysisSet::CODE_ONLY,
                    held: AnalysisSet::NONE,
                    holder: BasisHolder::Supplied,
                },
                "--sort code_lines needs code analysis; its basis holds --analyze none",
                "sort code_lines needs code analysis; its basis holds analyze none",
            ),
            (
                RequestError::AnalyzerNamedAsView {
                    value: "words".to_owned(),
                    analyzer: "words",
                    suggested_view: ViewSpec::Documents,
                },
                "invalid --view \"words\": words is an analyzer; its view is documents",
                "invalid view \"words\": words is an analyzer; its view is documents",
            ),
            (
                RequestError::AnalyzerNamedAsView {
                    value: "lines".to_owned(),
                    analyzer: "lines",
                    suggested_view: ViewSpec::Families,
                },
                "invalid --view \"lines\": lines is an analyzer; its view is families, with \
                 --analyze lines",
                "invalid view \"lines\": lines is an analyzer; its view is families, with \
                 analyze lines",
            ),
            (
                RequestError::IgnoredWithoutObservation(IgnoredEntries::Exclude),
                "--ignored=exclude needs .gitignore classification, and --no-gitignore turned it \
                 off; drop one of them",
                "ignored=exclude needs .gitignore classification, and read_controls turned it \
                 off; drop one of them",
            ),
            (
                RequestError::IgnoredWithoutObservation(IgnoredEntries::Only),
                "--ignored=only needs .gitignore classification, and --no-gitignore turned it \
                 off; drop one of them",
                "ignored=only needs .gitignore classification, and read_controls turned it off; \
                 drop one of them",
            ),
            (
                RequestError::ContentMismatch {
                    held: AnalysisSet::NONE,
                    requested: AnalysisSet::NONE.with_code(),
                },
                "--analyze lines,code cannot be answered by an index built with --analyze none; \
                 open the root again with --analyze lines,code",
                "analyze lines,code cannot be answered by an index built with analyze none; open \
                 the root again with analyze lines,code",
            ),
            (
                RequestError::ScopeUnsupported {
                    axis: ScopeAxis::OneFilesystem,
                    reason: ScopeAxis::OneFilesystem.reason(),
                },
                "unsupported scan configuration: --one-filesystem requires platform device \
                 identity",
                "unsupported scan configuration: one_filesystem requires platform device identity",
            ),
            (
                RequestError::ScopeUnsupported {
                    axis: ScopeAxis::FollowSymlinks,
                    reason: ScopeAxis::FollowSymlinks.reason(),
                },
                "unsupported scan configuration: follow_symlinks requires cycle, root-boundary, \
                 and filesystem-boundary semantics",
                "unsupported scan configuration: follow_symlinks requires cycle, root-boundary, \
                 and filesystem-boundary semantics",
            ),
            (
                RequestError::WatchContent { named: AnalysisSet::LINES_ONLY, views: Vec::new() },
                "--analyze is not yet supported with --watch; use a one-shot report",
                "analyze is not yet supported with watch; use a one-shot report",
            ),
            (
                RequestError::WatchContent {
                    named: AnalysisSet::NONE,
                    views: vec![ViewSpec::Code],
                },
                "--view code needs code analysis, which --watch cannot keep current; use a \
                 one-shot report",
                "view code needs code analysis, which watch cannot keep current; use a one-shot \
                 report",
            ),
            // The analyzers follow the views' order, so documents reads as needing words.
            (
                RequestError::WatchContent {
                    named: AnalysisSet::NONE,
                    views: vec![ViewSpec::Documents, ViewSpec::Code],
                },
                "--view documents,code needs words and code analysis, which --watch cannot keep \
                 current; use a one-shot report",
                "view documents,code needs words and code analysis, which watch cannot keep \
                 current; use a one-shot report",
            ),
            // Both axes, so dropping the view alone is not refused again for the analyzer.
            (
                RequestError::WatchContent {
                    named: AnalysisSet::WORDS_ONLY,
                    views: vec![ViewSpec::Code],
                },
                "--analyze words and --view code need words and code analysis, which --watch \
                 cannot keep current; use a one-shot report",
                "analyze words and view code need words and code analysis, which watch cannot \
                 keep current; use a one-shot report",
            ),
            (
                RequestError::WatchCacheOnly,
                "--watch cannot start from a --stale-ok answer: nothing verifies what changed \
                 between the snapshot and the start of the watch; drop --stale-ok",
                "watch cannot start from a stale_ok answer: nothing verifies what changed between \
                 the snapshot and the start of the watch; drop stale_ok",
            ),
            (
                RequestError::StaleOkCacheOff,
                "--stale-ok answers from the snapshot, which --cache off never reads; drop one of \
                 them",
                "stale_ok answers from the snapshot, which cache policy off never reads; drop one \
                 of them",
            ),
            (
                RequestError::ViewLimit { attempted: 17, limit: 16 },
                "report request contains 17 views or omissions; limit is 16",
                "report request contains 17 views or omissions; limit is 16",
            ),
            (
                RequestError::NoRoots,
                "a report needs at least one root",
                "a report needs at least one root",
            ),
            (
                RequestError::RootsOverlap {
                    inner: PathBuf::from("src/core"),
                    outer: PathBuf::from("src"),
                },
                "src/core is inside src; name one or the other",
                "src/core is inside src; name one or the other",
            ),
            // One spelling twice names it once; two spellings of one directory name both.
            (
                RequestError::RootsRepeated {
                    first: PathBuf::from("docs"),
                    second: PathBuf::from("docs"),
                },
                "docs is named twice; name it once",
                "docs is named twice; name it once",
            ),
            (
                RequestError::RootsRepeated {
                    first: PathBuf::from("docs"),
                    second: PathBuf::from("./docs"),
                },
                "./docs is the same directory as docs; name one or the other",
                "./docs is the same directory as docs; name one or the other",
            ),
        ];
        for (refusal, flags, fields) in cases {
            assert_eq!(refusal.message(&AxisNames::FLAGS), flags);
            assert_eq!(refusal.message(&AxisNames::FIELDS), fields);
            assert_eq!(refusal.to_string(), fields, "a refusal displays in the library's words");
        }
    }

    /// The watch-scope rule is the library's constant in the library's words, and the
    /// command line's words differ by knob names alone.
    ///
    /// Asserted against the constant rather than by quoting prose, so a rewording of the
    /// rule cannot leave this test measuring its own copy of it.
    #[test]
    fn the_watch_scope_refusal_substitutes_whole_words_only() {
        let source = crate::scan::WATCH_SCOPE_GUIDANCE;
        assert_eq!(RequestError::WatchScope.message(&AxisNames::FIELDS), source);

        let text = RequestError::WatchScope.message(&AxisNames::FLAGS);
        assert!(!text.contains("---"), "{text} re-substituted a replacement");

        // Hyphens stay inside a token, so `--scan-depth` is one word and not three.
        let names_word = |haystack: &str, word: &str| {
            haystack
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
                .any(|w| w == word)
        };
        let (fields, flags) = (&AxisNames::FIELDS, &AxisNames::FLAGS);
        let vocabulary = [
            (fields.scan_depth, flags.scan_depth),
            (fields.one_filesystem, flags.one_filesystem),
            (fields.modified_since, flags.modified_since),
            (fields.depth, flags.depth),
            (fields.include, flags.include),
        ];
        for (field, flag) in vocabulary {
            if names_word(source, field) {
                assert!(text.contains(flag), "{text} must name {flag} where the rule says {field}");
            }
            assert!(!names_word(&text, field), "{text} still names {field} untranslated");
        }
        let mut rebuilt = text.clone();
        for (field, flag) in vocabulary {
            rebuilt = rebuilt.replace(flag, field);
        }
        assert_eq!(rebuilt, source, "the flag wording must be the library's, knob names aside");
    }

    /// Code needs the code analyzer and documents the words analyzer; every other view
    /// answers from any basis. Lines alone answers neither: raw words by document format
    /// were a cheaper approximation nobody asks for by name.
    #[test]
    fn documents_and_code_are_the_views_that_need_content() {
        for content in [
            AnalysisSet::NONE,
            AnalysisSet::LINES_ONLY,
            AnalysisSet::CODE_ONLY,
            AnalysisSet::WORDS_ONLY,
            AnalysisSet::ALL,
        ] {
            for view in ViewSpec::ALL {
                let refused = check_views(&[view], content, BasisHolder::Index).is_err();
                assert_eq!(
                    refused,
                    (view == ViewSpec::Documents && !content.includes_words())
                        || (view == ViewSpec::Code && !content.includes_code()),
                    "{view:?} over {content:?}"
                );
            }
        }
        assert_eq!(check_observation(IgnoredEntries::Include, false), Ok(()));
        for ignored in [IgnoredEntries::Exclude, IgnoredEntries::Only] {
            assert_eq!(check_observation(ignored, true), Ok(()));
            assert_eq!(
                check_observation(ignored, false),
                Err(RequestError::IgnoredWithoutObservation(ignored))
            );
        }
    }

    // ---- the request model ----

    use crate::content::AnalysisRequest;

    fn root() -> &'static Path {
        Path::new("/tree")
    }

    fn instant() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_800_000_000)
    }

    /// A spec over the test root whose basis is every default and whose read is `read`.
    #[allow(clippy::large_types_passed_by_value)]
    fn reading(read: ReadSpec<'_>) -> RequestSpec<'_> {
        RequestSpec { read, ..RequestSpec::new(root()) }
    }

    fn built(spec: &RequestSpec<'_>) -> Request {
        Request::build(spec, instant(), &AxisNames::FIELDS).expect("the spec parses")
    }

    #[test]
    fn retained_population_allows_narrower_reads_only_from_include() {
        let read = |basis: Basis, ignored| {
            Request::read(
                basis,
                &ReadSpec { ignored, ..ReadSpec::default() },
                instant(),
                &AxisNames::FIELDS,
            )
        };
        let include = Basis {
            root: root().to_path_buf(),
            scope: Scope::default(),
            content: AnalysisSet::NONE,
        };
        assert_eq!(
            read(include.clone(), Some("exclude")).expect("narrow exclude").query.selection.ignored,
            IgnoredEntries::Exclude
        );
        assert_eq!(
            read(include, Some("only")).expect("narrow only").query.selection.ignored,
            IgnoredEntries::Only
        );

        let excluded = Basis {
            root: root().to_path_buf(),
            scope: Scope { population: IgnoredEntries::Exclude, ..Scope::default() },
            content: AnalysisSet::NONE,
        };
        assert_eq!(
            read(excluded.clone(), None).expect("basis default").query.selection.ignored,
            IgnoredEntries::Exclude
        );
        assert!(matches!(
            read(excluded, Some("only")),
            Err(RequestError::PopulationMismatch {
                held: IgnoredEntries::Exclude,
                requested: IgnoredEntries::Only
            })
        ));
    }

    fn refusal(spec: &RequestSpec<'_>, axes: &'static AxisNames) -> String {
        Request::build(spec, instant(), axes).expect_err("the spec is refused").message(axes)
    }

    /// A spec that names nothing builds the table, and every type that also declares a
    /// default for one of these axes agrees with it.
    #[test]
    fn list_formats_are_resolved_and_validated_in_the_shared_request() {
        use crate::report_format::Format;
        for (view, format, valid, tree) in [
            (None, None, true, true),
            (Some("list"), Some("tree"), true, true),
            (Some("list"), Some("paths"), true, false),
            (None, Some("json"), true, false),
            (Some("files"), None, true, false),
            (Some("files"), Some("tree"), true, true),
            (Some("tree"), Some("long"), true, false),
            (Some("largest"), Some("long"), true, false),
            (Some("summary"), Some("long"), false, false),
            (Some("full"), Some("paths"), false, false),
            (Some("list,summary"), Some("tree"), false, false),
            (Some("list,summary"), Some("json"), true, false),
        ] {
            let spec = RequestSpec {
                read: ReadSpec { views: view, format, ..ReadSpec::new() },
                ..RequestSpec::new(Path::new("/absent"))
            };
            let request =
                Request::build(&spec, SystemTime::UNIX_EPOCH, &AxisNames::FIELDS).expect("grammar");
            assert_eq!(request.validate().is_ok(), valid, "{view:?} {format:?}");
            if valid {
                assert_eq!(request.query.tree_for(request.query.views[0]), tree);
            }
        }
        let spec = RequestSpec::new(Path::new("/absent"));
        let request =
            Request::build(&spec, SystemTime::UNIX_EPOCH, &AxisNames::FIELDS).expect("default");
        assert_eq!(request.query.views, [ViewSpec::List]);
        assert!(
            !request.query.needs_selection_walk(),
            "an ordinary tree must retain its bounded projection cost"
        );
        let mut flat = request.clone();
        flat.query.format = Format::Paths;
        assert!(flat.query.needs_selection_walk());
        assert_eq!(flat.query.limit_for(ViewSpec::List), Bound::All);
        assert_eq!(request.query.limit_for(ViewSpec::List), Bound::All);
        assert_eq!(request.query.depth_for(ViewSpec::List), Bound::Limit(5));
    }

    #[test]
    fn an_empty_spec_builds_the_defaults_table() {
        let defaults = Request::DEFAULTS;
        assert_eq!(defaults.size, SizeMetric::Allocated);
        assert_eq!(defaults.words_per_page, 250);
        assert_eq!(defaults.content, AnalysisSet::NONE);
        assert!(defaults.read_controls);
        assert_eq!(defaults.control_limits, ControlLimits::default());
        // A watch serves no content -- `WatchContent` refuses one that names an analyzer
        // -- so its view is the report default for none, derived rather than declared.
        assert_eq!(defaults.report_view(AnalysisSet::NONE), ViewSpec::List);
        for content in [
            AnalysisSet::NONE,
            AnalysisSet::NONE.with_lines(),
            AnalysisSet::NONE.with_code(),
            AnalysisSet::NONE.with_words(),
            AnalysisSet::ALL,
        ] {
            assert_eq!(defaults.report_view(content), ViewSpec::default_for(content));
        }

        let request = built(&RequestSpec::new(root()));
        assert_eq!(request.basis.root, root());
        assert_eq!(request.basis.content, defaults.content);
        assert_eq!(request.basis.scope.read_controls, defaults.read_controls);
        assert_eq!(request.basis.scope.control_limits, defaults.control_limits);
        assert_eq!(request.basis.scope.max_depth, None);
        assert!(!request.basis.scope.one_filesystem);
        assert_eq!(request.query.views, vec![defaults.report_view(defaults.content)]);
        assert!(request.query.omitted_views.is_empty());
        assert_eq!(request.query.words_per_page, defaults.words_per_page);
        assert_eq!(request.query.selection.size, defaults.size);
        assert!(request.query.selection.is_unfiltered());
        let selection = &request.query.selection;
        assert_eq!((selection.depth, selection.limit, selection.sort), (None, None, None));
        assert!(!selection.reverse);
        assert_eq!(*request.query.axes, AxisNames::FIELDS);
        assert_eq!(request.now, instant());
        request.validate().expect("the defaults are a valid request");

        // The other homes of these defaults read the table rather than restating it.
        assert_eq!(SizeMetric::default(), defaults.size);
        assert_eq!(Selection::default().size, defaults.size);
        assert_eq!(Query::default().words_per_page, defaults.words_per_page);
        assert_eq!(ScanConfig::default().read_controls, defaults.read_controls);
        assert_eq!(ScanConfig::default().control_limits, defaults.control_limits);
        assert_eq!(AnalysisRequest::default().profile, defaults.content);
        assert_eq!(AnalysisSet::default(), defaults.content);
    }

    #[test]
    fn every_axis_of_a_spec_reaches_its_typed_value() {
        let include = ["*.rs".to_string()];
        let exclude = ["target/**".to_string()];
        let spec = RequestSpec {
            scan_depth: Some("3"),
            one_filesystem: true,
            read_controls: Some(false),
            control_budget: Some("all"),
            control_line_limit: Some("64KiB"),
            analyze: Some("code"),
            read: ReadSpec {
                views: Some("languages,tree"),
                format: None,
                words_per_page: Some("300"),
                include: &include,
                exclude: &exclude,
                min_size: Some("1KiB"),
                modified_since: Some("@1700000000"),
                modified_before: Some("@1800000000"),
                kinds: Some("file,dir"),
                ignored: Some("include"),
                depth: Some("all"),
                min_share: Some("1%"),
                breadth: Some("all"),
                limit: Some("5"),
                sort: Some("name"),
                reverse: true,
                size: Some("apparent"),
            },
            ..RequestSpec::new(root())
        };
        let request = Request::build(&spec, instant(), &AxisNames::FLAGS).expect("parses");
        let scope = &request.basis.scope;
        assert_eq!(scope.max_depth, Some(3));
        assert!(scope.one_filesystem);
        assert!(!scope.read_controls);
        assert_eq!(scope.control_limits, ControlLimits { budget: None, line_limit: Some(65_536) });
        assert_eq!(request.basis.content, AnalysisSet::NONE.with_code());
        assert_eq!(request.query.views, vec![ViewSpec::Languages, ViewSpec::Tree]);
        assert_eq!(request.query.words_per_page, 300);
        assert_eq!(*request.query.axes, AxisNames::FLAGS);
        let selection = &request.query.selection;
        assert_eq!(selection.include.len(), 1);
        assert_eq!(selection.exclude.len(), 1);
        assert_eq!(selection.min_size, Some(1024));
        assert_eq!(selection.modified.since, Some(1_700_000_000_000_000_000));
        assert_eq!(selection.modified.before, Some(1_800_000_000_000_000_000));
        assert_eq!(selection.kinds, vec![EntryKind::File, EntryKind::Dir]);
        assert_eq!(selection.ignored, IgnoredEntries::Include);
        assert_eq!(selection.depth, Some(Bound::All));
        assert_eq!(selection.limit, Some(Bound::Limit(5)));
        assert_eq!(selection.sort, Some(SortKey::Name));
        assert!(selection.reverse);
        assert_eq!(selection.size, SizeMetric::Apparent);
    }

    /// Relative windows are resolved once, against the request's own instant, so the
    /// selection a request carries is absolute and building it again at the same instant
    /// selects exactly the same entries however much later that happens.
    #[test]
    fn relative_windows_resolve_against_the_requests_instant() {
        let spec = reading(ReadSpec {
            modified_since: Some("2h"),
            modified_before: Some("now"),
            ..ReadSpec::new()
        });
        let request = built(&spec);
        let now = system_time_to_nanos(instant()).expect("representable");
        let two_hours = 2 * 60 * 60 * 1_000_000_000;
        assert_eq!(request.now, instant());
        assert_eq!(request.query.selection.modified.since, Some(now - two_hours));
        assert_eq!(request.query.selection.modified.before, Some(now));

        let later = instant() + Duration::from_secs(60);
        let moved = Request::build(&spec, later, &AxisNames::FIELDS).expect("parses");
        assert_eq!(moved.query.selection.modified.since, Some(now - two_hours + 60_000_000_000));
        let again = built(&spec);
        assert_eq!(again.query.selection.modified.since, request.query.selection.modified.since);
    }

    /// Each refusal `build` raises reads exactly as the path it replaces does today, in
    /// both vocabularies.
    #[test]
    fn build_refuses_each_axis_in_the_surfaces_words() {
        let spec = RequestSpec::new(root());
        let analyze = RequestSpec { analyze: Some("deep"), ..spec };
        for (axes, label) in [(&AxisNames::FLAGS, "--analyze"), (&AxisNames::FIELDS, "analyze")] {
            let today = AnalysisSet::parse_labeled("deep", label).expect_err("refused");
            assert_eq!(refusal(&analyze, axes), today);
        }
        for views in ["tree,tree", "tree,,types", "full,tree", "bogus"] {
            let spec = reading(ReadSpec { views: Some(views), ..ReadSpec::new() });
            for (axes, label) in [(&AxisNames::FLAGS, "--view"), (&AxisNames::FIELDS, "view")] {
                let today =
                    ViewSpec::resolve(Some(views), AnalysisSet::NONE, label).expect_err("refused");
                assert_eq!(refusal(&spec, axes), today);
            }
        }

        let cases: [(RequestSpec<'_>, &str, &str); 7] = [
            (
                reading(ReadSpec { words_per_page: Some("0"), ..ReadSpec::new() }),
                "invalid --words-per-page \"0\": expected a positive integer",
                "invalid words_per_page \"0\": expected a positive integer",
            ),
            (
                RequestSpec { scan_depth: Some("deep"), ..spec },
                "invalid --scan-depth \"deep\": expected a whole number",
                "invalid max_depth \"deep\": expected a whole number",
            ),
            (
                RequestSpec { control_budget: Some("lots"), ..spec },
                "invalid --gitignore-budget \"lots\": expected a number before the unit, as in \
                 `10M`, or `all` for no bound",
                "invalid control_budget \"lots\": expected a number before the unit, as in \
                 `10M`, or `all` for no bound",
            ),
            (
                reading(ReadSpec { ignored: Some("maybe"), ..ReadSpec::new() }),
                "invalid --ignored \"maybe\": expected one of include, \
                 exclude, only",
                "invalid ignored \"maybe\": expected one of include, exclude, only",
            ),
            (
                reading(ReadSpec { kinds: Some("file,socket"), ..ReadSpec::new() }),
                "invalid --kind \"socket\": expected one of file, dir, symlink, other",
                "invalid kind \"socket\": expected one of file, dir, symlink, other",
            ),
            (
                reading(ReadSpec { min_size: Some("10X"), ..ReadSpec::new() }),
                "invalid size \"10X\": unknown size unit \"X\"; use B, K/KB, M/MB, G/GB, T/TB, \
                 P/PB, or the binary forms KiB, MiB, GiB, TiB, PiB",
                "invalid size \"10X\": unknown size unit \"X\"; use B, K/KB, M/MB, G/GB, T/TB, \
                 P/PB, or the binary forms KiB, MiB, GiB, TiB, PiB",
            ),
            (
                reading(ReadSpec {
                    modified_since: Some("2300-01-01T00:00:00Z"),
                    ..ReadSpec::new()
                }),
                "invalid --modified-since \"2300-01-01T00:00:00Z\": that time is outside the \
                 range fdu can represent (about 1677 to 2262)",
                "invalid modified_since \"2300-01-01T00:00:00Z\": that time is outside the range \
                 fdu can represent (about 1677 to 2262)",
            ),
        ];
        for (spec, flags, fields) in cases {
            assert_eq!(refusal(&spec, &AxisNames::FLAGS), flags);
            assert_eq!(refusal(&spec, &AxisNames::FIELDS), fields);
        }
    }

    /// An analyzer's name in the view axis is refused with the view that shows it: alone
    /// when that view implies the analyzer, and beside the analyzer when it does not.
    #[test]
    fn analyzer_names_in_view_axis_point_to_both_correct_axes() {
        for (token, flags, fields) in [
            (
                "LiNeS",
                "invalid --view \"LiNeS\": lines is an analyzer; its view is families, with \
                 --analyze lines",
                "invalid view \"LiNeS\": lines is an analyzer; its view is families, with analyze \
                 lines",
            ),
            (
                "words",
                "invalid --view \"words\": words is an analyzer; its view is documents",
                "invalid view \"words\": words is an analyzer; its view is documents",
            ),
        ] {
            let spec = reading(ReadSpec { views: Some(token), ..ReadSpec::new() });
            for (axes, expected) in [(&AxisNames::FLAGS, flags), (&AxisNames::FIELDS, fields)] {
                let error = Request::build(&spec, instant(), axes).expect_err("not a view");
                assert!(matches!(error, RequestError::AnalyzerNamedAsView { .. }));
                assert_eq!(error.message(axes), expected);
            }
        }
        let combination = reading(ReadSpec { views: Some("full,words"), ..ReadSpec::new() });
        assert_eq!(
            refusal(&combination, &AxisNames::FLAGS),
            ViewSpec::resolve(Some("full,words"), AnalysisSet::NONE, "--view")
                .expect_err("full is exclusive")
        );
    }

    /// A request that builds its own basis enables what its content views imply, in union
    /// with what `analyze` names; a view with a metadata meaning implies nothing.
    #[test]
    fn a_built_basis_enables_the_analyzers_its_views_imply() {
        /// `analyze`, `views`, the basis's analyzers, and the views that implied them.
        type Case = (Option<&'static str>, Option<&'static str>, AnalysisSet, &'static [ViewSpec]);
        let cases: [Case; 10] = [
            (None, Some("code"), AnalysisSet::CODE_ONLY, &[ViewSpec::Code]),
            (None, Some("documents"), AnalysisSet::WORDS_ONLY, &[ViewSpec::Documents]),
            (
                None,
                Some("code,documents"),
                AnalysisSet::ALL,
                &[ViewSpec::Code, ViewSpec::Documents],
            ),
            (None, Some("full"), AnalysisSet::NONE, &[]),
            (None, Some("languages"), AnalysisSet::NONE, &[]),
            (None, Some("types,families,tree"), AnalysisSet::NONE, &[]),
            (Some("lines"), Some("code"), AnalysisSet::CODE_ONLY, &[ViewSpec::Code]),
            (Some("lines"), Some("documents"), AnalysisSet::WORDS_ONLY, &[ViewSpec::Documents]),
            // `none` is the empty set, not a refusal: the views still imply.
            (Some("none"), Some("code"), AnalysisSet::CODE_ONLY, &[ViewSpec::Code]),
            (Some("words"), Some("summary,code"), AnalysisSet::ALL, &[ViewSpec::Code]),
        ];
        for (analyze, views, content, implied_by) in cases {
            let spec = RequestSpec {
                analyze,
                read: ReadSpec { views, ..ReadSpec::new() },
                ..RequestSpec::new(root())
            };
            let request = built(&spec);
            assert_eq!(request.basis.content, content, "{analyze:?} {views:?}");
            assert_eq!(request.implied_by(), implied_by, "{analyze:?} {views:?}");
            request.validate().expect("a built basis answers its own views");
        }

        // One basis whichever axis named the analyzer, so one sidecar and one answer.
        for (view, analyzer) in
            [("code", "code"), ("documents", "words"), ("code,documents", "all")]
        {
            let by_view = built(&reading(ReadSpec { views: Some(view), ..ReadSpec::new() }));
            let by_analyzer =
                built(&RequestSpec { analyze: Some(analyzer), ..RequestSpec::new(root()) });
            assert_eq!(by_view.basis.content, by_analyzer.basis.content, "{view}");
            assert_eq!(by_view.query.views, by_analyzer.query.views, "{view}");
            assert!(by_analyzer.implied_by().is_empty(), "{analyzer} implied nothing");
        }

        // `full` implies nothing, so alone it enables no analyzer: the metadata digest stays
        // a metadata report and names the content views it skipped.
        let full = built(&reading(ReadSpec { views: Some("full"), ..ReadSpec::new() }));
        assert_eq!(full.query.omitted_views, [ViewSpec::Code, ViewSpec::Documents]);
    }

    /// A read of a held basis is never widened: a view its analyzers cannot answer is
    /// refused, naming the view, its analyzer, and what the holder was opened with, in each
    /// surface's words.
    #[test]
    fn a_held_basis_refuses_a_view_it_cannot_answer() {
        let read = |held, views| {
            Request::read(
                basis(held, true),
                &ReadSpec { views: Some(views), ..ReadSpec::new() },
                instant(),
                &AxisNames::FIELDS,
            )
        };
        let refused = read(AnalysisSet::NONE, "code").expect_err("nothing analyzed code");
        assert_eq!(
            refused,
            RequestError::ViewNeedsAnalyzer {
                view: ViewSpec::Code,
                held: AnalysisSet::NONE,
                holder: BasisHolder::Index,
            }
        );
        assert_eq!(
            refused.message(&AxisNames::FIELDS),
            "view code needs code analysis; this index was opened with analyze none"
        );
        assert_eq!(
            refused.message(&AxisNames::FLAGS),
            "--view code needs code analysis; this index was opened with --analyze none"
        );
        let refused = read(AnalysisSet::LINES_ONLY, "summary,documents").expect_err("no words");
        assert_eq!(
            refused.message(&AxisNames::FIELDS),
            "view documents needs words analysis; this index was opened with analyze lines"
        );
        let held = read(AnalysisSet::WORDS_ONLY, "documents").expect("words answers documents");
        assert_eq!(held.basis.content, AnalysisSet::WORDS_ONLY, "a read never widens a basis");
        assert!(held.implied_by().is_empty(), "a read implies nothing");
    }

    /// Each route's refusal names a remedy that route accepts.
    ///
    /// The same missing analyzer, refused four ways: a built request can add it, an index
    /// says what it was opened with, an opened root has no analyzer to open with, and a
    /// basis a caller supplied whole says what it holds rather than which flag to add.
    #[test]
    fn a_refusal_for_missing_analysis_names_its_routes_remedy() {
        let documents = |holder| {
            let request = request_with(
                &[ViewSpec::Documents],
                Selection::default(),
                basis(AnalysisSet::NONE, true),
            );
            match holder {
                BasisHolder::Index => request.validate_read(&basis(AnalysisSet::NONE, true)),
                BasisHolder::OpenedRoot => request.validate_opened(),
                _ => request.validate(),
            }
            .expect_err("nothing analyzed words")
            .message(&AxisNames::FIELDS)
        };
        assert_eq!(
            documents(BasisHolder::Index),
            "view documents needs words analysis; this index was opened with analyze none"
        );
        assert_eq!(
            documents(BasisHolder::OpenedRoot),
            "view documents needs words analysis, which an opened root never runs; use a \
             one-shot report, or an index opened with analyze words"
        );
        assert_eq!(
            documents(BasisHolder::Supplied),
            "view documents needs words analysis; its basis holds analyze none"
        );
        // The grammar's own opened-root read refuses in the opened root's words too.
        assert_eq!(
            Request::read_opened(
                &ReadSpec { views: Some("documents"), ..ReadSpec::new() },
                instant(),
                &AxisNames::FIELDS,
            )
            .map_err(|error| error.message(&AxisNames::FIELDS))
            .map(|_| ()),
            Err(documents(BasisHolder::OpenedRoot))
        );

        // A sort implies nothing, so even a built request refuses it, and only there is
        // naming the analyzer to add the remedy.
        let sort = |analyze| RequestSpec {
            analyze,
            read: ReadSpec { views: Some("files"), sort: Some("code_lines"), ..ReadSpec::new() },
            ..RequestSpec::new(root())
        };
        assert_eq!(
            built(&sort(None)).validate().map_err(|error| error.message(&AxisNames::FLAGS)),
            Err("--sort code_lines needs code analysis: add --analyze code".to_owned())
        );
        let held = Request::read(
            basis(AnalysisSet::WORDS_ONLY, true),
            &ReadSpec { views: Some("files"), sort: Some("code_lines"), ..ReadSpec::new() },
            instant(),
            &AxisNames::FIELDS,
        )
        .expect_err("an index without code cannot sort by it");
        assert_eq!(
            held.message(&AxisNames::FIELDS),
            "sort code_lines needs code analysis; this index was opened with analyze words"
        );
    }

    /// The order `build` names axes in, when more than one of them is wrong.
    ///
    /// A contract, not an accident: every surface renders the first refusal and stops, so
    /// the order decides which mistake a caller is told about, and one that drifted would
    /// change what two doors say about the same command line. Pinned by fixing one axis at
    /// a time and watching the next one speak -- content, views, the selection, the page
    /// denominator, then scope, which is the order the doc comment publishes and the order
    /// the command line reads its flags.
    #[test]
    fn build_names_a_bad_axis_in_the_order_it_publishes() {
        let everything = RequestSpec {
            scan_depth: Some("deep"),
            analyze: Some("deep"),
            read: ReadSpec {
                views: Some("bogus"),
                depth: Some("two"),
                words_per_page: Some("0"),
                ..ReadSpec::new()
            },
            ..RequestSpec::new(root())
        };
        let steps: [(RequestSpec<'_>, &str); 5] = [
            (everything, "analyze"),
            (RequestSpec { analyze: None, ..everything }, "view"),
            (
                RequestSpec {
                    analyze: None,
                    read: ReadSpec { views: None, ..everything.read },
                    ..everything
                },
                "depth",
            ),
            (
                RequestSpec {
                    analyze: None,
                    read: ReadSpec { views: None, depth: None, ..everything.read },
                    ..everything
                },
                "words_per_page",
            ),
            (
                RequestSpec {
                    analyze: None,
                    read: ReadSpec {
                        views: None,
                        format: None,
                        depth: None,
                        words_per_page: None,
                        ..everything.read
                    },
                    ..everything
                },
                "max_depth",
            ),
        ];
        for (spec, axis) in steps {
            let refused = refusal(&spec, &AxisNames::FIELDS);
            assert!(
                refused.starts_with(&format!("invalid {axis} ")),
                "expected {axis} to speak next, got {refused}"
            );
        }
        // And the last step is the only thing still wrong, so fixing it parses.
        Request::build(
            &RequestSpec {
                scan_depth: None,
                analyze: None,
                read: ReadSpec::new(),
                ..RequestSpec::new(root())
            },
            instant(),
            &AxisNames::FIELDS,
        )
        .expect("nothing left to refuse");
    }

    fn request_with(views: &[ViewSpec], selection: Selection, basis: Basis) -> Request {
        Request::new(
            basis,
            Query { selection, views: views.to_vec(), ..Query::default() },
            instant(),
        )
    }

    fn basis(content: AnalysisSet, read_controls: bool) -> Basis {
        Basis {
            root: root().to_path_buf(),
            scope: Scope { read_controls, ..Scope::default() },
            content,
        }
    }

    /// Moved from the report reader, where a surface-only check stood in for the model.
    #[test]
    fn language_grouping_is_metadata_only_while_documents_require_analysis() {
        let enabled = [
            AnalysisSet::NONE.with_lines(),
            AnalysisSet::NONE.with_code(),
            AnalysisSet::NONE.with_words(),
            AnalysisSet::ALL,
        ];
        for content in std::iter::once(AnalysisSet::NONE).chain(enabled) {
            request_with(&[ViewSpec::Languages], Selection::default(), basis(content, true))
                .validate()
                .expect("language grouping never requires content I/O");
        }

        // A basis is never widened by a view: documents over a basis without words is
        // refused, naming what the basis holds.
        for held in [AnalysisSet::NONE, AnalysisSet::LINES_ONLY, AnalysisSet::CODE_ONLY] {
            let documents =
                request_with(&[ViewSpec::Documents], Selection::default(), basis(held, true));
            assert_eq!(
                documents.validate(),
                Err(RequestError::ViewNeedsAnalyzer {
                    view: ViewSpec::Documents,
                    held,
                    holder: BasisHolder::Supplied,
                })
            );
        }
        for content in [AnalysisSet::WORDS_ONLY, AnalysisSet::ALL] {
            request_with(&[ViewSpec::Documents], Selection::default(), basis(content, true))
                .validate()
                .expect("a basis with words answers the document view");
        }

        request_with(
            &[ViewSpec::Types, ViewSpec::Families],
            Selection::default(),
            basis(AnalysisSet::NONE, true),
        )
        .validate()
        .expect("metadata grouping never requires content I/O");
    }

    #[test]
    fn code_view_and_metric_sort_require_their_registered_analyzer() {
        let plain = basis(AnalysisSet::NONE, true);
        let code = request_with(&[ViewSpec::Code], Selection::default(), plain.clone());
        assert_eq!(
            code.validate(),
            Err(RequestError::ViewNeedsAnalyzer {
                view: ViewSpec::Code,
                held: AnalysisSet::NONE,
                holder: BasisHolder::Supplied,
            })
        );
        let selection =
            Selection { sort: Some(SortKey::Metric("code_lines")), ..Selection::default() };
        let files = request_with(&[ViewSpec::Files], selection.clone(), plain);
        assert_eq!(
            files.validate(),
            Err(RequestError::SortNeedsAnalyzer {
                metric: "code_lines",
                analyzer: AnalysisSet::CODE_ONLY,
                held: AnalysisSet::NONE,
                holder: BasisHolder::Supplied,
            })
        );
        request_with(&[ViewSpec::Files], selection, basis(AnalysisSet::NONE.with_code(), true))
            .validate()
            .expect("code metrics are available with the code analyzer");
    }

    #[test]
    fn extension_view_refuses_metric_sort_before_reading() {
        let held = basis(AnalysisSet::NONE.with_code(), true);
        let selection =
            Selection { sort: Some(SortKey::Metric("code_lines")), ..Selection::default() };
        let request = request_with(&[ViewSpec::Extensions], selection.clone(), held.clone());
        let expected = invalid(
            request.query.axes.sort,
            "code_lines",
            "extensions cannot sort by content metrics; use size, count, or name, or select files or another metric-capable view",
        );
        assert_eq!(request.validate(), Err(expected.clone()));
        assert_eq!(request.validate_read(&held), Err(expected));
        request_with(&[ViewSpec::Files], selection, held)
            .validate()
            .expect("files retain metric sorting");
    }

    /// A scope this build cannot honour is refused by request validation itself.
    ///
    /// Both entry points, because they cover different routes: `validate` is what a
    /// one-shot report and both command lines ask, `validate_read` what a retained index,
    /// an opened root, and a watch session ask. Both weigh the scope the *request* names,
    /// not the one a holder retained, so the refusal cannot wait for a scan that a
    /// cache-only read never runs -- which is how one request came to name a snapshot miss
    /// under one delivery and a scope refusal under another.
    ///
    /// `follow_symlinks` is refused on every platform, which is how the `one_filesystem`
    /// rule -- refused only where the platform has no device identity -- is tested here.
    #[test]
    fn a_scope_this_build_cannot_honour_is_refused_by_request_validation() {
        let held = basis(AnalysisSet::NONE, true);
        let mut asked = held.clone();
        asked.scope.follow_symlinks = true;
        let request = request_with(&[ViewSpec::Summary], Selection::default(), asked);
        let refusal = RequestError::ScopeUnsupported {
            axis: ScopeAxis::FollowSymlinks,
            reason: ScopeAxis::FollowSymlinks.reason(),
        };

        assert_eq!(request.validate(), Err(refusal.clone()));
        assert_eq!(request.validate_read(&held), Err(refusal));
        // The sentence is the one the capability rule states, not a copy of it kept here:
        // an engine-internal caller that never builds a request prints the same words.
        assert_eq!(
            ScanConfig { follow_symlinks: true, ..ScanConfig::default() }
                .unsupported_axis()
                .expect("no build follows symbolic links")
                .reason(),
            ScopeAxis::FollowSymlinks.reason()
        );
        request_with(&[ViewSpec::Summary], Selection::default(), held)
            .validate()
            .expect("a scope this build honours is not refused");
    }

    /// Moved from the report reader: the refusal half of
    /// `an_index_that_observed_no_control_state_has_no_ignored_share_to_select_by`. The
    /// reader keeps the library-path half, its typed refusal of an unvalidated query.
    #[test]
    fn a_scope_that_observes_no_control_state_refuses_selection_by_ignored_state() {
        let exclude = Selection { ignored: IgnoredEntries::Exclude, ..Selection::default() };
        let only = Selection { ignored: IgnoredEntries::Only, ..Selection::default() };
        let blind = basis(AnalysisSet::NONE, false);

        let refused = request_with(&[ViewSpec::Summary], exclude.clone(), blind.clone())
            .validate()
            .expect_err("no entry can be shown to be ignored");
        assert_eq!(
            refused.message(&AxisNames::FLAGS),
            "--ignored=exclude needs .gitignore classification, and --no-gitignore turned it \
             off; drop one of them"
        );
        let refused = request_with(&[ViewSpec::Summary], only, blind.clone())
            .validate()
            .expect_err("no entry can be shown to be ignored");
        assert_eq!(
            refused.message(&AxisNames::FIELDS),
            "ignored=only needs .gitignore classification, and read_controls turned it off; \
             drop one of them"
        );
        request_with(&[ViewSpec::Summary], exclude, basis(AnalysisSet::NONE, true))
            .validate()
            .expect("an observing scope can select by ignored state");
        request_with(&[ViewSpec::Summary], Selection::default(), blind)
            .validate()
            .expect("admitting every entry needs no classification");
    }

    #[test]
    fn a_read_is_refused_by_a_holder_of_other_content() {
        let request = built(&RequestSpec { analyze: Some("lines"), ..RequestSpec::new(root()) });
        let held = basis(AnalysisSet::NONE, true);
        assert_eq!(
            request.validate_read(&held),
            Err(RequestError::ContentMismatch {
                held: AnalysisSet::NONE,
                requested: AnalysisSet::NONE.with_lines(),
            })
        );
        // A wider store is refused too: serving a narrower request from it is a projection
        // this model does not define.
        assert_eq!(
            request.validate_read(&basis(AnalysisSet::ALL, true)),
            Err(RequestError::ContentMismatch {
                held: AnalysisSet::ALL,
                requested: AnalysisSet::NONE.with_lines(),
            })
        );
        request
            .validate_read(&basis(AnalysisSet::NONE.with_lines(), true))
            .expect("equal content serves");

        // The remaining rules read what the holder observed, not what the request assumed.
        let exclude = built(&reading(ReadSpec { ignored: Some("exclude"), ..ReadSpec::new() }));
        assert_eq!(
            exclude.validate_read(&basis(AnalysisSet::NONE, false)),
            Err(RequestError::IgnoredWithoutObservation(IgnoredEntries::Exclude))
        );
        // Content equality comes first, so the refusal names the more fundamental mismatch.
        let documents = request_with(
            &[ViewSpec::Documents],
            Selection::default(),
            basis(AnalysisSet::NONE.with_words(), true),
        );
        assert!(matches!(
            documents.validate_read(&basis(AnalysisSet::NONE, true)),
            Err(RequestError::ContentMismatch { .. })
        ));
    }

    /// What a read validates against is what the index observed, not the configuration that
    /// happened to make it: a `ScanConfig`'s delivery fields cannot be recovered from an
    /// index and no answer depends on them.
    #[test]
    fn the_basis_a_retained_index_holds_is_what_it_observed() {
        let blind =
            crate::Index::new_with_scope("/root", crate::test_support::not_observing_controls());
        let held = Basis::held_by(&blind);
        assert_eq!(held.root, Path::new("/root"));
        assert!(!held.scope.read_controls, "an index that read no rule says so");
        assert_eq!(held.content, AnalysisSet::NONE, "a metadata index holds no analyzer");

        let observing =
            crate::Index::new_with_scope("/root", crate::test_support::observing_controls());
        assert!(Basis::held_by(&observing).scope.read_controls);

        // The limits are the tier's own, not the table's: an index admitted its control
        // files under the limits it was built with, and a basis that restated the defaults
        // here would compare equal to one taken under any other budget.
        let tight = ControlLimits { budget: Some(4_096), line_limit: None };
        let narrow = crate::Index::new_with_config(
            "/root",
            &ScanConfig { read_controls: true, control_limits: tight, ..ScanConfig::default() },
        );
        assert_eq!(Basis::held_by(&narrow).scope.control_limits, tight);
        assert_ne!(tight, Request::DEFAULTS.control_limits, "the fixture must differ");
        assert_eq!(
            Basis::held_by(&blind).scope.control_limits,
            Request::DEFAULTS.control_limits,
            "a scan that read no rule applied none, and names the table's"
        );

        // The scope a request would have to be built with to read this index.
        let request =
            built(&RequestSpec { read_controls: Some(false), ..RequestSpec::new(root()) });
        request
            .validate_read(&Basis::held_by(&blind))
            .expect("the index's own basis answers a request built the same way");
    }

    /// Each rule in the order `validate_delivery` applies it, and each one only under a
    /// watch: every one of these is a legal one-shot request.
    #[test]
    fn a_watch_refuses_what_it_cannot_keep_current() {
        let one_shot = Delivery {
            stale_ok: false,
            cache: CachePolicy::Auto,
            cache_path: None,
            accept_partial: false,
            watch: None,
            workers: Workers::default(),
            batch_size: ScanConfig::default().batch_size,
            order: crate::ScanOrder::default(),
        };
        let watching = Delivery { watch: Some(WatchDelivery::default()), ..one_shot.clone() };
        let cases = [
            (
                RequestSpec { scan_depth: Some("2"), ..RequestSpec::new(root()) },
                RequestError::WatchScope,
            ),
            (
                RequestSpec { one_filesystem: true, ..RequestSpec::new(root()) },
                RequestError::WatchScope,
            ),
            (
                RequestSpec { analyze: Some("lines"), ..RequestSpec::new(root()) },
                RequestError::WatchContent { named: AnalysisSet::LINES_ONLY, views: Vec::new() },
            ),
            // Analysis a view implied is refused the same way, naming the view the caller
            // wrote rather than an analyzer they never typed.
            (
                reading(ReadSpec { views: Some("summary,code"), ..ReadSpec::new() }),
                RequestError::WatchContent {
                    named: AnalysisSet::NONE,
                    views: vec![ViewSpec::Code],
                },
            ),
            // `none` names nothing, so only the view is named.
            (
                RequestSpec {
                    analyze: Some("none"),
                    read: ReadSpec { views: Some("code"), ..ReadSpec::new() },
                    ..RequestSpec::new(root())
                },
                RequestError::WatchContent {
                    named: AnalysisSet::NONE,
                    views: vec![ViewSpec::Code],
                },
            ),
            // Both axes enabled analysis, so both are named.
            (
                RequestSpec {
                    analyze: Some("lines"),
                    read: ReadSpec { views: Some("documents"), ..ReadSpec::new() },
                    ..RequestSpec::new(root())
                },
                RequestError::WatchContent {
                    named: AnalysisSet::LINES_ONLY,
                    views: vec![ViewSpec::Documents],
                },
            ),
        ];
        for (spec, expected) in cases {
            let request = built(&spec);
            assert_eq!(request.validate_delivery(&watching), Err(expected));
            request.validate_delivery(&one_shot).expect("a one-shot delivers every one");
        }
        assert_eq!(
            built(&reading(ReadSpec { views: Some("code"), ..ReadSpec::new() }))
                .validate_delivery(&watching)
                .map_err(|error| error.message(&AxisNames::FLAGS)),
            Err("--view code needs code analysis, which --watch cannot keep current; use a \
                 one-shot report"
                .to_owned())
        );
        // A caller who named an analyzer and a content view is told both, so dropping the
        // view does not lead straight to the analyzer's refusal.
        let both = RequestSpec {
            analyze: Some("words"),
            read: ReadSpec { views: Some("code"), ..ReadSpec::new() },
            ..RequestSpec::new(root())
        };
        assert_eq!(
            built(&both)
                .validate_delivery(&watching)
                .map_err(|error| error.message(&AxisNames::FLAGS)),
            Err("--analyze words and --view code need words and code analysis, which --watch \
                 cannot keep current; use a one-shot report"
                .to_owned())
        );
        // `full` and the default imply nothing, so a request whose views were chosen by its
        // analyzers names the analyzers.
        for views in [None, Some("full")] {
            let spec = RequestSpec {
                analyze: Some("all"),
                read: ReadSpec { views, ..ReadSpec::new() },
                ..RequestSpec::new(root())
            };
            assert_eq!(
                built(&spec).validate_delivery(&watching),
                Err(RequestError::WatchContent { named: AnalysisSet::ALL, views: Vec::new() }),
                "{views:?}"
            );
        }
        // A basis supplied whole names no view, so its analyzers are refused as named.
        assert_eq!(
            request_with(
                &[ViewSpec::Tree],
                Selection::default(),
                basis(AnalysisSet::CODE_ONLY, true)
            )
            .validate_delivery(&watching),
            Err(RequestError::WatchContent { named: AnalysisSet::CODE_ONLY, views: Vec::new() })
        );

        // A content view's analysis is served from a snapshot exactly as a named analyzer's
        // is: the basis is one basis, whichever axis enabled it.
        let implied = built(&reading(ReadSpec { views: Some("code"), ..ReadSpec::new() }));
        implied
            .validate_delivery(&Delivery { stale_ok: true, ..one_shot.clone() })
            .expect("a stale answer needs only a compatible sidecar");

        let plain = built(&RequestSpec::new(root()));
        plain.validate_delivery(&watching).expect("a full-scope metadata watch is deliverable");
        assert_eq!(
            plain.validate_delivery(&Delivery { stale_ok: true, ..watching.clone() }),
            Err(RequestError::WatchCacheOnly),
            "nothing verifies the window between the snapshot and the start of the watch"
        );
        plain
            .validate_delivery(&Delivery { stale_ok: true, ..one_shot })
            .expect("a one-shot report is exactly what a snapshot answers");
    }

    /// The order the three watch rules speak in, when a request breaks more than one.
    ///
    /// The command line and the Python API both render the first refusal and stop, so this
    /// decides which of three true statements a caller is told, and a rule moved within
    /// `validate_delivery` would silently change that. Scope first, because it is a fact
    /// about what a watcher can observe at all; then content, which is about what stays
    /// current; then the cache policy, which is about the window before the watch started.
    #[test]
    fn the_watch_rules_speak_in_one_order() {
        let cache_only_watch = Delivery {
            cache: CachePolicy::Auto,
            stale_ok: true,
            cache_path: None,
            accept_partial: false,
            watch: Some(WatchDelivery::default()),
            workers: Workers::default(),
            batch_size: ScanConfig::default().batch_size,
            order: crate::ScanOrder::default(),
        };
        let everything = RequestSpec {
            scan_depth: Some("2"),
            analyze: Some("lines"),
            ..RequestSpec::new(root())
        };
        let steps = [
            (everything, RequestError::WatchScope),
            (
                RequestSpec { scan_depth: None, ..everything },
                RequestError::WatchContent { named: AnalysisSet::LINES_ONLY, views: Vec::new() },
            ),
            (
                RequestSpec { scan_depth: None, analyze: None, ..everything },
                RequestError::WatchCacheOnly,
            ),
        ];
        for (spec, expected) in steps {
            assert_eq!(built(&spec).validate_delivery(&cache_only_watch), Err(expected));
        }
        built(&RequestSpec::new(root()))
            .validate_delivery(&Delivery { stale_ok: false, ..cache_only_watch })
            .expect("nothing left to refuse");
    }

    /// The other half of the watch-scope rule, which the command-line golden cannot
    /// assert: where a build cannot honor `one_filesystem` at all, the request is refused
    /// for that reason first, so the message differs by platform.
    #[cfg(unix)]
    #[test]
    fn a_watch_refuses_one_filesystem_where_the_build_honors_it() {
        let watch = Delivery {
            stale_ok: false,
            cache: CachePolicy::Auto,
            cache_path: None,
            accept_partial: false,
            watch: Some(WatchDelivery::default()),
            workers: Workers::default(),
            batch_size: ScanConfig::default().batch_size,
            order: crate::ScanOrder::default(),
        };
        let spec = RequestSpec { one_filesystem: true, ..RequestSpec::new(root()) };
        let request = built(&spec);
        request.validate().expect("one filesystem is honored on this build");
        assert_eq!(request.validate_delivery(&watch), Err(RequestError::WatchScope));
    }

    #[test]
    fn a_request_is_refused_past_the_views_one_report_carries() {
        let mut request = built(&RequestSpec::new(root()));
        request.query.views = vec![ViewSpec::Summary; crate::MAX_REPORT_VIEWS];
        request.validate().expect("the limit itself is accepted");
        request.query.omitted_views = vec![ViewSpec::Documents];
        assert_eq!(
            request.validate(),
            Err(RequestError::ViewLimit {
                attempted: crate::MAX_REPORT_VIEWS + 1,
                limit: crate::MAX_REPORT_VIEWS,
            })
        );
    }

    /// The refusal `Roots::resolve` returns, or a panic naming what it returned instead.
    fn roots_refusal<P: AsRef<Path>>(paths: &[P]) -> RequestError {
        match Roots::resolve(paths) {
            Err(crate::Error::InvalidRequest(refusal)) => refusal,
            other => panic!("expected a request refusal, got {other:?}"),
        }
    }

    /// A tree of three directories, `a`, `a/b`, and `c`, under a fresh temporary root.
    fn roots_fixture() -> (tempfile::TempDir, PathBuf) {
        let temporary = tempfile::tempdir().expect("tempdir");
        let base = temporary.path().canonicalize().expect("canonical tempdir");
        std::fs::create_dir_all(base.join("a/b")).expect("a/b");
        std::fs::create_dir(base.join("c")).expect("c");
        (temporary, base)
    }

    #[test]
    fn roots_keep_the_callers_order_labels_and_canonical_paths() {
        let (_temporary, base) = roots_fixture();
        let roots = Roots::resolve(&[base.join("c"), base.join("a")]).expect("disjoint roots");
        assert!(roots.is_several());
        let labels: Vec<_> = roots.iter().map(|root| root.label.clone()).collect();
        assert_eq!(labels, [base.join("c"), base.join("a")]);
        assert_eq!(roots.first().path, base.join("c"));
        assert!(!Roots::resolve(&[base.join("a")]).expect("one root").is_several());
    }

    /// A label is normalized by its components: separators repeated or trailing go, `/`
    /// stays itself, and a leading `.` stays because it is how the caller named the root.
    #[test]
    fn root_labels_are_normalized_by_their_components() {
        let (_temporary, base) = roots_fixture();
        for spelling in ["a/", "a//", "a/./"] {
            let given = format!("{}/{spelling}", base.display());
            let roots = Roots::resolve(&[given.as_str()]).expect("a directory");
            assert_eq!(roots.first().label, base.join("a"), "{spelling}");
            assert_eq!(roots.first().path, base.join("a"), "{spelling}");
        }
        let current = Roots::resolve(&["."]).expect("the working directory");
        assert_eq!(current.first().label, PathBuf::from("."));
        assert_eq!(current.first().path, Path::new(".").canonicalize().expect("cwd"));
        let dotted = Roots::resolve(&["./src/"]).expect("this crate's sources");
        assert_eq!(dotted.first().label, PathBuf::from("./src"));
        #[cfg(unix)]
        {
            let slash = Roots::resolve(&["/"]).expect("the filesystem root");
            assert_eq!(slash.first().label, PathBuf::from("/"));
        }
    }

    #[cfg(windows)]
    #[test]
    fn a_drive_root_label_stays_whole() {
        let drive = Roots::resolve(&[r"C:\"]).expect("the system drive");
        assert_eq!(drive.first().label, PathBuf::from(r"C:\"));
    }

    #[test]
    fn a_root_named_twice_is_refused_naming_both_spellings() {
        let (_temporary, base) = roots_fixture();
        let a = base.join("a");
        assert_eq!(
            roots_refusal(&[&a, &a]),
            RequestError::RootsRepeated { first: a.clone(), second: a.clone() }
        );
        let respelled = PathBuf::from(format!("{}/./a/", base.display()));
        assert_eq!(
            roots_refusal(&[&a, &respelled]),
            RequestError::RootsRepeated { first: a.clone(), second: a.clone() },
            "one directory spelled two ways normalizes to one label"
        );
        let relative = Path::new("./src");
        assert_eq!(
            roots_refusal(&[Path::new("src"), relative]),
            RequestError::RootsRepeated {
                first: PathBuf::from("src"),
                second: PathBuf::from("./src")
            },
            "a leading `.` is part of the label, so the refusal names both spellings"
        );
    }

    #[test]
    fn a_root_inside_another_is_refused_in_either_order() {
        let (_temporary, base) = roots_fixture();
        let (outer, inner) = (base.join("a"), base.join("a/b"));
        let expected = RequestError::RootsOverlap { inner: inner.clone(), outer: outer.clone() };
        assert_eq!(roots_refusal(&[&outer, &inner]), expected);
        assert_eq!(roots_refusal(&[&inner, &outer]), expected);
        // Whole components: `a` and `ab` are neighbours, not nested.
        std::fs::create_dir(base.join("ab")).expect("ab");
        Roots::resolve(&[base.join("a"), base.join("ab")]).expect("a prefix is not a parent");
        // Conservative: the filesystem root contains every other root.
        #[cfg(unix)]
        assert_eq!(
            roots_refusal(&[Path::new("/"), base.as_path()]),
            RequestError::RootsOverlap { inner: base.clone(), outer: PathBuf::from("/") }
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_alias_is_the_directory_it_names() {
        let (_temporary, base) = roots_fixture();
        std::os::unix::fs::symlink(base.join("a"), base.join("alias")).expect("alias");
        std::os::unix::fs::symlink(base.join("a/b"), base.join("deep")).expect("deep");
        assert_eq!(
            roots_refusal(&[base.join("a"), base.join("alias")]),
            RequestError::RootsRepeated { first: base.join("a"), second: base.join("alias") }
        );
        assert_eq!(
            roots_refusal(&[base.join("a"), base.join("deep")]),
            RequestError::RootsOverlap { inner: base.join("deep"), outer: base.join("a") }
        );
    }

    /// On macOS, `/System/Volumes/Data` holds the data volume, and firmlinks such as
    /// `/private` and `/Users` show its directories at the top as well: two paths that
    /// canonicalize apart and are one directory. Probed rather than assumed, since a
    /// temporary directory need not sit under a firmlink on every host.
    #[cfg(unix)]
    #[test]
    fn a_firmlink_alias_is_the_directory_it_names_where_the_host_has_one() {
        use std::os::unix::fs::MetadataExt;
        let (_temporary, base) = roots_fixture();
        let relative = base.strip_prefix("/").expect("absolute tempdir");
        let alias = Path::new("/System/Volumes/Data").join(relative);
        let identity = |path: &Path| std::fs::metadata(path).map(|m| (m.dev(), m.ino())).ok();
        if alias == base || identity(&alias).is_none() || identity(&alias) != identity(&base) {
            eprintln!("skipped: {} is not a firmlink alias on this host", alias.display());
            return;
        }
        assert_eq!(
            roots_refusal(&[base.as_path(), alias.as_path()]),
            RequestError::RootsRepeated { first: base.clone(), second: alias.clone() }
        );
        assert_eq!(
            roots_refusal(&[base.as_path(), alias.join("a").as_path()]),
            RequestError::RootsOverlap { inner: alias.join("a"), outer: base.clone() }
        );
    }

    /// A bind mount, or any alias, as identities: no host need have one for the rule to be
    /// pinned.
    #[test]
    fn the_overlap_check_compares_identities_as_well_as_paths() {
        let facts = |canonical: &str, identity: (u64, u64), ancestors: &[(u64, u64)]| RootFacts {
            canonical: PathBuf::from(canonical),
            identity: Some(identity),
            ancestors: ancestors.to_vec(),
        };
        let source = facts("/srv/data", (1, 10), &[(1, 3), (1, 2)]);
        // `/mnt/view` is `/srv/data` bound elsewhere: another path, the same directory.
        let bound = facts("/mnt/view", (1, 10), &[(1, 4), (1, 2)]);
        let below = facts("/mnt/view/x", (1, 30), &[(1, 10), (1, 4), (1, 2)]);
        let elsewhere = facts("/home/me", (1, 40), &[(1, 5), (1, 2)]);
        assert_eq!(
            first_overlap(&[source.clone(), bound]),
            Some(Overlap { outer: 0, inner: 1, same: true })
        );
        assert_eq!(
            first_overlap(&[below.clone(), source.clone()]),
            Some(Overlap { outer: 1, inner: 0, same: false })
        );
        assert_eq!(first_overlap(&[source.clone(), elsewhere.clone()]), None);
        // Without identities, as off Unix, only the paths decide.
        let unknown = |canonical: &str| RootFacts {
            canonical: PathBuf::from(canonical),
            ..RootFacts::default()
        };
        assert_eq!(
            first_overlap(&[unknown("/srv"), unknown("/srv/data")]),
            Some(Overlap { outer: 0, inner: 1, same: false })
        );
        assert_eq!(first_overlap(&[unknown("/srv/data"), unknown("/srv/data-old")]), None);
        // The first overlapping pair in argument order is the one named.
        assert_eq!(
            first_overlap(&[elsewhere, source, below]),
            Some(Overlap { outer: 1, inner: 2, same: false })
        );
    }

    /// Each root fails as one root always has, in argument order, before any overlap is
    /// considered: a missing root at the path as given, a file at its canonical path in the
    /// scanner's words.
    #[test]
    fn roots_are_validated_in_order_before_overlap() {
        let (_temporary, base) = roots_fixture();
        let missing = base.join("missing");
        match Roots::resolve(&[base.join("a"), base.join("a"), missing.clone()]) {
            Err(crate::Error::Io { path, source }) => {
                assert_eq!(path, missing);
                assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
            }
            other => panic!("expected the missing root's error, got {other:?}"),
        }
        std::fs::write(base.join("file"), b"x").expect("file");
        match Roots::resolve(&[base.join("c"), base.join("./file")]) {
            Err(crate::Error::Io { path, source }) => {
                assert_eq!(path, base.join("file"));
                assert_eq!(source.kind(), std::io::ErrorKind::NotADirectory);
                assert_eq!(source.to_string(), "scan root is not a directory");
            }
            other => panic!("expected the file root's error, got {other:?}"),
        }
        assert_eq!(roots_refusal::<&Path>(&[]), RequestError::NoRoots);
    }

    /// A name that is not UTF-8 keeps its bytes in the label, as a path does. Where the
    /// filesystem refuses such a name (APFS requires UTF-8), there is nothing to test.
    #[cfg(unix)]
    #[test]
    fn a_label_that_is_not_utf8_keeps_its_bytes() {
        use std::os::unix::ffi::OsStrExt;
        let (_temporary, base) = roots_fixture();
        let name = std::ffi::OsStr::from_bytes(b"caf\xe9");
        let root = base.join(name);
        if std::fs::create_dir(&root).is_err() {
            eprintln!("skipped: this filesystem refuses a name that is not UTF-8");
            return;
        }
        let roots = Roots::resolve(&[&root]).expect("a directory");
        assert_eq!(roots.first().label.as_os_str().as_bytes(), root.as_os_str().as_bytes());
        assert!(roots.first().label.to_str().is_none());
    }
}
