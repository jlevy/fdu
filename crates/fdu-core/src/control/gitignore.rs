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

/// Path components matched without a heap allocation. Deeper paths still match exactly;
/// they collect into a vector first.
const INLINE_COMPONENTS: usize = 32;

/// Positions a `**` pattern tracks on the stack: a path of up to this many components,
/// plus the empty prefix. Deeper paths use heap rows with the same arithmetic.
const INLINE_POSITIONS: usize = 64;

/// Call `each` with the normal components of `path`, then `last` if given, without a heap
/// allocation for a path of up to [`INLINE_COMPONENTS`] of them.
pub(super) fn with_components<R>(
    path: &Path,
    last: Option<&[u8]>,
    each: impl FnOnce(&[&[u8]]) -> R,
) -> R {
    let normal = path.components().filter_map(|component| match component {
        Component::Normal(value) => Some(value.as_encoded_bytes()),
        Component::CurDir | Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
            None
        }
    });
    with_collected(normal.chain(last), each)
}

/// Call `each` with the components `components` yields, without a heap allocation for up
/// to [`INLINE_COMPONENTS`] of them.
pub(super) fn with_collected<'a, R>(
    components: impl IntoIterator<Item = &'a [u8]>,
    each: impl FnOnce(&[&'a [u8]]) -> R,
) -> R {
    let mut inline: [&[u8]; INLINE_COMPONENTS] = [&[]; INLINE_COMPONENTS];
    let mut spilled: Vec<&[u8]> = Vec::new();
    let mut count = 0usize;
    for bytes in components {
        if count < INLINE_COMPONENTS {
            inline[count] = bytes;
        } else {
            if spilled.is_empty() {
                spilled.extend_from_slice(&inline);
            }
            spilled.push(bytes);
        }
        count += 1;
    }
    each(if count <= INLINE_COMPONENTS { &inline[..count] } else { &spilled[..] })
}

/// One control file's rules, parsed once and indexed for matching.
///
/// **How an entry is matched (H171).** Git's answer for one file is its last matching
/// line, and a rule's index in [`Self::patterns`] is its line order, so the answer is the
/// rule with the highest matching index. Negation and directory-only rules need nothing
/// more: the rule at that index says whether the entry is ignored, and a directory-only
/// rule simply does not match a file. So rather than testing every rule against every
/// entry, as git does, [`RuleIndex`] sorts the rules once, when the file is parsed, into
/// places an entry can ask cheaply for the highest index that could match it:
///
/// - a rule that is one literal name, such as `Makefile` or `vmlinux`, is found by
///   looking the entry's name up;
/// - a rule that is `*` and a literal tail, such as `*.o` or `*~` (git's `ENDSWITH`), is
///   found by the entry's extension, or checked with an `ends_with` when its tail has no
///   `.`, and `*` alone matches every name;
/// - an anchored rule of `n` segments, such as `/include/config/`, can match only a path
///   `n` components below its file, so only the rules of the entry's own depth are tried;
/// - every other rule is tried in descending index order, and only while its index is
///   above the best found so far, after checks git's matcher makes too: its length, its
///   literal prefix and suffix, and a literal run it must contain.
///
/// Git tests every rule, and so did fdu, although on `linux-v6.12` about 110 rules
/// govern each entry and 99% of entries match none of them. A rule `**/glob` is matched
/// as the basename rule `glob`, which git's semantics make it, so it is indexed as one.
///
/// The rules themselves are kept whole and still decide every answer: the index holds
/// only rule indices, hashes, and short lengths, never a copy of a rule's bytes. Each
/// indexed rule adds one 16-byte record, and a rule's segments are now allocated to their
/// exact size rather than a growable vector's, so the charge [`super::content_cost`]
/// makes per line still covers the matcher it retains, even for a file of the shortest
/// possible rules (`a_source_charge_covers_its_indexed_matcher`).
#[derive(Clone, Debug, Default)]
pub(super) struct Gitignore {
    /// Every accepted rule in file order: a rule's index is its precedence.
    patterns: Box<[Pattern]>,
    index: RuleIndex,
}

#[derive(Clone, Debug)]
struct Pattern {
    ignored: bool,
    directory_only: bool,
    shape: Shape,
    segments: Box<[Segment]>,
}

/// What a pattern is matched against, decided once when it is parsed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    /// No `/`: one glob against the entry's own name.
    Basename,
    /// Anchored or holding a `/` but no `**`: only a path of the pattern's length.
    Fixed,
    /// Holding a `**`, which spans any number of components.
    Spanning,
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
        let patterns: Box<[Pattern]> =
            source.split(|byte| *byte == b'\n').filter_map(Pattern::parse).collect();
        let index = RuleIndex::build(&patterns);
        Self { patterns, index }
    }

    /// Accepted executable rules; comments, blank lines, and rejected patterns do not count.
    pub(super) fn rule_count(&self) -> u64 {
        u64::try_from(self.patterns.len()).unwrap_or(u64::MAX)
    }

    /// Last matching line wins. `Some(false)` is an explicit negation; `None` means this
    /// control file expressed no opinion.
    ///
    /// Every classified entry asks this once per control file on its path, so it must not
    /// allocate: the components of an ordinary path fit a stack buffer, and only a path
    /// deeper than it spills to the heap (H162).
    pub(super) fn matches(&self, relative: &Path, is_dir: bool) -> Option<bool> {
        with_components(relative, None, |components| {
            let (name, directory) = components.split_last()?;
            self.decide(directory, &Name::new(name), is_dir)
        })
    }

    /// The answer for the entry `name` in `directory`, a path relative to this file's own
    /// directory: the rule with the highest matching index, found through [`RuleIndex`].
    ///
    /// The answer is [`Self::matches_in_order`]'s for the same path, which the property
    /// test `indexed_matching_answers_as_testing_every_rule_in_order` holds it to.
    pub(super) fn decide(
        &self,
        directory: &[&[u8]],
        name: &Name<'_>,
        is_dir: bool,
    ) -> Option<bool> {
        let index = &self.index;
        if index.in_order {
            return with_joined(directory, name.bytes, |path| self.matches_in_order(path, is_dir));
        }
        // Ranks are one more than a rule's index, so 0 is "no rule matched".
        let mut best = index.everything.rank(is_dir);
        if let Some(keyed) = with_hash(&index.names, name.hash)
            .iter()
            .find(|keyed| literal_eq(self.name_glob(keyed.pattern), name.bytes))
        {
            best = best.max(keyed.ranks.rank(is_dir));
        }
        if let Some(extension) = name.extension {
            for keyed in with_hash(&index.extensions, extension) {
                if ends_with_literal(name.bytes, tail(self.name_glob(keyed.pattern))) {
                    best = best.max(keyed.ranks.rank(is_dir));
                }
            }
        }
        for keyed in &index.tails {
            // A tail with no `.` is keyed by its last byte instead of a hash.
            if name.bytes.last().is_some_and(|last| u32::from(*last) == keyed.hash)
                && ends_with_literal(name.bytes, tail(self.name_glob(keyed.pattern)))
            {
                best = best.max(keyed.ranks.rank(is_dir));
            }
        }

        // Every remaining list is in descending index order, so each stops at its first
        // match, or at the first rule whose index cannot beat the best match already found.
        for candidate in &index.basenames {
            if candidate.index < best {
                break;
            }
            let glob = self.name_glob(candidate.index);
            if candidate.checks.admit(glob, name.bytes, is_dir) && glob_matches(glob, name.bytes) {
                best = candidate.index + 1;
                break;
            }
        }
        for candidate in index.anchored_at(&self.patterns, directory.len() + 1) {
            if candidate.index < best {
                break;
            }
            let pattern = self.pattern(candidate.index);
            if candidate.checks.admit(self.name_glob(candidate.index), name.bytes, is_dir)
                && fixed_matches(&pattern.segments, directory, name.bytes)
            {
                best = candidate.index + 1;
                break;
            }
        }
        for candidate in &index.spanning {
            if candidate.index < best {
                break;
            }
            let pattern = self.pattern(candidate.index);
            if candidate.checks.admit(self.name_glob(candidate.index), name.bytes, is_dir)
                && with_joined(directory, name.bytes, |path| pattern.matches(path, is_dir))
            {
                best = candidate.index + 1;
                break;
            }
        }
        best.checked_sub(1).map(|index| self.pattern(index).ignored)
    }

    /// [`Self::decide`] for a path already split, asserting that the indexed answer is the
    /// one testing every rule in order gives.
    #[cfg(test)]
    fn matches_components(&self, components: &[&[u8]], is_dir: bool) -> Option<bool> {
        let indexed = components
            .split_last()
            .and_then(|(name, directory)| self.decide(directory, &Name::new(name), is_dir));
        assert_eq!(
            indexed,
            self.matches_in_order(components, is_dir),
            "indexed and in-order answers"
        );
        indexed
    }

    /// Every rule tested in reverse order, the last match deciding: the matcher before
    /// [`RuleIndex`], and still the reference its answers are tested against.
    fn matches_in_order(&self, components: &[&[u8]], is_dir: bool) -> Option<bool> {
        self.patterns
            .iter()
            .filter(|pattern| pattern.matches(components, is_dir))
            .map(|pattern| pattern.ignored)
            .next_back()
    }

    fn pattern(&self, index: u32) -> &Pattern {
        &self.patterns[slot(index)]
    }

    fn name_glob(&self, index: u32) -> &[u8] {
        self.pattern(index).name_glob()
    }
}

