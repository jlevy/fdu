//! Hidden-path pruning removes entries from the index rather than from an answer.
//!
//! The axis test, and it is the whole reason this is not a tag. A `dotfile` tag leaves an
//! entry in the index and lets a query filter it, with both numbers visible; this decides
//! what the index holds. So the assertions here are about absence -- no row, no tally, no
//! subtree read -- rather than about a report that came back smaller.
//!
//! Scope changes what a snapshot means, so a rule and its allowlist are fingerprinted into
//! snapshot identity. That is also the thing a test has to check separately from the walk:
//! a pruned tree with the fingerprint wired up wrongly still walks correctly and then reads
//! back a snapshot describing a different retained set, which is the failure that looks
//! like a cache bug rather than a scope one.

use std::path::Path;
use std::sync::Arc;

use fdu_core::admission::HiddenPolicy;
use fdu_core::{Bound, CachePolicy, Index, OpenConfig, ScanConfig};

/// A tree with hidden files and directories at several depths, and a hidden control file.
fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp");
    let root = dir.path();
    std::fs::create_dir_all(root.join("src")).expect("mkdir");
    std::fs::create_dir_all(root.join(".git/objects")).expect("mkdir");
    std::fs::create_dir_all(root.join(".github/workflows")).expect("mkdir");
    std::fs::write(root.join(".gitignore"), "*.log\n").expect("write");
    std::fs::write(root.join("src/main.rs"), "fn main() {}").expect("write");
    std::fs::write(root.join("src/.env"), "SECRET=1").expect("write");
    std::fs::write(root.join("src/debug.log"), "noise").expect("write");
    std::fs::write(root.join(".git/objects/pack"), "opaque").expect("write");
    std::fs::write(root.join(".github/workflows/ci.yml"), "on: push").expect("write");
    dir
}

fn scanned(root: &Path, hidden: Option<HiddenPolicy>) -> (Index, fdu_core::ScanReport) {
    let config = ScanConfig { hidden: hidden.map(Arc::new), ..ScanConfig::default() };
    fdu_core::scan::scan_into_index(root, &config).expect("scan")
}

fn names(index: &Index, at: &str) -> Vec<String> {
    index
        .children(Path::new(at))
        .map(|children| children.map(|(name, _)| name.to_string_lossy().into_owned()).collect())
        .unwrap_or_default()
}

/// A pruned entry has no row, and its subtree was never read.
#[test]
fn pruning_removes_the_entry_and_never_reads_its_subtree() {
    let dir = fixture();

    let (kept, kept_report) = scanned(dir.path(), None);
    assert!(names(&kept, "").contains(&".git".to_string()));
    assert!(kept.rollup(Path::new(".git")).is_some(), "the subtree was walked");

    let (pruned, pruned_report) = scanned(dir.path(), Some(HiddenPolicy::prune_hidden([""; 0])));
    let top = names(&pruned, "");
    assert_eq!(top, vec!["src".to_string()], "only the one visible directory");
    assert!(pruned.rollup(Path::new(".git")).is_none(), "no such path in the index");
    assert_eq!(names(&pruned, "src"), vec!["debug.log".to_string(), "main.rs".to_string()]);

    // Not merely filtered afterwards: the walk read fewer directories, because a pruned
    // directory is not descended into. This is the claim a tag could not make.
    assert!(
        pruned_report.dirs_read < kept_report.dirs_read,
        "pruned {} directories, kept {}",
        pruned_report.dirs_read,
        kept_report.dirs_read,
    );
    assert!(pruned_report.entries < kept_report.entries);

    // And the totals move, because the entries are gone rather than hidden from a view.
    let whole = kept.rollup(Path::new("")).expect("root");
    let narrow = pruned.rollup(Path::new("")).expect("root");
    assert!(narrow.files < whole.files);
    assert!(narrow.bytes < whole.bytes);
}

/// An allowlisted name is admitted with its whole subtree.
#[test]
fn an_allowlisted_name_is_admitted_along_with_what_is_under_it() {
    let dir = fixture();
    let (index, _) = scanned(dir.path(), Some(HiddenPolicy::prune_hidden([".github"])));

    let top = names(&index, "");
    assert!(top.contains(&".github".to_string()), "{top:?}");
    assert!(!top.contains(&".git".to_string()), "still pruned: {top:?}");
    assert_eq!(names(&index, ".github"), vec!["workflows".to_string()]);
    assert_eq!(names(&index, ".github/workflows"), vec!["ci.yml".to_string()]);

    // The allowlist admits the name, not everything hidden beneath it.
    assert!(!names(&index, "src").contains(&".env".to_string()));
}

