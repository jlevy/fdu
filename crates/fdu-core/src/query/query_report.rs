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

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::classify::{ContentFamily, DetectionConfidence, DetectionSource};
use crate::content::{
    AnalysisSet, ContentIndex, ContentProvenance, CoverageReason, LogicalWordStats, MetricValues,
};
use crate::engine_contract::{EntryKind, Freshness, ScanScope};
use crate::index::{EntryId, ExtTally, Index, RollUpScalars};
use crate::query::query_selection::{Bound, Candidate, Selection, SizeMetric, SortKey};

/// Which roll-up or listing a view reports.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewSpec {
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
            Self::Tree
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
            Self::Files => Bound::All,
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
    pub const ALL: [Self; 10] = [
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
            "tree" => Ok(Self::Tree),
            "types" => Ok(Self::Types),
            "extensions" => Ok(Self::Extensions),
            "families" => Ok(Self::Families),
            "languages" => Ok(Self::Languages),
            "documents" | "docs" => Ok(Self::Documents),
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
            (false, false) => Self::Tree,
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
        let Some(spec) = spec else {
            return Ok((vec![Self::default_for(analysis)], Vec::new()));
        };

        let mut parsed: Vec<Self> = Vec::new();
        let mut full_seen = false;
        for raw in spec.split(',') {
            let token = raw.trim();
            if token.is_empty() {
                return Err(format!("invalid {label} {spec:?}: empty entry in the list"));
            }
            if token.eq_ignore_ascii_case("full") {
                if full_seen || !parsed.is_empty() {
                    return Err(format!("invalid {label} \"full\": {}", Self::FULL_IS_EXCLUSIVE));
                }
                full_seen = true;
                continue;
            }
            if full_seen {
                return Err(format!("invalid {label} \"full\": {}", Self::FULL_IS_EXCLUSIVE));
            }
            let view = Self::parse(token)
                .map_err(|expected| format!("invalid {label} {token:?}: {expected}"))?;
            if parsed.contains(&view) {
                return Err(format!("invalid {label} {spec:?}: {token:?} appears more than once"));
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
        !matches!(self, Self::Files)
    }
}

/// What the calling surface calls the two axes a report's diagnostics name.
///
/// The same reason `ViewSpec::resolve` and `AnalysisSet::parse_labeled` take a label: a
/// rule belongs to the library, but the words a caller can act on belong to the surface
/// they came through. Telling a Python caller to "add --analyze" names a flag that does
/// not exist in their surface -- the defect that made these messages worth moving here in
/// the first place, reappearing one door over (fdu-4apt).
///
/// Two axes rather than one because both diagnostics name both: the view that cannot be
/// answered, and the analyzer that would answer it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AxisNames {
    /// The view axis.
    pub view: &'static str,
    /// The analyzer axis.
    pub analyze: &'static str,
}

impl AxisNames {
    /// How the command line spells them.
    pub const FLAGS: Self = Self { view: "--view", analyze: "--analyze" };

    /// How the library and the Python API spell them, and the default: a `Query` built
    /// without saying otherwise belongs to a library caller, not to the command line.
    ///
    /// `view` singular, matching the label the binding already passes to
    /// `ViewSpec::resolve`, so every diagnostic about this axis names it one way. It also
    /// keeps the difference from the command line to the flag dashes alone, which is what
    /// the parity harness's `surface-label` class checks.
    pub const FIELDS: Self = Self { view: "view", analyze: "analyze" };
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
    pub axes: AxisNames,
    /// Fixed logical-word denominator used to derive page equivalents after aggregation.
    pub words_per_page: u64,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            selection: Selection::default(),
            views: Vec::new(),
            omitted_views: Vec::new(),
            axes: AxisNames::default(),
            words_per_page: 250,
        }
    }
}

