//! Named boolean facts about entries, and the rules that produce them.
//!
//! A **tag** is one bit: this entry is a dotfile, this entry is gitignored, this entry is
//! vendored. Tags are stored per entry beside `ext_id` and `group_id`, and like those they
//! are computed once where an entry enters the index rather than re-derived per query — so
//! a watch upsert is tagged identically to a scan upsert, by construction rather than by
//! two code paths agreeing.
//!
//! A **plane** is something else, and keeping the two apart is the point of this module.
//! A plane is a maintained per-directory aggregate for entries carrying some tag, and it
//! rides the ancestor-merge path, so its cost is paid on every mutation whether or not
//! anyone reads it. Tags are unbounded and nearly free; planes are a small *declared*
//! subset. The earlier design made them one and the same, and the coupling had already
//! forced `hidden` out of the model: a plane would have had to walk the `.git`, cache and
//! virtualenv trees it existed to exclude. Filtering by an unpromoted tag re-aggregates by
//! walking, which is the two-tier rule the query surface already applies to every other
//! predicate.
//!
//! # Tiers
//!
//! Every rule declares what it may read, and the engine refuses at enable time to run one
//! whose tier it cannot afford:
//!
//! - [`TagTier::Name`] sees the basename. Free — the walk already has it.
//! - [`TagTier::Path`] sees the relative path and control files read from the tree.
//!   Cheap, and where `gitignore` lives -- the one rule that costs a dependency, which is
//!   why it and it alone sits behind a cargo feature.
//! - [`TagTier::Content`] would need file bytes. That is fdu's content tier: a different
//!   cost class, opt-in for exactly that reason, and **rejected in v1**. Without this
//!   check, adding a `binary` or `text` tag would silently turn a metadata walk into a
//!   content walk, and the only symptom would be that scans got mysteriously slower.
//!
//! # What is deliberately not a tag
//!
//! Categorical facts. A file's mime type is not a boolean and would want one plane per
//! value; it belongs to the interned-key tally maps that `ext_id` and `group_id` already
//! use. Two shapes, and neither should absorb the other: booleans get a bit and maybe a
//! plane, categories get an id and a map.

#[cfg(feature = "gitignore")]
pub mod gitignore;

use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::Path;

/// Index of a tag rule within the enabled set, and the position of its bit.
///
/// Meaningful only alongside the [`TagRules`] that issued it, exactly as an `ExtId` is
/// meaningful only alongside its interner. Public projections resolve it to a name.
pub type TagId = u8;

/// One bit per enabled rule.
///
/// A `u32` caps the enabled set at 32 rules, which is deliberate: the set is engine
/// declared and small, and a wider bitset would be paying per entry — across millions of
/// entries — for a generality nothing has asked for. [`TagRules::from_names`] refuses to
/// build a set that would not fit rather than silently dropping the overflow.
pub type TagBits = u32;

/// Widest enabled rule set a [`TagBits`] can carry.
pub const MAX_TAG_RULES: usize = TagBits::BITS as usize;

/// What a rule is allowed to look at, and therefore what it costs.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
#[non_exhaustive]
pub enum TagTier {
    /// The entry's basename. Free: the walk already holds it.
    Name,
    /// The entry's relative path, and control files gathered during the walk.
    Path,
    /// The file's bytes. A different cost class, and refused in v1.
    Content,
}

/// One named boolean fact and the tier it needs to decide it.
#[derive(Clone, Debug)]
pub struct TagRule {
    /// Stable name, used on the command line, in rows, and in the fingerprint.
    pub id: Cow<'static, str>,
    /// What this rule reads.
    pub tier: TagTier,
}

/// A promoted rule, for which every directory roll-up maintains a plane.
///
/// The distinction promotion draws is between an *observation* and a *maintained
/// aggregate*. A tag is a bit on an entry: cheap to set, cheap to carry, and answering a
/// question about it means re-aggregating. A plane is roll-up state kept correct on every
/// mutation, so the same question is a read -- paid for on the ancestor-merge path, per
/// promoted rule, on every insert and removal in the tree.
///
/// That is why promotion is a declared subset rather than a property every tag has. Tags
/// are cheap bits; planes are the cost.
///
/// **A plane holds the entries *without* the tag.** For `gitignore` that is the unignored
/// side, which is what a browser shows. The complement derives as all-minus-plane for
/// every field that subtracts -- and `newest_mtime_ns` does not, so the derived side
/// reports it absent rather than wrong. The same choice `ChildRemainder` makes, for the
/// same reason: a maximum cannot be un-merged.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Promoted(pub TagId);

/// Decides a [`TagTier::Name`] rule from a basename.
type NameMatcher = fn(&OsStr) -> bool;

/// What actually decides one enabled rule for one entry.
///
/// An enum rather than a trait object because the set is engine-declared and closed, and
/// because the Name arm has to stay a bare function pointer: it runs once per entry per
/// insert, and a virtual call there would be paid across every entry of every tree for a
/// generality nothing has asked for.
#[derive(Clone, Debug)]
enum Matcher {
    /// Reads the basename. Free -- the walk already holds it.
    Name(NameMatcher),
    /// Reads the relative path and nothing else.
    ///
    /// Path-tier by [`TagTier`]'s own definition -- it reads more than a basename -- and
    /// distinct from the variant below in the thing that matters operationally: it needs no
    /// state gathered from the tree, so it is decided the moment an entry lands and there
    /// is nothing to bind. That is why the tier and the binding question are answered by
    /// two different predicates rather than one.
    PurePath(PurePathMatcher),
    /// Reads the relative path, against state gathered from the tree.
    ///
    /// Gated with the only rule that produces one. Without that feature the variant does
    /// not exist, so the Path-tier arms below vanish with it rather than becoming
    /// unreachable code the compiler has to be told about.
    #[cfg(feature = "gitignore")]
    Path(PathMatcher),
}

