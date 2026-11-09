//! The age profile: the selected regular files, partitioned by how long ago they changed.
//!
//! Change time is the newer of modification and status change
//! ([`Attrs::change_time_ns`](crate::Attrs::change_time_ns)), because the question this
//! view answers is when space arrived, and modification time misses every arrival that
//! kept an older one: a copy, an extracted archive, Cargo's crate sources.
//!
//! The bands are fixed, so every report is comparable with every other, and each is one
//! window query: the files `--kind file --changed-since <since_age> --changed-before
//! <before_age>` selects over the same selection and instant. `--kind file` is part of
//! the identity: without it a window admits a whole directory with activity in it, and
//! a band only ever holds files. Windows are half-open, `[since, before)`,
//! so an entry exactly on an edge falls in the newer band, and one stamped at the request
//! instant itself is `future`. A file whose change time was not observed is `unknown`,
//! which no window admits. The bands therefore partition the selection: their files and
//! bytes sum to what `summary` reports for it.

use crate::EntryKind;
use crate::query::{FileRow, IgnoredTally};

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
/// Files whose change time was not observed.
const UNKNOWN: &str = "unknown";
/// How many bands a profile has: the dated ones, the oldest, and unknown.
const BANDS: usize = DATED.len() + 2;

/// What the profile counts its bands by, which the section names so that no reader takes
/// it for the tree's age column, whose basis is modification activity.
pub const AGE_BASIS: &str = "changed";

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

    fn count(&mut self, row: &FileRow) {
        self.files += 1;
        self.bytes += row.bytes;
        self.allocated += row.allocated;
        match row.ignored {
            Some(true) => {
                if let Some(share) = self.ignored.as_mut() {
                    share.add(IgnoredTally {
                        files: 1,
                        dirs: 0,
                        bytes: row.bytes,
                        allocated: row.allocated,
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
    /// The band's label: `future`, `under 1h`, … `over 3y`, or `unknown`.
    pub band: &'static str,
    /// The oldest age the band holds, inclusive, in nanoseconds: its window's
    /// `--changed-since`. `None` for `over 3y`, which has no oldest age, and `unknown`.
    pub since_age_ns: Option<i64>,
    /// The newest age the band excludes, in nanoseconds: its window's `--changed-before`.
    /// `None` for `future`, which has no newer band, and `unknown`.
    pub before_age_ns: Option<i64>,
    /// [`Self::since_age_ns`] as an instant, in epoch nanoseconds, at the request instant.
    pub since_ns: Option<i64>,
    /// [`Self::before_age_ns`] as an instant.
    pub before_ns: Option<i64>,
    /// What the band holds.
    pub tally: AgeTally,
    /// What this band and every newer one hold, which answers "how much changed within
    /// this age?"; `None` for `unknown`, which has no place in that order.
    pub cumulative: Option<AgeTally>,
}

/// The selected regular files by change-time band.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgeProfile {
    /// [`AGE_BASIS`].
    pub basis: &'static str,
    /// Every band, newest first, then `over 3y` and `unknown`; empty bands included.
    pub bands: Vec<AgeBand>,
    /// Every band together: the files and bytes `summary` reports for the same selection.
    pub total: AgeTally,
}

/// The band a change time falls in at `now_ns`, by index into a profile's bands.
fn band_of(changed_ns: Option<i64>, now_ns: Option<i64>) -> usize {
    let (Some(changed), Some(now)) = (changed_ns, now_ns) else { return BANDS - 1 };
    let age = i128::from(now) - i128::from(changed);
    DATED.iter().position(|(_, oldest)| age <= i128::from(*oldest)).unwrap_or(DATED.len())
}

/// The age profile of `files`, the regular files a selection admits or covers, at
/// `now_ns`, the request instant.
///
/// `observed` says whether every root's index observed `.gitignore` rules, without which
/// no band has an ignored part. With no representable instant every file is `unknown`,
/// since no age can be measured from it.
pub(crate) fn age_profile<'a>(
    files: impl IntoIterator<Item = &'a FileRow>,
    now_ns: Option<i64>,
    observed: bool,
) -> AgeProfile {
    let mut tallies = [AgeTally::empty(observed); BANDS];
    for row in files.into_iter().filter(|row| row.kind == EntryKind::File) {
        tallies[band_of(row.changed_ns, now_ns)].count(row);
    }
    let instant = |age: Option<i64>| now_ns.zip(age).and_then(|(now, age)| now.checked_sub(age));
    let mut running = AgeTally::empty(observed);
    let mut total = AgeTally::empty(observed);
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
                index if index == DATED.len() => (OLDEST, None, Some(DATED[DATED.len() - 1].1)),
                _ => (UNKNOWN, None, None),
            };
            let cumulative = (band != UNKNOWN).then(|| {
                running.add(tally);
                running
            });
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
    AgeProfile { basis: AGE_BASIS, bands, total }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const NOW: i64 = 2_000_000_000_000_000_000;

    fn file(changed_ns: Option<i64>, bytes: u64, ignored: Option<bool>) -> FileRow {
        FileRow {
            root: 0,
            path: PathBuf::from("f"),
            kind: EntryKind::File,
            bytes,
            allocated: bytes,
            mtime_ns: 0,
            changed_ns,
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
        let mut rows = vec![file(Some(NOW + 5), 1, Some(false)), file(Some(NOW), 1, Some(false))];
        for (_, edge) in &DATED[1..] {
            rows.push(file(Some(NOW - edge), 1, Some(false)));
            rows.push(file(Some(NOW - edge - 1), 1, Some(false)));
        }
        rows.push(file(None, 1, Some(false)));
        let profile = age_profile(&rows, Some(NOW), true);
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
                ("unknown", 1),
            ]
        );
        assert_eq!(profile.total.files, rows.len() as u64);
        assert_eq!(profile.bands[8].cumulative.map(|tally| tally.files), Some(16));
        assert_eq!(profile.bands[9].cumulative, None);
    }

    #[test]
    fn each_band_is_the_window_it_names() {
        let profile = age_profile(&[], Some(NOW), true);
        let windows: Vec<_> = profile
            .bands
            .iter()
            .map(|band| (band.band, band.since_age_ns, band.before_age_ns))
            .collect();
        assert_eq!(windows[0], ("future", Some(0), None));
        assert_eq!(windows[1], ("under 1h", Some(HOUR), Some(0)));
        assert_eq!(windows[6], ("90d to 1y", Some(YEAR), Some(90 * DAY)));
        assert_eq!(windows[8], ("over 3y", None, Some(3 * YEAR)));
        assert_eq!(windows[9], ("unknown", None, None));
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
            file(Some(NOW - 10), 7, Some(true)),
            file(Some(NOW - 10), 3, Some(false)),
            file(Some(NOW - 2 * DAY), 5, None),
        ];
        let profile = age_profile(&rows, Some(NOW), true);
        let share = |index: usize| profile.bands[index].tally.ignored.map(|share| share.bytes);
        assert_eq!(share(1), Some(7));
        assert_eq!(share(3), None);
        assert_eq!(share(0), Some(0));
        assert_eq!(profile.total.ignored, None);
        assert_eq!(
            profile.bands[1].cumulative.and_then(|tally| tally.ignored),
            share(1).map(|bytes| { IgnoredTally { files: 1, dirs: 0, bytes, allocated: bytes } })
        );
        // Unobserved rules leave every band without one, empty bands included.
        let blind = age_profile(&rows, Some(NOW), false);
        assert!(blind.bands.iter().all(|band| band.tally.ignored.is_none()));
    }

    #[test]
    fn without_an_instant_every_file_is_unknown() {
        let rows = [file(Some(NOW), 1, Some(false))];
        let profile = age_profile(&rows, None, true);
        assert_eq!(profile.bands[9].tally.files, 1);
        assert!(
            profile.bands.iter().all(|band| band.since_ns.is_none() && band.before_ns.is_none())
        );
    }
}
