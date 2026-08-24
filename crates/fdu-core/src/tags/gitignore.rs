//! The `gitignore` tag rule's evaluator: every `.gitignore` under a root, composed.
//!
//! Behind the `gitignore` cargo feature, which is the only dependency the tag model costs.
//! The model itself is always on and pulls nothing; this rule needs a matcher engine, and
//! the one worth having is the one ripgrep and `fd` already use.
//!
//! # Why one matcher per directory rather than one for the tree
//!
//! A pattern in `docs/.gitignore` is relative to `docs/`, not to the root. Composing every
//! file into a single [`Gitignore`] would reinterpret every nested pattern against the
//! wrong base — `/build` in `docs/.gitignore` means `docs/build`, and a single matcher
//! would read it as the root's `build`. So each `.gitignore` gets its own matcher, keyed
//! by the directory it governs, and a path is decided by walking its ancestor chain.
//!
//! # Precedence
//!
//! Git's rule is that the deepest matching file wins, and within a file the last matching
//! pattern wins. [`Gitignore`] already implements the second. The first is why
//! [`GitignoreSet::is_ignored`] walks from the entry's own directory upward and returns on
//! the first matcher that expresses an opinion: a `!keep.log` in `docs/.gitignore` has to
//! beat a `*.log` at the root, and a set that consulted the root first would invert that.
//! Correct negation is the whole point of using a real matcher instead of a glob list.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ignore::gitignore::{Gitignore, GitignoreBuilder};

/// The name of the control file this rule reads.
pub(crate) const CONTROL_FILE: &str = ".gitignore";

/// Every `.gitignore` under one root, each bound to the directory it governs.
#[derive(Clone, Debug, Default)]
pub struct GitignoreSet {
    /// Keyed by the directory the file sits in, relative to the root. The root's own
    /// `.gitignore` is keyed by the empty path.
    ///
    /// A `BTreeMap` rather than a hash map because the number of `.gitignore` files in a
    /// tree is small — tens, not thousands — and ordered iteration makes the built set
    /// reproducible, which a fingerprint over it would need.
    by_directory: BTreeMap<PathBuf, Gitignore>,
    /// Whether this set has been shown a tree at all.
    ///
    /// `Default` leaves it false, which is the safe direction: a set nobody bound is one
    /// nobody should be evaluating.
    bound: bool,
}

impl GitignoreSet {
    /// Read the `.gitignore` in each named directory, skipping what cannot be read.
    ///
    /// The directories are *given*, not discovered, and that is the whole design. This set
    /// used to find its own control files by walking the tree with `ignore`'s walker,
    /// which made a tagging option pay for a second full traversal: `--cache only`, whose
    /// entire contract is that it does not touch the tree, opened by walking it; a cold
    /// scan visited every directory twice; and every `.gitignore` save re-walked the root.
    /// The index already lists every control file in the tree, so the caller hands them
    /// down and this reads exactly those files.
    ///
    /// A control file that cannot be parsed is not a fatal error: the tree is still
    /// answerable, just with one fewer opinion in it. Refusing to open an index because a
    /// `.gitignore` had a bad line would make a tagging option able to fail a scan, which
    /// is a far worse trade than tagging that directory less precisely. A file deleted
    /// since the index recorded it arrives here too, and is skipped the same way.
    pub fn from_directories<'a, I>(root: &Path, directories: I) -> Self
    where
        I: IntoIterator<Item = &'a Path>,
    {
        let mut by_directory = BTreeMap::new();
        for relative in directories {
            let directory = root.join(relative);
            // The builder's root is the governing directory, so a pattern anchored with a
            // leading slash anchors *there* -- which is what git means by it.
            let mut builder = GitignoreBuilder::new(&directory);
            if builder.add(directory.join(CONTROL_FILE)).is_some() {
                continue;
            }
            if let Ok(matcher) = builder.build() {
                by_directory.insert(relative.to_path_buf(), matcher);
            }
        }
        Self { by_directory, bound: true }
    }

    /// Whether any governing `.gitignore` ignores this relative path.
    ///
    /// Deepest first: the nearest control file that expresses an opinion decides, so a
    /// nested `!keep.log` beats a root-level `*.log`. A matcher with no opinion about this
    /// path is passed over rather than treated as a "no".
    ///
    /// `is_dir` is not cosmetic. A trailing-slash pattern -- `target/`, one of the most
    /// common lines in any `.gitignore` -- matches directories only, so passing `false`
    /// for a directory leaves `target` untagged while everything inside it is tagged,
    /// through the ancestor match. A consumer would see a directory it was told to show
    /// containing nothing it was told to show.
    pub fn is_ignored(&self, relative_path: &Path, is_dir: bool) -> bool {
        if self.by_directory.is_empty() {
            return false;
        }
        // From the entry's own directory upward to the root. `ancestors()` yields the path
        // itself first, which is right: a `.gitignore` inside a directory governs that
        // directory's own contents, and the directory is decided by its parent's file.
        let mut directory = relative_path.parent();
        while let Some(current) = directory {
            if let Some(matcher) = self.by_directory.get(current) {
                // The path is matched relative to the governing directory, which is the
                // base every pattern in that file was written against.
                let inside = relative_path.strip_prefix(current).unwrap_or(relative_path);
                let verdict = matcher.matched_path_or_any_parents(inside, is_dir);
                if verdict.is_ignore() {
                    return true;
                }
                if verdict.is_whitelist() {
                    return false;
                }
            }
            directory = current.parent();
        }
        false
    }

