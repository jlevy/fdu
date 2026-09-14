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

/// The scan scope a scan with control observation on records.
///
/// A test about ignore classification states that it wants it rather than inheriting the
/// default, so it keeps testing what it names whatever the default is. Without the
/// `gitignore` build feature this scope observes nothing either.
pub(crate) fn observing_controls() -> crate::ScanScope {
    crate::ScanConfig { read_controls: true, ..crate::ScanConfig::default() }.scope()
}

/// The scan scope a request that turns control observation off records.
pub(crate) fn not_observing_controls() -> crate::ScanScope {
    crate::ScanConfig { read_controls: false, ..crate::ScanConfig::default() }.scope()
}