/// A rule decided by the relative path alone.
type PurePathMatcher = fn(&Path) -> bool;

/// State a [`TagTier::Path`] rule matches against.
///
/// Built once when the rule set is, from the tree the set will be used on, and shared:
/// the set lives behind an `Arc` on the index, and rebuilding it is how a changed control
/// file takes effect.
#[cfg(feature = "gitignore")]
#[derive(Clone, Debug)]
enum PathMatcher {
    /// Every `.gitignore` under the root, composed with git's own precedence.
    Gitignore(std::sync::Arc<gitignore::GitignoreSet>),
}

#[cfg(feature = "gitignore")]
impl PathMatcher {
    fn matches(&self, relative_path: &Path, is_dir: bool) -> bool {
        match self {
            Self::Gitignore(set) => set.is_ignored(relative_path, is_dir),
        }
    }
}

/// The enabled rule set for one index.
///
/// Built once at open and then read-only, the same shape as
/// [`TypeRegistry`](crate::classify::TypeRegistry): an index built under one set is not
/// answerable under another, so the set's fingerprint is part of the scan scope and a
/// snapshot recorded under different rules is not reused.
#[derive(Clone, Debug, Default)]
pub struct TagRules {
    rules: Vec<TagRule>,
    matchers: Vec<Matcher>,
    /// Rules whose planes every roll-up maintains, sorted and deduplicated.
    ///
    /// Small by construction -- a handful at most -- so a sorted `Vec` scanned linearly
    /// beats anything with a node per entry, the same reasoning `by_group` records.
    promoted: Vec<Promoted>,
    fingerprint: u64,
}

/// Why a rule set was refused.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TagRuleError {
    /// A name that no rule in the engine's catalogue answers to.
    Unknown(String),
    /// The same rule named more than once.
    Duplicate(String),
    /// More rules than a [`TagBits`] can carry.
    TooMany(usize),
    /// A real rule, not enabled on this index.
    ///
    /// Separate from `Unknown` for the same reason `TierRefused` is: the name is spelled
    /// right and the answer is to enable it, not to correct it. Refused rather than
    /// treated as a filter matching nothing, because a mask of zero is indistinguishable
    /// from "no constraint" and the caller would get every entry back believing they had
    /// narrowed.
    NotEnabled {
        /// The rule that was asked for.
        id: String,
        /// What this index does evaluate.
        enabled: String,
    },
    /// An enabled rule that maintains no plane.
    ///
    /// Separate from `NotEnabled` because the two are fixed differently and one of them
    /// costs: enabling a rule is a branch per insert, while promoting one multiplies the
    /// reducer path on every mutation whether or not anyone reads it. A caller who
    /// misreads this as "not enabled" enables an already-enabled rule and is told the
    /// same thing again.
    ///
    /// Refused rather than answered from the totals, which is the tempting fallback: a
    /// tree where nothing carries the tag has a plane equal to the whole, so serving the
    /// whole would look right on exactly the trees that cannot tell the difference, and
    /// wrong everywhere the answer mattered.
    NotPromoted {
        /// The rule that was asked for.
        id: String,
        /// What this index does maintain a plane for.
        promoted: String,
    },
    /// A real rule this build was compiled without.
    ///
    /// Its own variant, not `Unknown`, because the answer is a build flag rather than a
    /// spelling correction -- and because a binary that silently ignored the request would
    /// answer a gitignore question with "nothing is ignored", which is a wrong answer
    /// rather than a missing one.
    Unavailable {
        /// The rule that was asked for.
        id: String,
        /// The cargo feature that carries it.
        feature: &'static str,
    },
    /// A rule whose tier the engine will not run.
    ///
    /// Carried as its own variant rather than folded into `Unknown` because the two need
    /// opposite responses: an unknown name is a typo, and this is a real rule that costs
    /// more than the caller asked to spend.
    TierRefused {
        /// The rule that was asked for.
        id: String,
        /// The tier it needs, which this version will not run.
        tier: TagTier,
    },
}

impl std::fmt::Display for TagRuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown(id) => {
                write!(f, "unknown tag rule {id:?}; available: {}", available_names())
            }
            Self::Duplicate(id) => write!(f, "tag rule {id:?} named twice"),
            Self::Unavailable { id, feature } => write!(
                f,
                "tag rule {id:?} is not in this build: it needs the {feature:?} cargo feature, \
                 which is on by default and off under --no-default-features"
            ),
            Self::NotEnabled { id, enabled } if enabled.is_empty() => {
                write!(f, "tag rule {id:?} is not enabled: this index evaluates no tag rules")
            }
            Self::NotEnabled { id, enabled } => {
                write!(f, "tag rule {id:?} is not enabled; enabled here: {enabled}")
            }
            Self::NotPromoted { id, promoted } if promoted.is_empty() => {
                write!(f, "tag rule {id:?} maintains no plane: this index promotes no tag rules")
            }
            Self::NotPromoted { id, promoted } => {
                write!(f, "tag rule {id:?} maintains no plane; promoted here: {promoted}")
            }
            Self::TooMany(count) => {
                write!(f, "{count} tag rules exceeds the {MAX_TAG_RULES} a tag set can carry")
            }
            Self::TierRefused { id, tier } => write!(
                f,
                "tag rule {id:?} reads {tier:?}-tier data, which this version will not enable: \
                 it would turn a metadata walk into a content walk"
            ),
        }
    }
}

impl std::error::Error for TagRuleError {}

