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
}
