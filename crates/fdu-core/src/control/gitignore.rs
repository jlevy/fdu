//! Small fixed gitignore matcher for the inventory partition.
//!
//! This is intentionally not a general pattern API. It implements the path semantics
//! the `MetaBrowser` client already exercises—comments and escapes, negation, rooted and
//! basename patterns, directory patterns, `*`, `?`, bracket expressions, and `**`—without
//! adding the regex/glob dependency stack to every shipped binary. Matching is byte
//! exact and case-sensitive on every platform; it does not inherit Git's repository-local
//! `core.ignorecase` setting.
//!
//! **Bracket expressions** are read the way git's `wildmatch` reads them, and a table of
//! verdicts recorded from `git check-ignore` pins each rule: `!` or `^` negation, a
//! leading `]` as a member, backslash escapes, ranges, and the twelve `[:name:]` classes
//! `alnum`, `alpha`, `blank`, `cntrl`, `digit`, `graph`, `lower`, `print`, `punct`,
//! `space`, `upper`, and `xdigit`. The classes are git's own ASCII-only ones, so the
//! locale never matters and no byte of a non-ASCII name is in any class. A class, like
//! `?`, matches one byte, so a multi-byte UTF-8 character needs one per byte. Collating
//! symbols such as `[.a.]` and equivalence classes such as `[=a=]` are not special, as in
//! git. A `/` inside a bracket expression is a member of the set, not a separator. An
//! expression that never closes, or that names an unknown class, makes its whole line
//! match nothing, which is also git's answer.
//!
//! **An escaped `/`** is a separator, because git reads `\/` as a literal `/` and in a
//! path only a separator is one: `a\/b` matches `a/b`, not a directory `a\` holding `b`.
//! Git's two exceptions hold, pinned by recorded verdicts too. A leading `\/` is not an
//! anchor, so its line matches nothing; and `**\/` matches one or more directories, never
//! zero, because git's zero-directory shortcut looks for an unescaped `/`. A trailing `/`
//! makes a pattern directory-only whether or not it is escaped, and a backslash it leaves
//! with nothing to escape makes the line match nothing.

use std::path::{Component, Path};

#[derive(Clone, Debug, Default)]
pub(super) struct Gitignore {
    patterns: Vec<Pattern>,
}

#[derive(Clone, Debug)]
struct Pattern {
    ignored: bool,
    directory_only: bool,
    matches_path: bool,
    segments: Vec<Segment>,
}

#[derive(Clone, Debug)]
enum Segment {
    DoubleStar,
    /// A `**` written before an escaped separator, as `**\/`, which git gives no
    /// zero-directory shortcut, so it matches one or more components.
    DoubleStarOneOrMore,
    Glob(Vec<u8>),
}

impl Gitignore {
    pub(super) fn parse(source: &[u8]) -> Self {
        let patterns = source.split(|byte| *byte == b'\n').filter_map(Pattern::parse).collect();
        Self { patterns }
    }

    /// Last matching line wins. `Some(false)` is an explicit negation; `None` means this
    /// control file expressed no opinion.
    pub(super) fn matches(&self, relative: &Path, is_dir: bool) -> Option<bool> {
        let components: Vec<&[u8]> = relative
            .components()
            .filter_map(|component| match component {
                Component::Normal(value) => Some(value.as_encoded_bytes()),
                Component::CurDir
                | Component::ParentDir
                | Component::RootDir
                | Component::Prefix(_) => None,
            })
            .collect();
        self.matches_components(&components, is_dir)
    }

    fn matches_components(&self, components: &[&[u8]], is_dir: bool) -> Option<bool> {
        self.patterns
            .iter()
            .filter(|pattern| pattern.matches(components, is_dir))
            .map(|pattern| pattern.ignored)
            .next_back()
    }
}