    /// A set that has not been bound to a tree yet.
    ///
    /// Distinct from a bound set that found no control files, and the distinction matters:
    /// both ignore nothing, but one of them is an answer and the other is the absence of
    /// one. [`TagRules::needs_binding`](crate::tags::TagRules::needs_binding) reports it so
    /// the open path can be asserted to close the window before anything reads a tag.
    pub fn unbound() -> Self {
        Self { by_directory: BTreeMap::new(), bound: false }
    }

    /// Whether this set is still waiting to be bound to a tree.
    pub fn is_unbound(&self) -> bool {
        !self.bound
    }

    /// Whether this set read anything at all.
    pub fn is_empty(&self) -> bool {
        self.by_directory.is_empty()
    }

    /// Control files this set was built from, as the directories they govern.
    ///
    /// Used to decide which subtree a changed control file invalidates without rebuilding
    /// first, so the escalation names the scope that actually moved.
    pub fn governed_directories(&self) -> impl Iterator<Item = &Path> {
        self.by_directory.keys().map(PathBuf::as_path)
    }
}

/// Whether a relative path is a control file this rule reads.
///
/// Named here rather than compared inline anywhere else, so "what counts as a control
/// file" has one definition and adding a second one later is a change to this function.
pub(crate) fn is_control_file(relative_path: &Path) -> bool {
    relative_path.file_name() == Some(std::ffi::OsStr::new(CONTROL_FILE))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Write a tree of control files and check what the composed set decides.
    fn tree(files: &[(&str, &str)]) -> (tempfile::TempDir, GitignoreSet) {
        let dir = tempfile::tempdir().expect("a temporary directory");
        for (path, contents) in files {
            let full = dir.path().join(path);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent).expect("mkdir");
            }
            std::fs::write(&full, contents).expect("write");
        }
        // The directories the fixture put a control file in -- which is exactly what an
        // index hands over in production, so the tests drive the same entry point.
        let governing: Vec<PathBuf> = files
            .iter()
            .filter(|(path, _)| is_control_file(Path::new(path)))
            .map(|(path, _)| Path::new(path).parent().unwrap_or(Path::new("")).to_path_buf())
            .collect();
        let set =
            GitignoreSet::from_directories(dir.path(), governing.iter().map(PathBuf::as_path));
        (dir, set)
    }

    #[test]
    fn an_empty_tree_ignores_nothing_and_says_so_without_walking_anything() {
        let (_dir, set) = tree(&[]);
        assert!(set.is_empty());
        assert!(!set.is_ignored(Path::new("anything"), false));
    }

    #[test]
    fn a_root_control_file_governs_the_whole_tree() {
        let (_dir, set) = tree(&[(".gitignore", "*.log\ntarget/\n")]);
        assert!(set.is_ignored(Path::new("debug.log"), false));
        assert!(set.is_ignored(Path::new("deep/nested/debug.log"), false));
        assert!(set.is_ignored(Path::new("target/release/binary"), false));
        assert!(!set.is_ignored(Path::new("src/main.rs"), false));
    }

    /// The deepest opinion wins, which is the case a glob list gets wrong.
    ///
    /// A single matcher built from every file at once would read `!keep.log` against the
    /// root and never apply it where it was written; consulting the root first would let
    /// `*.log` decide before the nested file was ever asked.
    #[test]
    fn a_nested_negation_beats_a_broader_rule_above_it() {
        let (_dir, set) = tree(&[(".gitignore", "*.log\n"), ("docs/.gitignore", "!keep.log\n")]);
        assert!(set.is_ignored(Path::new("debug.log"), false), "the root rule still applies");
        assert!(set.is_ignored(Path::new("src/debug.log"), false), "and applies deeper too");
        assert!(
            !set.is_ignored(Path::new("docs/keep.log"), false),
            "the nearer file has the last word, which is git's rule"
        );
        assert!(set.is_ignored(Path::new("docs/other.log"), false), "and only about what it names");
    }

    /// A leading slash anchors to the file's own directory, not to the root.
    ///
    /// This is the whole reason the set is per directory. Composed into one matcher,
    /// `/build` in `docs/.gitignore` would anchor at the root and ignore the wrong tree.
    #[test]
    fn an_anchored_pattern_anchors_where_it_was_written() {
        let (_dir, set) = tree(&[("docs/.gitignore", "/build\n")]);
        assert!(set.is_ignored(Path::new("docs/build"), false));
        assert!(!set.is_ignored(Path::new("build"), false), "the root's build is not docs' build");
        assert!(
            !set.is_ignored(Path::new("docs/deep/build"), false),
            "and an anchored pattern does not reach deeper"
        );
    }

    #[test]
    fn a_control_file_is_recognized_by_name_wherever_it_sits() {
        assert!(is_control_file(Path::new(".gitignore")));
        assert!(is_control_file(Path::new("docs/.gitignore")));
        assert!(!is_control_file(Path::new("gitignore")));
        assert!(!is_control_file(Path::new(".gitignore.bak")));
    }

    #[test]
    fn the_governed_directories_are_the_files_that_were_read() {
        let (_dir, set) = tree(&[(".gitignore", "*.log\n"), ("docs/.gitignore", "*.tmp\n")]);
        let governed: Vec<&Path> = set.governed_directories().collect();
        assert_eq!(governed, vec![Path::new(""), Path::new("docs")]);
    }
}
