//! peerwalk: run the ecosystem's directory walkers over one tree, on one job.
//!
//! fdu writes its own walker, so the question "is that worth doing" has to be answered
//! against the walkers a Rust program would otherwise reach for. The important one is
//! `ignore`, which is ripgrep's walker: it lives in the ripgrep repository and ripgrep
//! depends on it, so its speed is the closest thing to a known-good reference point in
//! this ecosystem.
//!
//! The comparison is only meaningful if the *job* is held fixed, and the natural jobs
//! are not the same. A search tool needs to know which entries are files; a disk-usage
//! tool needs their sizes, which costs a metadata call per entry that the search tool
//! never makes. Both are measured here, separately, and only the second is comparable
//! with fdu:
//!
//!   ignore-nostat  - enumerate and classify by `d_type`. ripgrep's job.
//!   ignore-stat    - enumerate and read metadata per entry. du's job, and fdu's.
//!   ignore-default - ripgrep's default filters on (gitignore, hidden, parents).
//!   walkdir        - the single-threaded walker `ignore` is built on.
//!   jwalk          - the parallel walkdir-alike.
//!
//! Tallies match `parfloor`, `walkspike`, `arena_spike` and fdu's summary, so a walker
//! that disagrees is broken rather than fast.
//!
//! Not a workspace member and never built by `make`: it takes third-party dependencies
//! that the shipped crate deliberately does not have. Build it in a throwaway directory
//! with the manifest in this directory's README.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
struct Tally {
    entries: AtomicU64,
    dirs: AtomicU64,
    apparent: AtomicU64,
    allocated: AtomicU64,
}

impl Tally {
    fn count(&self, is_dir: bool, meta: Option<&std::fs::Metadata>) {
        self.entries.fetch_add(1, Ordering::Relaxed);
        if is_dir {
            self.dirs.fetch_add(1, Ordering::Relaxed);
        } else if let Some(m) = meta {
            if m.is_file() {
                use std::os::unix::fs::MetadataExt;
                self.apparent.fetch_add(m.len(), Ordering::Relaxed);
                self.allocated.fetch_add(m.blocks() * 512, Ordering::Relaxed);
            }
        }
    }
    fn print(&self, variant: &str, threads: usize, wall_ns: u128) {
        println!(
            "{{\"variant\":\"{variant}\",\"threads\":{threads},\"dirs\":{},\"entries\":{},\
             \"bytes\":{},\"allocated\":{},\"wall_ns\":{wall_ns}}}",
            self.dirs.load(Ordering::Relaxed),
            self.entries.load(Ordering::Relaxed),
            self.apparent.load(Ordering::Relaxed),
            self.allocated.load(Ordering::Relaxed),
        );
    }
}

fn ignore_walk(root: &Path, threads: usize, stat: bool, filters: bool) -> Tally {
    let mut b = ignore::WalkBuilder::new(root);
    b.standard_filters(filters).follow_links(false).threads(threads);
    let t = Tally::default();
    // The root is descended but is not one of the entries, which is how walkspike,
    // parfloor and fdu all count. Every instrument has to agree for the tallies to
    // work as an oracle.
    let visit = |is_root: bool, is_dir: bool, meta: Option<std::fs::Metadata>, t: &Tally| {
        if !is_root {
            t.count(is_dir, meta.as_ref());
        }
    };
    if threads <= 1 {
        for r in b.build().flatten() {
            let is_dir = r.file_type().is_some_and(|f| f.is_dir());
            let meta = if stat && !is_dir { r.metadata().ok() } else { None };
            visit(r.depth() == 0, is_dir, meta, &t);
        }
        return t;
    }
    b.build_parallel().run(|| {
        let t = &t;
        Box::new(move |r| {
            if let Ok(e) = r {
                let is_dir = e.file_type().is_some_and(|f| f.is_dir());
                let meta = if stat && !is_dir { e.metadata().ok() } else { None };
                visit(e.depth() == 0, is_dir, meta, t);
            }
            ignore::WalkState::Continue
        })
    });
    t
}

fn walkdir_walk(root: &Path) -> Tally {
    let t = Tally::default();
    for e in walkdir::WalkDir::new(root).follow_links(false).into_iter().flatten() {
        if e.depth() == 0 {
            continue;
        }
        let is_dir = e.file_type().is_dir();
        let meta = if is_dir { None } else { e.metadata().ok() };
        t.count(is_dir, meta.as_ref());
    }
    t
}

fn jwalk_walk(root: &Path, threads: usize) -> Tally {
    let t = Tally::default();
    let w = jwalk::WalkDir::new(root)
        .parallelism(jwalk::Parallelism::RayonNewPool(threads))
        .skip_hidden(false);
    for e in w.into_iter().flatten() {
        if e.depth() == 0 {
            continue;
        }
        let is_dir = e.file_type().is_dir();
        let meta = if is_dir { None } else { e.metadata().ok() };
        t.count(is_dir, meta.as_ref());
    }
    t
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 3 {
        eprintln!("usage: peerwalk <variant> <root> [threads]");
        std::process::exit(2);
    }
    let root = Path::new(&a[2]);
    let threads: usize = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
    let start = std::time::Instant::now();
    let t = match a[1].as_str() {
        "ignore-nostat" => ignore_walk(root, threads, false, false),
        "ignore-stat" => ignore_walk(root, threads, true, false),
        "ignore-default" => ignore_walk(root, threads, false, true),
        "walkdir" => walkdir_walk(root),
        "jwalk" => jwalk_walk(root, threads),
        other => {
            eprintln!("peerwalk: unknown variant {other}");
            std::process::exit(2);
        }
    };
    t.print(&a[1], threads, start.elapsed().as_nanos());
}