/// Every rule the engine knows how to run, in a stable order.
///
/// Engine-declared rather than caller-supplied in v1. A runtime rule dialect is plausible
/// later — [`TypeRegistry`](crate::classify::TypeRegistry) already shows the shape — but a
/// closed set is what lets the tier check be a guarantee rather than a request.
fn catalogue() -> &'static [(TagRule, Decides)] {
    static CATALOGUE: std::sync::LazyLock<Vec<(TagRule, Decides)>> =
        std::sync::LazyLock::new(|| {
            vec![
                (
                    TagRule { id: Cow::Borrowed("dotfile"), tier: TagTier::Name },
                    Decides::Name(is_dotfile as NameMatcher),
                ),
                (
                    TagRule { id: Cow::Borrowed("vendored"), tier: TagTier::Path },
                    Decides::PurePath(is_vendored as PurePathMatcher),
                ),
                (
                    TagRule { id: Cow::Borrowed("documentation"), tier: TagTier::Path },
                    Decides::PurePath(is_documentation as PurePathMatcher),
                ),
                (
                    TagRule { id: Cow::Borrowed("gitignore"), tier: TagTier::Path },
                    Decides::Gitignore,
                ),
            ]
        });
    &CATALOGUE
}

/// How the catalogue says a rule is decided, before it is bound to a tree.
///
/// The catalogue is static and a Path-tier matcher is not: it is built from the tree the
/// rule set will be used on. So the catalogue names the *kind*, and
/// [`TagRules::from_names`] binds it.
#[derive(Clone, Copy, Debug)]
enum Decides {
    /// A pure function of the basename, ready to use as it stands.
    Name(NameMatcher),
    /// A pure function of the relative path, ready to use as it stands.
    PurePath(PurePathMatcher),
    /// Needs a `.gitignore` evaluator built from the root.
    Gitignore,
}

/// The cargo feature carrying the gitignore rule, named once.
///
/// Only reachable in a build without that feature, which is the build whose error message
/// has to name it.
#[cfg(not(feature = "gitignore"))]
const GITIGNORE_FEATURE: &str = "gitignore";

fn available_names() -> String {
    catalogue().iter().map(|(rule, _)| rule.id.as_ref()).collect::<Vec<_>>().join(", ")
}

/// A name beginning with a dot, which is the whole rule.
///
/// Deliberately *not* the hidden-path scope rule. This tags an entry and leaves it in the
/// index with both numbers visible; scope pruning removes it entirely. Different axes, and
/// the spec's own axis test is what separates them.
/// A path component naming a conventional vendored dependency tree.
///
/// The same predicate `Classification.flags.vendored` reports, and it has to be: the
/// classification and the tag are two views of one fact, and two copies of the rule would
/// let `--not-tag vendored` and a row's own `vendored: true` disagree about one file.
/// [`crate::classify`] calls this rather than keeping its own list.
pub(crate) fn is_vendored(relative_path: &Path) -> bool {
    const NAMES: [&str; 5] = ["vendor", "vendored", "third_party", "third-party", "node_modules"];
    relative_path.components().any(|component| {
        let component = component.as_os_str().to_string_lossy();
        NAMES.iter().any(|name| component.eq_ignore_ascii_case(name))
    })
}

/// A conventional documentation tree, or a basename every project uses for one.
///
/// Shared with `Classification.flags.documentation` for the reason `is_vendored` is.
pub(crate) fn is_documentation(relative_path: &Path) -> bool {
    const DIRECTORIES: [&str; 3] = ["doc", "docs", "documentation"];
    const STEMS: [&str; 3] = ["readme", "changelog", "contributing"];
    if relative_path.components().any(|component| {
        let component = component.as_os_str().to_string_lossy();
        DIRECTORIES.iter().any(|name| component.eq_ignore_ascii_case(name))
    }) {
        return true;
    }
    relative_path.file_name().and_then(|name| name.to_str()).is_some_and(|name| {
        STEMS.iter().any(|stem| name.eq_ignore_ascii_case(stem) || starts_with_stem(name, stem))
    })
}

/// `readme.md` matches the `readme` stem; `readmes.txt` does not.
///
/// `get(..len)` rather than a slice index, because a name is arbitrary UTF-8 and the stem's
/// byte length can land inside a character -- `réadme.md` would panic on the index form.
/// The classification carried this shape and the first copy of this predicate did not; it
/// is now one function rather than two, which is the point of the fold.
fn starts_with_stem(name: &str, stem: &str) -> bool {
    name.get(..stem.len()).is_some_and(|prefix| prefix.eq_ignore_ascii_case(stem))
        && name.as_bytes().get(stem.len()) == Some(&b'.')
}

fn is_dotfile(name: &OsStr) -> bool {
    let bytes = name.as_encoded_bytes();
    bytes.first() == Some(&b'.') && bytes != b"." && bytes != b".."
}

