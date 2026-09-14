//! Fixed rules that decide which filesystem facts belong in an index.
//!
//! Admission is scope, not query selection. A rejected row and its descendants are not
//! retained, while a control-only `.gitignore` remains an exact engine input without
//! becoming a visible row. Every filesystem producer calls these predicates before it
//! constructs an observation.

use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::path::{Component, Path};

use crate::{Attrs, EntryKind};

/// FNV-1a offset used by the stable hidden-policy identity.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a prime used by the stable hidden-policy identity.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Whether leading-dot path components are retained.
///
/// The allowlist contains exact component names. Exact names avoid adding a second glob
/// language beside query globs and `.gitignore` rules.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HiddenPolicy {
    prune: bool,
    allow: BTreeSet<OsString>,
    fingerprint: u64,
}

impl HiddenPolicy {
    /// Admit every component.
    pub fn keep_all() -> &'static Self {
        static KEEP_ALL: std::sync::LazyLock<HiddenPolicy> =
            std::sync::LazyLock::new(HiddenPolicy::default);
        &KEEP_ALL
    }

    /// Prune hidden components except for the exact names in `allow`.
    pub fn prune_hidden<I, S>(allow: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let allow = allow.into_iter().map(Into::into).collect();
        let fingerprint = hidden_fingerprint(&allow);
        Self { prune: true, allow, fingerprint }
    }

    /// Whether one path component is inside this scope.
    pub fn admits(&self, name: &OsStr) -> bool {
        !self.prune || !is_hidden(name) || self.allow.contains(name)
    }

    /// Whether the policy removes any hidden components.
    pub const fn is_pruning(&self) -> bool {
        self.prune
    }

    /// Exact allowlisted component names in deterministic order.
    pub fn allowed(&self) -> impl ExactSizeIterator<Item = &OsStr> {
        self.allow.iter().map(OsString::as_os_str)
    }

    /// Stable identity derived from the normalized policy.
    pub const fn fingerprint(&self) -> u64 {
        self.fingerprint
    }
}

/// What a producer should retain for one observed filesystem entry.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Disposition {
    /// Retain the ordinary row and any control signal it carries.
    Retain,
    /// Retain only the exact `.gitignore` signal.
    ControlOnly,
    /// Retain neither the row nor a control signal.
    Reject,
}

/// Decide one direct child after its native kind has been observed.
pub(crate) fn decide(
    name: &OsStr,
    kind: EntryKind,
    hidden: &HiddenPolicy,
    exclude_special: bool,
) -> Disposition {
    if !hidden.admits(name) {
        return if name == crate::control::CONTROL_FILE_NAME {
            Disposition::ControlOnly
        } else {
            Disposition::Reject
        };
    }
    if exclude_special && kind == EntryKind::Other {
        Disposition::Reject
    } else {
        Disposition::Retain
    }
}

/// Decide a complete relative path, rejecting a pruned ancestor before its final entry.
pub(crate) fn decide_path(
    path: &Path,
    kind: EntryKind,
    hidden: &HiddenPolicy,
    exclude_special: bool,
) -> Disposition {
    let mut components = path.components().peekable();
    while let Some(component) = components.next() {
        let Component::Normal(name) = component else {
            return Disposition::Reject;
        };
        if components.peek().is_some() {
            if !hidden.admits(name) {
                return Disposition::Reject;
            }
        } else {
            return decide(name, kind, hidden, exclude_special);
        }
    }
    Disposition::Retain
}

/// Whether an admitted row may be traversed as a directory.
pub(crate) fn should_descend(
    kind: EntryKind,
    attrs: Attrs,
    parent_depth: usize,
    root_dev: u64,
    max_depth: Option<usize>,
    one_filesystem: bool,
) -> bool {
    let child_depth = parent_depth.saturating_add(1);
    let within_depth = max_depth.is_none_or(|maximum| child_depth < maximum);
    let within_filesystem = !one_filesystem || attrs.dev == root_dev || attrs.dev == 0;
    kind.is_dir() && within_depth && within_filesystem
}

fn is_hidden(name: &OsStr) -> bool {
    let bytes = name.as_encoded_bytes();
    bytes.first() == Some(&b'.') && bytes != b"." && bytes != b".."
}

fn hidden_fingerprint(allow: &BTreeSet<OsString>) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    let mut mix = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    };
    mix(b"hidden:prune\x1f");
    for name in allow {
        mix(name.as_encoded_bytes());
        mix(b"\x1f");
    }
    if hash == 0 { 1 } else { hash }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_allowlists_are_exact_normalized_scope() {
        let first = HiddenPolicy::prune_hidden([".github", ".cargo"]);
        let reordered = HiddenPolicy::prune_hidden([".cargo", ".github"]);
        let narrower = HiddenPolicy::prune_hidden([".github"]);

        assert_eq!(first, reordered);
        assert_eq!(first.fingerprint(), reordered.fingerprint());
        assert_ne!(first.fingerprint(), narrower.fingerprint());
        assert!(first.admits(OsStr::new(".github")));
        assert!(!first.admits(OsStr::new(".git")));
        assert!(first.admits(OsStr::new("src")));
    }

    #[test]
    fn hidden_controls_remain_signals_but_hidden_ancestors_do_not() {
        let hidden = HiddenPolicy::prune_hidden::<[&str; 0], &str>([]);

        assert_eq!(
            decide_path(Path::new(".gitignore"), EntryKind::File, &hidden, true),
            Disposition::ControlOnly
        );
        assert_eq!(
            decide_path(Path::new(".git/.gitignore"), EntryKind::File, &hidden, true),
            Disposition::Reject
        );
        assert_eq!(
            decide_path(Path::new("socket"), EntryKind::Other, &hidden, true),
            Disposition::Reject
        );
    }

    #[cfg(unix)]
    #[test]
    fn unix_non_utf8_hidden_names_keep_exact_identity() {
        use std::os::unix::ffi::OsStringExt;

        let allowed = OsString::from_vec(vec![b'.', 0x80]);
        let other = OsString::from_vec(vec![b'.', 0x81]);
        let policy = HiddenPolicy::prune_hidden([allowed.clone()]);

        assert!(policy.admits(&allowed));
        assert!(!policy.admits(&other));
        assert_ne!(policy.fingerprint(), HiddenPolicy::prune_hidden([other]).fingerprint());
    }

    #[cfg(windows)]
    #[test]
    fn windows_surrogates_and_separators_do_not_bypass_hidden_scope() {
        use std::os::windows::ffi::OsStringExt;

        let allowed = OsString::from_wide(&[u16::from(b'.'), 0xd800]);
        let other = OsString::from_wide(&[u16::from(b'.'), 0xd801]);
        let policy = HiddenPolicy::prune_hidden([allowed.clone()]);

        assert!(policy.admits(&allowed));
        assert!(!policy.admits(&other));
        assert_ne!(policy.fingerprint(), HiddenPolicy::prune_hidden([other]).fingerprint());
        assert_eq!(
            decide_path(Path::new(r".hidden\child"), EntryKind::File, &policy, false),
            Disposition::Reject
        );
    }
}
