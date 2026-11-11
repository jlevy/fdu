//! The age profile: the selected regular files, partitioned by their age in the
//! report's time.
//!
//! The time is the report's clock ([`TimeBasis`]): change time by default, because the
//! question this view answers is when space arrived, and modification time misses every
//! arrival that kept an older one: a copy, an extracted archive, Cargo's crate sources.
//!
//! The bands are fixed, so every report is comparable with every other, and each is one
//! window query: the files `--kind file --since <since_age> --before <before_age>`
//! selects over the same selection, clock, and instant. `--kind file` is part of the
//! identity: without it a window admits a whole directory with activity in it, and a band
//! only ever holds files. Windows are half-open, `[since, before)`, so an entry exactly on
//! an edge falls in the newer band, and one stamped at the request instant itself is
//! `future`. The bands therefore partition the selection: their files and bytes sum to
//! what `summary` reports for it.

use std::path::{Path, PathBuf};

use crate::index::{EntryId, Index};
use crate::query::{FileRow, IgnoredTally};
use crate::{EntryKind, TimeBasis, Times};

const HOUR: i64 = 3_600_000_000_000;
const DAY: i64 = 24 * HOUR;
/// A year of 365.25 days, as the tree's age column counts one.
const YEAR: i64 = 365 * DAY + 6 * HOUR;

/// Each dated band's label and the oldest age it holds, inclusive. A band holds the ages
/// above the previous band's edge; `future` holds every age at or below zero, and the band
/// after the last edge every age above it.
const DATED: [(&str, i64); 8] = [
    ("future", 0),
    ("under 1h", HOUR),
    ("1h to 1d", DAY),
    ("1d to 7d", 7 * DAY),
    ("7d to 30d", 30 * DAY),
    ("30d to 90d", 90 * DAY),
    ("90d to 1y", YEAR),
    ("1y to 3y", 3 * YEAR),
];
/// Every age past the last dated edge.
const OLDEST: &str = "over 3y";
/// How many bands a profile has: the dated ones and the oldest.
const BANDS: usize = DATED.len() + 1;

/// The files, bytes, and ignored part one band, or several, holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AgeTally {
    /// Regular files.
    pub files: u64,
    /// Apparent bytes.
    pub bytes: u64,
    /// Allocated bytes.
    pub allocated: u64,
    /// The ignored part, or `None` when rules were unobserved or a counted file's
    /// classification is unknown.
    pub ignored: Option<IgnoredTally>,
}

impl AgeTally {
    fn empty(observed: bool) -> Self {
        Self { ignored: observed.then(IgnoredTally::default), ..Self::default() }
    }

    fn count(&mut self, file: &AgedFile) {
        self.files += 1;
        self.bytes += file.bytes;
        self.allocated += file.allocated;
        match file.ignored {
            Some(true) => {
                if let Some(share) = self.ignored.as_mut() {
                    share.add(IgnoredTally {
                        files: 1,
                        dirs: 0,
                        bytes: file.bytes,
                        allocated: file.allocated,
                    });
                }
            }
            Some(false) => {}
            None => self.ignored = None,
        }
    }

    fn add(&mut self, other: &Self) {
        self.files += other.files;
        self.bytes += other.bytes;
        self.allocated += other.allocated;
        self.ignored = self.ignored.zip(other.ignored).map(|(mut sum, share)| {
            sum.add(share);
            sum
        });
    }
}

/// One band of an age profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AgeBand {
    /// The band's label: `future`, `under 1h`, … `over 3y`.
    pub band: &'static str,
    /// The oldest age the band holds, inclusive, in nanoseconds: its window's `--since`.
    /// `None` for `over 3y`, which has no oldest age.
    pub since_age_ns: Option<i64>,
    /// The newest age the band excludes, in nanoseconds: its window's `--before`. `None`
    /// for `future`, which has no newer band.
    pub before_age_ns: Option<i64>,
    /// [`Self::since_age_ns`] as an instant, in epoch nanoseconds, at the request instant.
    pub since_ns: Option<i64>,
    /// [`Self::before_age_ns`] as an instant.
    pub before_ns: Option<i64>,
    /// What the band holds.
    pub tally: AgeTally,
    /// What this band and every newer one hold, which answers "how much changed within
    /// this age?".
    pub cumulative: AgeTally,
}

