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

use crate::classify::{ContentFamily, DetectionConfidence, DetectionSource};
use crate::content::{AnalysisSet, ContentProvenance, CoverageReason, LogicalWordStats, MetricDef};
use crate::control::ControlCoverage;
use crate::engine_contract::{EntryKind, ScanScope};
use crate::index::{EntryId, ExtTally, Index, RollUpScalars};
use crate::query::query_request::{Basis, Request};
use crate::query::query_selection::{
    Bound, IgnoredEntries, NameIdentity, Selection, SizeMetric, SortKey,
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
    /// One row per raw derived extension (the original `types` behavior).
    Extensions,
    /// One row per broad content family.
    Families,
    /// Code-family rows grouped by language/type.
    Languages,
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
            Self::List | Self::Files => Bound::All,
            Self::Largest | Self::Recent => Bound::Limit(20),
            _ => Bound::Limit(10),
        }
    }

    /// How deep a rendered tree descends when the caller named no depth.
    ///
    /// Two levels is what makes `fdu` answer "what is big here" at a glance: the root's
    /// children and theirs. Only the tree renders a hierarchy at all, so every other
    /// view is unbounded and the question does not arise.
    const fn default_depth(self) -> Bound {
        match self {
            Self::Tree => Bound::Limit(2),
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
    pub const ALL: [Self; 11] = [
        Self::List,
        Self::Summary,
        Self::Tree,
        Self::Families,
        Self::Types,
        Self::Extensions,
        Self::Languages,
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
            (true, true) => Self::Families,
            (true, false) => Self::Languages,
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
            return Ok((vec![Self::default_for(analysis)], Vec::new()));
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
            .filter(|view| view.is_summary_view())
            .partition(|view| !matches!(view, Self::Documents) || analysis.is_enabled())
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
    /// `ignored` is the one axis the command line splits into two switches, so it names
    /// both; it only reaches a diagnostic through a value no flag can produce.
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
        exclude_ignored: "--exclude-ignored",
        only_ignored: "--only-ignored",
        read_controls: "--no-gitignore",
        scan_depth: "--scan-depth",
        one_filesystem: "--one-filesystem",
        follow_symlinks: "follow_symlinks",
        include: "--include",
        modified_since: "--modified-since",
        modified_before: "--modified-before",
        kind: "--kind",
        ignored: "--exclude-ignored/--only-ignored",
        depth: "--depth",
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
    /// the index observed no control state.
    ///
    /// Counted over the selected entries, like every other tally on the row, so it is zero
    /// when the selection excludes ignored entries and the whole row when it admits only
    /// them.
    pub ignored: Option<IgnoredTally>,
    /// Newest modification time in this subtree, when it holds any files.
    pub newest_mtime_ns: Option<i64>,
    /// Children reported beneath this node.
    pub children: Vec<TreeNode>,
    /// Whether children were withheld by the depth or limit bound.
    pub truncated: bool,
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
    /// The ignored part of this row, or `None` when the index observed no control state.
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
    /// Metric used for every row's exact share.
    pub share_metric: ShareMetric,
    /// Logical words per derived page.
    pub words_per_page: u64,
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
    /// Whether `.gitignore` rules ignore this entry, or `None` when the index observed no
    /// control state.
    pub ignored: Option<bool>,
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
    /// The ignored part of what was selected, or `None` when the index observed no control
    /// state.
    pub ignored: Option<IgnoredTally>,
    /// Newest modification time, when anything was selected.
    pub newest_mtime_ns: Option<i64>,
}

/// One view's results.
#[derive(Clone, Debug)]
pub enum Section {
    /// A tree view.
    Tree {
        /// The view whose directory hierarchy is shown.
        view: ViewSpec,
        /// The bounded directory roll-ups.
        root: TreeNode,
    },
    /// A raw-extension view.
    Extensions {
        /// The rows, already sorted and bounded.
        rows: Vec<TypeRow>,
        /// Rows before the bound was applied.
        total: usize,
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
    /// Which size metric this report answers in.
    ///
    /// Carried on the report so a renderer shows the same number the ordering used;
    /// printing apparent bytes beside an allocated-bytes ranking looks like a sorting
    /// bug and is worse than either metric alone.
    pub size: SizeMetric,
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
    /// Not operational completeness: a refused control file leaves [`Self::complete`]
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
pub(crate) fn display_notes(query: &Query, ignore_rules: &ControlCoverage) -> Vec<String> {
    let mut notes = Vec::new();
    if !query.omitted_views.is_empty() {
        let names: Vec<&str> = query.omitted_views.iter().map(|view| view.label()).collect();
        notes.push(format!(
            "note: omitted {} — requires content analysis: add {} lines, code, words, or all",
            names.join(", "),
            query.axes.analyze
        ));
    }
    notes.extend(refused_controls_note(ignore_rules, query.axes));
    notes
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
fn refused_controls_note(ignore_rules: &ControlCoverage, axes: &AxisNames) -> Option<String> {
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
        [] => String::new(),
        [raise] => format!(" To apply them, raise {raise}, or set it to all"),
        raises => format!(" To apply them, raise {}, or set them to all", raises.join(" and ")),
    };
    let files = crate::report_format::human_count(observed.refused);
    let noun = if observed.refused == 1 { "file" } else { "files" };
    Some(format!(
        "note: {files} .gitignore {noun} not applied ({why}), so ignored shares under {} are \
         not exact; sizes are.{remedy}",
        directories.join(", ")
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
    // One traversal serves every filtered view in the request, so asking for three views
    // costs one pass rather than three.
    let walked = query.needs_selection_walk().then(|| walk(index, &query.selection, identity));
    // Unfiltered metric and file views share one `FileRow` walk only when more than one
    // section consumes it. A single section keeps ownership of its one traversal, so a
    // bounded file view does not clone every path before sorting and truncating it.
    // Summary, Tree, and Extensions keep roll-ups when unfiltered and do not consume rows.
    let row_consumers =
        query.views.iter().copied().filter(|view| needs_unfiltered_entry_rows(*view)).count();
    let unfiltered_rows = (walked.is_none() && row_consumers > 1).then(|| every_entry(index));

    let mut sections: Vec<Section> = query
        .views
        .iter()
        .map(|view| {
            build_section(*view, index, query, content, walked.as_ref(), unfiltered_rows.as_deref())
        })
        .collect();

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
    Ok(Report {
        age_reference_ns,
        format: query.format,
        notes: display_notes(query, &ignore_rules),
        status: TreeStatus::of(index, request),
        provenance: ReportProvenance::of(index, content, generated_at),
        scope: index.scope(),
        requested_analysis: content,
        requested_views: query.views.clone(),
        omitted_views: query.omitted_views.clone(),
        root: index.root_path().to_path_buf(),
        size: query.selection.size,
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
        status,
        provenance,
        scope,
        requested_analysis: AnalysisSet::NONE,
        requested_views: vec![ViewSpec::Summary],
        omitted_views: Vec::new(),
        root: root.to_path_buf(),
        size: request.query.selection.size,
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
}

impl Walked {
    /// One directory's filtered totals, with an ignored share exactly when observed.
    fn summary_of(&self, id: EntryId) -> SummaryRow {
        let mut row = self.per_directory.get(&id).copied().unwrap_or_default();
        row.ignored = self.observed.then(|| row.ignored.unwrap_or_default());
        row
    }
}

/// One directory's unfiltered totals from the roll-up state the index maintains, with its
/// ignored share, `all` less `unignored`, when the index observed control state.
fn unfiltered_summary(index: &Index, id: EntryId) -> SummaryRow {
    let observed = index.observes_controls();
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
fn walk(index: &Index, selection: &Selection, identity: NameIdentity) -> Walked {
    let observed = index.observes_controls();
    let mut walked = Walked {
        observed,
        per_directory: BTreeMap::new(),
        by_ext: BTreeMap::new(),
        ignored_by_ext: BTreeMap::new(),
        rows: Vec::new(),
        members: Vec::new(),
        visible: BTreeSet::new(),
    };
    // No entry of an index that read no rule can be shown to be ignored or not, so
    // `report_in` refuses a selection by ignored state before it reaches this walk.
    debug_assert!(
        observed || selection.ignored == IgnoredEntries::Include,
        "a selection by ignored state over an unobserving index is refused before the walk"
    );

    // Directory predicates see subtree values even in mixed listings. A file-only
    // selection needs no directory measurements and retains its existing query cost.
    let directories = (selection.kinds.is_empty() || selection.kinds.contains(&EntryKind::Dir))
        .then(|| query_subtrees::measure(index, selection, identity));
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
            let ignored = index.ignored_bit_of(child).unwrap_or(false);
            let mut measured = *attrs;
            let subtree = directories.as_ref().and_then(|values| values.get(&child)).copied();
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
                ignored: observed.then_some(ignored),
            };
            if matches {
                walked.rows.push(row.clone());
            }
            if matches || (covered && selection.ignored.admits(ignored)) {
                if kind == EntryKind::File {
                    walked.members.push(row);
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
            | ViewSpec::Documents
            | ViewSpec::Files
            | ViewSpec::Largest
            | ViewSpec::Recent
    )
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
fn build_section(
    view: ViewSpec,
    index: &Index,
    query: &Query,
    content: AnalysisSet,
    walked: Option<&Walked>,
    unfiltered_rows: Option<&[FileRow]>,
) -> Section {
    if query.tree_for(view) {
        return Section::Tree { view, root: tree_node(index, query, walked) };
    }
    match view {
        ViewSpec::Summary => Section::Summary(match walked {
            None => unfiltered_summary(index, EntryId::ROOT),
            Some(walked) => walked.summary_of(EntryId::ROOT),
        }),
        ViewSpec::Extensions => {
            let (rows, total) = extension_rows(index, query, walked);
            Section::Extensions { rows, total }
        }
        ViewSpec::Types | ViewSpec::Families | ViewSpec::Languages | ViewSpec::Documents => {
            Section::Metrics {
                view,
                summary: Box::new(metric_summary(
                    view,
                    index,
                    query,
                    content,
                    walked,
                    unfiltered_rows,
                )),
            }
        }
        ViewSpec::List
        | ViewSpec::Tree
        | ViewSpec::Files
        | ViewSpec::Largest
        | ViewSpec::Recent => {
            let (rows, total) = file_rows(view, index, query, walked, unfiltered_rows);
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
fn extension_rows(index: &Index, query: &Query, walked: Option<&Walked>) -> (Vec<TypeRow>, usize) {
    let observed = index.observes_controls();
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

    sort_rows(
        &mut rows,
        query,
        ViewSpec::Extensions,
        |row, metric| match metric {
            SizeMetric::Apparent => row.bytes,
            SizeMetric::Allocated => row.allocated,
        },
        |row| row.files,
        |_| None,
        |row| row.extension.clone(),
    );
    let total = truncate(&mut rows, query.limit_for(ViewSpec::Extensions));
    (rows, total)
}

fn metric_summary(
    view: ViewSpec,
    index: &Index,
    query: &Query,
    content: AnalysisSet,
    walked: Option<&Walked>,
    unfiltered_rows: Option<&[FileRow]>,
) -> MetricSummary {
    let group = if view == ViewSpec::Families { MetricGroup::Family } else { MetricGroup::Type };
    let files = walked.map_or_else(
        || entry_rows(index, None, unfiltered_rows),
        |walked| Cow::Borrowed(walked.members.as_slice()),
    );
    let mut grouped = BTreeMap::<String, MetricRow>::new();
    let wanted = index.content_identity(content);
    let held = index.content().and_then(|held| held.admit(&wanted));
    for file in files.iter().filter(|row| row.kind == EntryKind::File) {
        let cached = held.and_then(|content| content.file(&file.path));
        let classification = index.classify(&file.path);
        let included = match view {
            ViewSpec::Languages => classification.family == ContentFamily::Code,
            ViewSpec::Documents => {
                matches!(classification.family, ContentFamily::Prose | ContentFamily::Markup)
            }
            ViewSpec::Types | ViewSpec::Families => true,
            ViewSpec::List
            | ViewSpec::Tree
            | ViewSpec::Extensions
            | ViewSpec::Files
            | ViewSpec::Largest
            | ViewSpec::Recent
            | ViewSpec::Summary => false,
        };
        if !included {
            continue;
        }
        let id = match group {
            MetricGroup::Type => classification.file_type.as_str().to_string(),
            MetricGroup::Family => classification.family.as_str().to_string(),
        };
        let row = grouped.entry(id.clone()).or_insert_with(|| MetricRow {
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
            let selected = match view {
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
        total.document_raw_words = total.document_raw_words.saturating_add(row.document_raw_words);
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
        ViewSpec::Languages | ViewSpec::Documents if content.is_enabled() => ShareMetric::RawWords,
        ViewSpec::Languages | ViewSpec::Documents | ViewSpec::Types | ViewSpec::Families => {
            byte_share_metric
        }
        ViewSpec::List
        | ViewSpec::Tree
        | ViewSpec::Extensions
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
        |row, metric| match view {
            ViewSpec::Languages | ViewSpec::Documents => share_value(row, share_metric),
            _ => match metric {
                SizeMetric::Apparent => row.bytes,
                SizeMetric::Allocated => row.allocated,
            },
        },
        |row| row.files,
        |_| None,
        |row| row.id.clone(),
    );
    let total_rows = truncate(&mut rows, query.limit_for(view));
    MetricSummary {
        group,
        total,
        rows,
        total_rows,
        share_metric,
        words_per_page: query.words_per_page.max(1),
    }
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
    walked: Option<&Walked>,
    unfiltered_rows: Option<&[FileRow]>,
) -> (Vec<FileRow>, usize) {
    let mut rows = entry_rows(index, walked, unfiltered_rows).into_owned();
    if view.files_only() {
        rows.retain(|row| row.kind == EntryKind::File);
    }

    sort_rows(
        &mut rows,
        query,
        view,
        |row, metric| match metric {
            SizeMetric::Apparent => row.bytes,
            SizeMetric::Allocated => row.allocated,
        },
        |row| row.files.unwrap_or(1),
        |row| Some(row.mtime_ns),
        |row| row.path.to_string_lossy().into_owned(),
    );
    let total = truncate(&mut rows, query.limit_for(view));
    (rows, total)
}

/// Every entry in the index, for an unfiltered files view.
fn every_entry(index: &Index) -> Vec<FileRow> {
    let observed = index.observes_controls();
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
                ignored: observed.then(|| index.ignored_bit_of(child).unwrap_or(false)),
            });
            if kind == EntryKind::Dir {
                stack.push((child, child_path));
            }
        }
    }
    rows
}

/// The tree view's root node, expanded to the requested depth.
fn tree_node(index: &Index, query: &Query, walked: Option<&Walked>) -> TreeNode {
    let root_summary = match walked {
        None => unfiltered_summary(index, EntryId::ROOT),
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
        truncated: false,
    };
    expand(index, query, walked, EntryId::ROOT, &PathBuf::new(), &mut root, 0);
    root
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
    root_id: EntryId,
    root_path: &Path,
    node: &mut TreeNode,
    start_depth: usize,
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
            path: root_path.to_path_buf(),
            name: node.name.clone(),
            kind: node.kind,
            bytes: node.bytes,
            allocated: node.allocated,
            files: node.files,
            dirs: node.dirs,
            ignored: node.ignored,
            newest_mtime_ns: node.newest_mtime_ns,
            children: Vec::new(),
            truncated: false,
        },
        id: root_id,
        depth: start_depth,
        parent: None,
    }];

    let mut cursor = 0;
    while cursor < built.len() {
        let (id, depth) = (built[cursor].id, built[cursor].depth);
        let path = built[cursor].node.path.clone();

        if !query.selection.depth.unwrap_or(ViewSpec::Tree.default_depth()).admits(depth) {
            // `--depth 0` keeps du's meaning: totals for this node, nothing beneath it.
            // Files are already represented in this directory's totals and never become
            // tree rows. Only a directory child hidden by the depth bound makes the
            // rendered hierarchy incomplete.
            built[cursor].node.truncated = index.children_of(id).is_some_and(|mut children| {
                children.any(|(_, child)| {
                    index.kind_of(child) == Some(EntryKind::Dir)
                        && walked.is_none_or(|walked| walked.visible.contains(&child))
                })
            });
            cursor += 1;
            continue;
        }

        let mut rows = child_rows(index, query, walked, id, &path);
        let kept = query
            .selection
            .limit
            .unwrap_or(ViewSpec::Tree.default_limit())
            .limit()
            .unwrap_or(rows.len())
            .min(rows.len());
        built[cursor].node.truncated = kept < rows.len();
        rows.truncate(kept);

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
    node.truncated = root.node.truncated;
}

/// The directory children of one node, shaped and sorted but not yet expanded.
fn child_rows(
    index: &Index,
    query: &Query,
    walked: Option<&Walked>,
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
        // The tree view is a directory hierarchy: a file contributes its bytes to the
        // directory holding it rather than appearing as its own row.
        if kind != EntryKind::Dir {
            continue;
        }
        if walked.is_some_and(|walked| !walked.visible.contains(&child)) {
            continue;
        }
        let summary = match walked {
            None => unfiltered_summary(index, child),
            Some(walked) => walked.summary_of(child),
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
                truncated: false,
            },
            child,
        ));
    }

    sort_rows_by(
        &mut rows,
        query,
        ViewSpec::Tree,
        |(row, _), metric| match metric {
            SizeMetric::Apparent => row.bytes,
            SizeMetric::Allocated => row.allocated,
        },
        |(row, _)| row.files,
        |(row, _)| row.newest_mtime_ns,
        |(row, _)| row.name.clone(),
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
fn sort_rows<T>(
    rows: &mut [T],
    query: &Query,
    view: ViewSpec,
    size: impl Fn(&T, SizeMetric) -> u64,
    count: impl Fn(&T) -> u64,
    mtime: impl Fn(&T) -> Option<i64>,
    name: impl Fn(&T) -> String,
) {
    sort_rows_by(rows, query, view, size, count, mtime, name);
}

/// Sort rows by the effective key, with a stable name tiebreak.
fn sort_rows_by<T>(
    rows: &mut [T],
    query: &Query,
    view: ViewSpec,
    size: impl Fn(&T, SizeMetric) -> u64,
    count: impl Fn(&T) -> u64,
    mtime: impl Fn(&T) -> Option<i64>,
    name: impl Fn(&T) -> String,
) {
    let key = query.selection.sort.unwrap_or_else(|| view.default_sort());
    let metric = query.selection.size;

    rows.sort_by(|left, right| {
        let ordering = match key {
            // Size, count, and recency read most-first: the interesting end is the top.
            SortKey::Size => size(right, metric).cmp(&size(left, metric)),
            SortKey::Count => count(right).cmp(&count(left)),
            SortKey::Mtime => mtime(right).cmp(&mtime(left)),
            SortKey::Name => name(left).cmp(&name(right)),
        };
        // A name tiebreak keeps equal rows in a deterministic order, which is what makes
        // the goldens stable across runs and platforms.
        ordering.then_with(|| name(left).cmp(&name(right)))
    });

    if query.selection.reverse {
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
        let answer = report(&index, &request, &provenance()).expect("report");
        let rows = files_of(&answer);
        assert_eq!(answer.age_reference_ns, Some(0));
        assert_eq!(
            rows.iter().map(|r| r.age_ns).collect::<Vec<_>>(),
            [Some(-i128::from(i64::MAX)), Some(10)]
        );
        request.now = UNIX_EPOCH + Duration::from_secs(10_000_000_000);
        let answer = report(&index, &request, &provenance())
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

    /// A one-shot walk that finished with errors records that it was partial, not where,
    /// so no directory row can claim a complete subtree; a walk that finished records
    /// every directory as listed.
    #[test]
    fn a_partial_one_shot_index_marks_every_directory_row_incomplete() {
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
            ..Selection::default()
        };
        let root = tree_of(&run(&index, &query(&[ViewSpec::Tree], selection)));
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].name, "src");
        assert_eq!(root.children[0].children[0].name, "empty");
        let selection = Selection {
            include: vec![pattern("notes.txt")],
            depth: Some(Bound::Limit(0)),
            ..Selection::default()
        };
        let root = tree_of(&run(&index, &query(&[ViewSpec::Tree], selection)));
        assert!(!root.truncated);
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
            Section::Tree { root: node, .. } => node.clone(),
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
        assert_eq!(names, vec!["src", "docs"]);
        let src = &tree.children[0];
        assert_eq!(src.bytes, 350);
        let nested: Vec<&str> = src.children.iter().map(|child| child.name.as_str()).collect();
        assert_eq!(nested, vec!["deep"]);
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
        assert!(!omitted.is_empty(), "documents needs analysis and must be dropped");

        let query = Query { views: selected, omitted_views: omitted, ..Query::default() };
        let notes = display_notes(&query, &ControlCoverage::NotObserved);
        assert_eq!(notes.len(), 1, "{notes:?}");
        assert!(notes[0].contains("omitted documents"), "{notes:?}");

        // Nothing dropped, nothing said.
        let (selected, omitted) = ViewSpec::resolve(Some("full"), AnalysisSet::ALL, "view")
            .expect("full resolves with analyzers");
        assert!(omitted.is_empty(), "every view is answerable with analysis enabled");
        let query = Query { views: selected, omitted_views: omitted, ..Query::default() };
        assert!(display_notes(&query, &ControlCoverage::NotObserved).is_empty());
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

        let refused = |directory: &str, reason| RefusedControl {
            path: Path::new(directory).join(".gitignore"),
            reason,
        };
        let note = |limits, refusals: Vec<RefusedControl>, count: u64| {
            let coverage = ControlCoverage::Observed(ControlObservation {
                limits,
                applied: 7,
                refused: count,
                refusals,
            });
            refused_controls_note(&coverage, &AxisNames::FLAGS).expect("a refusal is noted")
        };
        let defaults = ControlLimits::default();
        let (budget, line_limit) = (ControlRefusalReason::Budget, ControlRefusalReason::LineLimit);

        assert_eq!(
            note(defaults, vec![refused("", budget), refused("pkg/a", budget)], 2),
            "note: 2 .gitignore files not applied (2 over the 4.0 MiB ignore-rule budget), so \
             ignored shares under ., pkg/a are not exact; sizes are. To apply them, raise \
             --gitignore-budget above 4.0 MiB, or set it to all"
        );
        assert_eq!(
            note(defaults, vec![refused("vendor", line_limit)], 1),
            "note: 1 .gitignore file not applied (1 with a line over the 16 KiB line limit), so \
             ignored shares under vendor are not exact; sizes are. To apply them, raise \
             --gitignore-line-limit above 16 KiB, or set it to all"
        );
        assert_eq!(
            note(defaults, vec![refused("a", budget), refused("b", line_limit)], 2),
            "note: 2 .gitignore files not applied (1 over the 4.0 MiB ignore-rule budget, 1 with \
             a line over the 16 KiB line limit), so ignored shares under a, b are not exact; \
             sizes are. To apply them, raise --gitignore-budget above 4.0 MiB and \
             --gitignore-line-limit above 16 KiB, or set them to all"
        );

        // Truncated, the note names every limit that could have refused an unlisted file:
        // both when both are bounded, and only the budget once the line limit is lifted.
        let listed: Vec<_> = (0..crate::MAX_RETAINED_ISSUES)
            .map(|index| refused(&format!("d{index:02}"), budget))
            .collect();
        assert_eq!(
            note(defaults, listed.clone(), 1_000),
            "note: 1,000 .gitignore files not applied (over the 4.0 MiB ignore-rule budget or \
             with a line over the 16 KiB line limit), so ignored shares under d00, d01, d02, \
             d03, d04, 995 more are not exact; sizes are. To apply them, raise \
             --gitignore-budget above 4.0 MiB and --gitignore-line-limit above 16 KiB, or set \
             them to all"
        );
        assert_eq!(
            note(ControlLimits { line_limit: None, ..defaults }, listed, 1_000),
            "note: 1,000 .gitignore files not applied (over the 4.0 MiB ignore-rule budget), so \
             ignored shares under d00, d01, d02, d03, d04, 995 more are not exact; sizes are. \
             To apply them, raise --gitignore-budget above 4.0 MiB, or set it to all"
        );

        // No engine path records a refusal by an unbounded limit, and the snapshot loader
        // rejects one, but a hand-built observation can still carry it: the note names the
        // limit without a size rather than a zero one, and raises only bounded limits.
        assert_eq!(
            note(ControlLimits { budget: None, ..defaults }, vec![refused("a", budget)], 1),
            "note: 1 .gitignore file not applied (1 over the ignore-rule budget), so ignored \
             shares under a are not exact; sizes are."
        );

        let complete = ControlCoverage::Observed(ControlObservation {
            limits: defaults,
            applied: 3,
            refused: 0,
            refusals: Vec::new(),
        });
        assert_eq!(refused_controls_note(&complete, &AxisNames::FLAGS), None);
        assert_eq!(refused_controls_note(&ControlCoverage::NotObserved, &AxisNames::FLAGS), None);
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
            let note = display_notes(&query, &ControlCoverage::NotObserved).remove(0);
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
    fn a_depth_bound_marks_only_hidden_directory_rows_as_truncated() {
        let index = sample();
        let selection = Selection { depth: Some(Bound::Limit(1)), ..Selection::default() };
        let tree = tree_of(&run(&index, &query(&[ViewSpec::Tree], selection)));

        let src = tree.children.iter().find(|child| child.name == "src").expect("src");
        assert!(src.truncated, "src with a hidden directory child is truncated");

        let docs = tree.children.iter().find(|child| child.name == "docs").expect("docs");
        assert!(docs.children.is_empty());
        assert!(
            !docs.truncated,
            "file children contribute to a directory row; they are not hidden tree rows"
        );
    }

    #[test]
    fn a_tree_limit_bounds_entries_per_directory_and_marks_truncation() {
        let index = sample();
        let selection = Selection { limit: Some(Bound::Limit(1)), ..Selection::default() };
        let tree = tree_of(&run(&index, &query(&[ViewSpec::Tree], selection)));
        assert_eq!(tree.children.len(), 1);
        assert!(tree.truncated);
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
        let Section::Tree { root: tree, .. } = &together.sections[8] else { panic!("tree") };
        assert_eq!((tree.files, tree.dirs), (3, 2));
        let Section::Extensions { rows, total } = &together.sections[9] else {
            panic!("extensions")
        };
        assert_eq!((*total, rows.len()), (3, 3));
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
        let Section::Tree { root: tree, .. } = &report.sections[1] else { panic!("a tree") };
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
