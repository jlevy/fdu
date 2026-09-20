//! Allocation regression guard for query row sharing.

use std::time::UNIX_EPOCH;

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

#[test]
fn bounded_single_file_view_does_not_clone_every_materialized_path() {
    const FILES: usize = 1_024;
    let mut index = Index::new("/allocation-test");
    let operations = (0..FILES)
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

    let _disable = DisableCounters;
    fdu_core::counters::reset();
    fdu_core::counters::enable(true);
    let report = fdu_core::query::report(&index, &request, &provenance).expect("report");
    let allocations = usize::try_from(fdu_core::counters::thread_snapshot().allocs)
        .expect("allocation count fits usize");
    fdu_core::counters::enable(false);

    let fdu_core::query::Section::Files { rows, total, .. } = &report.sections[0] else {
        panic!("largest files section")
    };
    assert_eq!((*total, rows.len()), (FILES, 1));
    assert_eq!(rows[0].bytes, u64::try_from(FILES).expect("fixture size"));
    assert!(
        allocations < FILES * 3,
        "a second full-tree FileRow/PathBuf clone adds at least one allocation per file: \
         {allocations} allocations for {FILES} files"
    );
}
