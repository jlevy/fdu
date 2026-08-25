//! A filtered answer assembled from bounded pages, losslessly.
//!
//! The property a provider contract needs and a truncating limit cannot give: every match
//! appears exactly once across the pages, the remainder is exact on every page, and a
//! continuation and a nonzero remainder are the same fact rather than two that can
//! disagree. A bound that only truncates satisfies none of these -- it returns a prefix and
//! says how many it dropped, with no way to ask for them.
//!
//! So the assertions here are about *assembly*: page the same selection at several limits
//! and require the concatenation to equal the unpaged answer, in order, with no repeats and
//! no gaps.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use fdu_core::query::Selection;
use fdu_core::{
    CachePolicy, EntryCursor, EntryPageRequest, IndexHandle, OpenConfig, ReadRequest, ScanConfig,
};

/// Files per directory in the fixture, and directories in it.
const PER_DIR: usize = 7;
const DIRS: usize = 5;

/// A tree whose paths interleave across directories, so path order is not directory order.
///
/// `a/z.rs` sorts before `ab.rs`, which is the case a page ordered by anything but the
/// tree's own key gets wrong -- and the case a naive "sort the directories, then their
/// files" assembly gets wrong too.
fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp");
    let root = dir.path();
    for directory in 0..DIRS {
        let child = root.join(format!("d{directory}"));
        std::fs::create_dir_all(&child).expect("mkdir");
        for file in 0..PER_DIR {
            std::fs::write(child.join(format!("f{file}.rs")), vec![b'x'; file + 1]).expect("write");
        }
    }
    std::fs::write(root.join("d0z.rs"), b"tie").expect("write");
    std::fs::write(root.join("top.txt"), b"top").expect("write");
    dir
}

fn opened(root: &Path) -> IndexHandle {
    let config = OpenConfig { policy: CachePolicy::Off, ..OpenConfig::default() };
    let (index, _) = fdu_core::open(root, &config).expect("open");
    IndexHandle::new(index)
}

fn request(limit: u32, after: Option<EntryCursor>, selection: Selection) -> ReadRequest {
    ReadRequest {
        entry_page: Some(EntryPageRequest { after, limit, selection, ..Default::default() }),
        ..Default::default()
    }
}

/// Read every page of `selection` at `limit`, returning the paths in the order they came.
fn assemble(handle: &IndexHandle, limit: u32, selection: &Selection) -> Vec<PathBuf> {
    let mut collected = Vec::new();
    let mut after = None;
    // Pinned across the assembly, which is what makes the pages one answer rather than
    // several: without it a write between two pages leaves the halves describing different
    // trees, and nothing in either half says so.
    let pin = handle.cursor().expect("cursor");
    loop {
        let mut wanted = request(limit, after.clone(), selection.clone());
        wanted.expected = Some(pin);
        let page = handle.read(&wanted).expect("read").entry_page.expect("a page was requested");
        assert!(
            page.rows.len() <= limit as usize,
            "a page must respect its own bound: {} rows at limit {limit}",
            page.rows.len()
        );
        assert_eq!(
            page.next.is_none(),
            page.remaining == 0,
            "a continuation and a remainder are one fact: next={:?}, remaining={}",
            page.next,
            page.remaining
        );
        assert!(
            page.remaining == 0 || !page.rows.is_empty(),
            "a continuation needs a nonempty page to continue from"
        );
        collected.extend(page.rows.iter().map(|row| row.path.clone()));
        match page.next {
            Some(next) => after = Some(next),
            None => break,
        }
    }
    collected
}

/// Every match appears exactly once across the pages, at every page size.
#[test]
fn pages_of_any_size_assemble_into_one_complete_answer() {
    let dir = fixture();
    let handle = opened(dir.path());
    let selection = Selection::default();

    let whole = assemble(&handle, u32::MAX, &selection);
    let expected = (DIRS * PER_DIR) + DIRS + 2;
    assert_eq!(whole.len(), expected, "the fixture's every entry: {whole:?}");

    for limit in [1, 2, 3, 7, 13, u32::MAX] {
        let paged = assemble(&handle, limit, &selection);
        assert_eq!(paged, whole, "assembly at limit {limit} must equal the unpaged answer");
        let unique: BTreeSet<&PathBuf> = paged.iter().collect();
        assert_eq!(unique.len(), paged.len(), "no path may appear twice at limit {limit}");
    }
}