/// The selected regular files by age band, on the report's clock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgeProfile {
    /// Every band, newest first, ending with `over 3y`; empty bands included.
    pub bands: Vec<AgeBand>,
    /// Every band together: the files and bytes `summary` reports for the same selection.
    pub total: AgeTally,
    /// Files whose status change was not observed, and which the change-time clock
    /// therefore timed by modification ([`crate::Attrs::times`]); zero on the
    /// modification-time clock, which needs no status change.
    pub timed_by_modification: u64,
}

/// The band a time falls in at `now`, by index into a profile's bands.
fn band_of(time: i64, now: i64) -> usize {
    let age = i128::from(now) - i128::from(time);
    DATED.iter().position(|(_, oldest)| age <= i128::from(*oldest)).unwrap_or(DATED.len())
}

/// One regular file as an age profile counts it. No path: banding never reads one, so a
/// profile of a whole index need not build one per entry (review F5 on #209).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AgedFile {
    /// Apparent bytes.
    pub(crate) bytes: u64,
    /// Allocated bytes.
    pub(crate) allocated: u64,
    /// Its time on both clocks.
    pub(crate) times: Times,
    /// Whether its status change was observed.
    pub(crate) status_observed: bool,
    /// Whether `.gitignore` rules ignore it; `None` where its classification is unknown.
    pub(crate) ignored: Option<bool>,
}

impl AgedFile {
    /// The file a row describes, or `None` for a row that is not a regular file.
    pub(crate) fn of_row(row: &FileRow) -> Option<Self> {
        (row.kind == EntryKind::File).then_some(Self {
            bytes: row.bytes,
            allocated: row.allocated,
            times: row.times,
            status_observed: row.status_observed,
            ignored: row.ignored,
        })
    }
}

/// Every regular file `index` holds, as an age profile counts it, handed to `each` in no
/// particular order: what `ages` bands when the selection filters nothing.
///
/// Read by entry id, as [`super::query_subtrees`] reads activity: no file's path is built.
/// A file's ignored classification is known when the rules governing its directory were
/// read ([`Index::ignored_classification_of`]); where every rule was, no path is built at
/// all, and otherwise only a directory's, to ask that of it.
pub(crate) fn each_file(index: &Index, mut each: impl FnMut(AgedFile)) {
    debug_assert!(!index.is_folded(), "a folded index keeps too few files to band");
    let observes = index.observes_controls();
    let all_known = observes && index.ignored_classification_complete_below(Path::new(""));
    // Each directory still to read, with its path where classification is decided by
    // path, and whether the classification of the entries directly in it is known.
    let known_in = |path: &Option<PathBuf>| match path {
        _ if !observes => false,
        None => all_known,
        Some(path) => index.children_classification_known(path),
    };
    let root_path = (observes && !all_known).then(PathBuf::new);
    let root_known = known_in(&root_path);
    let mut stack: Vec<(EntryId, Option<PathBuf>, bool)> =
        vec![(EntryId::ROOT, root_path, root_known)];
    while let Some((id, path, known)) = stack.pop() {
        let Some(children) = index.children_of(id) else { continue };
        for (name, child) in children {
            let (Some(kind), Some(attrs)) = (index.kind_of(child), index.attrs_of(child)) else {
                continue;
            };
            match kind {
                EntryKind::File => each(AgedFile {
                    bytes: attrs.size,
                    allocated: attrs.allocated,
                    times: attrs.times(),
                    status_observed: attrs.change_time_ns().is_some(),
                    ignored: known.then(|| index.entry_ignored(child)).flatten(),
                }),
                EntryKind::Dir => {
                    let child_path = path.as_ref().map(|path| path.join(name));
                    let child_known = known_in(&child_path);
                    stack.push((child, child_path, child_known));
                }
                _ => {}
            }
        }
    }
}