impl Pattern {
    fn parse(raw: &[u8]) -> Option<Self> {
        let mut line = raw.strip_suffix(b"\r").unwrap_or(raw);
        line = trim_unescaped_spaces(line);
        if line.is_empty() || line.first() == Some(&b'#') {
            return None;
        }

        let (ignored, mut body) =
            if line.first() == Some(&b'!') { (false, &line[1..]) } else { (true, line) };
        if body.is_empty() {
            return None;
        }

        // Git strips a trailing `/` before it looks at escapes, so `\/` there still makes
        // the pattern directory-only.
        let directory_only = body.last() == Some(&b'/');
        if directory_only {
            body = &body[..body.len() - 1];
        }
        // A backslash with nothing after it to escape fails git's match, so the line can
        // never match anything.
        if body.last() == Some(&b'\\') && is_escaped(body, body.len()) {
            return None;
        }
        let anchored = body.first() == Some(&b'/');
        if anchored {
            body = &body[1..];
        }
        // An escaped leading `/` is not an anchor: git must match it against a separator
        // before the first component, and no path has one.
        if body.is_empty() || body.starts_with(b"\\/") {
            return None;
        }

        let matches_path = anchored || body.contains(&b'/');
        let mut segments = Vec::new();
        // A malformed bracket expression aborts git's match wherever it appears, so the
        // line can never match anything and is dropped as if it were a comment.
        for (segment, before_escaped_separator) in
            split_segments(body)?.into_iter().filter(|(segment, _)| !segment.is_empty())
        {
            let segment = if matches_path && segment == b"**" {
                if before_escaped_separator {
                    Segment::DoubleStarOneOrMore
                } else {
                    Segment::DoubleStar
                }
            } else {
                Segment::Glob(normalize_glob(segment))
            };
            if matches!(segment, Segment::DoubleStar)
                && matches!(segments.last(), Some(Segment::DoubleStar))
            {
                continue;
            }
            segments.push(segment);
        }
        if segments.is_empty() {
            return None;
        }
        Some(Self { ignored, directory_only, matches_path, segments })
    }

    fn matches(&self, path: &[&[u8]], is_dir: bool) -> bool {
        if path.is_empty() {
            return false;
        }
        if !self.matches_path {
            let Some(Segment::Glob(pattern)) = self.segments.first() else {
                return false;
            };
            return path.last().is_some_and(|component| glob_matches(pattern, component))
                && (!self.directory_only || is_dir);
        }
        segment_path_matches(&self.segments, path, self.directory_only, is_dir)
    }
}

/// Split a pattern body at each `/` git treats as a separator, escaped or not.
///
/// Each segment comes with whether the separator after it was escaped, as `\/`, which
/// changes what a `**` before it may match. A `/` inside a bracket expression is a member
/// of the set, which no path component can contain, not a separator. A backslash hides the
/// byte after it from bracket parsing. `None` means a bracket expression is malformed,
/// which aborts git's whole match.
fn split_segments(body: &[u8]) -> Option<Vec<(&[u8], bool)>> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut position = 0;
    while position < body.len() {
        match body[position] {
            b'/' => {
                segments.push((&body[start..position], false));
                position += 1;
                start = position;
            }
            // Git reads `\/` as a literal `/`, and in a path only a separator is one.
            b'\\' if body.get(position + 1) == Some(&b'/') => {
                segments.push((&body[start..position], true));
                position += 2;
                start = position;
            }
            b'\\' => position += 2,
            b'[' => position += class_match(&body[position..], 0)?.1,
            _ => position += 1,
        }
    }
    segments.push((&body[start..], false));
    Some(segments)
}

fn normalize_glob(pattern: &[u8]) -> Vec<u8> {
    let mut normalized = Vec::with_capacity(pattern.len());
    let mut position = 0;
    let mut previous_wildcard = false;
    while position < pattern.len() {
        if pattern[position] == b'\\' && position + 1 < pattern.len() {
            normalized.extend_from_slice(&pattern[position..=position + 1]);
            position += 2;
            previous_wildcard = false;
        } else if pattern[position] == b'[' {
            // A `*` inside a class is a member, so a class is copied unchanged. The
            // pattern was validated when it was split, so the class always closes.
            let length = class_match(&pattern[position..], 0).map_or(1, |(_, length)| length);
            normalized.extend_from_slice(&pattern[position..position + length]);
            position += length;
            previous_wildcard = false;
        } else {
            let byte = pattern[position];
            if byte != b'*' || !previous_wildcard {
                normalized.push(byte);
            }
            previous_wildcard = byte == b'*';
            position += 1;
        }
    }
    normalized
}

fn segment_path_matches(
    pattern: &[Segment],
    path: &[&[u8]],
    directory_only: bool,
    target_is_dir: bool,
) -> bool {
    let mut previous = vec![false; path.len() + 1];
    previous[0] = true;

    for (position, segment) in pattern.iter().enumerate() {
        let mut current = vec![false; path.len() + 1];
        match segment {
            Segment::DoubleStar if position + 1 < pattern.len() => {
                current[0] = previous[0];
                for path_at in 1..=path.len() {
                    current[path_at] = previous[path_at] || current[path_at - 1];
                }
            }
            Segment::DoubleStar | Segment::DoubleStarOneOrMore => {
                // Git's trailing `/**` means contents *inside* the named directory,
                // not the directory itself, and `**\/` has no zero-directory shortcut,
                // so each consumes at least one component.
                for path_at in 1..=path.len() {
                    current[path_at] = previous[path_at - 1] || current[path_at - 1];
                }
            }
            Segment::Glob(glob) => {
                for path_at in 1..=path.len() {
                    current[path_at] =
                        previous[path_at - 1] && glob_matches(glob, path[path_at - 1]);
                }
            }
        }
        previous = current;
    }

    previous[path.len()] && (!directory_only || target_is_dir)
}

