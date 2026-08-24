//! Kernel-reported process counters sampled at phase boundaries.
//!
//! One shared snapshot/delta/rendering model serves every platform, while collectors
//! populate only facts their kernel exposes. Linux uses `/proc/self/io` and
//! `/proc/self/stat`; macOS uses `proc_pidinfo(PROC_PIDTASKINFO)`. Missing metrics remain
//! absent and are never rendered as zero.

#![cfg_attr(target_os = "macos", allow(unsafe_code))]

use std::fmt::Write as _;

/// Kernel source that populated a process snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Linux procfs (`/proc/self/io` and `/proc/self/stat`).
    LinuxProc,
    /// macOS `proc_pidinfo(PROC_PIDTASKINFO)`.
    MacOsProcTaskInfo,
}

impl Source {
    const fn group(self) -> &'static str {
        match self {
            Self::LinuxProc => "process (Linux procfs)",
            Self::MacOsProcTaskInfo => "process (macOS proc_taskinfo)",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct MacRaw {
    unix_syscalls: u32,
    mach_syscalls: u32,
    faults: u32,
    pageins: u32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Raw {
    #[default]
    None,
    MacOs(MacRaw),
}

/// Kernel-reported counters at one instant or across one delta.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    source: Option<Source>,
    raw: Raw,
    /// `read`-family syscalls, available from Linux procfs.
    pub read_syscalls: Option<u64>,
    /// `write`-family syscalls, available from Linux procfs.
    pub write_syscalls: Option<u64>,
    /// Bytes read through the syscall layer, including page-cache hits (Linux).
    pub bytes_through_read: Option<u64>,
    /// Bytes written through the syscall layer (Linux).
    pub bytes_through_write: Option<u64>,
    /// Bytes actually fetched from a block device (Linux).
    pub bytes_from_device: Option<u64>,
    /// Minor page faults reported by Linux procfs.
    pub minor_faults: Option<u64>,
    /// Major page faults reported by Linux procfs.
    pub major_faults: Option<u64>,
    /// Total page faults reported by macOS.
    pub page_faults: Option<u64>,
    /// Page-ins reported by macOS.
    pub pageins: Option<u64>,
    /// Total Mach and Unix syscalls reported by macOS.
    pub total_syscalls: Option<u64>,
}

impl Snapshot {
    /// Read the current platform's supported process counters.
    ///
    /// Sampling never fails the observed operation. An unavailable kernel interface
    /// yields an empty snapshot whose renderer states that no source was available.
    #[must_use]
    pub fn now() -> Self {
        #[cfg(target_os = "linux")]
        {
            linux_snapshot()
        }
        #[cfg(target_os = "macos")]
        {
            macos_snapshot()
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Self::default()
        }
    }

    /// Which kernel interface supplied this snapshot.
    #[must_use]
    pub const fn source(&self) -> Option<Source> {
        self.source
    }

    /// Whether at least one supported kernel source was available.
    #[must_use]
    pub const fn is_available(&self) -> bool {
        self.source.is_some()
    }

    /// This snapshot minus an earlier one: work performed between the samples.
    #[must_use]
    pub fn since(&self, earlier: &Self) -> Self {
        let Some(source) = self.source.filter(|source| Some(*source) == earlier.source) else {
            return Self::default();
        };

        if let (Raw::MacOs(later), Raw::MacOs(before)) = (self.raw, earlier.raw) {
            let unix = later.unix_syscalls.wrapping_sub(before.unix_syscalls);
            let mach = later.mach_syscalls.wrapping_sub(before.mach_syscalls);
            let faults = later.faults.wrapping_sub(before.faults);
            let pageins = later.pageins.wrapping_sub(before.pageins);
            return Self {
                source: Some(source),
                raw: Raw::MacOs(MacRaw {
                    unix_syscalls: unix,
                    mach_syscalls: mach,
                    faults,
                    pageins,
                }),
                total_syscalls: Some(u64::from(unix) + u64::from(mach)),
                page_faults: Some(u64::from(faults)),
                pageins: Some(u64::from(pageins)),
                ..Self::default()
            };
        }

        Self {
            source: Some(source),
            read_syscalls: delta(self.read_syscalls, earlier.read_syscalls),
            write_syscalls: delta(self.write_syscalls, earlier.write_syscalls),
            bytes_through_read: delta(self.bytes_through_read, earlier.bytes_through_read),
            bytes_through_write: delta(self.bytes_through_write, earlier.bytes_through_write),
            bytes_from_device: delta(self.bytes_from_device, earlier.bytes_from_device),
            minor_faults: delta(self.minor_faults, earlier.minor_faults),
            major_faults: delta(self.major_faults, earlier.major_faults),
            page_faults: delta(self.page_faults, earlier.page_faults),
            pageins: delta(self.pageins, earlier.pageins),
            total_syscalls: delta(self.total_syscalls, earlier.total_syscalls),
            raw: Raw::None,
        }
    }

    /// Share of syscall-layer read bytes served without block-device reads.
    #[must_use]
    pub fn cache_hit_ratio(&self) -> Option<f64> {
        let through = self.bytes_through_read?;
        if through == 0 {
            return None;
        }
        let device = self.bytes_from_device?;
        Some(super::ratio(through.saturating_sub(device), through))
    }

    /// Render only the metrics supported by this snapshot's source.
    #[must_use]
    pub fn render(&self) -> String {
        let Some(source) = self.source else {
            return "process counters: not available on this platform\n".to_string();
        };
        let group = source.group();
        let mut rows = Vec::new();
        push_metric(&mut rows, group, "read syscalls", self.read_syscalls);
        push_metric(&mut rows, group, "write syscalls", self.write_syscalls);
        push_metric(&mut rows, group, "bytes through read", self.bytes_through_read);
        push_metric(&mut rows, group, "bytes through write", self.bytes_through_write);
        push_metric(&mut rows, group, "bytes from device", self.bytes_from_device);
        push_metric(&mut rows, group, "minor page faults", self.minor_faults);
        push_metric(&mut rows, group, "major page faults", self.major_faults);
        push_metric(&mut rows, group, "page faults", self.page_faults);
        push_metric(&mut rows, group, "page-ins", self.pageins);
        push_metric(&mut rows, group, "syscalls (total)", self.total_syscalls);

        if rows.is_empty() {
            return format!("process counters: {} supplied no supported metrics\n", source.group());
        }
        let mut output = super::render_rows(&rows);
        if let Some(ratio) = self.cache_hit_ratio() {
            let _ = writeln!(output, "  {:<24}  {:>13.1}%", "page cache served", ratio * 100.0);
        }
        output
    }
}

fn delta(later: Option<u64>, earlier: Option<u64>) -> Option<u64> {
    Some(later?.saturating_sub(earlier?))
}

fn push_metric(
    rows: &mut Vec<(&'static str, &'static str, u64)>,
    group: &'static str,
    label: &'static str,
    value: Option<u64>,
) {
    if let Some(value) = value {
        rows.push((group, label, value));
    }
}

#[cfg(target_os = "linux")]
fn linux_snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    if let Ok(io) = std::fs::read_to_string("/proc/self/io") {
        for line in io.lines() {
            let Some((key, value)) = line.split_once(':') else { continue };
            let Ok(value) = value.trim().parse::<u64>() else { continue };
            match key {
                "syscr" => snapshot.read_syscalls = Some(value),
                "syscw" => snapshot.write_syscalls = Some(value),
                "rchar" => snapshot.bytes_through_read = Some(value),
                "wchar" => snapshot.bytes_through_write = Some(value),
                "read_bytes" => snapshot.bytes_from_device = Some(value),
                _ => {}
            }
        }
    }
    if let Ok(stat) = std::fs::read_to_string("/proc/self/stat")
        && let Some(rest) = stat.rsplit_once(')').map(|(_, rest)| rest)
    {
        let fields: Vec<&str> = rest.split_whitespace().collect();
        snapshot.minor_faults = fields.get(7).and_then(|value| value.parse().ok());
        snapshot.major_faults = fields.get(9).and_then(|value| value.parse().ok());
    }
    if snapshot.read_syscalls.is_some()
        || snapshot.write_syscalls.is_some()
        || snapshot.minor_faults.is_some()
        || snapshot.major_faults.is_some()
    {
        snapshot.source = Some(Source::LinuxProc);
    }
    snapshot
}

#[cfg(target_os = "macos")]
fn macos_snapshot() -> Snapshot {
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
    let Ok(size) = i32::try_from(size_of::<libc::proc_taskinfo>()) else {
        return Snapshot::default();
    };
    // SAFETY: `info` is initialized and paired with its exact size. The result must equal
    // that size before any field is used; querying the current process needs no privilege.
    let filled = unsafe {
        libc::proc_pidinfo(
            i32::try_from(std::process::id()).unwrap_or(-1),
            libc::PROC_PIDTASKINFO,
            0,
            (&raw mut info).cast(),
            size,
        )
    };
    if filled != size {
        return Snapshot::default();
    }

    let raw = MacRaw {
        unix_syscalls: signed_counter_bits(info.pti_syscalls_unix),
        mach_syscalls: signed_counter_bits(info.pti_syscalls_mach),
        faults: signed_counter_bits(info.pti_faults),
        pageins: signed_counter_bits(info.pti_pageins),
    };
    Snapshot {
        source: Some(Source::MacOsProcTaskInfo),
        raw: Raw::MacOs(raw),
        total_syscalls: Some(u64::from(raw.unix_syscalls) + u64::from(raw.mach_syscalls)),
        page_faults: Some(u64::from(raw.faults)),
        pageins: Some(u64::from(raw.pageins)),
        ..Snapshot::default()
    }
}

#[cfg(target_os = "macos")]
fn signed_counter_bits(value: i32) -> u32 {
    u32::from_ne_bytes(value.to_ne_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_snapshot_is_readable_and_differences_are_sane() {
        let before = Snapshot::now();
        if !before.is_available() {
            assert!(before.render().contains("not available"));
            return;
        }

        let scratch = tempfile::NamedTempFile::new().expect("scratch file");
        std::fs::write(scratch.path(), vec![b'x'; 4096]).expect("write scratch file");
        let mut total = 0_usize;
        for _ in 0..64 {
            total += std::fs::read(scratch.path()).map_or(0, |bytes| bytes.len());
        }
        assert!(total > 0);

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let difference = Snapshot::now().since(&before);
        #[cfg(target_os = "linux")]
        {
            assert!(difference.read_syscalls.is_some_and(|value| value > 0));
            assert!(difference.bytes_through_read.is_some_and(|value| value > 0));
            assert_eq!(difference.total_syscalls, None);
        }
        #[cfg(target_os = "macos")]
        {
            assert!(difference.total_syscalls.is_some_and(|value| value > 0));
            assert_eq!(difference.read_syscalls, None);
            assert_eq!(difference.bytes_through_read, None);
        }
    }

    #[test]
    fn render_omits_metrics_the_source_does_not_supply() {
        let mac = Snapshot {
            source: Some(Source::MacOsProcTaskInfo),
            total_syscalls: Some(12),
            page_faults: Some(3),
            pageins: Some(1),
            ..Snapshot::default()
        };

        let rendered = mac.render();
        assert!(rendered.contains("syscalls (total)"));
        assert!(rendered.contains("page-ins"));
        assert!(!rendered.contains("read syscalls"));
        assert!(!rendered.contains("bytes from device"));
    }

    #[test]
    fn macos_source_width_wraps_without_becoming_zero() {
        let before_raw = MacRaw {
            unix_syscalls: 0x7fff_fffe,
            mach_syscalls: 0xffff_fffe,
            faults: 0x7fff_ffff,
            pageins: 0xffff_ffff,
        };
        let later_raw = MacRaw {
            unix_syscalls: 0x8000_0001,
            mach_syscalls: 1,
            faults: 0x8000_0001,
            pageins: 2,
        };
        let snapshot = |raw| Snapshot {
            source: Some(Source::MacOsProcTaskInfo),
            raw: Raw::MacOs(raw),
            ..Snapshot::default()
        };

        let difference = snapshot(later_raw).since(&snapshot(before_raw));
        assert_eq!(difference.total_syscalls, Some(6));
        assert_eq!(difference.page_faults, Some(2));
        assert_eq!(difference.pageins, Some(3));
    }

    #[test]
    fn linux_differences_saturate_instead_of_wrapping() {
        let later = Snapshot {
            source: Some(Source::LinuxProc),
            read_syscalls: Some(5),
            ..Snapshot::default()
        };
        let earlier = Snapshot {
            source: Some(Source::LinuxProc),
            read_syscalls: Some(9),
            ..Snapshot::default()
        };
        assert_eq!(later.since(&earlier).read_syscalls, Some(0));
    }
}
