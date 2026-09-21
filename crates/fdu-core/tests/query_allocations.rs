//! Allocation regression guard for query row sharing.
//!
//! The counting allocator is process-global. These tests take a mutex around
//! enable/reset/snapshot so a parallel sibling cannot tear the count. CI on the
//! leftover cherry-pick saw Types at 966 against a two-view 8,214 for that reason.

use std::sync::{Mutex, PoisonError};
use std::time::UNIX_EPOCH;

static COUNTER_LOCK: Mutex<()> = Mutex::new(());

use fdu_core::query::{
    Basis, Bound, Provenance, Query, ReportSource, Request, Selection, ViewSpec,
};
use fdu_core::{Attrs, EntryKind, Index, Observation, Op};

#[global_allocator]
static ALLOCATOR: fdu_core::counters::alloc::CountingAlloc<std::alloc::System> =
    fdu_core::counters::system_allocator();

struct DisableCounters;

impl Drop for DisableCounters {
    fn drop(&mut self) {
        fdu_core::counters::enable(false);
        fdu_core::counters::reset();
    }
}

fn allocation_fixture(files: usize) -> Index {
    let mut index = Index::new("/allocation-test");
    let operations = (0..files)
        .map(|number| Op::Upsert {
            path: format!("source-file-{number:04}-with-an-ordinary-name.rs").into(),
            kind: EntryKind::File,
            attrs: Attrs {
                size: u64::try_from(number + 1).expect("fixture size"),
                allocated: u64::try_from(number + 1).expect("fixture size"),
                mtime_ns: i64::try_from(number).expect("fixture mtime"),
                ctime_ns: i64::try_from(number).expect("fixture ctime"),
                inode: u64::try_from(number).expect("fixture inode"),
                dev: 1,
            },
        })
        .collect();
    index.apply(&Observation::new(operations)).expect("build fixture");
    index
}

fn report_allocations(index: &Index, views: Vec<ViewSpec>, selection: Selection) -> usize {
    let request = Request::new(
        Basis::held_by(index),
        Query { views, selection, ..Query::default() },
        UNIX_EPOCH,
    );
    let provenance = Provenance {
        scan_started_at: None,
        generated_at: UNIX_EPOCH,
        source: ReportSource::ColdScan,
        complete: true,
        errors: Vec::new(),
    };

    let _guard = COUNTER_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    let _disable = DisableCounters;
    fdu_core::counters::reset();
    fdu_core::counters::enable(true);
    fdu_core::query::report(index, &request, &provenance).expect("report");
    let allocations = usize::try_from(fdu_core::counters::thread_snapshot().allocs)
        .expect("allocation count fits usize");
    fdu_core::counters::enable(false);
    allocations
}

#[test]
fn bounded_single_file_view_does_not_clone_every_materialized_path() {
    const FILES: usize = 1_024;
    let index = allocation_fixture(FILES);
    let query = Query {
        views: vec![ViewSpec::Largest],
        selection: Selection { limit: Some(Bound::Limit(1)), ..Selection::default() },
        ..Query::default()
    };
    let request = Request::new(Basis::held_by(&index), query, UNIX_EPOCH);
    let provenance = Provenance {
        scan_started_at: None,
        generated_at: UNIX_EPOCH,
        source: ReportSource::ColdScan,
        complete: true,
        errors: Vec::new(),
    };

    let (report, allocations) = {
        let _guard = COUNTER_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let _disable = DisableCounters;
        fdu_core::counters::reset();
        fdu_core::counters::enable(true);
        let report = fdu_core::query::report(&index, &request, &provenance).expect("report");
        let allocations = usize::try_from(fdu_core::counters::thread_snapshot().allocs)
            .expect("allocation count fits usize");
        fdu_core::counters::enable(false);
        (report, allocations)
    };

    let fdu_core::query::Section::Files { rows, total, .. } = &report.sections[0] else {
        panic!("largest files section")
    };
    assert_eq!((*total, rows.len()), (FILES, 1));
    assert_eq!(rows[0].bytes, u64::try_from(FILES).expect("fixture size"));
    let types = report_allocations(&index, vec![ViewSpec::Types], Selection::default());
    assert!(
        allocations < types,
        "a bounded Largest view must not exceed one measured full-tree walk: \
         {allocations} allocations versus {types} for Types"
    );
    assert!(
        allocations < FILES * 3,
        "a second full-tree FileRow/PathBuf clone adds at least one allocation per file: \
         {allocations} allocations for {FILES} files"
    );
}

#[test]
fn unfiltered_metric_views_share_one_every_entry_walk() {
    const FILES: usize = 1_024;
    let index = allocation_fixture(FILES);
    let types = report_allocations(&index, vec![ViewSpec::Types], Selection::default());
    let families = report_allocations(&index, vec![ViewSpec::Families], Selection::default());
    let both =
        report_allocations(&index, vec![ViewSpec::Types, ViewSpec::Families], Selection::default());

    // `every_entry` joins a path and clones it once per entry, so no complete measurement
    // of this fixture can come in under two allocations per file. Anything below that is a
    // torn reading rather than a cheap one, and has to fail as itself instead of quietly
    // satisfying the comparison below.
    assert!(
        types >= FILES * 2 && families >= FILES * 2 && both >= FILES * 2,
        "torn measurement: {types} for Types, {families} for Families, {both} for both, \
         against a floor of {} for {FILES} files",
        FILES * 2
    );

    // Compare the pair against the two views measured separately. That cancels the fixed
    // per-report overhead and the per-view aggregation cost, which are both larger than a
    // walk and so hid it: a `both < types * 2` bound expands to `F + 2W + 2A < 2F + 2W +
    // 2A`, true whenever the fixed overhead F is positive, no matter how many walks ran.
    // A never-share build measured 10,265 against that bound of 10,268 and passed. What
    // sharing saves is exactly one `every_entry` walk, so measure that, with a 2x margin.
    let saved = (types + families).saturating_sub(both);
    assert!(
        saved >= FILES,
        "H138 must save one every_entry walk: [Types, Families] allocated {both} against \
         {types} + {families} measured separately, a saving of {saved}; flipping \
         row_consumers > 1 to never share costs a second walk, which allocates at least \
         one PathBuf per entry"
    );
}
