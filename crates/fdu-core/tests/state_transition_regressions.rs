//! Regression coverage for retained-state transitions across reconciliation paths.

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::SystemTime;

use fdu_core::content::{AnalysisRequest, AnalysisSet, analyze_index};
use fdu_core::query::{Basis, Query, Request, report};
use fdu_core::scan::{ScanConfig, reconcile, reconcile_handle, reconcile_subtree, scan_into_index};
use fdu_core::{CachePolicy, Coverage, IndexHandle, OpenPath};

#[test]
fn control_read_failure_has_identical_cold_serial_and_parallel_rules() {
    for (subtree, threads) in [(".gitignore", 1), ("", 1), ("", 2)] {
        let root = tempfile::tempdir().expect("root");
        let control = root.path().join(".gitignore");
        fs::write(&control, b"*.log\n").expect("control");
        fs::write(root.path().join("a.log"), b"log").expect("file");
        let config = ScanConfig { threads: Some(threads), ..ScanConfig::default() };
        let (mut warm, baseline) = scan_into_index(root.path(), &config).expect("baseline");
        assert!(baseline.is_complete());
        fs::write(&control, b"# none\n").expect("change rules");
        fs::set_permissions(&control, fs::Permissions::from_mode(0o000)).expect("chmod");
        let denied = fs::read(&control).expect_err("fixture must enforce permission bits");
        assert_eq!(denied.kind(), std::io::ErrorKind::PermissionDenied);

        let result = reconcile_subtree(&mut warm, Path::new(subtree), &config, &mut |_| {});
        let cold = scan_into_index(root.path(), &config);
        fs::set_permissions(&control, fs::Permissions::from_mode(0o644)).expect("restore");

        let result = result.expect("partial reconciliation");
        let (cold, cold_report) = cold.expect("cold partial");
        assert!(!result.is_complete());
        assert!(!cold_report.is_complete());
        assert_eq!(
            warm.is_ignored(Path::new("a.log")).expect("warm controls"),
            cold.is_ignored(Path::new("a.log")).expect("cold controls"),
            "subtree={subtree:?}, threads={threads}"
        );
    }
}

#[test]
fn warm_partial_report_retains_reconciliation_error() {
    let root = tempfile::tempdir().expect("root");
    let cache = tempfile::tempdir().expect("cache");
    let blocked = root.path().join("blocked");
    fs::create_dir(&blocked).expect("directory");
    fs::write(blocked.join("file.txt"), b"held").expect("file");
    let config = ScanConfig { threads: Some(1), ..ScanConfig::default() };
    let (index, _) = scan_into_index(root.path(), &config).expect("baseline");
    let snapshot = cache.path().join("snapshot.fdu");
    fdu_core::snapshot::save(&index, &snapshot).expect("save");
    fs::set_permissions(&blocked, fs::Permissions::from_mode(0o000)).expect("chmod");
    let denied = fs::read_dir(&blocked).expect_err("fixture must enforce directory permissions");
    assert_eq!(denied.kind(), std::io::ErrorKind::PermissionDenied);

    let opened = fdu_core::open(
        &Basis {
            root: root.path().to_path_buf(),
            scope: config.into(),
            content: AnalysisSet::NONE,
        },
        &fdu_core::query::Delivery::new(CachePolicy::ReadOnly, Some(snapshot)),
    );
    fs::set_permissions(&blocked, fs::Permissions::from_mode(0o755)).expect("restore");

    let (index, opened) = opened.expect("partial warm open");
    assert_eq!(opened.path_taken, OpenPath::WarmRevalidate);
    assert_eq!(opened.scan.errors.len(), 1);
    let request = Request::new(Basis::held_by(&index), Query::default(), SystemTime::now());
    let answer = report(&index, &request, SystemTime::now()).expect("report");
    assert!(!answer.status.complete);
    assert_eq!(answer.status.errors.len(), 1);
    assert_eq!(answer.status.errors[0].path.as_deref(), Some(Path::new("blocked")));
}

