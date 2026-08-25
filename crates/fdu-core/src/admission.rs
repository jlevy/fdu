//! Which entries are inside the scan scope at all.
//!
//! Scope, not selection, and the difference is the whole point. A selection filter narrows
//! an answer over entries the index holds, so changing it re-reads and never re-walks; an
//! admission rule decides what the index holds, so changing it invalidates the snapshot and
//! the tree has to be read again. Everything here is therefore fingerprinted into
//! [`ScanScope`](crate::ScanScope) beside the depth and filesystem bounds it sits with.
//!
//! # Hidden paths are scope, and a `dotfile` tag is not
//!
//! Both mean the same entries -- [`TagRules`](crate::tags::TagRules)'s `dotfile` rule and
//! this share one predicate on purpose -- and they do opposite things with them. The tag
//! leaves an entry in the index and lets a query filter it, with both numbers visible and
//! a plane available to hold the complement. This removes it: a pruned entry has no row, no
//! tally, and no subtree, because the subtree is never read.
//!
//! That is why visibility could not be a promoted tag. A maintained plane for hidden
//! entries would have to walk `.git`, the caches and the virtualenvs -- routinely most of a
//! working tree -- in order to report them as excluded, and the consumer that would justify
//! paying for that wants them gone rather than counted.
//!
//! # Control files stay readable
//!
//! A `.gitignore` is a hidden file that governs entries which are not hidden, so pruning it
//! outright would answer a gitignore question with "nothing is ignored" -- a wrong answer
//! rather than a missing one. The walk records where it saw one and does not retain it, so
//! a Path-tier rule can still be bound to a tree whose dotfiles were pruned. See
//! [`ScanReport::control_dirs`](crate::scan::ScanReport::control_dirs).

use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};

/// Which entries the walk admits, beside the depth and filesystem bounds.
///
/// The default admits everything, which is fdu's own command-line default: a `du`
/// replacement counts what is there. It fingerprints to zero, so every snapshot recorded
/// before this existed stays loadable.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HiddenPolicy {
    prune: bool,
    allow: BTreeSet<OsString>,
    fingerprint: u64,
}

impl HiddenPolicy {
    /// Admit every entry: the default, and the one that fingerprints to zero.
    pub fn keep_all() -> &'static Self {
        static KEEP: std::sync::LazyLock<HiddenPolicy> =
            std::sync::LazyLock::new(HiddenPolicy::default);
        &KEEP
    }

    /// Prune hidden entries, admitting these exact names anyway.
    ///
    /// Exact names rather than patterns, deliberately. An allowlist is a statement about
    /// which specific things a tree needs kept -- `.github`, `.cargo` -- and a pattern
    /// language here would be a second glob dialect whose interaction with the tag rules
    /// nobody could hold in their head. A name that is not hidden is accepted and does
    /// nothing, because refusing it would make the allowlist's meaning depend on what the
    /// tree happens to contain.
    pub fn prune_hidden<I, S>(allow: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let allow: BTreeSet<OsString> = allow.into_iter().map(Into::into).collect();
        let fingerprint = fingerprint_of(&allow);
        Self { prune: true, allow, fingerprint }
    }

    /// Whether this entry is inside the scan scope.
    ///
    /// One `bool` when nothing is pruned, which is the default: the walk's hottest branch
    /// pays for this rule only where it is switched on.
    pub fn admits(&self, name: &OsStr) -> bool {
        if !self.prune {
            return true;
        }
        !is_hidden(name) || self.allow.contains(name)
    }

    /// Whether anything is pruned at all.
    pub const fn is_pruning(&self) -> bool {
        self.prune
    }

    /// The names admitted despite being hidden, in sorted order.
    pub fn allowed(&self) -> impl ExactSizeIterator<Item = &OsStr> {
        self.allow.iter().map(OsString::as_os_str)
    }

    /// Identity of this rule, for [`ScanScope`](crate::ScanScope).
    ///
    /// Zero exactly when nothing is pruned. A snapshot recorded under one allowlist cannot
    /// be reinterpreted under another: the entries the other one would have kept are not
    /// absent from the tree, they are absent from the *recording*, and nothing in the file
    /// distinguishes those.
    pub const fn fingerprint(&self) -> u64 {
        self.fingerprint
    }
}

/// Why an admission rule was refused.
///
/// Its own type rather than a string so both surfaces print one sentence. The command line
/// used to validate these values itself and the Python package validated them again in its
/// dataclass, which produced two messages for one mistake -- `--hidden` against `hidden`,
/// double quotes against single -- and the parity harness recorded the pair as a
/// difference between the surfaces. It was a difference between two copies of the same
/// rule.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AdmissionError {
    /// A mode that is neither `keep` nor `prune`.
    UnknownMode(String),
    /// An allowlist given where nothing is being pruned.
    ///
    /// Refused rather than ignored: it can only have been written by someone who believed
    /// pruning was on, and admitting everything would answer their question with the number
    /// they had asked not to receive.
    AllowlistWithoutPruning,
}

impl std::fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownMode(mode) => {
                write!(f, "invalid hidden mode {mode:?}: expected one of keep, prune")
            }
            Self::AllowlistWithoutPruning => write!(
                f,
                "a hidden allowlist needs hidden pruning: with hidden entries kept there is \
                 nothing for an allowlist to admit"
            ),
        }
    }
}

impl std::error::Error for AdmissionError {}

