//! A scope that admits three kinds, and holds to it however the fourth arrives.
//!
//! Sockets, FIFOs and device nodes occupy directory entries, so fdu counts them by
//! default: a du replacement that silently omitted one would be answering a different
//! question than it was asked. But a consumer whose own model has no name for them --
//! a viewer that can render a file, a directory or a symlink and nothing else -- cannot
//! take that answer without reclassifying a socket as a file, which is a wrong answer
//! rather than a missing one. So `exclude_special` is scope: the entry is not in the
//! index at all, and the identity says so.
//!
//! What is tested here is the *however*. A rule this one enforces at boot and nowhere
//! else is not a scope, it is a first-scan filter, and the difference is observable
//! within one session: `mkfifo` under a live watch, or a file replaced in place by a
//! socket between two refreshes. Each producer of rows -- the walk, reconciliation, the
//! watcher -- is asked separately here, because each learns a kind its own way and each
//! could carry the rule differently.

#![cfg(unix)]

use std::os::unix::net::UnixListener;
use std::path::Path;

use fdu_core::{CachePolicy, EntryKind, IndexHandle, OpenConfig, ScanConfig};

/// Bind a unix socket at `path`, or `None` where the platform will not have it.
///
/// The listener is returned rather than dropped: dropping it closes the descriptor, which
/// on unix leaves the socket file in place, but keeping it alive makes that independent of
/// a platform detail the test is not about. `bind` is the one call here that can fail for
/// a reason unrelated to the rule -- a path over `sun_path`'s 104 bytes on macOS -- so a
/// failure skips rather than fails.
fn socket_at(path: &Path) -> Option<UnixListener> {
    UnixListener::bind(path).ok()
}

fn config(exclude_special: bool) -> OpenConfig {
    OpenConfig {
        scan: ScanConfig { exclude_special, ..ScanConfig::default() },
        policy: CachePolicy::Off,
        ..OpenConfig::default()
    }
}

fn opened(root: &Path, exclude_special: bool) -> IndexHandle {
    let (index, _) = fdu_core::open(root, &config(exclude_special)).expect("open");
    IndexHandle::new(index)
}

/// Every path the index holds under `root`, in path order.
fn paths(handle: &IndexHandle) -> Vec<String> {
    let request = fdu_core::ReadRequest {
        entry_page: Some(fdu_core::EntryPageRequest { limit: u32::MAX, ..Default::default() }),
        ..Default::default()
    };
    handle
        .read(&request)
        .expect("read")
        .entry_page
        .expect("page")
        .rows
        .iter()
        .map(|row| row.path.display().to_string())
        .collect()
}

/// The kinds the index holds, in the order the page returned them.
fn kinds(handle: &IndexHandle) -> Vec<EntryKind> {
    let request = fdu_core::ReadRequest {
        entry_page: Some(fdu_core::EntryPageRequest { limit: u32::MAX, ..Default::default() }),
        ..Default::default()
    };
    handle
        .read(&request)
        .expect("read")
        .entry_page
        .expect("page")
        .rows
        .iter()
        .map(|row| row.entry.kind)
        .collect()
}

/// A fixture with one of each kind, or `None` where a socket cannot be made.
fn fixture() -> Option<(tempfile::TempDir, UnixListener)> {
    let dir = tempfile::tempdir().expect("temp");
    let root = dir.path();
    std::fs::create_dir(root.join("sub")).expect("mkdir");
    std::fs::write(root.join("sub/file.txt"), b"twelve bytes").expect("write");
    std::os::unix::fs::symlink("file.txt", root.join("sub/link")).expect("symlink");
    let listener = socket_at(&root.join("sub/sock"))?;
    Some((dir, listener))
}

/// Kept by default: the socket is an entry of the tree, so it is an entry of the index.
#[test]
fn a_default_scan_records_a_socket_like_any_other_entry() {
    let Some((dir, _listener)) = fixture() else {
        return;
    };
    let handle = opened(dir.path(), false);
    assert!(paths(&handle).contains(&"sub/sock".to_string()), "{:?}", paths(&handle));
    assert!(kinds(&handle).contains(&EntryKind::Other), "the socket is recorded as its own kind");
}

/// Pruned on request, and the three remaining kinds are all that is left.
#[test]
fn a_pruning_scan_records_three_kinds_and_no_others() {
    let Some((dir, _listener)) = fixture() else {
        return;
    };
    let handle = opened(dir.path(), true);
    let held = paths(&handle);
    assert!(!held.contains(&"sub/sock".to_string()), "the socket is out of scope: {held:?}");
    assert!(held.contains(&"sub/file.txt".to_string()), "and nothing else was dropped with it");
    assert!(held.contains(&"sub/link".to_string()), "the symlink is not a special object");
    assert!(held.contains(&"sub".to_string()));

    let seen = kinds(&handle);
    assert!(!seen.contains(&EntryKind::Other), "three kinds only, got {seen:?}");
    assert!(seen.contains(&EntryKind::File) && seen.contains(&EntryKind::Dir));
}

