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

/// One edit to a request field that decides which rows a page returns.
type Elsewhere = fn(&mut EntryPageRequest);

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

/// A continuation from another version is refused with an error, not an absent page.
///
/// The counts inside it were established against one image of the index. Serving it against
/// another would report a denominator for a tree that is no longer there. The *shape* of
/// the refusal matters as much as the refusal: this used to return `None`, which the read
/// folded into an absent projection -- indistinguishable from "that root is not a
/// directory", so a consumer that cannot tell those apart restarts an assembly it should
/// have failed.
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

    // No `expected` pin: the refusal is the cursor's own, not a courtesy the caller has to
    // ask for by redundantly pinning a read it already pinned by continuing.
    let refused = handle
        .read(&request(3, Some(cursor), Selection::default()))
        .expect_err("a continuation carrying counts from another image cannot be answered");
    assert!(
        matches!(refused, fdu_core::Error::VersionUnavailable { .. }),
        "and it says which kind of stale it is: {refused}"
    );
}

/// A continuation is bound to the question it was issued for, not only to the version.
///
/// The hole the version check alone left open: an *honest* cursor from one query, replayed
/// against another at the same instant, returned the second query's rows under the first
/// one's denominator and remainder. Nothing in the page revealed it -- a wrong answer
/// rather than a missing one.
#[test]
fn a_continuation_belongs_to_one_question_and_is_refused_by_any_other() {
    let dir = fixture();
    let handle = opened(dir.path());
    let issued = handle
        .read(&request(3, None, Selection::default()))
        .expect("read")
        .entry_page
        .expect("page")
        .next
        .expect("a continuation");

    let elsewhere = |mut wanted: ReadRequest, edit: fn(&mut EntryPageRequest)| {
        edit(wanted.entry_page.as_mut().expect("a page was requested"));
        handle.read(&wanted)
    };
    let cases: [(&str, Elsewhere); 6] = [
        ("a different subtree", |page| page.root = PathBuf::from("d0")),
        ("a different depth bound", |page| page.max_depth = Some(1)),
        ("a different selection", |page| {
            page.selection.kinds = vec![fdu_core::EntryKind::File];
        }),
        ("a different size bound", |page| {
            page.selection.min_size = Some(4);
        }),
        ("a different terminal suffix", |page| {
            page.selection.admit_terminal_extension(".rs").expect("suffix");
        }),
        ("a different ancestor name", |page| {
            page.selection.admit_ancestor_name("d1").expect("component");
        }),
    ];
    // The request's own `plane` is not exercised here because this fixture promotes none,
    // and a page cannot name a plane the index does not maintain. It reaches the same
    // fingerprint as the four above, on the line beside them.
    for (what, edit) in cases {
        let refused = elsewhere(request(3, Some(issued.clone()), Selection::default()), edit)
            .expect_err(what);
        assert!(
            refused.to_string().contains("continuation"),
            "{what} must be refused as a continuation, not answered: {refused}"
        );
    }

    // And the same question still works, or the four above are about a cursor that no
    // longer resumes anything.
    let served = handle
        .read(&request(3, Some(issued), Selection::default()))
        .expect("the question it was issued for")
        .entry_page
        .expect("page");
    assert!(!served.rows.is_empty());
}

