//! A maintained plane and a walk over the same restriction must reach the same numbers.
//!
//! The unit tests hold the plane arithmetic against a hand-built index, which is the right
//! place to pin the reducer algebra and the wrong place to notice that a real scan never
//! reaches it the same way. Three defects lived in exactly that gap, all of them invisible
//! to a report that only ever showed one tier:
//!
//! - `ensure_dir_chain` built its placeholder's contribution by hand as `dirs: 1` with no
//!   planes, and on a real walk nearly every directory is materialised as an ancestor
//!   before it is observed -- so a plane's directory count was near zero while its files
//!   and bytes were right.
//! - A rebind re-tagged every entry and left the planes derived from the old bits, which
//!   made `gitignore` -- the rule planes exist for -- report a plane equal to the tree,
//!   because a Path-tier rule cannot be bound until the walk has found its control files.
//! - An unfiltered `--view summary` was answered by a tier that retains aggregate tallies
//!   and no index, which has no plane to read and returned the whole tree under a plane's
//!   heading.
//!
//! What connects them is that a plane read is fast because it reads state maintained
//! elsewhere, so nothing about a wrong plane looks wrong. The walking tier computes the
//! same restriction from the entries themselves and is the only independent check there
//! is; these tests run both over a scanned tree and require them to agree.

use std::path::Path;
use std::sync::Arc;

use fdu_core::query::{Query, Selection, ViewSpec};
use fdu_core::tags::TagRules;
use fdu_core::{Bound, Index, ScanConfig};

/// A tree with tagged and untagged entries at several depths, plus git control files.
///
/// Directories deep enough that ancestors are created as placeholders, which is what makes
/// the walk different from a hand-built index.
fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp");
    let root = dir.path();
    std::fs::create_dir_all(root.join("src/deep/deeper")).expect("mkdir");
    std::fs::create_dir_all(root.join("docs")).expect("mkdir");
    std::fs::create_dir_all(root.join("target/debug")).expect("mkdir");
    std::fs::create_dir_all(root.join(".cache/blobs")).expect("mkdir");
    std::fs::write(root.join(".gitignore"), "*.log\ntarget/\n").expect("write");
    std::fs::write(root.join("docs/.gitignore"), "!keep.log\n").expect("write");
    std::fs::write(root.join("src/main.rs"), "fn main() {}").expect("write");
    std::fs::write(root.join("src/.env"), "SECRET=1").expect("write");
    std::fs::write(root.join("src/deep/deeper/nested.rs"), "// deep").expect("write");
    std::fs::write(root.join("src/deep/run.log"), "noise").expect("write");
    std::fs::write(root.join("docs/keep.log"), "kept").expect("write");
    std::fs::write(root.join("docs/guide.md"), "# guide").expect("write");
    std::fs::write(root.join("target/debug/out"), "binary").expect("write");
    std::fs::write(root.join(".cache/blobs/blob"), "cached").expect("write");
    dir
}

fn scanned(root: &Path, rule: &str) -> Index {
    let rules = Arc::new(
        TagRules::from_names([rule])
            .expect("the rule is in the catalogue")
            .with_promoted([rule])
            .expect("an enabled rule can be promoted"),
    );
    let config = ScanConfig { tags: Some(rules), ..ScanConfig::default() };
    let mut index = fdu_core::scan::scan_into_index(root, &config).expect("scan").0;
    // The step the session path takes and `scan_into_index` leaves to its caller: a
    // Path-tier rule cannot be bound until the walk has found the control files, so the
    // scan tags every entry under rules that answer "nothing is ignored". Reading tags
    // before this point is what `debug_assert_tags_bound` refuses -- and rebuilding the
    // planes this bind invalidates is what the rebind used to skip.
    let directories = index.control_file_directories();
    let bound =
        index.tag_rules().bound_to(root, directories.iter().map(std::path::PathBuf::as_path));
    index.adopt_tag_rules(Arc::new(bound));
    index
}

