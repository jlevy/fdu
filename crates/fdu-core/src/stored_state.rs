//! Stored-state identity: which requests a stored tier may answer.
//!
//! A **tier** is a unit of identity: entries and their roll-ups, `.gitignore` control
//! state, and content records. A **store** holds one or more tiers: a metadata snapshot
//! holds the entry and control tiers, a content sidecar holds the content tier, and a
//! retained [`Index`](crate::Index) holds all three. A store records the identity of every
//! tier it holds, and a stored tier answers a request only when the request's identity
//! for that tier is one it serves.
//!
//! Each identity is the engine fingerprint plus exactly the request parts that change the
//! tier's values. Operational settings such as worker counts, batch sizes, and scan order
//! never appear here, so they can never invalidate a store, and a request part that
//! changes a tier's values always does.

use crate::control::ControlLimits;
use crate::engine_contract::{ScanScope, ScopeIdentity};

/// Version of the fixed `.gitignore` control semantics, the first thing
/// [`ControlTierIdentity::ignore_rules_fingerprint`] hashes.
const IGNORE_RULES_VERSION: u64 = 2;

/// Which entries a scan retains: its depth, symlink, filesystem-boundary, hidden-entry,
/// and special-object settings.
///
/// This is [`ScanScope`] without its type-rules, reducer-set, and ignore-rules
/// fingerprints. It is the same filesystem-admission identity an opened root binds as
/// [`ScopeIdentity`], so the stored-state model names that type rather than keeping a
/// second struct with the same fields that could drift from it.
pub type EntryScope = ScopeIdentity;

/// Identity of an entry tier: the entries a store holds and the roll-ups derived from
/// them.
///
/// `.gitignore` observation is deliberately not part of it. Reading control files changes
/// which entries are classified as ignored, never which entries exist or what they
/// measure, so a store taken with observation on and one taken with it off hold equal
/// entry tiers. That is what lets a later projection serve one from the other.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct EntryTierIdentity {
    /// The engine fingerprint of the build that produced the tier
    /// ([`crate::snapshot::engine_fingerprint`]).
    pub engine: u64,
    /// Which entries a scan retains.
    pub scope: EntryScope,
    /// Identity of the type-classification rules roll-ups were tallied under.
    pub type_rules_fingerprint: u64,
    /// Identity of the enabled reducer set.
    pub reducers_fingerprint: u64,
}

impl EntryTierIdentity {
    /// The entry tier identity this build gives an index of `scope`.
    pub fn of_scope(scope: ScanScope) -> Self {
        Self {
            engine: crate::snapshot::engine_fingerprint(),
            scope: scope.scope_identity(),
            type_rules_fingerprint: scope.type_rules_fingerprint,
            reducers_fingerprint: scope.reducers_fingerprint,
        }
    }
}

/// Identity of a `.gitignore` control tier.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ControlTierIdentity {
    /// No control file was read and no entry was classified, so the tier cannot say
    /// whether any entry is ignored.
    NotObserved,
    /// Control files were read and admitted under `limits`.
    Observed {
        /// The budget and line limit that decided which control files apply.
        limits: ControlLimits,
    },
}

impl ControlTierIdentity {
    /// Whether the tier observed control state.
    pub const fn is_observed(self) -> bool {
        matches!(self, Self::Observed { .. })
    }

    /// The ignore-rules fingerprint a [`ScanScope`] with this control tier carries.
    ///
    /// Zero is reserved for [`Self::NotObserved`], which is what
    /// [`ScanScope::observes_controls`] tests. An observed tier hashes the control
    /// semantics version and each limit in turn with FNV-1a and is never zero, so a scope
    /// taken under one budget or line limit never matches one taken under another.
    pub fn ignore_rules_fingerprint(self) -> u64 {
        const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
        const FNV_PRIME: u64 = 0x100_0000_01b3;
        const UNBOUNDED: u8 = 0;
        const BOUNDED: u8 = 1;

        let Self::Observed { limits } = self else {
            return 0;
        };
        let mut fingerprint = FNV_OFFSET_BASIS;
        let mut mix = |bytes: &[u8]| {
            for byte in bytes {
                fingerprint ^= u64::from(*byte);
                fingerprint = fingerprint.wrapping_mul(FNV_PRIME);
            }
        };
        mix(&IGNORE_RULES_VERSION.to_le_bytes());
        for limit in [limits.budget, limits.line_limit] {
            match limit {
                None => mix(&[UNBOUNDED]),
                Some(limit) => {
                    mix(&[BOUNDED]);
                    mix(&u64::try_from(limit).unwrap_or(u64::MAX).to_le_bytes());
                }
            }
        }
        fingerprint.max(1)
    }
}