/// The catalog predicates page and resume like every other filter.
///
/// Worth its own test rather than trusting the shape check alone: these two are the first
/// axes evaluated from a *name* rather than from an entry, so the page walk has to reach
/// them at all, and the seek that resumes one has to reach them the same way. A predicate
/// applied on the first page and not on a continuation is an assembly that silently grows.
#[test]
fn a_catalog_predicate_pages_and_resumes_like_any_other_filter() {
    let dir = fixture();
    // Uppercase on purpose: the terminal is case-folded, so this is a match, and it is one
    // only if the fold happens on the real paging path rather than in the unit test.
    std::fs::write(dir.path().join("d1").join("UPPER.RS"), b"x").expect("write");
    let handle = opened(dir.path());

    let mut selection = Selection::default();
    selection.admit_terminal_extension(".rs").expect("suffix");
    selection.admit_ancestor_name("d1").expect("component");

    let expected = u32::try_from(PER_DIR).expect("a small fixture") + 1;
    let matches = assemble(&handle, 2, &selection);
    assert_eq!(
        matches.len(),
        expected as usize,
        "every `.rs` under `d1` and nothing else: {matches:?}"
    );
    for path in &matches {
        assert!(path.starts_with("d1"), "{path:?} has no `d1` ancestor");
        let name = path.file_name().expect("a name").to_string_lossy();
        let (_, terminal) = name.rsplit_once('.').expect("a suffix");
        assert!(terminal.eq_ignore_ascii_case("rs"), "{path:?} is not a terminal `.rs`");
    }

    // Same answer at every page size, which is what makes a continuation a continuation.
    for limit in [1, 3, expected, 100] {
        assert_eq!(
            assemble(&handle, limit, &selection),
            matches,
            "the assembly at limit {limit} differs from the assembly at 2"
        );
    }

    let page = handle.read(&request(2, None, selection)).expect("read").entry_page.expect("page");
    assert_eq!(page.total, u64::from(expected), "the denominator counts matches, not entries");
    assert_eq!(page.remaining, u64::from(expected - 2));
}

/// A token is engine-issued: a tampered one is refused rather than believed.
///
/// The counts are not a caller's to choose, and the type makes that so within Rust by
/// having no public fields and no constructor. Across a boundary that carries a string the
/// type cannot help, so the token carries a checksum -- which rules out the accident that
/// design invites: a token round-tripped through a wire format that dropped or reordered a
/// field, arriving as an honest-looking claim about an answer nobody computed.
#[test]
fn a_tampered_token_is_refused() {
    let dir = fixture();
    let handle = opened(dir.path());
    let issued = handle
        .read(&request(3, None, Selection::default()))
        .expect("read")
        .entry_page
        .expect("page")
        .next
        .expect("a continuation");

    let token = issued.encode();
    assert_eq!(fdu_core::EntryCursor::decode(&token).expect("its own token round-trips"), issued);

    // Flip one nibble in the middle: a field's value, not its framing.
    let mut edited: Vec<char> = token.chars().collect();
    let middle = edited.len() / 2;
    edited[middle] = if edited[middle] == '0' { '1' } else { '0' };
    let edited: String = edited.into_iter().collect();
    assert!(edited != token, "the edit has to change something");
    assert!(
        fdu_core::EntryCursor::decode(&edited).is_err(),
        "a token with one nibble changed is not a token this engine issued"
    );

    for invented in ["", "not-a-token", "00", &token[..token.len() - 2]] {
        assert!(
            fdu_core::EntryCursor::decode(invented).is_err(),
            "{invented:?} is not a continuation"
        );
    }
}

/// A caller who edits a token and repairs its checksum is still refused.
///
/// This is the property the checksum alone never had, and the previous test was honest
/// about not proving: `token_checksum` is unkeyed, so anybody can recompute it, and the
/// numbers a continuation carries -- the denominator, the aggregates, the delivered count --
/// are exactly the numbers a page reports without recomputing. An editable token is a
/// caller-authored claim about an answer nobody walked.
///
/// The forgery here is built the way an attacker would: decode the hex, rewrite a field,
/// recompute the trailing checksum over the new body, re-encode. It has to be assembled by
/// hand rather than through `EntryCursor`, because the type has no constructor -- which is
/// the point, and also why the string boundary needs its own answer.
#[test]
fn a_token_edited_and_re_checksummed_is_still_refused() {
    let dir = fixture();
    let handle = opened(dir.path());
    let selection = Selection::default();
    let issued = handle
        .read(&request(3, None, selection.clone()))
        .expect("read")
        .entry_page
        .expect("page")
        .next
        .expect("a continuation");

    let mut bytes = decode_hex(&issued.encode());
    // The denominator: eight bytes of magic, then `total` first among the scalars.
    let inflated = u64::from_le_bytes(bytes[8..16].try_into().expect("eight bytes")) + 1_000;
    bytes[8..16].copy_from_slice(&inflated.to_le_bytes());
    let repaired = fnv1a(&bytes[..bytes.len() - 8]);
    let end = bytes.len() - 8;
    bytes[end..].copy_from_slice(&repaired.to_le_bytes());
    let forged = encode_hex(&bytes);

    // It decodes: the checksum is repaired, so the encoding says nothing is wrong with it.
    let cursor = fdu_core::EntryCursor::decode(&forged)
        .expect("a repaired checksum passes the encoding's own check, which is the problem");
    assert_ne!(cursor, issued, "the edit has to have changed the value");

    // And the read refuses it, because the tag over those bytes is not one this index made.
    let refused = handle
        .read(&request(3, Some(cursor), selection))
        .expect_err("a forged continuation must not be answered");
    assert!(
        refused.to_string().contains("continuation"),
        "refused as a continuation rather than by accident: {refused}"
    );
}