fn glob_matches(pattern: &[u8], text: &[u8]) -> bool {
    let mut pattern_at = 0usize;
    let mut text_at = 0usize;
    let mut star_at = None;
    let mut star_text_at = 0usize;

    while text_at < text.len() {
        if pattern.get(pattern_at) == Some(&b'*') {
            star_at = Some(pattern_at);
            pattern_at += 1;
            star_text_at = text_at;
            continue;
        }

        let atom = match pattern.get(pattern_at) {
            Some(b'\\') if pattern_at + 1 < pattern.len() => {
                Some((text[text_at] == pattern[pattern_at + 1], 2))
            }
            Some(b'?') => Some((true, 1)),
            Some(b'[') => {
                let Some(class) = class_match(&pattern[pattern_at..], text[text_at]) else {
                    return false;
                };
                Some(class)
            }
            Some(literal) => Some((text[text_at] == *literal, 1)),
            None => None,
        };
        if let Some((true, consumed)) = atom {
            pattern_at += consumed;
            text_at += 1;
            continue;
        }

        let Some(star) = star_at else {
            return false;
        };
        star_text_at += 1;
        text_at = star_text_at;
        pattern_at = star + 1;
    }

    while pattern.get(pattern_at) == Some(&b'*') {
        pattern_at += 1;
    }
    pattern_at == pattern.len()
}

/// Read the bracket expression opening `pattern` the way git's `wildmatch` does.
///
/// Returns whether `candidate` is in the set and how many pattern bytes the expression
/// spans. The whole expression is always read, whatever the candidate, so the length does
/// not depend on it. `None` is git's abort: the expression never closes, or it names a
/// `[:class:]` git does not know, and either way the pattern can match nothing.
///
/// The rules, each checked against `git check-ignore`: `!` or `^` first negates; the first
/// element can be `]`, which is then a member; a backslash makes the byte after it a
/// member; `x-y` adds the bytes from `x` to `y`, where `x` has already been added as a
/// member, so a reversed range adds nothing more; `-` first, last, or right after a range
/// or class is a member; `[:name:]` adds a class; and a `[` that does not open a
/// well-formed `[:name:]` is a member.
fn class_match(pattern: &[u8], candidate: u8) -> Option<(bool, usize)> {
    debug_assert_eq!(pattern.first(), Some(&b'['));
    let mut position = 1;
    let negated = matches!(pattern.get(position), Some(b'!' | b'^'));
    if negated {
        position += 1;
    }
    let mut matched = false;
    // The byte a following `-` would start a range from. Git clears it after a range or a
    // class, which makes a `-` there a member.
    let mut range_start: Option<u8> = None;
    // The first `]` at or after the last `[:` name start. Git rescans for it at every
    // `[:`, which a line of repeated `[:` makes quadratic; a later `[:` that starts before
    // this one reuses it, so one evaluation reads each byte a bounded number of times.
    let mut name_close: Option<usize> = None;
    let mut first = true;
    loop {
        let byte = *pattern.get(position)?;
        if byte == b']' && !first {
            return Some((matched != negated, position + 1));
        }
        first = false;
        match byte {
            b'\\' => {
                position += 1;
                let escaped = *pattern.get(position)?;
                matched |= escaped == candidate;
                range_start = Some(escaped);
            }
            b'-' if range_start.is_some()
                && pattern.get(position + 1).is_some_and(|next| *next != b']') =>
            {
                position += 1;
                let mut end = pattern[position];
                if end == b'\\' {
                    position += 1;
                    end = *pattern.get(position)?;
                }
                matched |= range_start.is_some_and(|start| (start..=end).contains(&candidate));
                range_start = None;
            }
            b'[' if pattern.get(position + 1) == Some(&b':') => {
                let name_start = position + 2;
                let close = match name_close {
                    Some(close) if close >= name_start => close,
                    _ => {
                        name_start
                            + pattern.get(name_start..)?.iter().position(|byte| *byte == b']')?
                    }
                };
                name_close = Some(close);
                if close > name_start && pattern[close - 1] == b':' {
                    matched |= posix_class(&pattern[name_start..close - 1])?(candidate);
                    range_start = None;
                    position = close;
                } else {
                    // Not `[:name:]`: the `[` is a member and reading resumes at the `:`.
                    matched |= candidate == b'[';
                    range_start = Some(b'[');
                }
            }
            literal => {
                matched |= literal == candidate;
                range_start = Some(literal);
            }
        }
        position += 1;
    }
}