/// Rows come back in path order, which is what makes the cursor a total order.
#[test]
fn rows_are_in_path_order_across_directory_boundaries() {
    let dir = fixture();
    let handle = opened(dir.path());
    let paths = assemble(&handle, 4, &Selection::default());

    let mut sorted = paths.clone();
    sorted.sort();
    assert_eq!(paths, sorted, "a page's order has to be the order its cursor seeks in");

    // The interleaving case the fixture exists for: the separator sorts below every
    // character a name may start with, so a directory's contents precede its own sibling.
    let d0 = paths.iter().position(|path| path == Path::new("d0/f0.rs")).expect("d0/f0.rs");
    let tie = paths.iter().position(|path| path == Path::new("d0z.rs")).expect("d0z.rs");
    assert!(d0 < tie, "d0/f0.rs sorts before d0z.rs: {paths:?}");
}

/// The remainder is exact on every page, and it counts the whole suffix.
#[test]
fn the_remainder_is_the_exact_number_of_rows_still_to_come() {
    let dir = fixture();
    let handle = opened(dir.path());
    let selection = Selection::default();
    let total = assemble(&handle, u32::MAX, &selection).len() as u64;

    let limit = 6;
    let mut after = None;
    let mut delivered = 0;
    let pin = handle.cursor().expect("cursor");
    loop {
        let mut wanted = request(limit, after.clone(), selection.clone());
        wanted.expected = Some(pin);
        let page = handle.read(&wanted).expect("read").entry_page.expect("page");
        assert_eq!(page.total, total, "the denominator is the selection, not the page");
        delivered += page.rows.len() as u64;
        assert_eq!(
            page.remaining,
            total - delivered,
            "the remainder is what is left, counted rather than estimated"
        );
        match page.next {
            Some(next) => after = Some(next),
            None => break,
        }
    }
    assert_eq!(delivered, total);
}

/// The totals describe the whole selection, not the rows on this page.
///
/// The denominator a bounded page needs to be honest about itself: "7 of 37, 91 bytes in
/// all". Deriving the second half from the rows on screen would make it a statement about
/// the page, which is the thing the caller can already see.
#[test]
fn the_totals_describe_every_match_not_the_page() {
    let dir = fixture();
    let handle = opened(dir.path());
    let selection = Selection::default();

    let whole = handle
        .read(&request(u32::MAX, None, selection.clone()))
        .expect("read")
        .entry_page
        .expect("page");
    let first = handle.read(&request(3, None, selection)).expect("read").entry_page.expect("page");

    assert_eq!(first.rows.len(), 3, "the page is bounded");
    assert_eq!(first.total, whole.total, "and its denominator is not");
    assert_eq!(first.totals, whole.totals, "nor are its aggregates");
    assert!(
        first.total > first.rows.len() as u64,
        "and the denominator exceeds what is on screen, which is what makes it one"
    );
    // Deliberately not compared against the sum of the page's own `attrs.size`: a
    // directory row carries the directory inode's size, which is filesystem-dependent and
    // is not part of `totals.bytes` at all. Comparing them would be an assertion about
    // ext4.
    let shown: u64 = first
        .rows
        .iter()
        .filter(|row| row.entry.kind == fdu_core::EntryKind::File)
        .map(|row| row.entry.attrs.size)
        .sum();
    assert!(shown <= first.totals.bytes, "the page's files are a subset of the selection's");
}

