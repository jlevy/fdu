//! Kernel-reported process counters, sampled at phase boundaries.
//!
//! # Why this tier exists
//!
//! Application counters say which layer did the work, but they count what your code
//! *thinks* it did. If a library allocates behind your back, or a syscall you believed
//! was one call is three, an application counter will confidently report the wrong
//! number. These come from the kernel and cannot be talked out of the truth.
//!
//! The cost is one small file read per sample, so sample at phase boundaries — before
//! and after a walk, a load, a query — never per event.
//!
//! # What is actually available, which is less than it looks
//!
//! `/proc/self/io` reports `syscr` and `syscw`. They read like syscall counts and are
//! not: they count the read and write families only. Measured directly, a walk over
//! 17,128 directory entries — every one a `getdents64` or a `statx` — moved `syscr` by
//! **30**.
//!
//! The consequence is worth stating plainly, because assuming otherwise produces
//! confident nonsense:
//!
//! | Want to count | Cheap in-process source | Use instead |
//! | --- | --- | --- |
//! | `read`/`write` calls | `syscr`, `syscw` | — |
//! | Bytes through the syscall layer | `rchar`, `wchar` | — |
//! | Bytes actually fetched from a block device | `read_bytes` | — |
//! | `getdents64`, `statx`, `openat` | **none** | application counters, checked against `strace -c` |
//! | Page faults, context switches | `/proc/self/stat` | — |
//!
//! `read_bytes` against `rchar` is the page-cache hit rate, which is the number that
//! decides whether a "cold" measurement was actually cold — worth more than it looks
//! when a hypervisor sits under the guest and `drop_caches` does not reach the disk.
//!
//! # Portability
//!
//! Linux only. Everywhere else every field reads zero and [`Snapshot::available`] is
//! false, so a report can say "not available here" instead of implying the process did
//! nothing.

use std::fmt::Write as _;

/// Kernel-reported counters at one instant.
///
/// Subtract two snapshots with [`Snapshot::since`] to get a phase's activity.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    /// Whether the kernel supplied these numbers. False on platforms without
    /// `/proc/self`, where every other field is zero and means nothing.
    pub available: bool,
    /// `read`-family syscalls. Not directory reads or stats — see the module docs.
    pub read_syscalls: u64,
    /// `write`-family syscalls.
    pub write_syscalls: u64,
    /// Bytes read through the syscall layer, cache hits included.
    pub bytes_through_read: u64,
    /// Bytes written through the syscall layer.
    pub bytes_through_write: u64,
    /// Bytes actually fetched from a block device. Below `bytes_through_read` by
    /// whatever the page cache served.
    pub bytes_from_device: u64,
    /// Minor page faults: memory touched that needed no I/O. Tracks allocation and
    /// first-touch behaviour closely.
    pub minor_faults: u64,
    /// Major page faults: memory touched that needed I/O.
    pub major_faults: u64,
    /// Every syscall the process has made, where the platform will say.
    ///
    /// macOS only, via `proc_pidinfo`. Linux has no unprivileged in-process equivalent —
    /// `syscr` above covers the read family alone — so this stays zero there, and a
    /// report should not present its absence as "no syscalls".
    ///
    /// It is a **total**, not a per-type breakdown, which is still exactly the
    /// denominator a cross-check wants: if the application counters sum to N and the
    /// kernel says N + M, then M syscalls are unaccounted for and something is calling
    /// more than the call sites admit to.
    pub total_syscalls: u64,
}

