//! Deterministic public-API fixtures consumed by the machine-format parser gate.

use std::path::PathBuf;

use fdu_core::report_format::{Format, render_cache_status, render_change};
use fdu_core::{CacheScope, CacheState, CacheStatus, Change, ChangeKind, EntryKind};

fn main() {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let [kind, format] = arguments.as_slice() else {
        panic!("expected document kind and machine format");
    };
    let format = match format.as_str() {
        "json" => Format::Json,
        "jsonl" => Format::Jsonl,
        "yaml" => Format::Yaml,
        _ => panic!("expected json, jsonl, or yaml"),
    };
    let mut path = PathBuf::from("a { b } [ c ]\u{85}\u{2028}");
    if kind == "cache" {
        let identity = fdu_core::SnapshotIdentity {
            entries: fdu_core::EntryTierIdentity::of_scope(fdu_core::ScanScope::default()),
            controls: fdu_core::ControlTierIdentity::NotObserved,
        };
        let statuses = [
            CacheStatus {
                path: PathBuf::from("current"),
                bytes: u64::MAX,
                content: Some(fdu_core::ContentStatus {
                    bytes: u64::MAX,
                    state: fdu_core::ContentState::Stale(fdu_core::StaleReason::OlderFormat {
                        version: 1,
                    }),
                }),
                state: CacheState::Current(fdu_core::SnapshotInfo {
                    root: raw_path(),
                    identity,
                    entries: u64::MAX,
                }),
            },
            CacheStatus {
                path: PathBuf::from("stale"),
                bytes: 0,
                content: None,
                state: CacheState::Stale(fdu_core::StaleReason::NewerFormat { version: u32::MAX }),
            },
            CacheStatus {
                path: PathBuf::from("leftover"),
                bytes: 0,
                content: None,
                state: CacheState::Leftover(fdu_core::LeftoverKind::OrphanedContent),
            },
            CacheStatus {
                path: path.clone(),
                bytes: u64::MAX,
                content: None,
                state: CacheState::Unrecognized,
            },
            CacheStatus { path: raw_path(), bytes: 0, content: None, state: CacheState::Absent },
        ];
        println!("{}", render_cache_status(&statuses, CacheScope::All, format));
        return;
    }
    let change_kind = match kind.as_str() {
        "upsert" => ChangeKind::Upsert,
        "remove" => ChangeKind::Remove,
        "invalidate" => ChangeKind::Invalidate,
        "raw" => {
            path = raw_path();
            ChangeKind::Remove
        }
        _ => panic!("unknown document kind"),
    };
    let upsert = change_kind == ChangeKind::Upsert;
    let change = Change {
        path,
        kind: change_kind,
        entry_kind: upsert.then_some(EntryKind::File),
        bytes: upsert.then_some(u64::MAX),
        allocated: upsert.then_some(u64::MAX),
        mtime_ns: upsert.then_some(i64::MIN),
        ignored: upsert.then_some(false),
        clock: u64::MAX,
    };
    println!("{}", render_change(&change, format));
}

#[cfg(unix)]
fn raw_path() -> PathBuf {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    PathBuf::from(OsString::from_vec(vec![b'n', 0x80]))
}

#[cfg(windows)]
fn raw_path() -> PathBuf {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    PathBuf::from(OsString::from_wide(&[u16::from(b'n'), 0xD800]))
}

#[cfg(not(any(unix, windows)))]
fn raw_path() -> PathBuf {
    PathBuf::from("portable")
}