/// Git's `[:name:]` classes, which are ASCII-only and use git's own ctype rather than the
/// locale's: no byte at or above 0x80 is in any class, and `space` is tab, line feed,
/// carriage return, and space, without the vertical tab and form feed C's `isspace` adds.
fn posix_class(name: &[u8]) -> Option<fn(u8) -> bool> {
    Some(match name {
        b"alnum" => |byte: u8| byte.is_ascii_alphanumeric(),
        b"alpha" => |byte: u8| byte.is_ascii_alphabetic(),
        b"blank" => |byte: u8| matches!(byte, b' ' | b'\t'),
        b"cntrl" => |byte: u8| byte.is_ascii_control(),
        b"digit" => |byte: u8| byte.is_ascii_digit(),
        b"graph" => |byte: u8| byte.is_ascii_graphic(),
        b"lower" => |byte: u8| byte.is_ascii_lowercase(),
        b"print" => |byte: u8| byte.is_ascii_graphic() || byte == b' ',
        b"punct" => |byte: u8| byte.is_ascii_punctuation(),
        b"space" => |byte: u8| matches!(byte, b'\t' | b'\n' | b'\r' | b' '),
        b"upper" => |byte: u8| byte.is_ascii_uppercase(),
        b"xdigit" => |byte: u8| byte.is_ascii_hexdigit(),
        _ => return None,
    })
}

fn trim_unescaped_spaces(mut line: &[u8]) -> &[u8] {
    while line.last() == Some(&b' ') && !is_escaped(line, line.len() - 1) {
        line = &line[..line.len() - 1];
    }
    line
}