/// An entry's name, with the hashes every control file on its path looks it up by, so it
/// is hashed once however many files govern it.
pub(super) struct Name<'a> {
    bytes: &'a [u8],
    /// FNV-1a of the whole name, for literal names.
    hash: u32,
    /// FNV-1a of the bytes after the name's last `.`, when it has one, for literal tails.
    extension: Option<u32>,
}

impl<'a> Name<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        let extension = bytes
            .iter()
            .rposition(|byte| *byte == b'.')
            .map(|dot| fnv1a(bytes[dot + 1..].iter().copied()));
        Self { bytes, hash: fnv1a(bytes.iter().copied()), extension }
    }
}

/// 32-bit FNV-1a: a few instructions a byte for the short keys it hashes here, without a
/// dependency. Only which rules an entry is compared with depends on it, never an answer:
/// a colliding key is compared byte for byte and rejected.
fn fnv1a(bytes: impl IntoIterator<Item = u8>) -> u32 {
    bytes.into_iter().fold(FNV1A_OFFSET_BASIS, fnv1a_step)
}

const FNV1A_OFFSET_BASIS: u32 = 0x811c_9dc5;

fn fnv1a_step(hash: u32, byte: u8) -> u32 {
    (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
}

/// Where one file's rules are looked up from, built once when the file is parsed.
///
/// Every rule is in exactly one place. Keys are compared against the rule's own glob, so
/// nothing here copies a rule's bytes: each keyed or listed rule costs one 16-byte record,
/// less than the matcher shell [`super::content_cost`] already charges every line.
#[derive(Clone, Debug, Default)]
struct RuleIndex {
    /// The rules could not be indexed, because more of them than a `u32` counts were
    /// accepted, so they are matched in order instead.
    in_order: bool,
    /// Literal basename rules, one record per distinct name, in hash order.
    names: Box<[Keyed]>,
    /// `*tail` basename rules whose literal tail holds a `.`, one record per distinct
    /// tail, in order of the hash of the tail's text after its last `.`: a name ending in
    /// the tail has its own last `.` in the same place, so it is found by its extension.
    extensions: Box<[Keyed]>,
    /// `*tail` basename rules whose literal tail holds no `.`, one per distinct tail,
    /// keyed by the tail's last byte.
    tails: Box<[Keyed]>,
    /// The `*` basename rule, which matches every name.
    everything: Ranks,
    /// Every other basename rule, in descending index order.
    basenames: Box<[Candidate]>,
    /// Rules anchored to the file's directory without a `**`, grouped by their number of
    /// segments, the only depth they can match at, and in descending index order within
    /// each group.
    anchored: Box<[Candidate]>,
    /// Rules holding a `**` that are not a basename rule, in descending index order.
    spanning: Box<[Candidate]>,
}

/// The highest ranks, one more than the index, of the rules sharing one key: those that
/// match any entry, and those that match only a directory.
#[derive(Clone, Copy, Debug, Default)]
struct Ranks {
    any: u32,
    directories: u32,
}

impl Ranks {
    fn add(&mut self, rank: u32, directory_only: bool) {
        let slot = if directory_only { &mut self.directories } else { &mut self.any };
        *slot = (*slot).max(rank);
    }

    fn rank(self, is_dir: bool) -> u32 {
        if is_dir { self.any.max(self.directories) } else { self.any }
    }
}

/// A distinct literal key and the rules holding it.
#[derive(Clone, Copy, Debug)]
struct Keyed {
    /// The key's hash, or for a tail without a `.`, its last byte.
    hash: u32,
    /// One rule holding the key, whose glob the key is compared with.
    pattern: u32,
    ranks: Ranks,
}

/// A rule that must be tested, with the checks that rule it out before its glob runs.
#[derive(Clone, Copy, Debug)]
struct Candidate {
    index: u32,
    checks: Checks,
}

/// What the entry's name must satisfy for one rule's name glob to match it, derived from
/// the glob when it is parsed, as git derives its literal-prefix length.
///
/// Each is a necessary condition, so failing one skips the glob without changing any
/// answer. Lengths are saturated to 16 bits, which only weakens a check: a shorter prefix,
/// suffix, run, or minimum is still necessary, and the exact-length flag is dropped when
/// the minimum does not fit.
#[derive(Clone, Copy, Debug, Default)]
struct Checks {
    /// Bytes the glob consumes at least: one per literal, `?`, or bracket expression.
    min_len: u16,
    /// Literal bytes that open the glob.
    prefix: u16,
    /// Literal bytes that close the glob.
    suffix: u16,
    /// Where a run of literal bytes inside the glob starts, and its length.
    required_at: u16,
    required_len: u16,
    flags: u8,
}

impl Checks {
    /// The glob has no `*`, so the name is exactly `min_len` bytes.
    const EXACT: u8 = 1;
    /// The rule matches only a directory.
    const DIRECTORY_ONLY: u8 = 2;

    fn for_pattern(pattern: &Pattern) -> Self {
        let flags = if pattern.directory_only { Self::DIRECTORY_ONLY } else { 0 };
        match pattern.segments.last() {
            Some(Segment::Glob(glob)) => Self::of(glob, flags),
            // A rule ending in `**` constrains no name.
            _ => Self { flags, ..Self::default() },
        }
    }

    fn of(glob: &[u8], flags: u8) -> Self {
        let saturate = |value: usize| u16::try_from(value).unwrap_or(u16::MAX);
        let mut checks = Self { flags, ..Self::default() };
        let mut min_len = 0usize;
        let mut star = false;
        // The run of plain literal bytes being read, which an escaped byte also ends, so
        // that every run is a slice of the glob itself; and the longest run that closed
        // before the end of the glob without opening it.
        let mut run_start = None;
        let mut inner: Option<(usize, usize)> = None;
        let mut position = 0;
        while position < glob.len() {
            let (plain, consumed) = match glob[position] {
                b'*' => {
                    star = true;
                    (false, 1)
                }
                b'\\' if position + 1 < glob.len() => (false, 2),
                b'?' => (false, 1),
                b'[' => (false, class_match(&glob[position..], 0).map_or(1, |(_, length)| length)),
                _ => (true, 1),
            };
            min_len += usize::from(glob[position] != b'*');
            match (plain, run_start) {
                (true, None) => run_start = Some(position),
                (false, Some(start)) => {
                    if start == 0 {
                        checks.prefix = saturate(position);
                    } else if inner.is_none_or(|(_, length)| position - start > length) {
                        inner = Some((start, position - start));
                    }
                    run_start = None;
                }
                _ => {}
            }
            position += consumed;
        }
        if let Some(start) = run_start {
            if start == 0 {
                checks.prefix = saturate(glob.len());
            }
            checks.suffix = saturate(glob.len() - start);
        }
        checks.min_len = saturate(min_len);
        if !star && u16::try_from(min_len).is_ok() {
            checks.flags |= Self::EXACT;
        }
        // Every match contains each run somewhere; the prefix and suffix already pin
        // theirs, so only a run inside the glob adds to them, and only one of two or more
        // bytes rules out more than a single comparison would.
        if let Some((start, length)) = inner.filter(|(_, length)| *length >= 2) {
            if let Ok(at) = u16::try_from(start) {
                checks.required_at = at;
                checks.required_len = saturate(length);
            }
        }
        checks
    }

    /// Whether `name` passes every check, and the rule could therefore match it.
    fn admit(&self, glob: &[u8], name: &[u8], is_dir: bool) -> bool {
        if self.flags & Self::DIRECTORY_ONLY != 0 && !is_dir {
            return false;
        }
        let min_len = usize::from(self.min_len);
        if name.len() < min_len || (self.flags & Self::EXACT != 0 && name.len() != min_len) {
            return false;
        }
        let required_at = usize::from(self.required_at);
        name.starts_with(&glob[..usize::from(self.prefix)])
            && name.ends_with(&glob[glob.len() - usize::from(self.suffix)..])
            && contains(name, &glob[required_at..required_at + usize::from(self.required_len)])
    }
}

impl RuleIndex {
    fn build(patterns: &[Pattern]) -> Self {
        // A rank is one more than an index, so every rank must fit too.
        if u32::try_from(patterns.len()).is_err() {
            return Self { in_order: true, ..Self::default() };
        }
        let mut names = Vec::new();
        let mut extensions = Vec::new();
        let mut tails = Vec::new();
        let mut everything = Ranks::default();
        let mut basenames = Vec::new();
        let mut anchored = Vec::new();
        let mut spanning = Vec::new();
        for (index, pattern) in (0u32..).zip(patterns) {
            let rank = index + 1;
            let candidate = Candidate { index, checks: Checks::for_pattern(pattern) };
            let Some(glob) = pattern.basename_glob() else {
                if pattern.shape == Shape::Fixed {
                    anchored.push((pattern.segments.len(), candidate));
                } else {
                    spanning.push(candidate);
                }
                continue;
            };
            if is_literal(glob) {
                names.push((fnv1a(unescaped(glob)), index, rank, pattern.directory_only));
            } else if glob == b"*" {
                everything.add(rank, pattern.directory_only);
            } else if let Some(tail) = glob.strip_prefix(b"*").filter(|tail| is_literal(tail)) {
                let mut extension = None;
                for byte in unescaped(tail) {
                    extension = if byte == b'.' {
                        Some(FNV1A_OFFSET_BASIS)
                    } else {
                        extension.map(|hash| fnv1a_step(hash, byte))
                    };
                }
                if let Some(hash) = extension {
                    extensions.push((hash, index, rank, pattern.directory_only));
                } else {
                    let last = unescaped(tail).last().expect("a tail is not empty");
                    tails.push((u32::from(last), index, rank, pattern.directory_only));
                }
            } else {
                basenames.push(candidate);
            }
        }
        basenames.reverse();
        spanning.reverse();
        // Ascending segment count, then descending index within one count.
        anchored.sort_by(|(left_count, left), (right_count, right)| {
            left_count.cmp(right_count).then(right.index.cmp(&left.index))
        });
        let name_of = |index: u32| patterns[slot(index)].name_glob();
        let tail_of = |index: u32| tail(name_of(index));
        Self {
            in_order: false,
            names: merge_keys(names, name_of),
            extensions: merge_keys(extensions, tail_of),
            tails: merge_keys(tails, tail_of),
            everything,
            basenames: basenames.into_boxed_slice(),
            anchored: anchored.into_iter().map(|(_, candidate)| candidate).collect(),
            spanning: spanning.into_boxed_slice(),
        }
    }

    /// The anchored rules that can match an entry `depth` components below the file.
    fn anchored_at<'rules>(
        &'rules self,
        patterns: &[Pattern],
        depth: usize,
    ) -> &'rules [Candidate] {
        let count = |candidate: &Candidate| patterns[slot(candidate.index)].segments.len();
        // Most entries are deeper than every anchored rule; they skip both searches.
        if self.anchored.last().is_none_or(|deepest| count(deepest) < depth) {
            return &[];
        }
        let start = self.anchored.partition_point(|candidate| count(candidate) < depth);
        let rest = &self.anchored[start..];
        &rest[..rest.partition_point(|candidate| count(candidate) == depth)]
    }
}