/// A control file the walk pruned is still readable, so a Path-tier rule can bind.
///
/// `.gitignore` is hidden and governs entries that are not, so pruning it silently would
/// answer a gitignore question with "nothing is ignored". The walk records where it saw
/// one; the index never holds a row for it.
#[cfg(feature = "gitignore")]
#[test]
fn a_pruned_control_file_still_governs_the_entries_it_governs() {
    let dir = fixture();
    let rules =
        Arc::new(fdu_core::tags::TagRules::from_names(["gitignore"]).expect("in the catalogue"));
    let config = ScanConfig {
        hidden: Some(Arc::new(HiddenPolicy::prune_hidden([""; 0]))),
        tags: Some(rules),
        ..ScanConfig::default()
    };
    let (mut index, report) = fdu_core::scan::scan_into_index(dir.path(), &config).expect("scan");

    assert!(!names(&index, "").contains(&".gitignore".to_string()), "not retained");
    assert_eq!(report.control_dirs, vec![std::path::PathBuf::new()], "seen at the root");
    assert_eq!(index.pruned_control_dirs(), [std::path::PathBuf::new()], "and adopted");

    // Bind exactly as the session path does, off what the index knows.
    let directories = index.control_file_directories();
    assert_eq!(directories, vec![std::path::PathBuf::new()], "the pruned record is the record");
    let bound =
        index.tag_rules().bound_to(dir.path(), directories.iter().map(std::path::PathBuf::as_path));
    index.adopt_tag_rules(Arc::new(bound));

    // `*.log` came from a file that is not in the index, and it still decides.
    let bits = index.tag_bits_of(index.lookup(Path::new("src/debug.log")).expect("present"));
    assert_ne!(bits, 0, "src/debug.log is ignored by a .gitignore nobody can see");
    let kept = index.tag_bits_of(index.lookup(Path::new("src/main.rs")).expect("present"));
    assert_eq!(kept, 0, "main.rs is not");
}

/// A snapshot recorded under one admission rule is not read under another.
///
/// The entries the other rule would have kept are absent from the *recording* rather than
/// from the tree, and nothing in the file distinguishes those. Reinterpreting one would
/// answer a question about a tree with a description of a smaller one.
#[test]
fn an_admission_rule_is_part_of_snapshot_identity() {
    let dir = fixture();
    let cache = tempfile::tempdir().expect("cache");
    let path = cache.path().join("snap.fdu");

    let open_with = |hidden: Option<HiddenPolicy>, policy: CachePolicy| {
        let config = OpenConfig {
            scan: ScanConfig { hidden: hidden.map(Arc::new), ..ScanConfig::default() },
            cache_path: Some(path.clone()),
            policy,
            ..OpenConfig::default()
        };
        fdu_core::open(dir.path(), &config)
    };

    // Seed under pruning, which writes the snapshot.
    let seeded_files = open_with(Some(HiddenPolicy::prune_hidden([""; 0])), CachePolicy::Auto)
        .expect("seed")
        .0
        .total()
        .files;

    // Read back under `Only`, which is the policy this claim needs. Under `Auto` a
    // revalidation re-walks and corrects whatever the snapshot said, so a wrongly reused
    // one still produces the right numbers -- the reuse decision would be untested and the
    // test would read as though it had checked it. `Only` cannot touch the tree, so what
    // comes back is exactly what was accepted.
    let same = open_with(Some(HiddenPolicy::prune_hidden([""; 0])), CachePolicy::Only)
        .expect("the same rule reads its own snapshot")
        .0;
    assert_eq!(same.total().files, seeded_files);
    drop(same);

    // A wider allowlist is a different retained set. Refused, rather than answered with a
    // description of the narrower tree.
    assert!(
        open_with(Some(HiddenPolicy::prune_hidden([".github"])), CachePolicy::Only).is_err(),
        "a snapshot from a narrower rule must not be reused for a wider one",
    );

    // And so is admitting everything, which is what every index built before this existed
    // means -- so this direction is the one that must not silently reuse either.
    assert!(open_with(None, CachePolicy::Only).is_err());
}