/// The identity of every tier a metadata snapshot holds.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct SnapshotIdentity {
    /// The entry tier.
    pub entries: EntryTierIdentity,
    /// The `.gitignore` control tier.
    pub controls: ControlTierIdentity,
}

impl SnapshotIdentity {
    /// The scope an index holding these tiers records.
    pub fn scan_scope(self) -> ScanScope {
        let EntryScope {
            max_depth,
            follow_symlinks,
            one_filesystem,
            hidden_fingerprint,
            exclude_special,
        } = self.entries.scope;
        ScanScope {
            max_depth,
            follow_symlinks,
            one_filesystem,
            hidden_fingerprint,
            exclude_special,
            ignore_rules_fingerprint: self.controls.ignore_rules_fingerprint(),
            type_rules_fingerprint: self.entries.type_rules_fingerprint,
            reducers_fingerprint: self.entries.reducers_fingerprint,
        }
    }
}

/// Identity of a content tier: the per-file analysis records a sidecar holds.
///
/// The entry tier's identity rather than the whole snapshot's, because no metric depends on
/// `.gitignore` observation: every regular file in scope is an analysis candidate whether
/// or not it is ignored. Then the analyzer set the records were produced for, and the
/// analyzers' identities, versions, and options.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ContentTierIdentity {
    /// The entry tier the records were analyzed over.
    pub entries: EntryTierIdentity,
    /// The analyzer set the tier holds records for.
    pub analysis: crate::content::AnalysisSet,
    /// The type rules, options, and analyzer versions the records were produced under.
    pub provenance: crate::content::ContentProvenance,
}

/// How a stored tier answers a request.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Serves {
    /// The stored identity equals the requested one, so the stored tier holds what a cold
    /// run of the request would build.
    Exact,
    /// The stored tier cannot answer the request, which is a miss.
    Refuse,
}

/// Whether a snapshot of the `stored` identity answers a request for `wanted`.
///
/// Equality: a snapshot serves exactly the request whose identity for every tier equals
/// its own. Any relation beyond equality arrives with a projection that yields what a cold
/// run of `wanted` would, and is proven by its own test.
pub fn serves_snapshot(stored: SnapshotIdentity, wanted: SnapshotIdentity) -> Serves {
    if stored == wanted { Serves::Exact } else { Serves::Refuse }
}

// ---- write rules ----
//
// Each tier is written by what an absent item in it means. An absent entry changes every
// roll-up above it, so the entry tier is written only when the pass verified all of it.
// An absent content record is a miss that reads the file again, so records are written one
// at a time, each when the pass verified it.

/// Whether `index`'s entry tier, and the control tier stored with it, may be written.
///
/// Only a complete, fresh index: a snapshot missing an entry would be served as the tree's
/// totals on the next run, and an older complete snapshot is better than that.
pub(crate) fn entries_writable(index: &crate::Index) -> bool {
    index.freshness() == crate::Freshness::Fresh
        && index.state().coverage == crate::engine_contract::Coverage::Complete
}