/// Resolve a mode name and an allowlist into a rule, or `None` for the default.
///
/// The one place either value is judged, because the surfaces must reject the same input
/// with the same words. `None` is returned rather than a keep-everything policy so a caller
/// can leave [`ScanConfig::hidden`](crate::ScanConfig::hidden) unset and pay nothing.
pub fn parse_policy<I, S>(mode: &str, allow: I) -> Result<Option<HiddenPolicy>, AdmissionError>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let allow: Vec<OsString> = allow.into_iter().map(Into::into).collect();
    match mode {
        "keep" if allow.is_empty() => Ok(None),
        "keep" => Err(AdmissionError::AllowlistWithoutPruning),
        "prune" => Ok(Some(HiddenPolicy::prune_hidden(allow))),
        other => Err(AdmissionError::UnknownMode(other.to_string())),
    }
}

/// A name beginning with a dot, which is what "hidden" means here.
///
/// The same predicate the `dotfile` tag rule applies, and it has to be: the two axes are
/// distinguished by what they do with an entry, never by which entries they mean. Two
/// definitions of hidden would make `--hidden prune` and `--not-tag dotfile` disagree about
/// one file, and the disagreement would read as a bug in whichever surface was consulted
/// second.
///
/// Windows carries a `FILE_ATTRIBUTE_HIDDEN` bit that this does not read. Adding it would
/// mean the same tree admitted different entries on different platforms, and the parity
/// corpus would have to record that as a permanent deviation; the leading dot is what every
/// tool in this space means by hidden, on every platform.
pub(crate) fn is_hidden(name: &OsStr) -> bool {
    let bytes = name.as_encoded_bytes();
    bytes.first() == Some(&b'.') && bytes != b"." && bytes != b".."
}

/// Hash the rule and its allowlist, in the sorted order the set already holds them.
///
/// Sorted because the fingerprint is an identity, and a caller listing the same two names
/// in the other order has not asked a different question.
fn fingerprint_of(allow: &BTreeSet<OsString>) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    mix(b"hidden:prune\x1f");
    for name in allow {
        mix(name.as_encoded_bytes());
        mix(b"\x1f");
    }
    // Never zero, which is reserved for "nothing is pruned": an allowlist that happened to
    // hash to zero would read as the default and load a snapshot recorded without pruning.
    if hash == 0 { 1 } else { hash }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_admits_everything_and_invalidates_no_snapshot() {
        let keep = HiddenPolicy::keep_all();
        assert!(keep.admits(OsStr::new(".git")));
        assert!(keep.admits(OsStr::new("src")));
        assert!(!keep.is_pruning());
        assert_eq!(keep.fingerprint(), 0, "every snapshot written before this existed");
    }

    #[test]
    fn pruning_removes_hidden_names_except_the_ones_named() {
        let policy = HiddenPolicy::prune_hidden([".github", ".cargo"]);
        assert!(!policy.admits(OsStr::new(".git")));
        assert!(!policy.admits(OsStr::new(".env")));
        assert!(policy.admits(OsStr::new(".github")), "allowlisted");
        assert!(policy.admits(OsStr::new(".cargo")), "allowlisted");
        assert!(policy.admits(OsStr::new("src")));
        assert!(policy.admits(OsStr::new("..")), "a parent component is not a hidden entry");
        assert_ne!(policy.fingerprint(), 0);
    }

    /// The allowlist is an identity, so its order is not part of it and its content is.
    #[test]
    fn the_fingerprint_follows_the_allowlist_and_not_the_order_it_was_written_in() {
        let one = HiddenPolicy::prune_hidden([".github", ".cargo"]);
        let other = HiddenPolicy::prune_hidden([".cargo", ".github"]);
        assert_eq!(one.fingerprint(), other.fingerprint(), "same question, other spelling");

        let narrower = HiddenPolicy::prune_hidden([".github"]);
        assert_ne!(one.fingerprint(), narrower.fingerprint(), "a different retained set");

        let bare = HiddenPolicy::prune_hidden::<[&str; 0], &str>([]);
        assert_ne!(bare.fingerprint(), 0, "pruning nothing extra still prunes");
        assert_ne!(bare.fingerprint(), narrower.fingerprint());
    }

    /// Both surfaces reject the same input with the same words, because one rule judges it.
    #[test]
    fn a_refused_mode_or_allowlist_says_the_same_thing_to_every_caller() {
        assert!(parse_policy::<[&str; 0], &str>("keep", []).expect("default").is_none());
        assert!(parse_policy::<[&str; 0], &str>("prune", []).expect("prunes").is_some());
        assert!(parse_policy("prune", [".github"]).expect("prunes").is_some());

        let unknown = parse_policy::<[&str; 0], &str>("sometimes", []).expect_err("refused");
        assert_eq!(
            unknown.to_string(),
            "invalid hidden mode \"sometimes\": expected one of keep, prune",
        );

        let stray = parse_policy("keep", [".github"]).expect_err("refused");
        assert!(stray.to_string().contains("needs hidden pruning"), "{stray}");
    }

    /// Hidden means exactly what the `dotfile` tag means, because the axes differ in what
    /// they do rather than in which entries they are about.
    #[test]
    fn hidden_means_the_same_entries_the_dotfile_tag_means() {
        let rules = crate::tags::TagRules::from_names(["dotfile"]).expect("enables");
        for name in [".env", ".git", "src", "main.rs", "a.b.c", ".."] {
            let tagged = rules.evaluate(OsStr::new(name), false, || {
                std::borrow::Cow::Borrowed(std::path::Path::new(name))
            }) != 0;
            assert_eq!(tagged, is_hidden(OsStr::new(name)), "{name} disagrees across the axes");
        }
    }
}
