//! How large each filesystem is and how much of it is free, observed when a report asks.
//!
//! Capacity is a fact about the host at one instant, not about the tree a walk retained, so
//! it is never stored in a snapshot or served from the cache. The serving routes observe it
//! just before they read a report that names `volumes`, and the readers take the observation
//! as an argument, which keeps them pure: a reader given none reports capacity as unknown,
//! never as zero.
//!
//! One call per filesystem: `statfs` on Linux and macOS, which returns capacity and the
//! filesystem type together, `statvfs` on other Unix systems, and `GetDiskFreeSpaceExW`
//! with `GetVolumeInformationW` on Windows. The walk knows where the device changes, so the
//! mount table never names a row; it only decides which paths may be touched
//! ([`MountTable`]).
//!
//! Each call resolves its path, so on an unmounted automount trigger, or on a path through
//! one, it asks for the mount and waits for the answer, as a listing does. A caller
//! therefore names a root, which the user named, and below a root only a path the walk
//! listed where the mount table lists a mount at exactly that path whose topmost
//! filesystem is not an automount trigger, and where the stat that never mounts then finds
//! the device the walk counted ([`crate::query::observe_volumes`]). And it never calls this
//! while it holds an index guard: a filesystem that answers slowly, or never, then delays
//! one report rather than every reader and writer of a root.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

mod mount_table;

pub(crate) use mount_table::{MountAt, MountTable};

#[cfg(unix)]
#[allow(unsafe_code)]
mod capacity_statfs;

#[cfg(windows)]
#[allow(unsafe_code)]
mod capacity_windows;

/// What one filesystem holds and has free.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VolumeCapacity {
    /// The filesystem's size, in bytes.
    pub capacity_bytes: u64,
    /// The bytes an unprivileged writer can still use: what is left before writes fail.
    ///
    /// Not capacity less the filesystem's own used count, which differs on APFS and on a
    /// filesystem that reserves blocks for root: the share in use is
    /// `(capacity - available) / capacity`, what `duf` shows.
    pub available_bytes: u64,
    /// The filesystem type by name (`apfs`, `NTFS`), or on Linux, where the kernel gives a
    /// number, the common ones by the name `mount` gives them (`ext2/3/4`, `xfs`, `tmpfs`,
    /// `overlay`, `btrfs`, `zfs`) and any other as its number (`0x12345678`); `None`
    /// where the platform gives no type.
    pub filesystem: Option<String>,
}

impl VolumeCapacity {
    /// Whether available space on this filesystem is shared with other volumes, so that no
    /// two rows' available bytes may be added: an APFS container, btrfs subvolumes, and ZFS
    /// datasets each draw on one pool.
    pub fn shares_pool(&self) -> bool {
        self.filesystem.as_deref().is_some_and(|name| {
            matches!(name.to_ascii_lowercase().as_str(), "apfs" | "btrfs" | "zfs")
        })
    }
}

/// One filesystem's observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedVolume {
    /// The device the walk identified the filesystem by.
    pub device: u64,
    /// The path the call was made on.
    pub path: PathBuf,
    /// The capacity, or why it could not be observed.
    pub capacity: Result<VolumeCapacity, String>,
}

/// Capacity of every filesystem a report touches, observed at one instant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapacityObservation {
    /// When the calls were made.
    pub observed_at: SystemTime,
    /// One observation per device, in the order the devices were first named.
    pub volumes: Vec<ObservedVolume>,
}

impl CapacityObservation {
    /// The observation of `device`, if it was observed.
    pub fn volume(&self, device: u64) -> Option<&ObservedVolume> {
        self.volumes.iter().find(|volume| volume.device == device)
    }
}

/// Observe the capacity of each filesystem in `volumes`, one call per distinct device on
/// the first path named for it.
///
/// A path on a filesystem that cannot be queried keeps its row with the reason, so a
/// report shows the capacity as unknown rather than leaving the filesystem out.
pub fn observe_capacity<'a>(
    volumes: impl IntoIterator<Item = (u64, &'a Path)>,
) -> CapacityObservation {
    let observed_at = SystemTime::now();
    let mut observed: Vec<ObservedVolume> = Vec::new();
    for (device, path) in volumes {
        if observed.iter().any(|volume| volume.device == device) {
            continue;
        }
        let capacity = observe_path(path).map_err(|error| error.to_string());
        observed.push(ObservedVolume { device, path: path.to_path_buf(), capacity });
    }
    CapacityObservation { observed_at, volumes: observed }
}

#[cfg(unix)]
fn observe_path(path: &Path) -> std::io::Result<VolumeCapacity> {
    capacity_statfs::observe(path)
}

#[cfg(windows)]
fn observe_path(path: &Path) -> std::io::Result<VolumeCapacity> {
    capacity_windows::observe(path)
}

#[cfg(not(any(unix, windows)))]
fn observe_path(_path: &Path) -> std::io::Result<VolumeCapacity> {
    Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "this platform has no capacity query"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_filesystem_holding_a_directory_has_a_capacity_at_least_its_free_space() {
        let root = tempfile::tempdir().expect("root");
        let observation = observe_capacity([(7, root.path()), (7, Path::new("/elsewhere"))]);
        assert_eq!(observation.volumes.len(), 1, "one call per device");
        let volume = observation.volume(7).expect("observed");
        let capacity = volume.capacity.as_ref().expect("a temporary directory can be queried");
        assert!(capacity.capacity_bytes > 0);
        assert!(capacity.available_bytes <= capacity.capacity_bytes);
        // Every platform with a type names one: Linux by number, macOS and Windows by
        // name, so a failing `GetVolumeInformationW` or an empty `f_fstypename` shows here.
        #[cfg(any(target_os = "linux", target_os = "macos", windows))]
        assert!(
            capacity.filesystem.as_deref().is_some_and(|name| !name.is_empty()),
            "{capacity:?}"
        );
        assert!(observation.volume(8).is_none());
    }

    #[test]
    fn a_path_that_cannot_be_queried_keeps_its_row_with_the_reason() {
        let root = tempfile::tempdir().expect("root");
        let missing = root.path().join("absent");
        let observation = observe_capacity([(1, missing.as_path())]);
        assert!(observation.volume(1).expect("kept").capacity.is_err());
    }

    #[test]
    fn pooled_filesystems_are_named_whatever_their_spelling() {
        let pooled = |name: &str| {
            VolumeCapacity { capacity_bytes: 1, available_bytes: 1, filesystem: Some(name.into()) }
                .shares_pool()
        };
        assert!(pooled("apfs") && pooled("btrfs") && pooled("zfs") && pooled("APFS"));
        assert!(!pooled("ext2/3/4") && !pooled("0x12345678") && !pooled("NTFS"));
        let untyped = VolumeCapacity { capacity_bytes: 1, available_bytes: 1, filesystem: None };
        assert!(!untyped.shares_pool());
    }
}
