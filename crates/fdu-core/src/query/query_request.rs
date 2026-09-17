//! The request model: what determines an answer, the grammars its values are written in,
//! and the typed refusals that name a bad request in the caller's own vocabulary.
//!
//! A refusal is a value, not a sentence. Each surface renders it through its
//! [`AxisNames`], so the rule and its wording are stated once here while the command line
//! names flags and the library and the Python API name fields. The grammars used to live
//! in both front ends, each with its own copy of every spelling and every message, which
//! is how one request came to mean two things depending on the door it came through.

use std::fmt;
use std::time::SystemTime;

use crate::CachePolicy;
use crate::content::AnalysisSet;
use crate::engine_contract::EntryKind;
use crate::query::query_report::{AxisNames, ViewSpec};
use crate::query::query_selection::{Bound, IgnoredEntries, SizeMetric, SortKey};
use crate::query::query_values::system_time_to_nanos;

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
            Self::InvalidValue { axis, value, expected } => {
                format!("invalid {axis} {value:?}: {expected}")
            }
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

fn invalid(
    axis: &'static str,
    value: impl Into<String>,
    expected: impl Into<String>,
) -> RequestError {
    RequestError::InvalidValue { axis, value: value.into(), expected: expected.into() }
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
}
