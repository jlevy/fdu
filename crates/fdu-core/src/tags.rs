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
//! - [`TagTier::Path`] sees the relative path and control files found during the walk.
//!   Cheap, and where gitignore lives.
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

/// Decides a [`TagTier::Name`] rule from a basename.
type NameMatcher = fn(&OsStr) -> bool;

/// The enabled rule set for one index.
///
/// Built once at open and then read-only, the same shape as
/// [`TypeRegistry`](crate::classify::TypeRegistry): an index built under one set is not
/// answerable under another, so the set's fingerprint is part of the scan scope and a
/// snapshot recorded under different rules is not reused.
#[derive(Clone, Debug, Default)]
pub struct TagRules {
    rules: Vec<TagRule>,
    matchers: Vec<Option<NameMatcher>>,
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
            Self::NotEnabled { id, enabled } if enabled.is_empty() => {
                write!(f, "tag rule {id:?} is not enabled: this index evaluates no tag rules")
            }
            Self::NotEnabled { id, enabled } => {
                write!(f, "tag rule {id:?} is not enabled; enabled here: {enabled}")
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
fn catalogue() -> &'static [(TagRule, Option<NameMatcher>)] {
    static CATALOGUE: std::sync::LazyLock<Vec<(TagRule, Option<NameMatcher>)>> =
        std::sync::LazyLock::new(|| {
            vec![(
                TagRule { id: Cow::Borrowed("dotfile"), tier: TagTier::Name },
                Some(is_dotfile as NameMatcher),
            )]
        });
    &CATALOGUE
}

fn available_names() -> String {
    catalogue().iter().map(|(rule, _)| rule.id.as_ref()).collect::<Vec<_>>().join(", ")
}

/// A name beginning with a dot, which is the whole rule.
///
/// Deliberately *not* the hidden-path scope rule. This tags an entry and leaves it in the
/// index with both numbers visible; scope pruning removes it entirely. Different axes, and
/// the spec's own axis test is what separates them.
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
            rules.push(found.0.clone());
            matchers.push(found.1);
        }
        if rules.len() > MAX_TAG_RULES {
            return Err(TagRuleError::TooMany(rules.len()));
        }
        let fingerprint = fingerprint_of(&rules);
        Ok(Self { rules, matchers, fingerprint })
    }

    /// Identity of the enabled set, folded into the scan scope.
    ///
    /// Order-sensitive, because the order is the bit order: the same rules enabled in a
    /// different order produce different bits, and a snapshot's bits are only readable
    /// under the set that wrote them.
    pub fn fingerprint(&self) -> u64 {
        self.fingerprint
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
    pub fn evaluate<'a, F>(&self, name: &OsStr, relative_path: F) -> TagBits
    where
        F: FnOnce() -> Cow<'a, Path>,
    {
        if self.rules.is_empty() {
            return 0;
        }
        // Every rule in the v1 catalogue is Name-tier, so nothing reads a path yet. The
        // first Path-tier rule calls this; the closure exists so that adding one is a
        // change to this function rather than to every insert site in the index.
        let _ = relative_path;
        let mut bits: TagBits = 0;
        for (index, matcher) in self.matchers.iter().enumerate() {
            if matcher.is_some_and(|decide| decide(name)) {
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
                        enabled: self
                            .rules
                            .iter()
                            .map(|rule| rule.id.as_ref())
                            .collect::<Vec<_>>()
                            .join(", "),
                    });
                }
                return Err(TagRuleError::Unknown(name.to_string()));
            };
            mask |= 1 << id;
        }
        Ok(mask)
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

/// FNV-1a over the enabled names and tiers, in order.
fn fingerprint_of(rules: &[TagRule]) -> u64 {
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
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    fn bits(rules: &TagRules, name: &str) -> TagBits {
        rules.evaluate(&OsString::from(name), || Cow::Borrowed(Path::new(name)))
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