/// An age profile counted one file at a time: the selected regular files, at `now_ns`,
/// the request instant, on clock `time`.
///
/// `observed` says whether every root's index observed `.gitignore` rules, without which
/// no band has an ignored part. With no representable instant no age can be measured, so
/// no file is banded and only the total counts them;
/// [`crate::query::Request::validate`] refuses `ages` for such a request.
pub(crate) struct AgeCounter {
    tallies: [AgeTally; BANDS],
    unbanded: AgeTally,
    timed_by_modification: u64,
    now_ns: Option<i64>,
    time: TimeBasis,
    observed: bool,
}

impl AgeCounter {
    /// A profile with nothing counted yet.
    pub(crate) fn new(now_ns: Option<i64>, time: TimeBasis, observed: bool) -> Self {
        Self {
            tallies: [AgeTally::empty(observed); BANDS],
            unbanded: AgeTally::empty(observed),
            timed_by_modification: 0,
            now_ns,
            time,
            observed,
        }
    }

    /// Count one regular file.
    pub(crate) fn count(&mut self, file: AgedFile) {
        if self.time == TimeBasis::Changed && !file.status_observed {
            self.timed_by_modification += 1;
        }
        match self.now_ns {
            Some(now) => self.tallies[band_of(file.times.get(self.time), now)].count(&file),
            None => self.unbanded.count(&file),
        }
    }

    /// The profile of every file counted.
    pub(crate) fn finish(self) -> AgeProfile {
        let Self { tallies, unbanded, timed_by_modification, now_ns, observed, .. } = self;
        let instant =
            |age: Option<i64>| now_ns.zip(age).and_then(|(now, age)| now.checked_sub(age));
        let mut running = AgeTally::empty(observed);
        let mut total = unbanded;
        let bands = tallies
            .iter()
            .enumerate()
            .map(|(index, tally)| {
                total.add(tally);
                let (band, since_age_ns, before_age_ns) = match index {
                    index if index < DATED.len() => (
                        DATED[index].0,
                        Some(DATED[index].1),
                        index.checked_sub(1).map(|newer| DATED[newer].1),
                    ),
                    _ => (OLDEST, None, Some(DATED[DATED.len() - 1].1)),
                };
                running.add(tally);
                let cumulative = running;
                AgeBand {
                    band,
                    since_age_ns,
                    before_age_ns,
                    since_ns: instant(since_age_ns),
                    before_ns: instant(before_age_ns),
                    tally: *tally,
                    cumulative,
                }
            })
            .collect();
        AgeProfile { bands, total, timed_by_modification }
    }
}

