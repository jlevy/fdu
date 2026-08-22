//! Differential tests: every parallel producer must agree with the serial reference.
//!
//! The scan and reconciliation fast paths are operational, not semantic: worker depth,
//! bulk metadata, and immutable-baseline waves are supposed to change how fast the same
//! observations are produced, never which ones. These tests hold them to that by
//! comparing complete index images, roll-up state, and reported statistics against the
//! one-worker walker over a fixture built to hit the awkward cases.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use fdu_core::scan::{reconcile, scan_into_index};
use fdu_core::{EntryId, Index, ScanConfig, ScanOrder};

/// Worker counts every differential assertion is repeated at.
///
/// One is the reference. Two and four cross the wave-worker cap; sixteen exceeds it and
/// proves the clamp does not change results.
const WORKER_COUNTS: [usize; 4] = [1, 2, 4, 16];

/// Worker counts for cases that rebuild the fixture once per count.
///
/// The endpoints and one interior value are what distinguish serial, sub-cap, and
/// over-cap behavior; running the fourth as well only pays for another fixture on the
/// slowest runner in the matrix.
const SAMPLED_WORKER_COUNTS: [usize; 3] = [1, 4, 16];

/// A complete, order-independent image of an index.
///
/// Entry lines carry every stat-tier field the cache fingerprints, so a divergence in
/// device, inode, or either byte total fails here rather than surfacing later as a
/// cache that lies. Directory lines carry roll-up state, which is where an
/// out-of-order or double-counted apply would show up.
fn image(index: &Index) -> Vec<String> {
    let mut lines = Vec::new();
    let mut stack = vec![EntryId::ROOT];
    while let Some(id) = stack.pop() {
        let path = index.path_of(id).expect("live id");
        let kind = index.kind_of(id).expect("live id");
        let attrs = *index.attrs_of(id).expect("live id");
        let mut line = format!(
            "{}|{kind:?}|size={}|alloc={}|mtime={}|ctime={}|ino={}|dev={}",
            path.display(),
            attrs.size,
            attrs.allocated,
            attrs.mtime_ns,
            attrs.ctime_ns,
            attrs.inode,
            attrs.dev,
        );
        if let Some(rollup) = index.rollup_of(id) {
            let by_ext: BTreeMap<_, _> = rollup
                .by_ext
                .into_iter()
                .map(|(ext, tally)| (ext, (tally.files, tally.bytes, tally.allocated)))
                .collect();
            let _ = write!(
                line,
                "|rollup(files={},dirs={},bytes={},alloc={},newest={},ext={by_ext:?})",
                rollup.files, rollup.dirs, rollup.bytes, rollup.allocated, rollup.newest_mtime_ns,
            );
            for (_, child) in index.children_of(id).expect("live id") {
                stack.push(child);
            }
        }
        lines.push(line);
    }
    lines.sort();
    lines
}

/// Report the first differing line, so a failure names the entry rather than the tree.
fn assert_same_image(reference: &[String], candidate: &[String], context: &str) {
    if reference == candidate {
        return;
    }
    let mismatch = match reference.iter().zip(candidate).find(|(left, right)| left != right) {
        Some((left, right)) => format!("\n  serial:   {left}\n  parallel: {right}"),
        None => {
            format!("\n  serial has {} entries, parallel has {}", reference.len(), candidate.len())
        }
    };
    panic!("{context}: parallel index diverged from the serial reference{mismatch}");
}

fn config(threads: usize) -> ScanConfig {
    ScanConfig { threads: Some(threads), ..ScanConfig::default() }
}

fn write(path: &Path, contents: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, contents).expect("write file");
}

/// Children in the widest directory when a case needs batch splitting.
///
/// Past the engine's default observation batch, so one directory's entries are
/// published across several batches rather than one.
const WIDE_DIRECTORY_FILES: usize = 1200;