fn is_escaped(bytes: &[u8], position: usize) -> bool {
    let mut slashes = 0usize;
    let mut at = position;
    while at > 0 && bytes[at - 1] == b'\\' {
        slashes += 1;
        at -= 1;
    }
    slashes % 2 == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct ConformanceCase {
        source: &'static [u8],
        path: &'static str,
        is_dir: bool,
        ignored: bool,
    }

    fn verdict(source: &[u8], path: &str, is_dir: bool) -> Option<bool> {
        Gitignore::parse(source).matches(Path::new(path), is_dir)
    }

    #[test]
    fn comments_escapes_negation_and_last_match_follow_gitignore_order() {
        let source = b"# comment\n*.log\n!important.log\n\\#literal\n\\!literal\n";
        assert_eq!(verdict(source, "debug.log", false), Some(true));
        assert_eq!(verdict(source, "important.log", false), Some(false));
        assert_eq!(verdict(source, "#literal", false), Some(true));
        assert_eq!(verdict(source, "!literal", false), Some(true));
        assert_eq!(verdict(source, "main.rs", false), None);
    }

    #[test]
    fn rooted_basename_directory_and_double_star_patterns_are_distinct() {
        let source = b"/build\n*.tmp\ncache/\nsrc/**/generated?.[ch]\nabc/**\n";
        assert_eq!(verdict(source, "build", true), Some(true));
        assert_eq!(verdict(source, "nested/build", true), None);
        assert_eq!(verdict(source, "nested/file.tmp", false), Some(true));
        assert_eq!(verdict(source, "cache", false), None);
        assert_eq!(verdict(source, "cache", true), Some(true));
        assert_eq!(verdict(source, "cache/deep/file", false), None);
        assert_eq!(verdict(source, "src/generated1.c", false), Some(true));
        assert_eq!(verdict(source, "src/a/b/generated2.h", false), Some(true));
        assert_eq!(verdict(source, "src/a/b/generated22.h", false), None);
        assert_eq!(verdict(source, "abc", true), None);
        assert_eq!(verdict(source, "abc/child", false), Some(true));
        assert_eq!(verdict(source, "abc/deep/child", false), Some(true));
    }

    #[test]
    fn bare_double_star_and_invalid_trailing_escape_follow_git_syntax() {
        assert_eq!(verdict(b"**\n", "anything", false), Some(true));
        assert_eq!(verdict(b"invalid\\\n", "invalid\\", false), None);
    }

    #[test]
    fn long_wildcard_runs_are_stack_safe_without_changing_escaped_stars() {
        const LONG_WILDCARD_RUN_BYTES: usize = 64 * 1024;

        let source = vec![b'*'; LONG_WILDCARD_RUN_BYTES];
        assert_eq!(Gitignore::parse(&source).matches(Path::new("anything"), false), Some(true));
        assert_eq!(verdict(b"\\**\n", "*anything", false), Some(true));
        assert_eq!(verdict(b"\\**\n", "anything", false), None);
    }

    #[test]
    fn recorded_git_conformance_cases_cover_negation_and_edge_syntax() {
        let cases = [
            ConformanceCase {
                source: b"*.txt\n!docs/\n",
                path: "docs/readme.txt",
                is_dir: false,
                ignored: true,
            },
            ConformanceCase {
                source: b"*.tmp\n/*\n!/src\n",
                path: "src/x.tmp",
                is_dir: false,
                ignored: true,
            },
            ConformanceCase { source: b"///\n", path: "anything", is_dir: false, ignored: false },
            ConformanceCase { source: b"a/**/\n", path: "a/file", is_dir: false, ignored: false },
            ConformanceCase { source: b"[]]\n", path: "]", is_dir: false, ignored: true },
        ];

        for case in cases {
            assert_eq!(
                verdict(case.source, case.path, case.is_dir).unwrap_or(false),
                case.ignored,
                "git-derived verdict for {}",
                case.path
            );
            if let Some(git_ignored) = git_verdict(case) {
                assert_eq!(git_ignored, case.ignored, "git oracle for {}", case.path);
            }
        }
    }

    fn git_verdict(case: ConformanceCase) -> Option<bool> {
        let root = tempfile::tempdir().expect("gitignore oracle root");
        std::fs::write(root.path().join(".gitignore"), case.source).expect("oracle control");
        let path = root.path().join(case.path);
        if case.is_dir {
            std::fs::create_dir_all(&path).expect("oracle directory");
        } else {
            std::fs::create_dir_all(path.parent().expect("oracle parent"))
                .expect("oracle parent directory");
            std::fs::write(&path, b"fixture").expect("oracle file");
        }
        let init = match std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(root.path())
            .status()
        {
            Ok(status) => status,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
            Err(error) => panic!("start git oracle: {error}"),
        };
        assert!(init.success(), "initialize git oracle");
        let status = std::process::Command::new("git")
            .args(["check-ignore", "--no-index", "--quiet", "--", case.path])
            .current_dir(root.path())
            .status()
            .expect("run git check-ignore oracle");
        match status.code() {
            Some(0) => Some(true),
            Some(1) => Some(false),
            code => panic!("git check-ignore exited unexpectedly: {code:?}"),
        }
    }

    #[test]
    fn repeated_double_star_segments_have_bounded_matching_work() {
        let mut source = b"**/".repeat(40);
        source.extend_from_slice(b"x\n");
        let mut path = "a/".repeat(24);
        path.push('b');

        assert_eq!(Gitignore::parse(&source).matches(Path::new(&path), false), None);
    }

    #[test]
    fn repeated_class_name_openers_have_bounded_matching_work() {
        // Each `[:` looks ahead for a closing `]`, finds `a]` rather than `:]`, and makes
        // its `[` a member. Rescanning from every one, as git does, would read this line's
        // bytes thousands of times for each byte of the name. The short forms of both
        // shapes are in `BRACKET_CASES`.
        let openers = crate::control::CONTROL_LINE_GUARD_BYTES / 2 - 4;
        let mut source = b"*[".to_vec();
        source.extend(b"[:".repeat(openers));
        source.extend_from_slice(b"a]z\n");
        let matcher = Gitignore::parse(&source);

        assert_eq!(matcher.matches(Path::new(&"z".repeat(255)), false), None);
        assert_eq!(matcher.matches(Path::new("[z"), false), Some(true));
        // No `:` here: Windows parses `a:` at the start of a path as a drive.
        assert_eq!(matcher.matches(Path::new("xaz"), false), Some(true));
        assert_eq!(matcher.matches(Path::new("bz"), false), None);
    }

    /// One `.gitignore` line with the names real git ignored and kept for it.
    ///
    /// Every verdict was recorded from `git -c core.ignorecase=false check-ignore
    /// --no-index -v -z --stdin` (git 2.50.1), with the pattern as the only line and the
    /// user's and system's git configuration out of the way. The live oracle re-asks
    /// whichever git the test host has. No candidate name starts with `:`, which
    /// `check-ignore` would read as pathspec magic. Every name is a file, so a
    /// directory-only pattern keeps them all.
    struct RecordedCase {
        pattern: &'static [u8],
        ignored: &'static [&'static [u8]],
        kept: &'static [&'static [u8]],
    }

    #[rustfmt::skip]
    const BRACKET_CASES: &[RecordedCase] = &[
            // POSIX classes, which git reads with its own ASCII-only ctype.
            RecordedCase { pattern: b"x[[:alpha:]]", ignored: &[b"xa", b"xZ"], kept: &[b"x1", b"x_", b"x[", b"x:", b"x]", b"xa]"] },
            RecordedCase { pattern: b"[[:digit:]]", ignored: &[b"5"], kept: &[b"a"] },
            RecordedCase { pattern: b"[[:alnum:]]", ignored: &[b"a", b"5"], kept: &[b"-"] },
            RecordedCase { pattern: b"[[:upper:]]", ignored: &[b"A"], kept: &[b"a"] },
            RecordedCase { pattern: b"[[:lower:]]", ignored: &[b"a"], kept: &[b"A"] },
            RecordedCase { pattern: b"[[:space:]]x", ignored: &[b" x", b"\x09x", b"\x0dx", b"\x0ax"], kept: &[b"\x0bx", b"\x0cx", b"ax"] },
            RecordedCase { pattern: b"[[:blank:]]x", ignored: &[b" x", b"\x09x"], kept: &[b"\x0ax", b"\x0bx"] },
            RecordedCase { pattern: b"[[:punct:]]", ignored: &[b"!", b"~", b"_"], kept: &[b"a"] },
            RecordedCase { pattern: b"[[:xdigit:]]", ignored: &[b"f", b"F", b"9"], kept: &[b"g"] },
            RecordedCase { pattern: b"[[:cntrl:]]", ignored: &[b"\x01", b"\x7f"], kept: &[b"a", b" "] },
            RecordedCase { pattern: b"[[:graph:]]", ignored: &[b"a", b"~"], kept: &[b" "] },
            RecordedCase { pattern: b"[[:print:]]x", ignored: &[b" x", b"ax"], kept: &[b"\x7fx"] },
            RecordedCase { pattern: b"[![:alpha:]][![:alpha:]]", ignored: &[b"\xc3\xa9", b"12"], kept: &[b"ab", b"1a"] },
            RecordedCase { pattern: b"[[:alpha:]][[:alpha:]]", ignored: &[b"ab"], kept: &[b"\xc3\xa9"] },
            RecordedCase { pattern: b"[[:bogus:]]", ignored: &[], kept: &[b"b", b"[", b"[[:bogus:]]"] },
            RecordedCase { pattern: b"[[:alpha:]0-9]", ignored: &[b"a", b"5"], kept: &[b"-"] },
            RecordedCase { pattern: b"x[[:alpha]", ignored: &[b"xa", b"x[", b"x:"], kept: &[b"x]", b"xb"] },
            RecordedCase { pattern: b"[[:alpha:]", ignored: &[], kept: &[b"a", b"[", b"[[:alpha:]"] },
            RecordedCase { pattern: b"x[!:alpha:]", ignored: &[b"xb"], kept: &[b"xa", b"x:"] },
            RecordedCase { pattern: b"[[:alpha:]-z]", ignored: &[b"-", b"b"], kept: &[b"5"] },
            RecordedCase { pattern: b"x[[:]]", ignored: &[b"x[]", b"x:]"], kept: &[b"x]"] },
            RecordedCase { pattern: b"x[[::]]", ignored: &[], kept: &[b"x:", b"x["] },
            RecordedCase { pattern: b"x[[:-z]", ignored: &[b"x[", b"xa", b"x:"], kept: &[b"x9"] },
            RecordedCase { pattern: b"x[[:alpha:][:digit:]]", ignored: &[b"xa", b"x5"], kept: &[b"x-"] },
            RecordedCase { pattern: b"x[[.a.]]", ignored: &[b"xa]", b"x.]", b"x[]"], kept: &[b"xa"] },
            RecordedCase { pattern: b"x[[=a=]]", ignored: &[b"xa]", b"x=]"], kept: &[b"xa"] },
            RecordedCase { pattern: b"x[a-**]", ignored: &[b"x*", b"xa"], kept: &[b"xb"] },
            RecordedCase { pattern: b"*[[:[:a]z", ignored: &[b"[z", b"a:z"], kept: &[b"bz"] },
            RecordedCase { pattern: b"*[[:[:]z", ignored: &[], kept: &[b"[z", b"a:z"] },
            // Escapes inside a class.
            RecordedCase { pattern: b"[a\\-z]", ignored: &[b"a", b"-", b"z"], kept: &[b"b", b"\\"] },
            RecordedCase { pattern: b"[\\]]", ignored: &[b"]"], kept: &[b"\\", b"\\]"] },
            RecordedCase { pattern: b"[\\\\]", ignored: &[b"\\"], kept: &[b"]"] },
            RecordedCase { pattern: b"[\\a-c]", ignored: &[b"b"], kept: &[b"\\", b"-"] },
            RecordedCase { pattern: b"[a-\\c]", ignored: &[b"b"], kept: &[b"\\", b"-"] },
            RecordedCase { pattern: b"[\\!a]", ignored: &[b"!", b"a"], kept: &[b"b"] },
            RecordedCase { pattern: b"[x\\", ignored: &[], kept: &[b"x", b"[x\\"] },
            RecordedCase { pattern: b"\\[a]", ignored: &[b"[a]"], kept: &[b"a"] },
            RecordedCase { pattern: b"[a-\\]]", ignored: &[b"a"], kept: &[b"]", b"b"] },
            // A `]` first in the class is a member, not the terminator.
            RecordedCase { pattern: b"[]]", ignored: &[b"]"], kept: &[] },
            RecordedCase { pattern: b"[]a]", ignored: &[b"]", b"a"], kept: &[b"b"] },
            RecordedCase { pattern: b"[!]]", ignored: &[b"a"], kept: &[b"]"] },
            RecordedCase { pattern: b"[^]]", ignored: &[b"a"], kept: &[b"]"] },
            RecordedCase { pattern: b"[]-a]", ignored: &[b"^"], kept: &[b"\\", b"b"] },
            RecordedCase { pattern: b"[]", ignored: &[], kept: &[b"]", b"[]"] },
            RecordedCase { pattern: b"[a-]]", ignored: &[b"a]", b"-]"], kept: &[b"b]"] },
            // `!` and `^` negate only in first position.
            RecordedCase { pattern: b"[!a-c]", ignored: &[b"d"], kept: &[b"a"] },
            RecordedCase { pattern: b"[^a-c]", ignored: &[b"d"], kept: &[b"a"] },
            RecordedCase { pattern: b"[a!]", ignored: &[b"!"], kept: &[b"b"] },
            RecordedCase { pattern: b"[!!]", ignored: &[b"a"], kept: &[b"!"] },
            // Ranges: the start byte is itself a member, and a reversed range adds nothing.
            RecordedCase { pattern: b"[a-c]", ignored: &[b"a", b"b", b"c"], kept: &[b"d"] },
            RecordedCase { pattern: b"[c-a]", ignored: &[b"c"], kept: &[b"a", b"b"] },
            RecordedCase { pattern: b"[a-]", ignored: &[b"-", b"a"], kept: &[b"b"] },
            RecordedCase { pattern: b"[-a]", ignored: &[b"-", b"a"], kept: &[] },
            RecordedCase { pattern: b"[a-c-e]", ignored: &[b"-", b"e"], kept: &[b"d"] },
            RecordedCase { pattern: b"[a-a]", ignored: &[b"a"], kept: &[b"b"] },
            // An unterminated class makes the whole pattern match nothing.
            RecordedCase { pattern: b"[abc", ignored: &[], kept: &[b"[abc", b"a"] },
            RecordedCase { pattern: b"foo[", ignored: &[], kept: &[b"foo[", b"foo"] },
            RecordedCase { pattern: b"[!", ignored: &[], kept: &[b"[!", b"a"] },
            RecordedCase { pattern: b"*[", ignored: &[], kept: &[b"x[", b"x"] },
            RecordedCase { pattern: b"*[0-9]", ignored: &[b"file1"], kept: &[b"file"] },
            // A `/` inside a class is a set member, not a segment separator.
            RecordedCase { pattern: b"a[b/c]", ignored: &[b"ab", b"ac"], kept: &[b"a[b/c]"] },
            RecordedCase { pattern: b"[/]", ignored: &[], kept: &[b"a", b"x"] },
            RecordedCase { pattern: b"**/[[:digit:]]", ignored: &[b"d/5", b"5"], kept: &[b"d/a"] },
    ];

    /// Escaped separators, recorded the same way. A name's `/` separates components, so
    /// `a\/b` as a name is a directory `a\` holding `b`.
    #[rustfmt::skip]
    const ESCAPED_SLASH_CASES: &[RecordedCase] = &[
            // `\/` is a separator, not a backslash ending the segment before it.
            RecordedCase { pattern: b"a\\/b", ignored: &[b"a/b"], kept: &[b"a\\/b", b"ab", b"a\\b"] },
            RecordedCase { pattern: b"x\\/y", ignored: &[b"x/y"], kept: &[] },
            RecordedCase { pattern: b"a\\/b\\/c", ignored: &[b"a/b/c"], kept: &[b"a\\/b\\/c"] },
            RecordedCase { pattern: b"a\\\\/b", ignored: &[b"a\\/b"], kept: &[b"a/b"] },
            RecordedCase { pattern: b"a[/]\\/b", ignored: &[], kept: &[b"a/b"] },
            // A leading `\/` is not an anchor, so the line matches nothing.
            RecordedCase { pattern: b"\\/foo", ignored: &[], kept: &[b"foo", b"\\/foo", b"x/foo"] },
            RecordedCase { pattern: b"/\\/foo", ignored: &[], kept: &[b"foo", b"\\/foo"] },
            // `**\/` matches one or more directories, never zero.
            RecordedCase { pattern: b"x/**\\/y", ignored: &[b"x/q/y", b"x/q/r/y"], kept: &[b"x/y", b"y"] },
            RecordedCase { pattern: b"**\\/y", ignored: &[b"q/y", b"q/r/y"], kept: &[b"y"] },
            RecordedCase { pattern: b"x\\/**\\/y", ignored: &[b"x/q/y"], kept: &[b"x/y"] },
            RecordedCase { pattern: b"x/**/**\\/y", ignored: &[b"x/q/y"], kept: &[b"x/y"] },
            RecordedCase { pattern: b"x/**\\/**/y", ignored: &[b"x/q/y", b"x/q/r/y"], kept: &[b"x/y"] },
            RecordedCase { pattern: b"a/**\\/**", ignored: &[b"a/x/y"], kept: &[b"a/x"] },
            // An escaped `/` before `**` is an ordinary separator for it.
            RecordedCase { pattern: b"x\\/**/y", ignored: &[b"x/y", b"x/q/y"], kept: &[] },
            RecordedCase { pattern: b"a\\/**", ignored: &[b"a/x", b"a/x/y"], kept: &[b"a"] },
            // A trailing `/` is stripped escaped or not, leaving `\` with nothing to escape.
            RecordedCase { pattern: b"foo\\/", ignored: &[], kept: &[b"foo", b"foo\\"] },
            RecordedCase { pattern: b"foo\\\\/", ignored: &[], kept: &[b"foo", b"foo\\"] },
            RecordedCase { pattern: b"\\/", ignored: &[], kept: &[b"x"] },
    ];

    fn verdict_bytes(source: &[u8], path: &[u8]) -> bool {
        let components: Vec<&[u8]> = path.split(|byte| *byte == b'/').collect();
        Gitignore::parse(source).matches_components(&components, false).unwrap_or(false)
    }

    fn assert_recorded_verdicts(cases: &[RecordedCase]) {
        for case in cases {
            let mut source = case.pattern.to_vec();
            source.push(b'\n');
            for (names, expected) in [(case.ignored, true), (case.kept, false)] {
                for name in names {
                    assert_eq!(
                        verdict_bytes(&source, name),
                        expected,
                        "pattern {} against {}",
                        case.pattern.escape_ascii(),
                        name.escape_ascii()
                    );
                }
            }
        }
        #[cfg(unix)]
        git_recorded_oracle(cases);
    }

    #[test]
    fn bracket_expressions_answer_as_git_check_ignore_does() {
        assert_recorded_verdicts(BRACKET_CASES);
    }

    #[test]
    fn escaped_slashes_answer_as_git_check_ignore_does() {
        assert_recorded_verdicts(ESCAPED_SLASH_CASES);
    }

    /// Re-ask the host's git for every recorded verdict, when git is installed.
    ///
    /// Unix only: the names carry `\` and control bytes, which Windows paths cannot.
    #[cfg(unix)]
    fn git_recorded_oracle(cases: &[RecordedCase]) {
        use std::io::Write as _;
        use std::process::{Command, Stdio};

        let git = |root: &Path| {
            let mut command = Command::new("git");
            command
                .current_dir(root)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .args(["-c", "core.ignorecase=false", "-c", "core.excludesFile=/dev/null"]);
            command
        };
        let root = tempfile::tempdir().expect("recorded-verdict oracle root");
        match git(root.path()).args(["init", "--quiet"]).status() {
            Ok(status) => assert!(status.success(), "initialize recorded-verdict oracle"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
            Err(error) => panic!("start git recorded-verdict oracle: {error}"),
        }
        for case in cases {
            let mut source = case.pattern.to_vec();
            source.push(b'\n');
            std::fs::write(root.path().join(".gitignore"), &source).expect("oracle control");
            let mut child = git(root.path())
                .args(["check-ignore", "--no-index", "-z", "--stdin"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("run git recorded-verdict oracle");
            let mut names = Vec::new();
            for name in case.ignored.iter().chain(case.kept) {
                names.extend_from_slice(name);
                names.push(0);
            }
            child.stdin.take().expect("oracle stdin").write_all(&names).expect("oracle names");
            let output = child.wait_with_output().expect("finish git recorded-verdict oracle");
            assert!(
                matches!(output.status.code(), Some(0 | 1)),
                "git check-ignore failed for {}: {}",
                case.pattern.escape_ascii(),
                String::from_utf8_lossy(&output.stderr)
            );
            let mut observed: Vec<&[u8]> =
                output.stdout.split(|byte| *byte == 0).filter(|name| !name.is_empty()).collect();
            observed.sort_unstable();
            let mut recorded = case.ignored.to_vec();
            recorded.sort_unstable();
            assert_eq!(observed, recorded, "git oracle for {}", case.pattern.escape_ascii());
        }
    }
}
