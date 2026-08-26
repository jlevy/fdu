//! Small fixed gitignore matcher for the inventory partition.
//!
//! This is intentionally not a general pattern API. It implements the path semantics
//! the `MetaBrowser` client already exercises—comments and escapes, negation, rooted and
//! basename patterns, directory patterns, `*`, `?`, character classes, and `**`—without
//! adding the regex/glob dependency stack to every shipped binary.

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
        let segments = body
            .split(|byte| *byte == b'/')
            .filter(|segment| !segment.is_empty())
            .map(|segment| {
                if matches_path && segment == b"**" {
                    Segment::DoubleStar
                } else {
                    Segment::Glob(normalize_glob(segment))
                }
            })
            .collect();
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
            return path.iter().enumerate().any(|(position, component)| {
                glob_matches(pattern, component)
                    && (!self.directory_only || position + 1 < path.len() || is_dir)
            });
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
    fn visit(
        pattern: &[Segment],
        path: &[&[u8]],
        pattern_at: usize,
        path_at: usize,
        directory_only: bool,
        target_is_dir: bool,
    ) -> bool {
        if pattern_at == pattern.len() {
            // A pattern that names a directory also governs its descendants. When the
            // match consumes the complete target, a trailing slash requires that target
            // itself to be a directory.
            return path_at < path.len() || (!directory_only || target_is_dir);
        }
        match &pattern[pattern_at] {
            Segment::DoubleStar if pattern_at + 1 == pattern.len() => {
                // Git's trailing `/**` means contents *inside* the named directory,
                // not the directory itself, so it must consume at least one component.
                path_at < path.len()
            }
            Segment::DoubleStar => {
                visit(pattern, path, pattern_at + 1, path_at, directory_only, target_is_dir)
                    || (path_at < path.len()
                        && visit(
                            pattern,
                            path,
                            pattern_at,
                            path_at + 1,
                            directory_only,
                            target_is_dir,
                        ))
            }
            Segment::Glob(segment) => {
                path_at < path.len()
                    && glob_matches(segment, path[path_at])
                    && visit(
                        pattern,
                        path,
                        pattern_at + 1,
                        path_at + 1,
                        directory_only,
                        target_is_dir,
                    )
            }
        }
    }

    visit(pattern, path, 0, 0, directory_only, target_is_dir)
}

fn glob_matches(pattern: &[u8], text: &[u8]) -> bool {
    fn visit(
        pattern: &[u8],
        text: &[u8],
        pattern_at: usize,
        text_at: usize,
        width: usize,
        memo: &mut [Option<bool>],
    ) -> bool {
        let slot = pattern_at * width + text_at;
        if let Some(answer) = memo[slot] {
            return answer;
        }
        let answer = if pattern_at == pattern.len() {
            text_at == text.len()
        } else {
            match pattern[pattern_at] {
                b'\\' if pattern_at + 1 < pattern.len() => {
                    text.get(text_at) == pattern.get(pattern_at + 1)
                        && visit(pattern, text, pattern_at + 2, text_at + 1, width, memo)
                }
                b'?' => {
                    text_at < text.len()
                        && visit(pattern, text, pattern_at + 1, text_at + 1, width, memo)
                }
                b'*' => {
                    visit(pattern, text, pattern_at + 1, text_at, width, memo)
                        || (text_at < text.len()
                            && visit(pattern, text, pattern_at, text_at + 1, width, memo))
                }
                b'[' => match class_match(&pattern[pattern_at..], text.get(text_at).copied()) {
                    Some((matched, consumed)) => {
                        matched
                            && visit(pattern, text, pattern_at + consumed, text_at + 1, width, memo)
                    }
                    None => {
                        text.get(text_at) == Some(&b'[')
                            && visit(pattern, text, pattern_at + 1, text_at + 1, width, memo)
                    }
                },
                literal => {
                    text.get(text_at) == Some(&literal)
                        && visit(pattern, text, pattern_at + 1, text_at + 1, width, memo)
                }
            }
        };
        memo[slot] = Some(answer);
        answer
    }

    let width = text.len() + 1;
    let mut memo = vec![None; (pattern.len() + 1) * width];
    visit(pattern, text, 0, 0, width, &mut memo)
}

fn class_match(pattern: &[u8], candidate: Option<u8>) -> Option<(bool, usize)> {
    let candidate = candidate?;
    let end = pattern.iter().enumerate().skip(1).find(|(_, byte)| **byte == b']')?.0;
    if end == 1 {
        return None;
    }
    let mut position = 1;
    let negated = matches!(pattern.get(position), Some(b'!' | b'^'));
    if negated {
        position += 1;
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
        assert_eq!(verdict(source, "cache/deep/file", false), Some(true));
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
}