/// Children in the widest directory when a case rebuilds the fixture many times.
///
/// Still several claims wide, but the batch-splitting property belongs to the cold-scan
/// case that builds the tree once; paying for it in every cell of a mutation matrix
/// buys nothing and dominates the suite's runtime.
const NARROW_DIRECTORY_FILES: usize = 150;

/// A fixture built for the cases a fast path is most likely to get wrong.
///
/// Wide directories cross a claim boundary, deep chains outlive one wave, and the
/// awkward names, empty files, and links are the entries a bulk or batched reader is
/// most likely to mis-stage.
fn build_fixture(root: &Path, wide_files: usize) {
    write(&root.join("README.md"), b"top level");
    write(&root.join("empty"), b"");

    // Wide: many more children than one claim holds.
    for i in 0..wide_files {
        write(&root.join("wide").join(format!("file-{i:04}.dat")), format!("{i}").as_bytes());
    }

    // Deep: a chain far longer than the claim size, so a subtree spans many waves.
    let mut deep = root.join("deep");
    for level in 0..40 {
        deep = deep.join(format!("level-{level}"));
        write(&deep.join("leaf.txt"), format!("depth {level}").as_bytes());
    }

    // Many small directories, which is the topology the wave scheduler exists for.
    for outer in 0..12 {
        for inner in 0..6 {
            let dir = root.join("fanout").join(format!("d{outer:02}")).join(format!("s{inner}"));
            write(&dir.join("a.rs"), b"fn main() {}");
            write(&dir.join("b.json"), b"{}");
            fs::create_dir_all(dir.join("empty-dir")).expect("empty dir");
        }
    }

    // Names a batching reader has to keep byte-exact.
    write(&root.join("names").join("space in name.txt"), b"spaces");
    write(&root.join("names").join("unicode-\u{2603}-\u{1f600}.txt"), b"snowman");
    write(&root.join("names").join(".hidden"), b"hidden");
    write(&root.join("names").join("no-extension"), b"none");
    write(&root.join("names").join("trailing.dots..."), b"dots");
    write(&root.join("names").join("long-".repeat(40)), b"long");

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        // A name that is not valid UTF-8 must survive the round trip byte for byte,
        // where the filesystem allows one at all. APFS rejects the bytes outright with
        // EILSEQ, so this is the one fixture entry that is allowed not to appear.
        let raw = std::ffi::OsStr::from_bytes(b"invalid-\xff\xfe-name");
        if fs::write(root.join("names").join(raw), b"non-utf8").is_err() {
            eprintln!("fixture: this filesystem rejects non-UTF-8 names; that case is absent");
        }

        fs::create_dir_all(root.join("links")).expect("links dir");
        std::os::unix::fs::symlink("../README.md", root.join("links").join("to-readme"))
            .expect("symlink");
        std::os::unix::fs::symlink("nowhere", root.join("links").join("dangling"))
            .expect("dangling symlink");
        std::os::unix::fs::symlink("..", root.join("links").join("to-parent"))
            .expect("parent symlink");
        fs::hard_link(root.join("README.md"), root.join("links").join("hardlink"))
            .expect("hard link");
    }

    // A sparse-ish large file: apparent and allocated size should disagree.
    let sparse = root.join("sparse.bin");
    let file = fs::File::create(&sparse).expect("create sparse");
    file.set_len(8 * 1024 * 1024).expect("extend");
    drop(file);
}