/// Whether the content record `record` for the file at `path` may be written.
///
/// A record is written when it describes a file this pass verified: the index holds a
/// regular file there whose fingerprint is the record's, the entry was scanned or
/// revalidated by the pass rather than retained from a snapshot, and reading it did not
/// fail. A file the pass verified was listed by its parent, so its subtree was verified
/// down to it. A record under a subtree the pass could not verify describes a retained
/// file nobody checked, so it is left out, and the next run that verifies the file reads
/// it again.
pub(crate) fn content_record_writable(
    index: &crate::Index,
    path: &std::path::Path,
    record: &crate::content::FileAnalysis,
) -> bool {
    use crate::content::CoverageReason;

    if matches!(record.coverage, CoverageReason::IoError | CoverageReason::ChangedDuringRead) {
        return false;
    }
    // A complete, fresh pass verified every entry, and the content tier holds only records
    // that match their live entry, because a metadata change invalidates a file's record
    // and a commit checks the entry it lands on. So only a partial pass asks per file, and
    // the common write pays no lookup per record.
    if entries_writable(index) {
        return true;
    }
    let crate::PathState::Present { kind: crate::EntryKind::File, attrs } = index.path_state(path)
    else {
        return false;
    };
    attrs.fingerprint() == record.fingerprint
        && index.provenance(path).is_some_and(crate::Provenance::is_verified)
}

/// Whether `index`'s content tier may be written beside the store that holds
/// `stored_entries`, the entry tier of the snapshot already stored for its root, if any.
///
/// After a complete pass, always: the snapshot is written with it. After a partial pass,
/// only beside a snapshot of the same entry tier, which the sidecar pairs with. A partial
/// pass under another identity writes no snapshot, so replacing the sidecar would evict the
/// records that pair with the snapshot that stays.
pub(crate) fn content_tier_writable(
    index: &crate::Index,
    stored_entries: Option<EntryTierIdentity>,
) -> bool {
    entries_writable(index) || stored_entries == Some(index.snapshot_identity().entries)
}

// ---- fixed-width codecs ----
//
// Every store writes its tier identities in these encodings. Each is fixed-width and
// canonical: one identity has exactly one encoding, and a decoder refuses any byte that no
// encoder writes, so two stores hold equal identities exactly when their encoded bytes are
// equal. The engine fingerprint is not encoded here: a store writes it once, in its
// prologue beside the magic and format version, and every tier identity it holds shares
// it.

/// Encoded width of an [`EntryTierIdentity`] after its engine fingerprint: the maximum
/// depth, the scope flags, and the hidden-entry, type-rules, and reducer-set fingerprints.
pub(crate) const ENTRY_TIER_BYTES: usize = 8 + 1 + 8 + 8 + 8;

/// Encoded width of a [`ControlTierIdentity`]: the observation tag, then the budget and the
/// line limit, each a tag and eight bytes.
pub(crate) const CONTROL_TIER_BYTES: usize = 1 + 2 * (1 + 8);

/// Encoded width of a [`SnapshotIdentity`] after its engine fingerprint.
pub(crate) const SNAPSHOT_IDENTITY_BYTES: usize = ENTRY_TIER_BYTES + CONTROL_TIER_BYTES;

/// The encoded maximum depth of a scope with no depth bound.
const UNLIMITED_DEPTH: u64 = u64::MAX;

/// Scope flag for symlink-following traversal.
const SCOPE_FOLLOW_SYMLINKS: u8 = 1 << 0;
/// Scope flag for staying on the root filesystem.
const SCOPE_ONE_FILESYSTEM: u8 = 1 << 1;
/// Scope flag for excluding native special objects.
const SCOPE_EXCLUDE_SPECIAL: u8 = 1 << 2;
/// Every scope flag this encoding defines.
const SCOPE_KNOWN_FLAGS: u8 = SCOPE_FOLLOW_SYMLINKS | SCOPE_ONE_FILESYSTEM | SCOPE_EXCLUDE_SPECIAL;

/// Control tier tag for a tier that observed nothing, whose limit fields are all zero.
const CONTROLS_NOT_OBSERVED: u8 = 0;
/// Control tier tag for an observed tier, whose limit fields follow.
const CONTROLS_OBSERVED: u8 = 1;

/// Limit tag for no bound, whose eight value bytes are zero.
const UNBOUNDED_LIMIT: u8 = 0;
/// Limit tag for a bound, whose value is the eight bytes that follow.
const BOUNDED_LIMIT: u8 = 1;

/// Fills a fixed-width encoding field by field.
struct FixedWriter<const N: usize> {
    bytes: [u8; N],
    at: usize,
}

impl<const N: usize> FixedWriter<N> {
    const fn new() -> Self {
        Self { bytes: [0; N], at: 0 }
    }