/// Sum the plane a walk finds, by asking selection about every entry the index holds.
///
/// Deliberately not `report`: the point is an oracle that shares no code with the tier
/// under test, and a report built from `Selection { plane }` on the reading tier would be
/// checking the maintained state against itself.
fn walked(index: &Index, rule: &str) -> (u64, u64, u64, u64) {
    let bit = index.tag_rules().id_of(rule).expect("enabled");
    let (mut files, mut dirs, mut others, mut bytes) = (0, 0, 0, 0);
    let mut stack = vec![fdu_core::EntryId::ROOT];
    while let Some(id) = stack.pop() {
        let Some(children) = index.children_of(id) else {
            continue;
        };
        let children: Vec<_> = children.map(|(_, child)| child).collect();
        for child in children {
            let kind = index.kind_of(child).expect("live");
            if kind.is_dir() {
                stack.push(child);
            }
            // A tag rides on the entry alone, so a tagged directory is outside the plane
            // while its untagged descendants stay inside it.
            if index.tag_bits_of(child) & (1 << bit) != 0 {
                continue;
            }
            match kind {
                fdu_core::EntryKind::File => {
                    files += 1;
                    bytes += index.attrs_of(child).expect("live").size;
                }
                fdu_core::EntryKind::Dir => dirs += 1,
                _ => others += 1,
            }
        }
    }
    (files, dirs, others, bytes)
}

/// The rules this build carries, in the order a test should try them.
///
/// `gitignore` is a default feature and absent under `--no-default-features`, which is how
/// a library consumer builds and the one combination `make check` exercises that a local
/// run otherwise never does. Gating the list rather than the tests keeps the Name-tier
/// cases -- the ones that must hold in every build -- running everywhere.
const RULES: &[&str] = &[
    "dotfile",
    #[cfg(feature = "gitignore")]
    "gitignore",
];

/// The Path-tier rule, when this build has one: the case a rebind is required for.
#[cfg(feature = "gitignore")]
const REBOUND_RULE: &str = "gitignore";
#[cfg(not(feature = "gitignore"))]
const REBOUND_RULE: &str = "dotfile";

/// The maintained plane over a scanned tree equals what a walk of the same tree finds.
#[test]
fn a_scanned_plane_agrees_with_a_walk_of_the_same_restriction() {
    let dir = fixture();
    for rule in RULES.iter().copied() {
        let index = scanned(dir.path(), rule);
        let plane = index.tag_rules().plane_of(rule).expect("promoted");
        let read = index
            .plane_rollup_bounded(Path::new(""), plane, Bound::All)
            .expect("the root is a directory");
        assert_eq!(
            (read.files, read.dirs, read.others, read.bytes),
            walked(&index, rule),
            "{rule}: the maintained plane disagrees with a walk over the same entries",
        );

        // And it is a real restriction, so an equality that held only because the plane
        // was the whole tree would not pass for one.
        let whole = index.rollup(Path::new("")).expect("the root is a directory");
        assert!(read.files < whole.files, "{rule}: the plane must exclude something");
        assert!(read.dirs < whole.dirs, "{rule}: including directories");
    }
}

