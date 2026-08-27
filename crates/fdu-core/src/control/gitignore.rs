//! Small fixed gitignore matcher for the inventory partition.
//!
//! This is intentionally not a general pattern API. It implements the path semantics
//! the `MetaBrowser` client already exercises—comments and escapes, negation, rooted and
//! basename patterns, directory patterns, `*`, `?`, character classes, and `**`—without
//! adding the regex/glob dependency stack to every shipped binary. Matching is byte
//! exact and case-sensitive on every platform; it does not inherit Git's repository-local
//! `core.ignorecase` setting.

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
        self.patterns
            .iter()
            .filter(|pattern| pattern.matches(&components, is_dir))
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
        if body.last() == Some(&b'\\') && is_escaped(body, body.len()) {
            return None;
        }

        let directory_only = body.last() == Some(&b'/') && !is_escaped(body, body.len() - 1);
        if directory_only {
            body = &body[..body.len() - 1];
        }
        let anchored = body.first() == Some(&b'/');
        if anchored {
            body = &body[1..];
        }
        if body.is_empty() {
            return None;
        }

        let matches_path = anchored || body.contains(&b'/');
        let mut segments = Vec::new();
        for segment in body.split(|byte| *byte == b'/').filter(|segment| !segment.is_empty()) {
            let segment = if matches_path && segment == b"**" {
                Segment::DoubleStar
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

fn normalize_glob(pattern: &[u8]) -> Vec<u8> {
    let mut normalized = Vec::with_capacity(pattern.len());
    let mut position = 0;
    let mut previous_wildcard = false;
    while position < pattern.len() {
        if pattern[position] == b'\\' && position + 1 < pattern.len() {
            normalized.extend_from_slice(&pattern[position..=position + 1]);
            position += 2;
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
            Segment::DoubleStar if position + 1 == pattern.len() => {
                // Git's trailing `/**` means contents *inside* the named directory,
                // not the directory itself, so it consumes at least one component.
                for path_at in 1..=path.len() {
                    current[path_at] = previous[path_at - 1] || current[path_at - 1];
                }
            }
            Segment::DoubleStar => {
                current[0] = previous[0];
                for path_at in 1..=path.len() {
                    current[path_at] = previous[path_at] || current[path_at - 1];
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
            Some(b'[') => class_match(&pattern[pattern_at..], Some(text[text_at]))
                .or(Some((text[text_at] == b'[', 1))),
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

fn class_match(pattern: &[u8], candidate: Option<u8>) -> Option<(bool, usize)> {
    let candidate = candidate?;
    let mut position = 1;
    let negated = matches!(pattern.get(position), Some(b'!' | b'^'));
    if negated {
        position += 1;
    }
    let terminator_at = position + usize::from(pattern.get(position) == Some(&b']'));
    let end = pattern.iter().enumerate().skip(terminator_at).find(|(_, byte)| **byte == b']')?.0;
    if end == position {
        return None;
    }
    let mut matched = false;
    while position < end {
        let start = pattern[position];
        if position + 2 < end && pattern[position + 1] == b'-' {
            matched |= (start..=pattern[position + 2]).contains(&candidate);
            position += 3;
        } else {
            matched |= start == candidate;
            position += 1;
        }
    }
    Some((matched != negated, end + 1))
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
}