impl TagRules {
    /// The empty set: no rules enabled, every entry untagged, fingerprint zero.
    ///
    /// Zero on purpose. It is what every index built before tags existed fingerprints to,
    /// so adding this machinery invalidates no snapshot anybody already has.
    pub fn none() -> &'static Self {
        static NONE: std::sync::LazyLock<TagRules> = std::sync::LazyLock::new(TagRules::default);
        &NONE
    }

    /// Enable rules by name, in the order given.
    ///
    /// The order is the bit order, and it is the caller's, so a set is reproducible from
    /// what a user typed rather than from an engine-internal ordering they cannot see.
    ///
    /// A Path-tier rule comes back *unbound*: named, positioned, and carrying no state to
    /// match against yet. It cannot be otherwise, because the state a Path-tier rule reads
    /// is a set of control files whose locations only the index knows, and the index does
    /// not exist when a command line is parsed. [`TagRules::bound_to`] completes the set
    /// once it does, and [`TagRules::needs_binding`] is how a caller checks it did.
    pub fn from_names<I, S>(names: I) -> Result<Self, TagRuleError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut rules = Vec::new();
        let mut matchers = Vec::new();
        for name in names {
            let name = name.as_ref();
            let found = catalogue()
                .iter()
                .find(|(rule, _)| rule.id == name)
                .ok_or_else(|| TagRuleError::Unknown(name.to_string()))?;
            if rules.iter().any(|rule: &TagRule| rule.id == name) {
                return Err(TagRuleError::Duplicate(name.to_string()));
            }
            if found.0.tier == TagTier::Content {
                return Err(TagRuleError::TierRefused { id: name.to_string(), tier: found.0.tier });
            }
            matchers.push(bind(name, found.1)?);
            rules.push(found.0.clone());
        }
        if rules.len() > MAX_TAG_RULES {
            return Err(TagRuleError::TooMany(rules.len()));
        }
        let fingerprint = fingerprint_of(&rules, &[]);
        Ok(Self { rules, matchers, promoted: Vec::new(), fingerprint })
    }

    /// The same enabled set, with these rules promoted to maintained planes.
    ///
    /// Promotion changes what a stored roll-up *contains*, so it moves the fingerprint: a
    /// snapshot written without a plane cannot be reinterpreted as one with an empty
    /// plane, because those say different things -- "nothing was outside the tag" and
    /// "nobody was counting". Rebinding control files does not move it, and that
    /// difference is the whole distinction between what a rule reads and what it is.
    ///
    /// Naming a rule that is not enabled is refused rather than ignored: a caller that
    /// promoted a typo would get every plane silently empty and no way to tell that from
    /// a tree where the tag matched nothing.
    pub fn with_promoted<I, S>(mut self, names: I) -> Result<Self, TagRuleError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut promoted = Vec::new();
        for name in names {
            let name = name.as_ref();
            // The same three-way answer `mask_of` gives, because a caller promoting a rule
            // makes the same three mistakes: a typo, a real rule left off `from_names`, and
            // a name given twice. Reporting the second as `Unknown` sent the caller to
            // check their spelling of a name that was spelled correctly.
            let id = match self.id_of(name) {
                Some(id) => id,
                None if catalogue().iter().any(|(rule, _)| rule.id == name) => {
                    return Err(TagRuleError::NotEnabled {
                        id: name.to_string(),
                        enabled: self.enabled_names(),
                    });
                }
                None => return Err(TagRuleError::Unknown(name.to_string())),
            };
            if promoted.contains(&Promoted(id)) {
                return Err(TagRuleError::Duplicate(name.to_string()));
            }
            promoted.push(Promoted(id));
        }
        promoted.sort_unstable();
        self.fingerprint = fingerprint_of(&self.rules, &promoted);
        self.promoted = promoted;
        Ok(self)
    }

    /// Rules whose planes every roll-up maintains, sorted.
    pub fn promoted(&self) -> &[Promoted] {
        &self.promoted
    }

    /// The same enabled set, bound to the control files at these directories.
    ///
    /// Both the first binding and every rebinding: the enabled rules and their bit order
    /// do not move -- so the fingerprint does not move either, and no cache is
    /// invalidated -- while the state they match against is read from the tree. The index
    /// adopts the result and re-tags.
    ///
    /// `directories` are relative to `root` and come from the index, which already knows
    /// where every control file is. Discovering them here instead would mean walking the
    /// tree, and the caller that most needs this -- a cache-only open -- is precisely the
    /// one forbidden to.
    ///
    /// Infallible on purpose, and not `from_names` over the same names. Re-resolving names
    /// would reintroduce every way naming can fail -- unknown, duplicate, refused tier,
    /// missing feature -- into a path where none of them can happen, and would leave a
    /// caller holding a `Result` it has no sensible way to handle. Binding the matchers
    /// this set already holds cannot fail, and the fingerprint is carried across rather
    /// than recomputed, which is the property that makes this cache-safe.
    #[must_use]
    pub fn bound_to<'a, I>(&self, root: &Path, directories: I) -> Self
    where
        I: IntoIterator<Item = &'a Path>,
    {
        #[cfg(not(feature = "gitignore"))]
        let _ = (root, directories);
        #[cfg(feature = "gitignore")]
        let bound =
            std::sync::Arc::new(gitignore::GitignoreSet::from_directories(root, directories));
        let matchers = self
            .matchers
            .iter()
            .map(|matcher| match matcher {
                Matcher::Name(decide) => Matcher::Name(*decide),
                Matcher::PurePath(decide) => Matcher::PurePath(*decide),
                #[cfg(feature = "gitignore")]
                Matcher::Path(PathMatcher::Gitignore(_)) => {
                    Matcher::Path(PathMatcher::Gitignore(bound.clone()))
                }
            })
            .collect();
        Self {
            rules: self.rules.clone(),
            matchers,
            promoted: self.promoted.clone(),
            fingerprint: self.fingerprint,
        }
    }

    /// Whether a Path-tier rule is enabled but not yet bound to a tree.
    ///
    /// True between [`TagRules::from_names`] and [`TagRules::bound_to`], and the reason
    /// that window has to be closed before anything reads a tag: an unbound gitignore rule
    /// answers "nothing is ignored", which is a wrong answer rather than a missing one.
    /// The open path binds before it hands an index to anybody, and the index's own tag
    /// readers assert it in debug builds so a path that forgets fails loudly in tests
    /// rather than quietly in production. Evaluation itself does *not* assert: a scan tags
    /// each entry as it lands, before any binding is possible, and the bind that follows
    /// re-tags the tree. Writing under unbound rules is a step; reading under them is
    /// the bug.
    pub fn needs_binding(&self) -> bool {
        #[cfg(feature = "gitignore")]
        {
            self.matchers.iter().any(|matcher| {
                matches!(matcher, Matcher::Path(PathMatcher::Gitignore(set)) if set.is_unbound())
            })
        }
        #[cfg(not(feature = "gitignore"))]
        {
            false
        }
    }

    /// Directories whose control files this set read, relative to the root.
    ///
    /// The scope a rebuild moved. A consumer's cached rows for these subtrees may now be
    /// tagged differently without any entry beneath them having been touched, which is a
    /// change nothing else in the delta stream would tell them about.
    pub fn governed_directories(&self) -> Vec<std::path::PathBuf> {
        let mut governed = Vec::new();
        for matcher in &self.matchers {
            match matcher {
                Matcher::Name(_) | Matcher::PurePath(_) => {}
                #[cfg(feature = "gitignore")]
                Matcher::Path(PathMatcher::Gitignore(set)) => {
                    governed.extend(set.governed_directories().map(Path::to_path_buf));
                }
            }
        }
        governed.sort();
        governed.dedup();
        governed
    }

    /// Whether an enabled rule reads a control file that this path is.
    ///
    /// The signal a watch loop acts on: creating, editing or removing a path this answers
    /// `true` for changes what the rules decide, without changing which rules are enabled.
    pub fn is_control_file(&self, relative_path: &Path) -> bool {
        let _ = relative_path;
        self.matchers.iter().any(|matcher| match matcher {
            Matcher::Name(_) | Matcher::PurePath(_) => false,
            #[cfg(feature = "gitignore")]
            Matcher::Path(PathMatcher::Gitignore(_)) => gitignore::is_control_file(relative_path),
        })
    }

    /// Identity of the enabled set, folded into the scan scope.
    ///
    /// Order-sensitive, because the order is the bit order: the same rules enabled in a
    /// different order produce different bits, and a snapshot's bits are only readable
    /// under the set that wrote them.
    pub fn fingerprint(&self) -> u64 {
        self.fingerprint
    }

    /// Whether any enabled rule reads a path.
    ///
    /// The loader asks before it starts building paths at all: it holds a parent id and a
    /// basename per record, and constructing a relative path for every entry when nothing
    /// reads one is precisely the per-record allocation the snapshot format exists to
    /// avoid. False is the common case, and it is the whole answer.
    pub fn needs_path(&self) -> bool {
        self.matchers.iter().any(|matcher| !matches!(matcher, Matcher::Name(_)))
    }

    /// Whether any rule is enabled.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// The enabled rules, in bit order.
    pub fn rules(&self) -> &[TagRule] {
        &self.rules
    }

    /// Position of a rule by name, when enabled.
    pub fn id_of(&self, name: &str) -> Option<TagId> {
        self.rules
            .iter()
            .position(|rule| rule.id == name)
            .and_then(|index| TagId::try_from(index).ok())
    }

    /// Name of an enabled rule by position.
    pub fn name_of(&self, id: TagId) -> Option<&str> {
        self.rules.get(usize::from(id)).map(|rule| rule.id.as_ref())
    }

    /// Evaluate every enabled rule against one entry.
    ///
    /// `relative_path` is a closure rather than a value because the index's two insert
    /// paths are not symmetric. An upsert already holds the path; the snapshot loader holds
    /// a parent id and a basename, and reconstructing a path per record is precisely the
    /// work a callgrind profile put at about 27% of load in the allocator and which that
    /// path was rewritten to avoid. Tagging must not quietly hand it back. So the path is
    /// materialized at most once, and only for a rule that reads one — which no
    /// [`TagTier::Name`] rule does.
    ///
    /// Returns zero when nothing is enabled, which is the default and costs one branch.
    pub fn evaluate<'a, F>(&self, name: &OsStr, is_dir: bool, relative_path: F) -> TagBits
    where
        F: FnOnce() -> Cow<'a, Path>,
    {
        if self.rules.is_empty() {
            return 0;
        }
        // Materialized at most once, and only when a rule actually reads it. The closure
        // is `FnOnce`, so it is moved out on first use and every later Path-tier rule
        // reads the value it produced. Both locals are Path-tier machinery, so a build
        // with no Path-tier rule does not have them -- or the closure they hold -- at all.
        let (mut produce, mut path) = (Some(relative_path), None::<Cow<'a, Path>>);
        #[cfg(not(feature = "gitignore"))]
        let _ = is_dir;
        let mut bits: TagBits = 0;
        for (index, matcher) in self.matchers.iter().enumerate() {
            let hit = match matcher {
                Matcher::Name(decide) => decide(name),
                Matcher::PurePath(decide) => {
                    if path.is_none() {
                        path = produce.take().map(|produce| produce());
                    }
                    path.as_deref().is_some_and(decide)
                }
                #[cfg(feature = "gitignore")]
                Matcher::Path(decide) => {
                    if path.is_none() {
                        path = produce.take().map(|produce| produce());
                    }
                    path.as_deref().is_some_and(|path| decide.matches(path, is_dir))
                }
            };
            if hit {
                bits |= 1 << index;
            }
        }
        bits
    }

    /// Resolve rule names to a mask over this set.
    ///
    /// Lives here rather than on each surface because a mask is only meaningful against the
    /// set that issued it, and because the command line and the library must reject the
    /// same names with the same words. An empty iterator yields zero, which every
    /// [`TagFilter`](crate::query::TagFilter) field reads as "no constraint".
    pub fn mask_of<I, S>(&self, names: I) -> Result<TagBits, TagRuleError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut mask: TagBits = 0;
        for name in names {
            let name = name.as_ref();
            let Some(id) = self.id_of(name) else {
                if catalogue().iter().any(|(rule, _)| rule.id == name) {
                    return Err(TagRuleError::NotEnabled {
                        id: name.to_string(),
                        enabled: self.enabled_names(),
                    });
                }
                return Err(TagRuleError::Unknown(name.to_string()));
            };
            mask |= 1 << id;
        }
        Ok(mask)
    }

    /// Resolve one rule name to the plane this set maintains for it.
    ///
    /// The plane counterpart of [`mask_of`](Self::mask_of), and here for the same reason: a
    /// [`Promoted`] indexes into *this* set's bit order and means nothing against another,
    /// so the surfaces must not resolve names themselves. It answers in four ways rather
    /// than two, and each names the step that fixes it -- a typo, a rule to enable, a rule
    /// to promote, or the plane.
    pub fn plane_of(&self, name: &str) -> Result<Promoted, TagRuleError> {
        let Some(id) = self.id_of(name) else {
            if catalogue().iter().any(|(rule, _)| rule.id == name) {
                return Err(TagRuleError::NotEnabled {
                    id: name.to_string(),
                    enabled: self.enabled_names(),
                });
            }
            return Err(TagRuleError::Unknown(name.to_string()));
        };
        if !self.promoted.contains(&Promoted(id)) {
            return Err(TagRuleError::NotPromoted {
                id: name.to_string(),
                promoted: self.promoted_names().join(", "),
            });
        }
        Ok(Promoted(id))
    }

    /// Names of the rules this set maintains planes for, in bit order.
    pub fn promoted_names(&self) -> Vec<&str> {
        self.promoted.iter().filter_map(|Promoted(id)| self.name_of(*id)).collect()
    }

    /// The enabled rules as one comma-separated list, for an error that must name them.
    fn enabled_names(&self) -> String {
        self.rules.iter().map(|rule| rule.id.as_ref()).collect::<Vec<_>>().join(", ")
    }

    /// Names of the tags set in `bits`, in bit order.
    pub fn names_of(&self, bits: TagBits) -> Vec<&str> {
        self.rules
            .iter()
            .enumerate()
            .filter(|(index, _)| bits & (1 << index) != 0)
            .map(|(_, rule)| rule.id.as_ref())
            .collect()
    }
}