    fn put(&mut self, field: &[u8]) {
        let end = self.at + field.len();
        self.bytes[self.at..end].copy_from_slice(field);
        self.at = end;
    }

    fn finish(self) -> [u8; N] {
        debug_assert_eq!(self.at, N, "every field of a fixed-width encoding is written");
        self.bytes
    }
}

/// Reads a fixed-width encoding field by field.
struct FixedReader<'a> {
    rest: &'a [u8],
}

impl FixedReader<'_> {
    fn take<const W: usize>(&mut self) -> [u8; W] {
        let (field, rest) =
            self.rest.split_first_chunk::<W>().expect("a fixed-width encoding holds every field");
        self.rest = rest;
        *field
    }

    fn u8(&mut self) -> u8 {
        self.take::<1>()[0]
    }

    fn u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take())
    }
}

impl EntryTierIdentity {
    /// Encode every field but the engine fingerprint, which the store's prologue carries.
    pub(crate) fn encode(self) -> crate::Result<[u8; ENTRY_TIER_BYTES]> {
        let scope = self.scope;
        // A bound of `u64::MAX` would encode as no bound at all and read back as another
        // identity, so it is refused rather than aliased. No tree is that deep.
        let max_depth = match scope.max_depth.map(u64::try_from) {
            None => UNLIMITED_DEPTH,
            Some(Ok(depth)) if depth != UNLIMITED_DEPTH => depth,
            Some(_) => return Err(crate::Error::Snapshot("scan depth overflow".into())),
        };
        let mut flags = 0u8;
        if scope.follow_symlinks {
            flags |= SCOPE_FOLLOW_SYMLINKS;
        }
        if scope.one_filesystem {
            flags |= SCOPE_ONE_FILESYSTEM;
        }
        if scope.exclude_special {
            flags |= SCOPE_EXCLUDE_SPECIAL;
        }
        let mut out = FixedWriter::new();
        out.put(&max_depth.to_le_bytes());
        out.put(&[flags]);
        out.put(&scope.hidden_fingerprint.to_le_bytes());
        out.put(&self.type_rules_fingerprint.to_le_bytes());
        out.put(&self.reducers_fingerprint.to_le_bytes());
        Ok(out.finish())
    }

    /// Decode [`Self::encode`] under the engine fingerprint of the store that holds it, or
    /// `None` for a field no encoder writes.
    pub(crate) fn decode(engine: u64, bytes: &[u8; ENTRY_TIER_BYTES]) -> Option<Self> {
        let mut fields = FixedReader { rest: bytes };
        let max_depth = match fields.u64() {
            UNLIMITED_DEPTH => None,
            depth => Some(usize::try_from(depth).ok()?),
        };
        let flags = fields.u8();
        if flags & !SCOPE_KNOWN_FLAGS != 0 {
            return None;
        }
        let scope = EntryScope {
            max_depth,
            follow_symlinks: flags & SCOPE_FOLLOW_SYMLINKS != 0,
            one_filesystem: flags & SCOPE_ONE_FILESYSTEM != 0,
            hidden_fingerprint: fields.u64(),
            exclude_special: flags & SCOPE_EXCLUDE_SPECIAL != 0,
        };
        Some(Self {
            engine,
            scope,
            type_rules_fingerprint: fields.u64(),
            reducers_fingerprint: fields.u64(),
        })
    }
}

impl ControlTierIdentity {
    /// Encode the observation and, when observed, both limits.
    pub(crate) fn encode(self) -> crate::Result<[u8; CONTROL_TIER_BYTES]> {
        let mut out = FixedWriter::new();
        match self {
            Self::NotObserved => out.put(&[CONTROLS_NOT_OBSERVED; CONTROL_TIER_BYTES]),
            Self::Observed { limits } => {
                out.put(&[CONTROLS_OBSERVED]);
                for limit in [limits.budget, limits.line_limit] {
                    match limit {
                        None => {
                            out.put(&[UNBOUNDED_LIMIT]);
                            out.put(&0u64.to_le_bytes());
                        }
                        Some(limit) => {
                            let limit = u64::try_from(limit).map_err(|_| {
                                crate::Error::Snapshot("control limit overflow".into())
                            })?;
                            out.put(&[BOUNDED_LIMIT]);
                            out.put(&limit.to_le_bytes());
                        }
                    }
                }
            }
        }
        Ok(out.finish())
    }

