//! Views over a built index, and the report they produce.
//!
//! Every view is a pure function of an index and a [`Selection`]: they read, and nothing
//! else. Producers submit observations and the index commits them; a report can never
//! become a third way to change state.
//!
//! # Two metadata query tiers
//!
//! An unfiltered request reads the roll-up state the index already maintains, so it costs
//! O(directories) for a tree and O(1) for a summary regardless of how many files the tree
//! holds. Any selection filter forces the other tier: the report walks the retained
//! entries and re-aggregates only what the filter admits, because a pre-computed roll-up
//! cannot answer a question about a subset. Both tiers are milliseconds warm and neither
//! touches the filesystem; the difference is visible in a profile, not in a user's wait.
//! Optional content I/O happens before this pure reader boundary and is retained in the
//! index's separate derived tier.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::classify::{Classification, ContentFamily, DetectionConfidence, DetectionSource};
use crate::content::{
    AnalysisSet, CodeMetrics, ContentDetection, ContentProvenance, CoverageReason, FileAnalysis,
    LogicalWordStats, MetricDef,
};
use crate::control::ControlCoverage;
use crate::engine_contract::{EntryKind, ScanScope};
use crate::index::{EntryId, ExtTally, Index, RollUpScalars, newer};
use crate::query::query_request::{Basis, NamedRoot, Request, RequestError, Roots};
use crate::query::query_selection::{
    Bound, IgnoredEntries, NameIdentity, Selection, ShareThreshold, SizeMetric, SortKey,
};
use crate::query::{Rejection, ReportProvenance, TreeStatus, query_subtrees};

/// Which roll-up or listing a view reports.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewSpec {
    /// Matching entries, rendered as a tree or a flat list by the format axis.
    List,
    /// Per-directory roll-ups down the hierarchy.
    Tree,
    /// One row per stable detected file type.
    Types,
    /// One row per raw derived extension.
    Extensions,
    /// One row per broad content family.
    Families,
    /// Code-family rows grouped by language/type.
    Languages,
    /// Code totals, language ranking, and population contributions.
    Code,
    /// Prose and markup rows with text-volume metrics.
    Documents,
    /// A flat listing of matching entries.
    Files,
    /// The largest files, by size.
    ///
    /// A named preset over [`Self::Files`], not separate machinery:
    /// `largest ≡ files --sort size --limit 20`, restricted to regular files.
    Largest,
    /// The most recently modified files.
    ///
    /// `recent ≡ files --sort mtime --limit 20`, restricted to regular files.
    Recent,
    /// One aggregate row for everything selected.
    Summary,
}

impl ViewSpec {
    /// The ordering this view uses when the caller did not choose one.
    fn default_sort(self) -> SortKey {
        match self {
            // Size-ranked by default, because "what is big" is the question these answer.
            Self::List
            | Self::Tree
            | Self::Types
            | Self::Extensions
            | Self::Families
            | Self::Languages
            | Self::Code
            | Self::Documents
            | Self::Summary
            // `largest` lands here for its own reason: it is named for the ranking,
            // so the ranking is not a display default but the view's whole content.
            | Self::Largest => SortKey::Size,
            // A complete listing reads and diffs in name order, which is the only
            // reason name order is right here: the stability that justifies it
            // disappears the moment the list is truncated, which is why `files` is
            // unbounded and the two bounded presets sort by what they are named for.
            Self::Files => SortKey::Name,
            Self::Recent => SortKey::Mtime,
        }
    }

    /// The bound this view applies when the caller named none.
    ///
    /// `files` enumerates, so it is complete: it stands in for `fd` and `find`, and the
    /// incremental-sync watermark query depends on it — a watermark that silently returns
    /// twenty of 192,871 changed files loses data rather than merely under-reporting.
    /// The presets are summaries and bound themselves; every other view keeps the
    /// display default.
    const fn default_limit(self) -> Bound {
        match self {
            Self::Largest | Self::Recent => Bound::Limit(20),
            _ => Bound::All,
        }
    }

    /// How deep a rendered tree descends when the caller named no depth.
    ///
    /// Two levels is what makes `fdu` answer "what is big here" at a glance: the root's
    /// children and theirs. Only the tree renders a hierarchy at all, so every other
    /// view is unbounded and the question does not arise.
    const fn default_depth(self) -> Bound {
        match self {
            Self::Tree => Bound::Limit(5),
            _ => Bound::All,
        }
    }

    /// Whether this view reports regular files only.
    ///
    /// `tree` already reports directory sizes, so a `largest` that listed directories
    /// would duplicate it at a coarser grain and push the actual files out of the window.
    const fn files_only(self) -> bool {
        matches!(self, Self::Largest | Self::Recent)
    }

    /// Every view, in the order a full report renders them.
    ///
    /// One list, so a front end cannot hold a stale copy: the Python binding kept its own
    /// view parser and silently rejected `largest` and `recent` for exactly that reason.
    pub const ALL: [Self; 12] = [
        Self::List,
        Self::Summary,
        Self::Tree,
        Self::Families,
        Self::Types,
        Self::Extensions,
        Self::Languages,
        Self::Code,
        Self::Documents,
        Self::Largest,
        Self::Recent,
        Self::Files,
    ];

    /// Parse one view name.
    ///
    /// Lives here rather than in a front end because it is the axis's grammar, not one
    /// surface's flag parsing — the CLI and the Python binding must accept exactly the
    /// same words or the two disagree about what a request means.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "list" => Ok(Self::List),
            "tree" => Ok(Self::Tree),
            "types" => Ok(Self::Types),
            "extensions" => Ok(Self::Extensions),
            "families" => Ok(Self::Families),
            "languages" => Ok(Self::Languages),
            "code" => Ok(Self::Code),
            "documents" => Ok(Self::Documents),
            "largest" => Ok(Self::Largest),
            "recent" => Ok(Self::Recent),
            "files" => Ok(Self::Files),
            "summary" => Ok(Self::Summary),
            // The expectation only. Each front end names its own flag and quotes the
            // offending token, so neither ends up saying "invalid --view invalid view".
            _ => Err(format!("expected one of {}", Self::vocabulary())),
        }
    }

    /// The accepted spellings, for an error message that teaches the vocabulary.
    pub fn vocabulary() -> String {
        let mut names: Vec<&str> = Self::ALL.iter().map(|view| view.label()).collect();
        names.push("full");
        names.join(", ")
    }

    /// Stable wire label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Tree => "tree",
            Self::Types => "types",
            Self::Extensions => "extensions",
            Self::Families => "families",
            Self::Languages => "languages",
            Self::Code => "code",
            Self::Documents => "documents",
            Self::Largest => "largest",
            Self::Recent => "recent",
            Self::Files => "files",
            Self::Summary => "summary",
        }
    }

    /// The analyzers this view shows, and so requests when a request builds its own basis.
    ///
    /// Only a view with no metadata meaning implies anything. `code` and `documents`
    /// display nothing without analysis, so naming one is asking for its analyzer, and
    /// refusing it would only make the caller type the analyzer as well. `languages`,
    /// `types`, and `families` gain metrics under an analyzer but are metadata reports
    /// without one, and `full` is the metadata digest: none of them implies anything,
    /// because a display choice with a cheap meaning must never turn into a read of every
    /// file in the tree with nothing in the command to say so.
    ///
    /// A match over every view, so a new one forces this decision. A basis an index
    /// already holds is never widened by it: a read refuses a view its basis cannot
    /// answer ([`RequestError::ViewNeedsAnalyzer`](crate::query::RequestError)).
    pub const fn implies(self) -> AnalysisSet {
        match self {
            Self::Code => AnalysisSet::CODE_ONLY,
            Self::Documents => AnalysisSet::WORDS_ONLY,
            Self::List
            | Self::Tree
            | Self::Types
            | Self::Extensions
            | Self::Families
            | Self::Languages
            | Self::Files
            | Self::Largest
            | Self::Recent
            | Self::Summary => AnalysisSet::NONE,
        }
    }

    /// The analyzers whose results this view displays, when its basis holds them.
    ///
    /// The other half of [`Self::implies`]: a view shows everything it implies, and the
    /// grouping views show more than they imply. `types`, `families`, and `languages` add
    /// line, code, and word columns to each row under whichever analyzers ran, while `code`
    /// shows only code analysis and `documents` only words. What a report says about
    /// analysis no selected view displays is decided against this table, so a request that
    /// pays for code analysis and shows only `documents` says so rather than staying silent
    /// because one view displayed something.
    ///
    /// A match over every view, so a new one forces this decision too.
    pub const fn shows(self) -> AnalysisSet {
        match self {
            Self::Types | Self::Families | Self::Languages => AnalysisSet::ALL,
            Self::Code => AnalysisSet::CODE_ONLY,
            Self::Documents => AnalysisSet::WORDS_ONLY,
            Self::List
            | Self::Tree
            | Self::Extensions
            | Self::Files
            | Self::Largest
            | Self::Recent
            | Self::Summary => AnalysisSet::NONE,
        }
    }

    /// The view a request displays its analysis in when the caller named none.
    ///
    /// The converse of [`Self::implies`], and free for the same reason: it re-projects
    /// state the request already pays for. Without it, a request that reads every
    /// eligible file reports a directory tree containing none of the results. For each
    /// view that implies an analyzer, this is that view, which a test pins so the two
    /// tables cannot drift.
    pub const fn default_for(analysis: AnalysisSet) -> Self {
        match (analysis.includes_code(), analysis.includes_words()) {
            (true, _) => Self::Code,
            (false, true) => Self::Documents,
            (false, false) if analysis.is_enabled() => Self::Families,
            (false, false) => Self::List,
        }
    }

    /// Resolve the view axis from a caller's spec against what the analyzers can answer.
    ///
    /// The whole job in one place: the list grammar, `full` expansion, and the default
    /// when the caller named nothing. All three lived in the CLI, so `--view tree,tree`
    /// was a typo there and a silent no-op through the Python API -- one request meaning
    /// two things depending on which door it came through (fdu-jozr) -- and the binding
    /// kept its own partial copy that had already drifted (fdu-ggux, fdu-gw5b).
    ///
    /// Returns the views to render and the ones `full` had to drop, so a caller can state
    /// the omission rather than hide it.
    ///
    /// `label` names the axis as the calling surface spells it, for the reason
    /// `AnalysisSet::parse_labeled` takes one: rewriting the message afterwards hits the
    /// user's own token (fdu-7j6z).
    pub fn resolve(
        spec: Option<&str>,
        analysis: AnalysisSet,
        label: &str,
    ) -> Result<(Vec<Self>, Vec<Self>), String> {
        Self::resolve_rejecting(spec, analysis).map_err(|rejection| rejection.labeled(label))
    }

    /// [`Self::resolve`], refusing with the value and expectation rather than a sentence, so
    /// the request model can name the axis in a typed refusal.
    pub(crate) fn resolve_rejecting(
        spec: Option<&str>,
        analysis: AnalysisSet,
    ) -> Result<(Vec<Self>, Vec<Self>), Rejection> {
        ViewList::parse(spec).map(|views| views.resolve(analysis))
    }

    /// The views a request displays when its caller named none: the one
    /// [`Self::default_for`] the analyzers, or both content views when both analyzers ran.
    pub fn defaults_for(analysis: AnalysisSet) -> Vec<Self> {
        if analysis.includes_code() && analysis.includes_words() {
            vec![Self::Code, Self::Documents]
        } else {
            vec![Self::default_for(analysis)]
        }
    }

    /// Why `full` cannot appear beside another view.
    ///
    /// Stated once, here, because it was stated twice: the CLI and the Python binding
    /// each carried their own copy, and the binding's had lost the trailing clause. Two
    /// copies of one rule drift silently, and a parity test comparing surface to surface
    /// only catches it when the drift reaches the wording (fdu-gw5b).
    pub const FULL_IS_EXCLUSIVE: &'static str =
        "it names the whole report and cannot be combined with another view";

    /// The summary views `full` expands to, given what the analyzers can answer.
    ///
    /// Returns the satisfiable views and those it had to skip, so a caller can report the
    /// omission rather than drop it silently. A view is satisfiable when the analyzers
    /// include everything it [`implies`](Self::implies), the same test a read applies to a
    /// view its caller named: `full` implies nothing, so it never reads for a view it
    /// contains.
    pub fn full_report(analysis: AnalysisSet) -> (Vec<Self>, Vec<Self>) {
        Self::ALL
            .into_iter()
            .filter(|view| {
                view.is_summary_view()
                    && (!analysis.includes_code() || !matches!(view, Self::Languages))
            })
            .partition(|view| analysis.contains(view.implies()))
    }

    /// Whether this view belongs in `--view full`.
    ///
    /// `full` is every *summary* view. `files` is an unbounded enumeration, and putting
    /// one inside a digest destroys the digest.
    pub const fn is_summary_view(self) -> bool {
        !matches!(self, Self::List | Self::Files)
    }
}

/// The view axis as its caller wrote it: parsed, and not yet resolved against the
/// analyzers that will answer it.
///
/// Parsing and resolving are two steps because a request that builds its own basis needs
/// what the named views imply before it can know its analyzers, and `full` and the default
/// can only be resolved once it does. The grammar runs once either way, so a refusal is
/// the same whichever step a caller stops at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ViewList {
    /// The caller named no view, so the analyzers choose ([`ViewSpec::defaults_for`]).
    Default,
    /// `full`: every summary view the analyzers can answer.
    Full,
    /// The views named, in the caller's order.
    Named(Vec<ViewSpec>),
}

impl ViewList {
    /// Read a comma list of view names, or `full` alone.
    pub(crate) fn parse(spec: Option<&str>) -> Result<Self, Rejection> {
        let Some(spec) = spec else {
            return Ok(Self::Default);
        };
        let mut parsed: Vec<ViewSpec> = Vec::new();
        let mut full_seen = false;
        for raw in spec.split(',') {
            let token = raw.trim();
            if token.is_empty() {
                return Err(Rejection::new(spec, "empty entry in the list"));
            }
            if token.eq_ignore_ascii_case("full") {
                if full_seen || !parsed.is_empty() {
                    return Err(Rejection::new("full", ViewSpec::FULL_IS_EXCLUSIVE));
                }
                full_seen = true;
                continue;
            }
            if full_seen {
                return Err(Rejection::new("full", ViewSpec::FULL_IS_EXCLUSIVE));
            }
            let view =
                ViewSpec::parse(token).map_err(|expected| Rejection::new(token, expected))?;
            if parsed.contains(&view) {
                return Err(Rejection::new(spec, format!("{token:?} appears more than once")));
            }
            parsed.push(view);
        }
        Ok(if full_seen { Self::Full } else { Self::Named(parsed) })
    }

    /// The named views that imply an analyzer, in the caller's order.
    ///
    /// Empty for `full` and for the default, which imply nothing: one is the metadata
    /// digest and the other is chosen *by* the analyzers.
    pub(crate) fn implying(&self) -> Vec<ViewSpec> {
        match self {
            Self::Named(views) => {
                views.iter().copied().filter(|view| view.implies().is_enabled()).collect()
            }
            Self::Default | Self::Full => Vec::new(),
        }
    }

    /// Every analyzer the named views imply.
    pub(crate) fn implies(&self) -> AnalysisSet {
        self.implying().into_iter().fold(AnalysisSet::NONE, |set, view| set.union(view.implies()))
    }

    /// The views to render and the ones `full` had to drop, given the analyzers.
    pub(crate) fn resolve(self, analysis: AnalysisSet) -> (Vec<ViewSpec>, Vec<ViewSpec>) {
        match self {
            Self::Default => (ViewSpec::defaults_for(analysis), Vec::new()),
            Self::Full => ViewSpec::full_report(analysis),
            Self::Named(views) => (views, Vec::new()),
        }
    }
}

/// What the calling surface calls the knobs a report's diagnostics name.
///
/// The same reason `ViewSpec::resolve` and `AnalysisSet::parse_labeled` take a label: a
/// rule belongs to the library, but the words a caller can act on belong to the surface
/// they came through. Telling a Python caller to "add --analyze" names a flag that does
/// not exist in their surface -- the defect that made these messages worth moving here in
/// the first place, reappearing one door over (fdu-4apt).
///
/// The view and analyzer axes both, because both diagnostics name both: the view that
/// cannot be answered, and the analyzer that would answer it. The two control limits,
/// because the note about refused `.gitignore` files names the limit that applies them.
/// The ignored-state selections and the observation switch, because selecting by ignored
/// state in a scan that reads no `.gitignore` is refused by naming both.
///
/// And every other axis a [`RequestError`](crate::query::RequestError) names: the value
/// grammars of the request model reject a value by naming its axis, and the watch
/// refusals name the knobs a watch cannot honor, so one type states each rule and each
/// surface supplies only its words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AxisNames {
    /// The view axis.
    pub view: &'static str,
    /// The output format axis.
    pub format: &'static str,
    /// The analyzer axis.
    pub analyze: &'static str,
    /// The budget on retained `.gitignore` state.
    pub control_budget: &'static str,
    /// The limit on one `.gitignore` line.
    pub control_line_limit: &'static str,
    /// The selection of unignored entries only.
    pub exclude_ignored: &'static str,
    /// The selection of ignored entries only.
    pub only_ignored: &'static str,
    /// The switch that turns `.gitignore` observation off.
    pub read_controls: &'static str,
    /// The retention depth of a scan.
    pub scan_depth: &'static str,
    /// The switch that keeps a scan on the root's filesystem.
    pub one_filesystem: &'static str,
    /// The switch that walks into what a symbolic link points at.
    pub follow_symlinks: &'static str,
    /// The patterns an entry must match.
    pub include: &'static str,
    /// The inclusive lower bound on modification time.
    pub modified_since: &'static str,
    /// The exclusive upper bound on modification time.
    pub modified_before: &'static str,
    /// The entry kinds a selection admits.
    pub kind: &'static str,
    /// The selection by ignored state, as one axis.
    pub ignored: &'static str,
    /// How deep a rendered tree descends.
    pub depth: &'static str,
    /// The minimum displayed share of the selected root.
    pub min_share: &'static str,
    /// Maximum child rows per directory.
    pub breadth: &'static str,
    /// How many rows a view keeps.
    pub limit: &'static str,
    /// The ordering key.
    pub sort: &'static str,
    /// The size metric.
    pub size: &'static str,
    /// The logical-word denominator of a document page.
    pub words_per_page: &'static str,
    /// The cache policy.
    pub cache: &'static str,
    /// The request to answer from the snapshot alone.
    pub stale_ok: &'static str,
    /// The request to repeat the answer as a watch.
    pub watch: &'static str,
    /// What joins several settings in one suggestion: flags read as one command line,
    /// keyword arguments as one call's arguments.
    pub setting_separator: &'static str,
}

impl AxisNames {
    /// How the command line spells them.
    ///
    /// `ignored` names the CLI's one `--ignored` value; the two specific selection
    /// values are available for diagnostics that prescribe one of them.
    ///
    /// `follow_symlinks` is the one axis this surface cannot set at all, so it keeps the
    /// library's name: a request carrying it came from a library or `open` caller, and
    /// naming a `--follow-symlinks` that does not exist would point them at the wrong door.
    pub const FLAGS: Self = Self {
        view: "--view",
        format: "--format",
        analyze: "--analyze",
        control_budget: "--gitignore-budget",
        control_line_limit: "--gitignore-line-limit",
        exclude_ignored: "--ignored=exclude",
        only_ignored: "--ignored=only",
        read_controls: "--no-gitignore",
        scan_depth: "--scan-depth",
        one_filesystem: "--one-filesystem",
        follow_symlinks: "follow_symlinks",
        include: "--include",
        modified_since: "--modified-since",
        modified_before: "--modified-before",
        kind: "--kind",
        ignored: "--ignored",
        depth: "--depth",
        min_share: "--min-share",
        breadth: "--breadth",
        limit: "--limit",
        sort: "--sort",
        size: "--size",
        words_per_page: "--words-per-page",
        cache: "--cache",
        stale_ok: "--stale-ok",
        watch: "--watch",
        setting_separator: " ",
    };

    /// How the library and the Python API spell them, and the default: a `Query` built
    /// without saying otherwise belongs to a library caller, not to the command line.
    ///
    /// `view` singular, matching the label the binding already passes to
    /// `ViewSpec::resolve`, so every diagnostic about this axis names it one way. It also
    /// keeps the difference from the command line to the flag dashes alone, which is what
    /// the parity harness's `surface-label` class checks.
    ///
    /// `cache` is the one name that is not a field: the Python parameter is `cache`, but its
    /// refusal has always said `invalid cache policy`, and this type moved the wording
    /// without changing it.
    pub const FIELDS: Self = Self {
        view: "view",
        format: "format",
        analyze: "analyze",
        control_budget: "control_budget",
        control_line_limit: "control_line_limit",
        exclude_ignored: "ignored=exclude",
        only_ignored: "ignored=only",
        read_controls: "read_controls",
        scan_depth: "max_depth",
        one_filesystem: "one_filesystem",
        follow_symlinks: "follow_symlinks",
        include: "include",
        modified_since: "modified_since",
        modified_before: "modified_before",
        kind: "kind",
        ignored: "ignored",
        depth: "depth",
        min_share: "min_share",
        breadth: "breadth",
        limit: "limit",
        sort: "sort",
        size: "size",
        words_per_page: "words_per_page",
        cache: "cache policy",
        stale_ok: "stale_ok",
        watch: "watch",
        setting_separator: ", ",
    };
}

impl Default for AxisNames {
    fn default() -> Self {
        Self::FIELDS
    }
}

/// What a report was asked for.
#[derive(Clone, Debug)]
pub struct Query {
    /// Which entries to consider and how to shape results.
    pub selection: Selection,
    /// Which views to report, in the order they were requested.
    pub views: Vec<ViewSpec>,
    /// Presentation requested before projecting retained entries.
    pub format: crate::report_format::Format,
    /// Views `full` had to drop because the requested analyzers cannot answer them.
    ///
    /// Carried so the report can name the omission rather than leave a caller to notice a
    /// section is missing. It lived in the CLI, which meant only the CLI could tell anyone
    /// (fdu-x8u6); `ViewSpec::resolve` returns it and this is where it lands.
    pub omitted_views: Vec<ViewSpec>,
    /// What the requesting surface calls the axes its diagnostics name.
    ///
    /// Carried on the request because that is what knows which door the caller came
    /// through; the rules themselves stay here and are each stated once.
    pub axes: &'static AxisNames,
    /// Fixed logical-word denominator used to derive page equivalents after aggregation.
    pub words_per_page: u64,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            selection: Selection::default(),
            views: Vec::new(),
            format: crate::report_format::Format::Text,
            omitted_views: Vec::new(),
            axes: &AxisNames::FIELDS,
            words_per_page: crate::query::Request::DEFAULTS.words_per_page,
        }
    }
}

impl Query {
    /// Whether this view needs the directory hierarchy rather than matching flat rows.
    pub fn tree_for(&self, view: ViewSpec) -> bool {
        use crate::report_format::Format;
        match view {
            ViewSpec::List => matches!(self.format, Format::Text | Format::Tree),
            ViewSpec::Tree => !matches!(self.format, Format::Paths | Format::Long),
            ViewSpec::Files => self.format == Format::Tree,
            _ => false,
        }
    }

    /// Whether this read needs directory candidates and their subtree measurements.
    pub(crate) fn needs_selection_walk(&self) -> bool {
        !self.selection.is_unfiltered()
            || self.views.iter().any(|view| {
                matches!(view, ViewSpec::List | ViewSpec::Tree | ViewSpec::Files)
                    && !self.tree_for(*view)
            })
    }

    /// The bound to apply for `view`: the caller's if they named one, else the view's own.
    pub fn limit_for(&self, view: ViewSpec) -> Bound {
        self.selection.limit.unwrap_or_else(|| {
            if self.tree_for(view) {
                ViewSpec::Tree.default_limit()
            } else if view == ViewSpec::Tree {
                Bound::All
            } else {
                view.default_limit()
            }
        })
    }

    /// The tree depth to apply for `view`, on the same terms as `limit_for`.
    pub fn depth_for(&self, view: ViewSpec) -> Bound {
        self.selection.depth.unwrap_or_else(|| {
            if self.tree_for(view) { ViewSpec::Tree.default_depth() } else { view.default_depth() }
        })
    }

    /// The independent per-directory breadth cap.
    pub fn breadth_for(&self) -> Bound {
        self.selection.breadth.unwrap_or(Bound::All)
    }

    /// The minimum root-relative share for tree rows.
    pub fn min_share_for(&self) -> ShareThreshold {
        self.selection.min_share.clone().unwrap_or_else(ShareThreshold::one_percent)
    }
}

/// Which tier of the freshness ladder produced the index behind a report.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReportSource {
    /// The tree was walked from scratch.
    ColdScan,
    /// A snapshot was loaded and revalidated against the filesystem.
    WarmRevalidate,
    /// A snapshot answered without the filesystem being consulted.
    CacheOnly,
}

impl ReportSource {
    /// The weaker of two sources, which is what an answer built from both can claim: one no
    /// filesystem verified is weaker than a revalidated one, which is weaker than a cold
    /// walk. A report over several roots takes it across them, as one takes it across tiers.
    #[must_use]
    pub fn weaker(self, other: Self) -> Self {
        let rank = |source| match source {
            Self::ColdScan => 0,
            Self::WarmRevalidate => 1,
            Self::CacheOnly => 2,
        };
        if rank(other) > rank(self) { other } else { self }
    }
}

/// One directory's row in a tree view.
#[derive(Clone, Debug)]
pub struct TreeNode {
    /// Path relative to the index root; empty for the root itself.
    pub path: PathBuf,
    /// Final path component, or `.` for the root.
    pub name: String,
    /// What the entry is.
    pub kind: EntryKind,
    /// Whether this entry itself is gitignored, or `None` when its classification is unknown.
    /// This is independent of the selected subtree's `ignored` tally.
    pub entry_ignored: Option<bool>,
    /// Apparent bytes in this subtree.
    pub bytes: u64,
    /// Allocated bytes in this subtree.
    pub allocated: u64,
    /// Files in this subtree.
    pub files: u64,
    /// Directories in this subtree.
    pub dirs: u64,
    /// The part of this subtree's tallies that `.gitignore` rules ignore, or `None` when
    /// governing controls were unobserved or could not be verified.
    ///
    /// Counted over the selected entries, like every other tally on the row, so it is zero
    /// when the selection excludes ignored entries and the whole row when it admits only
    /// them.
    pub ignored: Option<IgnoredTally>,
    /// Newest modification time among the regular files in this subtree, when it holds
    /// any: the files-only recency of [`SummaryRow::newest_mtime_ns`], which a summary
    /// reports and which no directory's own churn moves.
    pub newest_mtime_ns: Option<i64>,
    /// Newest modification time among the entries this row counts, or `None` when it
    /// counts none.
    ///
    /// The row's own entry counts when the selection admits it, and so does every entry
    /// beneath it that the row's tallies count, of any kind: a directory's time moves when
    /// an entry in it is created, renamed, or removed, which no surviving file's time
    /// records, and a symlink is an entry like any other. The report root is a traversal
    /// boundary rather than an entry, so its own time never counts, as no list row shows
    /// it either. Under a selection that admits everything this is a list row's
    /// [`FileRow::mtime_ns`] for the same directory, so a tree and `--long` agree; under a
    /// filter each follows its own row's population. A lower bound when
    /// [`Self::complete`] is false.
    pub mtime_ns: Option<i64>,
    /// Whether every directory in this row's subtree was listed in full, so its tallies and
    /// [`Self::mtime_ns`] are exact rather than lower bounds; `None` for a file.
    ///
    /// False at a scan-depth boundary, below a listing that failed, and in an opened root
    /// for a directory discovery has not listed yet, by the rule a list row's
    /// [`FileRow::complete`] follows.
    pub complete: Option<bool>,
    /// Signed nanoseconds from [`Self::mtime_ns`] to the report's
    /// [`Report::age_reference_ns`], negative for a time after it; `None` when the row
    /// counts nothing, when its subtree is incomplete, since a lower-bound maximum is not
    /// an age, or when the reference cannot be represented.
    pub age_ns: Option<i128>,
    /// Children reported beneath this node.
    pub children: Vec<TreeNode>,
    /// Disjoint child subtrees first excluded at this node's display boundary.
    pub omissions: Vec<TreeOmission>,
    /// Whether children were withheld by the depth or limit bound.
    pub truncated: bool,
}

/// Which independent display bound first removed a tree subtree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeOmissionReason {
    /// Below the root-relative minimum share.
    Share,
    /// Beyond the per-directory breadth cap.
    Breadth,
    /// Beyond the maximum displayed depth.
    Depth,
    /// Beyond the section's total data-row cap.
    Rows,
}

impl TreeOmissionReason {
    /// Stable machine spelling.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Share => "share",
            Self::Breadth => "breadth",
            Self::Depth => "depth",
            Self::Rows => "rows",
        }
    }
}

/// A disjoint group of direct child subtrees omitted by one display bound.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeOmission {
    /// The first exclusion boundary.
    pub reason: TreeOmissionReason,
    /// Direct child roots omitted at this boundary.
    pub entries: usize,
    /// Exact recursive regular-file tally, distinct from direct child roots.
    pub files: Option<u64>,
    /// Exact apparent remainder when the subtree measurements are complete.
    pub bytes: Option<u64>,
    /// Exact allocated remainder when the subtree measurements are complete.
    pub allocated: Option<u64>,
    /// Ignored part of the omitted subtrees, when their governing rules are known.
    pub ignored: Option<IgnoredSize>,
}

/// Exact ignored byte sizes on a disjoint omitted subtree.
/// Directory inode usage is not part of either size metric.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IgnoredSize {
    /// Apparent bytes of ignored regular files.
    pub bytes: u64,
    /// Allocated bytes of ignored regular files.
    pub allocated: u64,
}

impl IgnoredSize {
    fn from_tally(tally: IgnoredTally) -> Self {
        Self { bytes: tally.bytes, allocated: tally.allocated }
    }

    fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            bytes: self.bytes.checked_add(other.bytes)?,
            allocated: self.allocated.checked_add(other.allocated)?,
        })
    }
}

/// Content not represented by a listed row in a projected tree, shared by every format.
/// The root gives context; each displayed direct child represents its entire recursive
/// subtree, even where expansion below that child is bounded. Only the root's immediate
/// omission boundaries contribute, or the omitted root when no root row is shown.
/// Unknown or overflowing constituents propagate as `None`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeRemainder {
    /// Recursive regular files outside displayed top-level subtrees.
    pub files: Option<u64>,
    /// Apparent bytes in hidden subtrees.
    pub bytes: Option<u64>,
    /// Allocated bytes in hidden subtrees.
    pub allocated: Option<u64>,
    /// Ignored part of the disjoint hidden subtrees, when classification is known.
    /// Human bar coloring consumes this fact; the machine remainder schema is unchanged.
    pub ignored: Option<IgnoredSize>,
    /// Applicable boundaries in stable share, depth, breadth, row order.
    pub reasons: Vec<TreeOmissionReason>,
}

impl TreeRemainder {
    /// Summarize the selected population not represented by displayed top-level rows.
    pub fn from_tree(root: Option<&TreeNode>, omissions: &[TreeOmission]) -> Option<Self> {
        let mut result = Self {
            files: Some(0),
            bytes: Some(0),
            allocated: Some(0),
            ignored: Some(IgnoredSize::default()),
            reasons: Vec::new(),
        };
        let mut add = |omission: &TreeOmission| {
            result.files = result.files.zip(omission.files).and_then(|(a, b)| a.checked_add(b));
            result.bytes = result.bytes.zip(omission.bytes).and_then(|(a, b)| a.checked_add(b));
            result.allocated =
                result.allocated.zip(omission.allocated).and_then(|(a, b)| a.checked_add(b));
            result.ignored =
                result.ignored.zip(omission.ignored).and_then(|(a, b)| a.checked_add(b));
            if !result.reasons.contains(&omission.reason) {
                result.reasons.push(omission.reason);
            }
        };
        // A displayed child directory already carries all of its descendants in its
        // rollup. Omissions below it limit expansion, not the represented population.
        let boundary = root.map_or(omissions, |root| root.omissions.as_slice());
        for omission in boundary {
            add(omission);
        }
        result.reasons.sort_by_key(|why| match why {
            TreeOmissionReason::Share => 0,
            TreeOmissionReason::Depth => 1,
            TreeOmissionReason::Breadth => 2,
            TreeOmissionReason::Rows => 3,
        });
        (!result.reasons.is_empty()).then_some(result)
    }
}

/// Resolved, independent display bounds for one hierarchical section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeDisplayLimits {
    /// Deepest displayed entry; root is depth zero.
    pub depth: Bound,
    /// Root-relative minimum share.
    pub min_share: ShareThreshold,
    /// Child rows admitted per directory.
    pub breadth: Bound,
    /// Data rows admitted in the section, including the root.
    pub rows: Bound,
}

impl Drop for TreeNode {
    /// Release children iteratively.
    ///
    /// The derived drop glue recurses once per level, so a deeply nested tree would
    /// exhaust the stack on release even after every renderer was made iterative — the
    /// same hazard the index avoids when freeing a subtree. Taking the children out first
    /// turns that recursion into a loop.
    fn drop(&mut self) {
        let mut pending = std::mem::take(&mut self.children);
        while let Some(mut node) = pending.pop() {
            pending.extend(std::mem::take(&mut node.children));
        }
    }
}

/// The part of a row's tallies that `.gitignore` rules ignore.
///
/// An entry is ignored when a rule matches it or any ancestor directory, as git cannot
/// re-include a file below an excluded directory. Below a refused `.gitignore`
/// ([`Report::ignore_rules`]) the split is not exact in either direction; the sizes it
/// divides are.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct IgnoredTally {
    /// Ignored files.
    pub files: u64,
    /// Ignored directories. Always zero on an extension row, which counts files only.
    pub dirs: u64,
    /// Apparent bytes across ignored files.
    pub bytes: u64,
    /// Allocated bytes across ignored files.
    pub allocated: u64,
}

impl IgnoredTally {
    /// The ignored share of a roll-up: what `all` holds beyond `unignored`.
    pub(crate) fn between(all: RollUpScalars, unignored: RollUpScalars) -> Self {
        Self {
            files: all.files.saturating_sub(unignored.files),
            dirs: all.dirs.saturating_sub(unignored.dirs),
            bytes: all.bytes.saturating_sub(unignored.bytes),
            allocated: all.allocated.saturating_sub(unignored.allocated),
        }
    }

    fn add(&mut self, other: Self) {
        self.files = self.files.saturating_add(other.files);
        self.dirs = self.dirs.saturating_add(other.dirs);
        self.bytes = self.bytes.saturating_add(other.bytes);
        self.allocated = self.allocated.saturating_add(other.allocated);
    }
}

/// One extension's row in a types view.
#[derive(Clone, Debug)]
pub struct TypeRow {
    /// The derived extension, including its leading dot.
    pub extension: String,
    /// Files with this extension.
    pub files: u64,
    /// Apparent bytes across those files.
    pub bytes: u64,
    /// Allocated bytes across those files.
    pub allocated: u64,
    /// The ignored part of this row, or `None` when governing controls are unknown.
    pub ignored: Option<IgnoredTally>,
}

/// Dimension used by a generic metric-summary section.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MetricGroup {
    /// Stable detected file type or language ID.
    Type,
    /// Broad code/prose/markup/data/binary family.
    Family,
}

/// Exact share represented as an integer fraction.
///
/// A grouped section's rows partition its denominator: with no share filter or row bound,
/// their numerators sum to it exactly.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct MetricShare {
    /// This row's value in the section's share metric. On a section's total row it is the
    /// denominator itself, the sum of the rows, which for document words can differ from
    /// the total row's own pooled `document_words`.
    pub numerator: u64,
    /// Sum of every row's numerator before the share filter and display truncation.
    pub denominator: u64,
}

/// Metric used as the numerator and denominator of grouped percentages.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShareMetric {
    /// Apparent file bytes selected by the query.
    ApparentBytes,
    /// Allocated filesystem bytes selected by the query.
    AllocatedBytes,
    /// Standard code lines from `code-sloc-v1`.
    CodeLines,
    /// Raw or reader-visible normalized document words, selected by analysis depth.
    DocumentWords,
    /// Whitespace-delimited words from the shared lines unit.
    RawWords,
}

impl ShareMetric {
    /// Stable machine label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ApparentBytes => "apparent_bytes",
            Self::AllocatedBytes => "allocated_bytes",
            Self::CodeLines => "code_lines",
            Self::DocumentWords => "document_words",
            Self::RawWords => "raw_words",
        }
    }
}

/// One stable group in a metric-summary section.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ReportMetricValues {
    /// Physical lines, present with the lines unit.
    pub physical_lines: Option<u64>,
    /// Blank lines, present with the lines unit.
    pub blank_lines: Option<u64>,
    /// Nonblank lines, present with the lines unit.
    pub nonblank_lines: Option<u64>,
    /// Raw words, present with the lines unit.
    pub raw_words: Option<u64>,
    /// Code lines, present with the code unit.
    pub code_lines: Option<u64>,
    /// Comment lines, present with the code unit.
    pub comment_lines: Option<u64>,
    /// Code-analyzer blank lines, present with the code unit.
    pub code_blank_lines: Option<u64>,
    /// Logical words, present with the words unit.
    pub logical_words: Option<u64>,
    /// Paragraphs, present with the words unit.
    pub paragraphs: Option<u64>,
    /// Reader-visible words, present with the words unit.
    pub visible_words: Option<u64>,
    /// Reader-visible logical words, present with the words unit.
    pub visible_logical_words: Option<u64>,
    /// Query-selected document words, present with the words unit.
    pub document_words: Option<u64>,
}

impl ReportMetricValues {
    fn for_analysis(analysis: AnalysisSet) -> Self {
        let lines = analysis.is_enabled().then_some(0);
        let code = analysis.includes_code().then_some(0);
        let words = analysis.includes_words().then_some(0);
        Self {
            physical_lines: lines,
            blank_lines: lines,
            nonblank_lines: lines,
            raw_words: lines,
            code_lines: code,
            comment_lines: code,
            code_blank_lines: code,
            logical_words: words,
            paragraphs: words,
            visible_words: words,
            visible_logical_words: words,
            document_words: words,
        }
    }

    fn add_assign(&mut self, other: &Self) {
        add_optional(&mut self.physical_lines, other.physical_lines);
        add_optional(&mut self.blank_lines, other.blank_lines);
        add_optional(&mut self.nonblank_lines, other.nonblank_lines);
        add_optional(&mut self.raw_words, other.raw_words);
        add_optional(&mut self.code_lines, other.code_lines);
        add_optional(&mut self.comment_lines, other.comment_lines);
        add_optional(&mut self.code_blank_lines, other.code_blank_lines);
        add_optional(&mut self.paragraphs, other.paragraphs);
        add_optional(&mut self.visible_words, other.visible_words);
    }
}

fn add_optional(total: &mut Option<u64>, value: Option<u64>) {
    if let (Some(total), Some(value)) = (total, value) {
        *total = total.saturating_add(value);
    }
}

/// One stable group in a metric-summary section.
#[derive(Clone, Debug)]
pub struct MetricRow {
    /// Analyzer units requested for this row.
    pub analysis: AnalysisSet,
    /// Stable type or family label.
    pub id: String,
    /// Broad family for type-grouped rows.
    pub family: ContentFamily,
    /// Matching regular files.
    pub files: u64,
    /// Apparent bytes.
    pub bytes: u64,
    /// Allocated bytes.
    pub allocated: u64,
    /// Files whose requested metrics completed.
    pub analyzed_files: u64,
    /// Additive content metric slots.
    pub metrics: ReportMetricValues,
    /// Additive sufficient statistics behind `logical_words`.
    pub(crate) logical_word_stats: LogicalWordStats,
    /// Additive sufficient statistics behind `visible_logical_words`.
    pub(crate) visible_logical_word_stats: LogicalWordStats,
    /// Query-selected raw document words before normalization.
    pub document_raw_words: u64,
    /// Additive sufficient statistics for the query-selected document projection.
    pub document_word_stats: LogicalWordStats,
    /// Analyzed files that supplied normalized document statistics.
    pub document_metric_files: u64,
    /// Explicit content-analysis outcomes.
    pub coverage: BTreeMap<CoverageReason, u64>,
    /// Lines-unit outcomes.
    pub lines_coverage: BTreeMap<CoverageReason, u64>,
    /// Code-unit outcomes when requested.
    pub code_coverage: Option<BTreeMap<CoverageReason, u64>>,
    /// Words-unit outcomes when requested.
    pub words_coverage: Option<BTreeMap<CoverageReason, u64>>,
    /// Files by the classification tier that established their type.
    pub detection_sources: BTreeMap<DetectionSource, u64>,
    /// Files by classification confidence.
    pub detection_confidence: BTreeMap<DetectionConfidence, u64>,
    /// Files carrying a bounded generated-file marker.
    pub generated_files: u64,
    /// Files below a conventional vendored path.
    pub vendored_files: u64,
    /// Files below a conventional documentation path or basename.
    pub documentation_files: u64,
    /// Exact share in the report's selected size metric.
    pub share: MetricShare,
}

impl MetricRow {
    /// Return one registry metric when its owning analyzer unit was requested.
    pub fn metric_value(&self, metric: &MetricDef) -> Option<u64> {
        if !self.analysis.contains(metric.owner) {
            return None;
        }
        match metric.name {
            "physical_lines" => self.metrics.physical_lines,
            "blank_lines" => self.metrics.blank_lines,
            "nonblank_lines" => self.metrics.nonblank_lines,
            "raw_words" => self.metrics.raw_words,
            "code_lines" => self.metrics.code_lines,
            "comment_lines" => self.metrics.comment_lines,
            "code_blank_lines" => self.metrics.code_blank_lines,
            "logical_words" => self.metrics.logical_words,
            "paragraphs" => self.metrics.paragraphs,
            "visible_words" => self.metrics.visible_words,
            "visible_logical_words" => self.metrics.visible_logical_words,
            "document_words" => self.metrics.document_words,
            _ => None,
        }
    }

    fn finish_derived_metrics(&mut self) {
        if self.analysis.includes_words() {
            self.metrics.logical_words = Some(self.logical_word_stats.logical_words());
            self.metrics.visible_logical_words =
                Some(self.visible_logical_word_stats.logical_words());
            self.metrics.document_words = Some(self.document_word_stats.logical_words());
        }
    }
}

/// Totals and grouped rows for types, families, languages, or documents.
#[derive(Clone, Debug)]
pub struct MetricSummary {
    /// Grouping dimension.
    pub group: MetricGroup,
    /// Totals across every row before display truncation.
    ///
    /// Logical, visible logical, and document words are derived from the pooled
    /// statistics, so they can differ from the sum of the rows' values. The total's share
    /// is the rows' sum over itself, the denominator every row's share uses.
    pub total: MetricRow,
    /// Sorted, display-bounded rows.
    pub rows: Vec<MetricRow>,
    /// Rows before the bound was applied.
    pub total_rows: usize,
    /// Rows removed by the explicit minimum-share filter before the row bound.
    pub share_omitted: usize,
    /// Metric used for every row's exact share.
    pub share_metric: ShareMetric,
    /// Logical words per derived page.
    pub words_per_page: u64,
}

/// Code measurements for one selected file population.
#[derive(Clone, Debug, Default)]
pub struct CodeTally {
    /// Code-family regular files, including files the analyzer could not measure.
    pub source_files: u64,
    /// Files with a successful code result.
    pub analyzed_files: u64,
    /// Sum of successful code results; coverage identifies unavailable contributions.
    pub metrics: CodeMetrics,
    /// Outcome counts for records that reached the code analyzer.
    pub coverage: BTreeMap<CoverageReason, u64>,
    /// Selected source files without a retained analyzer record.
    pub missing_records: u64,
}

impl CodeTally {
    fn add_file(&mut self, record: Option<&crate::content::FileAnalysis>) {
        self.source_files = self.source_files.saturating_add(1);
        match record.and_then(|record| record.code) {
            Some(outcome) => {
                *self.coverage.entry(outcome.coverage()).or_default() += 1;
                if let Some(value) = outcome.value() {
                    self.analyzed_files = self.analyzed_files.saturating_add(1);
                    self.metrics.code_lines =
                        self.metrics.code_lines.saturating_add(value.code_lines);
                    self.metrics.comment_lines =
                        self.metrics.comment_lines.saturating_add(value.comment_lines);
                    self.metrics.code_blank_lines =
                        self.metrics.code_blank_lines.saturating_add(value.code_blank_lines);
                }
            }
            None => self.missing_records = self.missing_records.saturating_add(1),
        }
    }
}

/// Complete measured contribution of one detected code language.
#[derive(Clone, Debug)]
pub struct CodeLanguageRow {
    /// Stable file-type ID.
    pub language: String,
    /// All selected files in this language.
    pub selected: CodeTally,
    /// Selected files known to be non-ignored, when classification was observed.
    pub non_ignored: Option<CodeTally>,
    /// Selected files known to be ignored, when classification was observed.
    pub ignored: Option<CodeTally>,
    /// Selected files whose ignore classification is unknown.
    pub unknown: CodeTally,
    /// Share of measured code lines in the selected population.
    pub share: MetricShare,
}

/// Code-first projection with explicit coverage and display omissions.
#[derive(Clone, Debug)]
pub struct CodeOverview {
    /// Selected ignored population for this answer.
    pub population: IgnoredEntries,
    /// Totals for the selected population before display bounds.
    pub selected: CodeTally,
    /// Known non-ignored contribution when classification was observed.
    pub non_ignored: Option<CodeTally>,
    /// Known ignored contribution when classification was observed.
    pub ignored: Option<CodeTally>,
    /// Contribution whose ignore classification could not be established.
    pub unknown: CodeTally,
    /// Selected regular files whose type cascade did not establish a family.
    pub unclassified_files: u64,
    /// Languages with at least one analyzed source file.
    pub analyzed_languages: u64,
    /// Language rows eligible before the display row bound.
    pub total_languages: usize,
    /// Rows removed by the explicit minimum-share filter before the row bound.
    pub share_omitted: usize,
    /// Detected code languages ordered by the resolved sort and stable name ties.
    pub languages: Vec<CodeLanguageRow>,
    /// Denominator of language shares, always measured selected code lines.
    pub share_metric: ShareMetric,
}

/// Analyzer identity attached to a content-capable report.
#[derive(Clone, Debug)]
pub struct ContentReportMetadata {
    /// Requested analysis profile.
    pub profile: AnalysisSet,
    /// Type-rule, option, and analyzer dialect identity.
    pub provenance: ContentProvenance,
}

/// One matching entry in a flat view. Directory size and mtime describe its subtree
/// after exclusions; other entries carry their own metadata. Nested rows may overlap.
#[derive(Clone, Debug)]
pub struct FileRow {
    /// Position of the row's root among the report's roots ([`Report::roots`]); 0 for a
    /// report over one root.
    pub root: usize,
    /// Path relative to the row's root.
    pub path: PathBuf,
    /// What the entry is.
    pub kind: EntryKind,
    /// Apparent bytes.
    pub bytes: u64,
    /// Allocated bytes.
    pub allocated: u64,
    /// Modification time in nanoseconds since the Unix epoch.
    pub mtime_ns: i64,
    /// Descendant regular files for a directory; absent for other entry kinds.
    pub files: Option<u64>,
    /// Descendant directories, excluding the matching root; absent for other kinds.
    pub dirs: Option<u64>,
    /// Whether a directory's eligible subtree was listed in full, so its bytes, counts,
    /// and modification time are exact; `Some(false)` makes them lower bounds and its age
    /// unknown. Absent for other kinds.
    pub complete: Option<bool>,
    /// Signed nanoseconds since modification at the request's reference instant, or
    /// `None` when the reference is unrepresentable or the subtree is incomplete.
    pub age_ns: Option<i128>,
    /// Whether `.gitignore` rules ignore this entry, or `None` when its governing rules
    /// were unobserved or could not be verified.
    pub ignored: Option<bool>,
    /// Value of the requested numeric metric used for sorting, when measured.
    pub sort_value: Option<u64>,
    /// Existing type and heuristic evidence for regular files.
    pub classification: Option<ContentDetection>,
}

/// The aggregate row of a summary view.
#[derive(Clone, Copy, Debug, Default)]
pub struct SummaryRow {
    /// Files selected.
    pub files: u64,
    /// Directories selected.
    pub dirs: u64,
    /// Apparent bytes.
    pub bytes: u64,
    /// Allocated bytes.
    pub allocated: u64,
    /// The ignored part of what was selected, or `None` when governing rules were
    /// unobserved or could not be verified for a contributing entry.
    pub ignored: Option<IgnoredTally>,
    /// Newest modification time, when anything was selected.
    pub newest_mtime_ns: Option<i64>,
}

/// One view's results.
#[derive(Clone, Debug)]
pub enum Section {
    /// A code-first overview with selected totals and bounded language rows.
    Code(Box<CodeOverview>),
    /// A tree view.
    Tree {
        /// The view whose directory hierarchy is shown.
        view: ViewSpec,
        /// Bounds resolved before projection.
        limits: TreeDisplayLimits,
        /// The bounded directory roll-ups of a report over one root, or `None` when the row
        /// limit is zero or the report has several roots.
        root: Option<Box<TreeNode>>,
        /// What the section's top boundary omitted: the root of one root's tree under a
        /// zero row limit, or the root rows the row limit cut from several roots' trees.
        omissions: Vec<TreeOmission>,
        /// The rows of a report over several roots, which has no single root node: a total
        /// and one ordinary tree per root. `None` for a report over one root.
        roots: Option<Box<RootTrees>>,
    },
    /// A raw-extension view.
    Extensions {
        /// The rows, already sorted and bounded.
        rows: Vec<TypeRow>,
        /// Rows before the bound was applied.
        total: usize,
        /// Rows removed by the explicit minimum-share filter before the row bound.
        share_omitted: usize,
    },
    /// A generic type/family content summary.
    Metrics {
        /// Requested preset that selected grouping and family filters.
        view: ViewSpec,
        /// Generic grouped metrics.
        summary: Box<MetricSummary>,
    },
    /// A flat listing: `files`, or one of its bounded presets.
    ///
    /// Carries the view for the same reason `Metrics` does — three views share this shape
    /// and a reader has to be told which one produced the rows.
    Files {
        /// Which of `files`, `largest`, or `recent` produced these rows.
        view: ViewSpec,
        /// The rows, already sorted and bounded for that view.
        rows: Vec<FileRow>,
        /// Rows before the bound was applied.
        ///
        /// Reported so a bound can never be silent: twenty rows of 192,871 look complete
        /// unless the report says otherwise, and a consumer reading the machine format
        /// has no other way to tell.
        total: usize,
    },
    /// A summary view.
    Summary(SummaryRow),
}

/// The rows of a tree over several roots.
///
/// No synthetic node joins the roots: a tree node is an entry, with a kind from the
/// snapshot's vocabulary, and the total is none. So the section carries the total beside
/// the trees, and each tree is the one its root alone would have, measured against the
/// combined total.
#[derive(Clone, Debug)]
pub struct RootTrees {
    /// The first row: every root's totals together, or `None` when the row limit is zero.
    pub total: Option<TreeTotal>,
    /// The trees the row limit kept, in display order: the tree sorter's, with each root's
    /// label as its name.
    pub trees: Vec<RootTree>,
}

/// One root's tree in a report over several roots.
#[derive(Clone, Debug)]
pub struct RootTree {
    /// Position of the tree's root among the report's roots.
    pub root: usize,
    /// The root's row and what it shows beneath it, named by the root's label, with paths
    /// relative to the root. Every root is a row, whatever its share, since its caller
    /// named it; its own bounds are recorded on it as one root's would be.
    pub tree: TreeNode,
}

/// The total row of a tree over several roots: the merge of their root rows.
///
/// The same values a [`TreeNode`] carries, without the identity of an entry: no path,
/// name, or kind, because no entry is the total.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeTotal {
    /// Apparent bytes under every root.
    pub bytes: u64,
    /// Allocated bytes under every root.
    pub allocated: u64,
    /// Files under every root.
    pub files: u64,
    /// Directories under every root.
    pub dirs: u64,
    /// The ignored part of the totals, or `None` when any root's is unknown.
    pub ignored: Option<IgnoredTally>,
    /// The newest regular file under any root ([`TreeNode::newest_mtime_ns`]).
    pub newest_mtime_ns: Option<i64>,
    /// The newest activity any root's row counts ([`TreeNode::mtime_ns`]).
    pub mtime_ns: Option<i64>,
    /// Whether every root was listed in full.
    pub complete: bool,
    /// Signed nanoseconds from [`Self::mtime_ns`] to the report's age reference, on the
    /// terms of [`TreeNode::age_ns`].
    pub age_ns: Option<i128>,
}

impl Section {
    /// Which view produced this section.
    pub fn view(&self) -> ViewSpec {
        match self {
            Self::Code(_) => ViewSpec::Code,
            Self::Extensions { .. } => ViewSpec::Extensions,
            Self::Tree { view, .. } | Self::Metrics { view, .. } | Self::Files { view, .. } => {
                *view
            }
            Self::Summary(_) => ViewSpec::Summary,
        }
    }
}

/// A rendered answer: provenance, plus one section per requested view.
#[derive(Clone, Debug)]
pub struct Report {
    /// Instant used for modification age, in epoch nanoseconds; unknown outside i64.
    pub age_reference_ns: Option<i64>,
    /// Requested presentation; Text resolves from each section projection.
    pub format: crate::report_format::Format,
    /// Completeness and bounded failure detail for this answer.
    pub status: TreeStatus,
    /// Source, currency, and timing for this answer and its retained tiers.
    pub provenance: ReportProvenance,
    /// The semantic scan scope represented by this report.
    ///
    /// A projected cache load constructs its index directly in the requested controls-off
    /// scope, so every report route reads this value from the same requested-scope index.
    pub scope: ScanScope,
    /// Analyzer units requested for this answer.
    pub requested_analysis: AnalysisSet,
    /// Resolved views requested and answerable by this analyzer set.
    pub requested_views: Vec<ViewSpec>,
    /// Resolved views requested but unavailable from this analyzer set.
    pub omitted_views: Vec<ViewSpec>,
    /// Absolute path of the indexed root of a report over one root, and `None` for a
    /// report over several, which names them in [`Self::roots`].
    ///
    /// Exactly one of the two is set, so a report over one root needs no label and has the
    /// same shape on every route: a one-shot report, an opened root's, and a watch's.
    pub root: Option<PathBuf>,
    /// The roots of a report over several, each with its label and canonical path, in the
    /// caller's order; `None` for a report over one root.
    ///
    /// Every path in the report stays relative to its own root, and rows, status errors,
    /// and `.gitignore` refusals carry the position of that root here, so
    /// `roots[row.root].path.join(&row.path)` names an entry exactly. Text prints each
    /// path after its root's label instead.
    pub roots: Option<Vec<NamedRoot>>,
    /// Remarks about the report itself, in the order a renderer should print them.
    ///
    /// Facts about what was asked for and what could be answered -- not telemetry about
    /// the run, which the schema deliberately excludes. Deliberately not serialised: a
    /// machine consumer reads the omission from which sections are absent, and adding a
    /// field to the envelope would be a schema change for something only humans read.
    pub notes: Vec<String>,
    /// Actionable suggestions, rendered once after factual notes; excluded from wire data.
    pub tips: Vec<String>,
    /// Surface vocabulary for actionable display-bound suggestions.
    pub axes: &'static AxisNames,
    /// Which size metric this report answers in.
    ///
    /// Carried on the report so a renderer shows the same number the ordering used;
    /// printing apparent bytes beside an allocated-bytes ranking looks like a sorting
    /// bug and is worse than either metric alone.
    pub size: SizeMetric,
    /// Registered numeric metric used for sorting, when selected.
    pub sort_metric: Option<&'static str>,
    /// Analyzer identity when sparse content records are present.
    pub analysis: Option<ContentReportMetadata>,
    /// Which entries the rows count by `.gitignore` classification.
    ///
    /// Carried for the renderer, like [`Self::size`], and not serialised: a row whose
    /// selection admits only ignored entries is wholly ignored, so text leaves the ignored
    /// share off rather than repeat the size beside it.
    pub ignored_entries: IgnoredEntries,
    /// Whether ignore classification applies every `.gitignore` in scope, serialised as
    /// `ignore_rules`.
    ///
    /// Not operational completeness: a refused control file leaves [`TreeStatus::complete`]
    /// true and every size exact, and costs only the ignored and unignored split below
    /// it. [`Self::notes`] names the directories and the knob that applies them.
    pub ignore_rules: ControlCoverage,
    /// One section per requested view, in request order.
    pub sections: Vec<Section>,
}

/// Remarks a report makes about itself.
///
/// Only what the request, the analyzers its basis holds, the resolved views, and the
/// index's coverage can establish. The CLI also prints a note quoting how many bytes
/// analysis read, which is walk telemetry the report envelope does not carry, so that one
/// stays with the performance footer where the rest of the run's telemetry lives.
///
/// Two remarks relate the analyzers to the views, one for each direction a request can
/// leave them unmatched. `full` implies nothing, so it names the views it skipped and the
/// one value of the analyzer axis that includes them alongside what already ran. And
/// analysis no selected view displays -- warming the sidecar is a supported use, so this
/// is a note rather than an error -- is named analyzer by analyzer against
/// [`ViewSpec::shows`], with a metric sort counting as a display of its analyzer. When no
/// selected view shows any analysis, the tip names the views [`ViewSpec::defaults_for`]
/// the analyzers; when some is shown, it keeps the caller's views and adds the ones that
/// show the rest, so following it never drops what the caller already sees.
///
/// `roots` names a report's roots when it has several, whose labels a note naming a
/// directory puts before it, as text puts a label before every path; `None` for one root.
fn display_notes(
    query: &Query,
    content: AnalysisSet,
    ignore_rules: &ControlCoverage,
    roots: Option<&[NamedRoot]>,
) -> (Vec<String>, Vec<String>) {
    let mut notes = Vec::new();
    let mut tips = Vec::new();
    let labels = |views: &[ViewSpec]| views.iter().map(|view| view.label()).collect::<Vec<_>>();
    if !query.omitted_views.is_empty() {
        let needed = query
            .omitted_views
            .iter()
            .fold(AnalysisSet::NONE, |set, view| set.union(view.implies()));
        let held = content.labels();
        let missing: Vec<&str> =
            needed.named().into_iter().filter(|name| !held.contains(name)).collect();
        let without = if content.is_enabled() {
            format!("{} analysis", missing.join(" and "))
        } else {
            "analysis".to_owned()
        };
        notes.push(format!(
            "note: full omits {} without {without}",
            labels(&query.omitted_views).join(", ")
        ));
        tips.push(format!(
            "tip: include them: {} {}",
            query.axes.analyze,
            content.union(needed).request_label()
        ));
    }
    if let Some((note, tip)) = refused_controls_note(ignore_rules, query.axes, roots) {
        notes.push(note);
        tips.extend(tip);
    }
    // A metric sort uses its analyzer even where no column shows it.
    let ranked = match query.selection.sort {
        Some(SortKey::Metric(name)) => crate::content::METRICS
            .iter()
            .find(|metric| metric.name == name)
            .map_or(AnalysisSet::NONE, |metric| metric.owner),
        _ => AnalysisSet::NONE,
    };
    let shown = query.views.iter().fold(ranked, |set, view| set.union(view.shows()));
    let unshown = unshown_analysis(content, shown);
    if unshown.is_enabled() {
        notes.push(format!(
            "note: {} analysis not shown by {}",
            unshown.named().join(" and "),
            labels(&query.views).join(", ")
        ));
        let mut views = if shown.is_enabled() { query.views.clone() } else { Vec::new() };
        views.extend(ViewSpec::defaults_for(unshown));
        tips.push(format!("tip: show it: {} {}", query.axes.view, labels(&views).join(",")));
    }
    (notes, tips)
}

/// The analyzers in `content` that `shown` does not display, as a set a caller could
/// request.
///
/// Every view that shows any analysis shows the shared line pass, so line counts go unshown
/// only when nothing is shown at all, and then the whole set is.
fn unshown_analysis(content: AnalysisSet, shown: AnalysisSet) -> AnalysisSet {
    if !shown.is_enabled() {
        return content;
    }
    let mut unshown = AnalysisSet::NONE;
    if content.includes_code() && !shown.includes_code() {
        unshown = unshown.with_code();
    }
    if content.includes_words() && !shown.includes_words() {
        unshown = unshown.with_words();
    }
    unshown
}

/// Directories a refused-controls note names before it counts the rest.
const REFUSED_DIRECTORIES_NAMED: usize = 5;

/// Say which `.gitignore` files were not applied, why, where, and what applies them.
///
/// The truncation states itself: the note names a few directories and counts the rest,
/// and a structured report lists up to [`crate::MAX_RETAINED_ISSUES`] beside the exact
/// count. Each limit is named with the refusals it caused only when every refusal is
/// listed; otherwise the note names every limit that could have refused an unlisted file.
/// The remedy raises exactly the limits it named, each by the name the requesting surface
/// uses, so lifting one never reads as lifting the other.
fn refused_controls_note(
    ignore_rules: &ControlCoverage,
    axes: &AxisNames,
    labels: Option<&[NamedRoot]>,
) -> Option<(String, Option<String>)> {
    use crate::control::ControlRefusalReason::{Budget, LineLimit};

    let ControlCoverage::Observed(observed) = ignore_rules else {
        return None;
    };
    if observed.is_complete() {
        return None;
    }
    let every_listed = observed.lists_every_refusal();
    let listed =
        |reason| observed.refusals.iter().filter(|refusal| refusal.reason == reason).count();
    // A listed reason certainly fired. When the list is truncated, a bounded limit may also
    // have refused an unlisted file; an unbounded one refuses nothing.
    let fired: Vec<_> = [Budget, LineLimit]
        .into_iter()
        .filter(|reason| {
            listed(*reason) > 0 || (!every_listed && observed.limits.limit_for(*reason).is_some())
        })
        .collect();
    let size = |reason| {
        observed.limits.limit_for(reason).map(|bytes| {
            crate::report_format::human_bytes(u64::try_from(bytes).unwrap_or(u64::MAX))
        })
    };
    // A refusal recorded under an unbounded limit names the limit without a size, never a
    // zero one.
    let over = |reason| {
        let (lead, noun) = match reason {
            Budget => ("over", "ignore-rule budget"),
            LineLimit => ("with a line over", "line limit"),
        };
        size(reason).map_or_else(
            || format!("{lead} the {noun}"),
            |size| format!("{lead} the {size} {noun}"),
        )
    };
    let why = if every_listed {
        let parts: Vec<String> =
            fired.iter().map(|reason| format!("{} {}", listed(*reason), over(*reason))).collect();
        parts.join(", ")
    } else {
        let parts: Vec<String> = fired.iter().map(|reason| over(*reason)).collect();
        parts.join(" or ")
    };

    let shown = observed.refusals.len().min(REFUSED_DIRECTORIES_NAMED);
    let mut directories: Vec<String> = observed.refusals[..shown]
        .iter()
        .map(|refusal| {
            let parent = refusal.path.parent().unwrap_or(Path::new(""));
            let shown = crate::query::labelled_path(labels, refusal.root, parent);
            // One root's own directory has no path to print, and is named as `.`.
            if shown.as_os_str().is_empty() { ".".to_owned() } else { shown.display().to_string() }
        })
        .collect();
    let unnamed = observed.refused.saturating_sub(u64::try_from(shown).unwrap_or(u64::MAX));
    if unnamed > 0 {
        directories.push(format!("{} more", crate::report_format::human_count(unnamed)));
    }

    // Only a bounded limit can be raised.
    let raises: Vec<String> = fired
        .iter()
        .filter_map(|reason| {
            let knob = match reason {
                Budget => axes.control_budget,
                LineLimit => axes.control_line_limit,
            };
            size(*reason).map(|size| format!("{knob} above {size}"))
        })
        .collect();
    let remedy = match raises.as_slice() {
        [] => None,
        [raise] => {
            Some(format!("tip: apply refused ignore files: raise {raise}, or set it to all"))
        }
        raises => Some(format!(
            "tip: apply refused ignore files: raise {}, or set them to all",
            raises.join(" and ")
        )),
    };
    let files = crate::report_format::human_count(observed.refused);
    let noun = if observed.refused == 1 { "file" } else { "files" };
    Some((
        format!(
            "note: ignore classification incomplete: {files} ignore {noun} not applied ({why}); affected: {}",
            directories.join(", ")
        ),
        remedy,
    ))
}

/// Build a report from an index.
///
/// Pure: the same index, request, and provenance always produce the same report, and
/// nothing here reads the filesystem or mutates the index.
///
/// # Errors
///
/// [`Error::InvalidRequest`](crate::Error::InvalidRequest) when this index cannot answer
/// the request: it holds another analyzer set than the read asks for, a view needs content
/// nothing analyzed, or the request selects by ignored state
/// ([`Selection::ignored`]) over an index that read no `.gitignore` -- which can say of no
/// entry that it is ignored or that it is not, so the request is refused rather than
/// answered with every entry or none.
pub fn report(
    index: &Index,
    request: &Request,
    generated_at: std::time::SystemTime,
) -> crate::Result<Report> {
    report_in(index, request, generated_at, NameIdentity::Native)
}

/// [`report`], with the selection evaluated against the named spelling of each path.
///
/// A one-shot report matches native names; an opened-root read matches portable ones, so
/// its report projection agrees with its flat and aggregate projections over one query.
pub(crate) fn report_in(
    index: &Index,
    request: &Request,
    generated_at: std::time::SystemTime,
    identity: NameIdentity,
) -> crate::Result<Report> {
    read_roots(&[index], None, request, generated_at, identity)
}

/// Build one report over several roots from one index per root, in the roots' order.
///
/// The report is the sum of the reports over each root, merged before any display bound:
/// sizes, counts, rows, shares, and bounds are taken over the union, once. A tree has a
/// total row and one tree per root, rows and issues carry the position of their root in
/// [`Report::roots`], and every path stays relative to its own root. With one root this is
/// [`report`], unchanged: no total, no labels, and [`Report::root`] set.
///
/// Pure, as [`report`] is. A caller that already holds an index per root, from
/// [`open`](crate::open) or [`scan_into_index`](crate::scan::scan_into_index), composes
/// them here; a one-shot caller uses
/// [`prepare_roots_report`](crate::prepare_roots_report), which walks each root first.
///
/// # Errors
///
/// [`Error::InvalidRequest`](crate::Error::InvalidRequest) with
/// [`RequestError::RootMismatch`] when the indexes are not the roots' indexes, one for one
/// and in order; with [`RequestError::RootScopesDiffer`] when they hold different scan
/// scopes; and whatever [`report`] refuses of any of them.
pub fn report_roots(
    indexes: &[&Index],
    roots: &Roots,
    request: &Request,
    generated_at: std::time::SystemTime,
) -> crate::Result<Report> {
    let named = roots.as_slice();
    for position in 0..indexes.len().max(named.len()) {
        let held = indexes.get(position).map(|index| index.root_path().to_path_buf());
        let requested = named.get(position).map(|root| root.path.clone());
        if held != requested {
            return Err(crate::Error::InvalidRequest(RequestError::RootMismatch {
                held: held.unwrap_or_default(),
                requested: requested.unwrap_or_default(),
            }));
        }
    }
    // One report has one scope: its sizes, its `.gitignore` coverage, and what it says it
    // scanned are each one root's under it, so indexes opened under different scopes have
    // no one answer (review B4 on #192). The engine's own runs take one request's scope.
    if let Some(other) = indexes.iter().find(|index| index.scope() != indexes[0].scope()) {
        return Err(crate::Error::InvalidRequest(RequestError::RootScopesDiffer {
            first: indexes[0].root_path().to_path_buf(),
            other: other.root_path().to_path_buf(),
        }));
    }
    let labels = roots.is_several().then_some(roots);
    read_roots(indexes, labels, request, generated_at, NameIdentity::Native)
}

/// [`report_roots`] for the engine's own runs, which built each index from its root and
/// need not check that they belong together; `roots` is `None` for one root.
pub(crate) fn read_indexes(
    indexes: &[&Index],
    roots: Option<&Roots>,
    request: &Request,
    generated_at: std::time::SystemTime,
) -> crate::Result<Report> {
    read_roots(indexes, roots, request, generated_at, NameIdentity::Native)
}

/// One root's part of a read: its index, and everything the reader derives from that index
/// before a section combines it with any other root's.
///
/// The single-root reader held these as local variables of one function; holding them per
/// root is what lets every section be one accumulation per index and one finalization over
/// them all, so a report over one root is the report over several with one input.
struct RootRead<'a> {
    /// Position among the report's roots; 0 for a report over one root.
    root: usize,
    /// The root's index.
    index: &'a Index,
    /// The filtered traversal, when the selection needs one.
    walked: Option<Walked>,
    /// Subtree measurements, for the walk's directory predicates and, where
    /// [`Self::tree_measured`], a walked tree's completeness.
    directories: Option<BTreeMap<EntryId, query_subtrees::SubtreeValues>>,
    /// Whether a walked tree reads its completeness from [`Self::directories`].
    tree_measured: bool,
    /// An unfiltered tree's activity pass, over an index that may hold an unlisted subtree.
    activity: Option<query_subtrees::ActivityTable>,
    /// Whether any requested view is a tree, which is what reads recency.
    tree_views: bool,
    /// Every entry as a row, built once when more than one unfiltered view consumes rows.
    unfiltered_rows: Option<Vec<FileRow>>,
}

impl<'a> RootRead<'a> {
    /// Validate `request` against what `index` holds and derive what its sections read.
    fn new(
        root: usize,
        index: &'a Index,
        request: &Request,
        identity: NameIdentity,
    ) -> crate::Result<Self> {
        // What this index holds is what it can be read for. Every surface validates before
        // it scans, in the vocabulary its own caller uses; this is the library path, and the
        // last one, so nothing produces an answer from an unvalidated request.
        request.validate_read(&Basis::held_by(index)).map_err(crate::Error::InvalidRequest)?;

        let query = &request.query;
        let needs_walk = query.needs_selection_walk();
        let tree_views = query.views.iter().any(|view| query.tree_for(*view));
        // A tree row's age needs its newest activity and whether its subtree was listed in
        // full. Over a complete index with no scan depth every subtree is complete, so an
        // unfiltered tree reads each row's activity from the roll-up the index maintains and
        // takes no pass. Otherwise an unfiltered tree reads both from one pass over the
        // index by id, which also proves which children a partial tree may hide below the
        // share threshold. A walked tree folds activity in the walk and, where a subtree can
        // be unlisted, reads completeness from the subtree measurements it shares with the
        // selection predicates.
        let every_subtree_listed = query_subtrees::every_subtree_listed(index);
        let activity = (tree_views && !needs_walk && !every_subtree_listed)
            .then(|| query_subtrees::activity(index));
        let tree_measured = tree_views && needs_walk && !every_subtree_listed;
        let directories = (tree_measured
            || (needs_walk
                && (query.selection.kinds.is_empty()
                    || query.selection.kinds.contains(&EntryKind::Dir))))
        .then(|| query_subtrees::measure(index, &query.selection, identity));
        // One traversal serves every filtered view in the request.
        let walked =
            needs_walk.then(|| walk(index, &query.selection, identity, directories.as_ref()));
        // Unfiltered metric and file views share one `FileRow` walk only when more than one
        // section consumes it. A single section keeps ownership of its one traversal, so a
        // bounded file view does not clone every path before sorting and truncating it.
        // Summary, Tree, and Extensions keep roll-ups when unfiltered and do not consume
        // rows.
        let row_consumers =
            query.views.iter().copied().filter(|view| needs_unfiltered_entry_rows(*view)).count();
        let unfiltered_rows = (walked.is_none() && row_consumers > 1).then(|| every_entry(index));
        Ok(Self {
            root,
            index,
            walked,
            directories,
            tree_measured,
            activity,
            tree_views,
            unfiltered_rows,
        })
    }

    /// Where this root's tree reads each row's newest activity and completeness, when the
    /// request has a tree.
    fn recency(&self) -> Option<TreeRecency<'_>> {
        let measured = self.directories.as_ref().filter(|_| self.tree_measured);
        match (&self.activity, &self.walked) {
            (Some(table), _) => Some(TreeRecency::Unfiltered(table)),
            (None, Some(walked)) => Some(TreeRecency::Walked { walked, measured }),
            (None, None) => self.tree_views.then_some(TreeRecency::Maintained(self.index)),
        }
    }

    /// The root's own totals under the selection.
    fn summary(&self) -> SummaryRow {
        match &self.walked {
            None => unfiltered_summary(self.index, EntryId::ROOT, Path::new("")),
            Some(walked) => walked.summary_of(EntryId::ROOT),
        }
    }

    /// The rows a flat view lists from this root.
    fn entry_rows(&self) -> Cow<'_, [FileRow]> {
        entry_rows(self.index, self.walked.as_ref(), self.unfiltered_rows.as_deref())
    }

    /// The files a grouped view counts from this root: what the selection admitted or
    /// covers, or every entry when it filters nothing.
    fn members(&self) -> Cow<'_, [FileRow]> {
        self.walked
            .as_ref()
            .map_or_else(|| self.entry_rows(), |walked| Cow::Borrowed(walked.members.as_slice()))
    }

    /// The content records this root's index holds for the request's analyzers.
    fn content(&self, content: AnalysisSet) -> Option<crate::stored_state::ContentProjection<'a>> {
        let wanted = self.index.content_identity(content);
        self.index.content().and_then(|held| held.admit(&wanted))
    }
}

/// Refuse roots whose combined totals no `u64` can hold, as every route refuses one root's
/// ([`crate::Error::UnrepresentableTotal`]).
///
/// Each root's totals are representable, since its index refused anything else. Their sum
/// may not be, and every count, size, and share across roots is a part of it, so checking
/// it once here is what makes every sum below exact rather than wrapped or saturated.
fn combined_totals_fit(reads: &[RootRead<'_>]) -> crate::Result<()> {
    totals_fit(reads.iter().filter_map(|read| {
        let (all, _) = read.index.partition_scalars_of(EntryId::ROOT)?;
        Some((read.index.root_path(), [all.files, all.dirs, all.bytes, all.allocated]))
    }))
}

/// [`combined_totals_fit`] for the summary tier, whose roots kept a reduced row each rather
/// than an index; each row is its whole root, since that tier filters nothing.
pub(crate) fn summary_totals_fit(parts: &[SummaryPart]) -> crate::Result<()> {
    totals_fit(parts.iter().map(|part| {
        let row = part.summary;
        (part.root.as_path(), [row.files, row.dirs, row.bytes, row.allocated])
    }))
}

/// Refuse totals, each root's files, directories, bytes, and allocated bytes, whose sum
/// no `u64` holds, naming the root whose addition overflowed.
fn totals_fit<'a>(roots: impl Iterator<Item = (&'a Path, [u64; 4])>) -> crate::Result<()> {
    const COUNTERS: [&str; 4] = ["files", "directories", "bytes", "allocated bytes"];
    let mut sums = [0_u64; 4];
    for (root, values) in roots {
        for ((sum, value), counter) in sums.iter_mut().zip(values).zip(COUNTERS) {
            *sum = sum.checked_add(value).ok_or_else(|| crate::Error::UnrepresentableTotal {
                path: root.to_path_buf(),
                counter,
            })?;
        }
    }
    Ok(())
}

/// Read one report from the indexes of its roots, in the roots' order.
///
/// Every section accumulates over each root's [`RootRead`] and finalizes once, so display
/// bounds apply after the merge, against the combined total, and a report over one root is
/// this with one input.
///
/// `roots` names the roots of a report over several, one per index; `None` reads a report
/// over one root, which has no labels and keeps [`Report::root`].
fn read_roots(
    indexes: &[&Index],
    roots: Option<&Roots>,
    request: &Request,
    generated_at: std::time::SystemTime,
    identity: NameIdentity,
) -> crate::Result<Report> {
    debug_assert!(
        roots.map_or(indexes.len() == 1, |roots| roots.as_slice().len() == indexes.len()),
        "one index per root, and labels exactly when there are several"
    );
    let reads = indexes
        .iter()
        .enumerate()
        .map(|(root, index)| RootRead::new(root, index, request, identity))
        .collect::<crate::Result<Vec<_>>>()?;
    combined_totals_fit(&reads)?;
    let labels = roots.map(Roots::as_slice);
    let index = reads[0].index;
    debug_assert!(
        reads.iter().all(|read| read.index.scope() == index.scope()),
        "every root of one request is read under one scope"
    );

    let query = &request.query;
    let content = request.basis.content;
    let unfiltered = reads.iter().all(|read| read.walked.is_none());
    let metric_consumers =
        query.views.iter().copied().filter(|view| needs_metric_resolution(*view)).count();
    // Every view that needs metric resolution also needs unfiltered entry rows, so more
    // than one metric consumer means each root built the rows they share.
    let mut shared_metric_summaries = (unfiltered && metric_consumers > 1)
        .then(|| metric_summaries(&query.views, &reads, query, content));

    let mut sections: Vec<Section> = query
        .views
        .iter()
        .enumerate()
        .map(|(position, view)| {
            let shared_metric_summary =
                shared_metric_summaries.as_mut().and_then(|summaries| summaries[position].take());
            if let Some(summary) = shared_metric_summary {
                return Section::Metrics { view: *view, summary: Box::new(summary) };
            }
            build_section(*view, &reads, labels, query, content)
        })
        .collect();

    let age_reference_ns = crate::query::system_time_to_nanos(request.now);
    measure_ages(&mut sections, age_reference_ns);
    let ignore_rules =
        merge_ignore_rules(reads.iter().map(|read| read.index.control_coverage()).collect());
    let (mut notes, mut tips) = display_notes(query, content, &ignore_rules, labels);
    if content.includes_words() {
        // Of the files the report's views show, not of every record the index holds: a
        // selection that leaves a Markdown file out says nothing of it. Each metric view's
        // total counts the selection before its rows are bounded, and every view that
        // groups Markdown counts all of it, so the largest total is the count; a sum would
        // count one file once per view.
        let text_only = sections
            .iter()
            .filter_map(|section| match section {
                Section::Metrics { summary, .. } => summary
                    .total
                    .words_coverage
                    .as_ref()?
                    .get(&crate::content::CoverageReason::TextOnly)
                    .copied(),
                _ => None,
            })
            .max()
            .unwrap_or(0);
        if text_only > 0 {
            let files = if text_only == 1 { "file" } else { "files" };
            notes.push(format!(
                "note: {text_only} Markdown {files} over {} MiB counted as plain text: every \
                 word counted visible, paragraphs are blank-line runs",
                crate::content::MARKDOWN_EXACT_BYTES / (1024 * 1024)
            ));
        }
    }
    tips.extend(retained_refusals_tip(query, &ignore_rules));
    // Completeness rises to the root, so a root is incomplete exactly when some directory
    // below it is, and the report when some root is.
    if !query.min_share_for().admits(0, 1)
        && reads.iter().any(|read| {
            read.tree_views
                && read.recency().is_some_and(|recency| !recency.complete(EntryId::ROOT))
        })
    {
        notes.push("note: incomplete subtrees remain visible below the size threshold".to_owned());
    }
    if reads.iter().any(|read| {
        read.index.observes_controls()
            && !read.index.ignored_classification_complete_below(Path::new(""))
    }) {
        notes.push(UNVERIFIED_IGNORED_NOTE.to_owned());
    }
    Ok(Report {
        age_reference_ns,
        format: query.format,
        notes,
        tips,
        axes: query.axes,
        status: TreeStatus::merge(
            reads.iter().map(|read| TreeStatus::of(read.index, request)).collect(),
        ),
        provenance: ReportProvenance::merge(
            reads
                .iter()
                .map(|read| ReportProvenance::of(read.index, content, generated_at))
                .collect(),
        ),
        scope: index.scope(),
        requested_analysis: content,
        requested_views: query.views.clone(),
        omitted_views: query.omitted_views.clone(),
        root: roots.is_none().then(|| index.root_path().to_path_buf()),
        roots: roots.map(|roots| roots.as_slice().to_vec()),
        size: query.selection.size,
        sort_metric: match query.selection.sort {
            Some(SortKey::Metric(name)) => Some(name),
            _ => None,
        },
        analysis: merge_analysis(reads.iter().map(|read| {
            read.content(content).map(|projected| ContentReportMetadata {
                profile: projected.identity().analysis,
                provenance: projected.identity().record_provenance(),
            })
        })),
        ignored_entries: query.selection.ignored,
        ignore_rules,
        sections,
    })
}

/// The `.gitignore` coverage of several roots as one: counts add, and refusals keep root
/// order, each marked with its root, within the shared retention bound.
///
/// One request reads every root under one scope, so either all of them observed control
/// state or none did. Were that ever to change, a root that observed nothing would make
/// the whole report unobserved, since no rule read under the others says anything of it.
/// Over one root this is that root's coverage.
fn merge_ignore_rules(parts: Vec<ControlCoverage>) -> ControlCoverage {
    debug_assert!(
        parts.iter().all(|part| matches!(part, ControlCoverage::Observed(_)))
            || parts.iter().all(|part| matches!(part, ControlCoverage::NotObserved)),
        "every root of one request observes control state or none does"
    );
    let mut merged: Option<crate::control::ControlObservation> = None;
    for (root, part) in parts.into_iter().enumerate() {
        let ControlCoverage::Observed(mut observed) = part else {
            return ControlCoverage::NotObserved;
        };
        for refusal in &mut observed.refusals {
            refusal.root = root;
        }
        match &mut merged {
            None => merged = Some(observed),
            Some(total) => {
                total.applied += observed.applied;
                total.rules += observed.rules;
                total.refused += observed.refused;
                let room = crate::MAX_RETAINED_ISSUES.saturating_sub(total.refusals.len());
                total.refusals.extend(observed.refusals.into_iter().take(room));
            }
        }
    }
    merged.map_or(ControlCoverage::NotObserved, ControlCoverage::Observed)
}

/// The analyzer identity of several roots as one: each analyzer once, in first-seen order.
///
/// One request reads every root with one analyzer set and one set of type rules, so their
/// fingerprints agree; a root whose index holds no admissible records adds nothing, and
/// the report has none only when no root does.
fn merge_analysis(
    parts: impl Iterator<Item = Option<ContentReportMetadata>>,
) -> Option<ContentReportMetadata> {
    parts.flatten().reduce(|mut merged, part| {
        debug_assert_eq!(
            merged.provenance.type_rules_fingerprint, part.provenance.type_rules_fingerprint,
            "every root of one request is classified by one set of type rules"
        );
        merged.profile = merged.profile.union(part.profile);
        for analyzer in part.provenance.analyzers {
            if !merged.provenance.analyzers.iter().any(|(id, _)| *id == analyzer.0) {
                merged.provenance.analyzers.push(analyzer);
            }
        }
        merged
    })
}

/// Said of a report whose ignored subtotals a refused or unreadable control file withheld.
const UNVERIFIED_IGNORED_NOTE: &str =
    "note: gitignored subtotals are unavailable where governing rules could not be verified";

/// Point at the structured report when the refused-controls note could not name every
/// directory it counted.
fn retained_refusals_tip(query: &Query, ignore_rules: &ControlCoverage) -> Option<String> {
    matches!(ignore_rules, ControlCoverage::Observed(observed) if observed.refusals.len() > REFUSED_DIRECTORIES_NAMED)
        .then(|| format!("tip: show retained ignore-file details: {} json", query.axes.format))
}

/// What the summary tier keeps of one root's walk: the reduced row and everything the
/// report states about the walk that produced it.
pub(crate) struct SummaryPart {
    /// The canonical root.
    pub(crate) root: PathBuf,
    /// The scope the walk observed.
    pub(crate) scope: ScanScope,
    /// The root's totals.
    pub(crate) summary: SummaryRow,
    /// The control table's coverage.
    pub(crate) ignore_rules: ControlCoverage,
    /// Whether the row withholds its ignored share because a governing rule could not be
    /// verified.
    pub(crate) ignored_unverified: bool,
    /// The walk's completeness and failures.
    pub(crate) status: TreeStatus,
    /// When the walk began.
    pub(crate) scan_started: std::time::SystemTime,
    /// Whether the walk read everything in scope.
    pub(crate) complete: bool,
}

/// Build a one-section report from an already reduced exact summary.
///
/// Pure for the same reason as [`report`]: scanning and time sampling happened before
/// this boundary.  The execution planner uses this when a one-shot request proves that
/// retaining paths and hierarchy cannot affect its answer. Every field is the one
/// [`report`] derives for that request from an index of the same walk: `ignore_rules` is
/// the control table's coverage, and `ignored_unverified` says the row withholds its
/// ignored share, which [`report`] reads from the index as the root's classification
/// being incomplete. The notes are therefore the same notes, in the same order, less
/// those about content and trees this tier never answers.
///
/// Over several roots, `parts` holds each root's reduction in the roots' order and merges
/// as [`report_roots`] merges indexes: the rows sum, the coverage adds, and the status and
/// provenance take every root's.
pub(crate) fn report_summary(
    parts: Vec<SummaryPart>,
    roots: Option<&Roots>,
    request: &Request,
    generated_at: std::time::SystemTime,
) -> Report {
    let query = &request.query;
    let first = parts.first().expect("a report reads at least one root");
    let (root, scope) = (first.root.clone(), first.scope);
    let ignored_unverified = parts.iter().any(|part| part.ignored_unverified);
    let summary = sum_roots(parts.iter().map(|part| part.summary));
    let provenance = ReportProvenance::merge(
        parts
            .iter()
            .map(|part| ReportProvenance::of_walk(part.scan_started, generated_at, part.complete))
            .collect(),
    );
    let (ignore_rules, status): (Vec<_>, Vec<_>) =
        parts.into_iter().map(|part| (part.ignore_rules, part.status)).unzip();
    let ignore_rules = merge_ignore_rules(ignore_rules);
    let labels = roots.map(Roots::as_slice);
    let (mut notes, mut tips) = display_notes(query, request.basis.content, &ignore_rules, labels);
    tips.extend(retained_refusals_tip(query, &ignore_rules));
    if ignored_unverified {
        notes.push(UNVERIFIED_IGNORED_NOTE.to_owned());
    }
    Report {
        age_reference_ns: crate::query::system_time_to_nanos(request.now),
        format: query.format,
        notes,
        tips,
        axes: query.axes,
        status: TreeStatus::merge(status),
        provenance,
        scope,
        requested_analysis: AnalysisSet::NONE,
        requested_views: query.views.clone(),
        omitted_views: query.omitted_views.clone(),
        root: roots.is_none().then_some(root),
        roots: roots.map(|roots| roots.as_slice().to_vec()),
        size: query.selection.size,
        sort_metric: match query.selection.sort {
            Some(SortKey::Metric(name)) => Some(name),
            _ => None,
        },
        // The planner only selects this tier when no analysis was requested, so there is
        // no analyzer provenance to report.
        analysis: None,
        ignored_entries: query.selection.ignored,
        ignore_rules,
        sections: vec![Section::Summary(summary)],
    }
}

/// Aggregates gathered by one filtered traversal.
struct Walked {
    /// Whether the walked index observed control state, so its rows carry ignored shares.
    observed: bool,
    /// Filtered subtree aggregates, keyed by directory id.
    ///
    /// A row's `ignored` stays `None` until an ignored entry is admitted beneath it;
    /// [`Self::summary_of`] is what reads it as the index's observation says.
    per_directory: BTreeMap<EntryId, DirectoryTally>,
    /// Filtered per-extension tallies.
    by_ext: BTreeMap<String, ExtTally>,
    /// The ignored part of each filtered per-extension tally, for extensions that have one.
    ignored_by_ext: BTreeMap<String, ExtTally>,
    /// Entries the selection admitted.
    rows: Vec<FileRow>,
    /// Regular files in the union of matches and selected subtrees, counted once.
    members: Vec<FileRow>,
    /// Directories in that union or on a path to it, including empty matches.
    visible: BTreeSet<EntryId>,
    visible_files: BTreeSet<EntryId>,
    /// Selected contents whose ignore classification is unavailable, by ancestor directory.
    unknown_ignored: BTreeSet<EntryId>,
}

impl Walked {
    /// One directory's filtered totals, with an ignored share exactly when observed.
    fn summary_of(&self, id: EntryId) -> SummaryRow {
        let mut row = self.per_directory.get(&id).map(|tally| tally.summary).unwrap_or_default();
        row.ignored = (self.observed && !self.unknown_ignored.contains(&id))
            .then(|| row.ignored.unwrap_or_default());
        row
    }

    /// The newest modification time among the entries one directory's row counts, its own
    /// entry included when the selection admitted it ([`TreeNode::mtime_ns`]).
    fn activity_of(&self, id: EntryId) -> Option<i64> {
        self.per_directory.get(&id).and_then(|tally| tally.newest_activity_ns)
    }
}

/// Where a tree reads each directory row's newest activity and completeness.
#[derive(Clone, Copy)]
enum TreeRecency<'a> {
    /// An unfiltered tree over an index whose every subtree was listed
    /// ([`query_subtrees::every_subtree_listed`]): each row's activity is read from the
    /// roll-up the index maintains ([`query_subtrees::maintained_activity`]), and every
    /// row is complete.
    Maintained(&'a Index),
    /// An unfiltered tree over an index that may hold an unlisted subtree: one pass over
    /// the index by id ([`query_subtrees::activity`]).
    Unfiltered(&'a query_subtrees::ActivityTable),
    /// A walked tree: activity the walk folded over what the selection counts, and
    /// completeness from the subtree measurements when the index or its scope can leave
    /// a subtree unlisted. Without them every subtree is complete.
    Walked {
        walked: &'a Walked,
        measured: Option<&'a BTreeMap<EntryId, query_subtrees::SubtreeValues>>,
    },
}

impl TreeRecency<'_> {
    /// The newest activity directory `id`'s row counts ([`TreeNode::mtime_ns`]).
    fn activity(self, id: EntryId) -> Option<i64> {
        match self {
            Self::Maintained(index) => query_subtrees::maintained_activity(index, id),
            Self::Unfiltered(table) => table.get(id).and_then(|value| value.newest_ns),
            Self::Walked { walked, .. } => walked.activity_of(id),
        }
    }

    /// Whether directory `id`'s subtree was listed in full ([`TreeNode::complete`]).
    ///
    /// A directory the measurements do not hold was pruned by the selection and is never
    /// a row; reading it as incomplete keeps it out of any share proof.
    fn complete(self, id: EntryId) -> bool {
        match self {
            Self::Maintained(_) => true,
            Self::Unfiltered(table) => table.get(id).is_some_and(|value| value.complete),
            Self::Walked { measured, .. } => {
                measured.is_none_or(|values| values.get(&id).is_some_and(|value| value.complete))
            }
        }
    }
}

/// Measure every row age in `sections` from `reference_ns`, the report's age reference.
///
/// The one rule for list rows and tree nodes alike: a signed difference from the row's
/// newest counted modification, and no age at all when the row counts nothing, when the
/// reference cannot be represented, or when the subtree is incomplete. An incomplete
/// subtree's time is a lower bound, and a lower-bound maximum is not an age: the activity
/// that would make the directory younger may sit in the part that was never listed.
fn measure_ages(sections: &mut [Section], reference_ns: Option<i64>) {
    let age = |mtime_ns: Option<i64>, complete: Option<bool>| match complete {
        Some(false) => None,
        Some(true) | None => Some(i128::from(reference_ns?) - i128::from(mtime_ns?)),
    };
    for section in sections {
        match section {
            Section::Files { rows, .. } => {
                for row in rows {
                    row.age_ns = age(Some(row.mtime_ns), row.complete);
                }
            }
            Section::Tree { root, roots, .. } => {
                // Iterative, as every tree traversal here is: a deep tree must not
                // exhaust the stack.
                let mut stack: Vec<&mut TreeNode> =
                    root.iter_mut().map(|root| &mut **root).collect();
                if let Some(roots) = roots {
                    if let Some(total) = &mut roots.total {
                        total.age_ns = age(total.mtime_ns, Some(total.complete));
                    }
                    stack.extend(roots.trees.iter_mut().map(|tree| &mut tree.tree));
                }
                while let Some(node) = stack.pop() {
                    node.age_ns = age(node.mtime_ns, node.complete);
                    stack.extend(node.children.iter_mut());
                }
            }
            _ => {}
        }
    }
}

impl Report {
    /// Measure this report's ages from another reference instant, in epoch nanoseconds.
    ///
    /// Every age is a function of a row's modification time, its completeness, and the
    /// reference, so this gives exactly the ages the same read at that reference has. A
    /// watch session uses it to compare two repaints with the reference held fixed, so
    /// ages that only grew older do not count as a change.
    #[cfg(feature = "watch")]
    pub(crate) fn measure_ages_from(&mut self, reference_ns: Option<i64>) {
        self.age_reference_ns = reference_ns;
        measure_ages(&mut self.sections, reference_ns);
    }
}

/// One directory's filtered totals in the walk, beside the newest activity its tree row
/// counts.
///
/// Activity is not a [`SummaryRow`] field because a summary keeps its files-only recency
/// (`newest_mtime_ns`); it rides in the same map entry so the walk folds it with no
/// lookup of its own.
#[derive(Clone, Copy, Default)]
struct DirectoryTally {
    summary: SummaryRow,
    /// The newest modification time among the counted entries beneath the directory, of
    /// any kind, and of the directory itself once its post-order visit adds its own time
    /// when the selection counted it.
    newest_activity_ns: Option<i64>,
}

/// One directory's unfiltered totals from the roll-up state the index maintains, with its
/// ignored share, `all` less `unignored`, when the index observed control state.
fn unfiltered_summary(index: &Index, id: EntryId, path: &Path) -> SummaryRow {
    let observed = index.observes_controls() && index.ignored_classification_complete_below(path);
    let Some((all, unignored)) = index.partition_scalars_of(id) else {
        return SummaryRow {
            ignored: observed.then(IgnoredTally::default),
            ..SummaryRow::default()
        };
    };
    SummaryRow {
        ignored: observed.then(|| IgnoredTally::between(all, unignored)),
        ..summary_from_scalars(all)
    }
}

/// Walk the retained index once, aggregating only what the selection admits.
///
/// Iterative rather than recursive: this engine is built for trees deep enough that a
/// recursive post-order would exhaust the stack.
fn walk(
    index: &Index,
    selection: &Selection,
    identity: NameIdentity,
    directories: Option<&BTreeMap<EntryId, query_subtrees::SubtreeValues>>,
) -> Walked {
    debug_assert!(!index.is_folded(), "a folded index keeps too few files to be filtered");
    let observed = index.observes_controls();
    let mut walked = Walked {
        observed,
        per_directory: BTreeMap::new(),
        by_ext: BTreeMap::new(),
        ignored_by_ext: BTreeMap::new(),
        rows: Vec::new(),
        members: Vec::new(),
        visible: BTreeSet::new(),
        visible_files: BTreeSet::new(),
        unknown_ignored: BTreeSet::new(),
    };
    // No entry of an index that read no rule can be shown to be ignored or not, so
    // `report_in` refuses a selection by ignored state before it reaches this walk.
    debug_assert!(
        observed || selection.ignored == IgnoredEntries::Include,
        "a selection by ignored state over an unobserving index is refused before the walk"
    );

    // (id, path, post-order, covered by a selected ancestor, own time when counted)
    //
    // A directory's own time rides on its frame rather than being folded into its entry
    // when its parent counts it: the post-order visit adds it, with no lookup of its own.
    // The root's frame carries none, as the root is a traversal boundary.
    let mut stack = vec![(EntryId::ROOT, PathBuf::new(), false, false, None)];
    while let Some((id, path, expanded, covered, own_time)) = stack.pop() {
        if expanded {
            // Post-order: every child has finished, so fold their totals into this one.
            // `total` already carries this directory's own admitted files and admitted
            // directory children, both tallied in the pre-order pass below; what is left
            // is to add what each child subtree found deeper down.
            let mut total = walked.per_directory.remove(&id).unwrap_or_default();
            total.newest_activity_ns = newer(total.newest_activity_ns, own_time);
            if let Some(children) = index.children_of(id) {
                for (_, child) in children {
                    if let Some(sub) = walked.per_directory.get(&child) {
                        let sub = *sub;
                        merge_summary(&mut total.summary, &sub.summary);
                        total.newest_activity_ns =
                            newer(total.newest_activity_ns, sub.newest_activity_ns);
                        if walked.unknown_ignored.contains(&child) {
                            walked.unknown_ignored.insert(id);
                        }
                    }
                }
            }
            if total.summary.files > 0 || total.summary.dirs > 0 {
                walked.visible.insert(id);
            }
            walked.per_directory.insert(id, total);
            continue;
        }

        stack.push((id, path.clone(), true, covered, own_time));
        let Some(children) = index.children_of(id) else {
            continue;
        };
        let children: Vec<(PathBuf, EntryId)> =
            children.map(|(name, child)| (path.join(name), child)).collect();

        for (child_path, child) in children {
            let (Some(kind), Some(attrs)) = (index.kind_of(child), index.attrs_of(child)) else {
                continue;
            };
            // Bound once, and as an `OsStr`: the bucket has to be derived from the same
            // bytes the index interned from, or a name that is not valid UTF-8 would be
            // filed under one label by the fast tier and another by this one.
            let file_name = child_path.file_name().unwrap_or_default();
            let classification = index.ignored_classification_of(&child_path, child);
            let ignored = classification.unwrap_or(false);
            let classification_admitted =
                classification.is_some() || selection.ignored == IgnoredEntries::Include;
            let mut measured = *attrs;
            let subtree = directories.and_then(|values| values.get(&child)).copied();
            if let Some(subtree) = subtree {
                measured.size = subtree.bytes;
                measured.allocated = subtree.allocated;
                measured.mtime_ns = subtree.mtime_ns;
            }
            let (pruned, matches) = query_subtrees::with_candidate(
                &child_path,
                kind,
                measured,
                ignored,
                identity,
                |candidate| {
                    (query_subtrees::pruned(selection, &candidate), selection.admits(&candidate))
                },
            );
            if pruned {
                continue;
            }
            // An incomplete subtree's newest activity is a lower bound, not an age, so no
            // modification bound can be shown to hold for it: `before` could be disproved
            // by any unlisted descendant, and `since`, which a lower bound could prove, is
            // held to the same rule so that a row's presence under a time filter always
            // means the filter was decided on a complete measurement. A size bound still
            // matches, since a lower bound at or above the minimum proves the true size is.
            let matches = matches
                && classification_admitted
                && (selection.modified.is_unbounded()
                    || subtree.is_none_or(|subtree| subtree.complete));
            let row = FileRow {
                root: 0,
                path: child_path.clone(),
                kind,
                bytes: measured.size,
                allocated: measured.allocated,
                mtime_ns: measured.mtime_ns,
                files: subtree.map(|subtree| subtree.files),
                dirs: subtree.map(|subtree| subtree.dirs),
                complete: subtree.map(|subtree| subtree.complete),
                age_ns: None,
                ignored: classification,
                sort_value: None,
                classification: None,
            };
            if matches {
                walked.rows.push(row.clone());
            }
            let counted = matches
                || (covered && classification_admitted && selection.ignored.admits(ignored));
            if counted {
                if classification.is_none() {
                    walked.unknown_ignored.insert(id);
                }
                if kind == EntryKind::File {
                    walked.members.push(row);
                    walked.visible_files.insert(child);
                } else if kind == EntryKind::Dir {
                    walked.visible.insert(child);
                }

                let tally = walked.per_directory.entry(id).or_default();
                // Every counted entry is activity in its directory's row, whatever its
                // kind; a directory's own time is added by its own post-order visit.
                if kind != EntryKind::Dir {
                    tally.newest_activity_ns =
                        newer(tally.newest_activity_ns, Some(attrs.mtime_ns));
                }
                if kind == EntryKind::File {
                    let own = &mut tally.summary;
                    own.files += 1;
                    own.bytes += attrs.size;
                    own.allocated += attrs.allocated;
                    own.newest_mtime_ns = Some(
                        own.newest_mtime_ns.map_or(attrs.mtime_ns, |seen| seen.max(attrs.mtime_ns)),
                    );
                    let bucket = crate::classify::ext_bucket(file_name);
                    if ignored {
                        own.ignored.get_or_insert_with(IgnoredTally::default).add(IgnoredTally {
                            files: 1,
                            dirs: 0,
                            bytes: attrs.size,
                            allocated: attrs.allocated,
                        });
                        let tally = walked.ignored_by_ext.entry(bucket.clone()).or_default();
                        tally.files += 1;
                        tally.bytes += attrs.size;
                        tally.allocated += attrs.allocated;
                    }
                    let tally = walked.by_ext.entry(bucket).or_default();
                    tally.files += 1;
                    tally.bytes += attrs.size;
                    tally.allocated += attrs.allocated;
                } else if kind == EntryKind::Dir {
                    // Tallied here, beside the file case, rather than in the post-order
                    // fold: the fold sees every directory the walk descended into, and
                    // counting there reported directories the selection had rejected.
                    // `--kind file` answered "6 files, 3 directories", and a summary
                    // disagreed with the files view over the very same query.
                    let own = &mut tally.summary;
                    own.dirs += 1;
                    if ignored {
                        own.ignored.get_or_insert_with(IgnoredTally::default).dirs += 1;
                    }
                }
            }

            if kind == EntryKind::Dir {
                let own_time = counted.then_some(attrs.mtime_ns);
                stack.push((child, child_path, false, covered || matches, own_time));
            }
        }
    }

    walked
}

/// Fold one subtree's filtered totals into another's.
fn merge_summary(into: &mut SummaryRow, from: &SummaryRow) {
    into.files += from.files;
    into.dirs += from.dirs;
    into.bytes += from.bytes;
    into.allocated += from.allocated;
    into.newest_mtime_ns = match (into.newest_mtime_ns, from.newest_mtime_ns) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (left, right) => left.or(right),
    };
    if let Some(share) = from.ignored {
        into.ignored.get_or_insert_with(IgnoredTally::default).add(share);
    }
}

/// Views that reconstruct every path into a [`FileRow`] when the selection is unfiltered.
fn needs_unfiltered_entry_rows(view: ViewSpec) -> bool {
    matches!(
        view,
        ViewSpec::Types
            | ViewSpec::Families
            | ViewSpec::Languages
            | ViewSpec::Code
            | ViewSpec::Documents
            | ViewSpec::Files
            | ViewSpec::Largest
            | ViewSpec::Recent
    )
}

/// Content-grouping views that resolve the same current classification and analysis row.
fn needs_metric_resolution(view: ViewSpec) -> bool {
    matches!(view, ViewSpec::Types | ViewSpec::Families | ViewSpec::Languages | ViewSpec::Documents)
}

/// The entry rows a view aggregates: the filtered walk, a shared unfiltered walk, or a
/// fresh [`every_entry`] when this is the only consumer.
fn entry_rows<'a>(
    index: &Index,
    walked: Option<&'a Walked>,
    unfiltered_rows: Option<&'a [FileRow]>,
) -> Cow<'a, [FileRow]> {
    match (walked, unfiltered_rows) {
        (Some(walked), _) => Cow::Borrowed(&walked.rows),
        (None, Some(rows)) => Cow::Borrowed(rows),
        (None, None) => Cow::Owned(every_entry(index)),
    }
}

/// Build one view's section over every root, using the pre-computed tier when the
/// selection allows.
fn build_section(
    view: ViewSpec,
    reads: &[RootRead<'_>],
    labels: Option<&[NamedRoot]>,
    query: &Query,
    content: AnalysisSet,
) -> Section {
    if query.tree_for(view) {
        let limits = TreeDisplayLimits {
            depth: query.depth_for(view),
            min_share: query.min_share_for(),
            breadth: query.breadth_for(),
            rows: query.limit_for(view),
        };
        if let Some(labels) = labels {
            let (roots, omissions) = root_trees(reads, labels, query, content);
            return Section::Tree {
                view,
                limits,
                root: None,
                omissions,
                roots: Some(Box::new(roots)),
            };
        }
        let (root, omissions) = tree_section(reads, query, content);
        return Section::Tree { view, limits, root: root.map(Box::new), omissions, roots: None };
    }
    match view {
        ViewSpec::Code => Section::Code(Box::new(code_overview(reads, query, content))),
        ViewSpec::Summary => Section::Summary(sum_roots(reads.iter().map(RootRead::summary))),
        ViewSpec::Extensions => {
            let (rows, total, share_omitted) = extension_rows(reads, query);
            Section::Extensions { rows, total, share_omitted }
        }
        ViewSpec::Types | ViewSpec::Families | ViewSpec::Languages | ViewSpec::Documents => {
            Section::Metrics {
                view,
                summary: Box::new(metric_summary(view, reads, query, content)),
            }
        }
        ViewSpec::List
        | ViewSpec::Tree
        | ViewSpec::Files
        | ViewSpec::Largest
        | ViewSpec::Recent => {
            let (rows, total) = file_rows(view, reads, labels, query, content);
            Section::Files { view, rows, total }
        }
    }
}

/// The totals of several roots as one row: counts and sizes add and the newest file time
/// is the newest of any, as if one walk had counted them all.
///
/// The ignored share is unknown when any root's is, because a sum with an unknown term is
/// unknown. That is what makes this not [`merge_summary`], where an absent share means a
/// subtree that met no ignored entry yet and contributes nothing. Over one root it is that
/// root's row. The adds cannot overflow: [`read_roots`] has already refused roots whose
/// combined totals no `u64` holds, and every row here is part of those.
fn sum_roots(rows: impl IntoIterator<Item = SummaryRow>) -> SummaryRow {
    let mut rows = rows.into_iter();
    let mut total = rows.next().unwrap_or_default();
    for row in rows {
        total.files += row.files;
        total.dirs += row.dirs;
        total.bytes += row.bytes;
        total.allocated += row.allocated;
        total.newest_mtime_ns = newer(total.newest_mtime_ns, row.newest_mtime_ns);
        total.ignored = total.ignored.zip(row.ignored).map(|(mut sum, share)| {
            sum.add(share);
            sum
        });
    }
    total
}

/// A summary row taken straight from pre-computed roll-up state, before any ignored share.
fn summary_from_scalars(rollup: RollUpScalars) -> SummaryRow {
    SummaryRow {
        files: rollup.files,
        dirs: rollup.dirs,
        bytes: rollup.bytes,
        allocated: rollup.allocated,
        ignored: None,
        newest_mtime_ns: (rollup.files > 0).then_some(rollup.newest_mtime_ns),
    }
}

/// What each extension tally in `all` holds beyond the same extension in `unignored`.
fn ignored_by_extension(
    all: &BTreeMap<String, ExtTally>,
    unignored: &BTreeMap<String, ExtTally>,
) -> BTreeMap<String, ExtTally> {
    all.iter()
        .filter_map(|(extension, tally)| {
            let kept = unignored.get(extension).copied().unwrap_or_default();
            let ignored = ExtTally {
                files: tally.files.saturating_sub(kept.files),
                bytes: tally.bytes.saturating_sub(kept.bytes),
                allocated: tally.allocated.saturating_sub(kept.allocated),
            };
            (ignored.files > 0).then(|| (extension.clone(), ignored))
        })
        .collect()
}

/// One root's per-extension tallies, before shares, sorting, and bounds.
struct ExtensionTallies {
    /// Every selected file, by extension.
    all: BTreeMap<String, ExtTally>,
    /// The ignored part of each, for extensions that have one.
    ignored: BTreeMap<String, ExtTally>,
    /// Whether the ignored parts are known, so rows carry an ignored share.
    observed: bool,
}

impl ExtensionTallies {
    /// What one root's selection counts, by extension.
    fn of(read: &RootRead<'_>) -> Self {
        let index = read.index;
        debug_assert!(!index.is_folded(), "a folded index keeps no extension tallies (H176)");
        // Extension partitions cannot attribute an unknown member to one bucket from the
        // roll-up alone, so withhold their ignored subtotals until the scope is known.
        let observed = match &read.walked {
            Some(walked) => walked.observed && !walked.unknown_ignored.contains(&EntryId::ROOT),
            None => {
                index.observes_controls()
                    && index.ignored_classification_complete_below(Path::new(""))
            }
        };
        let (all, ignored) = match &read.walked {
            None => match index.partition_total() {
                Ok(partitions) => {
                    let ignored =
                        ignored_by_extension(&partitions.all.by_ext, &partitions.unignored.by_ext);
                    (partitions.all.by_ext, ignored)
                }
                // An index that observed no control state has no unignored partition to
                // subtract, and its rows carry no ignored share.
                Err(_not_observed) => (index.total().by_ext, BTreeMap::new()),
            },
            Some(walked) => (walked.by_ext.clone(), walked.ignored_by_ext.clone()),
        };
        Self { all, ignored, observed }
    }

    /// Add another root's tallies: buckets add, and the ignored parts stay known only if
    /// they are known for both.
    fn absorb(&mut self, other: Self) {
        let add = |into: &mut BTreeMap<String, ExtTally>, from: BTreeMap<String, ExtTally>| {
            for (extension, tally) in from {
                let sum = into.entry(extension).or_default();
                sum.files += tally.files;
                sum.bytes += tally.bytes;
                sum.allocated += tally.allocated;
            }
        };
        add(&mut self.all, other.all);
        add(&mut self.ignored, other.ignored);
        self.observed &= other.observed;
    }
}

/// Rows for the types view, over every root.
fn extension_rows(reads: &[RootRead<'_>], query: &Query) -> (Vec<TypeRow>, usize, usize) {
    let mut tallies = reads.iter().map(ExtensionTallies::of);
    let mut merged = tallies.next().expect("a report reads at least one root");
    for other in tallies {
        merged.absorb(other);
    }
    let ExtensionTallies { all: tallies, ignored, observed } = merged;

    let mut rows: Vec<TypeRow> = tallies
        .into_iter()
        .map(|(extension, tally)| {
            let share = ignored.get(&extension).copied().unwrap_or_default();
            TypeRow {
                files: tally.files,
                bytes: tally.bytes,
                allocated: tally.allocated,
                ignored: observed.then_some(IgnoredTally {
                    files: share.files,
                    dirs: 0,
                    bytes: share.bytes,
                    allocated: share.allocated,
                }),
                extension,
            }
        })
        .collect();

    let before_share = rows.len();
    if let Some(threshold) = &query.selection.min_share {
        let root = sum_roots(reads.iter().map(RootRead::summary));
        let denominator = match query.selection.size {
            SizeMetric::Apparent => root.bytes,
            SizeMetric::Allocated => root.allocated,
        };
        rows.retain(|row| {
            threshold.admits(
                match query.selection.size {
                    SizeMetric::Apparent => row.bytes,
                    SizeMetric::Allocated => row.allocated,
                },
                denominator,
            )
        });
    }

    sort_rows(
        &mut rows,
        query,
        ViewSpec::Extensions,
        SortAccessors {
            size: |row: &TypeRow, metric| match metric {
                SizeMetric::Apparent => row.bytes,
                SizeMetric::Allocated => row.allocated,
            },
            count: |row: &TypeRow| row.files,
            mtime: |_: &TypeRow| None,
            name: borrowed_name(|row: &TypeRow| std::borrow::Cow::Borrowed(row.extension.as_str())),
            content_metric: |_: &TypeRow, _: &MetricDef| None,
            rank: |_: &TypeRow| 0,
        },
    );
    let total = truncate(&mut rows, query.limit_for(ViewSpec::Extensions));
    (rows, total, before_share - total)
}

/// One grouped section over every root: each root's files accumulate into one set of
/// buckets, which finish once.
///
/// One accumulator over every root's files is exactly the merge of one per root, since a
/// bucket's counts add and its pooled word statistics merge as sufficient statistics
/// ([`LogicalWordStats::add_assign`]); shares and bounds are taken only at the finish.
fn metric_summary(
    view: ViewSpec,
    reads: &[RootRead<'_>],
    query: &Query,
    content: AnalysisSet,
) -> MetricSummary {
    let mut accumulator = MetricAccumulator::new(view);
    for read in reads {
        let held = read.content(content);
        for file in read.members().iter().filter(|row| row.kind == EntryKind::File) {
            let cached = held.and_then(|content| content.file(&file.path));
            let classification = read.index.classify(&file.path);
            accumulator.push(content, file, cached, &classification);
        }
    }
    accumulator.finish(query, content)
}

/// Build several unfiltered metric sections in one file pass over each root.
fn metric_summaries(
    views: &[ViewSpec],
    reads: &[RootRead<'_>],
    query: &Query,
    content: AnalysisSet,
) -> Vec<Option<MetricSummary>> {
    let mut accumulators = views
        .iter()
        .copied()
        .filter(|view| needs_metric_resolution(*view))
        .map(MetricAccumulator::new)
        .collect::<Vec<_>>();
    for read in reads {
        let rows = read
            .unfiltered_rows
            .as_deref()
            .expect("multiple metric views share their unfiltered entry rows");
        let held = read.content(content);
        for file in rows.iter().filter(|row| row.kind == EntryKind::File) {
            let cached = held.and_then(|content| content.file(&file.path));
            let classification = read.index.classify(&file.path);
            for accumulator in &mut accumulators {
                accumulator.push(content, file, cached, &classification);
            }
        }
    }

    let mut finished =
        accumulators.into_iter().map(|accumulator| accumulator.finish(query, content));
    let summaries = views
        .iter()
        .map(|view| needs_metric_resolution(*view).then(|| finished.next().expect("metric view")))
        .collect();
    debug_assert!(finished.next().is_none());
    summaries
}

struct MetricAccumulator {
    view: ViewSpec,
    group: MetricGroup,
    grouped: BTreeMap<String, MetricRow>,
}

impl MetricAccumulator {
    fn new(view: ViewSpec) -> Self {
        debug_assert!(needs_metric_resolution(view));
        let group =
            if view == ViewSpec::Families { MetricGroup::Family } else { MetricGroup::Type };
        Self { view, group, grouped: BTreeMap::new() }
    }

    fn push(
        &mut self,
        content: AnalysisSet,
        file: &FileRow,
        cached: Option<&FileAnalysis>,
        classification: &Classification,
    ) {
        let included = match self.view {
            ViewSpec::Languages => classification.family == ContentFamily::Code,
            ViewSpec::Documents => {
                matches!(classification.family, ContentFamily::Prose | ContentFamily::Markup)
            }
            ViewSpec::Types | ViewSpec::Families => true,
            ViewSpec::List
            | ViewSpec::Tree
            | ViewSpec::Extensions
            | ViewSpec::Code
            | ViewSpec::Files
            | ViewSpec::Largest
            | ViewSpec::Recent
            | ViewSpec::Summary => false,
        };
        if !included {
            return;
        }
        let id = match self.group {
            MetricGroup::Type => classification.file_type.as_str().to_string(),
            MetricGroup::Family => classification.family.as_str().to_string(),
        };
        let row = self.grouped.entry(id.clone()).or_insert_with(|| MetricRow {
            analysis: content,
            id,
            family: classification.family,
            files: 0,
            bytes: 0,
            allocated: 0,
            analyzed_files: 0,
            metrics: ReportMetricValues::for_analysis(content),
            logical_word_stats: LogicalWordStats::default(),
            visible_logical_word_stats: LogicalWordStats::default(),
            document_raw_words: 0,
            document_word_stats: LogicalWordStats::default(),
            document_metric_files: 0,
            coverage: BTreeMap::new(),
            lines_coverage: BTreeMap::new(),
            code_coverage: content.includes_code().then(BTreeMap::new),
            words_coverage: content.includes_words().then(BTreeMap::new),
            detection_sources: BTreeMap::new(),
            detection_confidence: BTreeMap::new(),
            generated_files: 0,
            vendored_files: 0,
            documentation_files: 0,
            share: MetricShare::default(),
        });
        row.files = row.files.saturating_add(1);
        row.bytes = row.bytes.saturating_add(file.bytes);
        row.allocated = row.allocated.saturating_add(file.allocated);
        let detection = cached.map_or(
            (classification.source, classification.confidence, classification.flags),
            |record| (record.detection.source, record.detection.confidence, record.detection.flags),
        );
        *row.detection_sources.entry(detection.0).or_default() += 1;
        *row.detection_confidence.entry(detection.1).or_default() += 1;
        row.generated_files = row.generated_files.saturating_add(u64::from(detection.2.generated));
        row.vendored_files = row.vendored_files.saturating_add(u64::from(detection.2.vendored));
        row.documentation_files =
            row.documentation_files.saturating_add(u64::from(detection.2.documentation));
        if let Some(record) = cached {
            *row.lines_coverage.entry(record.lines.coverage()).or_default() += 1;
            if let (Some(coverage), Some(outcome)) = (&mut row.code_coverage, record.code) {
                *coverage.entry(outcome.coverage()).or_default() += 1;
            }
            if let (Some(coverage), Some(outcome)) = (&mut row.words_coverage, record.words) {
                *coverage.entry(outcome.coverage()).or_default() += 1;
            }
            let selected = match self.view {
                ViewSpec::Languages if content.includes_code() => {
                    record.code.map(|outcome| outcome.coverage())
                }
                ViewSpec::Documents if content.includes_words() => {
                    record.words.map(|outcome| outcome.coverage())
                }
                _ => None,
            }
            .unwrap_or(record.lines.coverage());
            *row.coverage.entry(selected).or_default() += 1;
            if selected == CoverageReason::Analyzed {
                row.analyzed_files = row.analyzed_files.saturating_add(1);
            }
            if let Some(lines) = record.lines.value() {
                add_optional(&mut row.metrics.physical_lines, Some(lines.physical_lines));
                add_optional(&mut row.metrics.blank_lines, Some(lines.blank_lines));
                add_optional(&mut row.metrics.nonblank_lines, Some(lines.nonblank_lines));
                add_optional(&mut row.metrics.raw_words, Some(lines.raw_words));
                row.document_raw_words = row.document_raw_words.saturating_add(lines.raw_words);
            }
            if let Some(code_metrics) = record.code.and_then(crate::content::AnalyzerOutcome::value)
            {
                add_optional(&mut row.metrics.code_lines, Some(code_metrics.code_lines));
                add_optional(&mut row.metrics.comment_lines, Some(code_metrics.comment_lines));
                add_optional(
                    &mut row.metrics.code_blank_lines,
                    Some(code_metrics.code_blank_lines),
                );
            }
            if let Some(words) = record.words.and_then(crate::content::AnalyzerOutcome::value) {
                add_optional(&mut row.metrics.paragraphs, Some(words.paragraphs));
                add_optional(&mut row.metrics.visible_words, Some(words.visible_words));
                row.logical_word_stats.add_assign(words.logical_word_stats);
                row.visible_logical_word_stats.add_assign(words.visible_logical_word_stats);
                row.document_metric_files = row.document_metric_files.saturating_add(1);
                if classification.file_type.as_str() == "markdown" {
                    row.document_raw_words = row
                        .document_raw_words
                        .saturating_sub(record.lines.value().map_or(0, |lines| lines.raw_words));
                    row.document_raw_words =
                        row.document_raw_words.saturating_add(words.visible_words);
                    row.document_word_stats.add_assign(words.visible_logical_word_stats);
                } else {
                    row.document_word_stats.add_assign(words.logical_word_stats);
                }
            }
        }
    }

    fn finish(self, query: &Query, content: AnalysisSet) -> MetricSummary {
        let Self { view, group, mut grouped } = self;

        for row in grouped.values_mut() {
            row.finish_derived_metrics();
        }

        let mut total = MetricRow {
            analysis: content,
            id: "total".to_string(),
            family: ContentFamily::Unknown,
            files: 0,
            bytes: 0,
            allocated: 0,
            analyzed_files: 0,
            metrics: ReportMetricValues::for_analysis(content),
            logical_word_stats: LogicalWordStats::default(),
            visible_logical_word_stats: LogicalWordStats::default(),
            document_raw_words: 0,
            document_word_stats: LogicalWordStats::default(),
            document_metric_files: 0,
            coverage: BTreeMap::new(),
            lines_coverage: BTreeMap::new(),
            code_coverage: content.includes_code().then(BTreeMap::new),
            words_coverage: content.includes_words().then(BTreeMap::new),
            detection_sources: BTreeMap::new(),
            detection_confidence: BTreeMap::new(),
            generated_files: 0,
            vendored_files: 0,
            documentation_files: 0,
            share: MetricShare::default(),
        };
        for row in grouped.values() {
            total.files = total.files.saturating_add(row.files);
            total.bytes = total.bytes.saturating_add(row.bytes);
            total.allocated = total.allocated.saturating_add(row.allocated);
            total.analyzed_files = total.analyzed_files.saturating_add(row.analyzed_files);
            total.metrics.add_assign(&row.metrics);
            total.logical_word_stats.add_assign(row.logical_word_stats);
            total.visible_logical_word_stats.add_assign(row.visible_logical_word_stats);
            total.document_raw_words =
                total.document_raw_words.saturating_add(row.document_raw_words);
            total.document_word_stats.add_assign(row.document_word_stats);
            total.document_metric_files =
                total.document_metric_files.saturating_add(row.document_metric_files);
            for (reason, count) in &row.coverage {
                *total.coverage.entry(*reason).or_default() += count;
            }
            merge_coverage(&mut total.lines_coverage, &row.lines_coverage);
            if let (Some(total), Some(row)) = (&mut total.code_coverage, &row.code_coverage) {
                merge_coverage(total, row);
            }
            if let (Some(total), Some(row)) = (&mut total.words_coverage, &row.words_coverage) {
                merge_coverage(total, row);
            }
            for (source, count) in &row.detection_sources {
                *total.detection_sources.entry(*source).or_default() += count;
            }
            for (confidence, count) in &row.detection_confidence {
                *total.detection_confidence.entry(*confidence).or_default() += count;
            }
            total.generated_files = total.generated_files.saturating_add(row.generated_files);
            total.vendored_files = total.vendored_files.saturating_add(row.vendored_files);
            total.documentation_files =
                total.documentation_files.saturating_add(row.documentation_files);
        }
        total.finish_derived_metrics();
        let byte_share_metric = match query.selection.size {
            SizeMetric::Apparent => ShareMetric::ApparentBytes,
            SizeMetric::Allocated => ShareMetric::AllocatedBytes,
        };
        let share_metric = match view {
            // The requested analyzers, not the stored ones: a share is a fact about what the
            // request asked to measure, and `validate_read` proved the index holds exactly it.
            ViewSpec::Languages if content.includes_code() => ShareMetric::CodeLines,
            ViewSpec::Documents if content.includes_words() => ShareMetric::DocumentWords,
            ViewSpec::Languages | ViewSpec::Documents if content.is_enabled() => {
                ShareMetric::RawWords
            }
            ViewSpec::Languages | ViewSpec::Documents | ViewSpec::Types | ViewSpec::Families => {
                byte_share_metric
            }
            ViewSpec::List
            | ViewSpec::Tree
            | ViewSpec::Extensions
            | ViewSpec::Code
            | ViewSpec::Files
            | ViewSpec::Largest
            | ViewSpec::Recent
            | ViewSpec::Summary => {
                unreachable!("only grouped views reach metric_summary")
            }
        };
        let mut rows = grouped.into_values().collect::<Vec<_>>();
        let denominator = share_denominator(&rows, share_metric);
        total.share = MetricShare { numerator: denominator, denominator };
        for row in &mut rows {
            row.share = MetricShare { numerator: share_value(row, share_metric), denominator };
        }
        sort_rows(
            &mut rows,
            query,
            view,
            SortAccessors {
                size: |row: &MetricRow, metric| match view {
                    ViewSpec::Languages | ViewSpec::Documents => share_value(row, share_metric),
                    _ => match metric {
                        SizeMetric::Apparent => row.bytes,
                        SizeMetric::Allocated => row.allocated,
                    },
                },
                count: |row: &MetricRow| row.files,
                mtime: |_: &MetricRow| None,
                name: borrowed_name(|row: &MetricRow| std::borrow::Cow::Borrowed(row.id.as_str())),
                content_metric: MetricRow::metric_value,
                rank: |_: &MetricRow| 0,
            },
        );
        let before_share = rows.len();
        if let Some(threshold) = &query.selection.min_share {
            rows.retain(|row| threshold.admits(row.share.numerator, row.share.denominator));
        }
        let share_omitted = before_share - rows.len();
        let total_rows = truncate(&mut rows, query.limit_for(view));
        MetricSummary {
            group,
            total,
            rows,
            total_rows,
            share_omitted,
            share_metric,
            words_per_page: query.words_per_page.max(1),
        }
    }
}

/// The code overview over every root: each root's source files tally into one set of
/// language rows, whose shares and bounds are taken once.
fn code_overview(reads: &[RootRead<'_>], query: &Query, content: AnalysisSet) -> CodeOverview {
    #[derive(Default)]
    struct SortFacts {
        apparent: u64,
        allocated: u64,
        newest_mtime_ns: Option<i64>,
        metric: MetricAggregate,
    }

    // The roots share one scope, so either every index observed control state or none
    // did; requiring all of them keeps the split honest if that ever changed.
    let split = query.selection.ignored == IgnoredEntries::Include
        && reads.iter().all(|read| read.index.observes_controls());
    let sort_key = query.selection.sort.unwrap_or(SortKey::Metric("code_lines"));
    let sort_metric = match sort_key {
        SortKey::Metric(name) => Some(name),
        _ => None,
    };
    let mut overview = CodeOverview {
        population: query.selection.ignored,
        selected: CodeTally::default(),
        non_ignored: split.then(CodeTally::default),
        ignored: split.then(CodeTally::default),
        unknown: CodeTally::default(),
        unclassified_files: 0,
        analyzed_languages: 0,
        total_languages: 0,
        share_omitted: 0,
        languages: Vec::new(),
        share_metric: ShareMetric::CodeLines,
    };
    let mut grouped = BTreeMap::<String, CodeLanguageRow>::new();
    let mut sort_facts = BTreeMap::<String, SortFacts>::new();
    for read in reads {
        let (index, held, files) = (read.index, read.content(content), read.members());
        for file in files.iter().filter(|row| row.kind == EntryKind::File) {
            let record = held.and_then(|tier| tier.file(&file.path));
            let classification = record.map_or_else(
                || index.classify(&file.path).into(),
                |record| record.detection.clone(),
            );
            if classification.family == ContentFamily::Unknown {
                overview.unclassified_files = overview.unclassified_files.saturating_add(1);
            }
            if classification.family != ContentFamily::Code {
                continue;
            }
            let language = classification.file_type.as_str().to_string();
            let facts = sort_facts.entry(language.clone()).or_default();
            facts.apparent = facts.apparent.saturating_add(file.bytes);
            facts.allocated = facts.allocated.saturating_add(file.allocated);
            facts.newest_mtime_ns =
                Some(facts.newest_mtime_ns.map_or(file.mtime_ns, |old| old.max(file.mtime_ns)));
            if let (Some(name), Some(record)) = (sort_metric, record) {
                if let Some(value) =
                    measured_file_metric(record, classification.file_type.as_str(), name)
                {
                    facts.metric.add(value);
                }
            }
            let row = grouped.entry(language.clone()).or_insert_with(|| CodeLanguageRow {
                language,
                selected: CodeTally::default(),
                non_ignored: split.then(CodeTally::default),
                ignored: split.then(CodeTally::default),
                unknown: CodeTally::default(),
                share: MetricShare::default(),
            });
            row.selected.add_file(record);
            overview.selected.add_file(record);
            match index.ignored_classification(&file.path) {
                Some(false) if split => {
                    row.non_ignored.as_mut().expect("split initialized").add_file(record);
                    overview.non_ignored.as_mut().expect("split initialized").add_file(record);
                }
                Some(true) if split => {
                    row.ignored.as_mut().expect("split initialized").add_file(record);
                    overview.ignored.as_mut().expect("split initialized").add_file(record);
                }
                None => {
                    row.unknown.add_file(record);
                    overview.unknown.add_file(record);
                }
                Some(_) => {}
            }
        }
    }
    let denominator = overview.selected.metrics.code_lines;
    overview.analyzed_languages =
        grouped.values().filter(|row| row.selected.analyzed_files > 0).count() as u64;
    overview.languages = grouped.into_values().collect();
    for row in &mut overview.languages {
        row.share = MetricShare { numerator: row.selected.metrics.code_lines, denominator };
    }
    let normalized =
        matches!(sort_metric, Some("logical_words" | "visible_logical_words" | "document_words"));
    overview.languages.sort_by(|left, right| {
        let left_facts = &sort_facts[&left.language];
        let right_facts = &sort_facts[&right.language];
        let ordering = match sort_key {
            SortKey::Size => {
                let value = |facts: &SortFacts| match query.selection.size {
                    SizeMetric::Apparent => facts.apparent,
                    SizeMetric::Allocated => facts.allocated,
                };
                value(right_facts).cmp(&value(left_facts))
            }
            SortKey::Count => right.selected.source_files.cmp(&left.selected.source_files),
            SortKey::Mtime => right_facts.newest_mtime_ns.cmp(&left_facts.newest_mtime_ns),
            SortKey::Name => left.language.cmp(&right.language),
            SortKey::Metric(_) => {
                let left_value = left_facts.metric.value(normalized);
                let right_value = right_facts.metric.value(normalized);
                match (left_value, right_value) {
                    (Some(left), Some(right)) if query.selection.reverse => left.cmp(&right),
                    (Some(left), Some(right)) => right.cmp(&left),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                }
            }
        };
        let ordering = if query.selection.reverse && !matches!(sort_key, SortKey::Metric(_)) {
            ordering.reverse()
        } else {
            ordering
        };
        ordering.then_with(|| left.language.cmp(&right.language))
    });
    let before_share = overview.languages.len();
    if let Some(threshold) = &query.selection.min_share {
        overview.languages.retain(|row| threshold.admits(row.share.numerator, denominator));
    }
    overview.share_omitted = before_share - overview.languages.len();
    overview.total_languages = overview.languages.len();
    if let Some(limit) = query.limit_for(ViewSpec::Code).limit() {
        overview.languages.truncate(limit);
    }
    overview
}

fn merge_coverage(total: &mut BTreeMap<CoverageReason, u64>, row: &BTreeMap<CoverageReason, u64>) {
    for (reason, count) in row {
        *total.entry(*reason).or_default() += count;
    }
}

/// The denominator every row of a grouped section shares: the sum of all its rows'
/// numerators, taken before the share filter and the row bound.
///
/// It is the rows' sum rather than the total row's own value because the two differ for
/// document words. Logical words are derived from pooled statistics
/// ([`LogicalWordStats::logical_words`]), whose regime the pool decides, so a section
/// mixing formats has a total that differs from the sum of its rows, in either direction.
/// Dividing by that total would make the rows' shares sum past 100% beside long-token
/// formats and fall short of it beside very short tokens (fdu-ij5n). For bytes, code
/// lines, and raw words the two are equal, so this rule changes nothing there.
fn share_denominator(rows: &[MetricRow], metric: ShareMetric) -> u64 {
    rows.iter().fold(0_u64, |sum, row| sum.saturating_add(share_value(row, metric)))
}

fn share_value(row: &MetricRow, metric: ShareMetric) -> u64 {
    match metric {
        ShareMetric::ApparentBytes => row.bytes,
        ShareMetric::AllocatedBytes => row.allocated,
        ShareMetric::CodeLines => row.metrics.code_lines.unwrap_or(0),
        ShareMetric::DocumentWords => document_words(row).unwrap_or(0),
        ShareMetric::RawWords => row.metrics.raw_words.unwrap_or(0),
    }
}

/// Derive the selected document volume only after every sufficient statistic is added.
pub fn document_words(row: &MetricRow) -> Option<u64> {
    row.metrics.document_words
}

/// Derived page inputs, absent unless the words unit was requested.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pages {
    /// Query-selected document words.
    pub words: u64,
    /// Words represented by one page.
    pub words_per_page: u64,
}

/// Build page inputs only when document words were measured.
pub fn pages(row: &MetricRow, words_per_page: u64) -> Option<Pages> {
    document_words(row).map(|words| Pages { words, words_per_page: words_per_page.max(1) })
}

/// Rows for a flat view, over every root.
fn file_rows(
    view: ViewSpec,
    reads: &[RootRead<'_>],
    labels: Option<&[NamedRoot]>,
    query: &Query,
    content: AnalysisSet,
) -> (Vec<FileRow>, usize) {
    let ranks = labels.map(label_ranks);
    let ranks = ranks.as_deref();
    if let [read] = reads {
        return root_file_rows(view, read, ranks, query, content);
    }
    // Each root's rows are bounded on their own first. That is exact for a sorted top-k:
    // the comparator orders a root's rows among themselves as it orders them alone, since
    // the root's rank ties within it, so no row the merged bound keeps can have been cut
    // by its own root's.
    let mut rows = Vec::new();
    let mut total = 0;
    for read in reads {
        let (part, count) = root_file_rows(view, read, ranks, query, content);
        rows.extend(part);
        total += count;
    }
    sort_file_rows(&mut rows, query, view, ranks);
    truncate(&mut rows, query.limit_for(view));
    (rows, total)
}

/// Each root's rank in label order, by position: where its rows fall among the roots' when
/// rows are otherwise equal or ordered by name.
///
/// Text prints a path after its root's label, so ordering by label and then by path is what
/// reads as sorted; ordering by the relative path alone would interleave the roots. Labels
/// that compare equal keep the caller's order. A report over one root has no labels and
/// takes no ranks ([`sort_file_rows`]).
fn label_ranks(labels: &[NamedRoot]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..labels.len()).collect();
    order.sort_by(|left, right| {
        labels[*left].label.to_string_lossy().cmp(&labels[*right].label.to_string_lossy())
    });
    let mut ranks = vec![0; labels.len()];
    for (rank, root) in order.into_iter().enumerate() {
        ranks[root] = rank;
    }
    ranks
}

/// Order flat rows by the view's key, ranking rows of different roots by label where names
/// decide ([`label_ranks`]).
///
/// One root, with no `ranks`, takes a constant rank: its own instance of the comparator,
/// which is the one-root comparator with no lookup per comparison (review C8 on #192).
fn sort_file_rows(rows: &mut [FileRow], query: &Query, view: ViewSpec, ranks: Option<&[usize]>) {
    match ranks {
        None => sort_file_rows_ranked(rows, query, view, |_: &FileRow| 0),
        Some(ranks) => sort_file_rows_ranked(rows, query, view, |row: &FileRow| ranks[row.root]),
    }
}

fn sort_file_rows_ranked(
    rows: &mut [FileRow],
    query: &Query,
    view: ViewSpec,
    rank: impl Fn(&FileRow) -> usize,
) {
    sort_rows(
        rows,
        query,
        view,
        SortAccessors {
            size: |row: &FileRow, metric| match metric {
                SizeMetric::Apparent => row.bytes,
                SizeMetric::Allocated => row.allocated,
            },
            count: |row: &FileRow| row.files.unwrap_or(1),
            mtime: |row: &FileRow| Some(row.mtime_ns),
            name: borrowed_name(|row: &FileRow| row.path.to_string_lossy()),
            content_metric: |row: &FileRow, _: &MetricDef| row.sort_value,
            rank,
        },
    );
}

/// One root's rows for a flat view: classified, sorted, and bounded on their own.
fn root_file_rows(
    view: ViewSpec,
    read: &RootRead<'_>,
    ranks: Option<&[usize]>,
    query: &Query,
    content: AnalysisSet,
) -> (Vec<FileRow>, usize) {
    let (index, walked) = (read.index, read.walked.as_ref());
    let mut rows = read.entry_rows().into_owned();
    if view.files_only() {
        rows.retain(|row| row.kind == EntryKind::File);
    }
    // Every row is built with root 0, which is already right for the first root.
    if read.root != 0 {
        for row in &mut rows {
            row.root = read.root;
        }
    }
    if let Some(SortKey::Metric(name)) = query.selection.sort {
        let sources = walked.map_or(rows.as_slice(), |walked| walked.members.as_slice());
        let values = metric_sort_values(index, content, sources, name);
        for row in &mut rows {
            row.sort_value = values.get(&row.path).copied();
        }
    }

    sort_file_rows(&mut rows, query, view, ranks);
    let total = truncate(&mut rows, query.limit_for(view));
    let held = index.content().and_then(|tier| tier.admit(&index.content_identity(content)));
    for row in &mut rows {
        if row.kind == EntryKind::File {
            row.classification = Some(held.and_then(|tier| tier.file(&row.path)).map_or_else(
                || index.classify(&row.path).into(),
                |record| record.detection.clone(),
            ));
        }
    }
    (rows, total)
}

#[derive(Clone, Copy)]
enum MetricMeasure {
    Additive(u64),
    Normalized(LogicalWordStats),
}

#[derive(Default)]
struct MetricAggregate {
    additive: u64,
    normalized: LogicalWordStats,
    measured: bool,
}

impl MetricAggregate {
    fn add(&mut self, measure: MetricMeasure) {
        self.measured = true;
        match measure {
            MetricMeasure::Additive(value) => {
                self.additive = self.additive.saturating_add(value);
            }
            MetricMeasure::Normalized(stats) => self.normalized.add_assign(stats),
        }
    }

    fn value(&self, normalized: bool) -> Option<u64> {
        self.measured.then(
            || {
                if normalized { self.normalized.logical_words() } else { self.additive }
            },
        )
    }
}

fn measured_file_metric(
    record: &crate::content::FileAnalysis,
    file_type: &str,
    metric_name: &str,
) -> Option<MetricMeasure> {
    let direct = |value| Some(MetricMeasure::Additive(value));
    match metric_name {
        "physical_lines" => direct(record.lines.value()?.physical_lines),
        "blank_lines" => direct(record.lines.value()?.blank_lines),
        "nonblank_lines" => direct(record.lines.value()?.nonblank_lines),
        "raw_words" => direct(record.lines.value()?.raw_words),
        "code_lines" => direct(record.code?.value()?.code_lines),
        "comment_lines" => direct(record.code?.value()?.comment_lines),
        "code_blank_lines" => direct(record.code?.value()?.code_blank_lines),
        "paragraphs" => direct(record.words?.value()?.paragraphs),
        "visible_words" => direct(record.words?.value()?.visible_words),
        "logical_words" => {
            Some(MetricMeasure::Normalized(record.words?.value()?.logical_word_stats))
        }
        "visible_logical_words" => {
            Some(MetricMeasure::Normalized(record.words?.value()?.visible_logical_word_stats))
        }
        "document_words" => {
            let words = record.words?.value()?;
            Some(MetricMeasure::Normalized(if file_type == "markdown" {
                words.visible_logical_word_stats
            } else {
                words.logical_word_stats
            }))
        }
        _ => None,
    }
}

fn metric_sort_values(
    index: &Index,
    content: AnalysisSet,
    sources: &[FileRow],
    metric_name: &str,
) -> BTreeMap<PathBuf, u64> {
    let held = index.content().and_then(|tier| tier.admit(&index.content_identity(content)));
    let mut aggregate = BTreeMap::<PathBuf, MetricAggregate>::new();
    for file in sources.iter().filter(|row| row.kind == EntryKind::File) {
        let Some(record) = held.and_then(|tier| tier.file(&file.path)) else { continue };
        let classification = index.classify(&file.path);
        let Some(measure) =
            measured_file_metric(record, classification.file_type.as_str(), metric_name)
        else {
            continue;
        };
        let mut path = Some(file.path.as_path());
        while let Some(current) = path {
            if current.as_os_str().is_empty() {
                break;
            }
            aggregate.entry(current.to_path_buf()).or_default().add(measure);
            path = current.parent();
        }
    }
    let normalized =
        matches!(metric_name, "logical_words" | "visible_logical_words" | "document_words");
    aggregate
        .into_iter()
        .filter_map(|(path, values)| values.value(normalized).map(|value| (path, value)))
        .collect()
}

/// Every entry in the index, for an unfiltered files view.
fn every_entry(index: &Index) -> Vec<FileRow> {
    debug_assert!(!index.is_folded(), "a folded index keeps too few files to list");
    let mut rows = Vec::new();
    let mut stack: Vec<(EntryId, PathBuf)> = vec![(EntryId::ROOT, PathBuf::new())];
    while let Some((id, path)) = stack.pop() {
        let Some(children) = index.children_of(id) else {
            continue;
        };
        let children: Vec<(PathBuf, EntryId)> =
            children.map(|(name, child)| (path.join(name), child)).collect();
        for (child_path, child) in children {
            let (Some(kind), Some(attrs)) = (index.kind_of(child), index.attrs_of(child)) else {
                continue;
            };
            rows.push(FileRow {
                root: 0,
                path: child_path.clone(),
                kind,
                bytes: attrs.size,
                allocated: attrs.allocated,
                mtime_ns: attrs.mtime_ns,
                files: None,
                dirs: None,
                complete: None,
                age_ns: None,
                ignored: index.ignored_classification_of(&child_path, child),
                sort_value: None,
                classification: None,
            });
            if kind == EntryKind::Dir {
                stack.push((child, child_path));
            }
        }
    }
    rows
}

/// The tree section's rows: the root, expanded to the requested depth and bounded, beside
/// what the section's top boundary omitted.
fn tree_section(
    reads: &[RootRead<'_>],
    query: &Query,
    content: AnalysisSet,
) -> (Option<TreeNode>, Vec<TreeOmission>) {
    debug_assert_eq!(reads.len(), 1, "several roots' trees assemble once the section has a total");
    let read = &reads[0];
    let recency = read.recency().expect("a tree view reads recency from its pass or its walk");
    let mut root = tree_root(read, recency, ".".to_string());
    let mut top = Vec::new();
    let limit = query.limit_for(ViewSpec::Tree);
    if limit == Bound::Limit(0) {
        let kept = cap_tree_forest(vec![(root, read.listed_in_full())], 0, &mut top);
        debug_assert!(kept.is_empty(), "a zero row cap keeps no root");
        return (None, top);
    }
    let grand = match query.selection.size {
        SizeMetric::Apparent => root.bytes,
        SizeMetric::Allocated => root.allocated,
    };
    let metric_values = tree_metric_values(read, query, content);
    expand(read.index, query, read.walked.as_ref(), &metric_values, recency, grand, &mut root);
    if let Some(cap) = limit.limit() {
        root = cap_tree_forest(vec![(root, read.listed_in_full())], cap, &mut top)
            .pop()
            .expect("a positive row cap keeps the root");
    }
    (Some(root), top)
}

/// The tree section of a report over several roots: a total row and one tree per root,
/// beside the root rows the row limit cut.
///
/// Each root's tree is the one its root alone would have, built by the same expansion with
/// the combined total as its share denominator, so a row's share is of the whole report
/// and `--depth` counts below each root. Every root is a row, whatever its share and with
/// no breadth bound, because the caller named it; the roots are ordered by the tree sorter
/// with their labels as names, so a metric sort, which measures no root, puts them in label
/// order. The row limit applies once, over the assembled pre-order with the total as the
/// first row, so `--limit 1` shows only the total and `--limit 0` nothing.
fn root_trees(
    reads: &[RootRead<'_>],
    labels: &[NamedRoot],
    query: &Query,
    content: AnalysisSet,
) -> (RootTrees, Vec<TreeOmission>) {
    let summary = sum_roots(reads.iter().map(RootRead::summary));
    let grand = match query.selection.size {
        SizeMetric::Apparent => summary.bytes,
        SizeMetric::Allocated => summary.allocated,
    };
    let limit = query.limit_for(ViewSpec::Tree);
    let mut rows: Vec<(TreeNode, usize)> = reads
        .iter()
        .map(|read| {
            let recency =
                read.recency().expect("a tree view reads recency from its pass or its walk");
            let label = labels[read.root].label.to_string_lossy().into_owned();
            let mut node = tree_root(read, recency, label);
            if limit != Bound::Limit(0) {
                let metric_values = tree_metric_values(read, query, content);
                let walked = read.walked.as_ref();
                expand(read.index, query, walked, &metric_values, recency, grand, &mut node);
            }
            (node, read.root)
        })
        .collect();
    sort_rows_by(
        &mut rows,
        query,
        ViewSpec::Tree,
        SortAccessors {
            size: |(row, _): &(TreeNode, usize), metric| match metric {
                SizeMetric::Apparent => row.bytes,
                SizeMetric::Allocated => row.allocated,
            },
            count: |(row, _): &(TreeNode, usize)| row.files,
            mtime: |(row, _): &(TreeNode, usize)| row.mtime_ns,
            name: borrowed_name(|(row, _): &(TreeNode, usize)| {
                std::borrow::Cow::Borrowed(row.name.as_str())
            }),
            content_metric: |_: &(TreeNode, usize), _: &MetricDef| None,
            rank: |_: &(TreeNode, usize)| 0,
        },
    );
    let total = TreeTotal {
        bytes: summary.bytes,
        allocated: summary.allocated,
        files: summary.files,
        dirs: summary.dirs,
        ignored: summary.ignored,
        newest_mtime_ns: summary.newest_mtime_ns,
        mtime_ns: rows.iter().fold(None, |newest, (row, _)| newer(newest, row.mtime_ns)),
        complete: rows.iter().all(|(row, _)| row.complete == Some(true)),
        age_ns: None,
    };
    let order: Vec<usize> = rows.iter().map(|(_, root)| *root).collect();
    let forest: Vec<(TreeNode, bool)> =
        rows.into_iter().map(|(node, root)| (node, reads[root].listed_in_full())).collect();
    let mut top = Vec::new();
    let (total, kept) = match limit {
        Bound::Limit(0) => (None, cap_tree_forest(forest, 0, &mut top)),
        // The total is the first row, so the trees share what is left of the cap.
        Bound::Limit(cap) => (Some(total), cap_tree_forest(forest, cap - 1, &mut top)),
        Bound::All => (Some(total), forest.into_iter().map(|(node, _)| node).collect()),
    };
    // A row cap keeps a prefix of the trees in order, so the kept trees are the first roots.
    let trees = kept.into_iter().zip(order).map(|(tree, root)| RootTree { root, tree }).collect();
    (RootTrees { total, trees }, top)
}

/// One root's tree row, before its children: the root's totals under the selection, its
/// newest activity, and whether it was listed in full.
fn tree_root(read: &RootRead<'_>, recency: TreeRecency<'_>, name: String) -> TreeNode {
    let summary = read.summary();
    TreeNode {
        path: PathBuf::new(),
        name,
        kind: EntryKind::Dir,
        entry_ignored: read.index.ignored_classification_of(Path::new(""), EntryId::ROOT),
        bytes: summary.bytes,
        allocated: summary.allocated,
        files: summary.files,
        dirs: summary.dirs,
        ignored: summary.ignored,
        newest_mtime_ns: summary.newest_mtime_ns,
        mtime_ns: recency.activity(EntryId::ROOT),
        complete: Some(recency.complete(EntryId::ROOT)),
        age_ns: None,
        children: Vec::new(),
        omissions: Vec::new(),
        truncated: false,
    }
}

/// The values a metric sort ranks one root's tree rows by, keyed by root-relative path.
fn tree_metric_values(
    read: &RootRead<'_>,
    query: &Query,
    content: AnalysisSet,
) -> BTreeMap<PathBuf, u64> {
    let Some(SortKey::Metric(name)) = query.selection.sort else {
        return BTreeMap::new();
    };
    match &read.walked {
        Some(walked) => metric_sort_values(read.index, content, &walked.members, name),
        None => metric_sort_values(read.index, content, &every_entry(read.index), name),
    }
}

impl RootRead<'_> {
    /// Whether this root's index listed its whole scope, so an omitted row's sizes are
    /// exact rather than lower bounds.
    fn listed_in_full(&self) -> bool {
        self.index.state().coverage == crate::Coverage::Complete
    }
}

/// The files a folded index counted in one directory without keeping them, as the rows
/// of one file each that a share threshold omits there: every such file is below it.
#[derive(Clone, Copy)]
struct FoldedRows {
    entries: usize,
    files: u64,
    bytes: u64,
    allocated: u64,
    /// Their ignored part, `None` exactly where a kept file's row in the same directory
    /// would carry no classification ([`Index::children_classification_known`]).
    ignored: Option<IgnoredSize>,
}

impl FoldedRows {
    /// No rows at all.
    const NONE: Self = Self {
        entries: 0,
        files: 0,
        bytes: 0,
        allocated: 0,
        ignored: Some(IgnoredSize { bytes: 0, allocated: 0 }),
    };

    /// The folded rows of directory `id` at `path`, if its index folded any there.
    fn of(index: &Index, id: EntryId, path: &Path) -> Option<Self> {
        let folded = index.folded_children(id)?;
        Some(Self {
            entries: usize::try_from(folded.files).unwrap_or(usize::MAX),
            files: folded.files,
            bytes: folded.bytes,
            allocated: folded.allocated,
            ignored: index.children_classification_known(path).then_some(folded.ignored),
        })
    }
}

/// What an omitted row contributes to its omission: the scalars of a [`TreeNode`],
/// without the path and name a node also carries. A row the share threshold omits is
/// counted here without ever becoming a node (H186).
#[derive(Clone, Copy)]
struct RowFacts {
    files: u64,
    bytes: u64,
    allocated: u64,
    ignored: Option<IgnoredTally>,
}

impl From<&TreeNode> for RowFacts {
    fn from(node: &TreeNode) -> Self {
        Self {
            files: node.files,
            bytes: node.bytes,
            allocated: node.allocated,
            ignored: node.ignored,
        }
    }
}

fn facts_of(rows: &[(TreeNode, EntryId)]) -> Vec<RowFacts> {
    rows.iter().map(|(row, _)| RowFacts::from(row)).collect()
}

fn record_omission(
    node: &mut TreeNode,
    reason: TreeOmissionReason,
    rows: &[RowFacts],
    folded: Option<FoldedRows>,
    complete: bool,
) {
    if rows.is_empty() && folded.is_none() {
        return;
    }
    // Folded rows are summed first. Every term is unsigned, so a checked sum overflows
    // exactly when the sum of all of them does, in whatever order they are added.
    let seed = folded.unwrap_or(FoldedRows::NONE);
    let bytes = complete
        .then(|| rows.iter().try_fold(seed.bytes, |sum, row| sum.checked_add(row.bytes)))
        .flatten();
    let allocated = complete
        .then(|| rows.iter().try_fold(seed.allocated, |sum, row| sum.checked_add(row.allocated)))
        .flatten();
    let files = complete
        .then(|| rows.iter().try_fold(seed.files, |sum, row| sum.checked_add(row.files)))
        .flatten();
    let ignored = complete
        .then(|| {
            rows.iter().try_fold(seed.ignored?, |sum, row| {
                sum.checked_add(IgnoredSize::from_tally(row.ignored?))
            })
        })
        .flatten();
    node.omissions.push(TreeOmission {
        reason,
        entries: rows.len().saturating_add(seed.entries),
        files,
        bytes,
        allocated,
        ignored,
    });
    node.truncated = true;
}

/// Keep the first `cap` rows of a forest in pre-order and omit the rest, each tree's rows
/// with that tree's completeness.
///
/// A cut row is recorded as a row omission on its parent, merged with any row omission
/// already there, and its whole subtree goes with it. A cut tree root has no parent, so its
/// record goes to `top`, the section's own top boundary. One definition for every row
/// bound: a single tree under a positive cap keeps its root, and under a zero cap leaves
/// only the record of the root in `top`; several trees under one cap share it in order.
/// Returns the trees that kept their roots, in order.
fn cap_tree_forest(
    trees: Vec<(TreeNode, bool)>,
    cap: usize,
    top: &mut Vec<TreeOmission>,
) -> Vec<TreeNode> {
    struct Pending {
        node: TreeNode,
        parent: Option<usize>,
        /// Whether this row's tree was listed in full, so its omission is exact.
        complete: bool,
    }
    let mut pending: Vec<Pending> = trees
        .into_iter()
        .rev()
        .map(|(node, complete)| Pending { node, parent: None, complete })
        .collect();
    let mut kept: Vec<Pending> = Vec::new();
    while let Some(mut item) = pending.pop() {
        if kept.len() == cap {
            let complete = item.complete;
            let omission = TreeOmission {
                reason: TreeOmissionReason::Rows,
                entries: 1,
                files: complete.then_some(item.node.files),
                bytes: complete.then_some(item.node.bytes),
                allocated: complete.then_some(item.node.allocated),
                ignored: complete
                    .then_some(item.node.ignored)
                    .flatten()
                    .map(IgnoredSize::from_tally),
            };
            let omissions = match item.parent {
                Some(parent) => {
                    let owner = &mut kept[parent].node;
                    owner.truncated = true;
                    &mut owner.omissions
                }
                None => &mut *top,
            };
            if let Some(existing) =
                omissions.iter_mut().find(|existing| existing.reason == TreeOmissionReason::Rows)
            {
                existing.entries += 1;
                existing.files =
                    existing.files.zip(omission.files).and_then(|(a, b)| a.checked_add(b));
                existing.bytes =
                    existing.bytes.zip(omission.bytes).and_then(|(a, b)| a.checked_add(b));
                existing.allocated =
                    existing.allocated.zip(omission.allocated).and_then(|(a, b)| a.checked_add(b));
                existing.ignored =
                    existing.ignored.zip(omission.ignored).and_then(|(a, b)| a.checked_add(b));
            } else {
                omissions.push(omission);
            }
            continue;
        }
        let children = std::mem::take(&mut item.node.children);
        let (current, complete) = (kept.len(), item.complete);
        kept.push(item);
        for child in children.into_iter().rev() {
            pending.push(Pending { node: child, parent: Some(current), complete });
        }
    }
    // Fold from the end: every parent's position is smaller than its child's, so by the time
    // a row is folded into its parent, every row beneath it already has been. Each row is
    // taken from its slot rather than removed, so no later row moves (review C9 on #192),
    // and children arrive last first, so each row's are reversed once, when it is folded.
    let mut slots: Vec<Option<Pending>> = kept.into_iter().map(Some).collect();
    for position in (0..slots.len()).rev() {
        let Some(parent) = slots[position].as_ref().and_then(|item| item.parent) else {
            continue;
        };
        let mut child = slots[position].take().expect("each row is folded once").node;
        child.children.reverse();
        slots[parent].as_mut().expect("a parent precedes its child").node.children.push(child);
    }
    slots
        .into_iter()
        .flatten()
        .map(|mut item| {
            item.node.children.reverse();
            item.node
        })
        .collect()
}

/// Attach a node's children, honoring the depth and per-directory limit bounds.
///
/// Iterative rather than recursive: this engine indexes trees deep enough that recursive
/// expansion would exhaust the stack, and a report that panics on a deep tree fails
/// exactly where the tool is most useful. Nodes are built flat with parent links in
/// pre-order, then folded together from the leaves up.
///
/// `grand` is the share threshold's denominator: the root's own total for one root, and
/// the total of every root for several, so a row's share means one thing in one report.
fn expand(
    index: &Index,
    query: &Query,
    walked: Option<&Walked>,
    metric_values: &BTreeMap<PathBuf, u64>,
    recency: TreeRecency<'_>,
    grand: u64,
    node: &mut TreeNode,
) {
    /// One node awaiting its children.
    struct Pending {
        node: TreeNode,
        id: EntryId,
        depth: usize,
        parent: Option<usize>,
    }

    let mut built = vec![Pending {
        // Only identity and bounds matter while expanding; the caller keeps the
        // populated root and receives its children back at the end.
        node: TreeNode {
            path: node.path.clone(),
            name: node.name.clone(),
            kind: node.kind,
            entry_ignored: node.entry_ignored,
            bytes: node.bytes,
            allocated: node.allocated,
            files: node.files,
            dirs: node.dirs,
            ignored: node.ignored,
            newest_mtime_ns: node.newest_mtime_ns,
            mtime_ns: node.mtime_ns,
            complete: node.complete,
            age_ns: node.age_ns,
            children: Vec::new(),
            omissions: Vec::new(),
            truncated: false,
        },
        id: EntryId::ROOT,
        depth: 0,
        parent: None,
    }];

    let threshold = query.min_share_for();
    let complete = index.state().coverage == crate::Coverage::Complete;
    let mut cursor = 0;
    while cursor < built.len() {
        let (id, depth) = (built[cursor].id, built[cursor].depth);
        let path = built[cursor].node.path.clone();

        let (mut rows, below_share) =
            child_rows(index, query, walked, metric_values, recency, id, &path, &threshold, grand);
        // A folded index kept only files that can reach the share, so the files it folded
        // here are rows below it as well.
        record_omission(
            &mut built[cursor].node,
            TreeOmissionReason::Share,
            &below_share,
            FoldedRows::of(index, id, &path),
            true,
        );
        if !query.depth_for(ViewSpec::Tree).admits(depth) {
            record_omission(
                &mut built[cursor].node,
                TreeOmissionReason::Depth,
                &facts_of(&rows),
                None,
                complete,
            );
            cursor += 1;
            continue;
        }
        if let Some(cap) = query.breadth_for().limit() {
            let hidden = rows.split_off(cap.min(rows.len()));
            record_omission(
                &mut built[cursor].node,
                TreeOmissionReason::Breadth,
                &facts_of(&hidden),
                None,
                complete,
            );
        }

        for (child_node, child_id) in rows {
            built.push(Pending {
                node: child_node,
                id: child_id,
                depth: depth + 1,
                parent: Some(cursor),
            });
        }
        cursor += 1;
    }

    // Fold from the end: every parent index is smaller than its child's, so removing the
    // last element never disturbs an index still to be used.
    for position in (1..built.len()).rev() {
        let child = built.remove(position);
        let parent = child.parent.expect("only the root has no parent");
        built[parent].node.children.insert(0, child.node);
    }

    let mut root = built.pop().expect("the root is always present");
    node.children = std::mem::take(&mut root.node.children);
    node.omissions = std::mem::take(&mut root.node.omissions);
    node.truncated = root.node.truncated;
}

/// The directory children of one node that the share threshold admits, shaped and
/// sorted but not yet expanded, beside the facts of those it omits.
///
/// Each child's roll-up decides its admission before anything else is built for it
/// (H186): a node carries a path and a name, and the share omits most children of a wide
/// directory, so the report used to build, sort, clone and free a node for every child
/// it would never show. The omitted children are summed by [`record_omission`] from their
/// facts alone. Admitting before sorting orders the admitted rows exactly as sorting all
/// of them would, since names are unique within a directory.
#[allow(clippy::too_many_arguments)]
fn child_rows(
    index: &Index,
    query: &Query,
    walked: Option<&Walked>,
    metric_values: &BTreeMap<PathBuf, u64>,
    recency: TreeRecency<'_>,
    id: EntryId,
    path: &Path,
    threshold: &ShareThreshold,
    grand: u64,
) -> (Vec<(TreeNode, EntryId)>, Vec<RowFacts>) {
    let Some(children) = index.children_of(id) else {
        return (Vec::new(), Vec::new());
    };
    // Every child's classification is known exactly when its parent's controls are.
    let children_known = index.children_classification_known(path);
    let mut child_path = path.to_path_buf();
    let mut rows: Vec<(TreeNode, EntryId)> = Vec::new();
    let mut below_share: Vec<RowFacts> = Vec::new();
    for (name, child) in children {
        let Some(kind) = index.kind_of(child) else {
            continue;
        };
        if !matches!(kind, EntryKind::Dir | EntryKind::File) {
            continue;
        }
        if walked.is_some_and(|walked| match kind {
            EntryKind::Dir => !walked.visible.contains(&child),
            EntryKind::File => !walked.visible_files.contains(&child),
            EntryKind::Symlink | EntryKind::Other => true,
        }) {
            continue;
        }
        let entry_ignored = children_known.then(|| index.entry_ignored(child)).flatten();
        child_path.push(name);
        let summary = if kind == EntryKind::File {
            let attrs = index.attrs_of(child).expect("live child has attributes");
            let ignored = entry_ignored.map(|ignored| {
                if ignored {
                    IgnoredTally {
                        files: 1,
                        dirs: 0,
                        bytes: attrs.size,
                        allocated: attrs.allocated,
                    }
                } else {
                    IgnoredTally::default()
                }
            });
            SummaryRow {
                files: 1,
                dirs: 0,
                bytes: attrs.size,
                allocated: attrs.allocated,
                ignored,
                newest_mtime_ns: Some(attrs.mtime_ns),
            }
        } else {
            match walked {
                None => unfiltered_summary(index, child, &child_path),
                Some(walked) => walked.summary_of(child),
            }
        };
        let value = match query.selection.size {
            SizeMetric::Apparent => summary.bytes,
            SizeMetric::Allocated => summary.allocated,
        };
        // A complete child below a partial root's observed total is also below the true
        // (at least as large) total. Only an incomplete child's own unknown contents
        // prevent that proof; unrelated scan errors do not.
        let (mtime_ns, complete) = match kind {
            EntryKind::File => (summary.newest_mtime_ns, None),
            _ => (recency.activity(child), Some(recency.complete(child))),
        };
        if complete != Some(false) && !threshold.admits(value, grand) {
            below_share.push(RowFacts {
                files: summary.files,
                bytes: summary.bytes,
                allocated: summary.allocated,
                ignored: summary.ignored,
            });
            child_path.pop();
            continue;
        }
        rows.push((
            TreeNode {
                path: child_path.clone(),
                name: name.to_string_lossy().into_owned(),
                kind,
                entry_ignored,
                bytes: summary.bytes,
                allocated: summary.allocated,
                files: summary.files,
                dirs: summary.dirs,
                ignored: summary.ignored,
                newest_mtime_ns: summary.newest_mtime_ns,
                mtime_ns,
                complete,
                age_ns: None,
                children: Vec::new(),
                omissions: Vec::new(),
                truncated: false,
            },
            child,
        ));
        child_path.pop();
    }

    sort_rows_by(
        &mut rows,
        query,
        ViewSpec::Tree,
        SortAccessors {
            size: |(row, _): &(TreeNode, EntryId), metric| match metric {
                SizeMetric::Apparent => row.bytes,
                SizeMetric::Allocated => row.allocated,
            },
            count: |(row, _): &(TreeNode, EntryId)| row.files,
            // The activity the age column shows, so the order matches the column it sorts
            // and the order a list sorted by recency already has.
            mtime: |(row, _): &(TreeNode, EntryId)| row.mtime_ns,
            name: borrowed_name(|(row, _): &(TreeNode, EntryId)| {
                std::borrow::Cow::Borrowed(row.name.as_str())
            }),
            content_metric: |(row, _): &(TreeNode, EntryId), _: &MetricDef| {
                metric_values.get(&row.path).copied()
            },
            rank: |_: &(TreeNode, EntryId)| 0,
        },
    );
    (rows, below_share)
}

/// Trim a row list to the configured limit.
fn truncate<T>(rows: &mut Vec<T>, limit: Bound) -> usize {
    let total = rows.len();
    if let Some(limit) = limit.limit() {
        rows.truncate(limit);
    }
    total
}

/// Sort rows by the effective key for a view.
struct SortAccessors<S, C, M, N, V, R> {
    size: S,
    count: C,
    mtime: M,
    name: N,
    content_metric: V,
    /// The rank of the row's root ([`label_ranks`]), which decides before the name does;
    /// constant for rows of one root, so it never reorders them among themselves.
    rank: R,
}

fn sort_rows<T>(
    rows: &mut [T],
    query: &Query,
    view: ViewSpec,
    accessors: SortAccessors<
        impl Fn(&T, SizeMetric) -> u64,
        impl Fn(&T) -> u64,
        impl Fn(&T) -> Option<i64>,
        impl for<'r> Fn(&'r T) -> std::borrow::Cow<'r, str>,
        impl Fn(&T, &MetricDef) -> Option<u64>,
        impl Fn(&T) -> usize,
    >,
) {
    sort_rows_by(rows, query, view, accessors);
}

/// A name accessor that borrows from its row, given the higher-ranked signature a closure
/// written inside a [`SortAccessors`] literal is not inferred to have.
fn borrowed_name<T, F>(name: F) -> F
where
    F: for<'r> Fn(&'r T) -> std::borrow::Cow<'r, str>,
{
    name
}

/// Sort rows by the effective key, with a stable name tiebreak.
///
/// The name accessor borrows: the tiebreak runs once per compared pair, and a sort that
/// cloned two `String`s for each was 4.6G instructions on an 80k-row listing (H186).
fn sort_rows_by<T>(
    rows: &mut [T],
    query: &Query,
    view: ViewSpec,
    accessors: SortAccessors<
        impl Fn(&T, SizeMetric) -> u64,
        impl Fn(&T) -> u64,
        impl Fn(&T) -> Option<i64>,
        impl for<'r> Fn(&'r T) -> std::borrow::Cow<'r, str>,
        impl Fn(&T, &MetricDef) -> Option<u64>,
        impl Fn(&T) -> usize,
    >,
) {
    let SortAccessors { size, count, mtime, name, content_metric, rank } = accessors;
    let key = query.selection.sort.unwrap_or_else(|| view.default_sort());
    let metric = query.selection.size;
    // A row's name is its path under its root, so the root's rank comes first wherever the
    // name decides: rows read as sorted by label and then path.
    let named = |left: &T, right: &T| {
        rank(left).cmp(&rank(right)).then_with(|| name(left).cmp(&name(right)))
    };

    rows.sort_by(|left, right| {
        let ordering = match key {
            // Size, count, and recency read most-first: the interesting end is the top.
            SortKey::Size => size(right, metric).cmp(&size(left, metric)),
            SortKey::Count => count(right).cmp(&count(left)),
            SortKey::Mtime => mtime(right).cmp(&mtime(left)),
            SortKey::Name => named(left, right),
            SortKey::Metric(metric_name) => {
                let definition =
                    crate::content::METRICS.iter().find(|entry| entry.name == metric_name);
                let left_value = definition.and_then(|definition| content_metric(left, definition));
                let right_value =
                    definition.and_then(|definition| content_metric(right, definition));
                match (left_value, right_value) {
                    (Some(left), Some(right)) if query.selection.reverse => left.cmp(&right),
                    (Some(left), Some(right)) => right.cmp(&left),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                }
            }
        };
        // A name tiebreak keeps equal rows in a deterministic order, which is what makes
        // the goldens stable across runs and platforms.
        ordering.then_with(|| named(left, right))
    });

    if query.selection.reverse && !matches!(key, SortKey::Metric(_)) {
        rows.reverse();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine_contract::{Attrs, Observation, Op};
    use crate::query::query_glob::Pattern;
    use crate::query::query_selection::ModifiedWindow;
    use std::fs;
    use std::time::{Duration, UNIX_EPOCH};

    fn attrs(size: u64, mtime_ns: i64) -> Attrs {
        Attrs {
            size,
            allocated: size.div_ceil(512) * 512,
            mtime_ns,
            ctime_ns: mtime_ns,
            inode: size.wrapping_mul(31).wrapping_add(mtime_ns.unsigned_abs()),
            dev: 1,
        }
    }

    fn upsert(path: &str, kind: EntryKind, attrs: Attrs) -> Op {
        Op::Upsert { path: PathBuf::from(path), kind, attrs }
    }

    /// A tree with two top-level directories, a nested level, and three extensions.
    fn sample() -> Index {
        let mut index = Index::new("/root");
        index
            .apply(&Observation::new(vec![
                upsert("src", EntryKind::Dir, Attrs::default()),
                upsert("src/main.rs", EntryKind::File, attrs(100, 10)),
                upsert("src/lib.rs", EntryKind::File, attrs(200, 20)),
                upsert("src/deep", EntryKind::Dir, Attrs::default()),
                upsert("src/deep/nested.rs", EntryKind::File, attrs(50, 40)),
                upsert("docs", EntryKind::Dir, Attrs::default()),
                upsert("docs/guide.md", EntryKind::File, attrs(300, 30)),
                upsert("notes.txt", EntryKind::File, attrs(7, 5)),
            ]))
            .expect("apply");
        index
    }

    /// A directory row of `bytes`, apparent and allocated alike, over `children`.
    fn row(name: &str, bytes: u64, children: Vec<TreeNode>) -> TreeNode {
        TreeNode {
            path: PathBuf::from(name),
            name: name.to_owned(),
            kind: EntryKind::Dir,
            entry_ignored: Some(false),
            bytes,
            allocated: bytes,
            files: 1,
            dirs: 0,
            ignored: Some(IgnoredTally::default()),
            newest_mtime_ns: None,
            mtime_ns: None,
            complete: Some(true),
            age_ns: None,
            children,
            omissions: Vec::new(),
            truncated: false,
        }
    }

    /// A row's name, with the entries and bytes of its row omission when it has one.
    type CappedRow = (String, Option<(usize, Option<u64>)>);

    /// Every row in pre-order, as [`CappedRow`]s.
    fn capped(trees: &[TreeNode]) -> Vec<CappedRow> {
        let mut out = Vec::new();
        let mut stack: Vec<&TreeNode> = trees.iter().rev().collect();
        while let Some(node) = stack.pop() {
            let rows = node.omissions.iter().find(|o| o.reason == TreeOmissionReason::Rows);
            out.push((node.name.clone(), rows.map(|o| (o.entries, o.bytes))));
            stack.extend(node.children.iter().rev());
        }
        out
    }

    /// The row cap keeps the first rows in pre-order, whole forest or one tree: a cut row
    /// takes its subtree with it into its parent's record, a cut tree root goes to the
    /// section's top record, and an incomplete tree's records are not exact.
    #[test]
    fn the_row_cap_keeps_the_first_rows_of_a_forest_in_pre_order() {
        let tree = || {
            row(
                ".",
                100,
                vec![row("a", 60, vec![row("a1", 40, Vec::new())]), row("b", 30, Vec::new())],
            )
        };
        let some = |name: &str| (name.to_owned(), None);
        let cut = |name: &str, entries, bytes| (name.to_owned(), Some((entries, Some(bytes))));

        let mut top = Vec::new();
        assert!(cap_tree_forest(vec![(tree(), true)], 0, &mut top).is_empty());
        assert_eq!(
            top,
            [TreeOmission {
                reason: TreeOmissionReason::Rows,
                entries: 1,
                files: Some(1),
                bytes: Some(100),
                allocated: Some(100),
                ignored: Some(IgnoredSize::default()),
            }]
        );
        for (cap, expected) in [
            (1, vec![cut(".", 2, 90)]),
            (2, vec![cut(".", 1, 30), cut("a", 1, 40)]),
            (3, vec![cut(".", 1, 30), some("a"), some("a1")]),
            (4, vec![some("."), some("a"), some("a1"), some("b")]),
        ] {
            let mut top = Vec::new();
            let kept = cap_tree_forest(vec![(tree(), true)], cap, &mut top);
            assert_eq!(capped(&kept), expected, "cap {cap}");
            assert!(top.is_empty(), "cap {cap} keeps the root");
            assert_eq!(kept[0].truncated, cap < 4, "cap {cap}");
        }
        let mut top = Vec::new();
        let kept = cap_tree_forest(vec![(tree(), false)], 1, &mut top);
        assert_eq!(capped(&kept), [(".".to_owned(), Some((2, None)))], "inexact when incomplete");

        // Several trees share one cap in order; the cut roots merge into one top record,
        // each exact by its own tree's completeness.
        let mut top = Vec::new();
        let forest = vec![
            (tree(), true),
            (row("x", 50, Vec::new()), true),
            (row("y", 5, Vec::new()), false),
        ];
        let kept = cap_tree_forest(forest, 3, &mut top);
        assert_eq!(capped(&kept), [cut(".", 1, 30), some("a"), some("a1")]);
        assert_eq!((top.len(), top[0].entries, top[0].bytes), (1, 2, None));
        let mut top = Vec::new();
        let forest = vec![(tree(), true), (row("x", 50, Vec::new()), true)];
        let kept = cap_tree_forest(forest, 5, &mut top);
        assert_eq!(capped(&kept), [some("."), some("a"), some("a1"), some("b"), some("x")]);
        assert!(top.is_empty());
    }

    #[test]
    fn ages_use_one_signed_reference_and_unrepresentable_clocks_are_unknown() {
        let mut index = sample();
        index.apply_ok(&Observation::new(vec![
            upsert("past", EntryKind::File, attrs(1, -10)),
            upsert("future", EntryKind::File, attrs(1, i64::MAX)),
        ]));
        let query = query(
            &[ViewSpec::Files],
            Selection { include: vec![pattern("past"), pattern("future")], ..Selection::default() },
        );
        let mut request = Request::new(Basis::held_by(&index), query, UNIX_EPOCH);
        let answer = report(&index, &request, UNIX_EPOCH).expect("report");
        let rows = files_of(&answer);
        assert_eq!(answer.age_reference_ns, Some(0));
        assert_eq!(
            rows.iter().map(|r| r.age_ns).collect::<Vec<_>>(),
            [Some(-i128::from(i64::MAX)), Some(10)]
        );
        request.now = UNIX_EPOCH + Duration::from_secs(10_000_000_000);
        let answer = report(&index, &request, UNIX_EPOCH)
            .expect("out-of-range reference is representable as unknown age");
        assert_eq!(answer.age_reference_ns, None);
        assert!(files_of(&answer).iter().all(|row| row.age_ns.is_none()));
    }

    #[test]
    fn matching_a_directory_selects_its_subtree_once() {
        let index = sample();
        let selection = Selection {
            include: vec![pattern("src"), pattern("deep")],
            kinds: vec![EntryKind::Dir],
            size: SizeMetric::Apparent,
            min_size: Some(40),
            ..Selection::default()
        };
        let report = run(
            &index,
            &query(&[ViewSpec::Files, ViewSpec::Summary, ViewSpec::Extensions], selection),
        );
        let rows = files_of(&report);
        assert_eq!(rows.len(), 2);
        assert_eq!((rows[0].bytes, rows[0].mtime_ns), (350, 40));
        let Section::Summary(summary) = &report.sections[1] else { panic!("summary") };
        assert_eq!((summary.files, summary.dirs, summary.bytes), (3, 2, 350));
        let Section::Extensions { rows, .. } = &report.sections[2] else { panic!("extensions") };
        assert_eq!((rows[0].files, rows[0].bytes), (3, 350));
    }

    #[test]
    fn a_selection_over_an_exactly_full_tree_sums_to_u64_max() {
        // Every route that builds an index refuses a total a u64 cannot hold (fdu-sqyk),
        // so every selected subset of an index's tree fits too; the filtered tier
        // re-aggregates entry by entry and must reach the exact bound without saturating
        // or wrapping.
        let exact =
            |size: u64, mtime_ns: i64| Attrs { size, allocated: size, ..attrs(1, mtime_ns) };
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            upsert("big", EntryKind::Dir, Attrs::default()),
            upsert("big/half.bin", EntryKind::File, exact(1 << 63, 1)),
            upsert("big/rest.bin", EntryKind::File, exact((1 << 63) - 1, 2)),
            upsert("empty.txt", EntryKind::File, exact(0, 3)),
        ]));
        assert_eq!(index.total().bytes, u64::MAX);
        let selection = Selection {
            include: vec![pattern("big")],
            size: SizeMetric::Apparent,
            min_size: Some(1),
            ..Selection::default()
        };
        let report = run(&index, &query(&[ViewSpec::Summary, ViewSpec::Files], selection));
        let Section::Summary(summary) = &report.sections[0] else { panic!("summary") };
        assert_eq!((summary.files, summary.dirs, summary.bytes), (2, 1, u64::MAX));
        let Section::Files { rows, .. } = &report.sections[1] else { panic!("files") };
        assert_eq!(rows.iter().map(|row| row.bytes).sum::<u64>(), u64::MAX);
    }

    #[test]
    fn subtree_predicates_include_directory_and_symlink_activity_but_only_file_bytes() {
        let mut index = sample();
        index
            .apply(&Observation::new(vec![
                upsert("src", EntryKind::Dir, attrs(9999, 45)),
                upsert("src/empty", EntryKind::Dir, attrs(8888, 60)),
                upsert("src/link", EntryKind::Symlink, attrs(7777, 70)),
                upsert("empty", EntryKind::Dir, attrs(6666, -10)),
            ]))
            .expect("apply");
        let base = Selection {
            include: vec![pattern("src"), pattern("empty")],
            size: SizeMetric::Apparent,
            ..Selection::default()
        };
        // A one-shot report matches native names, so the nested spelling is joined rather
        // than written with a literal separator: `src/empty` holds only on Unix.
        let nested = PathBuf::from("src").join("empty").to_string_lossy().into_owned();
        let rows = files_of(&run(&index, &query(&[ViewSpec::Files], base.clone())));
        assert_eq!(
            rows.iter()
                .map(|r| (r.path.to_string_lossy().into_owned(), r.bytes, r.mtime_ns))
                .collect::<Vec<_>>(),
            [("empty".into(), 0, -10), ("src".into(), 350, 70), (nested.clone(), 0, 60)]
        );
        for (before, since, expected) in [
            (70, 0, vec![nested.clone()]),
            (71, 70, vec!["src".to_owned()]),
            (0, -10, vec!["empty".to_owned()]),
        ] {
            let selection = Selection {
                modified: ModifiedWindow { before: Some(before), since: Some(since) },
                ..base.clone()
            };
            let rows = files_of(&run(&index, &query(&[ViewSpec::Files], selection)));
            assert_eq!(
                rows.iter().map(|r| r.path.to_string_lossy().into_owned()).collect::<Vec<_>>(),
                expected
            );
        }
        for (size, minimum, expected) in [
            (SizeMetric::Apparent, 350, 1),
            (SizeMetric::Apparent, 351, 0),
            (SizeMetric::Allocated, 1536, 1),
            (SizeMetric::Allocated, 1537, 0),
        ] {
            let selection = Selection { size, min_size: Some(minimum), ..base.clone() };
            assert_eq!(
                files_of(&run(&index, &query(&[ViewSpec::Files], selection))).len(),
                expected
            );
        }
    }

    #[test]
    fn exclusions_apply_before_subtree_bounds_and_selected_ancestor_coverage() {
        let index = classified_sample();
        for (ignored, exclude, bytes, newest) in [
            (IgnoredEntries::Include, vec![], 325, 70),
            (IgnoredEntries::Exclude, vec![], 300, 20),
            (IgnoredEntries::Include, vec![pattern("*.log")], 300, 20),
            (IgnoredEntries::Include, vec![pattern("lib.rs")], 125, 70),
        ] {
            let selection = Selection {
                include: vec![pattern("src")],
                kinds: vec![EntryKind::Dir],
                ignored,
                exclude,
                size: SizeMetric::Apparent,
                ..Selection::default()
            };
            let report = run(
                &index,
                &query(&[ViewSpec::Files, ViewSpec::Summary, ViewSpec::Types], selection),
            );
            let row = &files_of(&report)[0];
            assert_eq!((row.bytes, row.mtime_ns), (bytes, newest));
            let Section::Summary(summary) = &report.sections[1] else { panic!("summary") };
            assert_eq!(summary.bytes, bytes);
            let Section::Metrics { summary, .. } = &report.sections[2] else { panic!("types") };
            assert_eq!(summary.rows.iter().map(|r| r.bytes).sum::<u64>(), bytes);
        }
        let selection = Selection {
            include: vec![pattern("build")],
            exclude: vec![pattern("cache")],
            kinds: vec![EntryKind::Dir],
            ..Selection::default()
        };
        let report = run(&index, &query(&[ViewSpec::Files, ViewSpec::Summary], selection));
        assert_eq!(files_of(&report)[0].bytes, 0);
        let Section::Summary(summary) = &report.sections[1] else { panic!("summary") };
        assert_eq!((summary.files, summary.dirs, summary.bytes), (0, 1, 0));
        let only = Selection {
            include: vec![pattern("cache")],
            kinds: vec![EntryKind::Dir],
            ignored: IgnoredEntries::Only,
            ..Selection::default()
        };
        assert_eq!(files_of(&run(&index, &query(&[ViewSpec::Files], only)))[0].bytes, 1000);
    }

    /// A directory whose subtree was not listed in full says so, and its lower-bound
    /// activity is not an age: it matches no modification bound, in either direction,
    /// while a size bound still holds on the lower bound it can prove.
    #[test]
    fn an_incomplete_subtree_reports_lower_bounds_and_matches_no_time_bound() {
        // `--scan-depth 2`: `env/lib` sits at the boundary, retained and never listed, so
        // `env` is incomplete; `docs` holds only files at that depth and is complete.
        let mut index = Index::new_with_scope(
            "/root",
            crate::ScanScope { max_depth: Some(2), ..crate::ScanScope::default() },
        );
        index.apply_ok(&Observation::new(vec![
            upsert("env", EntryKind::Dir, attrs(0, 5)),
            upsert("env/lib", EntryKind::Dir, attrs(0, 7)),
            upsert("env/a.bin", EntryKind::File, attrs(100, 40)),
            upsert("docs", EntryKind::Dir, attrs(0, 5)),
            upsert("docs/guide.md", EntryKind::File, attrs(30, 50)),
        ]));
        let directories = |selection: Selection| {
            let selection =
                Selection { kinds: vec![EntryKind::Dir], size: SizeMetric::Apparent, ..selection };
            files_of(&run(&index, &flat(selection)))
                .into_iter()
                .map(|row| {
                    (row.path.to_string_lossy().into_owned(), row.complete, row.bytes, row.age_ns)
                })
                .collect::<Vec<_>>()
        };
        // Joined natively, as the report joins it: Windows spells this row `env\lib`.
        let env_lib = Path::new("env").join("lib").to_string_lossy().into_owned();
        assert_eq!(
            directories(Selection::default()),
            vec![
                ("env".to_string(), Some(false), 100, None),
                ("docs".to_string(), Some(true), 30, Some(-50)),
                (env_lib, Some(false), 0, None),
            ],
            "sizes are lower bounds and the age is unknown below the boundary"
        );
        for modified in [
            ModifiedWindow { since: None, before: Some(100) },
            ModifiedWindow { since: Some(0), before: None },
        ] {
            assert_eq!(
                directories(Selection { modified, ..Selection::default() }),
                vec![("docs".to_string(), Some(true), 30, Some(-50))],
                "an unknown age satisfies no bound, not even one its lower bound would prove"
            );
        }
        assert_eq!(
            directories(Selection { min_size: Some(100), ..Selection::default() }),
            vec![("env".to_string(), Some(false), 100, None)],
            "a lower bound at or above the minimum proves the true size is too"
        );
        assert!(directories(Selection { min_size: Some(101), ..Selection::default() }).is_empty());
        // A regular file has no subtree to be incomplete, and its own age stands.
        let files = files_of(&run(
            &index,
            &flat(Selection {
                kinds: vec![EntryKind::File],
                modified: ModifiedWindow { since: Some(45), before: None },
                ..Selection::default()
            }),
        ));
        assert_eq!(files.len(), 1);
        assert_eq!((files[0].complete, files[0].age_ns), (None, Some(-50)));
    }

    /// A legacy unscoped partial marker carries no authoritative child-list evidence,
    /// so no directory row can claim completeness. Scoped scan failures preserve healthy
    /// siblings, as the public partial-directory integration test proves.
    #[test]
    fn an_unscoped_partial_marker_marks_every_directory_row_incomplete() {
        let directories = |index: &Index| {
            files_of(&run(
                index,
                &flat(Selection { kinds: vec![EntryKind::Dir], ..Selection::default() }),
            ))
        };
        let mut partial = sample();
        partial.set_initial_freshness(false);
        let rows = directories(&partial);
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(|row| row.complete == Some(false) && row.age_ns.is_none()));
        let mut complete = sample();
        complete.set_initial_freshness(true);
        let rows = directories(&complete);
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(|row| row.complete == Some(true) && row.age_ns.is_some()));
    }

    /// While an opened root is discovering, a directory is complete exactly when
    /// discovery has listed it, so a report served mid-discovery marks the rest.
    #[test]
    fn an_opened_root_marks_a_directory_complete_only_once_discovery_listed_it() {
        let handle = crate::index::IndexHandle::new(Index::new("/root"));
        handle
            .transition_discovery(crate::index::DiscoveryTransition::Begin)
            .expect("begin discovery");
        handle
            .apply(&Observation::new(vec![
                upsert("known", EntryKind::Dir, attrs(0, 5)),
                upsert("pending", EntryKind::Dir, attrs(0, 5)),
            ]))
            .expect("seed directories");
        handle
            .apply_discovery(
                &Observation::new(Vec::new()),
                crate::index::DiscoveryCommit {
                    directory_complete: Some(PathBuf::from("known")),
                    transition: None,
                },
            )
            .expect("list one directory");
        let completeness = handle
            .read_with(|index| {
                files_of(&run(
                    index,
                    &flat(Selection { kinds: vec![EntryKind::Dir], ..Selection::default() }),
                ))
                .into_iter()
                .map(|row| (row.path.to_string_lossy().into_owned(), row.complete))
                .collect::<BTreeMap<_, _>>()
            })
            .expect("read");
        assert_eq!(
            completeness,
            BTreeMap::from([
                ("known".to_string(), Some(true)),
                ("pending".to_string(), Some(false))
            ])
        );
    }

    #[test]
    fn a_filtered_tree_keeps_empty_matches_and_only_folds_visible_directories() {
        let mut index = sample();
        index
            .apply(&Observation::new(vec![upsert("src/empty", EntryKind::Dir, Attrs::default())]))
            .expect("apply");
        let selection = Selection {
            include: vec![pattern("empty")],
            depth: Some(Bound::All),
            min_share: Some(ShareThreshold::parse("0%").expect("valid share")),
            ..Selection::default()
        };
        let root = tree_of(&run(&index, &query(&[ViewSpec::Tree], selection)));
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].name, "src");
        assert_eq!(root.children[0].children[0].name, "empty");
        let selection = Selection {
            include: vec![pattern("notes.txt")],
            depth: Some(Bound::Limit(0)),
            min_share: Some(ShareThreshold::parse("0%").expect("valid share")),
            ..Selection::default()
        };
        let root = tree_of(&run(&index, &query(&[ViewSpec::Tree], selection)));
        assert!(root.truncated, "a file leaf is hidden by depth zero");
        assert_eq!(root.bytes, 7);
    }

    /// One tree row's activity, completeness, and age.
    type RowAge = (Option<i64>, Option<bool>, Option<i128>);

    /// Every row of a tree by path, with its [`RowAge`].
    fn ages_of(root: &TreeNode) -> BTreeMap<PathBuf, RowAge> {
        let mut rows = BTreeMap::new();
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            rows.insert(node.path.clone(), (node.mtime_ns, node.complete, node.age_ns));
            stack.extend(node.children.iter());
        }
        rows
    }

    /// Every bound lifted, so a tree shows every row its selection counts.
    fn whole(selection: Selection) -> Selection {
        Selection {
            depth: Some(Bound::All),
            breadth: Some(Bound::All),
            limit: Some(Bound::All),
            min_share: Some(ShareThreshold::parse("0%").expect("share")),
            ..selection
        }
    }

    /// The sample with a directory and a symlink newer than any file, an empty directory,
    /// and an other entry, beside directories whose own times are older than their files.
    fn active_sample() -> Index {
        let mut index = sample();
        index.apply_ok(&Observation::new(vec![
            upsert("src", EntryKind::Dir, attrs(0, 3)),
            upsert("docs", EntryKind::Dir, attrs(0, 4)),
            upsert("src/deep", EntryKind::Dir, attrs(0, 6)),
            upsert("src/empty", EntryKind::Dir, attrs(0, 60)),
            upsert("docs/link", EntryKind::Symlink, attrs(9, 70)),
            upsert("fifo", EntryKind::Other, attrs(0, 80)),
        ]));
        index.set_initial_freshness(true);
        index
    }

    /// A tree row's age is the newest time among the entries it counts, of any kind and
    /// its own included, measured from the request's instant; the root's own time never
    /// counts. Under a selection that admits everything it is the list row's for the same
    /// directory, and the unfiltered pass and the walk agree on it.
    #[test]
    fn tree_rows_age_their_counted_activity_as_list_rows_do() {
        let index = active_sample();
        let joined = |parts: &[&str]| parts.iter().collect::<PathBuf>();
        let expected = BTreeMap::from([
            (PathBuf::new(), (Some(80), Some(true), Some(-80))),
            (joined(&["src"]), (Some(60), Some(true), Some(-60))),
            (joined(&["src", "deep"]), (Some(40), Some(true), Some(-40))),
            (joined(&["src", "empty"]), (Some(60), Some(true), Some(-60))),
            (joined(&["docs"]), (Some(70), Some(true), Some(-70))),
            (joined(&["notes.txt"]), (Some(5), None, Some(-5))),
            (joined(&["src", "main.rs"]), (Some(10), None, Some(-10))),
            (joined(&["src", "lib.rs"]), (Some(20), None, Some(-20))),
            (joined(&["src", "deep", "nested.rs"]), (Some(40), None, Some(-40))),
            (joined(&["docs", "guide.md"]), (Some(30), None, Some(-30))),
        ]);
        let unfiltered =
            tree_of(&run(&index, &query(&[ViewSpec::Tree], whole(Selection::default()))));
        assert_eq!(ages_of(&unfiltered), expected);
        // A flat files view beside the tree takes the walk, which must agree.
        let walked =
            run(&index, &query(&[ViewSpec::Tree, ViewSpec::Files], whole(Selection::default())));
        assert_eq!(ages_of(&tree_of(&walked)), expected);
        // The list row of each directory carries the same time.
        for row in files_of(&run(
            &index,
            &flat(Selection { kinds: vec![EntryKind::Dir], ..Selection::default() }),
        )) {
            assert_eq!(expected[&row.path].0, Some(row.mtime_ns), "{}", row.path.display());
        }
        // `newest_mtime_ns` keeps its files-only meaning.
        assert_eq!(unfiltered.newest_mtime_ns, Some(40));
        // A root's own time is not activity: an empty root counts nothing and has no age.
        let mut empty = Index::new("/empty");
        empty.set_initial_freshness(true);
        let root = tree_of(&run(&empty, &query(&[ViewSpec::Tree], Selection::default())));
        assert_eq!((root.mtime_ns, root.complete, root.age_ns), (None, Some(true), None));
    }

    /// Under a filter a row ages what it counts: `--kind file` counts no directory's own
    /// time, a window leaves its newest entries out, and a row that counts nothing has no
    /// age at all.
    #[test]
    fn a_filtered_tree_row_ages_only_what_its_selection_counts() {
        let index = active_sample();
        let files = tree_of(&run(
            &index,
            &query(
                &[ViewSpec::Tree],
                whole(Selection { kinds: vec![EntryKind::File], ..Selection::default() }),
            ),
        ));
        let rows = ages_of(&files);
        assert_eq!(rows[Path::new("")].0, Some(40), "the newest file, not the fifo");
        assert_eq!(rows[Path::new("src")].0, Some(40), "not src/empty's own time");
        assert_eq!(rows[Path::new("docs")].0, Some(30), "not the symlink");
        assert!(!rows.contains_key(Path::new("src/empty")), "an empty directory counts nothing");

        let older = tree_of(&run(
            &index,
            &query(
                &[ViewSpec::Tree],
                whole(Selection {
                    modified: crate::query::query_selection::ModifiedWindow {
                        since: None,
                        before: Some(25),
                    },
                    ..Selection::default()
                }),
            ),
        ));
        assert_eq!(older.mtime_ns, Some(20), "src/lib.rs is the newest entry before 25");
        assert_eq!(older.age_ns, Some(-20));

        let nothing = tree_of(&run(
            &index,
            &query(
                &[ViewSpec::Tree],
                whole(Selection { include: vec![pattern("absent")], ..Selection::default() }),
            ),
        ));
        assert_eq!((nothing.mtime_ns, nothing.complete, nothing.age_ns), (None, Some(true), None));
    }

    /// An incomplete subtree keeps its lower-bound activity and has no age; a time after
    /// the reference is a negative age; and `--sort mtime` orders by the activity the age
    /// column shows, so a directory whose newest entry is a symlink sorts by it.
    #[test]
    fn tree_ages_are_unknown_when_incomplete_and_sort_by_activity() {
        let mut bounded = Index::new_with_scope(
            "/root",
            crate::ScanScope { max_depth: Some(2), ..crate::ScanScope::default() },
        );
        bounded.apply_ok(&Observation::new(vec![
            upsert("env", EntryKind::Dir, attrs(0, 5)),
            upsert("env/lib", EntryKind::Dir, attrs(0, 7)),
            upsert("env/a.bin", EntryKind::File, attrs(100, 40)),
            upsert("docs", EntryKind::Dir, attrs(0, 5)),
            upsert("docs/guide.md", EntryKind::File, attrs(30, 50)),
        ]));
        bounded.set_initial_freshness(true);
        for views in [&[ViewSpec::Tree][..], &[ViewSpec::Tree, ViewSpec::Files]] {
            let rows =
                ages_of(&tree_of(&run(&bounded, &query(views, whole(Selection::default())))));
            assert_eq!(rows[Path::new("")], (Some(50), Some(false), None), "{views:?}");
            assert_eq!(rows[Path::new("env")], (Some(40), Some(false), None), "{views:?}");
            assert_eq!(rows[Path::new("docs")], (Some(50), Some(true), Some(-50)), "{views:?}");
        }

        let index = active_sample();
        let mut request = crate::test_support::read_of(
            &index,
            query(&[ViewSpec::Tree], whole(Selection::default())),
        );
        // A `SystemTime` is 100 ns apart on Windows, so the reference sits on a multiple of
        // that: a finer one would be truncated to an earlier instant there.
        request.now = UNIX_EPOCH;
        let root = tree_of(&report(&index, &request, generated_at()).expect("report"));
        assert_eq!(ages_of(&root)[Path::new("docs")].2, Some(-70), "70 is after the epoch");

        let by_activity = tree_of(&run(
            &index,
            &query(
                &[ViewSpec::Tree],
                whole(Selection { sort: Some(SortKey::Mtime), ..Selection::default() }),
            ),
        ));
        assert_eq!(
            by_activity.children.iter().map(|row| row.name.as_str()).collect::<Vec<_>>(),
            ["docs", "src", "notes.txt"],
            "docs's symlink (70) is newer than anything in src (60)"
        );
    }

    #[test]
    fn the_undocumented_docs_view_alias_is_rejected() {
        assert_eq!(
            ViewSpec::parse("documents").expect("the canonical view name parses"),
            ViewSpec::Documents
        );
        assert_eq!(
            ViewSpec::parse("docs").expect_err("an unreleased alias must not become a contract"),
            format!("expected one of {}", ViewSpec::vocabulary())
        );
    }

    #[test]
    fn code_analysis_defaults_to_code_overview_and_keeps_document_projection() {
        assert_eq!(
            ViewSpec::resolve(None, AnalysisSet::NONE.with_code(), "view").expect("default").0,
            vec![ViewSpec::Code]
        );
        assert_eq!(
            ViewSpec::resolve(None, AnalysisSet::ALL, "view").expect("combined default").0,
            vec![ViewSpec::Code, ViewSpec::Documents]
        );
        assert_eq!(
            ViewSpec::resolve(Some("files"), AnalysisSet::ALL, "view").expect("explicit").0,
            vec![ViewSpec::Files]
        );
    }

    /// The two directions between views and analyzers are one table read both ways: a view
    /// that implies an analyzer set is the view that set defaults to, so `--view code` and
    /// `--analyze code` cannot come to mean different reports.
    #[test]
    fn a_view_implies_exactly_the_analyzers_that_choose_it() {
        for view in ViewSpec::ALL {
            let implied = view.implies();
            match view {
                ViewSpec::Code | ViewSpec::Documents => {
                    assert!(implied.is_enabled(), "{view:?} has no metadata meaning");
                    assert_eq!(ViewSpec::default_for(implied), view, "{view:?}");
                }
                _ => assert_eq!(implied, AnalysisSet::NONE, "{view:?} has a metadata meaning"),
            }
        }
        assert_eq!(ViewSpec::Code.implies(), AnalysisSet::CODE_ONLY);
        assert_eq!(ViewSpec::Documents.implies(), AnalysisSet::WORDS_ONLY);
    }

    /// The list grammar parses before any analyzer is known, and only named views imply:
    /// `full` is the metadata digest and the default is chosen by the analyzers.
    #[test]
    fn only_named_content_views_imply_analysis() {
        let implied = |spec| ViewList::parse(spec).expect("parses").implies();
        assert_eq!(implied(Some("code")), AnalysisSet::CODE_ONLY);
        assert_eq!(implied(Some("documents")), AnalysisSet::WORDS_ONLY);
        assert_eq!(implied(Some("tree,code,documents")), AnalysisSet::ALL);
        for spec in [None, Some("full"), Some("languages"), Some("types,families,summary")] {
            assert_eq!(implied(spec), AnalysisSet::NONE, "{spec:?}");
        }
        assert_eq!(
            ViewList::parse(Some("summary,documents,code")).expect("parses").implying(),
            [ViewSpec::Documents, ViewSpec::Code],
            "the caller's order"
        );
    }

    /// `full` keeps a view exactly when the analyzers include what it implies, the test a
    /// read applies to a view its caller named.
    #[test]
    fn full_keeps_a_view_only_when_its_analyzers_ran() {
        let omitted = |content| ViewSpec::full_report(content).1;
        assert_eq!(omitted(AnalysisSet::NONE), [ViewSpec::Code, ViewSpec::Documents]);
        assert_eq!(omitted(AnalysisSet::LINES_ONLY), [ViewSpec::Code, ViewSpec::Documents]);
        assert_eq!(omitted(AnalysisSet::CODE_ONLY), [ViewSpec::Documents]);
        assert_eq!(omitted(AnalysisSet::WORDS_ONLY), [ViewSpec::Code]);
        assert!(omitted(AnalysisSet::ALL).is_empty());
    }

    fn generated_at() -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(1_001)
    }

    fn run(index: &Index, query: &Query) -> Report {
        report(index, &crate::test_support::read_of(index, query.clone()), generated_at())
            .expect("the query is answerable over this index")
    }

    fn query(views: &[ViewSpec], selection: Selection) -> Query {
        Query { selection, views: views.to_vec(), ..Query::default() }
    }

    /// A flat List: the projection whose rows carry subtree metrics and completeness.
    fn flat(selection: Selection) -> Query {
        Query {
            selection,
            views: vec![ViewSpec::List],
            format: crate::report_format::Format::Paths,
            ..Query::default()
        }
    }

    fn pattern(source: &str) -> Pattern {
        Pattern::parse(source).expect("pattern compiles")
    }

    fn summary_of(report: &Report) -> SummaryRow {
        match report.sections.first().expect("a section") {
            Section::Summary(row) => *row,
            other => panic!("expected a summary, got {other:?}"),
        }
    }

    fn files_of(report: &Report) -> Vec<FileRow> {
        match report.sections.first().expect("a section") {
            Section::Files { rows, .. } => rows.clone(),
            other => panic!("expected files, got {other:?}"),
        }
    }

    fn types_of(report: &Report) -> Vec<TypeRow> {
        match report.sections.first().expect("a section") {
            Section::Extensions { rows, .. } => rows.clone(),
            other => panic!("expected types, got {other:?}"),
        }
    }

    fn tree_of(report: &Report) -> TreeNode {
        match report.sections.first().expect("a section") {
            Section::Tree { root: Some(node), .. } => (**node).clone(),
            other => panic!("expected a tree, got {other:?}"),
        }
    }

    #[test]
    fn an_unfiltered_summary_matches_the_precomputed_rollup() {
        let index = sample();
        let row = summary_of(&run(&index, &query(&[ViewSpec::Summary], Selection::default())));
        assert_eq!(row.files, 5);
        assert_eq!(row.dirs, 3);
        assert_eq!(row.bytes, 657);
        assert_eq!(row.newest_mtime_ns, Some(40));
    }

    #[test]
    fn the_two_tiers_agree_on_the_same_question() {
        // The load-bearing property: reading pre-computed roll-ups and re-aggregating a
        // filtered walk must answer identically when the filter admits everything.
        let index = sample();
        let fast = summary_of(&run(&index, &query(&[ViewSpec::Summary], Selection::default())));

        // A filter that excludes nothing still forces the traversal tier.
        let admits_everything = Selection { min_size: Some(0), ..Selection::default() };
        assert!(!admits_everything.is_unfiltered());
        let slow = summary_of(&run(&index, &query(&[ViewSpec::Summary], admits_everything)));

        // `dirs` belongs in this comparison like every other tally. It used to be left
        // out because the two tiers genuinely disagreed: the traversal tier counted every
        // directory it descended into, so a filter admitting everything was the only
        // filter the two tiers could agree under.
        assert_eq!(
            (fast.files, fast.dirs, fast.bytes, fast.allocated, fast.newest_mtime_ns),
            (slow.files, slow.dirs, slow.bytes, slow.allocated, slow.newest_mtime_ns)
        );
    }

    #[test]
    fn extension_rows_account_for_every_file_in_both_tiers() {
        // The rows are a partition of the tree, not a selection from it, so they have to
        // sum to what the summary reports. They did not: a name with no extension was
        // dropped from the roll-up rather than bucketed, so a 657-byte tree came back as
        // rows totalling less and nothing in the output said which files were missing.
        let mut index = sample();
        index
            .apply(&Observation::new(vec![
                upsert("Makefile", EntryKind::File, attrs(28, 50)),
                upsert(".gitignore", EntryKind::File, attrs(11, 51)),
            ]))
            .expect("apply");

        // Both tiers: unfiltered reads the pre-computed roll-up, and any filter at all
        // forces the traversal to re-aggregate. They are separate code paths.
        for selection in [
            Selection::default(),
            Selection { min_size: Some(0), ..Selection::default() },
            Selection { kinds: vec![EntryKind::File], ..Selection::default() },
        ] {
            let rows = types_of(&run(&index, &query(&[ViewSpec::Extensions], selection.clone())));
            let summary = summary_of(&run(&index, &query(&[ViewSpec::Summary], selection.clone())));
            assert_eq!(
                rows.iter().map(|row| row.bytes).sum::<u64>(),
                summary.bytes,
                "bytes unaccounted for under {selection:?}: {rows:?}"
            );
            assert_eq!(
                rows.iter().map(|row| row.files).sum::<u64>(),
                summary.files,
                "files unaccounted for under {selection:?}: {rows:?}"
            );
        }
    }

    #[test]
    fn names_without_an_extension_share_one_bucket() {
        // `Makefile` and `.gitignore` have nothing in common as names, and inventing a
        // row per such name would turn the view into a file listing. One bucket keeps it
        // a roll-up while still accounting for the bytes.
        let mut index = sample();
        index
            .apply(&Observation::new(vec![
                upsert("Makefile", EntryKind::File, attrs(28, 50)),
                upsert(".gitignore", EntryKind::File, attrs(11, 51)),
            ]))
            .expect("apply");

        let rows = types_of(&run(&index, &query(&[ViewSpec::Extensions], Selection::default())));
        let bucket = rows
            .iter()
            .find(|row| row.extension == crate::classify::NO_EXTENSION)
            .expect("a bucket for the extension-less names");
        assert_eq!(bucket.files, 2);
        assert_eq!(bucket.bytes, 39);
        // And it never swallows a name that does have one.
        assert!(rows.iter().any(|row| row.extension == ".rs"), "{rows:?}");
    }

    #[test]
    fn a_summary_counts_the_union_of_listed_entries_and_directory_contents() {
        // One query must not give two answers. The directory tally is folded from the
        // walk while the files view is filtered entry by entry, so they are two paths to
        // the same number and drifted apart: `--kind file` answered "5 files, 3
        // directories" while the files view under the same selection listed no directory
        // at all.
        let index = sample();
        for selection in [
            Selection { kinds: vec![EntryKind::File], ..Selection::default() },
            Selection { kinds: vec![EntryKind::Dir], ..Selection::default() },
            Selection { include: vec![pattern("*.rs")], ..Selection::default() },
            Selection { exclude: vec![pattern("docs")], ..Selection::default() },
            Selection { min_size: Some(1_000_000), ..Selection::default() },
            Selection { min_size: Some(0), ..Selection::default() },
        ] {
            let summary = summary_of(&run(&index, &query(&[ViewSpec::Summary], selection.clone())));
            let listed = files_of(&run(&index, &query(&[ViewSpec::Files], selection.clone())));
            let dirs = listed.iter().filter(|row| row.kind == EntryKind::Dir).count() as u64;
            let files = every_entry(&index)
                .iter()
                .filter(|entry| {
                    entry.kind == EntryKind::File
                        && listed.iter().any(|row| {
                            entry.path == row.path
                                || (row.kind == EntryKind::Dir && entry.path.starts_with(&row.path))
                        })
                })
                .count() as u64;
            assert_eq!(summary.dirs, dirs, "directory counts disagree under {selection:?}");
            assert_eq!(summary.files, files, "file counts disagree under {selection:?}");
        }
    }

    #[test]
    fn a_rejected_directory_is_still_descended_into() {
        // Filtering a directory out of the tally must not filter out what is under it:
        // `--kind file` reports no directories and every file, at every depth.
        let index = sample();
        let selection = Selection { kinds: vec![EntryKind::File], ..Selection::default() };
        let row = summary_of(&run(&index, &query(&[ViewSpec::Summary], selection)));
        assert_eq!(row.dirs, 0, "no directory was admitted");
        assert_eq!(row.files, 5, "including src/deep/nested.rs, two levels down");
        assert_eq!(row.bytes, 657, "and its bytes");
    }

    #[test]
    fn nested_directory_counts_roll_up_through_every_level() {
        // The tally is taken in the pre-order pass and folded in the post-order one, so a
        // directory admitted three levels down has to reach the root through both.
        let index = sample();
        let selection = Selection { kinds: vec![EntryKind::Dir], ..Selection::default() };
        let root = tree_of(&run(&index, &query(&[ViewSpec::Tree], selection)));
        assert_eq!(root.dirs, 3, "src, src/deep, and docs");
        assert_eq!(root.files, 4, "matching directories cover their regular files");
        let src = root.children.iter().find(|node| node.name == "src").expect("src");
        assert_eq!(src.dirs, 1, "src/deep, counted for src as well as for the root");
    }

    #[test]
    fn selection_narrows_a_summary_to_what_it_admits() {
        let index = sample();
        let selection = Selection { include: vec![pattern("*.rs")], ..Selection::default() };
        let row = summary_of(&run(&index, &query(&[ViewSpec::Summary], selection)));
        assert_eq!(row.files, 3, "three .rs files");
        assert_eq!(row.bytes, 350);
    }

    #[test]
    fn a_files_view_lists_matching_entries_in_name_order_by_default() {
        let index = sample();
        let selection = Selection { include: vec![pattern("*.rs")], ..Selection::default() };
        let rows = files_of(&run(&index, &query(&[ViewSpec::Files], selection)));
        // Built from components so the expectation carries the native separator: a
        // literal "src/main.rs" passes on Unix and fails on Windows for a reason that
        // has nothing to do with the view under test.
        let paths: Vec<PathBuf> = rows.iter().map(|row| row.path.clone()).collect();
        let expected: Vec<PathBuf> = [["src", "deep", "nested.rs"].iter().collect::<PathBuf>()]
            .into_iter()
            .chain([["src", "lib.rs"].iter().collect::<PathBuf>()])
            .chain([["src", "main.rs"].iter().collect::<PathBuf>()])
            .collect();
        assert_eq!(paths, expected);
    }

    #[test]
    fn sorting_and_limiting_compose_without_a_dedicated_view() {
        // "Largest files" is not a view; it is files plus sort plus limit.
        let index = sample();
        // Apparent, because the sample's allocated sizes round to 512-byte blocks and tie.
        let selection = Selection {
            kinds: vec![EntryKind::File],
            sort: Some(SortKey::Size),
            limit: Some(Bound::Limit(2)),
            size: SizeMetric::Apparent,
            ..Selection::default()
        };
        let rows = files_of(&run(&index, &query(&[ViewSpec::Files], selection)));
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].bytes, 300, "largest first");
        assert_eq!(rows[1].bytes, 200);
    }

    #[test]
    fn reverse_flips_whatever_order_is_in_effect() {
        let index = sample();
        let selection = Selection {
            kinds: vec![EntryKind::File],
            sort: Some(SortKey::Size),
            reverse: true,
            size: SizeMetric::Apparent,
            ..Selection::default()
        };
        let rows = files_of(&run(&index, &query(&[ViewSpec::Files], selection)));
        assert_eq!(rows[0].bytes, 7, "smallest first once reversed");
    }

    #[test]
    fn a_modified_window_selects_by_time() {
        let index = sample();
        let selection = Selection {
            kinds: vec![EntryKind::File],
            modified: ModifiedWindow { since: Some(20), before: Some(40) },
            sort: Some(SortKey::Mtime),
            ..Selection::default()
        };
        let rows = files_of(&run(&index, &query(&[ViewSpec::Files], selection)));
        let mut times: Vec<i64> = rows.iter().map(|row| row.mtime_ns).collect();
        times.sort_unstable();
        assert_eq!(times, vec![20, 30], "inclusive start, exclusive end");
    }

    #[test]
    fn a_types_view_reports_both_size_metrics_per_extension() {
        let index = sample();
        let rows = types_of(&run(&index, &query(&[ViewSpec::Extensions], Selection::default())));
        let rs = rows.iter().find(|row| row.extension == ".rs").expect(".rs present");
        assert_eq!((rs.files, rs.bytes), (3, 350));
        assert_eq!(rs.allocated, 1536, "three files, one 512-byte block each");
        // Size-ranked by default: .rs (350) then .md (300) then .txt (7).
        let order: Vec<&str> = rows.iter().map(|row| row.extension.as_str()).collect();
        assert_eq!(order, vec![".rs", ".md", ".txt"]);
    }

    #[test]
    fn a_tree_view_reports_directories_with_their_subtree_totals() {
        let index = sample();
        let tree = tree_of(&run(&index, &query(&[ViewSpec::Tree], Selection::default())));
        assert_eq!(tree.name, ".");
        assert_eq!(tree.bytes, 657);
        // Size-ranked children: src (350) before docs (300).
        let names: Vec<&str> = tree.children.iter().map(|child| child.name.as_str()).collect();
        assert_eq!(names, vec!["src", "docs", "notes.txt"]);
        let src = &tree.children[0];
        assert_eq!(src.bytes, 350);
        let nested: Vec<&str> = src.children.iter().map(|child| child.name.as_str()).collect();
        assert!(nested.contains(&"deep"));
    }

    #[test]
    fn depth_zero_keeps_dus_meaning_of_root_totals_only() {
        let index = sample();
        let selection = Selection { depth: Some(Bound::Limit(0)), ..Selection::default() };
        let tree = tree_of(&run(&index, &query(&[ViewSpec::Tree], selection)));
        assert_eq!(tree.bytes, 657, "totals still cover the whole tree");
        assert!(tree.children.is_empty(), "but nothing below the root is listed");
        assert!(tree.truncated, "and the report says so rather than implying emptiness");
    }

    /// A view `full` had to drop is named on the report, so every surface says so.
    ///
    /// This lived in the CLI, which meant a caller reaching the same wall through the
    /// library got a report quietly missing a section and no way to learn why (fdu-x8u6).
    #[test]
    fn a_dropped_view_is_named_on_the_report_rather_than_by_one_surface() {
        let (selected, omitted) = ViewSpec::resolve(Some("full"), AnalysisSet::NONE, "view")
            .expect("full resolves without analyzers");
        assert!(omitted.contains(&ViewSpec::Code));
        assert!(omitted.contains(&ViewSpec::Documents));

        let query = Query { views: selected, omitted_views: omitted, ..Query::default() };
        let (notes, tips) =
            display_notes(&query, AnalysisSet::NONE, &ControlCoverage::NotObserved, None);
        assert_eq!(notes, ["note: full omits code, documents without analysis"]);
        assert_eq!(tips, ["tip: include them: analyze all"]);

        // Nothing dropped, nothing said.
        let (selected, omitted) = ViewSpec::resolve(Some("full"), AnalysisSet::ALL, "view")
            .expect("full resolves with analyzers");
        assert!(omitted.is_empty(), "every view is answerable with analysis enabled");
        let query = Query { views: selected, omitted_views: omitted, ..Query::default() };
        assert!(
            display_notes(&query, AnalysisSet::ALL, &ControlCoverage::NotObserved, None)
                .0
                .is_empty()
        );
    }

    /// `full` implies nothing, so under one analyzer it drops the view that needs the
    /// other. The note names the analyzer that is missing, and the tip names the one value
    /// that keeps what ran and adds what is missing -- naming only the missing analyzer
    /// would be a command that loses the one already chosen.
    #[test]
    fn full_names_the_missing_analyzer_and_a_value_that_keeps_the_present_one() {
        for (content, omitted, note) in [
            (
                AnalysisSet::LINES_ONLY,
                "code, documents",
                "note: full omits code, documents without code and words analysis",
            ),
            (
                AnalysisSet::CODE_ONLY,
                "documents",
                "note: full omits documents without words analysis",
            ),
            (AnalysisSet::WORDS_ONLY, "code", "note: full omits code without code analysis"),
        ] {
            let (selected, dropped) =
                ViewSpec::resolve(Some("full"), content, "view").expect("full resolves");
            let labels: Vec<&str> = dropped.iter().map(|view| view.label()).collect();
            assert_eq!(labels.join(", "), omitted, "{content:?}");
            let query = Query {
                views: selected,
                omitted_views: dropped,
                axes: &AxisNames::FLAGS,
                ..Query::default()
            };
            let (notes, tips) = display_notes(&query, content, &ControlCoverage::NotObserved, None);
            assert_eq!(notes, [note], "{content:?}");
            assert_eq!(tips, ["tip: include them: --analyze all"], "{content:?}");
        }
    }

    /// Analysis no selected view displays is named with the views where it would show:
    /// the default for the analyzers, which is the view each implies.
    #[test]
    fn analysis_no_view_shows_names_the_view_that_would_show_it() {
        for (content, views, note, tip) in [
            (
                AnalysisSet::CODE_ONLY,
                vec![ViewSpec::Summary],
                "note: code analysis not shown by summary",
                "tip: show it: --view code",
            ),
            (
                AnalysisSet::ALL,
                vec![ViewSpec::Tree],
                "note: code and words analysis not shown by tree",
                "tip: show it: --view code,documents",
            ),
            (
                AnalysisSet::LINES_ONLY,
                vec![ViewSpec::Summary, ViewSpec::Files],
                "note: lines analysis not shown by summary, files",
                "tip: show it: --view families",
            ),
            (
                AnalysisSet::WORDS_ONLY,
                vec![ViewSpec::List],
                "note: words analysis not shown by list",
                "tip: show it: --view documents",
            ),
        ] {
            let query = Query { views, axes: &AxisNames::FLAGS, ..Query::default() };
            let (notes, tips) = display_notes(&query, content, &ControlCoverage::NotObserved, None);
            assert_eq!((notes, tips), (vec![note.to_owned()], vec![tip.to_owned()]));
        }

        // A view that displays analysis, or a ranking by one of its metrics, says nothing.
        let shown = Query { views: vec![ViewSpec::Languages], ..Query::default() };
        let (notes, tips) =
            display_notes(&shown, AnalysisSet::CODE_ONLY, &ControlCoverage::NotObserved, None);
        assert!(notes.is_empty() && tips.is_empty(), "{notes:?} {tips:?}");
        let ranked = Query {
            views: vec![ViewSpec::Files],
            selection: Selection {
                sort: Some(SortKey::Metric("code_lines")),
                ..Selection::default()
            },
            ..Query::default()
        };
        let (notes, _) =
            display_notes(&ranked, AnalysisSet::CODE_ONLY, &ControlCoverage::NotObserved, None);
        assert!(notes.is_empty(), "{notes:?}");
    }

    /// Analysis some selected view displays and some does not is named analyzer by
    /// analyzer, and the tip keeps the caller's views, adding the ones that show the rest.
    #[test]
    fn analysis_partly_shown_names_what_no_view_displays() {
        for (content, views, sort, note, tip) in [
            (
                AnalysisSet::ALL,
                vec![ViewSpec::Documents],
                None,
                "note: code analysis not shown by documents",
                "tip: show it: --view documents,code",
            ),
            (
                AnalysisSet::ALL,
                vec![ViewSpec::Summary, ViewSpec::Code],
                None,
                "note: words analysis not shown by summary, code",
                "tip: show it: --view summary,code,documents",
            ),
            // A ranking displays its own analyzer and nothing else.
            (
                AnalysisSet::ALL,
                vec![ViewSpec::Files],
                Some(SortKey::Metric("code_lines")),
                "note: words analysis not shown by files",
                "tip: show it: --view files,documents",
            ),
        ] {
            let query = Query {
                views,
                selection: Selection { sort, ..Selection::default() },
                axes: &AxisNames::FLAGS,
                ..Query::default()
            };
            let (notes, tips) = display_notes(&query, content, &ControlCoverage::NotObserved, None);
            assert_eq!((notes, tips), (vec![note.to_owned()], vec![tip.to_owned()]));
        }

        // The grouping views show every analyzer, so beside one nothing goes unremarked.
        for view in [ViewSpec::Types, ViewSpec::Families, ViewSpec::Languages] {
            let query = Query { views: vec![view], ..Query::default() };
            let (notes, tips) =
                display_notes(&query, AnalysisSet::ALL, &ControlCoverage::NotObserved, None);
            assert!(notes.is_empty() && tips.is_empty(), "{view:?}: {notes:?} {tips:?}");
        }
    }

    /// The display table agrees with the implication table and the defaults: each view
    /// shows what it implies, and the default views for any analyzer set show all of it,
    /// so a request that names no view is never told its analysis went unshown.
    #[test]
    fn every_view_shows_what_it_implies_and_the_defaults_show_everything() {
        for view in ViewSpec::ALL {
            assert!(view.shows().contains(view.implies()), "{view:?}");
        }
        for content in [
            AnalysisSet::LINES_ONLY,
            AnalysisSet::CODE_ONLY,
            AnalysisSet::WORDS_ONLY,
            AnalysisSet::ALL,
        ] {
            let shown = ViewSpec::defaults_for(content)
                .into_iter()
                .fold(AnalysisSet::NONE, |set, view| set.union(view.shows()));
            assert!(shown.contains(content), "{content:?}");
            let query = Query { views: ViewSpec::defaults_for(content), ..Query::default() };
            let (notes, _) = display_notes(&query, content, &ControlCoverage::NotObserved, None);
            assert!(notes.is_empty(), "{content:?}: {notes:?}");
        }
    }

    /// A rule belongs to the library; the words a caller can act on belong to their
    /// surface. Both diagnostics name two axes, and naming them with flags told a Python
    /// caller to add an `--analyze` their surface does not have (fdu-4apt).
    ///
    /// Asserted as "names mine, never the other's" rather than by quoting either sentence,
    /// so rewording the rule cannot break this and changing the vocabulary cannot pass it.
    /// The refused-controls note names a few directories and counts the rest, breaks the
    /// reasons down only when every refusal is listed, and raises exactly the limits that
    /// fired, each by its own knob.
    #[test]
    fn the_refused_controls_note_bounds_its_list_and_matches_its_remedy_to_the_reasons() {
        use crate::control::{
            ControlLimits, ControlObservation, ControlRefusalReason, RefusedControl,
        };
        let defaults = ControlLimits::default();
        for (reason, wanted, unwanted) in [
            (ControlRefusalReason::Budget, "--gitignore-budget", "--gitignore-line-limit"),
            (ControlRefusalReason::LineLimit, "--gitignore-line-limit", "--gitignore-budget"),
        ] {
            let coverage = ControlCoverage::Observed(ControlObservation {
                limits: defaults,
                applied: 7,
                rules: 0,
                refused: 1,
                refusals: vec![RefusedControl {
                    root: 0,
                    path: Path::new("vendor").join(".gitignore"),
                    reason,
                }],
            });
            let (note, tip) =
                refused_controls_note(&coverage, &AxisNames::FLAGS, None).expect("refusal");
            assert!(
                note.starts_with(
                    "note: ignore classification incomplete: 1 ignore file not applied"
                )
            );
            assert!(note.contains("vendor"));
            assert!(!note.contains("--"), "facts do not repeat flag advice");
            let tip = tip.expect("bounded refusal has a remedy");
            assert!(tip.starts_with("tip:"));
            assert!(tip.contains(wanted));
            assert!(!tip.contains(unwanted));
        }
        let coverage = |limits, refused| {
            ControlCoverage::Observed(ControlObservation {
                limits,
                applied: 7,
                rules: 0,
                refused,
                refusals: (0..crate::MAX_RETAINED_ISSUES)
                    .map(|i| RefusedControl {
                        root: 0,
                        path: Path::new(&format!("d{i:02}")).join(".gitignore"),
                        reason: ControlRefusalReason::Budget,
                    })
                    .collect(),
            })
        };
        let (note, tip) = refused_controls_note(&coverage(defaults, 1000), &AxisNames::FLAGS, None)
            .expect("truncated");
        assert!(note.contains("995 more"));
        assert!(!note.contains("d05"));
        let tip = tip.expect("both bounded limits may explain unlisted refusals");
        assert!(tip.contains("--gitignore-budget") && tip.contains("--gitignore-line-limit"));
        let (_, tip) = refused_controls_note(
            &coverage(ControlLimits { line_limit: None, ..defaults }, 1000),
            &AxisNames::FLAGS,
            None,
        )
        .expect("truncated");
        assert!(!tip.expect("budget remedy").contains("--gitignore-line-limit"));
        let (_, tip) = refused_controls_note(
            &coverage(ControlLimits { budget: None, line_limit: None }, 64),
            &AxisNames::FLAGS,
            None,
        )
        .expect("retained refusal");
        assert!(tip.is_none(), "do not suggest raising unbounded limits");
        assert!(
            refused_controls_note(&ControlCoverage::NotObserved, &AxisNames::FLAGS, None).is_none()
        );
    }

    #[test]
    fn a_diagnostic_names_the_axes_the_requesting_surface_uses() {
        let (selected, omitted) = ViewSpec::resolve(Some("full"), AnalysisSet::NONE, "view")
            .expect("full resolves without analyzers");

        for (axes, mine, theirs) in [
            (&AxisNames::FLAGS, "--analyze", "analyze"),
            (&AxisNames::FIELDS, "analyze", "--analyze"),
        ] {
            let query = Query {
                views: selected.clone(),
                omitted_views: omitted.clone(),
                axes,
                ..Query::default()
            };
            // Anchored on the whole phrase, because `--analyze` contains `analyze`: a bare
            // `contains` for the other surface's spelling matches its own. That is the same
            // tokenisation trap the watch-scope substitution had to avoid.
            let tip = display_notes(&query, AnalysisSet::NONE, &ControlCoverage::NotObserved, None)
                .1
                .remove(0);
            assert!(tip.contains(&format!(": {mine} ")), "{tip} must name {mine}");
            assert!(!tip.contains(&format!(": {theirs} ")), "{tip} must not name {theirs}");
        }

        // The same for the hard error, which names the view axis as well. The rule is the
        // request model's; what a surface reads is this rendering of it.
        for (axes, view, analyze) in
            [(&AxisNames::FLAGS, "--view", "--analyze"), (&AxisNames::FIELDS, "view", "analyze")]
        {
            let error = crate::query::RequestError::ViewNeedsAnalyzer {
                view: ViewSpec::Documents,
                held: AnalysisSet::NONE,
                holder: crate::query::BasisHolder::Index,
            }
            .message(axes);
            assert!(error.starts_with(&format!("{view} documents")), "{error}");
            assert!(error.contains(&format!("with {analyze} ")), "{error}");
            let theirs = if analyze == "--analyze" { "analyze" } else { "--analyze" };
            assert!(!error.contains(&format!("with {theirs} ")), "{error}");
        }
    }

    /// A library caller who never says otherwise is not the command line, so the default
    /// vocabulary is the one their surface uses.
    #[test]
    fn the_default_vocabulary_is_the_librarys_own() {
        assert_eq!(*Query::default().axes, AxisNames::FIELDS);
    }

    #[test]
    fn a_depth_bound_marks_hidden_file_and_directory_rows_as_truncated() {
        let index = sample();
        let selection = Selection { depth: Some(Bound::Limit(1)), ..Selection::default() };
        let tree = tree_of(&run(&index, &query(&[ViewSpec::Tree], selection)));

        let src = tree.children.iter().find(|child| child.name == "src").expect("src");
        assert!(src.truncated, "src with a hidden directory child is truncated");

        let docs = tree.children.iter().find(|child| child.name == "docs").expect("docs");
        assert!(docs.children.is_empty());
        assert!(docs.truncated, "significant file leaves are rows beyond the depth boundary");
    }

    #[test]
    fn a_tree_limit_caps_section_rows_including_the_root() {
        let index = sample();
        let selection = Selection { limit: Some(Bound::Limit(1)), ..Selection::default() };
        let tree = tree_of(&run(&index, &query(&[ViewSpec::Tree], selection)));
        assert_eq!(tree.children.len(), 0);
        assert!(tree.truncated);
        assert!(tree.omissions.iter().any(|omission| omission.reason == TreeOmissionReason::Rows));
    }

    #[test]
    fn tree_share_is_root_relative_inclusive_and_does_not_hide_eleventh_sibling() {
        let mut index = Index::new("/root");
        let mut ops = Vec::new();
        for number in 0..11 {
            let directory = format!("d{number:02}");
            ops.push(upsert(&directory, EntryKind::Dir, Attrs::default()));
            ops.push(upsert(&format!("{directory}/file"), EntryKind::File, attrs(20, 1)));
        }
        ops.push(upsert("filler", EntryKind::File, attrs(780, 1)));
        index.apply(&Observation::new(ops)).expect("apply");
        let selection = Selection { size: SizeMetric::Apparent, ..Selection::default() };
        let root = tree_of(&run(&index, &query(&[ViewSpec::Tree], selection)));
        assert_eq!(root.children.len(), 12, "eleven 2% directories plus the large file");
        assert!(root.children.iter().any(|child| child.name == "d10"));
        assert!(root.omissions.is_empty());

        let selection = Selection {
            size: SizeMetric::Apparent,
            breadth: Some(Bound::Limit(10)),
            ..Selection::default()
        };
        let root = tree_of(&run(&index, &query(&[ViewSpec::Tree], selection)));
        assert_eq!(root.children.len(), 10);
        assert_eq!(root.omissions[0].reason, TreeOmissionReason::Breadth);
        assert_eq!(root.omissions[0].entries, 2);
        assert_eq!(root.omissions[0].bytes, Some(40));
    }

    #[test]
    fn tree_threshold_keeps_exact_one_percent_file_leaf() {
        let mut index = Index::new("/root");
        index
            .apply(&Observation::new(vec![
                upsert("one-percent", EntryKind::File, attrs(10, 1)),
                upsert("below", EntryKind::File, attrs(9, 1)),
                upsert("rest", EntryKind::File, attrs(981, 1)),
            ]))
            .expect("apply");
        let root = tree_of(&run(
            &index,
            &query(
                &[ViewSpec::Tree],
                Selection { size: SizeMetric::Apparent, ..Selection::default() },
            ),
        ));
        assert_eq!(root.children.len(), 2);
        assert!(
            root.children
                .iter()
                .any(|child| child.name == "one-percent" && child.kind == EntryKind::File)
        );
        assert_eq!(root.omissions[0].reason, TreeOmissionReason::Share);
        assert_eq!(root.omissions[0].bytes, Some(9));
    }

    /// Each display boundary hides a disjoint subtree. The breadth omission covers
    /// two files under one directory, so omitted child roots cannot stand in for files.
    fn mixed_remainder_index() -> Index {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            upsert("A", EntryKind::Dir, attrs(0, 0)),
            upsert("A/a1", EntryKind::Dir, attrs(0, 0)),
            upsert("A/a1/x", EntryKind::Dir, attrs(0, 0)),
            upsert("A/a1/x/leaf", EntryKind::File, attrs(1960, 0)),
            upsert("A/a1/tiny", EntryKind::File, attrs(40, 0)),
            upsert("A/a2", EntryKind::Dir, attrs(0, 0)),
            upsert("A/a2/file", EntryKind::File, attrs(2000, 0)),
            upsert("B", EntryKind::Dir, attrs(0, 0)),
            upsert("B/b1", EntryKind::Dir, attrs(0, 0)),
            upsert("B/b1/file", EntryKind::File, attrs(2000, 0)),
            upsert("B/b2", EntryKind::Dir, attrs(0, 0)),
            upsert("B/b2/file", EntryKind::File, attrs(2000, 0)),
            upsert("C", EntryKind::Dir, attrs(0, 0)),
            upsert("C/c1", EntryKind::Dir, attrs(0, 0)),
            upsert("C/c1/first", EntryKind::File, attrs(1000, 0)),
            upsert("C/c1/second", EntryKind::File, attrs(1000, 0)),
        ]));
        index
    }

    #[test]
    fn nested_mixed_omissions_have_one_exact_remainder_golden() {
        let index = mixed_remainder_index();
        let selection = Selection {
            size: SizeMetric::Apparent,
            depth: Some(Bound::Limit(3)),
            breadth: Some(Bound::Limit(2)),
            limit: Some(Bound::Limit(8)),
            min_share: Some(ShareThreshold::parse("0.5%").expect("share")),
            ..Selection::default()
        };
        let report = run(&index, &query(&[ViewSpec::Tree], selection));
        let Section::Tree { root: Some(root), omissions, .. } = &report.sections[0] else {
            panic!("expected tree")
        };
        let remainder = TreeRemainder::from_tree(Some(root), omissions).expect("hidden content");
        assert_eq!((root.files, root.bytes, root.allocated), (7, 10_000, 10_752));
        assert_eq!(
            (remainder.files, remainder.bytes, remainder.allocated),
            (Some(2), Some(2_000), Some(2_048))
        );
        assert_eq!(remainder.reasons, vec![TreeOmissionReason::Breadth]);
        assert_eq!(
            crate::report_format::render(&report, crate::report_format::Format::Text, false)
                .expect("render"),
            include_str!("../../tests/golden/remainder-tree.txt"),
        );
    }

    #[test]
    fn displayed_directories_represent_their_descendants_even_when_expansion_stops() {
        let index = mixed_remainder_index();
        let selection = Selection {
            size: SizeMetric::Apparent,
            depth: Some(Bound::Limit(1)),
            breadth: Some(Bound::All),
            limit: Some(Bound::All),
            min_share: Some(ShareThreshold::parse("0%").expect("share")),
            ..Selection::default()
        };
        let report = run(&index, &query(&[ViewSpec::Tree], selection));
        let Section::Tree { root: Some(root), omissions, .. } = &report.sections[0] else {
            panic!("expected tree")
        };
        assert_eq!((root.files, root.bytes), (7, 10_000));
        assert_eq!(root.children.len(), 3);
        assert!(root.children.iter().all(|child| child.kind == EntryKind::Dir));
        assert!(root.children.iter().any(|child| !child.omissions.is_empty()));
        assert!(TreeRemainder::from_tree(Some(root), omissions).is_none());
        let diagnostics = crate::report_format::diagnostic_lines(&report);
        assert!(!diagnostics.notes.iter().any(|note| note.contains("… and more")));
        assert!(diagnostics.notes.iter().any(|note| note.contains("depth 1")));
        assert!(diagnostics.tips.contains(&format!("tip: show more: {}=all", report.axes.depth)));

        let depth_zero = run(
            &index,
            &query(
                &[ViewSpec::Tree],
                Selection {
                    size: SizeMetric::Apparent,
                    depth: Some(Bound::Limit(0)),
                    min_share: Some(ShareThreshold::parse("0%").expect("share")),
                    ..Selection::default()
                },
            ),
        );
        let Section::Tree { root: Some(root), omissions, .. } = &depth_zero.sections[0] else {
            panic!("expected tree")
        };
        let remainder = TreeRemainder::from_tree(Some(root), omissions).expect("root alone");
        assert_eq!((remainder.files, remainder.bytes), (Some(7), Some(10_000)));
    }

    #[test]
    fn selected_leaf_ledger_conserves_every_top_level_tree_partition() {
        // This ledger is independent of the index rollups and omission records. A listed
        // direct child represents all selected leaves below it, whether or not its own
        // children were expanded. The root is context, not a second represented set.
        let files = [
            ("src/main.rs", 100, 512, false),
            ("src/lib.rs", 200, 512, false),
            ("src/debug.log", 25, 512, true),
            ("docs/guide.md", 300, 512, false),
            ("build/cache/out.bin", 1_000, 1_024, true),
        ];
        let dirs = [("src", false), ("docs", false), ("build", true), ("build/cache", true)];
        let index = classified_sample();
        let within = |path: &str, parent: &Path| {
            parent.as_os_str().is_empty() || Path::new(path).starts_with(parent)
        };
        for (population, excluded) in [
            (IgnoredEntries::Include, None),
            (IgnoredEntries::Exclude, None),
            (IgnoredEntries::Only, None),
            (IgnoredEntries::Include, Some("build")),
            (IgnoredEntries::Include, Some("*.rs")),
        ] {
            let admitted = |path: &str, ignored: bool| {
                population.admits(ignored)
                    && !matches!(excluded, Some("build") if Path::new(path).starts_with("build"))
                    && !matches!(excluded, Some("*.rs") if Path::new(path)
                        .extension()
                        .is_some_and(|extension| extension.eq_ignore_ascii_case("rs")))
            };
            let selected_files: Vec<_> = files
                .iter()
                .copied()
                .filter(|(path, _, _, ignored)| admitted(path, *ignored))
                .collect();
            let selected_dirs: Vec<_> =
                dirs.iter().copied().filter(|(path, ignored)| admitted(path, *ignored)).collect();
            for size in [SizeMetric::Apparent, SizeMetric::Allocated] {
                for share in ["0%", "1%", "50%", "100%"] {
                    for depth in [Bound::Limit(0), Bound::Limit(1), Bound::All] {
                        for breadth in
                            [Bound::Limit(0), Bound::Limit(1), Bound::Limit(2), Bound::All]
                        {
                            for limit in
                                [Bound::Limit(0), Bound::Limit(1), Bound::Limit(3), Bound::All]
                            {
                                let selection = Selection {
                                    ignored: population,
                                    exclude: excluded
                                        .map_or_else(Vec::new, |name| vec![pattern(name)]),
                                    size,
                                    min_share: Some(ShareThreshold::parse(share).expect("share")),
                                    depth: Some(depth),
                                    breadth: Some(breadth),
                                    limit: Some(limit),
                                    ..Selection::default()
                                };
                                let report = run(
                                    &index,
                                    &query(&[ViewSpec::Summary, ViewSpec::Tree], selection.clone()),
                                );
                                let Section::Summary(summary) = &report.sections[0] else {
                                    panic!("summary")
                                };
                                let Section::Tree { root, omissions, .. } = &report.sections[1]
                                else {
                                    panic!("tree")
                                };
                                let expected = (
                                    selected_files.len() as u64,
                                    selected_dirs.len() as u64,
                                    selected_files.iter().map(|file| file.1).sum::<u64>(),
                                    selected_files.iter().map(|file| file.2).sum::<u64>(),
                                );
                                let context = format!("{selection:?}");
                                assert_eq!(
                                    (summary.files, summary.dirs, summary.bytes, summary.allocated),
                                    expected,
                                    "summary: {context}"
                                );
                                if let Some(root) = root {
                                    assert_eq!(
                                        (root.files, root.dirs, root.bytes, root.allocated),
                                        expected,
                                        "root: {context}"
                                    );
                                    let mut stack = vec![root.as_ref()];
                                    while let Some(node) = stack.pop() {
                                        let contained: Vec<_> = selected_files
                                            .iter()
                                            .filter(|file| within(file.0, &node.path))
                                            .collect();
                                        let contained_dirs = selected_dirs
                                            .iter()
                                            .filter(|dir| {
                                                within(dir.0, &node.path)
                                                    && Path::new(dir.0) != node.path
                                            })
                                            .count()
                                            as u64;
                                        let ignored: Vec<_> =
                                            contained.iter().filter(|file| file.3).collect();
                                        let ignored_dirs = selected_dirs
                                            .iter()
                                            .filter(|dir| {
                                                dir.1
                                                    && within(dir.0, &node.path)
                                                    && Path::new(dir.0) != node.path
                                            })
                                            .count()
                                            as u64;
                                        assert_eq!(
                                            (node.files, node.dirs, node.bytes, node.allocated),
                                            (
                                                contained.len() as u64,
                                                contained_dirs,
                                                contained.iter().map(|file| file.1).sum::<u64>(),
                                                contained.iter().map(|file| file.2).sum::<u64>(),
                                            ),
                                            "node {:?}: {context}",
                                            node.path
                                        );
                                        assert_eq!(
                                            node.ignored.map(|part| (
                                                part.files,
                                                part.dirs,
                                                part.bytes,
                                                part.allocated
                                            )),
                                            Some((
                                                ignored.len() as u64,
                                                ignored_dirs,
                                                ignored.iter().map(|file| file.1).sum::<u64>(),
                                                ignored.iter().map(|file| file.2).sum::<u64>(),
                                            )),
                                            "ignored {:?}: {context}",
                                            node.path
                                        );
                                        stack.extend(node.children.iter());
                                    }
                                }
                                let represented: Vec<_> = root
                                    .iter()
                                    .flat_map(|root| root.children.iter().map(|child| &child.path))
                                    .collect();
                                let uncovered: Vec<_> = selected_files
                                    .iter()
                                    .filter(|file| {
                                        !represented.iter().any(|path| within(file.0, path))
                                    })
                                    .collect();
                                let remainder =
                                    TreeRemainder::from_tree(root.as_deref(), omissions);
                                if let Some(remainder) = remainder {
                                    assert_eq!(
                                        (remainder.files, remainder.bytes, remainder.allocated),
                                        (
                                            Some(uncovered.len() as u64),
                                            Some(uncovered.iter().map(|file| file.1).sum::<u64>()),
                                            Some(uncovered.iter().map(|file| file.2).sum::<u64>()),
                                        ),
                                        "remainder: {context}"
                                    );
                                    assert_eq!(
                                        remainder.ignored.map(|part| (part.bytes, part.allocated)),
                                        Some((
                                            uncovered
                                                .iter()
                                                .filter(|file| file.3)
                                                .map(|file| file.1)
                                                .sum(),
                                            uncovered
                                                .iter()
                                                .filter(|file| file.3)
                                                .map(|file| file.2)
                                                .sum(),
                                        )),
                                        "remainder ignored: {context}"
                                    );
                                } else {
                                    assert!(uncovered.is_empty(), "missing remainder: {context}");
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn empty_tree_and_unrepresentable_remainder_stay_honest() {
        let empty = Index::new_with_scope("/root", crate::test_support::observing_controls());
        let report = run(
            &empty,
            &query(
                &[ViewSpec::Tree],
                Selection {
                    depth: Some(Bound::Limit(0)),
                    min_share: Some(ShareThreshold::parse("1%").expect("share")),
                    ..Selection::default()
                },
            ),
        );
        let Section::Tree { root: Some(root), omissions, .. } = &report.sections[0] else {
            panic!("empty tree")
        };
        assert_eq!((root.files, root.dirs, root.bytes, root.allocated), (0, 0, 0, 0));
        assert!(TreeRemainder::from_tree(Some(root), omissions).is_none());

        let first = TreeOmission {
            reason: TreeOmissionReason::Share,
            entries: 1,
            files: Some(u64::MAX),
            bytes: Some(u64::MAX),
            allocated: Some(u64::MAX),
            ignored: Some(IgnoredSize { bytes: u64::MAX, allocated: u64::MAX }),
        };
        let second = TreeOmission {
            reason: TreeOmissionReason::Breadth,
            entries: 1,
            files: Some(1),
            bytes: Some(1),
            allocated: None,
            ignored: Some(IgnoredSize { bytes: 1, allocated: 1 }),
        };
        let remainder = TreeRemainder::from_tree(None, &[first, second]).expect("omissions");
        assert_eq!((remainder.files, remainder.bytes, remainder.allocated), (None, None, None));
        assert_eq!(remainder.ignored, None);
    }

    #[test]
    fn unbounded_tree_expands_every_entry_without_a_remainder_or_diagnostics() {
        let index = mixed_remainder_index();
        let selection = Selection {
            size: SizeMetric::Apparent,
            depth: Some(Bound::All),
            breadth: Some(Bound::All),
            limit: Some(Bound::All),
            min_share: Some(ShareThreshold::parse("0%").expect("share")),
            ..Selection::default()
        };
        let report = run(&index, &query(&[ViewSpec::Tree], selection));
        let Section::Tree { root: Some(root), omissions, .. } = &report.sections[0] else {
            panic!("expected tree")
        };
        let mut paths = std::collections::BTreeSet::new();
        let mut stack = vec![root.as_ref()];
        while let Some(node) = stack.pop() {
            assert!(node.omissions.is_empty() && !node.truncated);
            paths.insert(node.path.to_string_lossy().replace('\\', "/"));
            stack.extend(&node.children);
        }
        assert_eq!(
            paths,
            [
                "",
                "A",
                "A/a1",
                "A/a1/tiny",
                "A/a1/x",
                "A/a1/x/leaf",
                "A/a2",
                "A/a2/file",
                "B",
                "B/b1",
                "B/b1/file",
                "B/b2",
                "B/b2/file",
                "C",
                "C/c1",
                "C/c1/first",
                "C/c1/second"
            ]
            .into_iter()
            .map(str::to_owned)
            .collect()
        );
        assert_eq!((root.files, root.dirs, root.bytes, root.allocated), (7, 9, 10_000, 10_752));
        assert!(omissions.is_empty());
        assert!(TreeRemainder::from_tree(Some(root), omissions).is_none());
        assert!(report.notes.is_empty() && report.tips.is_empty());
    }

    #[test]
    fn an_unlisted_hidden_branch_makes_remainder_counts_and_sizes_unknown() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            upsert("known", EntryKind::File, attrs(100, 0)),
            upsert("denied", EntryKind::Dir, attrs(0, 0)),
        ]));
        index.set_initial_scan_freshness(&[crate::Error::io(
            Path::new("/root").join("denied"),
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "failed listing"),
        )]);
        let selection = Selection {
            size: SizeMetric::Apparent,
            depth: Some(Bound::Limit(0)),
            min_share: Some(ShareThreshold::parse("0%").expect("share")),
            ..Selection::default()
        };
        let report = run(&index, &query(&[ViewSpec::Tree], selection));
        let Section::Tree { root: Some(root), omissions, .. } = &report.sections[0] else {
            panic!("expected tree")
        };
        let remainder = TreeRemainder::from_tree(Some(root), omissions).expect("depth bound");
        assert_eq!((remainder.files, remainder.bytes, remainder.allocated), (None, None, None));
        assert_eq!(remainder.reasons, vec![TreeOmissionReason::Depth]);
        let text = crate::report_format::render(&report, crate::report_format::Format::Text, false)
            .expect("render");
        // The root's age is unknown too, and the remainder's age cell is blank beneath it.
        assert!(text.contains("100 B  unknown  . 1 file"), "{text}");
        assert!(text.contains("—     unknown             … and more files (count unknown)"));
    }

    /// A failed listing must not turn off the default threshold for verified siblings.
    /// The whole rendered report is golden; exact arithmetic and unknown retention stay
    /// visible together rather than passing as isolated bounds/status assertions.
    #[test]
    fn partial_tree_keeps_default_share_pruning_golden() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            upsert("large", EntryKind::File, attrs(9898, 0)),
            upsert("one-percent", EntryKind::File, attrs(100, 0)),
            upsert("tiny", EntryKind::File, attrs(1, 0)),
            upsert("zero", EntryKind::File, attrs(0, 0)),
            upsert("small", EntryKind::Dir, attrs(0, 0)),
            upsert("small/tiny", EntryKind::File, attrs(1, 0)),
            upsert("denied", EntryKind::Dir, attrs(0, 0)),
        ]));
        index.set_initial_scan_freshness(&[crate::Error::io(
            Path::new("/root").join("denied"),
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "failed listing"),
        )]);
        let selection = Selection { size: SizeMetric::Apparent, ..Selection::default() };
        // An hour after every time in the tree, as the command-line test that shares this
        // golden stamps its fixture an hour and a half minute back.
        let mut request =
            crate::test_support::read_of(&index, query(&[ViewSpec::Tree], selection.clone()));
        request.now = UNIX_EPOCH + Duration::from_secs(3_600);
        let report = report(&index, &request, generated_at()).expect("report");
        assert!(!report.status.complete);
        assert_eq!(
            crate::report_format::render(&report, crate::report_format::Format::Text, false)
                .expect("render"),
            include_str!("../../tests/golden/partial-tree.txt"),
        );
        // A filtered partial tree uses the selected total (102 B), not the whole
        // observed root (10,000 B), while reusing selection's subtree measurement.
        let filtered = run(
            &index,
            &query(
                &[ViewSpec::Tree],
                Selection {
                    exclude: vec![pattern("large")],
                    min_share: Some(ShareThreshold::parse("50%").expect("share")),
                    ..selection.clone()
                },
            ),
        );
        let filtered_root = tree_of(&filtered);
        assert_eq!(filtered_root.children.len(), 2);
        assert_eq!(filtered_root.children[0].name, "one-percent");
        assert_eq!(filtered_root.children[1].name, "denied");
        let unbounded = run(
            &index,
            &query(
                &[ViewSpec::Tree],
                Selection {
                    min_share: Some(ShareThreshold::parse("0%").expect("share")),
                    ..selection
                },
            ),
        );
        assert_eq!(tree_of(&unbounded).children.len(), 6, "zero share lifts only share pruning");
    }

    #[test]
    fn tree_zero_and_composed_caps_keep_typed_first_boundaries() {
        let index = sample();
        let selection = Selection { limit: Some(Bound::Limit(0)), ..Selection::default() };
        let report = run(&index, &query(&[ViewSpec::Tree], selection));
        let Section::Tree { root: None, omissions, .. } = &report.sections[0] else {
            panic!("zero row cap must omit the root")
        };
        assert_eq!(omissions[0].reason, TreeOmissionReason::Rows);
        assert_eq!(omissions[0].entries, 1);

        let selection = Selection {
            size: SizeMetric::Apparent,
            depth: Some(Bound::Limit(1)),
            breadth: Some(Bound::Limit(1)),
            limit: Some(Bound::Limit(2)),
            min_share: Some(ShareThreshold::parse("1%").expect("share")),
            ..Selection::default()
        };
        let root = tree_of(&run(&index, &query(&[ViewSpec::Tree], selection)));
        assert_eq!(root.children.len(), 1);
        assert!(
            root.omissions.iter().any(|omission| omission.reason == TreeOmissionReason::Breadth)
        );
        assert!(
            root.children[0]
                .omissions
                .iter()
                .any(|omission| omission.reason == TreeOmissionReason::Depth)
        );
    }

    #[test]
    fn requesting_more_views_never_changes_another_views_answer() {
        // The property that makes `--view types,tree` one scan and one consistent state.
        let index = sample();
        let alone = types_of(&run(&index, &query(&[ViewSpec::Extensions], Selection::default())));
        let together = run(
            &index,
            &query(
                &[ViewSpec::Extensions, ViewSpec::Tree, ViewSpec::Summary],
                Selection::default(),
            ),
        );
        let with_others = match &together.sections[0] {
            Section::Extensions { rows, .. } => rows.clone(),
            other => panic!("expected types first, got {other:?}"),
        };

        assert_eq!(alone.len(), with_others.len());
        for (left, right) in alone.iter().zip(with_others.iter()) {
            assert_eq!(
                (&left.extension, left.files, left.bytes),
                (&right.extension, right.files, right.bytes)
            );
        }
        assert_eq!(together.sections.len(), 3, "one section per view, in request order");
        assert_eq!(together.sections[1].view(), ViewSpec::Tree);
        assert_eq!(together.sections[2].view(), ViewSpec::Summary);
    }

    #[test]
    fn analyzed_unfiltered_views_together_match_independent_answers_and_each_view_alone() {
        const RUST: &str = "fn main() {\n    println!(\"hi\");\n}\n";
        const MARKDOWN: &str = "# Guide\n\nA small useful guide.\n";
        const TEXT: &str = "plain notes here\n";
        let root = tempfile::tempdir().expect("root");
        fs::create_dir_all(root.path().join("src")).expect("src");
        fs::create_dir_all(root.path().join("docs")).expect("docs");
        fs::write(root.path().join("src/main.rs"), RUST).expect("rust");
        fs::write(root.path().join("docs/guide.md"), MARKDOWN).expect("markdown");
        fs::write(root.path().join("notes.txt"), TEXT).expect("text");
        for (path, seconds) in [("src/main.rs", 10), ("notes.txt", 20), ("docs/guide.md", 30)] {
            fs::File::options()
                .write(true)
                .open(root.path().join(path))
                .expect("open for timestamp")
                .set_times(
                    fs::FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(seconds)),
                )
                .expect("set timestamp");
        }
        let (mut index, _) = crate::scan::scan_into_index(
            root.path(),
            &crate::ScanConfig { read_controls: false, ..crate::ScanConfig::default() },
        )
        .expect("scan");
        crate::content::analyze_index(
            &mut index,
            crate::content::AnalysisRequest {
                profile: AnalysisSet::ALL,
                ..crate::content::AnalysisRequest::default()
            },
        );

        let views = [
            ViewSpec::Types,
            ViewSpec::Families,
            ViewSpec::Languages,
            ViewSpec::Documents,
            ViewSpec::Files,
            ViewSpec::Largest,
            ViewSpec::Recent,
            ViewSpec::Summary,
            ViewSpec::Tree,
            ViewSpec::Extensions,
        ];
        let selection = Selection { size: SizeMetric::Apparent, ..Selection::default() };
        let together = run(&index, &query(&views, selection.clone()));
        for (i, view) in views.iter().enumerate() {
            let alone = run(&index, &query(&[*view], selection.clone()));
            assert_eq!(
                format!("{:?}", together.sections[i]),
                format!("{:?}", alone.sections[0]),
                "{view:?} changed when requested with the other views"
            );
        }

        for (at, view, files) in [(0, ViewSpec::Types, 3), (1, ViewSpec::Families, 3)] {
            let Section::Metrics { view: actual, summary } = &together.sections[at] else {
                panic!("expected {view:?} metrics")
            };
            assert_eq!(*actual, view);
            assert_eq!(summary.total.files, files);
            assert_eq!(summary.total.analyzed_files, files);
        }
        let Section::Metrics { summary: languages, .. } = &together.sections[2] else {
            panic!("languages")
        };
        assert_eq!(languages.total.files, 1);
        assert_eq!(languages.total.metrics.code_lines, Some(3));
        let Section::Metrics { summary: documents, .. } = &together.sections[3] else {
            panic!("documents")
        };
        assert_eq!(documents.total.files, 2);
        assert_eq!(documents.total.analyzed_files, 2);
        assert_eq!(documents.total.document_metric_files, 2);
        assert_eq!(documents.total.document_raw_words, 8);
        assert_eq!(documents.total.document_word_stats.logical_words(), 8);
        assert_eq!(documents.total.share, MetricShare { numerator: 8, denominator: 8 });

        let Section::Files { rows: files, total, .. } = &together.sections[4] else {
            panic!("files")
        };
        assert_eq!(*total, 5);
        assert_eq!(
            files.iter().map(|row| row.path.as_path()).collect::<Vec<_>>(),
            ["docs", "docs/guide.md", "notes.txt", "src", "src/main.rs"].map(Path::new).to_vec()
        );
        let Section::Files { rows: largest, total, .. } = &together.sections[5] else {
            panic!("largest")
        };
        assert_eq!(*total, 3);
        assert_eq!(
            largest.iter().map(|row| row.path.as_path()).collect::<Vec<_>>(),
            ["src/main.rs", "docs/guide.md", "notes.txt"].map(Path::new).to_vec()
        );
        let Section::Files { rows: recent, total, .. } = &together.sections[6] else {
            panic!("recent")
        };
        assert_eq!(*total, 3);
        assert_eq!(
            recent.iter().map(|row| row.path.as_path()).collect::<Vec<_>>(),
            ["docs/guide.md", "notes.txt", "src/main.rs"].map(Path::new).to_vec()
        );
        let Section::Summary(summary) = &together.sections[7] else { panic!("summary") };
        assert_eq!((summary.files, summary.dirs), (3, 2));
        assert_eq!(
            summary.bytes,
            u64::try_from(RUST.len() + MARKDOWN.len() + TEXT.len()).expect("fixture bytes")
        );
        let Section::Tree { root: Some(tree), .. } = &together.sections[8] else { panic!("tree") };
        assert_eq!((tree.files, tree.dirs), (3, 2));
        let Section::Extensions { rows, total, .. } = &together.sections[9] else {
            panic!("extensions")
        };
        assert_eq!((*total, rows.len()), (3, 3));
    }

    #[test]
    fn one_pass_metric_summaries_match_independent_views() {
        let root = tempfile::tempdir().expect("root");
        fs::write(root.path().join("main.rs"), "fn main() {}\n").expect("rust");
        fs::write(root.path().join("guide.md"), "# Guide\n\nWords.\n").expect("markdown");
        fs::write(root.path().join(".gitignore"), "generated/\n").expect("ignore rules");
        fs::create_dir(root.path().join("generated")).expect("generated directory");
        fs::write(root.path().join("generated/app.js"), "const a = 1;\nconst b = 2;\n")
            .expect("ignored code");
        fs::write(root.path().join("script"), "#!/bin/sh\necho hello\n").expect("detected code");
        let (mut index, _) =
            crate::scan::scan_into_index(root.path(), &crate::ScanConfig::default()).expect("scan");
        crate::content::analyze_index(
            &mut index,
            crate::content::AnalysisRequest {
                profile: AnalysisSet::ALL,
                ..crate::content::AnalysisRequest::default()
            },
        );

        let views = [
            ViewSpec::Types,
            ViewSpec::Summary,
            ViewSpec::Families,
            ViewSpec::Languages,
            ViewSpec::Documents,
            ViewSpec::Code,
        ];
        let request = query(&views, Selection::default());
        let read = RootRead {
            root: 0,
            index: &index,
            walked: None,
            directories: None,
            tree_measured: false,
            activity: None,
            tree_views: false,
            unfiltered_rows: Some(every_entry(&index)),
        };
        let reads = std::slice::from_ref(&read);
        let summaries = metric_summaries(&views, reads, &request, AnalysisSet::ALL);

        assert!(summaries[1].is_none(), "non-metric views keep their own projection");
        assert!(summaries[5].is_none(), "Code keeps its admitted-content classification");
        for (position, view) in
            views.iter().copied().enumerate().filter(|(_, view)| needs_metric_resolution(*view))
        {
            let independent = metric_summary(view, reads, &request, AnalysisSet::ALL);
            assert_eq!(
                format!("{:?}", summaries[position].as_ref().expect("metric summary")),
                format!("{independent:?}"),
                "{view:?} changed in the one-pass multi-view aggregation"
            );
        }

        // Exercise the integrated population and presentation controls through the report
        // boundary. Include uses the shared pass; Exclude and Only use the filtered path.
        // Content-detected extensionless code also keeps Code's classification distinct.
        // Summary alone does not accept a share threshold, so compare additive views here.
        let views = views.into_iter().filter(|view| *view != ViewSpec::Summary).collect::<Vec<_>>();
        for ignored in [IgnoredEntries::Include, IgnoredEntries::Exclude, IgnoredEntries::Only] {
            for sort in [None, Some(SortKey::Metric("code_lines")), Some(SortKey::Name)] {
                for min_share in [None, Some(ShareThreshold::parse("50%").expect("share"))] {
                    let selection = Selection {
                        ignored,
                        sort,
                        min_share,
                        limit: Some(Bound::Limit(1)),
                        ..Selection::default()
                    };
                    let together = run(&index, &query(&views, selection.clone()));
                    for (position, view) in views.iter().enumerate() {
                        let alone = run(&index, &query(&[*view], selection.clone()));
                        assert_eq!(
                            format!("{:?}", together.sections[position]),
                            format!("{:?}", alone.sections[0]),
                            "{view:?} differs for {selection:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn code_overview_keeps_a_complete_language_table_and_population_contributions() {
        let root = tempfile::tempdir().expect("root");
        fs::create_dir_all(root.path().join("generated")).expect("generated");
        fs::write(root.path().join(".gitignore"), "generated/\n").expect("ignore rules");
        fs::create_dir_all(root.path().join("src")).expect("src");
        fs::write(root.path().join("src/main.rs"), "fn main() {\n}\n// comment\n").expect("rust");
        fs::write(
            root.path().join("generated/app.js"),
            "// Code generated by fixture\nconst answer = 42;\n",
        )
        .expect("javascript");
        fs::write(root.path().join("README.md"), "# Guide\n").expect("documentation");
        fs::write(root.path().join("mystery.widget"), "opaque text\n").expect("unknown type");
        let (mut index, _) =
            crate::scan::scan_into_index(root.path(), &crate::ScanConfig::default()).expect("scan");
        crate::content::analyze_index(
            &mut index,
            crate::content::AnalysisRequest { profile: AnalysisSet::NONE.with_code(), workers: 1 },
        );
        let answer = run(&index, &query(&[ViewSpec::Code], Selection::default()));
        let Section::Code(overview) = &answer.sections[0] else { panic!("code overview") };
        assert_eq!(overview.selected.metrics.code_lines, 3);
        assert_eq!(overview.selected.metrics.comment_lines, 2);
        assert_eq!(overview.selected.analyzed_files, 2);
        assert_eq!(overview.analyzed_languages, 2);
        assert_eq!(overview.unclassified_files, 2); // .gitignore and mystery.widget
        assert_eq!(overview.languages.len(), 2);
        assert_eq!(overview.languages[0].share.denominator, 3);
        assert_eq!(overview.non_ignored.as_ref().expect("classified").metrics.code_lines, 2);
        assert_eq!(overview.ignored.as_ref().expect("classified").metrics.code_lines, 1);
        assert_eq!(overview.unknown.source_files, 0);

        for (population, expected_lines) in
            [(IgnoredEntries::Exclude, 2), (IgnoredEntries::Only, 1)]
        {
            let selected = run(
                &index,
                &query(
                    &[ViewSpec::Code],
                    Selection { ignored: population, ..Selection::default() },
                ),
            );
            let Section::Code(overview) = &selected.sections[0] else { panic!("population") };
            assert_eq!(overview.selected.metrics.code_lines, expected_lines);
            assert!(overview.non_ignored.is_none() && overview.ignored.is_none());
            assert_eq!(overview.unknown.source_files, 0);
        }

        let threshold = run(
            &index,
            &query(
                &[ViewSpec::Code],
                Selection {
                    min_share: Some(ShareThreshold::parse("50%").expect("share")),
                    ..Selection::default()
                },
            ),
        );
        let Section::Code(overview) = &threshold.sections[0] else { panic!("threshold") };
        assert_eq!(overview.selected.metrics.code_lines, 3);
        assert_eq!(overview.total_languages, 1);
        assert_eq!(overview.share_omitted, 1);
        assert_eq!(overview.languages[0].language, "rust");

        let bounded = run(
            &index,
            &query(
                &[ViewSpec::Code],
                Selection { limit: Some(Bound::Limit(0)), ..Selection::default() },
            ),
        );
        let Section::Code(overview) = &bounded.sections[0] else { panic!("bounded") };
        assert_eq!(overview.selected.metrics.code_lines, 3);
        assert_eq!(overview.total_languages, 2);
        assert_eq!(overview.share_omitted, 0);
        assert!(overview.languages.is_empty());

        let code_and_extensions = run(
            &index,
            &query(
                &[ViewSpec::Languages, ViewSpec::Extensions],
                Selection {
                    min_share: Some(ShareThreshold::parse("100%").expect("share")),
                    ..Selection::default()
                },
            ),
        );
        let Section::Metrics { summary, .. } = &code_and_extensions.sections[0] else {
            panic!("languages")
        };
        let Section::Extensions { rows, total, share_omitted } = &code_and_extensions.sections[1]
        else {
            panic!("extensions")
        };
        assert_eq!(summary.total.metrics.code_lines, Some(3));
        assert!(summary.rows.is_empty() && summary.total_rows == 0);
        assert_eq!(summary.share_omitted, 2);
        assert!(rows.is_empty() && *total == 0);
        assert!(*share_omitted > 0);

        let reversed_code = run(
            &index,
            &query(&[ViewSpec::Code], Selection { reverse: true, ..Selection::default() }),
        );
        let Section::Code(reversed) = &reversed_code.sections[0] else { panic!("reversed code") };
        assert_eq!(reversed.languages[0].language, "javascript");
        assert_eq!(reversed.languages[1].language, "rust");

        let named_code = run(
            &index,
            &query(
                &[ViewSpec::Code],
                Selection { sort: Some(SortKey::Name), ..Selection::default() },
            ),
        );
        let Section::Code(named) = &named_code.sections[0] else { panic!("named code") };
        assert_eq!(named.languages[0].language, "javascript");
        assert_eq!(named.languages[1].language, "rust");

        let sized_code = run(
            &index,
            &query(
                &[ViewSpec::Code],
                Selection {
                    sort: Some(SortKey::Size),
                    size: SizeMetric::Apparent,
                    ..Selection::default()
                },
            ),
        );
        let Section::Code(sized) = &sized_code.sections[0] else { panic!("sized code") };
        assert_eq!(sized.languages[0].language, "javascript");

        let files = |reverse| {
            let selection = Selection {
                kinds: vec![EntryKind::File],
                sort: Some(SortKey::Metric("code_lines")),
                reverse,
                ..Selection::default()
            };
            run(&index, &query(&[ViewSpec::Files], selection))
        };
        let descending = files(false);
        let Section::Files { rows, .. } = &descending.sections[0] else { panic!("files") };
        assert_eq!(rows[0].path, PathBuf::from("src/main.rs"));
        assert_eq!(rows[0].sort_value, Some(2));
        assert_eq!(rows[1].path, PathBuf::from("generated/app.js"));
        assert_eq!(rows[1].sort_value, Some(1));
        assert!(rows[2..].iter().all(|row| row.sort_value.is_none()));
        let classification = rows[1].classification.as_ref().expect("file classification");
        assert!(classification.flags.generated);

        let ascending = files(true);
        let Section::Files { rows, .. } = &ascending.sections[0] else { panic!("files") };
        assert_eq!((rows[0].sort_value, rows[1].sort_value), (Some(1), Some(2)));
        assert!(rows[2..].iter().all(|row| row.sort_value.is_none()));

        let mut dir_query = query(
            &[ViewSpec::List],
            Selection {
                kinds: vec![EntryKind::Dir],
                sort: Some(SortKey::Metric("code_lines")),
                ..Selection::default()
            },
        );
        dir_query.format = crate::report_format::Format::Json;
        let dirs = run(&index, &dir_query);
        let Section::Files { rows, .. } = &dirs.sections[0] else { panic!("directories") };
        assert_eq!((rows[0].path.as_path(), rows[0].sort_value), (Path::new("src"), Some(2)));
        assert_eq!((rows[1].path.as_path(), rows[1].sort_value), (Path::new("generated"), Some(1)));
    }

    #[test]
    fn code_overview_counts_selected_directory_members_once() {
        let root = tempfile::tempdir().expect("root");
        fs::create_dir_all(root.path().join("src/deep")).expect("directories");
        fs::write(root.path().join("src/main.rs"), "fn main() {}\n").expect("main");
        fs::write(root.path().join("src/deep/keep.rs"), "fn keep() {}\n").expect("keep");
        fs::write(root.path().join("src/deep/skip.rs"), "fn skip() {}\n").expect("skip");
        fs::write(root.path().join("outside.rs"), "fn outside() {}\n").expect("outside");
        let (mut index, _) =
            crate::scan::scan_into_index(root.path(), &crate::ScanConfig::default()).expect("scan");
        crate::content::analyze_index(
            &mut index,
            crate::content::AnalysisRequest { profile: AnalysisSet::NONE.with_code(), workers: 1 },
        );
        let selection = Selection {
            include: vec![pattern("src"), pattern("deep")],
            exclude: vec![pattern("skip.rs")],
            ..Selection::default()
        };
        let answer = run(&index, &query(&[ViewSpec::Code], selection));
        let Section::Code(overview) = &answer.sections[0] else { panic!("code overview") };
        assert_eq!(overview.selected.source_files, 2);
        assert_eq!(overview.selected.analyzed_files, 2);
        assert_eq!(overview.selected.metrics.code_lines, 2);
        assert_eq!(overview.languages.len(), 1);
        assert_eq!(overview.languages[0].selected.source_files, 2);
    }

    #[test]
    fn code_overview_uses_retained_content_detection_for_ambiguous_sources() {
        let root = tempfile::tempdir().expect("root");
        fs::write(root.path().join("ambiguous.h"), "namespace demo { int value; }\n")
            .expect("header");
        fs::write(root.path().join("script.inc"), "# vim: set filetype=rust:\nfn main() {}\n")
            .expect("modeline");
        let (mut index, _) =
            crate::scan::scan_into_index(root.path(), &crate::ScanConfig::default()).expect("scan");
        crate::content::analyze_index(
            &mut index,
            crate::content::AnalysisRequest { profile: AnalysisSet::NONE.with_code(), workers: 1 },
        );
        let answer = run(&index, &query(&[ViewSpec::Code], Selection::default()));
        let Section::Code(overview) = &answer.sections[0] else { panic!("code overview") };
        assert_eq!(overview.selected.source_files, 2);
        assert_eq!(overview.selected.analyzed_files, 2);
        assert!(overview.languages.iter().any(|row| row.language == "cpp"));
        assert!(overview.languages.iter().any(|row| row.language == "rust"));
    }

    #[test]
    fn refused_control_subtrees_do_not_enter_a_known_ignored_population() {
        let root = tempfile::tempdir().expect("root");
        fs::create_dir_all(root.path().join("guarded")).expect("directory");
        fs::write(root.path().join("known.rs"), "fn known() {}\n").expect("known source");
        fs::write(root.path().join("guarded/.gitignore"), "*.rs\n").expect("refused control");
        fs::write(root.path().join("guarded/uncertain.rs"), "fn uncertain() {}\n")
            .expect("uncertain source");
        let config = crate::ScanConfig {
            control_limits: crate::control::ControlLimits {
                line_limit: Some(1),
                ..crate::control::ControlLimits::default()
            },
            ..crate::ScanConfig::default()
        };
        let (index, _) = crate::scan::scan_into_index(root.path(), &config).expect("scan");
        assert_eq!(index.ignored_classification(Path::new("known.rs")), Some(false));
        assert_eq!(index.ignored_classification(Path::new("guarded/uncertain.rs")), None);
        for selection in
            [Selection::default(), Selection { min_size: Some(0), ..Selection::default() }]
        {
            let report = run(
                &index,
                &query(&[ViewSpec::Summary, ViewSpec::Tree, ViewSpec::Extensions], selection),
            );
            let Section::Summary(summary) = &report.sections[0] else { panic!("summary") };
            let Section::Tree { root: Some(tree), .. } = &report.sections[1] else {
                panic!("tree")
            };
            let Section::Extensions { rows, .. } = &report.sections[2] else {
                panic!("extensions")
            };
            assert_eq!(summary.ignored, None);
            assert_eq!(tree.ignored, None);
            assert_eq!(
                tree.children
                    .iter()
                    .find(|node| node.path == Path::new("known.rs"))
                    .and_then(|node| node.ignored),
                Some(IgnoredTally::default())
            );
            assert!(
                tree.children
                    .iter()
                    .find(|node| node.path == Path::new("guarded"))
                    .is_some_and(|node| node.ignored.is_none())
            );
            assert!(rows.iter().all(|row| row.ignored.is_none()));
            assert!(
                report
                    .notes
                    .iter()
                    .any(|note| note.contains("gitignored subtotals are unavailable"))
            );
            assert!(
                crate::report_format::report_notes(&report)
                    .iter()
                    .all(|note| !note.contains("include gitignored sizes"))
            );
        }
        let hidden_report = run(
            &index,
            &query(
                &[ViewSpec::Tree],
                Selection { depth: Some(Bound::Limit(0)), ..Selection::default() },
            ),
        );
        let Section::Tree { root, omissions, .. } = &hidden_report.sections[0] else {
            panic!("expected tree")
        };
        let hidden = TreeRemainder::from_tree(root.as_deref(), omissions).expect("hidden rows");
        assert!(hidden.bytes.is_some() && hidden.ignored.is_none());
        for population in [IgnoredEntries::Exclude, IgnoredEntries::Only] {
            let answer = run(
                &index,
                &query(
                    &[ViewSpec::Files],
                    Selection {
                        kinds: vec![EntryKind::File],
                        ignored: population,
                        ..Selection::default()
                    },
                ),
            );
            let Section::Files { rows, .. } = &answer.sections[0] else { panic!("files") };
            assert!(rows.iter().all(|row| !row.path.starts_with("guarded")));
            assert_eq!(
                rows.iter().any(|row| row.path == Path::new("known.rs")),
                population == IgnoredEntries::Exclude
            );
        }
    }

    #[test]
    fn a_report_derives_provenance_from_its_index() {
        let index = sample();
        let report = run(&index, &query(&[ViewSpec::Summary], Selection::default()));
        assert_eq!(report.provenance.source, ReportSource::ColdScan);
        assert!(report.status.complete);
        assert!(report.provenance.scan_started_at.is_some());
        assert_eq!(report.provenance.generated_at, generated_at());
        assert_eq!(report.root.as_deref(), Some(Path::new("/root")));
    }

    #[test]
    fn reporting_is_pure_and_repeatable() {
        let index = sample();
        let request = query(&[ViewSpec::Tree, ViewSpec::Extensions], Selection::default());
        assert_eq!(format!("{:?}", run(&index, &request)), format!("{:?}", run(&index, &request)));
    }

    #[test]
    fn metadata_grouping_views_use_the_generic_metric_projection() {
        let index = sample();
        let apparent = Selection { size: SizeMetric::Apparent, ..Selection::default() };
        let report = run(
            &index,
            &query(&[ViewSpec::Types, ViewSpec::Families, ViewSpec::Languages], apparent),
        );
        let Section::Metrics { summary: types, .. } = &report.sections[0] else {
            panic!("expected type metrics")
        };
        let rust = types.rows.iter().find(|row| row.id == "rust").expect("rust");
        assert_eq!((rust.files, rust.bytes), (3, 350));
        assert_eq!((rust.share.numerator, rust.share.denominator), (350, 657));
        assert_eq!(types.share_metric, ShareMetric::ApparentBytes);

        let Section::Metrics { summary: families, .. } = &report.sections[1] else {
            panic!("expected family metrics")
        };
        assert!(families.rows.iter().any(|row| row.id == "code"));
        assert!(families.rows.iter().any(|row| row.id == "prose"));
        assert_eq!(families.share_metric, ShareMetric::ApparentBytes);

        let Section::Metrics { summary: languages, .. } = &report.sections[2] else {
            panic!("expected language metrics")
        };
        let rust = languages.rows.iter().find(|row| row.id == "rust").expect("rust");
        assert_eq!((rust.files, rust.bytes), (3, 350));
        assert_eq!((rust.share.numerator, rust.share.denominator), (350, 350));
        assert_eq!(languages.share_metric, ShareMetric::ApparentBytes);
    }

    /// The control file's name, spelled once for the fixtures that write one.
    const CONTROL: &str = ".gitignore";

    /// A small tree under a control source ignoring `build/` and `*.log`.
    fn classified_sample() -> Index {
        let mut index = Index::new_with_scope("/root", crate::test_support::observing_controls());
        index
            .apply(&Observation::new(vec![
                Op::ControlUpsert {
                    path: PathBuf::from(CONTROL),
                    source: b"build/\n*.log\n".to_vec(),
                },
                upsert("src", EntryKind::Dir, Attrs::default()),
                upsert("src/main.rs", EntryKind::File, attrs(100, 10)),
                upsert("src/lib.rs", EntryKind::File, attrs(200, 20)),
                upsert("src/debug.log", EntryKind::File, attrs(25, 70)),
                upsert("docs", EntryKind::Dir, Attrs::default()),
                upsert("docs/guide.md", EntryKind::File, attrs(300, 30)),
                upsert("build", EntryKind::Dir, Attrs::default()),
                upsert("build/cache", EntryKind::Dir, Attrs::default()),
                upsert("build/cache/out.bin", EntryKind::File, attrs(1_000, 60)),
            ]))
            .expect("apply");
        index
    }

    /// H186: a directory's children are admitted from their roll-ups before any row is
    /// built, in the order sorting every row would give; the children the share omits are
    /// summed from their facts alone, an ignored tally included; an incomplete child below
    /// the share is admitted regardless; and `complete = false` withholds the sums.
    #[test]
    fn child_rows_admit_from_roll_ups_and_sum_what_the_share_omits() {
        let mut index = Index::new_with_scope("/root", crate::test_support::observing_controls());
        index.apply_ok(&Observation::new(vec![
            Op::ControlUpsert { path: PathBuf::from(CONTROL), source: b"*.log\n".to_vec() },
            upsert("big", EntryKind::Dir, Attrs::default()),
            upsert("big/a", EntryKind::File, attrs(1_000, 10)),
            upsert("small", EntryKind::Dir, Attrs::default()),
            upsert("small/x.log", EntryKind::File, attrs(5, 20)),
            upsert("tiny", EntryKind::Dir, Attrs::default()),
            upsert("tiny/t", EntryKind::File, attrs(3, 30)),
            upsert("f1", EntryKind::File, attrs(200, 40)),
            upsert("f2", EntryKind::File, attrs(2, 50)),
        ]));
        let id = |path: &str| index.lookup(Path::new(path)).expect("an indexed path");
        let measured = |newest_ns, complete| query_subtrees::DirectoryActivity {
            newest_ns: Some(newest_ns),
            complete,
        };
        let activity = query_subtrees::ActivityTable::of(
            &index,
            [
                (id("big"), measured(10, true)),
                (id("small"), measured(20, true)),
                (id("tiny"), measured(30, false)),
            ],
        );
        // Apparent bytes, so the shares below are the file sizes' rather than their
        // allocated blocks', which round every small file up to one.
        let query = query(
            &[ViewSpec::Tree],
            Selection { size: SizeMetric::Apparent, ..Selection::default() },
        );
        let grand = 1_000 + 5 + 3 + 200 + 2;

        let (rows, omitted) = child_rows(
            &index,
            &query,
            None,
            &BTreeMap::new(),
            TreeRecency::Unfiltered(&activity),
            EntryId::ROOT,
            Path::new(""),
            &ShareThreshold::one_percent(),
            grand,
        );
        // 1% of 1,210 is 12.1: `big` and `f1` clear it, `tiny` does not but is incomplete,
        // and the rows come sorted by size, largest first, as the tree sorts them. Each
        // directory row carries its activity and completeness, and a file its own time.
        assert_eq!(
            rows.iter().map(|(row, id)| (row.name.as_str(), row.bytes, *id)).collect::<Vec<_>>(),
            [("big", 1_000, id("big")), ("f1", 200, id("f1")), ("tiny", 3, id("tiny"))]
        );
        assert_eq!(
            rows.iter().map(|(row, _)| (row.mtime_ns, row.complete)).collect::<Vec<_>>(),
            [(Some(10), Some(true)), (Some(40), None), (Some(30), Some(false))]
        );
        assert!(rows.iter().all(|(row, _)| row.path.as_path() == Path::new(&row.name)));
        // `small` and `f2` are below the share: their facts alone, `small` with the ignored
        // file it holds and `f2` with an empty tally, since its classification is known.
        let mut facts: Vec<_> =
            omitted.iter().map(|row| (row.bytes, row.files, row.allocated, row.ignored)).collect();
        facts.sort_unstable_by_key(|(bytes, ..)| *bytes);
        assert_eq!(
            facts,
            [
                (2, 1, 512, Some(IgnoredTally::default())),
                (5, 1, 512, Some(IgnoredTally { files: 1, dirs: 0, bytes: 5, allocated: 512 })),
            ]
        );

        for complete in [true, false] {
            let mut node = TreeNode {
                path: PathBuf::new(),
                name: String::new(),
                kind: EntryKind::Dir,
                entry_ignored: None,
                bytes: grand,
                allocated: 0,
                files: 5,
                dirs: 3,
                ignored: None,
                newest_mtime_ns: None,
                mtime_ns: None,
                complete: Some(complete),
                age_ns: None,
                children: Vec::new(),
                omissions: Vec::new(),
                truncated: false,
            };
            record_omission(&mut node, TreeOmissionReason::Share, &omitted, None, complete);
            assert!(node.truncated);
            let [omission] = node.omissions.as_slice() else {
                panic!("one Share omission, complete = {complete}: {:?}", node.omissions)
            };
            assert_eq!(omission.reason, TreeOmissionReason::Share);
            assert_eq!(omission.entries, 2);
            let sums = (omission.files, omission.bytes, omission.allocated, omission.ignored);
            let expected = if complete {
                (Some(2), Some(7), Some(1_024), Some(IgnoredSize { bytes: 5, allocated: 512 }))
            } else {
                (None, None, None, None)
            };
            assert_eq!(sums, expected, "complete = {complete}");
        }
    }

    #[test]
    fn tree_entry_classification_is_independent_of_selected_subtree_tallies() {
        let mut index = Index::new_with_scope("/root", crate::test_support::observing_controls());
        index.apply_ok(&Observation::new(vec![
            Op::ControlUpsert {
                path: PathBuf::from(CONTROL),
                source: b"empty/\nzero.txt\n*.log\n".to_vec(),
            },
            upsert("empty", EntryKind::Dir, Attrs::default()),
            upsert("zero.txt", EntryKind::File, attrs(0, 0)),
            upsert("mixed", EntryKind::Dir, Attrs::default()),
            upsert("mixed/ignored.log", EntryKind::File, attrs(5, 5)),
            upsert("mixed/ordinary.rs", EntryKind::File, attrs(7, 7)),
        ]));
        let selection = |ignored| Selection {
            ignored,
            depth: Some(Bound::All),
            breadth: Some(Bound::All),
            limit: Some(Bound::All),
            min_share: Some(ShareThreshold::parse("0%").expect("share")),
            ..Selection::default()
        };
        let included =
            tree_of(&run(&index, &query(&[ViewSpec::Tree], selection(IgnoredEntries::Include))));
        assert_eq!(included.entry_ignored, Some(false));
        let child = |name: &str| included.children.iter().find(|row| row.name == name).expect(name);
        let empty = child("empty");
        assert_eq!((empty.kind, empty.bytes, empty.entry_ignored), (EntryKind::Dir, 0, Some(true)));
        assert_eq!(child("zero.txt").entry_ignored, Some(true));
        let mixed = child("mixed");
        assert_eq!(mixed.entry_ignored, Some(false));
        assert_eq!(mixed.ignored.expect("known").bytes, 5);
        assert_eq!(
            mixed
                .children
                .iter()
                .find(|row| row.name == "ignored.log")
                .expect("ignored leaf")
                .entry_ignored,
            Some(true)
        );

        let only =
            tree_of(&run(&index, &query(&[ViewSpec::Tree], selection(IgnoredEntries::Only))));
        assert_eq!(only.entry_ignored, Some(false));
        let mixed_only = only.children.iter().find(|row| row.name == "mixed").expect("ancestor");
        assert_eq!(mixed_only.entry_ignored, Some(false));
        assert_eq!(mixed_only.ignored.expect("known").bytes, 5);

        let mut unobserved =
            Index::new_with_scope("/root", crate::test_support::not_observing_controls());
        unobserved.apply_ok(&Observation::new(vec![upsert(
            "plain",
            EntryKind::Dir,
            Attrs::default(),
        )]));
        let unknown = tree_of(&run(
            &unobserved,
            &query(&[ViewSpec::Tree], selection(IgnoredEntries::Include)),
        ));
        assert_eq!(unknown.entry_ignored, None);
        assert_eq!(unknown.children[0].entry_ignored, None);
    }

    #[test]
    fn ignored_size_interpretation_note_appears_once_only_when_relevant() {
        let classified = classified_sample();
        for (population, expected) in
            [(IgnoredEntries::Include, 1), (IgnoredEntries::Exclude, 0), (IgnoredEntries::Only, 0)]
        {
            let report = run(
                &classified,
                &query(
                    &[ViewSpec::Summary, ViewSpec::Tree, ViewSpec::Extensions],
                    Selection {
                        ignored: population,
                        depth: Some(Bound::All),
                        min_share: Some(ShareThreshold::parse("0%").expect("share")),
                        ..Selection::default()
                    },
                ),
            );
            let notes = crate::report_format::report_notes(&report);
            assert_eq!(
                notes.iter().filter(|note| note.contains("include gitignored sizes")).count(),
                expected,
                "{population:?}: {notes:?}"
            );
        }
        let mut unobserved =
            Index::new_with_scope("/root", crate::test_support::not_observing_controls());
        unobserved.apply_ok(&Observation::new(vec![upsert(
            "plain.rs",
            EntryKind::File,
            attrs(10, 0),
        )]));
        let blind =
            run(&unobserved, &query(&[ViewSpec::Summary, ViewSpec::Tree], Selection::default()));
        assert!(
            crate::report_format::report_notes(&blind)
                .iter()
                .all(|note| !note.contains("include gitignored sizes"))
        );
    }

    #[test]
    fn tree_remainder_counts_only_unrepresented_root_children() {
        let index = classified_sample();
        let remainder = |selection| {
            let report = run(&index, &query(&[ViewSpec::Tree], selection));
            let Section::Tree { root, omissions, .. } = &report.sections[0] else {
                panic!("expected tree")
            };
            TreeRemainder::from_tree(root.as_deref(), omissions).expect("hidden rows")
        };
        let mixed = remainder(Selection {
            size: SizeMetric::Apparent,
            depth: Some(Bound::Limit(1)),
            breadth: Some(Bound::Limit(2)),
            min_share: Some(ShareThreshold::parse("5%").expect("share")),
            ..Selection::default()
        });
        let usage = |ignored: Option<IgnoredSize>| ignored.map(|part| (part.bytes, part.allocated));
        assert_eq!(
            (mixed.files, mixed.bytes, usage(mixed.ignored)),
            (Some(1), Some(300), Some((0, 0)))
        );
        assert_eq!(mixed.reasons, vec![TreeOmissionReason::Breadth]);

        for limit in [Bound::Limit(1), Bound::Limit(0)] {
            let rows = remainder(Selection { limit: Some(limit), ..Selection::default() });
            assert_eq!((rows.bytes, usage(rows.ignored)), (Some(1_625), Some((1_025, 1_536))));
        }
        let excluded = remainder(Selection {
            depth: Some(Bound::Limit(0)),
            ignored: IgnoredEntries::Exclude,
            ..Selection::default()
        });
        assert_eq!((excluded.bytes, excluded.ignored), (Some(600), Some(IgnoredSize::default())));
        let only = remainder(Selection {
            depth: Some(Bound::Limit(0)),
            ignored: IgnoredEntries::Only,
            ..Selection::default()
        });
        assert_eq!((only.bytes, usage(only.ignored)), (Some(1_025), Some((1_025, 1_536))));
    }

    #[test]
    fn colored_tree_populations_share_one_golden() {
        let classified = classified_sample();
        let mut unobserved =
            Index::new_with_scope("/root", crate::test_support::not_observing_controls());
        unobserved.apply_ok(&Observation::new(vec![upsert(
            "plain.rs",
            EntryKind::File,
            attrs(10, 0),
        )]));

        let root = tempfile::tempdir().expect("root");
        fs::create_dir(root.path().join("guarded")).expect("directory");
        fs::write(root.path().join("known.rs"), "K").expect("known file");
        fs::write(root.path().join("guarded/.gitignore"), "*.rs\n").expect("control");
        fs::write(root.path().join("guarded/uncertain.rs"), "U").expect("uncertain file");
        let config = crate::ScanConfig {
            control_limits: crate::control::ControlLimits {
                line_limit: Some(1),
                ..crate::control::ControlLimits::default()
            },
            ..crate::ScanConfig::default()
        };
        let (refused, _) = crate::scan::scan_into_index(root.path(), &config).expect("scan");
        assert_eq!(refused.ignored_classification(Path::new("known.rs")), Some(false));
        assert_eq!(refused.ignored_classification(Path::new("guarded/uncertain.rs")), None);

        let selection = |ignored| Selection {
            size: SizeMetric::Apparent,
            depth: Some(Bound::Limit(0)),
            ignored,
            ..Selection::default()
        };
        let cases = [
            ("INCLUDE", &classified, selection(IgnoredEntries::Include)),
            (
                "INCLUDE EXPANDED",
                &classified,
                Selection {
                    depth: Some(Bound::Limit(2)),
                    min_share: Some(ShareThreshold::parse("0%").expect("share")),
                    ..selection(IgnoredEntries::Include)
                },
            ),
            ("EXCLUDE", &classified, selection(IgnoredEntries::Exclude)),
            ("ONLY", &classified, selection(IgnoredEntries::Only)),
            ("NO CONTROLS", &unobserved, selection(IgnoredEntries::Include)),
            ("REFUSED CONTROL", &refused, selection(IgnoredEntries::Include)),
        ];
        let mut actual = String::new();
        for (label, index, selection) in cases {
            // Ages are incidental here, and one fixture is written to disk as the test
            // runs. Read each tree a microsecond after its newest activity, so every age is
            // `0s` however long ago the fixture was written. Not at that instant itself: a
            // `SystemTime` is 100 ns apart on Windows, and the instant would be truncated to
            // one before the newest time there, which renders `-0s`.
            let query = query(&[ViewSpec::Tree], selection);
            let newest = tree_of(&run(index, &query)).mtime_ns.expect("every tree has activity");
            let mut request = crate::test_support::read_of(index, query);
            request.now = UNIX_EPOCH
                + Duration::from_nanos(u64::try_from(newest).expect("after the epoch"))
                + Duration::from_micros(1);
            let report = report(index, &request, generated_at()).expect("report");
            actual.push_str(label);
            actual.push('\n');
            actual.push_str(
                &crate::report_format::render(&report, crate::report_format::Format::Text, true)
                    .expect("colored tree")
                    .replace('\u{1b}', "<ESC>"),
            );
            actual.push('\n');
        }
        assert_eq!(actual, include_str!("../../tests/golden/tree-populations.txt"));
    }

    #[test]
    fn tree_remainder_ignored_share_is_unknown_without_control_observation() {
        let mut index =
            Index::new_with_scope("/root", crate::test_support::not_observing_controls());
        index.apply_ok(&Observation::new(vec![upsert("file", EntryKind::File, attrs(10, 0))]));
        let report = run(
            &index,
            &query(
                &[ViewSpec::Tree],
                Selection { depth: Some(Bound::Limit(0)), ..Selection::default() },
            ),
        );
        let Section::Tree { root, omissions, .. } = &report.sections[0] else {
            panic!("expected tree")
        };
        let hidden = TreeRemainder::from_tree(root.as_deref(), omissions).expect("hidden file");
        assert_eq!((hidden.bytes, hidden.ignored), (Some(10), None));
    }

    fn ignored_of(row: &SummaryRow) -> IgnoredTally {
        row.ignored.expect("an observing index reports an ignored share")
    }

    /// Both tiers report the same ignored share on every row kind that carries one: the
    /// unfiltered tier subtracts the maintained partitions, and the traversal tier counts
    /// the entries it admits.
    #[test]
    fn the_two_tiers_agree_on_the_ignored_share() {
        let index = classified_sample();
        let expected = IgnoredTally { files: 2, dirs: 2, bytes: 1_025, allocated: 1_024 + 512 };
        for selection in
            [Selection::default(), Selection { min_size: Some(0), ..Selection::default() }]
        {
            let unfiltered = selection.is_unfiltered();
            let summary = summary_of(&run(&index, &query(&[ViewSpec::Summary], selection.clone())));
            assert_eq!(ignored_of(&summary), expected, "unfiltered: {unfiltered}");

            let tree = tree_of(&run(&index, &query(&[ViewSpec::Tree], selection.clone())));
            assert_eq!(tree.ignored, Some(expected), "unfiltered: {unfiltered}");
            let child = |name: &str| {
                tree.children.iter().find(|node| node.name == name).expect(name).ignored
            };
            assert_eq!(
                child("build"),
                Some(IgnoredTally { files: 1, dirs: 1, bytes: 1_000, allocated: 1_024 }),
                "an ignored directory is wholly ignored below it, unfiltered: {unfiltered}"
            );
            assert_eq!(
                child("src"),
                Some(IgnoredTally { files: 1, dirs: 0, bytes: 25, allocated: 512 }),
                "unfiltered: {unfiltered}"
            );
            assert_eq!(child("docs"), Some(IgnoredTally::default()), "observed, nothing ignored");

            let rows = types_of(&run(&index, &query(&[ViewSpec::Extensions], selection.clone())));
            let row = |extension: &str| {
                rows.iter().find(|row| row.extension == extension).expect(extension).ignored
            };
            assert_eq!(
                row(".log"),
                Some(IgnoredTally { files: 1, dirs: 0, bytes: 25, allocated: 512 }),
                "unfiltered: {unfiltered}"
            );
            assert_eq!(row(".rs"), Some(IgnoredTally::default()), "unfiltered: {unfiltered}");

            let files = files_of(&run(&index, &query(&[ViewSpec::Files], selection)));
            let flag =
                |path: PathBuf| files.iter().find(|row| row.path == path).map(|row| row.ignored);
            assert_eq!(flag(PathBuf::from("build")), Some(Some(true)));
            assert_eq!(flag(PathBuf::from("src")), Some(Some(false)));
            assert_eq!(flag(["src", "debug.log"].iter().collect()), Some(Some(true)));
            assert_eq!(flag(["build", "cache", "out.bin"].iter().collect()), Some(Some(true)));
        }
    }

    /// Excluding ignored entries and selecting only them split the tree into two parts that
    /// sum to it, and each sizes and ranks its rows by what it selected.
    #[test]
    fn ignored_entries_partition_the_tree_and_rank_by_what_they_select() {
        let index = classified_sample();
        // Apparent, so the ranking is by the distinct sizes the sample wrote rather than by
        // the 512-byte blocks they round up to.
        let apparent = Selection { size: SizeMetric::Apparent, ..Selection::default() };
        let with = |ignored| Selection { ignored, ..apparent.clone() };
        let summary = |selection| summary_of(&run(&index, &query(&[ViewSpec::Summary], selection)));
        let total = summary(apparent.clone());
        let kept = summary(with(IgnoredEntries::Exclude));
        let only = summary(with(IgnoredEntries::Only));
        assert_eq!(
            (kept.files + only.files, kept.dirs + only.dirs, kept.bytes + only.bytes),
            (total.files, total.dirs, total.bytes)
        );
        assert_eq!(ignored_of(&kept), IgnoredTally::default());
        let whole = ignored_of(&only);
        assert_eq!((whole.files, whole.dirs, whole.bytes), (only.files, only.dirs, only.bytes));

        let ranked = |selection| {
            tree_of(&run(&index, &query(&[ViewSpec::Tree], selection)))
                .children
                .iter()
                .map(|node| (node.name.clone(), node.bytes))
                .collect::<Vec<_>>()
        };
        let row = |name: &str, bytes: u64| (name.to_string(), bytes);
        assert_eq!(
            ranked(apparent.clone()),
            [row("build", 1_000), row("src", 325), row("docs", 300)]
        );
        assert_eq!(
            ranked(with(IgnoredEntries::Exclude)),
            [row("docs", 300), row("src", 300)],
            "unignored sizes rank the rows, with the name breaking the tie"
        );
        assert_eq!(ranked(with(IgnoredEntries::Only)), [row("build", 1_000), row("src", 25)]);
    }

    /// An index that read no rule reports no ignored share on any row, and refuses a
    /// selection by ignored state in the vocabulary of the surface that asked.
    #[test]
    fn an_index_that_observed_no_control_state_has_no_ignored_share_to_select_by() {
        let mut index =
            Index::new_with_scope("/root", crate::test_support::not_observing_controls());
        index
            .apply(&Observation::new(vec![
                upsert("build", EntryKind::Dir, Attrs::default()),
                upsert("build/out.bin", EntryKind::File, attrs(1_000, 60)),
            ]))
            .expect("apply");
        let views = [ViewSpec::Summary, ViewSpec::Tree, ViewSpec::Extensions, ViewSpec::Files];
        let report = run(&index, &query(&views, Selection::default()));
        let Section::Summary(summary) = &report.sections[0] else { panic!("a summary") };
        let Section::Tree { root: Some(tree), .. } = &report.sections[1] else { panic!("a tree") };
        let Section::Extensions { rows: extensions, .. } = &report.sections[2] else {
            panic!("extensions")
        };
        let Section::Files { rows: files, .. } = &report.sections[3] else { panic!("files") };
        assert_eq!(summary.ignored, None);
        assert_eq!(tree.ignored, None);
        assert!(tree.children.iter().all(|node| node.ignored.is_none()));
        assert!(extensions.iter().all(|row| row.ignored.is_none()));
        assert!(files.iter().all(|row| row.ignored.is_none()));

        let exclude = Query {
            selection: Selection { ignored: IgnoredEntries::Exclude, ..Selection::default() },
            views: vec![ViewSpec::Summary],
            ..Query::default()
        };
        let only = Query {
            selection: Selection { ignored: IgnoredEntries::Only, ..Selection::default() },
            ..exclude.clone()
        };
        // The request model refuses such a request before it reaches a reader, in each
        // surface's words (`query_request`'s tests). The library path refuses it too, rather
        // than answering with no rows: a caller that reaches `report` without validating gets
        // the same typed refusal every other surface renders.
        for refused in [exclude, only] {
            assert!(
                matches!(
                    super::report(
                        &index,
                        &crate::test_support::read_of(&index, refused),
                        generated_at()
                    ),
                    Err(crate::Error::InvalidRequest(
                        crate::query::RequestError::IgnoredWithoutObservation(_)
                    ))
                ),
                "a selection by ignored state over an unobserving index is refused"
            );
        }
        assert_eq!(summary_of(&run(&index, &query(&views, Selection::default()))).files, 1);
    }

    // ---- several roots --------------------------------------------------------------

    /// An index at `root` holding `entries`, each a path, a kind, and for a file its size
    /// and time.
    fn rooted(root: &str, entries: &[(&str, EntryKind, u64, i64)]) -> Index {
        let mut index = Index::new(root);
        index
            .apply(&Observation::new(
                entries
                    .iter()
                    .map(|(path, kind, size, mtime)| match kind {
                        EntryKind::File => upsert(path, *kind, attrs(*size, *mtime)),
                        _ => upsert(path, *kind, Attrs { mtime_ns: *mtime, ..Attrs::default() }),
                    })
                    .collect(),
            ))
            .expect("apply");
        index
    }

    /// Two disjoint roots, `/docs` and `/src`, labelled `docs` and `src`.
    fn two_roots() -> (Index, Index, Roots) {
        use EntryKind::{Dir, File};
        let docs = rooted(
            "/docs",
            &[
                ("guide.md", File, 300, 30),
                ("api", Dir, 0, 35),
                ("api/index.md", File, 40, 50),
                ("logo.png", File, 9, 7),
            ],
        );
        let src = rooted(
            "/src",
            &[
                ("main.rs", File, 100, 10),
                ("lib.rs", File, 200, 20),
                ("deep", Dir, 0, 3),
                ("deep/nested.rs", File, 50, 40),
                ("notes.md", File, 5, 5),
            ],
        );
        (docs, src, labelled(&[("docs", "/docs"), ("src", "/src")]))
    }

    fn labelled(roots: &[(&str, &str)]) -> Roots {
        Roots::named(
            roots
                .iter()
                .map(|(label, path)| NamedRoot { label: label.into(), path: path.into() })
                .collect(),
        )
    }

    fn run_roots(indexes: &[&Index], roots: &Roots, query: &Query) -> Report {
        let request = crate::test_support::read_of(indexes[0], query.clone());
        report_roots(indexes, roots, &request, generated_at()).expect("answerable")
    }

    fn root_trees_of(report: &Report) -> (&RootTrees, &[TreeOmission]) {
        match report.sections.first().expect("a section") {
            Section::Tree { root: None, roots: Some(roots), omissions, .. } => (roots, omissions),
            other => panic!("expected the trees of several roots, got {other:?}"),
        }
    }

    /// Every display bound off, so a section shows everything it counts.
    fn unbounded() -> Selection {
        Selection {
            min_share: Some(ShareThreshold::parse("0%").expect("share")),
            limit: Some(Bound::All),
            depth: Some(Bound::All),
            ..Selection::default()
        }
    }

    /// A tree's rows in pre-order with their size and share-bearing fields, for comparing
    /// trees that differ only in their root's name.
    fn shape(root: &TreeNode) -> Vec<String> {
        let mut out = Vec::new();
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            out.push(format!(
                "{} {} {} {} {:?} {:?} {:?}",
                node.path.display(),
                node.bytes,
                node.files,
                node.dirs,
                node.mtime_ns,
                node.age_ns,
                node.omissions
            ));
            stack.extend(node.children.iter().rev());
        }
        out
    }

    /// A report over several roots is the sum of the reports over each: every additive
    /// value adds, every maximum is the maximum, and each root's tree is its own.
    #[test]
    fn several_roots_report_the_sum_of_their_single_root_reports() {
        let (docs, src, roots) = two_roots();
        let both = [&docs, &src];

        let summary =
            summary_of(&run_roots(&both, &roots, &query(&[ViewSpec::Summary], unbounded())));
        let (one, two) = (
            summary_of(&run(&docs, &query(&[ViewSpec::Summary], unbounded()))),
            summary_of(&run(&src, &query(&[ViewSpec::Summary], unbounded()))),
        );
        assert_eq!(
            (summary.files, summary.dirs, summary.bytes, summary.allocated),
            (
                one.files + two.files,
                one.dirs + two.dirs,
                one.bytes + two.bytes,
                one.allocated + two.allocated
            )
        );
        assert_eq!(summary.newest_mtime_ns, Some(50));

        let types =
            types_of(&run_roots(&both, &roots, &query(&[ViewSpec::Extensions], unbounded())));
        let mut expected = BTreeMap::<String, (u64, u64)>::new();
        for index in [&docs, &src] {
            for row in types_of(&run(index, &query(&[ViewSpec::Extensions], unbounded()))) {
                let sum = expected.entry(row.extension).or_default();
                sum.0 += row.files;
                sum.1 += row.bytes;
            }
        }
        let merged: BTreeMap<String, (u64, u64)> =
            types.into_iter().map(|row| (row.extension, (row.files, row.bytes))).collect();
        assert_eq!(merged, expected);
        assert_eq!(merged[".md"], (3, 345), "one bucket counts both roots' Markdown");

        let files = files_of(&run_roots(&both, &roots, &query(&[ViewSpec::Files], unbounded())));
        let mut union: Vec<(usize, PathBuf)> = Vec::new();
        for (root, index) in [&docs, &src].into_iter().enumerate() {
            union.extend(
                files_of(&run(index, &query(&[ViewSpec::Files], unbounded())))
                    .into_iter()
                    .map(|row| (root, row.path)),
            );
        }
        let mut listed: Vec<(usize, PathBuf)> =
            files.iter().map(|row| (row.root, row.path.clone())).collect();
        listed.sort();
        union.sort();
        assert_eq!(listed, union, "every row of each root, once, with its root");

        let report = run_roots(&both, &roots, &query(&[ViewSpec::Tree], unbounded()));
        assert_eq!(report.root, None);
        assert_eq!(report.roots.as_deref(), Some(roots.as_slice()));
        let (trees, top) = root_trees_of(&report);
        assert!(top.is_empty());
        let total = trees.total.expect("a total row");
        assert_eq!(
            (total.bytes, total.files, total.dirs),
            (summary.bytes, summary.files, summary.dirs)
        );
        assert_eq!(total.mtime_ns, Some(50), "the newest activity under any root");
        assert!(total.complete);
        assert_eq!(total.age_ns, Some(i128::from(report.age_reference_ns.expect("now")) - 50));
        for (index, label) in [(&docs, "docs"), (&src, "src")] {
            let alone = tree_of(&run(index, &query(&[ViewSpec::Tree], unbounded())));
            let tree = trees
                .trees
                .iter()
                .find(|tree| tree.tree.name == label)
                .unwrap_or_else(|| panic!("{label} is a row"));
            assert_eq!(report.roots.as_ref().expect("roots")[tree.root].label, Path::new(label));
            assert_eq!(shape(&tree.tree), shape(&alone), "{label}'s tree is its own");
        }
    }

    /// Pooled word statistics merge as sufficient statistics: the combined total's logical
    /// words come from the merged pool, not from adding two rounded estimates.
    #[test]
    fn several_roots_pool_their_word_statistics() {
        let directories: Vec<tempfile::TempDir> =
            (0..2).map(|_| tempfile::tempdir().expect("tempdir")).collect();
        fs::write(directories[0].path().join("a.md"), "# Title\n\nshort words here\n").expect("a");
        fs::write(directories[0].path().join("b.txt"), "x ".repeat(40)).expect("b");
        fs::write(
            directories[1].path().join("c.md"),
            "incomprehensibilities notwithstanding extraordinarily\n",
        )
        .expect("c");
        let indexes: Vec<Index> = directories
            .iter()
            .map(|directory| {
                let root = directory.path().canonicalize().expect("canonical");
                let (mut index, _) =
                    crate::scan::scan_into_index(&root, &crate::ScanConfig::default())
                        .expect("scan");
                crate::content::analyze_index(
                    &mut index,
                    crate::content::AnalysisRequest {
                        profile: AnalysisSet::ALL,
                        ..crate::content::AnalysisRequest::default()
                    },
                );
                index
            })
            .collect();
        let roots = Roots::named(
            indexes
                .iter()
                .enumerate()
                .map(|(position, index)| NamedRoot {
                    label: format!("r{position}").into(),
                    path: index.root_path().to_path_buf(),
                })
                .collect(),
        );
        let documents = |report: &Report| match report.sections.first() {
            Some(Section::Metrics { summary, .. }) => summary.total.clone(),
            other => panic!("expected documents, got {other:?}"),
        };
        let words = query(&[ViewSpec::Documents], unbounded());
        let request = |index: &Index| {
            let mut request = crate::test_support::read_of(index, words.clone());
            request.basis.content = AnalysisSet::ALL;
            request
        };
        let alone: Vec<MetricRow> = indexes
            .iter()
            .map(|index| documents(&report(index, &request(index), generated_at()).expect("one")))
            .collect();
        let both = report_roots(
            &indexes.iter().collect::<Vec<_>>(),
            &roots,
            &request(&indexes[0]),
            generated_at(),
        )
        .expect("both");
        let merged = documents(&both);
        let mut pooled = alone[0].logical_word_stats;
        pooled.add_assign(alone[1].logical_word_stats);
        assert_eq!(merged.logical_word_stats, pooled);
        assert_eq!(merged.files, alone[0].files + alone[1].files);
    }

    /// Roots whose combined totals no `u64` holds are refused, naming the root whose
    /// addition overflowed and the counter, as one root's unrepresentable total is.
    #[test]
    fn combined_totals_past_u64_are_refused_naming_the_root() {
        let roots = [(Path::new("/a"), [1, 1, u64::MAX - 5, 10]), (Path::new("/b"), [1, 1, 5, 10])];
        totals_fit(roots.into_iter()).expect("exactly u64::MAX fits");
        let over = [(Path::new("/a"), [1, 1, u64::MAX - 5, 10]), (Path::new("/b"), [1, 1, 6, 10])];
        match totals_fit(over.into_iter()) {
            Err(crate::Error::UnrepresentableTotal { path, counter }) => {
                assert_eq!((path, counter), (PathBuf::from("/b"), "bytes"));
            }
            other => panic!("expected an unrepresentable total, got {other:?}"),
        }
    }

    /// The ignored share of a sum is unknown when any term's is, never a partial sum.
    #[test]
    fn a_sum_of_roots_withholds_an_ignored_share_any_root_withholds() {
        let known = SummaryRow {
            files: 2,
            bytes: 10,
            ignored: Some(IgnoredTally { files: 1, dirs: 0, bytes: 4, allocated: 4 }),
            newest_mtime_ns: Some(3),
            ..SummaryRow::default()
        };
        let unknown = SummaryRow { ignored: None, newest_mtime_ns: None, ..known };
        let both = sum_roots([known, known]);
        assert_eq!(both.ignored.map(|share| share.bytes), Some(8));
        assert_eq!((both.files, both.bytes, both.newest_mtime_ns), (4, 20, Some(3)));
        assert_eq!(sum_roots([known, unknown]).ignored, None);
        assert_eq!(sum_roots([unknown, known]).newest_mtime_ns, Some(3));
        assert_eq!(format!("{:?}", sum_roots([known])), format!("{known:?}"));
    }

    /// A flat view's top rows over several roots are exactly the top rows of their union,
    /// under every key and direction, although each root is bounded on its own first.
    #[test]
    fn a_bounded_flat_view_over_several_roots_is_the_top_of_their_union() {
        let mut seed = 0x2545_f491_4f6c_dd1d_u64;
        let mut next = move |bound: u64| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed % bound
        };
        let names = ["a", "b", "c", "dd", "e"];
        let indexes: Vec<Index> = (0..3)
            .map(|root| {
                let entries: Vec<(String, EntryKind, u64, i64)> = (0..12)
                    .map(|file| {
                        let name =
                            format!("{}{file}", names[usize::try_from(next(5)).expect("small")]);
                        (
                            name,
                            EntryKind::File,
                            next(6) * 100,
                            i64::try_from(next(4)).expect("small"),
                        )
                    })
                    .collect();
                let borrowed: Vec<(&str, EntryKind, u64, i64)> = entries
                    .iter()
                    .map(|(name, kind, size, mtime)| (name.as_str(), *kind, *size, *mtime))
                    .collect();
                rooted(&format!("/r{root}"), &borrowed)
            })
            .collect();
        // Labels whose order differs from the argument order, so rank is not position.
        let roots = labelled(&[("zeta", "/r0"), ("alpha", "/r1"), ("mid", "/r2")]);
        let both: Vec<&Index> = indexes.iter().collect();
        let key = |row: &FileRow| {
            format!("{}/{}", roots.as_slice()[row.root].label.display(), row.path.display())
        };
        for sort in [SortKey::Size, SortKey::Name, SortKey::Count, SortKey::Mtime] {
            for reverse in [false, true] {
                let selection = |limit| Selection {
                    sort: Some(sort),
                    reverse,
                    limit: Some(limit),
                    ..Selection::default()
                };
                let everything = files_of(&run_roots(
                    &both,
                    &roots,
                    &query(&[ViewSpec::Files], selection(Bound::All)),
                ));
                for limit in [1, 4, 9, 20] {
                    let top = files_of(&run_roots(
                        &both,
                        &roots,
                        &query(&[ViewSpec::Files], selection(Bound::Limit(limit))),
                    ));
                    let expected: Vec<String> = everything.iter().take(limit).map(key).collect();
                    assert_eq!(
                        top.iter().map(key).collect::<Vec<_>>(),
                        expected,
                        "{sort:?} reverse={reverse} limit={limit}"
                    );
                }
                if sort == SortKey::Name && !reverse {
                    let listed: Vec<String> = everything.iter().map(key).collect();
                    let mut sorted = listed.clone();
                    sorted.sort_by_key(|path| {
                        let (label, rest) = path.split_once('/').expect("labelled");
                        (label.to_owned(), rest.to_owned())
                    });
                    assert_eq!(listed, sorted, "name order is label, then path");
                    assert!(listed[0].starts_with("alpha/"), "{listed:?}");
                }
            }
        }
    }

    /// Several roots under one tree: every root is a row whatever its share, `--depth`
    /// counts below each root, `--breadth` never hides a root, shares are of the total, and
    /// the row limit applies once with the total as the first row.
    #[test]
    fn a_tree_over_several_roots_bounds_once_and_shows_every_root() {
        use EntryKind::{Dir, File};
        // `big/x` is 1.5% of `big`, but 0.5% of the total, so the 1% default omits it.
        let big = rooted("/big", &[("x", File, 15, 1), ("y", File, 985, 2)]);
        let other = rooted("/other", &[("d", Dir, 0, 3), ("d/z", File, 2_000, 4)]);
        let empty = rooted("/empty", &[]);
        let roots = labelled(&[("big", "/big"), ("other", "/other"), ("empty", "/empty")]);
        let all = [&big, &other, &empty];
        // Apparent bytes, so each file's size is exactly what the shares above say.
        let tree = |selection: Selection| {
            let selection = Selection { size: SizeMetric::Apparent, ..selection };
            run_roots(&all, &roots, &query(&[ViewSpec::Tree], selection))
        };
        let names = |trees: &RootTrees| {
            trees.trees.iter().map(|tree| tree.tree.name.clone()).collect::<Vec<_>>()
        };

        let report = tree(Selection::default());
        let (trees, top) = root_trees_of(&report);
        assert!(top.is_empty());
        assert_eq!(names(trees), ["other", "big", "empty"], "by size, the empty root included");
        let big_tree = &trees.trees[1].tree;
        assert_eq!(trees.trees[1].root, 0);
        assert_eq!(
            big_tree.children.iter().map(|child| child.name.as_str()).collect::<Vec<_>>(),
            ["y"],
            "x is under 1% of the total though not of its root"
        );
        assert!(
            TreeRemainder::from_tree(Some(big_tree), &[]).is_some(),
            "its remainder is its own"
        );
        assert_eq!(trees.trees[2].tree.bytes, 0);

        let reversed = tree(Selection { reverse: true, ..Selection::default() });
        assert_eq!(names(root_trees_of(&reversed).0), ["empty", "big", "other"]);
        let by_name = tree(Selection { sort: Some(SortKey::Name), ..Selection::default() });
        assert_eq!(names(root_trees_of(&by_name).0), ["big", "empty", "other"]);

        let shallow = tree(Selection { depth: Some(Bound::Limit(0)), ..Selection::default() });
        let (trees, _) = root_trees_of(&shallow);
        assert_eq!(trees.trees.len(), 3, "depth counts below each root");
        assert!(trees.trees.iter().all(|tree| tree.tree.children.is_empty()));
        let narrow = tree(Selection { breadth: Some(Bound::Limit(1)), ..unbounded() });
        assert_eq!(root_trees_of(&narrow).0.trees.len(), 3, "breadth bounds no root");

        let only_total = tree(Selection { limit: Some(Bound::Limit(1)), ..Selection::default() });
        let (trees, top) = root_trees_of(&only_total);
        assert!(trees.total.is_some() && trees.trees.is_empty());
        assert_eq!((top.len(), top[0].entries, top[0].reason), (1, 3, TreeOmissionReason::Rows));
        assert_eq!(top[0].bytes, Some(3_000));
        let first = tree(Selection { limit: Some(Bound::Limit(2)), ..Selection::default() });
        let (trees, top) = root_trees_of(&first);
        assert_eq!(names(trees), ["other"]);
        assert!(trees.trees[0].tree.children.is_empty(), "its children are past the cap");
        assert_eq!(top[0].entries, 2);
        let none = tree(Selection { limit: Some(Bound::Limit(0)), ..Selection::default() });
        let (trees, top) = root_trees_of(&none);
        assert!(trees.total.is_none() && trees.trees.is_empty());
        assert_eq!((top[0].entries, top[0].bytes), (3, Some(3_000)));
    }

    /// A metric sort measures no root, so roots fall back to their labels, as rows with no
    /// value sort last among rows that have one.
    #[test]
    fn a_metric_sort_orders_roots_by_label() {
        let (mut docs, mut src, _) = two_roots();
        let code = AnalysisSet::NONE.with_code();
        for index in [&mut docs, &mut src] {
            index.prepare_content_analysis(crate::content::AnalysisRequest {
                profile: code,
                ..crate::content::AnalysisRequest::default()
            });
        }
        let by_lines =
            Selection { sort: Some(SortKey::Metric("code_lines")), ..Selection::default() };
        let mut request =
            crate::test_support::read_of(&src, query(&[ViewSpec::Tree], by_lines.clone()));
        request.basis.content = code;
        // `src` is the larger root, so a size sort lists it first; by label it is second.
        let roots = labelled(&[("src", "/src"), ("docs", "/docs")]);
        let names = |report: &Report| {
            root_trees_of(report)
                .0
                .trees
                .iter()
                .map(|tree| tree.tree.name.clone())
                .collect::<Vec<_>>()
        };
        let report =
            report_roots(&[&src, &docs], &roots, &request, generated_at()).expect("sorted");
        assert_eq!(names(&report), ["docs", "src"]);
        request.query.selection = Selection { sort: None, ..by_lines };
        let report = report_roots(&[&src, &docs], &roots, &request, generated_at()).expect("sized");
        assert_eq!(names(&report), ["src", "docs"]);
    }

    #[test]
    fn report_roots_over_one_root_is_the_single_root_report() {
        let index = sample();
        let one = labelled(&[("root", "/root")]);
        for views in
            [&[ViewSpec::Tree][..], &[ViewSpec::Files], &[ViewSpec::Summary, ViewSpec::Extensions]]
        {
            let query = query(views, Selection::default());
            let request = crate::test_support::read_of(&index, query);
            let alone = report(&index, &request, generated_at()).expect("one");
            let through = report_roots(&[&index], &one, &request, generated_at()).expect("roots");
            for format in [crate::report_format::Format::Json, crate::report_format::Format::Text] {
                assert_eq!(
                    crate::report_format::render(&through, format, false).expect("render"),
                    crate::report_format::render(&alone, format, false).expect("render"),
                    "{views:?} {format:?}"
                );
            }
            assert!(through.roots.is_none() && through.root.is_some());
        }
    }

    #[test]
    fn report_roots_refuses_indexes_that_are_not_the_roots() {
        let (docs, src, roots) = two_roots();
        let request =
            crate::test_support::read_of(&docs, query(&[ViewSpec::Tree], Selection::default()));
        for indexes in [&[&src, &docs][..], &[&docs]] {
            assert!(matches!(
                report_roots(indexes, &roots, &request, generated_at()),
                Err(crate::Error::InvalidRequest(RequestError::RootMismatch { .. }))
            ));
        }
    }

    /// Indexes opened under different scan scopes have no one scope to report under, and
    /// are refused rather than reported under the first one's (review B4 on #192).
    #[test]
    fn report_roots_refuses_indexes_under_different_scopes() {
        let (docs, _, roots) = two_roots();
        let shallow =
            Index::new_with_scope("/src", ScanScope { max_depth: Some(1), ..ScanScope::default() });
        let request =
            crate::test_support::read_of(&docs, query(&[ViewSpec::Tree], Selection::default()));
        assert!(matches!(
            report_roots(&[&docs, &shallow], &roots, &request, generated_at()),
            Err(crate::Error::InvalidRequest(RequestError::RootScopesDiffer { first, other }))
                if first == Path::new("/docs") && other == Path::new("/src")
        ));
    }

    /// A flat row's size is a layout decision: the root index rides in padding today, and
    /// the next 8-byte field would grow every row a walk builds by a ninth (review C13 on
    /// #192). Pinned where the layout is known.
    #[cfg(all(unix, target_pointer_width = "64"))]
    #[test]
    fn a_flat_row_keeps_its_size() {
        assert_eq!(std::mem::size_of::<FileRow>(), 176);
    }

    /// Coverage over several roots adds its counts and keeps each refusal's root, and a
    /// note naming a refused file's directory puts that root's label before it.
    #[test]
    fn ignore_coverage_over_several_roots_adds_and_labels_its_refusals() {
        use crate::control::{
            ControlLimits, ControlObservation, ControlRefusalReason, RefusedControl,
        };
        let observed = |refused: Vec<&str>| {
            ControlCoverage::Observed(ControlObservation {
                limits: ControlLimits::default(),
                applied: 2,
                rules: 5,
                refused: refused.len() as u64,
                refusals: refused
                    .into_iter()
                    .map(|path| RefusedControl {
                        root: 0,
                        path: PathBuf::from(path),
                        reason: ControlRefusalReason::Budget,
                    })
                    .collect(),
            })
        };
        let merged =
            merge_ignore_rules(vec![observed(vec!["a/.gitignore"]), observed(vec![".gitignore"])]);
        let ControlCoverage::Observed(merged) = merged else { panic!("observed") };
        assert_eq!((merged.applied, merged.rules, merged.refused), (4, 10, 2));
        assert_eq!(
            merged
                .refusals
                .iter()
                .map(|refusal| (refusal.root, refusal.path.clone()))
                .collect::<Vec<_>>(),
            [(0, PathBuf::from("a/.gitignore")), (1, PathBuf::from(".gitignore"))]
        );
        let roots = labelled(&[("docs", "/docs"), ("src", "/src")]);
        let (note, _) = refused_controls_note(
            &ControlCoverage::Observed(merged),
            &AxisNames::FLAGS,
            Some(roots.as_slice()),
        )
        .expect("a note");
        // Joined as every text path is, so with `\` on Windows.
        let affected = format!("affected: {}, src", Path::new("docs").join("a").display());
        assert!(note.ends_with(&affected), "{note}");
        let many: Vec<String> = (0..40).map(|i| format!("d{i}/.gitignore")).collect();
        let many: Vec<&str> = many.iter().map(String::as_str).collect();
        let ControlCoverage::Observed(bounded) =
            merge_ignore_rules(vec![observed(many.clone()), observed(many)])
        else {
            panic!("observed")
        };
        assert_eq!((bounded.refused, bounded.refusals.len()), (80, crate::MAX_RETAINED_ISSUES));
        assert!(!bounded.lists_every_refusal());
        assert_eq!(
            merge_ignore_rules(vec![ControlCoverage::NotObserved, ControlCoverage::NotObserved]),
            ControlCoverage::NotObserved
        );
    }

    /// Status over several roots: complete when every root is, the first incomplete root's
    /// coverage, errors in root order within the retention bound, every drop counted.
    #[test]
    fn status_over_several_roots_merges_in_root_order_within_the_bound() {
        let failed = |paths: &[&str], omitted| TreeStatus {
            complete: false,
            coverage: crate::Coverage::Partial(crate::CoverageReason::Inaccessible),
            errors: paths
                .iter()
                .map(|path| {
                    crate::query::StatusIssue::of_one_root(crate::Issue::provider_failure(
                        Some(Path::new(path)),
                        "denied".to_owned(),
                    ))
                })
                .collect(),
            errors_omitted: omitted,
        };
        let complete = TreeStatus {
            complete: true,
            coverage: crate::Coverage::Complete,
            errors: Vec::new(),
            errors_omitted: 0,
        };
        let merged = TreeStatus::merge(vec![complete.clone(), failed(&["a", "b"], 3)]);
        assert!(!merged.complete);
        assert_eq!(merged.coverage, crate::Coverage::Partial(crate::CoverageReason::Inaccessible));
        assert_eq!(merged.errors.iter().map(|detail| detail.root).collect::<Vec<_>>(), [1, 1]);
        assert_eq!(merged.errors_omitted, 3);
        let paths: Vec<String> = (0..40).map(|i| format!("p{i:02}")).collect();
        let paths: Vec<&str> = paths.iter().map(String::as_str).collect();
        let bounded = TreeStatus::merge(vec![failed(&paths, 1), failed(&paths, 2)]);
        assert_eq!(bounded.errors.len(), crate::MAX_RETAINED_ISSUES);
        assert_eq!(bounded.errors[39].root, 0);
        assert_eq!(bounded.errors[40].root, 1);
        assert_eq!(bounded.errors_omitted, 1 + 2 + (80 - 64));
        assert!(TreeStatus::merge(vec![complete.clone(), complete]).complete);
    }
}
