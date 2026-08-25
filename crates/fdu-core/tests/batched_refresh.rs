//! A refresh over many hint paths is one operation, not a loop over a one-path one.
//!
//! The difference is the reason the consumer contract takes a path *set*. Iterating gives N
//! reconciliations, N announcements and N terminal positions, so a receipt covering them
//! describes a range rather than a boundary and a caller cannot say which of the N its
//! cursor is past. It also pays N ancestor merges where the union costs one.
//!
//! So the assertions here are about the shape of the operation -- how many subtrees were
//! actually walked, and what the announcements looked like from a consumer reading in the
//! middle -- rather than about the rows, which a loop would get right too.

use std::path::{Path, PathBuf};

use fdu_core::admission::HiddenPolicy;
use fdu_core::{CachePolicy, Index, IndexHandle, OpenConfig, RefusedPath, ScanConfig, StateChange};

/// A tree deep and wide enough that overlapping hints have somewhere to overlap.
fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp");
    let root = dir.path();
    for area in ["src", "docs", "vendor"] {
        std::fs::create_dir_all(root.join(area).join("nested")).expect("mkdir");
        std::fs::write(root.join(area).join("top.txt"), b"x").expect("write");
        std::fs::write(root.join(area).join("nested/deep.txt"), b"xx").expect("write");
    }
    std::fs::create_dir_all(root.join(".git")).expect("mkdir");
    std::fs::write(root.join(".git/config"), b"[core]").expect("write");
    dir
}

fn opened(root: &Path, hidden: Option<HiddenPolicy>) -> (Index, ScanConfig) {
    let scan = ScanConfig { hidden: hidden.map(std::sync::Arc::new), ..ScanConfig::default() };
    let config =
        OpenConfig { policy: CachePolicy::Off, scan: scan.clone(), ..OpenConfig::default() };
    let (index, _) = fdu_core::open(root, &config).expect("open");
    (index, scan)
}

fn paths(names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(PathBuf::from).collect()
}

/// Overlapping hints cost one walk, which is the whole point of batching them.
#[test]
fn hints_under_one_directory_fold_into_a_single_walk() {
    let dir = fixture();
    let (mut index, scan) = opened(dir.path(), None);

    let receipt = fdu_core::scan::reconcile_paths(
        &mut index,
        &paths(&["src/nested", "src", "docs", "src/top.txt"]),
        &scan,
        &mut |_| {},
    )
    .expect("refresh");

    assert_eq!(
        receipt.accepted,
        paths(&["src/nested", "src", "docs", "src/top.txt"]),
        "every path was reconciled, including the three folded into one walk"
    );
    assert_eq!(receipt.walked, paths(&["docs", "src"]), "and only two subtrees were actually read");
    assert!(receipt.rejected.is_empty(), "{:?}", receipt.rejected);
}

/// A hint naming the root collapses the batch to one whole-tree walk.
#[test]
fn a_hint_at_the_root_covers_every_other() {
    let dir = fixture();
    let (mut index, scan) = opened(dir.path(), None);

    let receipt = fdu_core::scan::reconcile_paths(
        &mut index,
        &paths(&["src", "", "docs"]),
        &scan,
        &mut |_| {},
    )
    .expect("refresh");

    assert_eq!(receipt.walked, vec![PathBuf::new()], "the root covers everything below it");
    assert_eq!(receipt.accepted.len(), 3, "and all three paths were reconciled by it");
}

/// Refusal is a real answer, and each refusal says which rule refused it.
///
/// A receipt that listed only what it did would make "reconciled, nothing had changed" and
/// "never looked" the same answer -- and a caller feeding its own watcher's hints would
/// re-send a path forever waiting for a change it will never be told about.
#[test]
fn a_path_the_scope_does_not_hold_is_refused_by_name() {
    let dir = fixture();
    let (mut index, scan) = opened(dir.path(), Some(HiddenPolicy::prune_hidden([""; 0])));

    let receipt = fdu_core::scan::reconcile_paths(
        &mut index,
        &paths(&["src", "../escape", ".git/config"]),
        &scan,
        &mut |_| {},
    )
    .expect("refresh");

    assert_eq!(receipt.accepted, paths(&["src"]));
    assert_eq!(
        receipt.rejected,
        vec![
            (PathBuf::from("../escape"), RefusedPath::OutsideRoot),
            (PathBuf::from(".git/config"), RefusedPath::NotAdmitted),
        ],
        "each refusal names the rule that made it, not merely that one was made"
    );
    assert_eq!(receipt.walked, paths(&["src"]), "and nothing refused was walked");
}

/// A depth bound refuses a hint below it rather than walking to find nothing.
#[test]
fn a_hint_below_the_depth_bound_is_refused() {
    let dir = fixture();
    let scan = ScanConfig { max_depth: Some(1), ..ScanConfig::default() };
    let config =
        OpenConfig { policy: CachePolicy::Off, scan: scan.clone(), ..OpenConfig::default() };
    let (mut index, _) = fdu_core::open(dir.path(), &config).expect("open");

    let receipt = fdu_core::scan::reconcile_paths(
        &mut index,
        &paths(&["src", "src/nested"]),
        &scan,
        &mut |_| {},
    )
    .expect("refresh");

    assert_eq!(receipt.accepted, paths(&["src"]));
    assert_eq!(receipt.rejected, vec![(PathBuf::from("src/nested"), RefusedPath::BeyondDepth)]);
}