    /// Decode [`Self::encode`], or `None` for a tag or value no encoder writes.
    pub(crate) fn decode(bytes: &[u8; CONTROL_TIER_BYTES]) -> Option<Self> {
        let mut fields = FixedReader { rest: bytes };
        match fields.u8() {
            CONTROLS_NOT_OBSERVED => {
                bytes[1..].iter().all(|byte| *byte == 0).then_some(Self::NotObserved)
            }
            CONTROLS_OBSERVED => {
                let mut limit = || match (fields.u8(), fields.u64()) {
                    (UNBOUNDED_LIMIT, 0) => Some(None),
                    (BOUNDED_LIMIT, value) => usize::try_from(value).ok().map(Some),
                    _ => None,
                };
                let budget = limit()?;
                let line_limit = limit()?;
                Some(Self::Observed { limits: ControlLimits { budget, line_limit } })
            }
            _ => None,
        }
    }
}

impl SnapshotIdentity {
    /// Encode the entry tier's fields, then the control tier.
    pub(crate) fn encode(self) -> crate::Result<[u8; SNAPSHOT_IDENTITY_BYTES]> {
        let mut out = FixedWriter::new();
        out.put(&self.entries.encode()?);
        out.put(&self.controls.encode()?);
        Ok(out.finish())
    }