/// Turn a catalogue entry into the matcher that will decide it for this tree.
///
/// The one place a Path-tier rule is bound, and the one place a rule missing from the
/// build is refused. Both belong together: "this build does not carry the rule" and "this
/// is how the rule is wired up" are the same question asked of the same table.
// Infallible in a build that carries every rule, and not in one that does not: the `Err`
// arm below is the refusal a missing feature produces. Scoped to the feature rather than
// allowed outright, so the lint still speaks in the configuration where it can be right.
#[cfg_attr(feature = "gitignore", allow(clippy::unnecessary_wraps))]
fn bind(name: &str, decides: Decides) -> Result<Matcher, TagRuleError> {
    let _ = name;
    match decides {
        Decides::Name(matcher) => Ok(Matcher::Name(matcher)),
        Decides::PurePath(matcher) => Ok(Matcher::PurePath(matcher)),
        // Unbound: the rule is named and positioned, and the state it matches against
        // arrives with `TagRules::bound_to` once an index can say where the control files
        // are. Nothing here can read the tree, because on the cache-only path nothing may.
        #[cfg(feature = "gitignore")]
        Decides::Gitignore => Ok(Matcher::Path(PathMatcher::Gitignore(std::sync::Arc::new(
            gitignore::GitignoreSet::unbound(),
        )))),
        #[cfg(not(feature = "gitignore"))]
        Decides::Gitignore => {
            Err(TagRuleError::Unavailable { id: name.to_string(), feature: GITIGNORE_FEATURE })
        }
    }
}