impl Snapshot {
    /// Read the current values.
    ///
    /// Never fails: an unreadable or absent `/proc` yields an unavailable snapshot
    /// rather than an error, because instrumentation must not be able to break the
    /// program it is watching.
    #[must_use]
    pub fn now() -> Self {
        #[cfg(target_os = "linux")]
        {
            let mut snapshot = Self::default();
            if let Ok(io) = std::fs::read_to_string("/proc/self/io") {
                snapshot.available = true;
                for line in io.lines() {
                    let Some((key, value)) = line.split_once(':') else { continue };
                    let Ok(value) = value.trim().parse::<u64>() else { continue };
                    match key {
                        "syscr" => snapshot.read_syscalls = value,
                        "syscw" => snapshot.write_syscalls = value,
                        "rchar" => snapshot.bytes_through_read = value,
                        "wchar" => snapshot.bytes_through_write = value,
                        "read_bytes" => snapshot.bytes_from_device = value,
                        _ => {}
                    }
                }
            }
            // Fields 10 and 12 of /proc/self/stat are minflt and majflt. They sit after
            // the comm field, which may itself contain spaces and parentheses, so the
            // split has to start after the last ')' rather than at the first space.
            if let Ok(stat) = std::fs::read_to_string("/proc/self/stat") {
                if let Some(rest) = stat.rsplit_once(')').map(|(_, rest)| rest) {
                    let fields: Vec<&str> = rest.split_whitespace().collect();
                    // `rest` starts at field 3 (state), so minflt is index 7 and majflt
                    // index 9.
                    if let Some(value) = fields.get(7).and_then(|v| v.parse().ok()) {
                        snapshot.minor_faults = value;
                        snapshot.available = true;
                    }
                    if let Some(value) = fields.get(9).and_then(|v| v.parse().ok()) {
                        snapshot.major_faults = value;
                    }
                }
            }
            snapshot
        }
        #[cfg(target_os = "macos")]
        {
            let mut info = libc::proc_taskinfo {
                pti_virtual_size: 0,
                pti_resident_size: 0,
                pti_total_user: 0,
                pti_total_system: 0,
                pti_threads_user: 0,
                pti_threads_system: 0,
                pti_policy: 0,
                pti_faults: 0,
                pti_pageins: 0,
                pti_cow_faults: 0,
                pti_messages_sent: 0,
                pti_messages_received: 0,
                pti_syscalls_mach: 0,
                pti_syscalls_unix: 0,
                pti_csw: 0,
                pti_threadnum: 0,
                pti_numrunning: 0,
                pti_priority: 0,
            };
            let size = i32::try_from(size_of::<libc::proc_taskinfo>()).unwrap_or(0);
            // SAFETY: `info` is a fully initialized `proc_taskinfo` and its exact size is
            // passed alongside a pointer to it, which is what `proc_pidinfo` requires.
            // Querying one's own pid needs no privilege. The return value is checked
            // against the requested size before any field is read, because a short
            // return means the kernel filled less than the struct.
            let filled = unsafe {
                libc::proc_pidinfo(
                    // A pid always fits an i32; the fallback simply makes the call fail
                    // rather than silently querying a wrapped pid.
                    i32::try_from(std::process::id()).unwrap_or(-1),
                    libc::PROC_PIDTASKINFO,
                    0,
                    (&raw mut info).cast(),
                    size,
                )
            };
            if filled != size {
                return Self::default();
            }
            Self {
                available: true,
                // These are `i32` in the kernel struct and wrap somewhere past two
                // billion. Saturating rather than wrapping keeps a long run's report
                // implausible-looking instead of quietly wrong.
                total_syscalls: u64::try_from(info.pti_syscalls_unix).unwrap_or(0)
                    + u64::try_from(info.pti_syscalls_mach).unwrap_or(0),
                minor_faults: u64::try_from(info.pti_faults).unwrap_or(0),
                major_faults: u64::try_from(info.pti_pageins).unwrap_or(0),
                ..Self::default()
            }
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Self::default()
        }
    }

    /// This snapshot minus an earlier one: what happened in between.
    ///
    /// Saturating, so a counter that wrapped or a snapshot taken out of order yields
    /// zero rather than an absurd number.
    #[must_use]
    pub fn since(&self, earlier: &Self) -> Self {
        Self {
            available: self.available && earlier.available,
            read_syscalls: self.read_syscalls.saturating_sub(earlier.read_syscalls),
            write_syscalls: self.write_syscalls.saturating_sub(earlier.write_syscalls),
            bytes_through_read: self.bytes_through_read.saturating_sub(earlier.bytes_through_read),
            bytes_through_write: self
                .bytes_through_write
                .saturating_sub(earlier.bytes_through_write),
            bytes_from_device: self.bytes_from_device.saturating_sub(earlier.bytes_from_device),
            minor_faults: self.minor_faults.saturating_sub(earlier.minor_faults),
            major_faults: self.major_faults.saturating_sub(earlier.major_faults),
            total_syscalls: self.total_syscalls.saturating_sub(earlier.total_syscalls),
        }
    }