/// Past the bound, a path is refused rather than silently dropped or fatal.
#[test]
fn more_paths_than_one_refresh_accepts_are_refused_by_position() {
    let dir = fixture();
    let (mut index, scan) = opened(dir.path(), None);

    let over = fdu_core::scan::MAX_REFRESH_PATHS + 3;
    let many: Vec<PathBuf> = (0..over).map(|_| PathBuf::from("src")).collect();
    let receipt =
        fdu_core::scan::reconcile_paths(&mut index, &many, &scan, &mut |_| {}).expect("refresh");

    assert_eq!(receipt.accepted.len(), fdu_core::scan::MAX_REFRESH_PATHS);
    assert_eq!(receipt.rejected.len(), 3);
    assert!(
        receipt.rejected.iter().all(|(_, reason)| *reason == RefusedPath::Bounded),
        "{:?}",
        receipt.rejected
    );
    assert_eq!(
        receipt.walked,
        paths(&["src"]),
        "and a thousand copies of one path are still one walk"
    );
}

/// An empty hint set is a no-op, deliberately not a whole-tree refresh.
///
/// Conflating them would make a dropped hint list mean "re-read everything", which is the
/// most expensive possible response to having lost track of what changed.
#[test]
fn an_empty_hint_set_reads_nothing() {
    let dir = fixture();
    let (mut index, scan) = opened(dir.path(), None);
    let mut deltas = 0;

    let receipt = fdu_core::scan::reconcile_paths(&mut index, &[], &scan, &mut |_| deltas += 1)
        .expect("refresh");

    assert!(receipt.accepted.is_empty() && receipt.walked.is_empty());
    assert_eq!(receipt.reconciliation.scan.dirs_read, 0, "nothing was read");
    assert_eq!(deltas, 0, "and nothing was committed");
}

/// The whole batch is announced before any of it is read.
///
/// Announcing each subtree just before its own walk would let a consumer read an index
/// where half the batch is marked reconciling and half still claims to be fresh, with
/// nothing saying the second half is about to move. That is the same "one instant, one
/// answer" rule the bundled read exists for, applied to a multi-subtree operation.
#[test]
fn every_subtree_is_announced_before_any_is_walked() {
    let dir = fixture();
    let (index, scan) = opened(dir.path(), None);
    let handle = IndexHandle::new(index);

    // Every subtree must have something to report, or the walk emits no operations at all
    // and the ordering check below has nothing to order against. The first draft skipped
    // this and passed against an implementation that announced each subtree just before
    // its own walk -- vacuously, because there were no operations either way.
    for area in ["src", "docs", "vendor"] {
        std::fs::write(dir.path().join(area).join("added.txt"), b"new").expect("write");
    }

    let mut announced: Vec<PathBuf> = Vec::new();
    let mut first_operation_after: Option<usize> = None;
    fdu_core::scan::reconcile_paths_handle(
        &handle,
        &paths(&["src", "docs", "vendor"]),
        &scan,
        &mut |delta| {
            for change in &delta.state {
                if let StateChange::Freshness { path, freshness, .. } = change
                    && *freshness == fdu_core::Freshness::Reconciling
                {
                    announced.push(path.clone());
                }
            }
            if !delta.ops.is_empty() && first_operation_after.is_none() {
                first_operation_after = Some(announced.len());
            }
        },
    )
    .expect("refresh");

    assert_eq!(
        announced,
        paths(&["docs", "src", "vendor"]),
        "every subtree in the batch announced itself"
    );
    assert_eq!(
        first_operation_after,
        Some(announced.len()),
        "the first row moved only after the last announcement, not between them"
    );
}

/// One batch, one terminal position: resuming from it sees nothing the batch already said.
///
/// The property iteration cannot provide. Each of N calls names its own position, so a
/// receipt covering them describes a range and a caller resuming from the last one has no
/// way to know whether the earlier ones are behind it or interleaved with another writer.
#[test]
fn a_batch_leaves_one_position_to_resume_from() {
    let dir = fixture();
    let (index, scan) = opened(dir.path(), None);
    let handle = IndexHandle::new(index);
    let before = handle.cursor().expect("cursor");

    fdu_core::scan::reconcile_paths_handle(
        &handle,
        &paths(&["src", "docs", "vendor"]),
        &scan,
        &mut |_| {},
    )
    .expect("refresh");

    let after = handle.cursor().expect("cursor");
    assert!(after.clock > before.clock, "the batch committed something");
    assert!(
        handle.since(after).expect("resume").deltas.is_empty(),
        "and the position it left sits past every delta it committed"
    );
    assert_eq!(after.session, before.session, "one session throughout");
}

/// A batched refresh sees the same rows a per-path loop would.
///
/// The batching is about the shape of the operation, not about what it observes: if the two
/// disagreed about the tree, the shape would not be worth having.
#[test]
fn a_batch_and_a_loop_agree_about_the_tree() {
    let dir = fixture();
    let (mut batched, scan) = opened(dir.path(), None);
    let (mut looped, _) = opened(dir.path(), None);

    std::fs::write(dir.path().join("src/added.rs"), b"fn main() {}").expect("write");
    std::fs::remove_file(dir.path().join("docs/top.txt")).expect("remove");

    fdu_core::scan::reconcile_paths(&mut batched, &paths(&["src", "docs"]), &scan, &mut |_| {})
        .expect("batched");
    for one in ["src", "docs"] {
        fdu_core::scan::reconcile_subtree(&mut looped, Path::new(one), &scan, &mut |_| {})
            .expect("looped");
    }

    assert_eq!(batched.total().files, looped.total().files);
    assert_eq!(batched.total().bytes, looped.total().bytes);
    assert!(batched.lookup(Path::new("src/added.rs")).is_some(), "the new file arrived");
    assert!(batched.lookup(Path::new("docs/top.txt")).is_none(), "and the removed one left");
}