/// The same agreement for a scan that never rebinds, which is its own entry point.
///
/// A Name-tier rule is decided as each entry lands, so `scan_into_index` answers about it
/// without the bind step a Path-tier rule needs -- and that is the configuration where a
/// directory materialised as an ancestor is never revisited by anything. The placeholder
/// path built its contribution by hand and gave it no planes, so this is where that showed:
/// files and bytes right, directories near zero. With a rebind in play the rebuild hides
/// it, which is exactly why this case is asserted separately.
#[test]
fn a_scan_that_never_rebinds_still_counts_directories_into_its_planes() {
    let dir = tempfile::tempdir().expect("temp");
    let root = dir.path();
    // Deep enough that the walker reports entries whose ancestors it has not yet
    // materialised, which is the only way through `ensure_dir_chain`.
    std::fs::create_dir_all(root.join("a")).expect("mkdir");
    std::fs::create_dir_all(root.join("b/c/d")).expect("mkdir");
    std::fs::write(root.join("a/f1"), "x").expect("write");
    std::fs::write(root.join("b/c/d/f2"), "x").expect("write");
    std::fs::write(root.join(".dot"), "x").expect("write");

    let rules = Arc::new(
        TagRules::from_names(["dotfile"])
            .expect("in the catalogue")
            .with_promoted(["dotfile"])
            .expect("enabled rules can be promoted"),
    );
    let config = ScanConfig { tags: Some(rules), ..ScanConfig::default() };
    let index = fdu_core::scan::scan_into_index(root, &config).expect("scan").0;
    let plane = index.tag_rules().plane_of("dotfile").expect("promoted");
    let read = index
        .plane_rollup_bounded(Path::new(""), plane, Bound::All)
        .expect("the root is a directory");
    assert_eq!(
        (read.files, read.dirs, read.others, read.bytes),
        walked(&index, "dotfile"),
        "a plane built during the walk disagrees with the entries the walk left behind",
    );
    assert_eq!(read.dirs, 4, "a, b, b/c and b/c/d are all untagged");
}

/// Every directory agrees, not only the root.
///
/// The root is the one figure a broken plane is most likely to get right by accident: it
/// is where a missing per-directory contribution can be masked by a correct total arriving
/// through some other path.
#[test]
fn every_directory_in_a_scanned_tree_holds_its_own_plane() {
    let dir = fixture();
    let index = scanned(dir.path(), REBOUND_RULE);
    let plane = index.tag_rules().plane_of(REBOUND_RULE).expect("promoted");

    let mut checked = 0;
    let mut stack = vec![(fdu_core::EntryId::ROOT, std::path::PathBuf::new())];
    while let Some((id, path)) = stack.pop() {
        let Some(children) = index.children_of(id) else {
            continue;
        };
        for (name, child) in children.map(|(name, child)| (name.to_os_string(), child)) {
            if index.kind_of(child) == Some(fdu_core::EntryKind::Dir) {
                stack.push((child, path.join(&name)));
            }
        }
        let read = index.plane_rollup_of(id, plane, Bound::All).expect("a directory");
        let whole = index.rollup_of(id).expect("a directory");
        assert!(read.files <= whole.files, "{}: plane files exceed the total", path.display());
        assert!(read.dirs <= whole.dirs, "{}: plane dirs exceed the total", path.display());
        assert!(read.bytes <= whole.bytes, "{}: plane bytes exceed the total", path.display());
        checked += 1;
    }
    assert!(checked >= 8, "the fixture should reach every directory, saw {checked}");
}

/// A plane request is answered from an index, never from the tier that retains none.
///
/// An unfiltered summary is normally served by a transient tier that accumulates totals
/// and keeps no per-directory state. It cannot answer about a plane, and it does not fail
/// when asked: it returns the whole tree, in the right shape, under the plane's heading.
#[test]
fn an_unfiltered_summary_still_answers_a_plane_from_maintained_state() {
    let dir = fixture();
    let index = scanned(dir.path(), REBOUND_RULE);
    let plane = index.tag_rules().plane_of(REBOUND_RULE).expect("promoted");
    let selection = Selection { plane: Some(plane), ..Selection::default() };
    assert!(selection.is_unfiltered(), "a plane must not cost the walking tier");

    let expected = walked(&index, REBOUND_RULE);
    let query = Query { selection, views: vec![ViewSpec::Summary], ..Query::default() };
    let provenance = fdu_core::query::Provenance {
        scan_started_at: None,
        generated_at: std::time::SystemTime::UNIX_EPOCH,
        source: fdu_core::query::ReportSource::ColdScan,
        complete: true,
        errors: Vec::new(),
    };
    let report = fdu_core::query::report(&index, &query, &provenance);
    let [fdu_core::query::Section::Summary(summary)] = report.sections.as_slice() else {
        panic!("one summary section");
    };
    assert_eq!(
        (summary.files, summary.dirs, summary.others, summary.bytes),
        expected,
        "an unfiltered summary answered a plane request with something else",
    );
}
