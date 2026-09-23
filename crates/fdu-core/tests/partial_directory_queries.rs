//! Public report rows retain scoped completeness after a partial cold walk.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::SystemTime;

use fdu_core::content::AnalysisSet;
use fdu_core::query::{
    Basis, Delivery, ModifiedWindow, Query, Request, Scope, Section, Selection, ViewSpec,
};
use fdu_core::{CachePolicy, EntryKind};

#[test]
fn partial_cold_list_keeps_healthy_age_and_refuses_unknown_modification_times() {
    let root = tempfile::tempdir().expect("root");
    let blocked = root.path().join("ancestor/blocked");
    fs::create_dir_all(&blocked).expect("blocked directory");
    fs::write(blocked.join("unknown.txt"), b"unread").expect("unknown descendant");
    fs::create_dir(root.path().join("healthy")).expect("healthy directory");
    fs::write(root.path().join("healthy/known.txt"), b"known").expect("known descendant");
    fs::set_permissions(&blocked, fs::Permissions::from_mode(0o000)).expect("deny listing");
    let permission_probe = fs::read_dir(&blocked);
    if permission_probe.is_ok() {
        fs::set_permissions(&blocked, fs::Permissions::from_mode(0o700))
            .expect("restore permissions");
        if std::env::var_os("FDU_TEST_ALLOW_NO_PERMISSION_BITS").as_deref()
            == Some(std::ffi::OsStr::new("1"))
        {
            eprintln!(
                "skipped by FDU_TEST_ALLOW_NO_PERMISSION_BITS=1: host permits denied listing"
            );
            return;
        }
        panic!("permission fixture precondition failed: host permits mode-000 directory listing");
    }
    assert_eq!(
        permission_probe.err().expect("denied listing").kind(),
        std::io::ErrorKind::PermissionDenied
    );
    let basis =
        Basis { root: root.path().into(), scope: Scope::default(), content: AnalysisSet::NONE };
    let opened = fdu_core::open(&basis, &Delivery::new(CachePolicy::Off, None));
    fs::set_permissions(&blocked, fs::Permissions::from_mode(0o700)).expect("restore permissions");
    let (index, open_report) = opened.expect("partial answer");
    assert!(!open_report.is_complete());
    let now = SystemTime::now();
    let mut request = Request::new(
        basis,
        Query {
            views: vec![ViewSpec::List],
            format: fdu_core::report_format::Format::Json,
            selection: Selection { kinds: vec![EntryKind::Dir], ..Selection::default() },
            ..Query::default()
        },
        now,
    );
    let answer = fdu_core::query::report(&index, &request, now).expect("list report");
    assert!(!answer.status.complete, "root remains partial");
    let Section::Files { rows, .. } = &answer.sections[0] else {
        panic!("flat list section");
    };
    assert_eq!(rows.len(), 3);
    for path in ["ancestor", "ancestor/blocked"] {
        let row = rows.iter().find(|row| row.path == Path::new(path)).expect("incomplete row");
        assert_eq!(row.complete, Some(false), "{path}");
        assert_eq!(row.age_ns, None, "{path}");
    }
    let healthy = rows.iter().find(|row| row.path == Path::new("healthy")).expect("healthy row");
    assert_eq!(healthy.complete, Some(true));
    assert!(healthy.age_ns.is_some());
    assert_eq!(healthy.files, Some(1));
    for modified in [
        ModifiedWindow { since: Some(0), before: None },
        ModifiedWindow { since: None, before: Some(i64::MAX) },
    ] {
        request.query.selection.modified = modified;
        let filtered = fdu_core::query::report(&index, &request, now).expect("time-bounded list");
        let Section::Files { rows, .. } = &filtered.sections[0] else {
            panic!("flat list section");
        };
        assert_eq!(rows.len(), 1, "unknown times cannot satisfy either bound");
        assert_eq!(rows[0].path, Path::new("healthy"));
    }
}