/// A pruned entry is absent rather than counted as a file: the tallies conserve.
///
/// The failure this rules out is the one a viewer would otherwise have to make for itself.
/// Folding a socket into the file count keeps every row renderable and makes every total
/// wrong by one, in a way nothing in the answer reveals.
#[test]
fn a_pruned_special_object_is_not_counted_as_a_regular_file() {
    let Some((dir, _listener)) = fixture() else {
        return;
    };
    let whole = fdu_core::ReadRequest { total: true, ..Default::default() };
    let kept = opened(dir.path(), false).read(&whole).expect("read");
    let pruned = opened(dir.path(), true).read(&whole).expect("read");

    let (kept, pruned) = (kept.total.expect("totals"), pruned.total.expect("totals"));
    assert_eq!(pruned.files, kept.files, "the file count is the same tree either way");
    assert_eq!(pruned.dirs, kept.dirs, "and so is the directory count");
    assert_eq!(
        pruned.others,
        kept.others - 1,
        "exactly the socket left the tally it was already in: {kept:?} against {pruned:?}"
    );
}

/// The scope is part of the identity, so it moves the fingerprint a cache key derives from.
#[test]
fn the_exclusion_is_scope_and_shows_in_the_scan_scope() {
    let Some((dir, _listener)) = fixture() else {
        return;
    };
    let kept = opened(dir.path(), false).scope().expect("scope");
    let pruned = opened(dir.path(), true).scope().expect("scope");

    assert!(!kept.exclude_special, "the default keeps");
    assert!(pruned.exclude_special, "and the request is recorded rather than applied silently");
    assert_ne!(kept, pruned, "two inventories cannot share one identity");
}

/// A socket created after the scan does not enter the index on a refresh.
///
/// The reconciliation half. A rule applied only by the walker would hold until the first
/// refresh and then quietly stop holding, which is the worse failure of the two: the index
/// would be right when nobody had touched the tree and wrong afterwards.
#[test]
fn a_socket_created_after_the_scan_is_not_admitted_by_a_refresh() {
    let Some((dir, _listener)) = fixture() else {
        return;
    };
    let handle = opened(dir.path(), true);
    let Some(_late) = socket_at(&dir.path().join("sub/late")) else {
        return;
    };
    std::fs::write(dir.path().join("sub/ordinary.txt"), b"seen").expect("write");

    let scan = ScanConfig { exclude_special: true, ..ScanConfig::default() };
    fdu_core::scan::reconcile_handle(&handle, &scan, &mut |_| {}).expect("refresh");

    let held = paths(&handle);
    assert!(
        !held.contains(&"sub/late".to_string()),
        "a refresh admits what a scan would: {held:?}"
    );
    assert!(held.contains(&"sub/ordinary.txt".to_string()), "and still admits what a scan would");
}

/// A file replaced in place by a socket loses its row, rather than keeping a stale one.
///
/// The case that makes "skip it" the wrong implementation of "exclude it". The path is
/// still there, so nothing removes the row on the way past; unless the exclusion is a
/// removal, the index reports a twelve-byte file at a path that holds a socket, forever,
/// because no later pass will look at it again.
#[test]
fn a_file_replaced_by_a_socket_loses_the_row_it_had() {
    let Some((dir, _listener)) = fixture() else {
        return;
    };
    let handle = opened(dir.path(), true);
    assert!(paths(&handle).contains(&"sub/file.txt".to_string()), "the file starts recorded");

    std::fs::remove_file(dir.path().join("sub/file.txt")).expect("unlink");
    let Some(_replacement) = socket_at(&dir.path().join("sub/file.txt")) else {
        return;
    };

    let scan = ScanConfig { exclude_special: true, ..ScanConfig::default() };
    fdu_core::scan::reconcile_handle(&handle, &scan, &mut |_| {}).expect("refresh");

    let held = paths(&handle);
    assert!(
        !held.contains(&"sub/file.txt".to_string()),
        "the row describes something that is no longer there: {held:?}"
    );
}

/// The same replacement, reconciled one path at a time rather than by sweeping a directory.
///
/// A separate code path from the listing loop above, and the one a targeted
/// `refresh(path)` takes. It never lists a parent, so the sweep that removes what is no
/// longer in `seen` cannot help it: the removal has to be the exclusion's own doing.
#[test]
fn a_single_path_refresh_removes_a_row_the_scope_no_longer_holds() {
    let Some((dir, _listener)) = fixture() else {
        return;
    };
    let handle = opened(dir.path(), true);

    std::fs::remove_file(dir.path().join("sub/file.txt")).expect("unlink");
    let Some(_replacement) = socket_at(&dir.path().join("sub/file.txt")) else {
        return;
    };

    let scan = ScanConfig { exclude_special: true, ..ScanConfig::default() };
    fdu_core::scan::reconcile_subtree_handle(
        &handle,
        Path::new("sub/file.txt"),
        &scan,
        &mut |_| {},
    )
    .expect("refresh one path");

    let held = paths(&handle);
    assert!(
        !held.contains(&"sub/file.txt".to_string()),
        "one path is enough to drop a row the scope stopped holding: {held:?}"
    );
}