/// One record per distinct key, in hash order: `keys` holds `(hash, index, rank,
/// directory_only)` for each rule, and `key` reads the literal glob of the rule at an
/// index. Rules with the same key fold their ranks into one record; distinct keys that
/// share a hash keep a record each.
fn merge_keys<'rules>(
    mut keys: Vec<(u32, u32, u32, bool)>,
    key: impl Fn(u32) -> &'rules [u8],
) -> Box<[Keyed]> {
    // Sorting by the key itself puts every rule of one key together, so a file of many
    // keys that share a hash still merges in `n log n`.
    keys.sort_by(|(left_hash, left, ..), (right_hash, right, ..)| {
        left_hash.cmp(right_hash).then_with(|| unescaped(key(*left)).cmp(unescaped(key(*right))))
    });
    let mut merged: Vec<Keyed> = Vec::with_capacity(keys.len());
    for (hash, index, rank, directory_only) in keys {
        match merged.last_mut() {
            Some(last) if last.hash == hash && literal_key_eq(key(last.pattern), key(index)) => {
                last.ranks.add(rank, directory_only);
            }
            _ => {
                let mut ranks = Ranks::default();
                ranks.add(rank, directory_only);
                merged.push(Keyed { hash, pattern: index, ranks });
            }
        }
    }
    merged.into_boxed_slice()
}

/// A rule index as a position in the rules. A `u32` always fits: fdu builds only for
/// targets whose `usize` is at least that wide.
fn slot(index: u32) -> usize {
    usize::try_from(index).expect("a rule index fits in usize")
}

/// The records of `sorted` whose hash is `hash`.
fn with_hash(sorted: &[Keyed], hash: u32) -> &[Keyed] {
    let start = sorted.partition_point(|keyed| keyed.hash < hash);
    let rest = &sorted[start..];
    let end = rest.iter().position(|keyed| keyed.hash != hash).unwrap_or(rest.len());
    &rest[..end]
}

/// Whether `glob` holds no `*`, `?`, or bracket expression, so it matches exactly the
/// bytes [`unescaped`] reads from it.
fn is_literal(glob: &[u8]) -> bool {
    let mut position = 0;
    while position < glob.len() {
        match glob[position] {
            b'\\' => position += 2,
            b'*' | b'?' | b'[' => return false,
            _ => position += 1,
        }
    }
    true
}

/// The bytes a literal glob matches: each `\x` is the byte `x`, as in [`glob_matches`],
/// which also reads a backslash with nothing after it as itself.
fn unescaped(glob: &[u8]) -> impl Iterator<Item = u8> + '_ {
    let mut position = 0;
    std::iter::from_fn(move || {
        let byte = *glob.get(position)?;
        if byte == b'\\' && position + 1 < glob.len() {
            position += 2;
            Some(glob[position - 1])
        } else {
            position += 1;
            Some(byte)
        }
    })
}

/// Whether the literal `glob` matches exactly `text`.
fn literal_eq(glob: &[u8], text: &[u8]) -> bool {
    unescaped(glob).eq(text.iter().copied())
}

/// Whether two literal globs match the same name.
fn literal_key_eq(left: &[u8], right: &[u8]) -> bool {
    unescaped(left).eq(unescaped(right))
}

/// Whether `name` ends with the bytes the literal `tail` matches.
fn ends_with_literal(name: &[u8], tail: &[u8]) -> bool {
    if !tail.contains(&b'\\') {
        return name.ends_with(tail);
    }
    let length = unescaped(tail).count();
    name.len() >= length && literal_eq(tail, &name[name.len() - length..])
}

/// The literal after the leading `*` of an ends-with rule's glob.
fn tail(glob: &[u8]) -> &[u8] {
    glob.strip_prefix(b"*").unwrap_or(glob)
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    needle.is_empty() || haystack.windows(needle.len()).any(|window| window == needle)
}