/// A token re-versioned onto another index is refused by the tag, not by the version.
///
/// The plain cross-index case is caught before the tag is consulted: an index mints its
/// session identity per construction, so a token from another open names a version this one
/// never had. That check is a comparison against a value the token itself carries, though,
/// and a caller holding the encoding can edit it -- which is what makes the version check
/// insufficient on its own and this test the one that matters. Session, clock and checksum
/// are rewritten to what the second index accepts, leaving the version and the shape with
/// nothing to object to. What refuses it is the tag, over bytes that no longer match it.
#[test]
fn a_continuation_re_versioned_onto_another_index_is_still_refused() {
    let dir = fixture();
    let origin = opened(dir.path());
    let elsewhere = opened(dir.path());

    let issued = origin
        .read(&request(3, None, Selection::default()))
        .expect("read")
        .entry_page
        .expect("page")
        .next
        .expect("a continuation");

    // Through the string boundary, which is how a token actually travels between two
    // handles: a wire format carries the text, not the value.
    let carried = fdu_core::EntryCursor::decode(&issued.encode()).expect("its own token");
    assert!(
        origin.read(&request(3, Some(carried.clone()), Selection::default())).is_ok(),
        "the index that issued it still honours it, or the refusals below prove nothing"
    );
    assert!(
        elsewhere.read(&request(3, Some(carried), Selection::default())).is_err(),
        "another index must not answer a continuation it did not issue"
    );

    // Now re-version it to the second index and repair the checksum, so only the tag is
    // left to refuse it. Session and clock are the third and fourth scalars after the
    // magic, which is where `EntryCursor::encode` writes them.
    let wanted = elsewhere.cursor().expect("cursor");
    let mut bytes = decode_hex(&issued.encode());
    bytes[24..32].copy_from_slice(&wanted.session.0.to_le_bytes());
    bytes[32..40].copy_from_slice(&wanted.clock.0.to_le_bytes());
    let end = bytes.len() - 8;
    let repaired = fnv1a(&bytes[..end]);
    bytes[end..].copy_from_slice(&repaired.to_le_bytes());

    let reversioned = fdu_core::EntryCursor::decode(&encode_hex(&bytes)).expect("well-formed");
    assert_eq!(reversioned.version(), wanted, "the edit has to actually move the version");
    let refused = elsewhere
        .read(&request(3, Some(reversioned), Selection::default()))
        .expect_err("a re-versioned continuation is still not one this index issued");
    assert!(
        refused.to_string().contains("continuation"),
        "refused as a continuation rather than as a stale version: {refused}"
    );
}

/// A token far larger than one this engine could issue is refused without decoding it.
#[test]
fn an_oversized_token_is_refused() {
    let huge = "ab".repeat(200_000);
    assert!(
        fdu_core::EntryCursor::decode(&huge).is_err(),
        "a token with no possible path that long is not one this engine issued"
    );
}

/// Hex helpers for the forgery above, kept beside it rather than shared.
///
/// A test that reached into the engine's own encoder would be testing it against itself;
/// these are written out so the forged token is built the way a caller outside the process
/// would have to build one.
fn decode_hex(token: &str) -> Vec<u8> {
    token
        .as_bytes()
        .chunks(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).expect("ascii"), 16).expect("hex"))
        .collect()
}

fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut hex, byte| {
        let _ = write!(hex, "{byte:02x}");
        hex
    })
}

fn fnv1a(body: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in body {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
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