/// Keeping is still keeping: the same replacement under the default scope updates the row.
///
/// The other side of the rule, which a filter that dropped every `Other` unconditionally
/// would fail. Without this, `a_file_replaced_by_a_socket_loses_the_row_it_had` passes for
/// an implementation that excludes special objects always and ignores the flag.
#[test]
fn the_default_scope_records_the_replacement_rather_than_dropping_it() {
    let Some((dir, _listener)) = fixture() else {
        return;
    };
    let handle = opened(dir.path(), false);

    std::fs::remove_file(dir.path().join("sub/file.txt")).expect("unlink");
    let Some(_replacement) = socket_at(&dir.path().join("sub/file.txt")) else {
        return;
    };

    let scan = ScanConfig::default();
    fdu_core::scan::reconcile_handle(&handle, &scan, &mut |_| {}).expect("refresh");

    let request = fdu_core::ReadRequest {
        entry_page: Some(fdu_core::EntryPageRequest { limit: u32::MAX, ..Default::default() }),
        ..Default::default()
    };
    let row = handle
        .read(&request)
        .expect("read")
        .entry_page
        .expect("page")
        .rows
        .into_iter()
        .find(|row| row.path == Path::new("sub/file.txt"))
        .expect("the path is still recorded when nothing excludes it");
    assert_eq!(row.entry.kind, EntryKind::Other, "and it is recorded as what it now is");
}

/// The same rule through the parallel reconciler, which is a separate listing loop.
///
/// `reconcile_handle` shares the index with readers and always walks serially;
/// `reconcile` owns it and fans out across workers when there is more than one. Two loops,
/// each learning a kind its own way, and a rule carried by one of them is a rule that holds
/// or not depending on how the caller asked -- which is what the first draft of this change
/// did, and what this test was written to catch.
#[test]
fn the_parallel_reconciler_excludes_what_the_serial_one_does() {
    let Some((dir, _listener)) = fixture() else {
        return;
    };
    let scan = ScanConfig { exclude_special: true, threads: Some(4), ..ScanConfig::default() };
    let config =
        OpenConfig { scan: scan.clone(), policy: CachePolicy::Off, ..OpenConfig::default() };
    let (mut index, _) = fdu_core::open(dir.path(), &config).expect("open");

    let Some(_late) = socket_at(&dir.path().join("sub/late")) else {
        return;
    };
    std::fs::write(dir.path().join("sub/ordinary.txt"), b"seen").expect("write");
    fdu_core::scan::reconcile(&mut index, &scan, &mut |_| {}).expect("refresh");

    let handle = IndexHandle::new(index);
    let held = paths(&handle);
    assert!(!held.contains(&"sub/late".to_string()), "one rule, both reconcilers: {held:?}");
    assert!(held.contains(&"sub/ordinary.txt".to_string()), "and it admits what it should");
}

/// A file the parallel reconciler finds replaced by a socket loses its row too.
#[test]
fn the_parallel_reconciler_removes_a_row_the_scope_no_longer_holds() {
    let Some((dir, _listener)) = fixture() else {
        return;
    };
    let scan = ScanConfig { exclude_special: true, threads: Some(4), ..ScanConfig::default() };
    let config =
        OpenConfig { scan: scan.clone(), policy: CachePolicy::Off, ..OpenConfig::default() };
    let (mut index, _) = fdu_core::open(dir.path(), &config).expect("open");

    std::fs::remove_file(dir.path().join("sub/file.txt")).expect("unlink");
    let Some(_replacement) = socket_at(&dir.path().join("sub/file.txt")) else {
        return;
    };
    fdu_core::scan::reconcile(&mut index, &scan, &mut |_| {}).expect("refresh");

    let handle = IndexHandle::new(index);
    let held = paths(&handle);
    assert!(
        !held.contains(&"sub/file.txt".to_string()),
        "the row describes something that is no longer there: {held:?}"
    );
}

/// The same rule through the serial walker, which is a separate listing loop again.
///
/// A single-threaded scan is a different function from the fan-out one, and on any host
/// with more than one CPU nothing else here reaches it: every other test in this file gets
/// the parallel walker and would pass with the serial one carrying no rule at all.
#[test]
fn the_serial_walker_excludes_what_the_parallel_one_does() {
    let Some((dir, _listener)) = fixture() else {
        return;
    };
    let config = OpenConfig {
        scan: ScanConfig { exclude_special: true, threads: Some(1), ..ScanConfig::default() },
        policy: CachePolicy::Off,
        ..OpenConfig::default()
    };
    let (index, _) = fdu_core::open(dir.path(), &config).expect("open");
    let handle = IndexHandle::new(index);

    let held = paths(&handle);
    assert!(!held.contains(&"sub/sock".to_string()), "one rule, both walkers: {held:?}");
    assert!(held.contains(&"sub/file.txt".to_string()), "and it admits what it should");
    assert!(!kinds(&handle).contains(&EntryKind::Other), "three kinds only");
}
