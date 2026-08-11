//! Glob patterns for selecting entries out of a built index.
//!
//! # Why this is first-party
//!
//! `globset` is the obvious dependency and a good one, but the selection axis is
//! feature-independent — Rust callers and the Python bindings get the same filters the
//! CLI does — so it would land in the core crate's always-on tree, which today holds
//! exactly one crate. `globset` brings roughly six transitive crates with it, against a
//! stated policy of keeping that list short, to match a handful of user-supplied patterns
//! at query time rather than the hundreds of ignore rules per walked entry that motivate
//! its automaton. The escape hatch is deliberate and cheap: if the pattern language grows
//! toward regexes or real gitignore semantics, or a measurement shows matching on the hot
//! path, `globset`/`ignore` goes through the dependency cool-off and this module is
//! deleted. Until then the language stays closed, and closed is what makes it testable.
//!
//! # The language
//!
//! - `*` matches any run of characters within one path component, including none.
//! - `?` matches exactly one character within a component.
//! - `**` as a whole component matches zero or more components.
//! - `[abc]`, `[a-z]`, `[!abc]` match one character from a class.
//! - `{a,b}` expands to alternatives before matching, so `*.{rs,toml}` is two patterns.
//! - `\` escapes the next character.
//!
//! A pattern containing `/` matches against an entry's path relative to the index root;
//! a pattern without one matches against the entry's file name at any depth. That is the
//! convention `fd` and gitignore use, and it is why `--include '*.rs'` finds every Rust
//! file in the tree rather than only those in the root.

use std::path::Path;

use crate::types::{Error, Result};

/// How many alternatives one pattern may expand to before it is rejected.
///
/// Nested braces multiply, so a pattern like `{a,b}{c,d}{e,f}…` can expand without bound.
/// The cap turns that into an error at parse time instead of an allocation the caller
/// never asked for.
const MAX_ALTERNATIVES: usize = 1_024;

/// A compiled glob pattern.
#[derive(Clone, Debug)]
pub struct Pattern {
    /// Brace-free alternatives; the pattern matches when any of them does.
    alternatives: Vec<Alternative>,
    /// Whether the pattern is matched against the whole relative path.
    anchored: bool,
    /// The pattern as written, for diagnostics.
    source: String,
}

/// One brace-free alternative, split into path components.
#[derive(Clone, Debug)]
struct Alternative {
    components: Vec<Component>,
}

/// One path component of a pattern.
#[derive(Clone, Debug)]
enum Component {
    /// `**`: zero or more whole components.
    AnyComponents,
    /// Tokens matched against exactly one component.
    Tokens(Vec<Token>),
}

/// One matching unit inside a component.
#[derive(Clone, Debug)]
enum Token {
    /// A literal character.
    Literal(char),
    /// `?`: exactly one character.
    AnyChar,
    /// `*`: any run of characters, including none.
    AnyRun,
    /// A character class.
    Class {
        /// Whether the class is negated with `!` or `^`.
        negated: bool,
        /// Inclusive character ranges; a single character is a range to itself.
        ranges: Vec<(char, char)>,
    },
}

impl Pattern {
    /// Compile a pattern, expanding braces.
    pub fn parse(source: &str) -> Result<Self> {
        if source.is_empty() {
            return Err(glob_error(source, "expected a pattern, as in `*.rs` or `src/**/*.rs`"));
        }

        let expanded = expand_braces(source)?;
        let anchored = source.contains('/');
        let mut alternatives = Vec::with_capacity(expanded.len());
        for candidate in expanded {
            alternatives.push(Alternative { components: parse_components(source, &candidate)? });
        }
        Ok(Self { alternatives, anchored, source: source.to_string() })
    }

