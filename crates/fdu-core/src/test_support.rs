//! Helpers shared by the crate's unit tests.

/// Whether this process is actually subject to Unix permission bits.
///
/// Several fixtures induce a real `EACCES` by removing read or search permission. A
/// process with `CAP_DAC_OVERRIDE` — anything running as root, which is the normal case
/// inside a container or dev image — reads the directory anyway, so those fixtures
/// silently stop testing what they claim to and fail on the assertion that the error
/// happened at all. Probing the capability is better than probing the user id: it asks
/// the question the fixture actually depends on, and it needs no libc on any platform.
#[cfg(unix)]
pub(crate) fn permission_bits_are_enforced() -> bool {
    use std::os::unix::fs::PermissionsExt;

    let Ok(directory) = tempfile::tempdir() else {
        return false;
    };
    let path = directory.path().join("unreadable");
    if std::fs::write(&path, b"probe").is_err() {
        return false;
    }
    if std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).is_err() {
        return false;
    }
    std::fs::read(&path).is_err()
}

/// Establish the permission-fixture precondition or require an explicit host opt-out.
///
/// Returning `false` is a deliberate skip: the operator named this host as unable to
/// represent the fixture. Without that declaration, a green test must mean its
/// permission assertions actually ran.
#[cfg(unix)]
pub(crate) fn require_permission_bits() -> bool {
    if permission_bits_are_enforced() {
        return true;
    }
    if std::env::var_os("FDU_TEST_ALLOW_NO_PERMISSION_BITS").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        eprintln!(
            "skipped by FDU_TEST_ALLOW_NO_PERMISSION_BITS=1: this host does not enforce Unix \
             permission bits for the test process"
        );
        return false;
    }
    panic!(
        "permission fixture precondition failed: this process can read a mode-000 file; \
         run on a host that enforces Unix permission bits, or explicitly opt out with \
         FDU_TEST_ALLOW_NO_PERMISSION_BITS=1"
    );
}

/// The scan scope a scan with control observation on records.
///
/// A test about ignore classification states that it wants it rather than inheriting the
/// default, so it keeps testing what it names whatever the default is.
pub(crate) fn observing_controls() -> crate::ScanScope {
    crate::ScanConfig { read_controls: true, ..crate::ScanConfig::default() }.scope()
}

/// The scan scope a request that turns control observation off records.
pub(crate) fn not_observing_controls() -> crate::ScanScope {
    crate::ScanConfig { read_controls: false, ..crate::ScanConfig::default() }.scope()
}

/// The request a test makes of an index it just built: the index's own basis, the query,
/// and a fixed instant, so repeated pure reads have the same age reference.
///
/// A test names the query it is about, which is the axis it varies; the basis is whatever
/// the fixture index holds, so [`Request::validate_read`](crate::query::Request::validate_read)
/// admits it and the test is about the report rather than about composing a request. A test
/// *about* a refusal builds its own mismatched basis instead.
pub(crate) fn read_of(index: &crate::Index, query: crate::query::Query) -> crate::query::Request {
    crate::query::Request::new(crate::query::Basis::held_by(index), query, std::time::UNIX_EPOCH)
}

/// Return once the filesystem holding `root` has allocated the blocks of every file a
/// fixture wrote there, so each later walk of the fixture reads the same allocated bytes.
///
/// ext4, the usual filesystem of a Linux temporary directory, allocates a buffered write's
/// blocks only when it writes the data back (delayed allocation). Until then a stat counts
/// the reservation, and afterwards the allocation, but writeback drops the one before it
/// charges the other, so a stat between the two reads a one-block file as `st_blocks` 0.
/// The kernel decides when writeback runs, so a differential that walks one fixture twice
/// and compares allocated bytes can come out a block apart whenever writeback lands inside
/// a walk. The control-case summary differential failed that way in CI: every count and
/// apparent byte equal, allocated bytes 4096 apart. `sync -f` is `syncfs`, which returns
/// once the filesystem is written back, so no block is left to move. It is a command, as
/// `mkfifo` is in these tests, because the workspace keeps unsafe code to the platform
/// readers; the short flag is the one busybox also accepts, so a musl test host can run
/// it.
///
/// Elsewhere this does nothing. Windows reads a file's length as its allocation, and the
/// other Unix hosts have no `syncfs`, while POSIX lets `sync` return before the writes it
/// schedules are complete.
pub(crate) fn settle_allocations(root: &std::path::Path) {
    #[cfg(target_os = "linux")]
    {
        let status =
            std::process::Command::new("sync").arg("-f").arg(root).status().expect("run sync");
        assert!(status.success(), "sync -f exited with {status}");
    }
    #[cfg(not(target_os = "linux"))]
    let _ = root;
}

/// Whether the filesystem under `dir` resolves a name in another case to the stored entry,
/// as APFS and NTFS do by default and an ext4 casefold directory does.
pub(crate) fn resolves_case_insensitively(dir: &std::path::Path) -> bool {
    let probe = dir.join("CaseProbe");
    std::fs::write(&probe, b"probe").expect("case probe");
    let insensitive = std::fs::symlink_metadata(dir.join("caseprobe")).is_ok();
    std::fs::remove_file(probe).expect("remove case probe");
    insensitive
}

/// How a test's control lookups resolve a case variant of `.gitignore` under its root.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CaseLookups {
    /// As the host filesystem resolves them.
    Host,
    /// As a case-insensitive directory would, through
    /// [`crate::scan::install_case_folding_control_lookup`].
    Folded,
}

impl CaseLookups {
    /// The modes a case-variant control test runs under `dir` on this host, each with
    /// whether a lookup there resolves `.gitignore` to a stored `.GITIGNORE`.
    ///
    /// The host always runs: on a case-insensitive volume (the macOS and Windows runners)
    /// it is the real rule, and on a case-sensitive one the negative. Folded runs only on a
    /// case-sensitive host, where it is the one way to reach the rule's other branch; a
    /// case-insensitive host already resolves as it would. The mode is printed, so a run
    /// says which branch it exercised for real.
    pub(crate) fn on_this_host(dir: &std::path::Path) -> Vec<(Self, bool)> {
        if resolves_case_insensitively(dir) {
            eprintln!(
                "the temporary directory is case-insensitive: the host exercises a case-variant \
                 control for real, and folded lookups are not needed"
            );
            vec![(Self::Host, true)]
        } else {
            eprintln!(
                "the temporary directory is case-sensitive: a case-variant control is exercised \
                 through folded lookups, and the host is the negative"
            );
            vec![(Self::Host, false), (Self::Folded, true)]
        }
    }

    /// Resolve control lookups under `root` in this mode until the guard drops.
    pub(crate) fn install(self, root: &std::path::Path) -> Option<crate::scan::CaseFoldingGuard> {
        match self {
            Self::Host => None,
            Self::Folded => Some(crate::scan::install_case_folding_control_lookup(root)),
        }
    }
}