#[test]
fn cold_partial_can_become_complete_after_direct_and_shared_root_retries() {
    for shared in [false, true] {
        let root = tempfile::tempdir().expect("root");
        let blocked = root.path().join("blocked");
        fs::create_dir(&blocked).expect("directory");
        fs::write(blocked.join("file.txt"), b"held").expect("file");
        let config = ScanConfig { threads: Some(1), ..ScanConfig::default() };
        fs::set_permissions(&blocked, fs::Permissions::from_mode(0o000)).expect("chmod");
        let denied =
            fs::read_dir(&blocked).expect_err("fixture must enforce directory permissions");
        assert_eq!(denied.kind(), std::io::ErrorKind::PermissionDenied);
        let scanned = scan_into_index(root.path(), &config);
        fs::set_permissions(&blocked, fs::Permissions::from_mode(0o755)).expect("restore");
        let (mut index, initial) = scanned.expect("cold partial");
        assert!(!initial.is_complete());

        if shared {
            let handle = IndexHandle::new(index);
            let retry = reconcile_handle(&handle, &config, &mut |_| {}).expect("shared retry");
            assert!(retry.is_complete());
            index = handle.snapshot().expect("snapshot");
        } else {
            let retry = reconcile(&mut index, &config, &mut |_| {}).expect("direct retry");
            assert!(retry.is_complete());
        }
        let request = Request::new(Basis::held_by(&index), Query::default(), SystemTime::now());
        let status = fdu_core::query::TreeStatus::of(&index, &request);
        assert_eq!(status.coverage, Coverage::Complete, "shared={shared}");
        assert!(status.errors.is_empty());
    }
}

#[test]
fn content_coverage_tracks_mutation_addition_and_pure_removal() {
    let root = tempfile::tempdir().expect("root");
    let path = root.path().join("file.txt");
    fs::write(&path, b"one\n").expect("file");
    let config = ScanConfig { threads: Some(1), ..ScanConfig::default() };
    let (mut index, _) = scan_into_index(root.path(), &config).expect("baseline");
    let analysis = AnalysisRequest { profile: AnalysisSet::LINES_ONLY, workers: 1 };
    assert!(analyze_index(&mut index, analysis).is_complete());

    fs::write(&path, b"two\nlines\n").expect("change");
    reconcile(&mut index, &config, &mut |_| {}).expect("reconcile mutation");
    assert_content_pending(&index);
    assert_metadata_complete(&index);
    assert!(analyze_index(&mut index, analysis).is_complete());
    assert_content_complete(&index);

    fs::write(root.path().join("added.txt"), b"added\n").expect("add");
    reconcile(&mut index, &config, &mut |_| {}).expect("reconcile addition");
    assert_content_pending(&index);
    assert!(analyze_index(&mut index, analysis).is_complete());
    assert_content_complete(&index);

    fs::remove_file(root.path().join("added.txt")).expect("remove");
    reconcile(&mut index, &config, &mut |_| {}).expect("reconcile removal");
    assert_content_complete(&index);
}

fn content_request(index: &fdu_core::Index) -> Request {
    Request::new(Basis::held_by(index), Query::default(), SystemTime::now())
}

fn assert_content_pending(index: &fdu_core::Index) {
    let answer = report(index, &content_request(index), SystemTime::now()).expect("report");
    assert!(!answer.status.complete);
    assert!(answer.status.errors.is_empty(), "pending analysis is not an I/O failure");
    assert_eq!(answer.provenance.freshness, fdu_core::Freshness::Partial);
}

fn assert_content_complete(index: &fdu_core::Index) {
    let answer = report(index, &content_request(index), SystemTime::now()).expect("report");
    assert!(answer.status.complete);
    assert_eq!(answer.provenance.freshness, fdu_core::Freshness::Fresh);
}

fn assert_metadata_complete(index: &fdu_core::Index) {
    let mut basis = Basis::held_by(index);
    basis.content = AnalysisSet::NONE;
    let request = Request::new(basis, Query::default(), SystemTime::now());
    assert!(fdu_core::query::TreeStatus::of(index, &request).complete);
    assert_eq!(
        fdu_core::query::ReportProvenance::of(index, AnalysisSet::NONE, SystemTime::now())
            .freshness,
        fdu_core::Freshness::Fresh
    );
}