    /// Whether this pattern matches an entry.
    ///
    /// `relative` is the entry's path below the index root, and `name` its final
    /// component; both are supplied because which one applies depends on the pattern.
    pub fn matches(&self, relative: &Path, name: &str) -> bool {
        if self.anchored {
            let path = relative.to_string_lossy();
            let parts: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();
            self.alternatives.iter().any(|alt| match_components(&alt.components, &parts))
        } else {
            self.alternatives.iter().any(|alt| match_components(&alt.components, &[name]))
        }
    }

    /// The pattern as it was written.
    pub fn source(&self) -> &str {
        &self.source
    }
}

/// Expand `{a,b}` alternation into brace-free patterns.
fn expand_braces(source: &str) -> Result<Vec<String>> {
    let mut pending = vec![String::new()];
    let mut chars = source.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                let escaped = chars
                    .next()
                    .ok_or_else(|| glob_error(source, "pattern ends with a trailing `\\`"))?;
                for candidate in &mut pending {
                    candidate.push('\\');
                    candidate.push(escaped);
                }
            }
            '{' => {
                let group = take_group(source, &mut chars)?;
                let branches = split_branches(&group);
                let mut grown = Vec::with_capacity(pending.len() * branches.len());
                for candidate in &pending {
                    for branch in &branches {
                        // Branches may themselves contain braces, so each is expanded in
                        // turn; the cap below is what keeps that from running away.
                        for nested in expand_braces(branch)? {
                            grown.push(format!("{candidate}{nested}"));
                        }
                    }
                }
                if grown.len() > MAX_ALTERNATIVES {
                    return Err(glob_error(
                        source,
                        "pattern expands to too many alternatives; write several patterns instead",
                    ));
                }
                pending = grown;
            }
            '}' => return Err(glob_error(source, "unmatched `}` in pattern")),
            _ => {
                for candidate in &mut pending {
                    candidate.push(ch);
                }
            }
        }
    }

    Ok(pending)
}

/// Read a brace group's contents, honoring nesting and escapes.
fn take_group(source: &str, chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<String> {
    let mut depth = 1usize;
    let mut group = String::new();
    for ch in chars.by_ref() {
        match ch {
            '{' => {
                depth += 1;
                group.push(ch);
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(group);
                }
                group.push(ch);
            }
            _ => group.push(ch),
        }
    }
    Err(glob_error(source, "unmatched `{` in pattern"))
}

/// Split a brace group on top-level commas.
fn split_branches(group: &str) -> Vec<String> {
    let mut branches = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut chars = group.chars();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                current.push(ch);
                if let Some(escaped) = chars.next() {
                    current.push(escaped);
                }
            }
            '{' => {
                depth += 1;
                current.push(ch);
            }
            '}' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if depth == 0 => branches.push(std::mem::take(&mut current)),
            _ => current.push(ch),
        }
    }
    branches.push(current);
    branches
}

/// Split a brace-free pattern into components and tokenize each.
fn parse_components(source: &str, pattern: &str) -> Result<Vec<Component>> {
    let mut components = Vec::new();
    for part in pattern.split('/') {
        if part.is_empty() {
            // A leading or doubled `/` carries no matching meaning; the path side filters
            // empty components too, so both sides agree.
            continue;
        }
        if part == "**" {
            components.push(Component::AnyComponents);
        } else {
            components.push(Component::Tokens(tokenize(source, part)?));
        }
    }
    Ok(components)
}

/// Tokenize one component.
fn tokenize(source: &str, part: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = part.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '*' => {
                // `***` and friends collapse: repeated stars inside a component add
                // nothing, and collapsing keeps the matcher from backtracking over them.
                while chars.peek() == Some(&'*') {
                    chars.next();
                }
                tokens.push(Token::AnyRun);
            }
            '?' => tokens.push(Token::AnyChar),
            '[' => tokens.push(parse_class(source, &mut chars)?),
            '\\' => {
                let escaped = chars
                    .next()
                    .ok_or_else(|| glob_error(source, "pattern ends with a trailing `\\`"))?;
                tokens.push(Token::Literal(escaped));
            }
            _ => tokens.push(Token::Literal(ch)),
        }
    }
    Ok(tokens)
}