impl Query {
    /// The bound to apply for `view`: the caller's if they named one, else the view's own.
    pub fn limit_for(&self, view: ViewSpec) -> Bound {
        self.selection.limit.unwrap_or_else(|| view.default_limit())
    }

    /// The tree depth to apply for `view`, on the same terms as `limit_for`.
    pub fn depth_for(&self, view: ViewSpec) -> Bound {
        self.selection.depth.unwrap_or_else(|| view.default_depth())
    }

    /// Reject views that have no metadata-only projection and lack required analysis.
    ///
    /// Names both axes as the requesting surface spells them, so the advice is something
    /// the caller can actually act on: a Python caller has no `--analyze` to add.
    pub fn validate_analysis(&self, profile: AnalysisSet) -> Result<(), String> {
        for view in &self.views {
            match view {
                ViewSpec::Documents if !profile.is_enabled() => {
                    return Err(format!(
                        "{} documents requires content analysis: add {} lines, code, words, or all; views never enable content analysis implicitly",
                        self.axes.view, self.axes.analyze
                    ));
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

/// Facts about how a report's index was produced.
///
/// Passed in rather than sampled inside [`report`] so the view layer stays a pure
/// function of its inputs: the same index and query must always produce the same report,
/// which is what makes the goldens meaningful and the tests deterministic.
#[derive(Clone, Debug)]
pub struct Provenance {
    /// When the walk or revalidation behind this index began.
    ///
    /// This, not the finish time, is the sound watermark for an incremental follow-up
    /// query: a file modified mid-scan may have been observed before the modification,
    /// so only the start bound is conservative.
    pub scan_started_at: Option<SystemTime>,
    /// When the report was rendered.
    pub generated_at: SystemTime,
    /// Which cache tier answered.
    pub source: ReportSource,
    /// Whether every path in scope was read successfully.
    pub complete: bool,
    /// Per-path failures that made this result partial, already rendered.
    ///
    /// Rendered strings rather than error values: this crosses into serialization, and a
    /// report is evidence about a run, not a place to re-handle its errors.
    pub errors: Vec<String>,
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
}

impl ShareMetric {
    /// Stable machine label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ApparentBytes => "apparent_bytes",
            Self::AllocatedBytes => "allocated_bytes",
            Self::CodeLines => "code_lines",
            Self::DocumentWords => "document_words",
        }
    }
}

/// One stable group in a metric-summary section.
#[derive(Clone, Debug)]
pub struct MetricRow {
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
    pub metrics: MetricValues,
    /// Query-selected raw document words before normalization.
    pub document_raw_words: u64,
    /// Additive sufficient statistics for the query-selected document projection.
    pub document_word_stats: LogicalWordStats,
    /// Analyzed files that supplied normalized document statistics.
    pub document_metric_files: u64,
    /// Explicit content-analysis outcomes.
    pub coverage: BTreeMap<CoverageReason, u64>,
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

/// One entry's row in a files view.
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
    /// Newest modification time, when anything was selected.
    pub newest_mtime_ns: Option<i64>,
}

/// One view's results.
#[derive(Clone, Debug)]
pub enum Section {
    /// A tree view.
    Tree(TreeNode),
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
            Self::Tree(_) => ViewSpec::Tree,
            Self::Extensions { .. } => ViewSpec::Extensions,
            Self::Metrics { view, .. } | Self::Files { view, .. } => *view,
            Self::Summary(_) => ViewSpec::Summary,
        }
    }
}

/// A rendered answer: provenance, plus one section per requested view.
#[derive(Clone, Debug)]
pub struct Report {
    /// When the walk behind this index began.
    pub scan_started_at: Option<SystemTime>,
    /// When this report was rendered.
    pub generated_at: SystemTime,
    /// Which cache tier answered.
    pub source: ReportSource,
    /// Whether every path in scope was read successfully.
    pub complete: bool,
    /// Per-path failures that made this result partial.
    pub errors: Vec<String>,
    /// How current the index is.
    pub freshness: Freshness,
    /// The semantic scan scope represented by this report.
    ///
    /// A report-only cache projection may consume stronger internal control state that
    /// no report view exposes; this field still names the weaker requested scope.
    pub scope: ScanScope,
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
    /// One section per requested view, in request order.
    pub sections: Vec<Section>,
}

/// Remarks a report makes about itself.
///
/// Only what the request and the resolved views can establish. The CLI also prints a note
/// quoting how many bytes analysis read, which is walk telemetry the report envelope does
/// not carry, so that one stays with the performance footer where the rest of the run's
/// telemetry lives.
fn display_notes(query: &Query) -> Vec<String> {
    if query.omitted_views.is_empty() {
        return Vec::new();
    }
    let names: Vec<&str> = query.omitted_views.iter().map(|view| view.label()).collect();
    vec![format!(
        "note: omitted {} — requires content analysis: add {} lines, code, words, or all",
        names.join(", "),
        query.axes.analyze
    )]
}

/// Build a report from an index.
///
/// Pure: the same index, query, and provenance always produce the same report, and
/// nothing here reads the filesystem or mutates the index.
pub fn report(index: &Index, query: &Query, provenance: &Provenance) -> Report {
    // One traversal serves every filtered view in the request, so asking for three views
    // costs one pass rather than three.
    let walked = (!query.selection.is_unfiltered()).then(|| walk(index, &query.selection));

    let sections = query
        .views
        .iter()
        .map(|view| build_section(*view, index, query, walked.as_ref()))
        .collect();

    Report {
        notes: display_notes(query),
        scan_started_at: provenance.scan_started_at,
        generated_at: provenance.generated_at,
        source: provenance.source,
        complete: provenance.complete,
        errors: provenance.errors.clone(),
        freshness: index.freshness(),
        scope: index.scope(),
        root: index.root_path().to_path_buf(),
        size: query.selection.size,
        analysis: index.content().and_then(|content| {
            Some(ContentReportMetadata {
                profile: content.profile()?,
                provenance: content.provenance()?.clone(),
            })
        }),
        sections,
    }
}

/// Build a one-section report from an already reduced exact summary.
///
/// Pure for the same reason as [`report`]: scanning and time sampling happened before
/// this boundary.  The execution planner uses this when a one-shot request proves that
/// retaining paths and hierarchy cannot affect its answer.
pub(crate) fn report_summary(
    root: &Path,
    scope: ScanScope,
    size: SizeMetric,
    summary: SummaryRow,
    freshness: Freshness,
    provenance: &Provenance,
) -> Report {
    Report {
        // A compact summary resolves one view and drops none.
        notes: Vec::new(),
        scan_started_at: provenance.scan_started_at,
        generated_at: provenance.generated_at,
        source: provenance.source,
        complete: provenance.complete,
        errors: provenance.errors.clone(),
        freshness,
        scope,
        root: root.to_path_buf(),
        size,
        // The planner only selects this tier when no analysis was requested, so there is
        // no analyzer provenance to report.
        analysis: None,
        sections: vec![Section::Summary(summary)],
    }
}

/// Aggregates gathered by one filtered traversal.
struct Walked {
    /// Filtered subtree aggregates, keyed by directory id.
    per_directory: BTreeMap<EntryId, SummaryRow>,
    /// Filtered per-extension tallies.
    by_ext: BTreeMap<String, ExtTally>,
    /// Entries the selection admitted.
    rows: Vec<FileRow>,
}

/// Walk the retained index once, aggregating only what the selection admits.
///
/// Iterative rather than recursive: this engine is built for trees deep enough that a
/// recursive post-order would exhaust the stack.
fn walk(index: &Index, selection: &Selection) -> Walked {
    let mut walked =
        Walked { per_directory: BTreeMap::new(), by_ext: BTreeMap::new(), rows: Vec::new() };

    // (id, path, whether its children have already been pushed)
    let mut stack: Vec<(EntryId, PathBuf, bool)> = vec![(EntryId::ROOT, PathBuf::new(), false)];
    while let Some((id, path, expanded)) = stack.pop() {
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
            walked.per_directory.insert(id, total);
            continue;
        }

        stack.push((id, path.clone(), true));
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
            let name = file_name.to_string_lossy().into_owned();
            let candidate = Candidate {
                relative: &child_path,
                name: &name,
                kind,
                bytes: attrs.size,
                allocated: attrs.allocated,
                mtime_ns: attrs.mtime_ns,
            };

            if selection.admits(&candidate) {
                walked.rows.push(FileRow {
                    path: child_path.clone(),
                    kind,
                    bytes: attrs.size,
                    allocated: attrs.allocated,
                    mtime_ns: attrs.mtime_ns,
                });

                if kind == EntryKind::File {
                    let own = walked.per_directory.entry(id).or_default();
                    own.files += 1;
                    own.bytes += attrs.size;
                    own.allocated += attrs.allocated;
                    own.newest_mtime_ns = Some(
                        own.newest_mtime_ns.map_or(attrs.mtime_ns, |seen| seen.max(attrs.mtime_ns)),
                    );

                    let tally =
                        walked.by_ext.entry(index.types().ext_bucket(file_name)).or_default();
                    tally.files += 1;
                    tally.bytes += attrs.size;
                    tally.allocated += attrs.allocated;
                } else if kind == EntryKind::Dir {
                    // Tallied here, beside the file case, rather than in the post-order
                    // fold: the fold sees every directory the walk descended into, and
                    // counting there reported directories the selection had rejected.
                    // `--kind file` answered "6 files, 3 directories", and a summary
                    // disagreed with the files view over the very same query.
                    walked.per_directory.entry(id).or_default().dirs += 1;
                }
            }

            if kind == EntryKind::Dir {
                stack.push((child, child_path, false));
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
}

/// Build one view's section, using the pre-computed tier when the selection allows.
fn build_section(view: ViewSpec, index: &Index, query: &Query, walked: Option<&Walked>) -> Section {
    match view {
        ViewSpec::Summary => Section::Summary(match walked {
            None => summary_from_scalars(index.total_scalars()),
            Some(walked) => walked.per_directory.get(&EntryId::ROOT).copied().unwrap_or_default(),
        }),
        ViewSpec::Extensions => {
            let (rows, total) = extension_rows(index, query, walked);
            Section::Extensions { rows, total }
        }
        ViewSpec::Types | ViewSpec::Families | ViewSpec::Languages | ViewSpec::Documents => {
            Section::Metrics { view, summary: Box::new(metric_summary(view, index, query, walked)) }
        }
        ViewSpec::Files | ViewSpec::Largest | ViewSpec::Recent => {
            let (rows, total) = file_rows(view, index, query, walked);
            Section::Files { view, rows, total }
        }
        ViewSpec::Tree => Section::Tree(tree_node(index, query, walked)),
    }
}

/// A summary row taken straight from pre-computed roll-up state.
fn summary_from_scalars(rollup: RollUpScalars) -> SummaryRow {
    SummaryRow {
        files: rollup.files,
        dirs: rollup.dirs,
        bytes: rollup.bytes,
        allocated: rollup.allocated,
        newest_mtime_ns: (rollup.files > 0).then_some(rollup.newest_mtime_ns),
    }
}

/// Rows for the types view.
fn extension_rows(index: &Index, query: &Query, walked: Option<&Walked>) -> (Vec<TypeRow>, usize) {
    let tallies: BTreeMap<String, ExtTally> = match walked {
        None => index.total().by_ext,
        Some(walked) => walked.by_ext.clone(),
    };

    let mut rows: Vec<TypeRow> = tallies
        .into_iter()
        .map(|(extension, tally)| TypeRow {
            extension,
            files: tally.files,
            bytes: tally.bytes,
            allocated: tally.allocated,
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
    walked: Option<&Walked>,
) -> MetricSummary {
    let group = if view == ViewSpec::Families { MetricGroup::Family } else { MetricGroup::Type };
    let files = walked.map_or_else(|| every_entry(index), |walked| walked.rows.clone());
    let mut grouped = BTreeMap::<String, MetricRow>::new();
    for file in files.into_iter().filter(|row| row.kind == EntryKind::File) {
        let cached = index.content().and_then(|content| content.file(&file.path));
        let classification = cached
            .map_or_else(|| index.classify(&file.path), |record| record.classification.clone());
        let included = match view {
            ViewSpec::Languages => classification.family == ContentFamily::Code,
            ViewSpec::Documents => {
                matches!(classification.family, ContentFamily::Prose | ContentFamily::Markup)
            }
            ViewSpec::Types | ViewSpec::Families => true,
            ViewSpec::Tree
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
            id,
            family: classification.family,
            files: 0,
            bytes: 0,
            allocated: 0,
            analyzed_files: 0,
            metrics: MetricValues::default(),
            document_raw_words: 0,
            document_word_stats: LogicalWordStats::default(),
            document_metric_files: 0,
            coverage: BTreeMap::new(),
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
        *row.detection_sources.entry(classification.source).or_default() += 1;
        *row.detection_confidence.entry(classification.confidence).or_default() += 1;
        row.generated_files =
            row.generated_files.saturating_add(u64::from(classification.flags.generated));
        row.vendored_files =
            row.vendored_files.saturating_add(u64::from(classification.flags.vendored));
        row.documentation_files =
            row.documentation_files.saturating_add(u64::from(classification.flags.documentation));
        if let Some(record) = cached {
            *row.coverage.entry(record.coverage).or_default() += 1;
            if record.coverage == CoverageReason::Analyzed {
                row.analyzed_files = row.analyzed_files.saturating_add(1);
                row.metrics.add_assign(&record.metrics);
                if record.profile.includes_words() {
                    row.document_metric_files = row.document_metric_files.saturating_add(1);
                    if classification.file_type.as_str() == "markdown" {
                        row.document_raw_words =
                            row.document_raw_words.saturating_add(record.metrics.visible_words);
                        row.document_word_stats
                            .add_assign(record.metrics.visible_logical_word_stats);
                    } else {
                        row.document_raw_words =
                            row.document_raw_words.saturating_add(record.metrics.raw_words);
                        row.document_word_stats.add_assign(record.metrics.logical_word_stats);
                    }
                } else {
                    row.document_raw_words =
                        row.document_raw_words.saturating_add(record.metrics.raw_words);
                }
            }
        }
    }

    let mut total = MetricRow {
        id: "total".to_string(),
        family: ContentFamily::Unknown,
        files: 0,
        bytes: 0,
        allocated: 0,
        analyzed_files: 0,
        metrics: MetricValues::default(),
        document_raw_words: 0,
        document_word_stats: LogicalWordStats::default(),
        document_metric_files: 0,
        coverage: BTreeMap::new(),
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
        total.document_raw_words = total.document_raw_words.saturating_add(row.document_raw_words);
        total.document_word_stats.add_assign(row.document_word_stats);
        total.document_metric_files =
            total.document_metric_files.saturating_add(row.document_metric_files);
        for (reason, count) in &row.coverage {
            *total.coverage.entry(*reason).or_default() += count;
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
    let byte_share_metric = match query.selection.size {
        SizeMetric::Apparent => ShareMetric::ApparentBytes,
        SizeMetric::Allocated => ShareMetric::AllocatedBytes,
    };
    let share_metric = match view {
        ViewSpec::Languages
            if index
                .content()
                .and_then(ContentIndex::profile)
                .is_some_and(AnalysisSet::includes_code) =>
        {
            ShareMetric::CodeLines
        }
        ViewSpec::Documents => ShareMetric::DocumentWords,
        ViewSpec::Languages | ViewSpec::Types | ViewSpec::Families => byte_share_metric,
        ViewSpec::Tree
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

fn share_value(row: &MetricRow, metric: ShareMetric) -> u64 {
    match metric {
        ShareMetric::ApparentBytes => row.bytes,
        ShareMetric::AllocatedBytes => row.allocated,
        ShareMetric::CodeLines => row.metrics.code_lines,
        ShareMetric::DocumentWords => document_words(row),
    }
}

/// Derive the selected document volume only after every sufficient statistic is added.
pub fn document_words(row: &MetricRow) -> u64 {
    if row.document_metric_files > 0 {
        row.document_word_stats.logical_words()
    } else {
        row.document_raw_words
    }
}

/// Rows for the files view.
fn file_rows(
    view: ViewSpec,
    index: &Index,
    query: &Query,
    walked: Option<&Walked>,
) -> (Vec<FileRow>, usize) {
    let mut rows = match walked {
        Some(walked) => walked.rows.clone(),
        None => every_entry(index),
    };
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
        |_| 1,
        |row| Some(row.mtime_ns),
        |row| row.path.to_string_lossy().into_owned(),
    );
    let total = truncate(&mut rows, query.limit_for(view));
    (rows, total)
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
        None => summary_from_scalars(index.total_scalars()),
        Some(walked) => walked.per_directory.get(&EntryId::ROOT).copied().unwrap_or_default(),
    };

    let mut root = TreeNode {
        path: PathBuf::new(),
        name: ".".to_string(),
        kind: EntryKind::Dir,
        bytes: root_summary.bytes,
        allocated: root_summary.allocated,
        files: root_summary.files,
        dirs: root_summary.dirs,
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

        if !query.depth_for(ViewSpec::Tree).admits(depth) {
            // `--depth 0` keeps du's meaning: totals for this node, nothing beneath it.
            // Files are already represented in this directory's totals and never become
            // tree rows. Only a directory child hidden by the depth bound makes the
            // rendered hierarchy incomplete.
            built[cursor].node.truncated = index.children_of(id).is_some_and(|mut children| {
                children.any(|(_, child)| index.kind_of(child) == Some(EntryKind::Dir))
            });
            cursor += 1;
            continue;
        }

        let mut rows = child_rows(index, query, walked, id, &path);
        let kept = query.limit_for(ViewSpec::Tree).limit().unwrap_or(rows.len()).min(rows.len());
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
        let summary = match walked {
            None => index.rollup_scalars_of(child).map(summary_from_scalars).unwrap_or_default(),
            Some(walked) => walked.per_directory.get(&child).copied().unwrap_or_default(),
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

    fn provenance() -> Provenance {
        Provenance {
            scan_started_at: Some(UNIX_EPOCH + Duration::from_secs(1_000)),
            generated_at: UNIX_EPOCH + Duration::from_secs(1_001),
            source: ReportSource::ColdScan,
            complete: true,
            errors: Vec::new(),
        }
    }

    fn run(index: &Index, request: &Query) -> Report {
        report(index, request, &provenance())
    }

    fn query(views: &[ViewSpec], selection: Selection) -> Query {
        Query { selection, views: views.to_vec(), ..Query::default() }
    }

    #[test]
    fn language_grouping_is_metadata_only_while_documents_require_analysis() {
        let languages = query(&[ViewSpec::Languages], Selection::default());
        for profile in [
            AnalysisSet::NONE,
            AnalysisSet::NONE.with_lines(),
            AnalysisSet::NONE.with_code(),
            AnalysisSet::NONE.with_words(),
            AnalysisSet::ALL,
        ] {
            languages
                .validate_analysis(profile)
                .expect("language grouping never requires content I/O");
        }

        let documents = query(&[ViewSpec::Documents], Selection::default());
        assert!(documents.validate_analysis(AnalysisSet::NONE).is_err());
        for profile in [
            AnalysisSet::NONE.with_lines(),
            AnalysisSet::NONE.with_code(),
            AnalysisSet::NONE.with_words(),
            AnalysisSet::ALL,
        ] {
            documents
                .validate_analysis(profile)
                .expect("every enabled profile includes the basic document metrics");
        }

        query(&[ViewSpec::Types, ViewSpec::Families], Selection::default())
            .validate_analysis(AnalysisSet::NONE)
            .expect("metadata grouping never requires content I/O");
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
            Section::Tree(node) => node.clone(),
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
    fn a_summary_and_a_files_view_agree_on_how_many_directories_a_filter_admits() {
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
            let files = listed.iter().filter(|row| row.kind == EntryKind::File).count() as u64;
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
        assert_eq!(root.files, 0, "no file was admitted");
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
        let selection = Selection {
            kinds: vec![EntryKind::File],
            sort: Some(SortKey::Size),
            limit: Some(Bound::Limit(2)),
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
        let notes = display_notes(&query);
        assert_eq!(notes.len(), 1, "{notes:?}");
        assert!(notes[0].contains("omitted documents"), "{notes:?}");

        // Nothing dropped, nothing said.
        let (selected, omitted) = ViewSpec::resolve(Some("full"), AnalysisSet::ALL, "view")
            .expect("full resolves with analyzers");
        assert!(omitted.is_empty(), "every view is answerable with analysis enabled");
        let query = Query { views: selected, omitted_views: omitted, ..Query::default() };
        assert!(display_notes(&query).is_empty());
    }

    /// A rule belongs to the library; the words a caller can act on belong to their
    /// surface. Both diagnostics name two axes, and naming them with flags told a Python
    /// caller to add an `--analyze` their surface does not have (fdu-4apt).
    ///
    /// Asserted as "names mine, never the other's" rather than by quoting either sentence,
    /// so rewording the rule cannot break this and changing the vocabulary cannot pass it.
    #[test]
    fn a_diagnostic_names_the_axes_the_requesting_surface_uses() {
        let (selected, omitted) = ViewSpec::resolve(Some("full"), AnalysisSet::NONE, "view")
            .expect("full resolves without analyzers");

        for (axes, mine, theirs) in [
            (AxisNames::FLAGS, "--analyze", "analyze"),
            (AxisNames::FIELDS, "analyze", "--analyze"),
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
            let note = display_notes(&query).remove(0);
            assert!(note.contains(&format!("add {mine} ")), "{note} must name {mine}");
            assert!(!note.contains(&format!("add {theirs} ")), "{note} must not name {theirs}");
        }

        // The same for the hard error, which names the view axis as well.
        for (axes, view, analyze) in
            [(AxisNames::FLAGS, "--view", "--analyze"), (AxisNames::FIELDS, "view", "analyze")]
        {
            let query = Query { views: vec![ViewSpec::Documents], axes, ..Query::default() };
            let error = query
                .validate_analysis(AnalysisSet::NONE)
                .expect_err("documents cannot be answered without analysis");
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
        assert_eq!(Query::default().axes, AxisNames::FIELDS);
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
    fn a_report_carries_the_provenance_it_was_given() {
        let index = sample();
        let report = run(&index, &query(&[ViewSpec::Summary], Selection::default()));
        assert_eq!(report.source, ReportSource::ColdScan);
        assert!(report.complete);
        assert_eq!(report.scan_started_at, Some(UNIX_EPOCH + Duration::from_secs(1_000)));
        assert_eq!(report.generated_at, UNIX_EPOCH + Duration::from_secs(1_001));
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
        let report = run(
            &index,
            &query(
                &[ViewSpec::Types, ViewSpec::Families, ViewSpec::Languages],
                Selection::default(),
            ),
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
}