/// Call `each` with `directory` and then `name` as one path, without a heap allocation for
/// a path of up to [`INLINE_COMPONENTS`] components.
fn with_joined<R>(directory: &[&[u8]], name: &[u8], each: impl FnOnce(&[&[u8]]) -> R) -> R {
    let count = directory.len() + 1;
    if count <= INLINE_COMPONENTS {
        let mut inline: [&[u8]; INLINE_COMPONENTS] = [&[]; INLINE_COMPONENTS];
        inline[..directory.len()].copy_from_slice(directory);
        inline[directory.len()] = name;
        each(&inline[..count])
    } else {
        let mut joined = Vec::with_capacity(count);
        joined.extend_from_slice(directory);
        joined.push(name);
        each(&joined)
    }
}

/// Whether a `Fixed` rule's segments match `directory` and then `name`, one each.
fn fixed_matches(segments: &[Segment], directory: &[&[u8]], name: &[u8]) -> bool {
    let Some((Segment::Glob(last), leading)) = segments.split_last() else {
        return false;
    };
    leading.len() == directory.len()
        && glob_matches(last, name)
        && leading.iter().zip(directory).all(|(segment, component)| match segment {
            Segment::Glob(glob) => glob_matches(glob, component),
            Segment::DoubleStar | Segment::DoubleStarOneOrMore => false,
        })
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
        for (segment, before_escaped_separator) in split_segments(body)? {
            // After the single leading anchor and trailing directory marker have been
            // removed, an empty component is a repeated separator. Normalized paths can
            // never contain one, so git makes the whole pattern match nothing.
            if segment.is_empty() {
                return None;
            }
            let segment =
                if matches_path && segment.len() >= 2 && segment.iter().all(|byte| *byte == b'*') {
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
        let shape = if !matches_path {
            Shape::Basename
        } else if segments.iter().all(|segment| matches!(segment, Segment::Glob(_))) {
            Shape::Fixed
        } else {
            Shape::Spanning
        };
        Some(Self { ignored, directory_only, shape, segments: segments.into_boxed_slice() })
    }

    /// The glob the entry's own name must match, or nothing when the rule ends in `**` and
    /// constrains no name.
    fn name_glob(&self) -> &[u8] {
        match self.segments.last() {
            Some(Segment::Glob(glob)) => glob,
            _ => &[],
        }
    }

    /// The glob this rule tests against the entry's name alone, wherever the entry is,
    /// when that is all it tests.
    ///
    /// That is a basename rule, and also `**/glob`: a leading `**/` matches zero or more
    /// directories, so the rule matches any path whose last component matches `glob`,
    /// exactly as the basename rule `glob` does, directory-only or not (t3070 has `foo`
    /// against `**/foo`). `**\/glob` needs one directory at least, so it is not one.
    fn basename_glob(&self) -> Option<&[u8]> {
        match (self.shape, &*self.segments) {
            (Shape::Basename, [Segment::Glob(glob)])
            | (Shape::Spanning, [Segment::DoubleStar, Segment::Glob(glob)]) => Some(glob),
            _ => None,
        }
    }

    fn matches(&self, path: &[&[u8]], is_dir: bool) -> bool {
        if path.is_empty() {
            return false;
        }
        if self.shape == Shape::Basename {
            let Some(Segment::Glob(pattern)) = self.segments.first() else {
                return false;
            };
            return path.last().is_some_and(|component| glob_matches(pattern, component))
                && (!self.directory_only || is_dir);
        }
        segment_path_matches(
            &self.segments,
            self.shape == Shape::Fixed,
            path,
            self.directory_only,
            is_dir,
        )
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
    glob_only: bool,
    path: &[&[u8]],
    directory_only: bool,
    target_is_dir: bool,
) -> bool {
    if directory_only && !target_is_dir {
        return false;
    }
    // Without a `**`, each segment consumes exactly one component, so only a path of the
    // pattern's length can match, and it matches segment for segment. This is the shape
    // of every anchored rule such as `/vmlinux`, which most entries fail on length alone.
    if glob_only {
        return pattern.len() == path.len()
            && pattern.iter().zip(path).all(|(segment, component)| match segment {
                Segment::Glob(glob) => glob_matches(glob, component),
                Segment::DoubleStar | Segment::DoubleStarOneOrMore => false,
            });
    }
    let positions = path.len() + 1;
    if positions <= INLINE_POSITIONS {
        let mut previous = [false; INLINE_POSITIONS];
        let mut current = [false; INLINE_POSITIONS];
        double_star_matches(pattern, path, &mut previous[..positions], &mut current[..positions])
    } else {
        double_star_matches(pattern, path, &mut vec![false; positions], &mut vec![false; positions])
    }
}

/// The general matcher for a pattern holding `**`: `previous[i]` says the segments so
/// far can consume exactly the first `i` path components. The rows are supplied by the
/// caller so the common depth needs no allocation.
fn double_star_matches<'rows>(
    pattern: &[Segment],
    path: &[&[u8]],
    mut previous: &'rows mut [bool],
    mut current: &'rows mut [bool],
) -> bool {
    previous.fill(false);
    previous[0] = true;

    for (position, segment) in pattern.iter().enumerate() {
        current.fill(false);
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
        std::mem::swap(&mut previous, &mut current);
    }

    previous[path.len()]
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

    /// The matcher before H162: heap rows for every pattern and no length shortcut. Kept
    /// as the oracle the allocation-free paths must agree with.
    fn reference_segment_path_matches(
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

    #[test]
    fn allocation_free_matching_agrees_with_the_heap_rows_at_every_depth() {
        // Depths straddle both stack buffers: 32 inline components, 63 inline positions.
        let sources: &[&[u8]] = &[
            b"/a",
            b"/a/b",
            b"a/b",
            b"/a/*",
            b"*/b",
            b"a/**",
            b"**/b",
            b"a/**/b",
            b"/a/**/b/",
            b"**/a/**",
            b"a/**\\/b",
            b"/[ab]/?",
            b"a/b/",
            b"**/**/b",
            b"**\\/b",
            b"a/**\\/**\\/b",
            b"/**",
            b"**/b/",
            b"!a/**/b",
            b"/\xc3\xa9/*",
        ];
        for &source in sources {
            let pattern = Pattern::parse(source).expect("fixture pattern parses");
            for depth in [1usize, 2, 3, 31, 32, 33, 62, 63, 64, 70] {
                for shape in 0..5 {
                    let names: Vec<Vec<u8>> = (0..depth)
                        // All `a`; `a` ending in `b`; `a` then all `b`; a multi-byte name then
                        // `a`; distinct names.
                        .map(|at| match (shape, at) {
                            (1, at) if at + 1 == depth => b"b".to_vec(),
                            (2, at) if at > 0 => b"b".to_vec(),
                            (3, 0) => "\u{e9}".as_bytes().to_vec(),
                            (0..=3, _) => b"a".to_vec(),
                            _ => format!("n{at}").into_bytes(),
                        })
                        .collect();
                    let path: Vec<&[u8]> = names.iter().map(Vec::as_slice).collect();
                    let joined = names.join(&b'/');
                    let text = std::str::from_utf8(&joined).expect("fixture names are UTF-8");
                    for is_dir in [false, true] {
                        let expected = if pattern.shape == Shape::Basename {
                            pattern.matches(&path, is_dir)
                        } else {
                            reference_segment_path_matches(
                                &pattern.segments,
                                &path,
                                pattern.directory_only,
                                is_dir,
                            )
                        };
                        assert_eq!(
                            pattern.matches(&path, is_dir),
                            expected,
                            "{} against a depth-{depth} path, shape {shape}, dir {is_dir}",
                            String::from_utf8_lossy(source)
                        );
                        let mut line = source.to_vec();
                        line.push(b'\n');
                        assert_eq!(
                            Gitignore::parse(&line).matches(Path::new(text), is_dir),
                            expected.then_some(pattern.ignored),
                            "{} through the public matcher at depth {depth}",
                            String::from_utf8_lossy(source)
                        );
                    }
                }
            }
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
        let openers = crate::control::DEFAULT_CONTROL_LINE_LIMIT / 2 - 4;
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

    /// Empty path components and complete multi-star components, recorded from git.
    #[rustfmt::skip]
    const PATH_SEGMENT_CASES: &[RecordedCase] = &[
            RecordedCase { pattern: b"a//b", ignored: &[], kept: &[b"a/b", b"a/q/b"] },
            RecordedCase { pattern: b"//foo", ignored: &[], kept: &[b"foo", b"x/foo"] },
            RecordedCase { pattern: b"a\\//b", ignored: &[], kept: &[b"a/b", b"a/q/b"] },
            RecordedCase { pattern: b"a/**//", ignored: &[], kept: &[b"a/x", b"a/x/y"] },
            RecordedCase { pattern: b"a/***/b", ignored: &[b"a/b", b"a/q/b", b"a/q/r/b"], kept: &[b"b", b"q/a/b"] },
            RecordedCase { pattern: b"***/x", ignored: &[b"x", b"q/x", b"q/r/x"], kept: &[b"y"] },
            RecordedCase { pattern: b"x/***", ignored: &[b"x/a", b"x/a/b"], kept: &[b"x", b"q/x/a"] },
            RecordedCase { pattern: b"a/****\\/b", ignored: &[b"a/q/b", b"a/q/r/b"], kept: &[b"a/b"] },
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

    #[test]
    fn path_segment_edges_answer_as_git_check_ignore_does() {
        assert_recorded_verdicts(PATH_SEGMENT_CASES);
    }

    /// The rows of git's `t/t3070-wildmatch.sh` that gitignore matching shares, each
    /// against a name of one component, with git's own expected answer.
    ///
    /// A rule holding a `/` takes the table's first column (`wildmatch` with
    /// `WM_PATHNAME`); a rule without one is matched against the name alone, so it takes
    /// the third (`pathmatch`), which agrees with the first on names without a `/`. Rows
    /// are left out when git's rule for a `.gitignore` line differs from `wildmatch`'s:
    /// an empty text or pattern; `XXX/`, which the test tool turns into a leading `/`; a
    /// text with `//` or a trailing `/`, which no path has; and a basename rule against a
    /// path of several components, which a `.gitignore` matches against the last one.
    /// The case-insensitive columns do not apply: matching is case-sensitive.
    #[rustfmt::skip]
    const T3070_NAME_CASES: &[RecordedCase] = &[
            RecordedCase { pattern: b"foo", ignored: &[b"foo"], kept: &[] },
            RecordedCase { pattern: b"bar", ignored: &[], kept: &[b"foo"] },
            RecordedCase { pattern: b"???", ignored: &[b"foo"], kept: &[] },
            RecordedCase { pattern: b"??", ignored: &[], kept: &[b"foo"] },
            RecordedCase { pattern: b"*", ignored: &[b"foo"], kept: &[] },
            RecordedCase { pattern: b"f*", ignored: &[b"foo"], kept: &[] },
            RecordedCase { pattern: b"*f", ignored: &[], kept: &[b"foo"] },
            RecordedCase { pattern: b"*foo*", ignored: &[b"foo"], kept: &[] },
            RecordedCase { pattern: b"*ob*a*r*", ignored: &[b"foobar"], kept: &[] },
            RecordedCase { pattern: b"*ab", ignored: &[b"aaaaaaabababab"], kept: &[] },
            RecordedCase { pattern: b"foo\\*", ignored: &[b"foo*"], kept: &[] },
            RecordedCase { pattern: b"foo\\*bar", ignored: &[], kept: &[b"foobar"] },
            RecordedCase { pattern: b"f\\\\oo", ignored: &[b"f\\oo"], kept: &[] },
            RecordedCase { pattern: b"foo\\", ignored: &[], kept: &[b"foo\\"] },
            RecordedCase { pattern: b"*[al]?", ignored: &[b"ball"], kept: &[] },
            RecordedCase { pattern: b"[ten]", ignored: &[], kept: &[b"ten"] },
            RecordedCase { pattern: b"**[!te]", ignored: &[b"ten"], kept: &[] },
            RecordedCase { pattern: b"**[!ten]", ignored: &[], kept: &[b"ten"] },
            RecordedCase { pattern: b"t[a-g]n", ignored: &[b"ten"], kept: &[] },
            RecordedCase { pattern: b"t[!a-g]n", ignored: &[b"ton"], kept: &[b"ten"] },
            RecordedCase { pattern: b"t[^a-g]n", ignored: &[b"ton"], kept: &[] },
            RecordedCase { pattern: b"a[]]b", ignored: &[b"a]b"], kept: &[] },
            RecordedCase { pattern: b"a[]-]b", ignored: &[b"a-b", b"a]b"], kept: &[b"aab"] },
            RecordedCase { pattern: b"a[]a-]b", ignored: &[b"aab"], kept: &[] },
            RecordedCase { pattern: b"]", ignored: &[b"]"], kept: &[] },
            RecordedCase { pattern: b"foo**bar", ignored: &[b"foobazbar"], kept: &[] },
            RecordedCase { pattern: b"f[^eiu][^eiu][^eiu][^eiu][^eiu]r", ignored: &[b"foo-bar"], kept: &[] },
            RecordedCase { pattern: b"**/foo", ignored: &[b"foo"], kept: &[] },
            RecordedCase { pattern: b"a[c-c]st", ignored: &[], kept: &[b"acrt"] },
            RecordedCase { pattern: b"a[c-c]rt", ignored: &[b"acrt"], kept: &[] },
            RecordedCase { pattern: b"[!]-]", ignored: &[b"a"], kept: &[b"]"] },
            RecordedCase { pattern: b"\\", ignored: &[], kept: &[b"\\"] },
            RecordedCase { pattern: b"@foo", ignored: &[b"@foo"], kept: &[b"foo"] },
            RecordedCase { pattern: b"\\[ab]", ignored: &[b"[ab]"], kept: &[] },
            RecordedCase { pattern: b"[[]ab]", ignored: &[b"[ab]"], kept: &[] },
            RecordedCase { pattern: b"[[:]ab]", ignored: &[b"[ab]"], kept: &[] },
            RecordedCase { pattern: b"[[::]ab]", ignored: &[], kept: &[b"[ab]"] },
            RecordedCase { pattern: b"[[:digit]ab]", ignored: &[b"[ab]"], kept: &[] },
            RecordedCase { pattern: b"[\\[:]ab]", ignored: &[b"[ab]"], kept: &[] },
            RecordedCase { pattern: b"\\??\\?b", ignored: &[b"?a?b"], kept: &[] },
            RecordedCase { pattern: b"\\a\\b\\c", ignored: &[b"abc"], kept: &[] },
            RecordedCase { pattern: b"[[:alpha:]][[:digit:]][[:upper:]]", ignored: &[b"a1B"], kept: &[] },
            RecordedCase { pattern: b"[[:digit:][:upper:][:space:]]", ignored: &[b"A", b"1", b" "], kept: &[b"a"] },
            RecordedCase { pattern: b"[[:digit:][:upper:][:spaci:]]", ignored: &[], kept: &[b"1"] },
            RecordedCase { pattern: b"[[:xdigit:]]", ignored: &[b"5", b"f", b"D"], kept: &[] },
            RecordedCase { pattern: b"[[:alnum:][:alpha:][:blank:][:cntrl:][:digit:][:graph:][:lower:][:print:][:punct:][:space:][:upper:][:xdigit:]]", ignored: &[b"_"], kept: &[] },
            RecordedCase { pattern: b"[a-c[:digit:]x-z]", ignored: &[b"5", b"b", b"y"], kept: &[b"q"] },
            RecordedCase { pattern: b"[\\\\-^]", ignored: &[b"]"], kept: &[b"["] },
            RecordedCase { pattern: b"[\\-_]", ignored: &[b"-"], kept: &[] },
            RecordedCase { pattern: b"[\\]]", ignored: &[b"]"], kept: &[b"\\]", b"\\"] },
            RecordedCase { pattern: b"a[]b", ignored: &[], kept: &[b"ab", b"a[]b"] },
            RecordedCase { pattern: b"ab[", ignored: &[], kept: &[b"ab["] },
            RecordedCase { pattern: b"[!", ignored: &[], kept: &[b"ab"] },
            RecordedCase { pattern: b"[-", ignored: &[], kept: &[b"ab"] },
            RecordedCase { pattern: b"[-]", ignored: &[b"-"], kept: &[] },
            RecordedCase { pattern: b"[a-", ignored: &[], kept: &[b"-"] },
            RecordedCase { pattern: b"[!a-", ignored: &[], kept: &[b"-"] },
            RecordedCase { pattern: b"[--A]", ignored: &[b"-", b"5"], kept: &[] },
            RecordedCase { pattern: b"[ --]", ignored: &[b" ", b"$", b"-"], kept: &[b"0"] },
            RecordedCase { pattern: b"[---]", ignored: &[b"-"], kept: &[] },
            RecordedCase { pattern: b"[------]", ignored: &[b"-"], kept: &[] },
            RecordedCase { pattern: b"[a-e-n]", ignored: &[b"-"], kept: &[b"j"] },
            RecordedCase { pattern: b"[!------]", ignored: &[b"a"], kept: &[] },
            RecordedCase { pattern: b"[]-a]", ignored: &[b"^"], kept: &[b"["] },
            RecordedCase { pattern: b"[!]-a]", ignored: &[b"["], kept: &[b"^"] },
            RecordedCase { pattern: b"[a^bc]", ignored: &[b"^"], kept: &[] },
            RecordedCase { pattern: b"[a-]b]", ignored: &[b"-b]"], kept: &[] },
            RecordedCase { pattern: b"[\\]", ignored: &[], kept: &[b"\\"] },
            RecordedCase { pattern: b"[\\\\]", ignored: &[b"\\"], kept: &[] },
            RecordedCase { pattern: b"[!\\\\]", ignored: &[], kept: &[b"\\"] },
            RecordedCase { pattern: b"[A-\\\\]", ignored: &[b"G"], kept: &[] },
            RecordedCase { pattern: b"b*a", ignored: &[], kept: &[b"aaabbb"] },
            RecordedCase { pattern: b"*ba*", ignored: &[], kept: &[b"aabcaa"] },
            RecordedCase { pattern: b"[,]", ignored: &[b","], kept: &[] },
            RecordedCase { pattern: b"[\\\\,]", ignored: &[b",", b"\\"], kept: &[] },
            RecordedCase { pattern: b"[,-.]", ignored: &[b"-"], kept: &[b"+", b"-.]"] },
            RecordedCase { pattern: b"[\\1-\\3]", ignored: &[b"2", b"3"], kept: &[b"4"] },
            RecordedCase { pattern: b"[[-\\]]", ignored: &[b"\\", b"[", b"]"], kept: &[b"-"] },
            RecordedCase { pattern: b"-*-*-*-*-*-*-12-*-*-*-m-*-*-*", ignored: &[b"-adobe-courier-bold-o-normal--12-120-75-75-m-70-iso8859-1"], kept: &[b"-adobe-courier-bold-o-normal--12-120-75-75-X-70-iso8859-1"] },
            RecordedCase { pattern: b"*/*/*", ignored: &[], kept: &[b"foo"] },
            RecordedCase { pattern: b"*X*i", ignored: &[b"abcXdefXghi"], kept: &[] },
            RecordedCase { pattern: b"fo", ignored: &[], kept: &[b"foo"] },
            RecordedCase { pattern: b"[A-Z]", ignored: &[b"A"], kept: &[b"a"] },
            RecordedCase { pattern: b"[a-z]", ignored: &[b"a"], kept: &[b"A"] },
            RecordedCase { pattern: b"[[:upper:]]", ignored: &[b"A"], kept: &[b"a"] },
            RecordedCase { pattern: b"[[:lower:]]", ignored: &[b"a"], kept: &[b"A"] },
            RecordedCase { pattern: b"[B-Za]", ignored: &[b"a"], kept: &[b"A"] },
            RecordedCase { pattern: b"[B-a]", ignored: &[b"a"], kept: &[b"A"] },
            RecordedCase { pattern: b"[Z-y]", ignored: &[b"Z"], kept: &[b"z"] },
    ];

    /// The same table's rows against paths of several components, and against the name
    /// `.`, matched at the pattern level. `git check-ignore` is not asked about these: it
    /// also excludes a path whose parent directory matches, which `foo/*` against
    /// `foo/bba/arr` would, and it reads `.` as the repository's own root.
    #[rustfmt::skip]
    const T3070_PATH_CASES: &[(&[u8], &[u8], bool)] = &[
            (b"foo/**/bar", b"foo/baz/bar", true),
            (b"foo/**/**/bar", b"foo/baz/bar", true),
            (b"foo/**/bar", b"foo/b/a/z/bar", true),
            (b"foo/**/**/bar", b"foo/b/a/z/bar", true),
            (b"foo/**/bar", b"foo/bar", true),
            (b"foo/**/**/bar", b"foo/bar", true),
            (b"foo[/]bar", b"foo/bar", false),
            (b"**/foo", b"bar/baz/foo", true),
            (b"*/foo", b"bar/baz/foo", false),
            (b"**/bar*", b"foo/bar/baz", false),
            (b"**/bar/*", b"deep/foo/bar/baz", true),
            (b"**/bar/*", b"deep/foo/bar", false),
            (b"**/bar**", b"foo/bar/baz", false),
            (b"*/bar/**", b"foo/bar/baz/x", true),
            (b"*/bar/**", b"deep/foo/bar/baz/x", false),
            (b"**/bar/*/*", b"deep/foo/bar/baz/x", true),
            (b"**/t[o]", b"foo/bar/baz/to", true),
            (b"[[:digit:][:upper:][:space:]]", b".", false),
            (b"[[:digit:][:punct:][:space:]]", b".", true),
            (b"[^[:alnum:][:alpha:][:blank:][:cntrl:][:digit:][:lower:][:space:][:upper:][:xdigit:]]", b".", true),
            (b"**/*a*b*g*n*t", b"abcd/abcdefg/abcdefghijk/abcdefghijklmnop.txt", true),
            (b"**/*a*b*g*n*t", b"abcd/abcdefg/abcdefghijk/abcdefghijklmnop.txtz", false),
            (b"*/*/*", b"foo/bar", false),
            (b"*/*/*", b"foo/bba/arr", true),
            (b"*/*/*", b"foo/bb/aa/rr", false),
            (b"**/**/**", b"foo/bb/aa/rr", true),
            (b"*/*X*/*/*i", b"ab/cXd/efXg/hi", true),
            (b"**/*X*/**/*i", b"ab/cXd/efXg/hi", true),
            (b"foo/bar", b"foo/bar", true),
            (b"foo/*", b"foo/bar", true),
            (b"foo/*", b"foo/bba/arr", false),
            (b"foo/**", b"foo/bba/arr", true),
            (b"foo/*arr", b"foo/bba/arr", false),
            (b"foo/**arr", b"foo/bba/arr", false),
            (b"foo/*z", b"foo/bba/arr", false),
            (b"foo/**z", b"foo/bba/arr", false),
    ];

    #[test]
    fn t3070_wildmatch_rows_answer_as_git_does() {
        assert_recorded_verdicts(T3070_NAME_CASES);
        for &(pattern, path, expected) in T3070_PATH_CASES {
            let mut source = pattern.to_vec();
            source.push(b'\n');
            assert_eq!(
                verdict_bytes(&source, path),
                expected,
                "t3070: {} against {}",
                pattern.escape_ascii(),
                path.escape_ascii()
            );
        }
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

    /// Deterministic `SplitMix64`, so a failing case replays from its printed seed.
    struct Random(u64);

    impl Random {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut value = self.0;
            value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            value ^ (value >> 31)
        }

        fn below(&mut self, bound: usize) -> usize {
            usize::try_from(self.next() % u64::try_from(bound).expect("small bound")).expect("fits")
        }

        fn chance(&mut self, one_in: usize) -> bool {
            self.below(one_in) == 0
        }

        fn pick<T: Copy>(&mut self, items: &[T]) -> T {
            items[self.below(items.len())]
        }
    }

    /// One piece of a generated glob, which renders to pattern text and can also be
    /// instantiated as bytes it matches, or may match, so generated names hit rules often.
    #[derive(Clone, Copy, Debug)]
    enum Atom {
        Byte(u8),
        Escaped(u8),
        Any,
        Star,
        /// A bracket expression and bytes to try against it, members or not.
        Class(&'static [u8], &'static [u8]),
    }

    /// Plain bytes: none is special to a glob, and they include `.`, `#`, `!`, a space,
    /// a lone `]`, and bytes of a non-UTF-8 and of a multi-byte UTF-8 name.
    const PLAIN: &[u8] = b"abc.~#!- ]\xff\xc3\xa9";
    const ESCAPED: &[u8] = b"*?[\\#! a.]";
    const CLASSES: &[(&[u8], &[u8])] = &[
        (b"[a-c]", b"abd"),
        (b"[!a]", b"ab."),
        (b"[^.]", b".a"),
        (b"[]a]", b"]ab"),
        (b"[[:alpha:]]", b"a1"),
        (b"[a-]", b"-ab"),
        (b"[\\]]", b"]\\"),
        (b"[.]", b".a"),
        (b"[\xc3]", b"\xc3a"),
    ];
    /// Bytes a generated name is made of, special to a glob or not.
    const NAME_BYTES: &[u8] = b"abc.~#!- ]*?[\\\xff\xc3\xa9";

    impl Atom {
        fn render(self, into: &mut Vec<u8>) {
            match self {
                Self::Byte(byte) => into.push(byte),
                Self::Escaped(byte) => into.extend_from_slice(&[b'\\', byte]),
                Self::Any => into.push(b'?'),
                Self::Star => into.push(b'*'),
                Self::Class(class, _) => into.extend_from_slice(class),
            }
        }

        fn instantiate(self, random: &mut Random, into: &mut Vec<u8>) {
            match self {
                Self::Byte(byte) | Self::Escaped(byte) => into.push(byte),
                Self::Any => into.push(random.pick(NAME_BYTES)),
                Self::Star => {
                    for _ in 0..random.below(3) {
                        into.push(random.pick(NAME_BYTES));
                    }
                }
                Self::Class(_, tries) => into.push(random.pick(tries)),
            }
        }
    }

    #[derive(Clone, Debug)]
    enum Part {
        Atoms(Vec<Atom>),
        /// `**`, followed by an escaped separator when `escaped`.
        DoubleStar {
            escaped: bool,
        },
    }

    #[derive(Debug)]
    struct GeneratedRule {
        line: Vec<u8>,
        parts: Vec<Part>,
    }

    fn literal_atoms(random: &mut Random, dot: bool) -> Vec<Atom> {
        let mut atoms: Vec<Atom> = (0..=random.below(3))
            .map(|_| {
                if random.chance(5) {
                    Atom::Escaped(random.pick(ESCAPED))
                } else {
                    Atom::Byte(random.pick(PLAIN))
                }
            })
            .collect();
        if dot {
            let at = random.below(atoms.len() + 1);
            atoms.insert(at, Atom::Byte(b'.'));
        }
        atoms
    }

    fn glob_atoms(random: &mut Random) -> Vec<Atom> {
        (0..=random.below(4))
            .map(|_| match random.below(8) {
                0 => Atom::Any,
                1 | 2 => Atom::Star,
                3 => {
                    let (class, tries) = random.pick(CLASSES);
                    Atom::Class(class, tries)
                }
                4 => Atom::Escaped(random.pick(ESCAPED)),
                _ => Atom::Byte(random.pick(PLAIN)),
            })
            .collect()
    }

    fn segment(random: &mut Random) -> Part {
        Part::Atoms(if random.chance(2) {
            literal_atoms(random, false)
        } else {
            glob_atoms(random)
        })
    }

    /// A random `.gitignore` line of one of the shapes the index places differently, and
    /// the parts it was made of.
    fn generated_rule(random: &mut Random) -> GeneratedRule {
        let star_then = |atoms: Vec<Atom>| {
            Part::Atoms(std::iter::once(Atom::Star).chain(atoms).collect::<Vec<_>>())
        };
        let mut anchored = false;
        // A rule matching every name decides every entry it is last for, so it is rarer.
        let mut kind = random.below(15);
        if matches!(kind, 4 | 11) && !random.chance(4) {
            kind = 5;
        }
        let parts = match kind {
            0 | 1 => {
                let dot = random.chance(3);
                vec![Part::Atoms(literal_atoms(random, dot))]
            }
            2 => vec![star_then(literal_atoms(random, true))],
            3 => vec![star_then(literal_atoms(random, false))],
            4 => vec![Part::Atoms(vec![Atom::Star])],
            5 => vec![Part::Atoms(glob_atoms(random))],
            6 => {
                anchored = true;
                vec![segment(random)]
            }
            7 => {
                anchored = random.chance(2);
                (0..=random.below(3)).map(|_| segment(random)).collect()
            }
            8 => vec![Part::DoubleStar { escaped: false }, segment(random)],
            9 => vec![segment(random), Part::DoubleStar { escaped: false }],
            10 => vec![segment(random), Part::DoubleStar { escaped: false }, segment(random)],
            11 => {
                anchored = random.chance(2);
                vec![Part::DoubleStar { escaped: false }]
            }
            12 => vec![Part::DoubleStar { escaped: true }, segment(random)],
            13 => vec![
                Part::Atoms(vec![Atom::Star, Atom::Star, Atom::Star]),
                Part::DoubleStar { escaped: false },
                segment(random),
            ],
            _ => {
                return GeneratedRule {
                    line: if random.chance(2) { b"#x".to_vec() } else { Vec::new() },
                    parts: Vec::new(),
                };
            }
        };
        let mut line = Vec::new();
        if random.chance(4) {
            line.push(b'!');
        }
        if anchored {
            line.push(b'/');
        }
        for (at, part) in parts.iter().enumerate() {
            match part {
                Part::Atoms(atoms) => atoms.iter().for_each(|atom| atom.render(&mut line)),
                Part::DoubleStar { .. } => line.extend_from_slice(b"**"),
            }
            if at + 1 < parts.len() {
                if matches!(part, Part::DoubleStar { escaped: true }) {
                    line.push(b'\\');
                }
                line.push(b'/');
            }
        }
        if random.chance(5) {
            line.push(b'/');
        }
        match random.below(12) {
            0 => line.extend_from_slice(b"  "),
            1 => line.extend_from_slice(b"\\ "),
            2 => line.push(b'\r'),
            _ => {}
        }
        GeneratedRule { line, parts }
    }

    /// A path that `rule` matches, or nearly does.
    fn instantiated_path(random: &mut Random, rule: &GeneratedRule) -> Vec<Vec<u8>> {
        let mut path: Vec<Vec<u8>> = Vec::new();
        for part in &rule.parts {
            match part {
                Part::Atoms(atoms) => {
                    let mut component = Vec::new();
                    for atom in atoms {
                        atom.instantiate(random, &mut component);
                    }
                    path.push(component);
                }
                Part::DoubleStar { .. } => {
                    for _ in 0..random.below(3) {
                        path.push(random_name(random));
                    }
                }
            }
        }
        path.retain(|component| !component.is_empty());
        path
    }

    fn random_name(random: &mut Random) -> Vec<u8> {
        (0..=random.below(4)).map(|_| random.pick(NAME_BYTES)).collect()
    }

    /// Where [`RuleIndex::build`] places a rule, for counting what the test exercised.
    fn placement(pattern: &Pattern) -> &'static str {
        match pattern.basename_glob() {
            Some(glob) if is_literal(glob) => "names",
            Some(b"*") => "everything",
            Some(glob) => match glob.strip_prefix(b"*").filter(|tail| is_literal(tail)) {
                Some(tail) if unescaped(tail).any(|byte| byte == b'.') => "extensions",
                Some(_) => "tails",
                None => "basenames",
            },
            None if pattern.shape == Shape::Fixed => "anchored",
            None => "spanning",
        }
    }

    /// The indexed matcher answers exactly as testing every rule in order does, for random
    /// files mixing every shape the index places differently, against paths made to match
    /// their rules and paths made at random.
    #[test]
    fn indexed_matching_answers_as_testing_every_rule_in_order() {
        const FILES: u64 = 6000;
        const QUERIES: usize = 24;
        let mut decided: std::collections::BTreeMap<&str, usize> =
            std::collections::BTreeMap::new();
        let mut outcomes = [0usize; 3];
        for seed in 0..FILES {
            let mut random = Random(seed);
            let rules: Vec<GeneratedRule> =
                (0..=random.below(10)).map(|_| generated_rule(&mut random)).collect();
            let mut source =
                rules.iter().map(|rule| rule.line.as_slice()).collect::<Vec<_>>().join(&b'\n');
            if random.chance(2) {
                source.push(b'\n');
            }
            let matcher = Gitignore::parse(&source);
            for _ in 0..QUERIES {
                let mut path = if random.chance(3) {
                    (0..=random.below(4)).map(|_| random_name(&mut random)).collect()
                } else {
                    let rule = &rules[random.below(rules.len())];
                    instantiated_path(&mut random, rule)
                };
                if path.is_empty() || random.chance(4) {
                    path.push(random_name(&mut random));
                }
                if random.chance(4) {
                    path.insert(0, random_name(&mut random));
                }
                let components: Vec<&[u8]> = path.iter().map(Vec::as_slice).collect();
                let is_dir = random.chance(2);
                let (name, directory) = components.split_last().expect("a path has a name");
                let expected = matcher.matches_in_order(&components, is_dir);
                assert_eq!(
                    matcher.decide(directory, &Name::new(name), is_dir),
                    expected,
                    "seed {seed}: {} against {} (dir {is_dir})",
                    source.escape_ascii(),
                    components.join(&b'/').escape_ascii()
                );
                outcomes[match expected {
                    None => 0,
                    Some(false) => 1,
                    Some(true) => 2,
                }] += 1;
                if let Some(index) = matcher
                    .patterns
                    .iter()
                    .rposition(|pattern| pattern.matches(&components, is_dir))
                {
                    *decided.entry(placement(&matcher.patterns[index])).or_default() += 1;
                }
            }
        }
        // Every place a rule can be indexed decided answers the reference agreed with, and
        // each kind of answer came up often.
        for place in
            ["names", "extensions", "tails", "everything", "basenames", "anchored", "spanning"]
        {
            assert!(decided.get(place).is_some_and(|count| *count >= 500), "{place}: {decided:?}");
        }
        assert!(outcomes.iter().all(|count| *count >= 5000), "{outcomes:?}");
    }

    /// Keys whose hashes collide are still told apart: `costarring` and `liquid` have the
    /// same 32-bit FNV-1a, as their extensions do in `*.costarring` and `*.liquid`.
    #[test]
    fn colliding_keys_answer_for_their_own_rules() {
        assert_eq!(fnv1a(*b"costarring"), fnv1a(*b"liquid"));
        let names = Gitignore::parse(b"costarring\n!liquid\nliquid/\n");
        assert_eq!(names.matches(Path::new("costarring"), false), Some(true));
        assert_eq!(names.matches(Path::new("liquid"), false), Some(false));
        assert_eq!(names.matches(Path::new("liquid"), true), Some(true));
        assert_eq!(names.matches(Path::new("other"), false), None);
        let tails = Gitignore::parse(b"*.costarring\n!*.liquid\n*~\n!*x~\n");
        assert_eq!(tails.matches(Path::new("a.costarring"), false), Some(true));
        assert_eq!(tails.matches(Path::new("a.liquid"), false), Some(false));
        assert_eq!(tails.matches(Path::new("liquid"), false), None);
        assert_eq!(tails.matches(Path::new("a~"), false), Some(true));
        assert_eq!(tails.matches(Path::new("ax~"), false), Some(false));
    }

    /// The checks read from a glob, including those a rule ending in `**` does not have.
    #[test]
    fn glob_checks_hold_what_every_match_must_contain() {
        fn checks(glob: &[u8]) -> (u16, &[u8], &[u8], &[u8], bool) {
            let checks = Checks::of(glob, 0);
            let at = usize::from(checks.required_at);
            (
                checks.min_len,
                &glob[..usize::from(checks.prefix)],
                &glob[glob.len() - usize::from(checks.suffix)..],
                &glob[at..at + usize::from(checks.required_len)],
                checks.flags & Checks::EXACT != 0,
            )
        }
        assert_eq!(checks(b"*.o.*"), (3, &b""[..], &b""[..], &b".o."[..], false));
        assert_eq!(checks(b"*.asn1.[ch]"), (7, &b""[..], &b""[..], &b".asn1."[..], false));
        assert_eq!(checks(b"vmlinux*"), (7, &b"vmlinux"[..], &b""[..], &b""[..], false));
        assert_eq!(checks(b"vmlinux"), (7, &b"vmlinux"[..], &b"vmlinux"[..], &b""[..], true));
        assert_eq!(checks(b"a?bc\\*de[x]f"), (9, &b"a"[..], &b"f"[..], &b"bc"[..], true));
        assert_eq!(checks(b"*[.]"), (1, &b""[..], &b""[..], &b""[..], false));
        let any = Checks::for_pattern(&Pattern::parse(b"a/**").expect("a rule"));
        assert!(any.admit(&[], b"", false));
    }

    /// What a parsed matcher holds, in bytes, by the same accounting as `content_cost`:
    /// allocations by their capacity, without the allocator's own overhead.
    fn footprint(matcher: &Gitignore) -> usize {
        use std::mem::size_of;
        let rules: usize = matcher
            .patterns
            .iter()
            .map(|pattern| {
                size_of::<Pattern>()
                    + pattern
                        .segments
                        .iter()
                        .map(|segment| {
                            size_of::<Segment>()
                                + match segment {
                                    Segment::Glob(glob) => glob.capacity(),
                                    Segment::DoubleStar | Segment::DoubleStarOneOrMore => 0,
                                }
                        })
                        .sum::<usize>()
            })
            .sum();
        let index = &matcher.index;
        size_of::<Gitignore>()
            + rules
            + size_of::<Keyed>() * (index.names.len() + index.extensions.len() + index.tails.len())
            + size_of::<Candidate>()
                * (index.basenames.len() + index.anchored.len() + index.spanning.len())
    }

    /// The charge a source pays still covers the matcher it retains, index included: each
    /// indexed rule adds a 16-byte record, which the rules' own shells, now allocated to
    /// their exact size, leave room for. Checked on files of the shortest distinct rules
    /// of each shape, where the per-line charge is tightest.
    #[test]
    fn a_source_charge_covers_its_indexed_matcher() {
        assert_eq!(std::mem::size_of::<Keyed>(), 16);
        assert_eq!(std::mem::size_of::<Candidate>(), 16);
        let id = |at: usize| {
            [
                b'a' + u8::try_from(at % 26).expect("letter"),
                b'a' + u8::try_from(at / 26 % 26).expect("letter"),
            ]
        };
        // Each line is the first bytes, a distinct two-letter id when asked for, and the
        // last bytes: the shortest rules of every shape the index places differently.
        let shapes: &[(&[u8], bool, &[u8])] = &[
            (b"a", false, b""),
            (b"*", false, b""),
            (b"?", false, b""),
            (b"", true, b""),
            (b"*", true, b""),
            (b"*.", true, b""),
            (b"?", true, b""),
            (b"/", true, b""),
            (b"a/", true, b""),
            (b"**/", true, b""),
            (b"", true, b"/**"),
            (b"!", true, b"/"),
            (b"[", true, b"]"),
            (b"\\#", true, b""),
            (b"*/*/", true, b""),
            (b"/", true, b"/b/c/d"),
        ];
        for (shape, &(first, with_id, last)) in shapes.iter().enumerate() {
            let source: Vec<u8> = (0..676)
                .flat_map(|at| {
                    let id = if with_id { id(at).to_vec() } else { Vec::new() };
                    [first, &id, last, b"\n"].concat()
                })
                .collect();
            let matcher = Gitignore::parse(&source);
            assert_eq!(matcher.rule_count(), 676, "shape {shape}");
            let retained = source.len() + footprint(&matcher);
            let charge = super::super::content_cost(&source);
            assert!(retained <= charge, "shape {shape}: {retained} retained, {charge} charged");
        }
    }
}
