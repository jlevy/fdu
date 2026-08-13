//! File-type recognition.
//!
//! Phase 1 ships only the cheapest tier of the planned cascade: the compound-tail
//! extension. The full design is a priority-ordered cascade of declarative rules
//! (exact filename, extension, shebang, bounded content probe) compiled to automata at
//! build time, deliberately compatible with metabrowser's `[[kind]]` manifest dialect.
//! Nothing here should grow runtime rule parsing; that is what the compiled dialect is
//! for.

use std::ffi::OsStr;

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
/// use std::ffi::OsStr;
/// use fdu::classify::derive_ext;
///
/// assert_eq!(derive_ext(OsStr::new("archive.tar.gz")).as_deref(), Some(".tar.gz"));
/// assert_eq!(derive_ext(OsStr::new("notes.MD")).as_deref(), Some(".md"));
/// assert_eq!(derive_ext(OsStr::new(".gitignore")), None);
/// assert_eq!(derive_ext(OsStr::new("README")), None);
/// ```
pub fn derive_ext(name: &OsStr) -> Option<String> {
    derive_ext_native(name)
}

#[cfg(unix)]
fn derive_ext_native(name: &OsStr) -> Option<String> {
    use std::os::unix::ffi::OsStrExt;

    derive_ext_units(name.as_bytes(), b'.', |unit| unit.to_ascii_lowercase())
        .and_then(|units| String::from_utf8(units).ok())
}

#[cfg(windows)]
fn derive_ext_native(name: &OsStr) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;

    let units: Vec<u16> = name.encode_wide().collect();
    derive_ext_units(&units, u16::from(b'.'), |unit| {
        if unit <= u16::from(u8::MAX) {
            u16::from(u8::try_from(unit).expect("bounded to one byte").to_ascii_lowercase())
        } else {
            unit
        }
    })
    .and_then(|extension| String::from_utf16(&extension).ok())
}

#[cfg(not(any(unix, windows)))]
fn derive_ext_native(name: &OsStr) -> Option<String> {
    derive_ext_str(name.to_str()?)
}

fn derive_ext_units<T: Copy + Eq + From<u8>>(
    name: &[T],
    dot: T,
    lowercase: impl Fn(T) -> T,
) -> Option<Vec<T>> {
    let searchable = if name.first() == Some(&dot) { &name[1..] } else { name };
    let dot_index = searchable.iter().rposition(|unit| *unit == dot)?;
    let (stem, last) = searchable.split_at(dot_index);
    if last.len() <= 1 {
        return None;
    }

    let mut extension = Vec::new();
    if let Some(inner_dot) = stem.iter().rposition(|unit| *unit == dot) {
        let inner = &stem[inner_dot..];
        let tar = [
            dot,
            lowercase_ascii_unit(b't', &lowercase),
            lowercase_ascii_unit(b'a', &lowercase),
            lowercase_ascii_unit(b'r', &lowercase),
        ];
        if inner.len() == tar.len() && inner.iter().copied().map(&lowercase).eq(tar) {
            extension.extend(inner.iter().copied().map(&lowercase));
        }
    }
    extension.extend(last.iter().copied().map(lowercase));
    Some(extension)
}

fn lowercase_ascii_unit<T: Copy + From<u8>>(byte: u8, lowercase: &impl Fn(T) -> T) -> T {
    lowercase(T::from(byte))
}

#[cfg(not(any(unix, windows)))]
fn derive_ext_str(name: &str) -> Option<String> {
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
    use std::ffi::OsStr;

    #[test]
    fn plain_extensions_lowercase() {
        assert_eq!(derive_ext(OsStr::new("main.RS")).as_deref(), Some(".rs"));
        assert_eq!(derive_ext(OsStr::new("Photo.JPEG")).as_deref(), Some(".jpeg"));
    }

    #[test]
    fn tar_pairs_fold_into_one_extension() {
        assert_eq!(derive_ext(OsStr::new("archive.tar.gz")).as_deref(), Some(".tar.gz"));
        assert_eq!(derive_ext(OsStr::new("archive.tar.zst")).as_deref(), Some(".tar.zst"));
        assert_eq!(derive_ext(OsStr::new("archive.TAR.BZ2")).as_deref(), Some(".tar.bz2"));
        // Only .tar folds; an unrelated inner segment is not part of the extension.
        assert_eq!(derive_ext(OsStr::new("release.v2.zip")).as_deref(), Some(".zip"));
    }

    #[test]
    fn names_without_a_usable_extension() {
        assert_eq!(derive_ext(OsStr::new("README")), None);
        assert_eq!(derive_ext(OsStr::new(".gitignore")), None);
        assert_eq!(derive_ext(OsStr::new(".bashrc")), None);
        assert_eq!(derive_ext(OsStr::new("trailing.")), None);
        assert_eq!(derive_ext(OsStr::new("")), None);
    }

    #[test]
    fn dotfiles_with_a_real_extension_keep_it() {
        assert_eq!(derive_ext(OsStr::new(".eslintrc.json")).as_deref(), Some(".json"));
    }

    #[cfg(unix)]
    #[test]
    fn ascii_extension_survives_non_unicode_stem() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let name = OsString::from_vec(vec![b'n', 0xff, b'.', b'R', b'S']);
        assert_eq!(derive_ext(&name).as_deref(), Some(".rs"));
    }

    #[cfg(windows)]
    #[test]
    fn ascii_extension_survives_unpaired_wide_stem() {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;

        let name = OsString::from_wide(&[
            u16::from(b'n'),
            0xd800,
            u16::from(b'.'),
            u16::from(b'R'),
            u16::from(b'S'),
        ]);
        assert_eq!(derive_ext(&name).as_deref(), Some(".rs"));
    }
}