/// Parse a `[...]` character class.
fn parse_class(source: &str, chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<Token> {
    let negated = matches!(chars.peek(), Some('!' | '^'));
    if negated {
        chars.next();
    }

    let mut ranges = Vec::new();
    let mut first = true;
    while let Some(ch) = chars.next() {
        // A `]` in the first position is a literal, per the shell convention.
        if ch == ']' && !first {
            if ranges.is_empty() {
                return Err(glob_error(source, "empty character class in pattern"));
            }
            return Ok(Token::Class { negated, ranges });
        }
        first = false;

        let start = if ch == '\\' {
            chars.next().ok_or_else(|| glob_error(source, "pattern ends with a trailing `\\`"))?
        } else {
            ch
        };

        if chars.peek() == Some(&'-') {
            chars.next();
            match chars.next() {
                // A `-` before the closing bracket is a literal `-`.
                Some(']') => {
                    ranges.push((start, start));
                    ranges.push(('-', '-'));
                    return Ok(Token::Class { negated, ranges });
                }
                Some(end) => ranges.push((start, end)),
                None => return Err(glob_error(source, "unmatched `[` in pattern")),
            }
        } else {
            ranges.push((start, start));
        }
    }
    Err(glob_error(source, "unmatched `[` in pattern"))
}

/// Match a component list against a path's components.
fn match_components(pattern: &[Component], parts: &[&str]) -> bool {
    match pattern.split_first() {
        None => parts.is_empty(),
        Some((Component::AnyComponents, rest)) => {
            // `**` consumes zero or more whole components; try each split.
            (0..=parts.len()).any(|skip| match_components(rest, &parts[skip..]))
        }
        Some((Component::Tokens(tokens), rest)) => match parts.split_first() {
            Some((part, remaining)) => {
                match_tokens(tokens, part) && match_components(rest, remaining)
            }
            None => false,
        },
    }
}

/// Match one component's tokens against one path component.
fn match_tokens(tokens: &[Token], part: &str) -> bool {
    let chars: Vec<char> = part.chars().collect();
    match_tokens_at(tokens, &chars)
}

/// Backtracking matcher over one component.
fn match_tokens_at(tokens: &[Token], text: &[char]) -> bool {
    match tokens.split_first() {
        None => text.is_empty(),
        Some((Token::AnyRun, rest)) => {
            (0..=text.len()).any(|skip| match_tokens_at(rest, &text[skip..]))
        }
        Some((token, rest)) => match text.split_first() {
            Some((ch, remaining)) => match_one(token, *ch) && match_tokens_at(rest, remaining),
            None => false,
        },
    }
}

/// Whether a single token matches a single character.
fn match_one(token: &Token, ch: char) -> bool {
    match token {
        Token::Literal(expected) => *expected == ch,
        Token::AnyChar => true,
        Token::Class { negated, ranges } => {
            let inside = ranges.iter().any(|(start, end)| *start <= ch && ch <= *end);
            inside != *negated
        }
        // Handled by the caller, which must special-case the variable-width token.
        Token::AnyRun => false,
    }
}

/// Build a pattern rejection.
fn glob_error(source: &str, hint: &str) -> Error {
    Error::InvalidValue { kind: "pattern", value: source.to_string(), hint: hint.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn matches(pattern: &str, path: &str) -> bool {
        let compiled = Pattern::parse(pattern).expect("pattern compiles");
        let relative = PathBuf::from(path);
        let name = relative
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        compiled.matches(&relative, &name)
    }

    fn rejection(pattern: &str) -> String {
        match Pattern::parse(pattern) {
            Err(Error::InvalidValue { kind: "pattern", hint, .. }) => hint,
            other => panic!("expected {pattern:?} to be rejected, got {other:?}"),
        }
    }

    #[test]
    fn bare_patterns_match_the_file_name_at_any_depth() {
        // The convention that makes `--include '*.rs'` mean what a user expects.
        assert!(matches("*.rs", "main.rs"));
        assert!(matches("*.rs", "src/deep/nested/main.rs"));
        assert!(!matches("*.rs", "src/main.toml"));
        assert!(matches("main.rs", "a/b/c/main.rs"));
    }

    #[test]
    fn patterns_with_a_separator_match_the_whole_relative_path() {
        assert!(matches("src/*.rs", "src/main.rs"));
        assert!(!matches("src/*.rs", "other/main.rs"));
        // A single `*` does not cross a component boundary.
        assert!(!matches("src/*.rs", "src/deep/main.rs"));
    }

    #[test]
    fn double_star_crosses_component_boundaries_including_none() {
        assert!(matches("src/**/*.rs", "src/main.rs"), "** matches zero components");
        assert!(matches("src/**/*.rs", "src/a/main.rs"));
        assert!(matches("src/**/*.rs", "src/a/b/c/main.rs"));
        assert!(!matches("src/**/*.rs", "other/a/main.rs"));
        assert!(matches("**/target/**", "a/b/target/c/d"));
    }

    #[test]
    fn braces_expand_to_alternatives() {
        assert!(matches("*.{rs,toml}", "main.rs"));
        assert!(matches("*.{rs,toml}", "Cargo.toml"));
        assert!(!matches("*.{rs,toml}", "notes.md"));
        // Nested braces expand too.
        assert!(matches("{src,tests}/*.{rs,md}", "tests/readme.md"));
        assert!(!matches("{src,tests}/*.{rs,md}", "docs/readme.md"));
    }

    #[test]
    fn character_classes_match_one_character() {
        assert!(matches("file[0-9].txt", "file7.txt"));
        assert!(!matches("file[0-9].txt", "filex.txt"));
        assert!(matches("file[!0-9].txt", "filex.txt"));
        assert!(!matches("file[!0-9].txt", "file7.txt"));
        assert!(matches("[abc]at", "cat"));
    }

    #[test]
    fn question_mark_matches_exactly_one_character() {
        assert!(matches("?.rs", "a.rs"));
        assert!(!matches("?.rs", "ab.rs"));
        assert!(!matches("?.rs", ".rs"));
    }

    #[test]
    fn escapes_make_metacharacters_literal() {
        assert!(matches(r"\*.rs", "*.rs"));
        assert!(!matches(r"\*.rs", "main.rs"));
        assert!(matches(r"a\?b", "a?b"));
    }

    #[test]
    fn stars_match_empty_runs() {
        assert!(matches("*", "anything"));
        assert!(matches("*.rs", ".rs"));
        assert!(matches("a*b", "ab"));
    }

    #[test]
    fn repeated_stars_inside_a_component_collapse() {
        // `***` is not a third syntax; it is `*` written twice over.
        assert!(matches("a***b", "axyzb"));
        assert!(matches("a***b", "ab"));
    }

    #[test]
    fn malformed_patterns_are_rejected_with_a_reason() {
        assert!(rejection("").contains("expected a pattern"));
        assert!(rejection("{a,b").contains("unmatched `{`"));
        assert!(rejection("a}b").contains("unmatched `}`"));
        assert!(rejection("[abc").contains("unmatched `[`"));
        assert!(rejection("[]").contains("unmatched `[`"));
        assert!(rejection(r"abc\").contains("trailing `\\`"));
    }

    #[test]
    fn runaway_brace_expansion_is_rejected_rather_than_allocated() {
        let bomb = "{a,b}".repeat(11);
        assert!(rejection(&bomb).contains("too many alternatives"));
    }

    #[test]
    fn source_is_retained_for_diagnostics() {
        assert_eq!(Pattern::parse("*.rs").expect("compiles").source(), "*.rs");
    }
}