    /// The share of read bytes the page cache served, if any reads happened.
    ///
    /// `None` when nothing was read, which is different from a hit rate of zero and
    /// should be rendered differently.
    #[must_use]
    pub fn cache_hit_ratio(&self) -> Option<f64> {
        if self.bytes_through_read == 0 {
            return None;
        }
        let served = self.bytes_through_read.saturating_sub(self.bytes_from_device);
        Some(crate::ratio(served, self.bytes_through_read))
    }

    /// Render as grouped lines, matching the application counters' format.
    #[must_use]
    pub fn render(&self) -> String {
        if !self.available {
            return "process counters: not available on this platform\n".to_string();
        }
        let rows = [
            ("process", "read syscalls", self.read_syscalls),
            ("process", "write syscalls", self.write_syscalls),
            ("process", "bytes through read", self.bytes_through_read),
            ("process", "bytes from device", self.bytes_from_device),
            ("process", "minor page faults", self.minor_faults),
            ("process", "major page faults", self.major_faults),
            ("process", "syscalls (total)", self.total_syscalls),
        ];
        let mut out = crate::render_rows(&rows);
        if let Some(ratio) = self.cache_hit_ratio() {
            let _ = writeln!(out, "  {:<20}  {:>13.1}%", "page cache served", ratio * 100.0);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_snapshot_is_readable_and_differences_are_sane() {
        let before = Snapshot::now();
        if !before.available {
            // Neither Linux nor macOS: the contract is that everything reads zero and
            // the report says so rather than implying the process did nothing.
            assert!(before.render().contains("not available"));
            return;
        }

        // Read a file that exists on every platform this runs on. An earlier version
        // read `/proc/self/stat`, which was fine while `available` implied Linux and
        // broke the moment macOS started reporting too — the assumption was in the test
        // rather than in the code, which is the harder kind to notice.
        let scratch = std::env::temp_dir().join("perfkit-snapshot-probe");
        std::fs::write(&scratch, vec![b'x'; 4096]).expect("write scratch file");
        let mut total = 0_usize;
        for _ in 0..64 {
            total += std::fs::read(&scratch).map_or(0, |bytes| bytes.len());
        }
        let _ = std::fs::remove_file(&scratch);
        assert!(total > 0, "the reads actually happened");

        // Which counters move is a platform fact, not an implementation detail, so the
        // assertions differ by platform rather than being weakened to their intersection.
        // The outer gate matches exactly the platforms where `available` can be true, so
        // that `delta` is not an unused binding everywhere else — `warnings = "deny"`
        // makes that a build failure rather than a lint, and only on that platform.
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let delta = Snapshot::now().since(&before);
            #[cfg(target_os = "linux")]
            {
                assert!(delta.read_syscalls > 0, "reading files moves the read counter: {delta:?}");
                assert!(delta.bytes_through_read > 0, "and the byte counter: {delta:?}");
            }
            #[cfg(target_os = "macos")]
            {
                // macOS reports a total syscall count and no per-family breakdown, so
                // the read-specific counters stay zero here by design.
                assert!(delta.total_syscalls > 0, "reads move the total count: {delta:?}");
            }
        }
    }

    #[test]
    fn differences_never_go_negative() {
        let later = Snapshot { available: true, read_syscalls: 5, ..Snapshot::default() };
        let earlier = Snapshot { available: true, read_syscalls: 9, ..Snapshot::default() };
        assert_eq!(later.since(&earlier).read_syscalls, 0, "out-of-order snapshots saturate");
    }

    #[test]
    fn cache_hit_ratio_distinguishes_nothing_read_from_nothing_cached() {
        let nothing = Snapshot { available: true, ..Snapshot::default() };
        assert_eq!(nothing.cache_hit_ratio(), None, "no reads is not a zero hit rate");

        let all_cached = Snapshot {
            available: true,
            bytes_through_read: 1000,
            bytes_from_device: 0,
            ..Snapshot::default()
        };
        assert_eq!(all_cached.cache_hit_ratio(), Some(1.0));

        let none_cached = Snapshot {
            available: true,
            bytes_through_read: 1000,
            bytes_from_device: 1000,
            ..Snapshot::default()
        };
        assert_eq!(none_cached.cache_hit_ratio(), Some(0.0));
    }
}