    /// Decode [`Self::encode`] under the engine fingerprint of the snapshot that holds it.
    pub(crate) fn decode(engine: u64, bytes: &[u8; SNAPSHOT_IDENTITY_BYTES]) -> Option<Self> {
        let mut fields = FixedReader { rest: bytes };
        Some(Self {
            entries: EntryTierIdentity::decode(engine, &fields.take())?,
            controls: ControlTierIdentity::decode(&fields.take())?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScanConfig;

    fn limits(budget: Option<usize>, line_limit: Option<usize>) -> ControlLimits {
        ControlLimits { budget, line_limit }
    }

    #[test]
    fn a_snapshot_serves_exactly_the_identity_it_was_taken_under() {
        let base = ScanConfig::default().snapshot_identity();
        assert_eq!(serves_snapshot(base, base), Serves::Exact);

        let entries = base.entries;
        let mut refused = vec![
            SnapshotIdentity {
                entries: EntryTierIdentity { engine: entries.engine ^ 1, ..entries },
                ..base
            },
            SnapshotIdentity {
                entries: EntryTierIdentity {
                    type_rules_fingerprint: entries.type_rules_fingerprint ^ 1,
                    ..entries
                },
                ..base
            },
            SnapshotIdentity {
                entries: EntryTierIdentity {
                    reducers_fingerprint: entries.reducers_fingerprint ^ 1,
                    ..entries
                },
                ..base
            },
            SnapshotIdentity { controls: ControlTierIdentity::NotObserved, ..base },
            SnapshotIdentity {
                controls: ControlTierIdentity::Observed { limits: limits(None, None) },
                ..base
            },
        ];
        for config in [
            ScanConfig { max_depth: Some(1), ..ScanConfig::default() },
            ScanConfig { one_filesystem: true, ..ScanConfig::default() },
            ScanConfig { exclude_special: true, ..ScanConfig::default() },
            ScanConfig {
                hidden: Some(std::sync::Arc::new(crate::HiddenPolicy::prune_hidden(
                    std::iter::empty::<std::ffi::OsString>(),
                ))),
                ..ScanConfig::default()
            },
        ] {
            refused.push(config.snapshot_identity());
        }
        for wanted in refused {
            assert_eq!(serves_snapshot(base, wanted), Serves::Refuse, "{wanted:?}");
            assert_eq!(serves_snapshot(wanted, base), Serves::Refuse, "{wanted:?}");
        }
    }

    #[test]
    fn control_settings_change_only_the_control_tier() {
        let base = ScanConfig::default();
        for config in [
            ScanConfig { read_controls: false, ..base.clone() },
            ScanConfig { control_limits: limits(None, Some(1)), ..base.clone() },
            ScanConfig {
                read_controls: false,
                control_limits: limits(Some(1), None),
                ..base.clone()
            },
        ] {
            assert_eq!(config.snapshot_identity().entries, base.snapshot_identity().entries);
            assert_ne!(config.snapshot_identity().controls, base.snapshot_identity().controls);
        }
        // Limits decide nothing when nothing is observed, so they leave no trace.
        let blind = ScanConfig { read_controls: false, ..base.clone() };
        let blind_other_limits = ScanConfig { control_limits: limits(None, None), ..blind.clone() };
        assert_eq!(blind.snapshot_identity(), blind_other_limits.snapshot_identity());
        assert_eq!(blind.control_identity(), ControlTierIdentity::NotObserved);
    }

    #[test]
    fn the_ignore_rules_fingerprint_reserves_zero_for_an_unobserved_tier() {
        assert_eq!(ControlTierIdentity::NotObserved.ignore_rules_fingerprint(), 0);
        let defaults = ControlLimits::default();
        let observed = [
            defaults,
            limits(None, defaults.line_limit),
            limits(defaults.budget, None),
            limits(None, None),
            // The same values in each other's places are a different tier.
            limits(defaults.line_limit, defaults.budget),
        ]
        .map(|limits| ControlTierIdentity::Observed { limits }.ignore_rules_fingerprint());
        for (index, fingerprint) in observed.iter().enumerate() {
            assert_ne!(*fingerprint, 0);
            assert!(!observed[index + 1..].contains(fingerprint), "{observed:?}");
        }
    }

    #[test]
    fn the_scope_an_identity_composes_is_the_one_a_scan_records() {
        for config in [
            ScanConfig::default(),
            ScanConfig { read_controls: false, ..ScanConfig::default() },
            ScanConfig { control_limits: limits(None, None), ..ScanConfig::default() },
            ScanConfig { max_depth: Some(3), exclude_special: true, ..ScanConfig::default() },
        ] {
            let identity = config.snapshot_identity();
            let scope = identity.scan_scope();
            assert_eq!(scope, config.scope());
            assert_eq!(EntryTierIdentity::of_scope(scope), identity.entries);
            assert_eq!(scope.observes_controls(), identity.controls.is_observed());

            let index = crate::Index::new_with_config("/root", &config);
            assert_eq!(index.snapshot_identity(), identity);
            assert_eq!(index.control_identity(), config.control_identity());
        }
    }

    #[test]
    fn each_tier_is_writable_by_its_own_rule() {
        let config = ScanConfig::default();
        let entries = config.snapshot_identity().entries;
        let other =
            ScanConfig { max_depth: Some(2), ..ScanConfig::default() }.snapshot_identity().entries;

        let mut complete = crate::Index::new_with_config("/root", &config);
        complete.set_initial_freshness(true);
        assert!(entries_writable(&complete));
        for stored in [None, Some(entries), Some(other)] {
            assert!(content_tier_writable(&complete, stored), "a complete pass writes: {stored:?}");
        }

        let mut partial = crate::Index::new_with_config("/root", &config);
        partial.set_initial_freshness(false);
        assert!(!entries_writable(&partial), "an absent entry would change totals");
        assert!(content_tier_writable(&partial, Some(entries)), "it pairs with the stored tier");
        assert!(!content_tier_writable(&partial, Some(other)), "it would evict another pair");
        assert!(!content_tier_writable(&partial, None), "nothing stored pairs with it");

        let mut unverified = complete.clone();
        unverified.mark_unverified();
        assert!(!entries_writable(&unverified), "a cache-only index verified nothing");
    }

    /// Identities that differ in every encoded field, including the edges of each range.
    fn identities() -> Vec<SnapshotIdentity> {
        let base = ScanConfig::default().snapshot_identity();
        let entries = base.entries;
        let mut all = vec![base];
        for scope in [
            EntryScope { max_depth: Some(0), ..entries.scope },
            EntryScope { max_depth: Some(7), follow_symlinks: true, ..entries.scope },
            EntryScope { one_filesystem: true, exclude_special: true, ..entries.scope },
            EntryScope { hidden_fingerprint: u64::MAX, ..entries.scope },
        ] {
            all.push(SnapshotIdentity { entries: EntryTierIdentity { scope, ..entries }, ..base });
        }
        all.push(SnapshotIdentity {
            entries: EntryTierIdentity {
                type_rules_fingerprint: 0,
                reducers_fingerprint: u64::MAX,
                ..entries
            },
            ..base
        });
        for controls in [
            ControlTierIdentity::NotObserved,
            ControlTierIdentity::Observed { limits: limits(None, None) },
            ControlTierIdentity::Observed { limits: limits(Some(0), Some(usize::MAX)) },
            ControlTierIdentity::Observed { limits: limits(Some(1), None) },
        ] {
            all.push(SnapshotIdentity { controls, ..base });
        }
        all
    }

    #[test]
    fn every_identity_round_trips_through_its_fixed_width_encoding() {
        let identities = identities();
        let encoded = identities
            .iter()
            .map(|identity| identity.encode().expect("encode"))
            .collect::<Vec<_>>();
        for (identity, bytes) in identities.iter().zip(&encoded) {
            let engine = identity.entries.engine;
            assert_eq!(SnapshotIdentity::decode(engine, bytes), Some(*identity));
            let (entry_bytes, control_bytes) = bytes.split_at(ENTRY_TIER_BYTES);
            assert_eq!(
                EntryTierIdentity::decode(engine, entry_bytes.try_into().expect("width")),
                Some(identity.entries)
            );
            assert_eq!(
                ControlTierIdentity::decode(control_bytes.try_into().expect("width")),
                Some(identity.controls)
            );
        }
        // Canonical: distinct identities never share an encoding.
        for (index, bytes) in encoded.iter().enumerate() {
            assert!(!encoded[index + 1..].contains(bytes), "{:?}", identities[index]);
        }
    }

    #[test]
    fn a_depth_bound_that_would_read_back_as_unbounded_is_refused() {
        let entries = ScanConfig::default().snapshot_identity().entries;
        let aliased = EntryTierIdentity {
            scope: EntryScope { max_depth: usize::try_from(UNLIMITED_DEPTH).ok(), ..entries.scope },
            ..entries
        };
        if aliased.scope.max_depth.is_some() {
            assert!(aliased.encode().is_err());
        }
    }

    #[test]
    fn bytes_no_encoder_writes_are_refused() {
        let base = ScanConfig::default().snapshot_identity();
        let engine = base.entries.engine;
        let entries = base.entries.encode().expect("encode");
        let flags_at = 8;
        let mut unknown_flag = entries;
        unknown_flag[flags_at] |= 1 << 7;
        assert_eq!(EntryTierIdentity::decode(engine, &unknown_flag), None);

        let observed = ControlTierIdentity::Observed { limits: limits(None, Some(1)) };
        let controls = observed.encode().expect("encode");
        let (budget_tag_at, budget_at, line_tag_at) = (1, 2, 10);
        assert_eq!(controls[budget_tag_at], UNBOUNDED_LIMIT);
        assert_eq!(controls[line_tag_at], BOUNDED_LIMIT);
        let mut forged = Vec::new();
        let mut unknown_tag = controls;
        unknown_tag[0] = 2;
        forged.push(unknown_tag);
        let mut unknown_limit_tag = controls;
        unknown_limit_tag[line_tag_at] = 2;
        forged.push(unknown_limit_tag);
        let mut unbounded_with_a_value = controls;
        unbounded_with_a_value[budget_at] = 1;
        forged.push(unbounded_with_a_value);
        let mut unobserved_with_limits = controls;
        unobserved_with_limits[0] = CONTROLS_NOT_OBSERVED;
        forged.push(unobserved_with_limits);
        for bytes in forged {
            assert_eq!(ControlTierIdentity::decode(&bytes), None, "{bytes:?}");
        }
    }

    #[test]
    fn the_engine_fingerprint_comes_from_the_store_not_the_encoding() {
        let identity = ScanConfig::default().snapshot_identity();
        let bytes = identity.encode().expect("encode");
        let other = SnapshotIdentity::decode(!identity.entries.engine, &bytes).expect("decode");
        assert_eq!(other.entries.engine, !identity.entries.engine);
        assert_eq!(serves_snapshot(other, identity), Serves::Refuse);
    }
}