/// A selection narrows what is paged, and the page still accounts for all of it.
#[test]
fn a_filtered_selection_pages_only_its_own_matches() {
    let dir = fixture();
    let handle = opened(dir.path());

    let selection = Selection {
        kinds: vec![fdu_core::EntryKind::File],
        min_size: Some(4),
        ..Selection::default()
    };

    let matches = assemble(&handle, 3, &selection);
    assert!(!matches.is_empty(), "the fixture must produce some matches");
    for path in &matches {
        let attrs = handle
            .read(&ReadRequest { rollups: vec![path.clone()], ..Default::default() })
            .expect("read");
        // A file has no roll-up, which is the check: everything paged here is a file.
        assert!(attrs.rollups[0].is_none(), "{path:?} should be a file");
    }

    let page = handle.read(&request(3, None, selection)).expect("read").entry_page.expect("page");
    assert_eq!(page.total, matches.len() as u64, "the denominator counts matches, not entries");
    assert_eq!(page.totals.dirs, 0, "no directory was admitted");
}

/// A zero limit is refused rather than served, because it cannot be continued.
#[test]
fn a_zero_limit_is_refused() {
    let dir = fixture();
    let handle = opened(dir.path());
    let error = handle
        .read(&request(0, None, Selection::default()))
        .expect_err("a page of zero rows names no cursor to continue from");
    assert!(error.to_string().contains("page limit"), "{error}");
}

/// A page pinned to a version the index has moved past is refused, not quietly continued.
///
/// The other half of lossless assembly: a name cursor keeps page two from skipping a row,
/// and only the pin keeps page two from describing a different tree than page one.
#[test]
fn a_stale_pin_refuses_rather_than_paging_a_different_tree() {
    let dir = fixture();
    let handle = opened(dir.path());
    let pin = handle.cursor().expect("cursor");

    std::fs::write(dir.path().join("d0/late.rs"), b"late").expect("write");
    let scan = ScanConfig::default();
    fdu_core::scan::reconcile_handle(&handle, &scan, &mut |_| {}).expect("refresh");

    let mut wanted = request(3, None, Selection::default());
    wanted.expected = Some(pin);
    assert!(
        matches!(handle.read(&wanted), Err(fdu_core::Error::VersionUnavailable { .. })),
        "an assembly pinned to a version that has aged out restarts rather than straddling"
    );
}

/// The page charges what it considered, not only what it returned.
#[test]
fn paging_a_narrow_selection_charges_the_entries_it_examined() {
    let dir = fixture();
    let handle = opened(dir.path());

    let selection = Selection {
        include: vec![fdu_core::query::Pattern::parse("f0.rs").expect("pattern")],
        ..Selection::default()
    };

    let bundle = handle.read(&request(2, None, selection)).expect("read");
    let page = bundle.entry_page.expect("page");
    let charged = bundle.projections.entry_page;

    assert!(page.rows.len() <= 2, "the page is bounded");
    assert!(
        charged.entries_visited > page.rows.len() as u64,
        "a narrow selection over a wide tree costs every entry it rejected: {} visited for {} rows",
        charged.entries_visited,
        page.rows.len()
    );
    assert_eq!(charged.rows, page.rows.len() as u64, "and charges the rows it returned");
}