/// The age profile of `rows`, the entries a selection admits or covers, at `now_ns`: their
/// regular files, as [`AgeCounter`] counts them.
#[cfg(test)]
fn age_profile<'a>(
    rows: impl IntoIterator<Item = &'a FileRow>,
    now_ns: Option<i64>,
    time: TimeBasis,
    observed: bool,
) -> AgeProfile {
    let mut counter = AgeCounter::new(now_ns, time, observed);
    rows.into_iter().filter_map(AgedFile::of_row).for_each(|file| counter.count(file));
    counter.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const NOW: i64 = 2_000_000_000_000_000_000;

    fn file(changed: i64, bytes: u64, ignored: Option<bool>) -> FileRow {
        FileRow {
            root: 0,
            path: PathBuf::from("f"),
            kind: EntryKind::File,
            bytes,
            allocated: bytes,
            times: crate::Times { modified: 0, changed },
            status_observed: true,
            files: None,
            dirs: None,
            complete: None,
            age_ns: None,
            ignored,
            sort_value: None,
            classification: None,
        }
    }

    fn labels(profile: &AgeProfile) -> Vec<(&'static str, u64)> {
        profile.bands.iter().map(|band| (band.band, band.tally.files)).collect()
    }

    #[test]
    fn an_edge_falls_in_the_newer_band_and_the_request_instant_is_future() {
        // One file exactly on each edge, and one just past it.
        let mut rows = vec![file(NOW + 5, 1, Some(false)), file(NOW, 1, Some(false))];
        for (_, edge) in &DATED[1..] {
            rows.push(file(NOW - edge, 1, Some(false)));
            rows.push(file(NOW - edge - 1, 1, Some(false)));
        }
        let profile = age_profile(&rows, Some(NOW), TimeBasis::Changed, true);
        assert_eq!(
            labels(&profile),
            [
                ("future", 2),
                ("under 1h", 1),
                ("1h to 1d", 2),
                ("1d to 7d", 2),
                ("7d to 30d", 2),
                ("30d to 90d", 2),
                ("90d to 1y", 2),
                ("1y to 3y", 2),
                ("over 3y", 1),
            ]
        );
        assert_eq!(profile.total.files, rows.len() as u64);
        assert_eq!(profile.bands[8].cumulative.files, 16);
        // Every file here is dated by its change time; on the modification clock they are
        // all from the epoch, the oldest band.
        let modified = age_profile(&rows, Some(NOW), TimeBasis::Modified, true);
        assert_eq!(modified.bands[8].tally.files, 16);
        assert_eq!((profile.timed_by_modification, modified.timed_by_modification), (0, 0));
    }

    #[test]
    fn a_file_with_no_observed_status_change_is_counted_on_the_change_clock() {
        let mut blind = file(NOW - 10, 1, Some(false));
        blind.status_observed = false;
        let rows = [blind, file(NOW - 10, 1, Some(false))];
        assert_eq!(
            age_profile(&rows, Some(NOW), TimeBasis::Changed, true).timed_by_modification,
            1
        );
        assert_eq!(
            age_profile(&rows, Some(NOW), TimeBasis::Modified, true).timed_by_modification,
            0
        );
    }

    #[test]
    fn each_band_is_the_window_it_names() {
        let profile = age_profile(&[], Some(NOW), TimeBasis::Changed, true);
        let windows: Vec<_> = profile
            .bands
            .iter()
            .map(|band| (band.band, band.since_age_ns, band.before_age_ns))
            .collect();
        assert_eq!(windows[0], ("future", Some(0), None));
        assert_eq!(windows[1], ("under 1h", Some(HOUR), Some(0)));
        assert_eq!(windows[6], ("90d to 1y", Some(YEAR), Some(90 * DAY)));
        assert_eq!(windows[8], ("over 3y", None, Some(3 * YEAR)));
        assert_eq!(windows.len(), 9);
        // As instants, `since` is inclusive and `before` exclusive at the request instant.
        assert_eq!(
            (profile.bands[1].since_ns, profile.bands[1].before_ns),
            (Some(NOW - HOUR), Some(NOW))
        );
        // An empty profile keeps every band, with an observed ignored part of zero.
        assert!(profile.bands.iter().all(|band| band.tally == AgeTally::empty(true)));
    }

    #[test]
    fn the_ignored_part_is_unknown_wherever_a_counted_file_is_unclassified() {
        let rows = [
            file(NOW - 10, 7, Some(true)),
            file(NOW - 10, 3, Some(false)),
            file(NOW - 2 * DAY, 5, None),
        ];
        let profile = age_profile(&rows, Some(NOW), TimeBasis::Changed, true);
        let share = |index: usize| profile.bands[index].tally.ignored.map(|share| share.bytes);
        assert_eq!(share(1), Some(7));
        assert_eq!(share(3), None);
        assert_eq!(share(0), Some(0));
        assert_eq!(profile.total.ignored, None);
        assert_eq!(
            profile.bands[1].cumulative.ignored,
            share(1).map(|bytes| { IgnoredTally { files: 1, dirs: 0, bytes, allocated: bytes } })
        );
        // Unobserved rules leave every band without one, empty bands included.
        let blind = age_profile(&rows, Some(NOW), TimeBasis::Changed, false);
        assert!(blind.bands.iter().all(|band| band.tally.ignored.is_none()));
    }

    #[test]
    fn without_an_instant_no_file_is_banded_and_the_total_holds_them() {
        let rows = [file(NOW, 1, Some(false))];
        let profile = age_profile(&rows, None, TimeBasis::Changed, true);
        assert_eq!(profile.total.files, 1);
        assert!(profile.bands.iter().all(|band| band.tally.files == 0));
        assert!(
            profile.bands.iter().all(|band| band.since_ns.is_none() && band.before_ns.is_none())
        );
    }
}
