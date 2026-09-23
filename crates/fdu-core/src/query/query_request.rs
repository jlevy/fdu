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
use crate::query::query_report::{AxisNames, Query, ViewSpec};
use crate::query::query_selection::{Bound, IgnoredEntries, Selection, SizeMetric, SortKey};
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
            control_limits: self.control_limits,
            threads: delivery.workers.scan,
            batch_size: delivery.batch_size,
            order: delivery.order,
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
    /// `ScanConfig` still carries delivery fields (`threads`, `batch_size`, `order`), and
    /// `threads` stays the authoritative scan worker count until worker counts move into
    /// [`Delivery`]; none of them changes an answer.
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
        let scope = parse_scope(spec, axes)?;
        Ok(Self { root: spec.root.to_path_buf(), scope, content })
    }

    /// The limits a basis records when its scan observed no control state at all.
    ///
    /// The defaults table's, because they are what the scope would have been taken under
    /// had it read a rule; no rule this model states reads them when `read_controls` is
    /// off, so this is a placeholder named rather than a value implied.
    const UNOBSERVED_LIMITS: ControlLimits = Request::DEFAULTS.control_limits;
}

/// How a request is carried out, which never changes what its answer says.
///
/// One worker count is here, because it has nowhere else to wait: scan threads ride in
/// [`ScanConfig::threads`] inside [`Basis::scope`] until the execution plan model takes
/// them, and the content readers' count rides in
/// [`AnalysisRequest`](crate::content::AnalysisRequest), which a request does not carry --
/// [`Basis::content`] is the analyzer set, which is what changes an answer. Phase 2 replaces
/// both with one `Workers`.
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
            cache_path,
            accept_partial: false,
            watch: None,
            workers: Workers::default(),
            batch_size: ScanConfig::default().batch_size,
            order: crate::ScanOrder::default(),
        }
    }

    /// Representative deliveries for checking policy independently of route.
    /// Worker counts and cache location are fixed; every cache, partial-answer, and
    /// watch choice is represented.
    pub fn enumerate() -> impl Iterator<Item = Self> {
        [
            CachePolicy::Auto,
            CachePolicy::Refresh,
            CachePolicy::ReadOnly,
            CachePolicy::Only,
            CachePolicy::Off,
        ]
        .into_iter()
        .flat_map(|cache| {
            [false, true].into_iter().flat_map(move |accept_partial| {
                [None, Some(WatchDelivery::default())].into_iter().map(move |watch| Self {
                    cache,
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
#[derive(Clone, Debug)]
pub struct Request {
    /// Root, scope, and content: what a holder of stored state must match.
    pub basis: Basis,
    /// Selection, views, and view options.
    pub query: Query,
    /// The instant relative time windows were resolved against.
    ///
    /// Fixed when the request is built, so [`Selection::modified`] is absolute and a watch
    /// that builds its request once at start never slides its window.
    pub now: SystemTime,
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
            words_per_page: None,
            include: &[],
            exclude: &[],
            min_size: None,
            modified_since: None,
            modified_before: None,
            kinds: None,
            ignored: None,
            depth: None,
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
/// refuses a watch that names an analyzer, so the content a watch serves is always none and
/// its view is the report default for none. A field would have restated `tree` beside the
/// rule that makes it true, which is the shape a default drifts out of.
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
    pub const fn new(basis: Basis, query: Query, now: SystemTime) -> Self {
        Self { basis, query, now }
    }

    /// One read of what `basis` holds, written in the value grammars.
    ///
    /// The constructor every read site wants: a holder supplies the basis it was opened
    /// with, and the caller supplies only what this read asks. The view default comes from
    /// the analyzers the basis already holds, so a typed set never has to be spelled back
    /// into the grammar to find out what a request that read files displays, and no caller
    /// builds a request with a throw-away basis and overwrites it afterwards.
    ///
    /// # Errors
    ///
    /// [`RequestError`] for a value no grammar accepts, and for a read the basis cannot
    /// answer: [`Self::validate`]'s rules, which here are the holder's own.
    pub fn read(
        basis: Basis,
        spec: &ReadSpec<'_>,
        now: SystemTime,
        axes: &'static AxisNames,
    ) -> Result<Self, RequestError> {
        let query = build_query(basis.content, spec, now, axes)?;
        let request = Self::new(basis, query, now);
        request.validate()?;
        Ok(request)
    }

    /// Parse a spec into a request, resolving relative time windows against `now`.
    ///
    /// Refusals name each axis as `axes` spells it, and so do the report diagnostics of the
    /// query built here. Every value is parsed before any is checked against another, in
    /// the order the command line reads its flags: content, views, selection, the page
    /// denominator, then scope. Rules that relate axes are [`Self::validate`]'s.
    pub fn build(
        spec: &RequestSpec<'_>,
        now: SystemTime,
        axes: &'static AxisNames,
    ) -> Result<Self, RequestError> {
        // The three steps in the order the command line reads its flags, which is the order
        // a refusal names when two axes are both wrong: content, then everything this read
        // supplies, then scope. `Basis::build` runs the first and the third together, for
        // the callers that fix a basis and read it many times.
        let content = parse_content(spec, axes)?;
        let query = build_query(content, &spec.read, now, axes)?;
        let scope = parse_scope(spec, axes)?;
        Ok(Self::new(Basis { root: spec.root.to_path_buf(), scope, content }, query, now))
    }

    /// Refuse a request no holder of its own basis could answer.
    ///
    /// In order: more views than one report carries, a view its content cannot answer, and
    /// a selection by ignored state its scope does not observe.
    pub fn validate(&self) -> Result<(), RequestError> {
        self.validate_against(&self.basis)
    }

    /// Refuse a read that `held`, the basis of a retained index or opened root, cannot
    /// answer.
    ///
    /// Content must be equal: an index built with other analyzers holds other metrics, and
    /// serving a narrower request from a wider store is a projection this model does not
    /// define. The remaining rules are [`Self::validate`]'s, applied to what `held`
    /// observed rather than to what the request says it would have. Scope equality is not
    /// checked here; `ScanConfig` owns it.
    pub fn validate_read(&self, held: &Basis) -> Result<(), RequestError> {
        if held.content != self.basis.content {
            return Err(RequestError::ContentMismatch {
                held: held.content,
                requested: self.basis.content,
            });
        }
        self.validate_against(held)
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
    ///   as fresh.
    /// - A snapshot nothing verified ([`RequestError::WatchCacheOnly`]): the window between
    ///   the snapshot and the session's start is never observed, so the first answer would
    ///   describe a tree that may have moved and every later one would build on it.
    ///
    /// Each was a guard on one surface, which is why a library caller and a Python caller
    /// could ask for what the command line refuses.
    pub fn validate_delivery(&self, delivery: &Delivery) -> Result<(), RequestError> {
        if delivery.watch.is_none() {
            return Ok(());
        }
        if self.basis.scope.max_depth.is_some() || self.basis.scope.one_filesystem {
            return Err(RequestError::WatchScope);
        }
        if self.basis.content.is_enabled() {
            return Err(RequestError::WatchContent);
        }
        if delivery.cache == CachePolicy::Only {
            return Err(RequestError::WatchCacheOnly);
        }
        Ok(())
    }

    fn validate_against(&self, basis: &Basis) -> Result<(), RequestError> {
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
        check_views(&self.query.views, basis.content)?;
        check_observation(self.query.selection.ignored, basis.scope.read_controls)
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

/// The query one read supplies, parsed against the analyzers its basis holds.
///
/// `content` decides the view default and nothing else here: a request that paid to read
/// files displays what it read. Every value is parsed before any is checked against
/// another, in one order for every surface -- views, selection, then the page denominator.
fn build_query(
    content: AnalysisSet,
    spec: &ReadSpec<'_>,
    now: SystemTime,
    axes: &'static AxisNames,
) -> Result<Query, RequestError> {
    let (views, omitted_views) = ViewSpec::resolve_rejecting(spec.views, content)
        .map_err(|rejection| rejection.on(axes.view))?;

    let mut selection = Selection {
        depth: spec.depth.map(|value| parse_bound(value, axes.depth)).transpose()?,
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
    if let Some(value) = spec.ignored {
        selection.ignored = IgnoredEntries::parse(value)
            .map_err(|expected| Rejection::new(value, expected).on(axes.ignored))?;
    }
    let words_per_page =
        spec.words_per_page.map_or(Ok(Request::DEFAULTS.words_per_page), |value| {
            value
                .trim()
                .parse::<u64>()
                .ok()
                .filter(|words| *words > 0)
                .ok_or_else(|| invalid(axes.words_per_page, value, "expected a positive integer"))
        })?;

    Ok(Query { selection, views, omitted_views, axes, words_per_page })
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
#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// A view has no metadata-only projection, and the request enables no analyzer.
    ViewNeedsContent(ViewSpec),
    /// A selection by ignored state over a scan that observes no `.gitignore`.
    IgnoredWithoutObservation(IgnoredEntries),
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
    WatchContent,
    /// A watch was asked to start from a snapshot nothing verifies.
    WatchCacheOnly,
    /// A read names more views than one report may carry.
    ViewLimit {
        /// Views and omitted views the request carries.
        attempted: usize,
        /// The most one report accepts.
        limit: usize,
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
            Self::ViewNeedsContent(view) => format!(
                "{} {} requires content analysis: add {} lines, code, words, or all; views never \
                 enable content analysis implicitly",
                axes.view,
                view.label(),
                axes.analyze
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
            Self::WatchContent => format!(
                "{} is not yet supported with {}; use a one-shot report",
                axes.analyze, axes.watch
            ),
            Self::WatchCacheOnly => format!(
                "{watch} cannot start from {cache} only: nothing verifies what changed between \
                 the snapshot and the start of the watch; use {cache} auto or read-only",
                watch = axes.watch,
                cache = axes.cache,
            ),
            Self::ViewLimit { attempted, limit } => {
                format!("report request contains {attempted} views or omissions; limit is {limit}")
            }
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

/// Refuse a view that no enabled analyzer can answer.
///
/// A match over every view rather than a list of the exceptions, so a new view forces a
/// decision here about whether it needs content.
pub(crate) fn check_views(views: &[ViewSpec], content: AnalysisSet) -> Result<(), RequestError> {
    for view in views {
        match view {
            ViewSpec::Documents if !content.is_enabled() => {
                return Err(RequestError::ViewNeedsContent(*view));
            }
            ViewSpec::Tree
            | ViewSpec::Types
            | ViewSpec::Extensions
            | ViewSpec::Families
            | ViewSpec::Languages
            | ViewSpec::Documents
            | ViewSpec::Files
            | ViewSpec::Largest
            | ViewSpec::Recent
            | ViewSpec::Summary => {}
        }
    }
    Ok(())
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

/// Parse an ordering key: `size`, `count`, `mtime`, or `name`.
pub fn parse_sort(value: &str, axis: &'static str) -> Result<SortKey, RequestError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "size" => Ok(SortKey::Size),
        "count" => Ok(SortKey::Count),
        "mtime" => Ok(SortKey::Mtime),
        "name" => Ok(SortKey::Name),
        other => Err(invalid(axis, other, "expected one of size, count, mtime, name")),
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

/// Parse a cache policy: `auto`, `refresh`, `read-only`, `only`, or `off`.
pub fn parse_cache_policy(value: &str, axis: &'static str) -> Result<CachePolicy, RequestError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Ok(CachePolicy::Auto),
        "refresh" => Ok(CachePolicy::Refresh),
        "read-only" => Ok(CachePolicy::ReadOnly),
        "only" => Ok(CachePolicy::Only),
        "off" => Ok(CachePolicy::Off),
        other => Err(invalid(axis, other, "expected one of auto, refresh, read-only, only, off")),
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
                "invalid --sort \"newest\": expected one of size, count, mtime, name",
                "invalid sort \"newest\": expected one of size, count, mtime, name",
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
                "invalid --cache \"readonly\": expected one of auto, refresh, read-only, only, off",
                "invalid cache policy \"readonly\": expected one of auto, refresh, read-only, \
                 only, off",
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
        ] {
            assert_eq!(parse_sort(spelling, axis), Ok(key));
        }
        assert_eq!(parse_size_metric("Allocated", axis), Ok(SizeMetric::Allocated));
        assert_eq!(parse_size_metric("apparent", axis), Ok(SizeMetric::Apparent));
        for (spelling, policy) in [
            ("auto", CachePolicy::Auto),
            ("refresh", CachePolicy::Refresh),
            ("read-only", CachePolicy::ReadOnly),
            ("ONLY", CachePolicy::Only),
            ("off", CachePolicy::Off),
        ] {
            assert_eq!(parse_cache_policy(spelling, axis), Ok(policy));
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
                RequestError::ViewNeedsContent(ViewSpec::Documents),
                "--view documents requires content analysis: add --analyze lines, code, words, \
                 or all; views never enable content analysis implicitly",
                "view documents requires content analysis: add analyze lines, code, words, or \
                 all; views never enable content analysis implicitly",
            ),
            (
                RequestError::IgnoredWithoutObservation(IgnoredEntries::Exclude),
                "--exclude-ignored needs .gitignore classification, and --no-gitignore turned it \
                 off; drop one of them",
                "ignored=exclude needs .gitignore classification, and read_controls turned it \
                 off; drop one of them",
            ),
            (
                RequestError::IgnoredWithoutObservation(IgnoredEntries::Only),
                "--only-ignored needs .gitignore classification, and --no-gitignore turned it \
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
                RequestError::WatchContent,
                "--analyze is not yet supported with --watch; use a one-shot report",
                "analyze is not yet supported with watch; use a one-shot report",
            ),
            (
                RequestError::WatchCacheOnly,
                "--watch cannot start from --cache only: nothing verifies what changed between the \
                 snapshot and the start of the watch; use --cache auto or read-only",
                "watch cannot start from cache policy only: nothing verifies what changed between \
                 the snapshot and the start of the watch; use cache policy auto or read-only",
            ),
            (
                RequestError::ViewLimit { attempted: 17, limit: 16 },
                "report request contains 17 views or omissions; limit is 16",
                "report request contains 17 views or omissions; limit is 16",
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

    #[test]
    fn documents_is_the_only_view_that_needs_content() {
        for content in [AnalysisSet::NONE, AnalysisSet::NONE.with_lines(), AnalysisSet::ALL] {
            for view in ViewSpec::ALL {
                let refused = check_views(&[view], content).is_err();
                assert_eq!(
                    refused,
                    view == ViewSpec::Documents && !content.is_enabled(),
                    "{view:?}"
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
    fn reading(read: ReadSpec<'_>) -> RequestSpec<'_> {
        RequestSpec { read, ..RequestSpec::new(root()) }
    }

    fn built(spec: &RequestSpec<'_>) -> Request {
        Request::build(spec, instant(), &AxisNames::FIELDS).expect("the spec parses")
    }

    fn refusal(spec: &RequestSpec<'_>, axes: &'static AxisNames) -> String {
        Request::build(spec, instant(), axes).expect_err("the spec is refused").message(axes)
    }

    /// A spec that names nothing builds the table, and every type that also declares a
    /// default for one of these axes agrees with it.
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
        assert_eq!(defaults.report_view(AnalysisSet::NONE), ViewSpec::Tree);
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
                words_per_page: Some("300"),
                include: &include,
                exclude: &exclude,
                min_size: Some("1KiB"),
                modified_since: Some("@1700000000"),
                modified_before: Some("@1800000000"),
                kinds: Some("file,dir"),
                ignored: Some("include"),
                depth: Some("all"),
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
                "invalid --exclude-ignored/--only-ignored \"maybe\": expected one of include, \
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
            scope: Scope { read_controls, ..ScanConfig::default() },
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

        let documents = request_with(
            &[ViewSpec::Documents],
            Selection::default(),
            basis(AnalysisSet::NONE, true),
        );
        assert_eq!(documents.validate(), Err(RequestError::ViewNeedsContent(ViewSpec::Documents)));
        for content in enabled {
            request_with(&[ViewSpec::Documents], Selection::default(), basis(content, true))
                .validate()
                .expect("every enabled profile includes the basic document metrics");
        }

        request_with(
            &[ViewSpec::Types, ViewSpec::Families],
            Selection::default(),
            basis(AnalysisSet::NONE, true),
        )
        .validate()
        .expect("metadata grouping never requires content I/O");
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
            "--exclude-ignored needs .gitignore classification, and --no-gitignore turned it \
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
                RequestError::WatchContent,
            ),
        ];
        for (spec, expected) in cases {
            let request = built(&spec);
            assert_eq!(request.validate_delivery(&watching), Err(expected));
            request.validate_delivery(&one_shot).expect("a one-shot delivers all three");
        }

        let plain = built(&RequestSpec::new(root()));
        plain.validate_delivery(&watching).expect("a full-scope metadata watch is deliverable");
        assert_eq!(
            plain.validate_delivery(&Delivery { cache: CachePolicy::Only, ..watching.clone() }),
            Err(RequestError::WatchCacheOnly),
            "nothing verifies the window between the snapshot and the start of the watch"
        );
        plain
            .validate_delivery(&Delivery { cache: CachePolicy::Only, ..one_shot })
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
            cache: CachePolicy::Only,
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
            (RequestSpec { scan_depth: None, ..everything }, RequestError::WatchContent),
            (
                RequestSpec { scan_depth: None, analyze: None, ..everything },
                RequestError::WatchCacheOnly,
            ),
        ];
        for (spec, expected) in steps {
            assert_eq!(built(&spec).validate_delivery(&cache_only_watch), Err(expected));
        }
        built(&RequestSpec::new(root()))
            .validate_delivery(&Delivery { cache: CachePolicy::Auto, ..cache_only_watch })
            .expect("nothing left to refuse");
    }

    /// The other half of the watch-scope rule, which the command-line golden cannot
    /// assert: where a build cannot honor `one_filesystem` at all, the request is refused
    /// for that reason first, so the message differs by platform.
    #[cfg(unix)]
    #[test]
    fn a_watch_refuses_one_filesystem_where_the_build_honors_it() {
        let watch = Delivery {
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
}