/// Page two costs a page, not a pass over everything before it.
///
/// The property a provider contract needs beside losslessness, and the one the first
/// version of this surface did not have: every page restarted at the root, filtered the
/// whole subtree, and recomputed the selection-wide denominator, so assembling P pages
/// cost P passes over the index. Bounded and lossless and quadratic is still unusable at
/// the sizes a catalog is for.
///
/// `entries_visited` makes it directly testable, which is why the counter exists. The
/// assertion is a ratio rather than a constant: the first page pays for the denominator,
/// and what must not happen is that every page after it pays again.
#[test]
fn continuing_an_assembly_costs_a_page_rather_than_a_pass() {
    let dir = wide_fixture();
    let handle = opened(dir.path());
    let selection = Selection::default();
    let limit = 5;

    let mut wanted = request(limit, None, selection.clone());
    let pin = handle.cursor().expect("cursor");
    wanted.expected = Some(pin);
    let first = handle.read(&wanted).expect("read");
    let opening = first.projections.entry_page.entries_visited;
    let page = first.entry_page.expect("page");
    assert!(page.total > 200, "the fixture has to be big enough to tell the two apart");

    // Deep into the assembly, so a scan-from-the-top would have the whole prefix to cross.
    let mut cursor = page.next.expect("a continuation");
    let mut costs = Vec::new();
    while costs.len() < 20 {
        let mut wanted = request(limit, Some(cursor.clone()), selection.clone());
        wanted.expected = Some(pin);
        let bundle = handle.read(&wanted).expect("read");
        costs.push(bundle.projections.entry_page.entries_visited);
        match bundle.entry_page.expect("page").next {
            Some(next) => cursor = next,
            None => break,
        }
    }
    assert_eq!(costs.len(), 20, "the fixture must not run out of pages before the telling ones");

    // Flat, which is the property rather than a proxy for it. A continuation that crossed
    // the prefix would cost more with every page; one that does not costs the same at page
    // twenty as at page two, whatever the tree's size.
    let (first_few, last_few) = (costs[0], *costs.last().expect("pages"));
    assert!(last_few <= first_few * 2, "continuation cost must not grow with position: {costs:?}");
    assert!(
        first_few * 4 < opening,
        "and it must not repeat the opening pass: {first_few} visited against {opening}"
    );
}

/// The denominator is established once and carried, not recomputed per page.
#[test]
fn every_page_reports_the_denominator_the_first_one_established() {
    let dir = wide_fixture();
    let handle = opened(dir.path());
    let selection = Selection { kinds: vec![fdu_core::EntryKind::File], ..Selection::default() };

    let pin = handle.cursor().expect("cursor");
    let mut wanted = request(7, None, selection.clone());
    wanted.expected = Some(pin);
    let first = handle.read(&wanted).expect("read").entry_page.expect("page");
    let (total, totals) = (first.total, first.totals);

    let mut cursor = first.next.expect("a continuation");
    let mut delivered = first.rows.len() as u64;
    loop {
        let mut wanted = request(7, Some(cursor.clone()), selection.clone());
        wanted.expected = Some(pin);
        let page = handle.read(&wanted).expect("read").entry_page.expect("page");
        assert_eq!(page.total, total, "one denominator across the assembly");
        assert_eq!(page.totals, totals, "and one set of aggregates");
        delivered += page.rows.len() as u64;
        assert_eq!(page.remaining, total - delivered, "the remainder counts down from it");
        match page.next {
            Some(next) => cursor = next,
            None => break,
        }
    }
    assert_eq!(delivered, total, "and the assembly is still complete");
}

/// A continuation from another version is refused rather than answered.
///
/// The counts inside it were established against one image of the index. Serving it
/// against another would report a denominator for a tree that is no longer there, which is
/// worse than the stale-pin case a caller can already detect: nothing in the page would
/// say so.
#[test]
fn a_continuation_from_another_version_is_refused() {
    let dir = fixture();
    let handle = opened(dir.path());
    let page = handle
        .read(&request(3, None, Selection::default()))
        .expect("read")
        .entry_page
        .expect("page");
    let cursor = page.next.expect("a continuation");

    std::fs::write(dir.path().join("d0/late.rs"), b"late").expect("write");
    fdu_core::scan::reconcile_handle(&handle, &ScanConfig::default(), &mut |_| {})
        .expect("refresh");

    let served = handle.read(&request(3, Some(cursor), Selection::default())).expect("read");
    assert!(
        served.entry_page.is_none(),
        "a continuation carrying counts from another image cannot be answered from this one"
    );
}

/// A tree wide and deep enough that a prefix scan and a seek differ by a lot.
fn wide_fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp");
    let root = dir.path();
    for outer in 0..12 {
        for inner in 0..6 {
            let leaf = root.join(format!("d{outer:02}/s{inner}"));
            std::fs::create_dir_all(&leaf).expect("mkdir");
            for file in 0..8 {
                std::fs::write(leaf.join(format!("f{file}.rs")), vec![b'x'; file + 1])
                    .expect("write");
            }
        }
    }
    dir
}