/// One named mutation shape applied between the baseline scan and reconciliation.
type Mutation = (&'static str, fn(&Path));

/// Mutation shapes applied between the baseline scan and reconciliation.
fn mutations() -> Vec<Mutation> {
    vec![
        ("no-op", |_root: &Path| {}),
        ("touch-one-file", |root: &Path| {
            write(&root.join("README.md"), b"top level, edited");
        }),
        ("remove-one-file", |root: &Path| {
            fs::remove_file(root.join("empty")).expect("remove");
        }),
        ("add-files-and-dirs", |root: &Path| {
            write(&root.join("added").join("new.txt"), b"new");
            write(&root.join("wide").join("file-9999.dat"), b"appended");
            fs::create_dir_all(root.join("added").join("empty")).expect("dir");
        }),
        ("remove-a-whole-subtree", |root: &Path| {
            fs::remove_dir_all(root.join("fanout").join("d00")).expect("remove subtree");
        }),
        ("replace-a-directory-with-a-file", |root: &Path| {
            fs::remove_dir_all(root.join("fanout").join("d01")).expect("remove subtree");
            write(&root.join("fanout").join("d01"), b"now a file");
        }),
        ("replace-a-file-with-a-directory", |root: &Path| {
            fs::remove_file(root.join("README.md")).expect("remove file");
            write(&root.join("README.md").join("inner.txt"), b"now a directory");
        }),
        ("rename-a-subtree", |root: &Path| {
            fs::rename(root.join("fanout").join("d02"), root.join("fanout").join("renamed"))
                .expect("rename");
        }),
        ("churn-every-wide-file", |root: &Path| {
            for i in 0..NARROW_DIRECTORY_FILES {
                write(
                    &root.join("wide").join(format!("file-{i:04}.dat")),
                    format!("churned {i}").as_bytes(),
                );
            }
        }),
        ("truncate-the-deep-chain", |root: &Path| {
            fs::remove_dir_all(root.join("deep").join("level-0").join("level-1"))
                .expect("remove deep");
        }),
        ("grow-the-deep-chain", |root: &Path| {
            let mut deep = root.join("deep");
            for level in 0..40 {
                deep = deep.join(format!("level-{level}"));
            }
            for extra in 40..56 {
                deep = deep.join(format!("level-{extra}"));
                write(&deep.join("leaf.txt"), format!("depth {extra}").as_bytes());
            }
        }),
    ]
}

fn fixture_root(label: &str, wide_files: usize) -> (tempfile::TempDir, PathBuf) {
    let dir =
        tempfile::Builder::new().prefix(&format!("fdu-diff-{label}-")).tempdir().expect("tempdir");
    let root = dir.path().to_path_buf();
    build_fixture(&root, wide_files);
    (dir, root)
}

#[test]
fn cold_scans_agree_across_worker_counts_and_orders() {
    for order in [ScanOrder::BreadthFirst, ScanOrder::DepthFirst] {
        let (_guard, root) = fixture_root("cold", WIDE_DIRECTORY_FILES);
        let mut reference = None;
        for threads in WORKER_COUNTS {
            let settings = ScanConfig { order, ..config(threads) };
            let (index, report) = scan_into_index(&root, &settings).expect("cold scan");
            assert!(report.is_complete(), "{order:?}/{threads}: scan reported errors");
            let candidate = image(&index);
            match &reference {
                None => reference = Some(candidate),
                Some(expected) => assert_same_image(
                    expected,
                    &candidate,
                    &format!("cold scan, {order:?}, {threads} workers"),
                ),
            }
        }
    }
}

#[test]
fn reconciliation_agrees_with_a_fresh_cold_scan_for_every_mutation() {
    for (label, mutate) in mutations() {
        let mut reference: Option<Vec<String>> = None;
        for threads in SAMPLED_WORKER_COUNTS {
            let (_guard, root) = fixture_root(label, NARROW_DIRECTORY_FILES);
            let settings = config(threads);
            let (mut index, baseline) = scan_into_index(&root, &settings).expect("baseline scan");
            assert!(baseline.is_complete(), "{label}/{threads}: baseline scan was partial");

            mutate(&root);

            let report = reconcile(&mut index, &settings, &mut |_| {}).expect("reconcile");
            assert!(report.is_complete(), "{label}/{threads}: reconciliation was partial");

            // The strongest available oracle: whatever the incremental path produced
            // must be indistinguishable from scanning the mutated tree from scratch.
            let (fresh, _) = scan_into_index(&root, &config(1)).expect("verification scan");
            assert_same_image(
                &image(&fresh),
                &image(&index),
                &format!("reconcile after {label} with {threads} workers"),
            );

            // Statistics are part of the contract too: a wave that double-counts or
            // drops work reports a different pass than the serial reconciler does.
            let stats = format!(
                "entries={} dirs_read={} inserted={} updated={} removed={} unchanged={}",
                report.scan.entries,
                report.scan.dirs_read,
                report.apply.inserted,
                report.apply.updated,
                report.apply.removed,
                report.apply.unchanged,
            );
            match &reference {
                None => reference = Some(vec![stats]),
                Some(expected) => assert_eq!(
                    expected[0], stats,
                    "reconcile statistics after {label} diverged at {threads} workers"
                ),
            }
        }
    }
}

#[test]
fn reconciliation_is_idempotent_across_worker_counts() {
    for threads in WORKER_COUNTS {
        let (_guard, root) = fixture_root("idempotent", WIDE_DIRECTORY_FILES);
        let settings = config(threads);
        let (mut index, _) = scan_into_index(&root, &settings).expect("baseline scan");
        let before = image(&index);

        let report = reconcile(&mut index, &settings, &mut |_| {}).expect("reconcile");
        assert_same_image(&before, &image(&index), &format!("no-op reconcile, {threads} workers"));
        assert_eq!(
            report.apply.inserted + report.apply.updated + report.apply.removed,
            0,
            "an unchanged tree produced effective operations at {threads} workers"
        );
    }
}

/// Build `directories` directories of `files_each` files, all with the same contents.
fn build_bulk_tree(root: &Path, directories: usize, files_each: usize, contents: &[u8]) {
    for directory in 0..directories {
        let dir = root.join(format!("d{directory:05}"));
        fs::create_dir_all(&dir).expect("bulk dir");
        for file in 0..files_each {
            fs::write(dir.join(format!("f{file:03}.txt")), contents).expect("bulk file");
        }
    }
}

// The shipped deferred-operation cap is deliberately not exercised here. Reaching it
// needs roughly 67,000 files in one wave, which triples this suite's wall time on every
// platform in the matrix, and `scan.rs` already covers the overflow and the resumed
// serial fallback by injecting a small budget. A run against the real constants was
// verified out of tree during review and converged to the fresh-scan oracle.

#[test]
fn the_automatic_worker_pool_agrees_with_the_serial_reference() {
    // Every other differential case pins an explicit thread count, which takes the
    // fixed-pool branch and never constructs a calibration. Only `threads: None`
    // exercises the adaptive reserve, so it needs its own equivalence case.
    let dir = tempfile::Builder::new().prefix("fdu-automatic-").tempdir().expect("tempdir");
    let root = dir.path();
    // Past the 16,384-entry calibration threshold, so the reserve decision is reached.
    build_bulk_tree(root, 280, 60, b"automatic");

    let automatic = ScanConfig { threads: None, ..ScanConfig::default() };
    let (index, report) = scan_into_index(root, &automatic).expect("automatic scan");
    assert!(report.is_complete());
    let (reference, _) = scan_into_index(root, &config(1)).expect("serial scan");
    assert_same_image(&image(&reference), &image(&index), "automatic cold scan");

    let mut warm = index;
    build_bulk_tree(root, 280, 60, b"automatic, changed");
    let warm_report = reconcile(&mut warm, &automatic, &mut |_| {}).expect("automatic reconcile");
    assert!(warm_report.is_complete());
    let (fresh, _) = scan_into_index(root, &config(1)).expect("verification scan");
    assert_same_image(&image(&fresh), &image(&warm), "automatic reconcile");
}

#[test]
fn a_bounded_scan_scope_reconciles_the_same_way_at_every_worker_count() {
    // Depth and one-filesystem bounds are semantic scope, and the wave worker evaluates
    // them itself rather than deferring to the serial descent decision. Windows has no
    // device identity to bound by and rejects the flag, so only depth is varied there.
    let one_filesystem = cfg!(unix);
    for depth in [0usize, 1, 3] {
        let mut reference: Option<Vec<String>> = None;
        for threads in WORKER_COUNTS {
            let (_guard, root) = fixture_root("scoped", NARROW_DIRECTORY_FILES);
            let settings = ScanConfig { max_depth: Some(depth), one_filesystem, ..config(threads) };
            let (mut index, baseline) = scan_into_index(&root, &settings).expect("baseline scan");
            assert!(baseline.is_complete(), "depth {depth}/{threads}: baseline was partial");

            write(&root.join("added-at-root.txt"), b"root level");
            write(&root.join("fanout").join("d03").join("s0").join("added-deep.txt"), b"deep");
            fs::remove_dir_all(root.join("fanout").join("d04")).expect("remove subtree");

            let report = reconcile(&mut index, &settings, &mut |_| {}).expect("reconcile");
            assert!(report.is_complete(), "depth {depth}/{threads}: reconciliation was partial");

            let (fresh, _) = scan_into_index(&root, &ScanConfig { threads: Some(1), ..settings })
                .expect("verification scan");
            assert_same_image(
                &image(&fresh),
                &image(&index),
                &format!("scoped reconcile at depth {depth} with {threads} workers"),
            );

            let candidate = image(&index);
            match &reference {
                None => reference = Some(candidate),
                Some(expected) => assert_eq!(
                    expected.len(),
                    candidate.len(),
                    "depth {depth}: retained entry count changed at {threads} workers"
                ),
            }
        }
    }
}

#[test]
fn reconciling_a_tree_that_is_changing_underneath_converges_once_it_settles() {
    // Waves compare against an immutable index image while the filesystem keeps moving.
    // A pass over a churning tree may legitimately be partial or out of date; what it
    // may never do is panic, corrupt the index, or fail to converge afterwards.
    use std::sync::atomic::{AtomicBool, Ordering};

    let dir = tempfile::Builder::new().prefix("fdu-churn-").tempdir().expect("tempdir");
    let root = dir.path().to_path_buf();
    build_bulk_tree(&root, 200, 20, b"initial");

    let settings = config(4);
    let (mut index, _) = scan_into_index(&root, &settings).expect("baseline scan");

    let stop = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let churn_root = root.clone();
        let churn = scope.spawn(|| {
            let root = churn_root;
            let mut round = 0u32;
            while !stop.load(Ordering::Relaxed) {
                let directory = root.join(format!("d{:05}", round % 200));
                let _ = fs::write(directory.join("churned.txt"), format!("round {round}"));
                let _ = fs::remove_file(directory.join("f001.txt"));
                let _ = fs::create_dir_all(directory.join(format!("sub{}", round % 4)));
                let _ = fs::remove_dir_all(root.join(format!("d{:05}", (round + 7) % 200)));
                round = round.wrapping_add(1);
            }
        });

        for _ in 0..8 {
            // Errors from a vanishing path are expected; a panic or a poisoned index is
            // not, and neither is a hard failure of the call itself.
            reconcile(&mut index, &settings, &mut |_| {}).expect("reconcile under churn");
        }
        stop.store(true, Ordering::Relaxed);
        churn.join().expect("churn thread");
    });

    // Quiesced: the index must now agree exactly with a fresh serial walk.
    let settled = reconcile(&mut index, &settings, &mut |_| {}).expect("settling reconcile");
    assert!(settled.is_complete(), "the settled pass still reported errors");
    let (fresh, _) = scan_into_index(&root, &config(1)).expect("verification scan");
    assert_same_image(&image(&fresh), &image(&index), "reconcile after concurrent churn");
}
