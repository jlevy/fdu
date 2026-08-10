//! File-type recognition.
//!
//! Phase 1 ships only the cheapest tier of the planned cascade: the compound-tail
//! extension. The full design is a priority-ordered cascade of declarative rules
//! (exact filename, extension, shebang, bounded content probe) compiled to automata at
//! build time, deliberately compatible with metabrowser's `[[kind]]` manifest dialect.
//! Nothing here should grow runtime rule parsing; that is what the compiled dialect is
//! for.

/// Extract the compound-tail extension from a file name, lowercased and including the
/// leading dot.
///
/// "Compound tail" means `archive.tar.gz` yields `.tar.gz` rather than `.gz`, because
/// the pair is what a human means by the file's type. Only `.tar` is folded this way;
/// generalizing to an arbitrary set of compound stems belongs in the rule dialect, not
/// in a hand-maintained list here.
///
/// Returns `None` for names with no usable extension, including dotfiles like
/// `.gitignore` — a leading dot marks a hidden file, it does not introduce an extension.
///
/// ```
/// use fdu::classify::derive_ext;
///
/// assert_eq!(derive_ext("archive.tar.gz").as_deref(), Some(".tar.gz"));
/// assert_eq!(derive_ext("notes.MD").as_deref(), Some(".md"));
/// assert_eq!(derive_ext(".gitignore"), None);
/// assert_eq!(derive_ext("README"), None);
/// ```
pub fn derive_ext(name: &str) -> Option<String> {
    // Skip a leading dot so dotfiles are not read as all-extension.
    let searchable = name.strip_prefix('.').unwrap_or(name);
    let dot = searchable.rfind('.')?;
    let (stem, last) = searchable.split_at(dot);
    if last.len() <= 1 {
        // A trailing dot with nothing after it is not an extension.
        return None;
    }

    if let Some(inner_dot) = stem.rfind('.') {
        if stem[inner_dot..].eq_ignore_ascii_case(".tar") {
            return Some(format!(".tar{}", last.to_ascii_lowercase()));
        }
    }

    Some(last.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::derive_ext;

    #[test]
    fn plain_extensions_lowercase() {
        assert_eq!(derive_ext("main.RS").as_deref(), Some(".rs"));
        assert_eq!(derive_ext("Photo.JPEG").as_deref(), Some(".jpeg"));
    }

    #[test]
    fn tar_pairs_fold_into_one_extension() {
        assert_eq!(derive_ext("archive.tar.gz").as_deref(), Some(".tar.gz"));
        assert_eq!(derive_ext("archive.tar.zst").as_deref(), Some(".tar.zst"));
        assert_eq!(derive_ext("archive.TAR.BZ2").as_deref(), Some(".tar.bz2"));
        // Only .tar folds; an unrelated inner segment is not part of the extension.
        assert_eq!(derive_ext("release.v2.zip").as_deref(), Some(".zip"));
    }

    #[test]
    fn names_without_a_usable_extension() {
        assert_eq!(derive_ext("README"), None);
        assert_eq!(derive_ext(".gitignore"), None);
        assert_eq!(derive_ext(".bashrc"), None);
        assert_eq!(derive_ext("trailing."), None);
        assert_eq!(derive_ext(""), None);
    }

    #[test]
    fn dotfiles_with_a_real_extension_keep_it() {
        assert_eq!(derive_ext(".eslintrc.json").as_deref(), Some(".json"));
    }
}