/// FNV-1a over the enabled names and tiers, in order.
fn fingerprint_of(rules: &[TagRule], promoted: &[Promoted]) -> u64 {
    if rules.is_empty() {
        return 0;
    }
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut mix = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x1000_0000_01b3);
        }
    };
    for rule in rules {
        mix(rule.id.as_bytes());
        mix(&[rule.tier as u8]);
        mix(b"\x1f");
    }
    // Only when something is promoted, so an unpromoted set fingerprints exactly as it did
    // before planes existed and no cache written under it is discarded.
    if !promoted.is_empty() {
        mix(b"planes\x1f");
        for Promoted(id) in promoted {
            mix(&[*id]);
        }
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    fn bits(rules: &TagRules, name: &str) -> TagBits {
        rules.evaluate(&OsString::from(name), false, || Cow::Borrowed(Path::new(name)))
    }

    #[test]
    fn the_empty_set_fingerprints_to_zero_so_no_snapshot_is_invalidated() {
        // Every index built before this module existed recorded zero here. If the empty
        // set hashed to anything else, shipping tags would discard every cache in the
        // world to express "still no rules".
        assert_eq!(TagRules::none().fingerprint(), 0);
        assert_eq!(TagRules::from_names::<[&str; 0], &str>([]).expect("empty").fingerprint(), 0);
        assert!(TagRules::none().is_empty());
    }

    #[test]
    fn the_dotfile_rule_tags_what_a_person_would_call_hidden() {
        let rules = TagRules::from_names(["dotfile"]).expect("enables");
        assert_ne!(bits(&rules, ".gitignore"), 0);
        assert_ne!(bits(&rules, ".config"), 0);
        assert_eq!(bits(&rules, "README.md"), 0);
        // A relative-path component, not a dotfile.
        assert_eq!(bits(&rules, "."), 0);
        assert_eq!(bits(&rules, ".."), 0);
    }

    #[test]
    fn an_unknown_rule_lists_the_ones_that_exist() {
        let error = TagRules::from_names(["nope"]).expect_err("rejected");
        assert_eq!(error, TagRuleError::Unknown("nope".to_string()));
        assert!(error.to_string().contains("dotfile"), "{error}");
    }

    #[test]
    fn a_rule_named_twice_is_a_typo_rather_than_a_no_op() {
        assert_eq!(
            TagRules::from_names(["dotfile", "dotfile"]).expect_err("rejected"),
            TagRuleError::Duplicate("dotfile".to_string())
        );
    }

    /// The tier check is the guarantee that a metadata walk stays a metadata walk.
    ///
    /// There is no Content-tier rule in the catalogue yet, so this exercises the refusal
    /// directly rather than through `from_names` — the alternative is a test that passes
    /// because the thing it guards against does not exist, which stops being true the
    /// moment somebody adds one.
    #[test]
    fn a_content_tier_rule_is_refused_with_a_message_about_cost() {
        assert!(
            catalogue().iter().all(|(rule, _)| rule.tier != TagTier::Content),
            "no Content-tier rule ships in v1"
        );
        let refused =
            TagRuleError::TierRefused { id: "binary".to_string(), tier: TagTier::Content };
        let message = refused.to_string();
        assert!(message.contains("content walk"), "{message}");
        assert!(message.contains("binary"), "{message}");
    }

    #[test]
    fn the_fingerprint_follows_the_enabled_set_and_its_order() {
        let one = TagRules::from_names(["dotfile"]).expect("enables");
        assert_ne!(one.fingerprint(), 0, "an enabled rule is not the empty set");
        assert_eq!(
            one.fingerprint(),
            TagRules::from_names(["dotfile"]).expect("enables").fingerprint()
        );
    }

    /// A rebind reads the tree again without moving the identity of the set.
    ///
    /// The load-bearing half is the fingerprint. If rebinding produced a different one,
    /// every `.gitignore` save would invalidate the snapshot for a tree whose enabled
    /// rules had not changed at all -- a cache thrown away to express "the same rules,
    /// applied to a file that moved".
    #[cfg(feature = "gitignore")]
    #[test]
    fn rebinding_re_reads_the_control_files_without_moving_the_fingerprint() {
        let dir = tempfile::tempdir().expect("a temporary directory");
        std::fs::write(dir.path().join(".gitignore"), "*.log\n").expect("write");

        let named = TagRules::from_names(["gitignore"]).expect("enables");
        assert!(named.needs_binding(), "a Path-tier rule arrives unbound and says so");
        let bits = |rules: &TagRules, path: &str| {
            rules.evaluate(&OsString::from(path), false, || Cow::Borrowed(Path::new(path)))
        };

        // The root governs, which is what an index would report: one control file, in the
        // directory the empty relative path names.
        let governing = [std::path::PathBuf::new()];
        let rules = named.bound_to(dir.path(), governing.iter().map(std::path::PathBuf::as_path));
        assert!(!rules.needs_binding());
        assert_ne!(bits(&rules, "debug.log"), 0);
        assert_eq!(bits(&rules, "notes.txt"), 0);

        std::fs::write(dir.path().join(".gitignore"), "*.txt\n").expect("rewrite");
        let rebound = rules.bound_to(dir.path(), governing.iter().map(std::path::PathBuf::as_path));
        assert_eq!(bits(&rebound, "debug.log"), 0, "the old rule is gone");
        assert_ne!(bits(&rebound, "notes.txt"), 0, "and the new one applies");
        assert_eq!(
            rebound.fingerprint(),
            rules.fingerprint(),
            "the same rules were enabled, so no snapshot is invalidated"
        );
        assert_eq!(rebound.governed_directories(), vec![std::path::PathBuf::new()]);
    }

    /// The control-file question is asked of the enabled set, not of the catalogue.
    #[cfg(feature = "gitignore")]
    #[test]
    fn only_a_set_that_reads_control_files_recognizes_one() {
        let names = TagRules::from_names(["dotfile"]).expect("enables");
        assert!(
            !names.is_control_file(Path::new(".gitignore")),
            "a Name-tier set reads no files, so nothing is a control file for it"
        );

        let paths = TagRules::from_names(["gitignore"]).expect("enables");
        assert!(paths.is_control_file(Path::new(".gitignore")));
        assert!(paths.is_control_file(Path::new("docs/.gitignore")));
        assert!(!paths.is_control_file(Path::new("docs/notes.md")));
    }

    /// A build without the rule refuses it by name and says which flag carries it.
    ///
    /// Exercised through `Display` rather than through `from_names`, because in a build
    /// that *has* the feature the refusal is unreachable -- and a test that passes because
    /// the thing it guards cannot happen stops being true in the build where it can.
    #[test]
    fn a_rule_missing_from_the_build_names_the_feature_rather_than_the_spelling() {
        let refused =
            TagRuleError::Unavailable { id: "gitignore".to_string(), feature: "gitignore" };
        let message = refused.to_string();
        assert!(message.contains("not in this build"), "{message}");
        assert!(message.contains("cargo feature"), "{message}");
        assert!(message.contains("--no-default-features"), "{message}");
    }

    #[test]
    fn a_mask_over_a_rule_that_is_real_but_off_says_so_rather_than_matching_nothing() {
        let rules = TagRules::from_names(["dotfile"]).expect("enables");
        assert_eq!(rules.mask_of(["dotfile"]).expect("enabled"), 1);
        assert_eq!(rules.mask_of::<[&str; 0], &str>([]).expect("empty"), 0, "no names, no filter");

        // A typo and a real-but-off rule need opposite advice, so they are different errors.
        assert!(matches!(rules.mask_of(["nope"]).expect_err("rejected"), TagRuleError::Unknown(_)));
        // Nothing in the catalogue is off while `dotfile` is the only rule, so this
        // exercises the refusal against the empty set, where it is reachable today.
        let error = TagRules::none().mask_of(["dotfile"]).expect_err("rejected");
        assert!(matches!(error, TagRuleError::NotEnabled { .. }), "{error:?}");
        assert!(error.to_string().contains("no tag rules"), "{error}");
    }

    /// A tag and a classification flag are one fact, so they answer identically.
    ///
    /// They were two copies of one list, and the copies had already drifted: the
    /// classification's stem check used `get(..len)` and the newer one indexed, which
    /// panics on a name whose stem length lands inside a character. One predicate now backs
    /// both, and this is what says so -- for every path either view might see, including
    /// the ones that separate the two lists.
    #[test]
    fn a_flag_and_its_tag_are_one_fact_with_one_predicate_behind_them() {
        let rules =
            TagRules::from_names(["vendored", "documentation"]).expect("both are in the catalogue");
        let tag = |path: &str, name: &str| -> Vec<&str> {
            let relative = std::path::PathBuf::from(path);
            let bits =
                rules.evaluate(OsStr::new(name), false, || Cow::Borrowed(relative.as_path()));
            rules.names_of(bits)
        };

        for (path, expected) in [
            ("node_modules/left-pad/index.js", vec!["vendored"]),
            ("third-party/zlib/zlib.c", vec!["vendored"]),
            ("VENDOR/thing.rs", vec!["vendored"]),
            ("docs/guide.md", vec!["documentation"]),
            ("README.md", vec!["documentation"]),
            ("CHANGELOG", vec!["documentation"]),
            // A stem is a whole component, not a prefix: `readmes.txt` is not a readme.
            ("readmes.txt", vec![]),
            // Multi-byte, which is where the drifted copy would have panicked rather than
            // answered.
            ("réadme.md", vec![]),
            ("vendor/docs/api.md", vec!["vendored", "documentation"]),
            ("src/main.rs", vec![]),
        ] {
            let name = std::path::Path::new(path)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            assert_eq!(tag(path, &name), expected, "tagging {path}");

            // The classification reports the same two, unchanged and without a rule
            // enabled: a consumer reading the flag needs no tag set at all.
            let verdict = crate::classify::classify_with(
                crate::classify::TypeRegistry::compiled(),
                std::path::Path::new(path),
                None,
            );
            assert_eq!(
                verdict.flags.vendored,
                expected.contains(&"vendored"),
                "classification disagrees about vendored for {path}",
            );
            assert_eq!(
                verdict.flags.documentation,
                expected.contains(&"documentation"),
                "classification disagrees about documentation for {path}",
            );
        }
    }

    /// `generated` is not a tag rule, and the tier check is the reason rather than an
    /// oversight.
    ///
    /// It reads the file's opening bytes. Enabling it as a tag would silently turn a
    /// metadata walk into a content walk, and the only symptom would be that scans got
    /// mysteriously slower -- which is exactly the failure `TagTier::Content` exists to
    /// refuse. It stays on the classification of a file whose bytes were read for some
    /// other reason, where it is free.
    #[test]
    fn the_generated_flag_stays_a_classification_because_its_tier_is_refused() {
        let refused = TagRules::from_names(["generated"]).expect_err("not in the catalogue");
        assert!(matches!(refused, TagRuleError::Unknown(_)), "{refused:?}");

        let bytes = b"// Code generated by protoc. DO NOT EDIT.
fn main() {}";
        let verdict = crate::classify::classify_with(
            crate::classify::TypeRegistry::compiled(),
            std::path::Path::new("src/api.rs"),
            Some(bytes),
        );
        assert!(verdict.flags.generated, "the classification still reports it");

        // And nothing about it reaches the tag set, whose whole cost claim is that it
        // never opens a file.
        let named = TagRules::from_names(["vendored", "documentation", "dotfile"])
            .expect("every Name- and Path-tier rule");
        assert!(!named.needs_binding(), "none of these reads a control file");
    }

    /// A plane name has four answers, and each names a different next step.
    ///
    /// The one that matters is the third. An enabled-but-unpromoted rule is spelled right
    /// and enabled, so both of the errors a caller would reach for first are wrong advice:
    /// they would go on checking a spelling that is correct, or re-enabling a rule that is
    /// already on, while the thing that is missing is promotion -- which is the one of the
    /// three that costs, and therefore the one nobody enables by accident.
    #[test]
    fn a_plane_name_is_answered_by_the_step_that_would_fix_it() {
        let enabled = TagRules::from_names(["dotfile"]).expect("enables");

        let typo = enabled.plane_of("dotfil").expect_err("rejected");
        assert!(matches!(typo, TagRuleError::Unknown(_)), "{typo:?}");
        assert!(typo.to_string().contains("available:"), "{typo}");

        let off = TagRules::none().plane_of("dotfile").expect_err("rejected");
        assert!(matches!(off, TagRuleError::NotEnabled { .. }), "{off:?}");

        let unpromoted = enabled.plane_of("dotfile").expect_err("rejected");
        assert!(matches!(unpromoted, TagRuleError::NotPromoted { .. }), "{unpromoted:?}");
        let message = unpromoted.to_string();
        assert!(message.contains("maintains no plane"), "{message}");
        assert!(message.contains("promotes no tag rules"), "{message}");

        let promoted = enabled.with_promoted(["dotfile"]).expect("promotes");
        assert_eq!(
            promoted.plane_of("dotfile").expect("promoted"),
            Promoted(promoted.id_of("dotfile").expect("enabled")),
            "the plane is this set's own bit position",
        );
        assert_eq!(promoted.promoted_names(), vec!["dotfile"]);
    }

    /// Promoting a real rule that is off is not a spelling problem.
    ///
    /// It reported one: `with_promoted` resolved through the enabled set and called every
    /// miss `Unknown`, so promoting `gitignore` without enabling it sent the caller to
    /// check the spelling of a name spelled correctly.
    #[test]
    fn promoting_a_rule_that_is_off_says_to_enable_it_rather_than_to_respell_it() {
        let error = TagRules::from_names(["dotfile"])
            .expect("enables")
            .with_promoted(["gitignore"])
            .expect_err("rejected");
        assert!(matches!(error, TagRuleError::NotEnabled { .. }), "{error:?}");
        assert!(error.to_string().contains("dotfile"), "names what is enabled: {error}");

        let typo = TagRules::from_names(["dotfile"])
            .expect("enables")
            .with_promoted(["dotfil"])
            .expect_err("rejected");
        assert!(matches!(typo, TagRuleError::Unknown(_)), "{typo:?}");
    }

    #[test]
    fn ids_and_names_round_trip_through_the_enabled_set() {
        let rules = TagRules::from_names(["dotfile"]).expect("enables");
        let id = rules.id_of("dotfile").expect("enabled");
        assert_eq!(rules.name_of(id), Some("dotfile"));
        assert_eq!(rules.id_of("gitignore"), None, "not enabled is not the same as unknown");
        assert_eq!(rules.names_of(bits(&rules, ".env")), vec!["dotfile"]);
        assert!(rules.names_of(bits(&rules, "main.rs")).is_empty());
    }
}
