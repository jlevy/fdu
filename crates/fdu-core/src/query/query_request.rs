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
    pub scope: ScanConfig,
    /// The analyzers whose results the answer may report.
    pub content: AnalysisSet,
}

/// How a request is carried out, which never changes what its answer says.
///
/// No worker counts yet: they stay in [`ScanConfig::threads`] and
/// [`AnalysisRequest::workers`](crate::content::AnalysisRequest::workers) until the
/// execution plan model takes them.
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
}

/// How a watch repeats its answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WatchDelivery {
    /// The longest a repaint waits for changes; change detection itself is event-driven.
    pub interval: Duration,
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

/// Every default a request takes when its caller names nothing, stated once.
///
/// | Axis | Default |
/// | --- | --- |
/// | Size | allocated |
/// | Views of a report | [`Self::report_view`]: [`ViewSpec::default_for`] the content |
/// | Views of a watch | `tree`, the report default for no content, which is all a watch serves |
/// | Words per page | 250 |
/// | Content | no analyzer |
/// | `.gitignore` | observed, under the default budget and line limit |
///
/// Each answers a question: "how much disk does this use" is allocated bytes, as `du`
/// reports them; a request that pays to read files displays what it read; and a tree is
/// what "what is big here" looks like. Surfaces take these rather than declaring their own,
/// because a default declared twice drifts: size was apparent in Rust and allocated
/// everywhere else, and `words_per_page` was written out in three places.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestDefaults {
    /// The size metric.
    pub size: SizeMetric,
    /// The view a watch reports.
    pub watch_view: ViewSpec,
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
        watch_view: ViewSpec::Tree,
        words_per_page: 250,
        content: AnalysisSet::NONE,
        read_controls: true,
        control_limits: ControlLimits {
            budget: Some(DEFAULT_CONTROL_BUDGET),
            line_limit: Some(DEFAULT_CONTROL_LINE_LIMIT),
        },
    };

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
        let content = spec.analyze.map_or(Ok(Self::DEFAULTS.content), |value| {
            AnalysisSet::parse_rejecting(value).map_err(|rejection| rejection.on(axes.analyze))
        })?;
        let (views, omitted_views) = ViewSpec::resolve_rejecting(spec.views, content)
            .map_err(|rejection| rejection.on(axes.view))?;

        let mut selection = Selection {
            depth: spec.depth.map(|value| parse_bound(value, axes.depth)).transpose()?,
            limit: spec.limit.map(|value| parse_bound(value, axes.limit)).transpose()?,
            reverse: spec.reverse,
            size: spec
                .size
                .map_or(Ok(Self::DEFAULTS.size), |value| parse_size_metric(value, axes.size))?,
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
            spec.words_per_page.map_or(Ok(Self::DEFAULTS.words_per_page), |value| {
                value.trim().parse::<u64>().ok().filter(|words| *words > 0).ok_or_else(|| {
                    invalid(axes.words_per_page, value, "expected a positive integer")
                })
            })?;

        let limits = Self::DEFAULTS.control_limits;
        let scope = ScanConfig {
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
            read_controls: spec.read_controls.unwrap_or(Self::DEFAULTS.read_controls),
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
            ..ScanConfig::default()
        };

        Ok(Self {
            basis: Basis { root: spec.root.to_path_buf(), scope, content },
            query: Query { selection, views, omitted_views, axes, words_per_page },
            now,
        })
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
    /// Declared with the model so the harness and the execution plan can be written against
    /// it, but it refuses nothing yet. The watch refusals ([`RequestError::WatchScope`],
    /// [`RequestError::WatchContent`], and [`RequestError::WatchCacheOnly`]) move here when
    /// the command line and the watch session call it; until then the command line's
    /// guards and `ScanConfig`'s watch-scope check are the rule.
    pub fn validate_delivery(&self, delivery: &Delivery) -> Result<(), RequestError> {
        let _ = (self, delivery);
        Ok(())
    }

    fn validate_against(&self, basis: &Basis) -> Result<(), RequestError> {
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
    if set.is_enabled() { set.labels().join(",") } else { "none".to_string() }
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
        assert_eq!(defaults.watch_view, ViewSpec::Tree);
        // A watch serves no content, so its view is the report default for none.
        assert_eq!(defaults.watch_view, defaults.report_view(AnalysisSet::NONE));
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
        let spec = RequestSpec {
            modified_since: Some("2h"),
            modified_before: Some("now"),
            ..RequestSpec::new(root())
        };
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
            let spec = RequestSpec { views: Some(views), ..spec };
            for (axes, label) in [(&AxisNames::FLAGS, "--view"), (&AxisNames::FIELDS, "view")] {
                let today =
                    ViewSpec::resolve(Some(views), AnalysisSet::NONE, label).expect_err("refused");
                assert_eq!(refusal(&spec, axes), today);
            }
        }

        let cases: [(RequestSpec<'_>, &str, &str); 7] = [
            (
                RequestSpec { words_per_page: Some("0"), ..spec },
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
                RequestSpec { ignored: Some("maybe"), ..spec },
                "invalid --exclude-ignored/--only-ignored \"maybe\": expected one of include, \
                 exclude, only",
                "invalid ignored \"maybe\": expected one of include, exclude, only",
            ),
            (
                RequestSpec { kinds: Some("file,socket"), ..spec },
                "invalid --kind \"socket\": expected one of file, dir, symlink, other",
                "invalid kind \"socket\": expected one of file, dir, symlink, other",
            ),
            (
                RequestSpec { min_size: Some("10X"), ..spec },
                "invalid size \"10X\": unknown size unit \"X\"; use B, K/KB, M/MB, G/GB, T/TB, \
                 P/PB, or the binary forms KiB, MiB, GiB, TiB, PiB",
                "invalid size \"10X\": unknown size unit \"X\"; use B, K/KB, M/MB, G/GB, T/TB, \
                 P/PB, or the binary forms KiB, MiB, GiB, TiB, PiB",
            ),
            (
                RequestSpec { modified_since: Some("2300-01-01T00:00:00Z"), ..spec },
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

    fn request_with(views: &[ViewSpec], selection: Selection, basis: Basis) -> Request {
        Request {
            basis,
            query: Query { selection, views: views.to_vec(), ..Query::default() },
            now: instant(),
        }
    }

    fn basis(content: AnalysisSet, read_controls: bool) -> Basis {
        Basis {
            root: root().to_path_buf(),
            scope: ScanConfig { read_controls, ..ScanConfig::default() },
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
        let exclude = built(&RequestSpec { ignored: Some("exclude"), ..RequestSpec::new(root()) });
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
