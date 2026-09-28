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
use crate::index::{EntryId, ExtTally, Index, RollUpScalars};
use crate::query::query_request::{Basis, Request};
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

    /// The view a request displays its analysis in when the caller named none.
    ///
    /// A view may never enable an analyzer — that would let a display choice authorize
    /// filesystem reads — but the reverse is free, because it re-projects state already
    /// paid for. Without this, a request that reads every eligible file reports a
    /// directory tree containing none of the results.
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
        let Some(spec) = spec else {
            return Ok((
                if analysis.includes_code() && analysis.includes_words() {
                    vec![Self::Code, Self::Documents]
                } else {
                    vec![Self::default_for(analysis)]
                },
                Vec::new(),
            ));
        };

        let mut parsed: Vec<Self> = Vec::new();
        let mut full_seen = false;
        for raw in spec.split(',') {
            let token = raw.trim();
            if token.is_empty() {
                return Err(Rejection::new(spec, "empty entry in the list"));
            }
            if token.eq_ignore_ascii_case("full") {
                if full_seen || !parsed.is_empty() {
                    return Err(Rejection::new("full", Self::FULL_IS_EXCLUSIVE));
                }
                full_seen = true;
                continue;
            }
            if full_seen {
                return Err(Rejection::new("full", Self::FULL_IS_EXCLUSIVE));
            }
            let view = Self::parse(token).map_err(|expected| Rejection::new(token, expected))?;
            if parsed.contains(&view) {
                return Err(Rejection::new(spec, format!("{token:?} appears more than once")));
            }
            parsed.push(view);
        }

        if full_seen {
            return Ok(Self::full_report(analysis));
        }
        Ok((parsed, Vec::new()))
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
    /// omission rather than drop it silently.
    pub fn full_report(analysis: AnalysisSet) -> (Vec<Self>, Vec<Self>) {
        Self::ALL
            .into_iter()
            .filter(|view| {
                view.is_summary_view()
                    && (!analysis.includes_code() || !matches!(view, Self::Languages))
            })
            .partition(|view| {
                (!matches!(view, Self::Documents) || analysis.is_enabled())
                    && (!matches!(view, Self::Code) || analysis.includes_code())
            })
    }

    /// Whether this view belongs in `--view full`.
    ///
    /// `full` is every *summary* view. `files` is an unbounded enumeration, and putting
    /// one inside a digest destroys the digest.
    pub const fn is_summary_view(self) -> bool {
        !matches!(self, Self::List | Self::Files)
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
    /// The request to repeat the answer as a watch.
    pub watch: &'static str,
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
        watch: "--watch",
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
        watch: "watch",
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

/// One directory's row in a tree view.
#[derive(Clone, Debug)]
pub struct TreeNode {
    /// Path relative to the index root; empty for the root itself.
    pub path: PathBuf,
    /// Final path component, or `.` for the root.
    pub name: String,
    /// What the entry is.
    pub kind: EntryKind,
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
    /// Newest modification time in this subtree, when it holds any files.
    pub newest_mtime_ns: Option<i64>,
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
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct MetricShare {
    /// Selected size contributed by this row.
    pub numerator: u64,
    /// Selected size across every row before display truncation.
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
    /// Path relative to the index root.
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
        /// The bounded directory roll-ups.
        root: Option<Box<TreeNode>>,
        /// The omitted root when the section row limit is zero.
        omissions: Vec<TreeOmission>,
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
    /// Absolute path of the indexed root.
    pub root: PathBuf,
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
/// Only what the request, the resolved views, and the index's coverage can establish. The
/// CLI also prints a note quoting how many bytes analysis read, which is walk telemetry the
/// report envelope does not carry, so that one stays with the performance footer where the
/// rest of the run's telemetry lives.
pub(crate) fn display_notes(
    query: &Query,
    ignore_rules: &ControlCoverage,
) -> (Vec<String>, Vec<String>) {
    let mut notes = Vec::new();
    let mut tips = Vec::new();
    if !query.omitted_views.is_empty() {
        let names: Vec<&str> = query.omitted_views.iter().map(|view| view.label()).collect();
        notes.push(format!("note: omitted {}: content analysis required", names.join(", ")));
        let analysis = if query.omitted_views.contains(&ViewSpec::Code) { "code" } else { "lines" };
        tips.push(format!("tip: include omitted views: add {} {analysis}", query.axes.analyze));
    }
    if let Some((note, tip)) = refused_controls_note(ignore_rules, query.axes) {
        notes.push(note);
        tips.extend(tip);
    }
    (notes, tips)
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
        .map(|refusal| match refusal.path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.display().to_string(),
            _ => ".".to_string(),
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
    // What this index holds is what it can be read for. Every surface validates before it
    // scans, in the vocabulary its own caller uses; this is the library path, and the last
    // one, so nothing produces an answer from an unvalidated request.
    request.validate_read(&Basis::held_by(index)).map_err(crate::Error::InvalidRequest)?;

    let query = &request.query;
    let content = request.basis.content;
    // Share subtree measurements between selection predicates and partial-tree proof.
    // Complete unfiltered metadata reports keep their retained-rollup fast path.
    let needs_walk = query.needs_selection_walk();
    let needs_tree_measurements = query.views.iter().any(|view| query.tree_for(*view))
        && !query.min_share_for().admits(0, 1)
        && (index.state().coverage != crate::Coverage::Complete
            || index.scope().max_depth.is_some());
    let directories = (needs_tree_measurements
        || (needs_walk
            && (query.selection.kinds.is_empty()
                || query.selection.kinds.contains(&EntryKind::Dir))))
    .then(|| query_subtrees::measure(index, &query.selection, identity));
    // One traversal serves every filtered view in the request.
    let walked = needs_walk.then(|| walk(index, &query.selection, identity, directories.as_ref()));
    let tree_measurements = directories.as_ref().filter(|_| needs_tree_measurements);
    // Unfiltered metric and file views share one `FileRow` walk only when more than one
    // section consumes it. A single section keeps ownership of its one traversal, so a
    // bounded file view does not clone every path before sorting and truncating it.
    // Summary, Tree, and Extensions keep roll-ups when unfiltered and do not consume rows.
    let row_consumers =
        query.views.iter().copied().filter(|view| needs_unfiltered_entry_rows(*view)).count();
    let unfiltered_rows = (walked.is_none() && row_consumers > 1).then(|| every_entry(index));
    let metric_consumers =
        query.views.iter().copied().filter(|view| needs_metric_resolution(*view)).count();
    let mut shared_metric_summaries = (walked.is_none() && metric_consumers > 1).then(|| {
        metric_summaries(
            &query.views,
            index,
            query,
            content,
            unfiltered_rows
                .as_deref()
                .expect("multiple metric views share their unfiltered entry rows"),
        )
    });

    // A plain loop, not an iterator chain: this is the deepest path a report takes, and
    // in a debug build each adapter between here and `build_section` is a frame of its
    // own, which is what put a deep tree past the stack `deep_rendering_is_stack_safe`
    // allows on Windows.
    let mut sections: Vec<Section> = Vec::with_capacity(query.views.len());
    for (position, view) in query.views.iter().copied().enumerate() {
        let shared_metric_summary =
            shared_metric_summaries.as_mut().and_then(|summaries| summaries[position].take());
        sections.push(build_section(
            view,
            index,
            query,
            content,
            walked.as_ref(),
            unfiltered_rows.as_deref(),
            tree_measurements,
            shared_metric_summary,
        ));
    }

    let age_reference_ns = crate::query::system_time_to_nanos(request.now);
    for section in &mut sections {
        if let Section::Files { rows, .. } = section {
            for row in rows {
                // An incomplete subtree's mtime is a lower bound, and a lower-bound
                // maximum is not an age: the activity that would make the directory
                // younger may sit in the part that was never listed.
                row.age_ns = match row.complete {
                    Some(false) => None,
                    Some(true) | None => {
                        age_reference_ns.map(|now| i128::from(now) - i128::from(row.mtime_ns))
                    }
                };
            }
        }
    }
    let ignore_rules = index.control_coverage();
    let (mut notes, mut tips) = display_notes(query, &ignore_rules);
    if content.is_enabled()
        && query.selection.sort.is_none_or(|sort| !matches!(sort, SortKey::Metric(_)))
        && !query.views.iter().any(|view| {
            matches!(
                view,
                ViewSpec::Types
                    | ViewSpec::Families
                    | ViewSpec::Languages
                    | ViewSpec::Code
                    | ViewSpec::Documents
            )
        })
    {
        notes.push("note: requested analysis is not displayed by the selected views".to_owned());
        tips.push(format!("tip: show analysis: {} families, languages, or full", query.axes.view));
    }
    if matches!(&ignore_rules, ControlCoverage::Observed(observed) if observed.refusals.len() > REFUSED_DIRECTORIES_NAMED)
    {
        tips.push(format!("tip: show retained ignore-file details: {} json", query.axes.format));
    }
    if tree_measurements.is_some_and(|values| values.values().any(|value| !value.complete)) {
        notes.push("note: incomplete subtrees remain visible below the size threshold".to_owned());
    }
    if index.observes_controls() && !index.ignored_classification_complete_below(Path::new("")) {
        notes.push(
            "note: ignored subtotals are unavailable where governing rules could not be verified"
                .to_owned(),
        );
    }
    Ok(Report {
        age_reference_ns,
        format: query.format,
        notes,
        tips,
        axes: query.axes,
        status: TreeStatus::of(index, request),
        provenance: ReportProvenance::of(index, content, generated_at),
        scope: index.scope(),
        requested_analysis: content,
        requested_views: query.views.clone(),
        omitted_views: query.omitted_views.clone(),
        root: index.root_path().to_path_buf(),
        size: query.selection.size,
        sort_metric: match query.selection.sort {
            Some(SortKey::Metric(name)) => Some(name),
            _ => None,
        },
        analysis: index.content().and_then(|held| {
            let wanted = index.content_identity(content);
            let projected = held.admit(&wanted)?;
            Some(ContentReportMetadata {
                profile: projected.identity().analysis,
                provenance: projected.identity().record_provenance(),
            })
        }),
        ignored_entries: query.selection.ignored,
        ignore_rules,
        sections,
    })
}

/// Build a one-section report from an already reduced exact summary.
///
/// Pure for the same reason as [`report`]: scanning and time sampling happened before
/// this boundary.  The execution planner uses this when a one-shot request proves that
/// retaining paths and hierarchy cannot affect its answer.
pub(crate) fn report_summary(
    root: &Path,
    scope: ScanScope,
    request: &Request,
    summary: SummaryRow,
    status: TreeStatus,
    provenance: ReportProvenance,
) -> Report {
    Report {
        age_reference_ns: crate::query::system_time_to_nanos(request.now),
        format: request.query.format,
        // A compact summary resolves one view and drops none.
        notes: Vec::new(),
        tips: Vec::new(),
        axes: request.query.axes,
        status,
        provenance,
        scope,
        requested_analysis: AnalysisSet::NONE,
        requested_views: vec![ViewSpec::Summary],
        omitted_views: Vec::new(),
        root: root.to_path_buf(),
        size: request.query.selection.size,
        sort_metric: None,
        // The planner only selects this tier when no analysis was requested, so there is
        // no analyzer provenance to report.
        analysis: None,
        // An unfiltered summary selects every entry.
        ignored_entries: IgnoredEntries::Include,
        // Nor when control state is observed, since it retains no table to classify with.
        ignore_rules: ControlCoverage::NotObserved,
        sections: vec![Section::Summary(SummaryRow { ignored: None, ..summary })],
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
    per_directory: BTreeMap<EntryId, SummaryRow>,
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
        let mut row = self.per_directory.get(&id).copied().unwrap_or_default();
        row.ignored = (self.observed && !self.unknown_ignored.contains(&id))
            .then(|| row.ignored.unwrap_or_default());
        row
    }
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

    // (id, path, post-order, covered by a selected ancestor)
    let mut stack = vec![(EntryId::ROOT, PathBuf::new(), false, false)];
    while let Some((id, path, expanded, covered)) = stack.pop() {
        if expanded {
            // Post-order: every child has finished, so fold their totals into this one.
            // `total` already carries this directory's own admitted files and admitted
            // directory children, both tallied in the pre-order pass below; what is left
            // is to add what each child subtree found deeper down.
            let mut total = walked.per_directory.remove(&id).unwrap_or_default();
            if let Some(children) = index.children_of(id) {
                for (_, child) in children {
                    if let Some(sub) = walked.per_directory.get(&child) {
                        let sub = *sub;
                        merge_summary(&mut total, &sub);
                        if walked.unknown_ignored.contains(&child) {
                            walked.unknown_ignored.insert(id);
                        }
                    }
                }
            }
            if total.files > 0 || total.dirs > 0 {
                walked.visible.insert(id);
            }
            walked.per_directory.insert(id, total);
            continue;
        }

        stack.push((id, path.clone(), true, covered));
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
            if matches || (covered && classification_admitted && selection.ignored.admits(ignored))
            {
                if classification.is_none() {
                    walked.unknown_ignored.insert(id);
                }
                if kind == EntryKind::File {
                    walked.members.push(row);
                    walked.visible_files.insert(child);
                } else if kind == EntryKind::Dir {
                    walked.visible.insert(child);
                }

                if kind == EntryKind::File {
                    let own = walked.per_directory.entry(id).or_default();
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
                    let own = walked.per_directory.entry(id).or_default();
                    own.dirs += 1;
                    if ignored {
                        own.ignored.get_or_insert_with(IgnoredTally::default).dirs += 1;
                    }
                }
            }

            if kind == EntryKind::Dir {
                stack.push((child, child_path, false, covered || matches));
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

/// Build one view's section, using the pre-computed tier when the selection allows.
#[allow(clippy::too_many_arguments)] // Each shared input is computed once per request.
fn build_section(
    view: ViewSpec,
    index: &Index,
    query: &Query,
    content: AnalysisSet,
    walked: Option<&Walked>,
    unfiltered_rows: Option<&[FileRow]>,
    tree_measurements: Option<&BTreeMap<EntryId, query_subtrees::SubtreeValues>>,
    shared_metric_summary: Option<Box<MetricSummary>>,
) -> Section {
    if query.tree_for(view) {
        let (root, omissions) = tree_node(index, query, content, walked, tree_measurements);
        let limits = TreeDisplayLimits {
            depth: query.depth_for(view),
            min_share: query.min_share_for(),
            breadth: query.breadth_for(),
            rows: query.limit_for(view),
        };
        return Section::Tree { view, limits, root: root.map(Box::new), omissions };
    }
    match view {
        ViewSpec::Code => {
            Section::Code(Box::new(code_overview(index, query, content, walked, unfiltered_rows)))
        }
        ViewSpec::Summary => Section::Summary(match walked {
            None => unfiltered_summary(index, EntryId::ROOT, Path::new("")),
            Some(walked) => walked.summary_of(EntryId::ROOT),
        }),
        ViewSpec::Extensions => {
            let (rows, total, share_omitted) = extension_rows(index, query, walked);
            Section::Extensions { rows, total, share_omitted }
        }
        ViewSpec::Types | ViewSpec::Families | ViewSpec::Languages | ViewSpec::Documents => {
            Section::Metrics {
                view,
                summary: shared_metric_summary.unwrap_or_else(|| {
                    Box::new(metric_summary(view, index, query, content, walked, unfiltered_rows))
                }),
            }
        }
        ViewSpec::List
        | ViewSpec::Tree
        | ViewSpec::Files
        | ViewSpec::Largest
        | ViewSpec::Recent => {
            let (rows, total) = file_rows(view, index, query, content, walked, unfiltered_rows);
            Section::Files { view, rows, total }
        }
    }
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

/// Rows for the types view.
fn extension_rows(
    index: &Index,
    query: &Query,
    walked: Option<&Walked>,
) -> (Vec<TypeRow>, usize, usize) {
    // Extension partitions cannot attribute an unknown member to one bucket from the
    // roll-up alone, so withhold their ignored subtotals until the scope is known.
    let observed = match walked {
        Some(walked) => walked.observed && !walked.unknown_ignored.contains(&EntryId::ROOT),
        None => {
            index.observes_controls() && index.ignored_classification_complete_below(Path::new(""))
        }
    };
    let (tallies, ignored): (BTreeMap<String, ExtTally>, BTreeMap<String, ExtTally>) = match walked
    {
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
        let root = match walked {
            None => unfiltered_summary(index, EntryId::ROOT, Path::new("")),
            Some(walked) => walked.summary_of(EntryId::ROOT),
        };
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
            name: |row: &TypeRow| row.extension.clone(),
            content_metric: |_: &TypeRow, _: &MetricDef| None,
        },
    );
    let total = truncate(&mut rows, query.limit_for(ViewSpec::Extensions));
    (rows, total, before_share - total)
}

fn metric_summary(
    view: ViewSpec,
    index: &Index,
    query: &Query,
    content: AnalysisSet,
    walked: Option<&Walked>,
    unfiltered_rows: Option<&[FileRow]>,
) -> MetricSummary {
    let mut accumulator = MetricAccumulator::new(view);
    let files = walked.map_or_else(
        || entry_rows(index, None, unfiltered_rows),
        |walked| Cow::Borrowed(walked.members.as_slice()),
    );
    let wanted = index.content_identity(content);
    let held = index.content().and_then(|held| held.admit(&wanted));
    for file in files.iter().filter(|row| row.kind == EntryKind::File) {
        let cached = held.and_then(|content| content.file(&file.path));
        let classification = index.classify(&file.path);
        accumulator.push(content, file, cached, &classification);
    }
    accumulator.finish(query, content)
}

/// Build several unfiltered metric sections in one file pass.
fn metric_summaries(
    views: &[ViewSpec],
    index: &Index,
    query: &Query,
    content: AnalysisSet,
    rows: &[FileRow],
) -> Vec<Option<Box<MetricSummary>>> {
    let mut accumulators = views
        .iter()
        .copied()
        .filter(|view| needs_metric_resolution(*view))
        .map(MetricAccumulator::new)
        .collect::<Vec<_>>();
    let wanted = index.content_identity(content);
    let held = index.content().and_then(|held| held.admit(&wanted));
    for file in rows.iter().filter(|row| row.kind == EntryKind::File) {
        let cached = held.and_then(|content| content.file(&file.path));
        let classification = index.classify(&file.path);
        for accumulator in &mut accumulators {
            accumulator.push(content, file, cached, &classification);
        }
    }

    // Boxed where each is finished: a summary is a few hundred bytes, and carrying it by
    // value through the per-view closure and `build_section` put those bytes in every
    // frame on the path a deep tree renders on, which on a Windows debug build was enough
    // to overflow the 64 KiB stack `deep_rendering_is_stack_safe` allows.
    let mut finished =
        accumulators.into_iter().map(|accumulator| Box::new(accumulator.finish(query, content)));
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
        let denominator = share_value(&total, share_metric);
        total.share = MetricShare { numerator: denominator, denominator };
        let mut rows = grouped.into_values().collect::<Vec<_>>();
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
                name: |row: &MetricRow| row.id.clone(),
                content_metric: MetricRow::metric_value,
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

fn code_overview(
    index: &Index,
    query: &Query,
    content: AnalysisSet,
    walked: Option<&Walked>,
    unfiltered_rows: Option<&[FileRow]>,
) -> CodeOverview {
    #[derive(Default)]
    struct SortFacts {
        apparent: u64,
        allocated: u64,
        newest_mtime_ns: Option<i64>,
        metric: MetricAggregate,
    }

    let files = walked.map_or_else(
        || entry_rows(index, None, unfiltered_rows),
        |walked| Cow::Borrowed(walked.members.as_slice()),
    );
    let held = index.content().and_then(|tier| tier.admit(&index.content_identity(content)));
    let split = query.selection.ignored == IgnoredEntries::Include && index.observes_controls();
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
    for file in files.iter().filter(|row| row.kind == EntryKind::File) {
        let record = held.and_then(|tier| tier.file(&file.path));
        let classification = record
            .map_or_else(|| index.classify(&file.path).into(), |record| record.detection.clone());
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

/// Rows for the files view.
fn file_rows(
    view: ViewSpec,
    index: &Index,
    query: &Query,
    content: AnalysisSet,
    walked: Option<&Walked>,
    unfiltered_rows: Option<&[FileRow]>,
) -> (Vec<FileRow>, usize) {
    let mut rows = entry_rows(index, walked, unfiltered_rows).into_owned();
    if view.files_only() {
        rows.retain(|row| row.kind == EntryKind::File);
    }
    if let Some(SortKey::Metric(name)) = query.selection.sort {
        let sources = walked.map_or(rows.as_slice(), |walked| walked.members.as_slice());
        let values = metric_sort_values(index, content, sources, name);
        for row in &mut rows {
            row.sort_value = values.get(&row.path).copied();
        }
    }

    sort_rows(
        &mut rows,
        query,
        view,
        SortAccessors {
            size: |row: &FileRow, metric| match metric {
                SizeMetric::Apparent => row.bytes,
                SizeMetric::Allocated => row.allocated,
            },
            count: |row: &FileRow| row.files.unwrap_or(1),
            mtime: |row: &FileRow| Some(row.mtime_ns),
            name: |row: &FileRow| row.path.to_string_lossy().into_owned(),
            content_metric: |row: &FileRow, _: &MetricDef| row.sort_value,
        },
    );
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

/// The tree view's root node, expanded to the requested depth.
fn tree_node(
    index: &Index,
    query: &Query,
    content: AnalysisSet,
    walked: Option<&Walked>,
    tree_measurements: Option<&BTreeMap<EntryId, query_subtrees::SubtreeValues>>,
) -> (Option<TreeNode>, Vec<TreeOmission>) {
    let unfiltered = (walked.is_none() && matches!(query.selection.sort, Some(SortKey::Metric(_))))
        .then(|| every_entry(index));
    let metric_values = if let Some(SortKey::Metric(name)) = query.selection.sort {
        let sources = walked.map_or_else(
            || unfiltered.as_deref().unwrap_or(&[]),
            |walked| walked.members.as_slice(),
        );
        metric_sort_values(index, content, sources, name)
    } else {
        BTreeMap::new()
    };
    let root_summary = match walked {
        None => unfiltered_summary(index, EntryId::ROOT, Path::new("")),
        Some(walked) => walked.summary_of(EntryId::ROOT),
    };

    let mut root = TreeNode {
        path: PathBuf::new(),
        name: ".".to_string(),
        kind: EntryKind::Dir,
        bytes: root_summary.bytes,
        allocated: root_summary.allocated,
        files: root_summary.files,
        dirs: root_summary.dirs,
        ignored: root_summary.ignored,
        newest_mtime_ns: root_summary.newest_mtime_ns,
        children: Vec::new(),
        omissions: Vec::new(),
        truncated: false,
    };
    if query.limit_for(ViewSpec::Tree) == Bound::Limit(0) {
        let complete = index.state().coverage == crate::Coverage::Complete;
        let omitted = TreeOmission {
            reason: TreeOmissionReason::Rows,
            entries: 1,
            files: complete.then_some(root.files),
            bytes: complete.then_some(root.bytes),
            allocated: complete.then_some(root.allocated),
            ignored: complete.then_some(root.ignored).flatten().map(IgnoredSize::from_tally),
        };
        return (None, vec![omitted]);
    }
    expand(index, query, walked, &metric_values, tree_measurements, &mut root);
    if let Some(cap) = query.limit_for(ViewSpec::Tree).limit() {
        root = cap_tree_rows(root, cap, index.state().coverage == crate::Coverage::Complete);
    }
    (Some(root), Vec::new())
}

fn record_omission(
    node: &mut TreeNode,
    reason: TreeOmissionReason,
    rows: &[(TreeNode, EntryId)],
    complete: bool,
) {
    if rows.is_empty() {
        return;
    }
    let bytes = complete
        .then(|| rows.iter().try_fold(0u64, |sum, (row, _)| sum.checked_add(row.bytes)))
        .flatten();
    let allocated = complete
        .then(|| rows.iter().try_fold(0u64, |sum, (row, _)| sum.checked_add(row.allocated)))
        .flatten();
    let files = complete
        .then(|| rows.iter().try_fold(0u64, |sum, (row, _)| sum.checked_add(row.files)))
        .flatten();
    let ignored = complete
        .then(|| {
            rows.iter().try_fold(IgnoredSize::default(), |sum, (row, _)| {
                sum.checked_add(IgnoredSize::from_tally(row.ignored?))
            })
        })
        .flatten();
    node.omissions.push(TreeOmission {
        reason,
        entries: rows.len(),
        files,
        bytes,
        allocated,
        ignored,
    });
    node.truncated = true;
}

fn cap_tree_rows(root: TreeNode, cap: usize, complete: bool) -> TreeNode {
    struct Pending {
        node: TreeNode,
        parent: Option<usize>,
    }
    let mut pending = vec![Pending { node: root, parent: None }];
    let mut kept: Vec<Pending> = Vec::new();
    while let Some(mut item) = pending.pop() {
        if kept.len() == cap {
            let parent = item.parent.expect("root admitted by positive row cap");
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
            let owner = &mut kept[parent].node;
            if let Some(existing) = owner
                .omissions
                .iter_mut()
                .find(|existing| existing.reason == TreeOmissionReason::Rows)
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
                owner.omissions.push(omission);
            }
            owner.truncated = true;
            continue;
        }
        let children = std::mem::take(&mut item.node.children);
        let current = kept.len();
        kept.push(item);
        for child in children.into_iter().rev() {
            pending.push(Pending { node: child, parent: Some(current) });
        }
    }
    for position in (1..kept.len()).rev() {
        let child = kept.remove(position);
        kept[child.parent.expect("only root lacks parent")].node.children.insert(0, child.node);
    }
    kept.pop().expect("positive cap admits root").node
}

/// Attach a node's children, honoring the depth and per-directory limit bounds.
///
/// Iterative rather than recursive: this engine indexes trees deep enough that recursive
/// expansion would exhaust the stack, and a report that panics on a deep tree fails
/// exactly where the tool is most useful. Nodes are built flat with parent links in
/// pre-order, then folded together from the leaves up.
fn expand(
    index: &Index,
    query: &Query,
    walked: Option<&Walked>,
    metric_values: &BTreeMap<PathBuf, u64>,
    tree_measurements: Option<&BTreeMap<EntryId, query_subtrees::SubtreeValues>>,
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
            bytes: node.bytes,
            allocated: node.allocated,
            files: node.files,
            dirs: node.dirs,
            ignored: node.ignored,
            newest_mtime_ns: node.newest_mtime_ns,
            children: Vec::new(),
            omissions: Vec::new(),
            truncated: false,
        },
        id: EntryId::ROOT,
        depth: 0,
        parent: None,
    }];

    let threshold = query.min_share_for();
    let grand = match query.selection.size {
        SizeMetric::Apparent => node.bytes,
        SizeMetric::Allocated => node.allocated,
    };
    let complete = index.state().coverage == crate::Coverage::Complete;
    let mut cursor = 0;
    while cursor < built.len() {
        let (id, depth) = (built[cursor].id, built[cursor].depth);
        let path = built[cursor].node.path.clone();

        let mut rows = child_rows(index, query, walked, metric_values, id, &path);
        let mut below_share = Vec::new();
        rows.retain(|(row, child)| {
            let value = match query.selection.size {
                SizeMetric::Apparent => row.bytes,
                SizeMetric::Allocated => row.allocated,
            };
            // A complete child below a partial root's observed total is also below
            // the true (at least as large) total. Only an incomplete child's own
            // unknown contents prevent that proof; unrelated scan errors do not.
            let child_complete = row.kind == EntryKind::File
                || tree_measurements
                    .is_none_or(|values| values.get(child).is_some_and(|subtree| subtree.complete));
            let eligible = !child_complete || threshold.admits(value, grand);
            if !eligible {
                below_share.push((row.clone(), *child));
            }
            eligible
        });
        record_omission(&mut built[cursor].node, TreeOmissionReason::Share, &below_share, true);
        if !query.depth_for(ViewSpec::Tree).admits(depth) {
            record_omission(&mut built[cursor].node, TreeOmissionReason::Depth, &rows, complete);
            cursor += 1;
            continue;
        }
        if let Some(cap) = query.breadth_for().limit() {
            let hidden = rows.split_off(cap.min(rows.len()));
            record_omission(
                &mut built[cursor].node,
                TreeOmissionReason::Breadth,
                &hidden,
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

/// The directory children of one node, shaped and sorted but not yet expanded.
fn child_rows(
    index: &Index,
    query: &Query,
    walked: Option<&Walked>,
    metric_values: &BTreeMap<PathBuf, u64>,
    id: EntryId,
    path: &Path,
) -> Vec<(TreeNode, EntryId)> {
    let Some(children) = index.children_of(id) else {
        return Vec::new();
    };
    let children: Vec<(PathBuf, EntryId)> =
        children.map(|(name, child)| (path.join(name), child)).collect();

    let mut rows: Vec<(TreeNode, EntryId)> = Vec::new();
    for (child_path, child) in children {
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
        let summary = if kind == EntryKind::File {
            let attrs = index.attrs_of(child).expect("live child has attributes");
            let ignored = index.ignored_classification(&child_path).map(|ignored| {
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
        let name = child_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        rows.push((
            TreeNode {
                path: child_path,
                name,
                kind,
                bytes: summary.bytes,
                allocated: summary.allocated,
                files: summary.files,
                dirs: summary.dirs,
                ignored: summary.ignored,
                newest_mtime_ns: summary.newest_mtime_ns,
                children: Vec::new(),
                omissions: Vec::new(),
                truncated: false,
            },
            child,
        ));
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
            mtime: |(row, _): &(TreeNode, EntryId)| row.newest_mtime_ns,
            name: |(row, _): &(TreeNode, EntryId)| row.name.clone(),
            content_metric: |(row, _): &(TreeNode, EntryId), _: &MetricDef| {
                metric_values.get(&row.path).copied()
            },
        },
    );
    rows
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
struct SortAccessors<S, C, M, N, V> {
    size: S,
    count: C,
    mtime: M,
    name: N,
    content_metric: V,
}

fn sort_rows<T>(
    rows: &mut [T],
    query: &Query,
    view: ViewSpec,
    accessors: SortAccessors<
        impl Fn(&T, SizeMetric) -> u64,
        impl Fn(&T) -> u64,
        impl Fn(&T) -> Option<i64>,
        impl Fn(&T) -> String,
        impl Fn(&T, &MetricDef) -> Option<u64>,
    >,
) {
    sort_rows_by(rows, query, view, accessors);
}

/// Sort rows by the effective key, with a stable name tiebreak.
fn sort_rows_by<T>(
    rows: &mut [T],
    query: &Query,
    view: ViewSpec,
    accessors: SortAccessors<
        impl Fn(&T, SizeMetric) -> u64,
        impl Fn(&T) -> u64,
        impl Fn(&T) -> Option<i64>,
        impl Fn(&T) -> String,
        impl Fn(&T, &MetricDef) -> Option<u64>,
    >,
) {
    let SortAccessors { size, count, mtime, name, content_metric } = accessors;
    let key = query.selection.sort.unwrap_or_else(|| view.default_sort());
    let metric = query.selection.size;

    rows.sort_by(|left, right| {
        let ordering = match key {
            // Size, count, and recency read most-first: the interesting end is the top.
            SortKey::Size => size(right, metric).cmp(&size(left, metric)),
            SortKey::Count => count(right).cmp(&count(left)),
            SortKey::Mtime => mtime(right).cmp(&mtime(left)),
            SortKey::Name => name(left).cmp(&name(right)),
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
        ordering.then_with(|| name(left).cmp(&name(right)))
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
        let (notes, _) = display_notes(&query, &ControlCoverage::NotObserved);
        assert_eq!(notes.len(), 1, "{notes:?}");
        assert!(notes[0].contains("code") && notes[0].contains("documents"), "{notes:?}");

        // Nothing dropped, nothing said.
        let (selected, omitted) = ViewSpec::resolve(Some("full"), AnalysisSet::ALL, "view")
            .expect("full resolves with analyzers");
        assert!(omitted.is_empty(), "every view is answerable with analysis enabled");
        let query = Query { views: selected, omitted_views: omitted, ..Query::default() };
        assert!(display_notes(&query, &ControlCoverage::NotObserved).0.is_empty());
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
                    path: Path::new("vendor").join(".gitignore"),
                    reason,
                }],
            });
            let (note, tip) = refused_controls_note(&coverage, &AxisNames::FLAGS).expect("refusal");
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
                        path: Path::new(&format!("d{i:02}")).join(".gitignore"),
                        reason: ControlRefusalReason::Budget,
                    })
                    .collect(),
            })
        };
        let (note, tip) =
            refused_controls_note(&coverage(defaults, 1000), &AxisNames::FLAGS).expect("truncated");
        assert!(note.contains("995 more"));
        assert!(!note.contains("d05"));
        let tip = tip.expect("both bounded limits may explain unlisted refusals");
        assert!(tip.contains("--gitignore-budget") && tip.contains("--gitignore-line-limit"));
        let (_, tip) = refused_controls_note(
            &coverage(ControlLimits { line_limit: None, ..defaults }, 1000),
            &AxisNames::FLAGS,
        )
        .expect("truncated");
        assert!(!tip.expect("budget remedy").contains("--gitignore-line-limit"));
        let (_, tip) = refused_controls_note(
            &coverage(ControlLimits { budget: None, line_limit: None }, 64),
            &AxisNames::FLAGS,
        )
        .expect("retained refusal");
        assert!(tip.is_none(), "do not suggest raising unbounded limits");
        assert!(refused_controls_note(&ControlCoverage::NotObserved, &AxisNames::FLAGS).is_none());
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
            let note = display_notes(&query, &ControlCoverage::NotObserved).1.remove(0);
            assert!(note.contains(&format!("add {mine} ")), "{note} must name {mine}");
            assert!(!note.contains(&format!("add {theirs} ")), "{note} must not name {theirs}");
        }

        // The same for the hard error, which names the view axis as well. The rule is the
        // request model's; what a surface reads is this rendering of it.
        for (axes, view, analyze) in
            [(&AxisNames::FLAGS, "--view", "--analyze"), (&AxisNames::FIELDS, "view", "analyze")]
        {
            let error =
                crate::query::RequestError::ViewNeedsContent(ViewSpec::Documents).message(axes);
            assert!(error.starts_with(&format!("{view} documents")), "{error}");
            assert!(error.contains(&format!("add {analyze} ")), "{error}");
            let theirs = if analyze == "--analyze" { "analyze" } else { "--analyze" };
            assert!(!error.contains(&format!("add {theirs} ")), "{error}");
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
        assert!(!diagnostics.notes.iter().any(|note| note.contains("more covers")));
        assert!(diagnostics.notes.iter().any(|note| note.contains("depth 1")));
        assert!(
            diagnostics.tips.contains(&format!("tip: expand deeper: {}=all", report.axes.depth))
        );

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
        assert!(text.contains("—     unknown    … and more files (count unknown)"));
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
        let report = run(&index, &query(&[ViewSpec::Tree], selection.clone()));
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
                report.notes.iter().any(|note| note.contains("ignored subtotals are unavailable"))
            );
            assert!(
                crate::report_format::report_notes(&report)
                    .iter()
                    .all(|note| !note.contains("gitignored sizes are included"))
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
    fn one_pass_metric_summaries_match_independent_views() {
        let root = tempfile::tempdir().expect("root");
        fs::write(root.path().join("main.rs"), "fn main() {}\n").expect("rust");
        fs::write(root.path().join("guide.md"), "# Guide\n\nWords.\n").expect("markdown");
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
            ViewSpec::Summary,
            ViewSpec::Families,
            ViewSpec::Languages,
            ViewSpec::Documents,
        ];
        let query = query(&views, Selection::default());
        let rows = every_entry(&index);
        let summaries = metric_summaries(&views, &index, &query, AnalysisSet::ALL, &rows);

        assert!(summaries[1].is_none(), "non-metric views keep their own projection");
        for (position, view) in
            views.iter().copied().enumerate().filter(|(_, view)| needs_metric_resolution(*view))
        {
            let independent =
                metric_summary(view, &index, &query, AnalysisSet::ALL, None, Some(&rows));
            assert_eq!(
                format!("{:?}", summaries[position].as_ref().expect("metric summary")),
                format!("{independent:?}"),
                "{view:?} changed in the one-pass multi-view aggregation"
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
        assert_eq!(report.root, Path::new("/root"));
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
                notes.iter().filter(|note| note.contains("gitignored sizes are included")).count(),
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
                .all(|note| !note.contains("gitignored sizes are included"))
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
            ("EXCLUDE", &classified, selection(IgnoredEntries::Exclude)),
            ("ONLY", &classified, selection(IgnoredEntries::Only)),
            ("NO CONTROLS", &unobserved, selection(IgnoredEntries::Include)),
            ("REFUSED CONTROL", &refused, selection(IgnoredEntries::Include)),
        ];
        let mut actual = String::new();
        for (label, index, selection) in cases {
            let report = run(index, &query(&[ViewSpec::Tree], selection));
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
}