/// A warm start of a pruned tree can still bind, because the record survives the snapshot.
///
/// The entries are gone -- that is what pruning means -- so a loader has nothing to look
/// at. Without the recorded directories a second run of the same command answers
/// differently from the first, and the difference reads as a cache fault.
#[cfg(feature = "gitignore")]
#[test]
fn a_warm_start_of_a_pruned_tree_still_knows_where_the_control_files_were() {
    let dir = fixture();
    let cache = tempfile::tempdir().expect("cache");
    let path = cache.path().join("snap.fdu");
    let config = OpenConfig {
        scan: ScanConfig {
            hidden: Some(Arc::new(HiddenPolicy::prune_hidden([""; 0]))),
            tags: Some(Arc::new(
                fdu_core::tags::TagRules::from_names(["gitignore"]).expect("in the catalogue"),
            )),
            ..ScanConfig::default()
        },
        cache_path: Some(path.clone()),
        policy: CachePolicy::Auto,
        ..OpenConfig::default()
    };

    let (cold, _) = fdu_core::open(dir.path(), &config).expect("cold");
    let cold_ignored = ignored_paths(&cold);
    assert!(cold_ignored.contains(&"src/debug.log".to_string()), "{cold_ignored:?}");
    drop(cold);

    // `Only`, deliberately. Under `Auto` a revalidation re-walks and re-records the
    // directories, so the snapshot's copy is never consulted and this would pass with the
    // section removed -- which it did, until the policy said what the test meant. `Only`
    // is contractually forbidden to touch the tree, so the file is the whole story.
    let only = OpenConfig { policy: CachePolicy::Only, ..config.clone() };
    let (warm, _) = fdu_core::open(dir.path(), &only).expect("warm");
    assert_eq!(ignored_paths(&warm), cold_ignored, "one tree, one answer, either way in");
    drop(warm);

    // And under `Auto`, where the walk happens anyway, for the same answer by the other
    // route: the two must not disagree about which files are ignored.
    let (revalidated, _) = fdu_core::open(dir.path(), &config).expect("warm");
    assert_eq!(ignored_paths(&revalidated), cold_ignored);
}

#[cfg(feature = "gitignore")]
fn ignored_paths(index: &Index) -> Vec<String> {
    let mut found = Vec::new();
    let mut stack = vec![(fdu_core::EntryId::ROOT, std::path::PathBuf::new())];
    while let Some((id, path)) = stack.pop() {
        let Some(children) = index.children_of(id) else {
            continue;
        };
        for (name, child) in children.map(|(name, child)| (name.to_os_string(), child)) {
            let child_path = path.join(&name);
            if index.kind_of(child) == Some(fdu_core::EntryKind::Dir) {
                stack.push((child, child_path.clone()));
            }
            if index.tag_bits_of(child) != 0 {
                found.push(child_path.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    found.sort();
    found
}

/// A refresh applies the rule the scan applied, so a new hidden file does not appear.
#[test]
fn a_refresh_admits_exactly_what_the_scan_admitted() {
    let dir = fixture();
    let config = ScanConfig {
        hidden: Some(Arc::new(HiddenPolicy::prune_hidden([""; 0]))),
        ..ScanConfig::default()
    };
    let (mut index, _) = fdu_core::scan::scan_into_index(dir.path(), &config).expect("scan");
    let before = index.rollup_bounded(Path::new(""), Bound::All).expect("root").files;

    std::fs::write(dir.path().join("src/.secret"), "x").expect("write");
    std::fs::write(dir.path().join("src/added.rs"), "fn added() {}").expect("write");
    fdu_core::scan::reconcile(&mut index, &config, &mut |_| {}).expect("reconcile");

    assert!(!names(&index, "src").contains(&".secret".to_string()), "still outside scope");
    assert!(names(&index, "src").contains(&"added.rs".to_string()), "and inside it");
    assert_eq!(
        index.rollup_bounded(Path::new(""), Bound::All).expect("root").files,
        before + 1,
        "one file entered the index, not two",
    );
}
