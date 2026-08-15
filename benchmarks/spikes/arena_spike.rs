// arena_spike: measure the floor for an index-retaining parallel scan when the
// consumer is replaced by worker-local arenas plus one post-hoc roll-up pass.
// It is the consumer-side companion to walkspike.c, which isolates enumeration:
// same syscall load and worker count as fdu's portable scan, different retained
// representation.
//
// Retains, per entry: name bytes (worker-local arena), kind, size, blocks, mtime,
// parent directory slot. Retains, per directory: a global tally slot with parent
// link and depth. That is the state a tree or summary view needs; it is what
// fdu's Entry keeps, minus per-entry BTreeMaps, boxed nodes and owned paths.
// Roll-ups are computed once, bottom-up by depth, after the walk - the shape S4
// and H60 describe - instead of once per entry per ancestor.
//
// Deliberately NOT a correctness prototype: no delta contract, no arbitration,
// no progressive publication, no error provenance, no symlink records, and a
// grace-pass termination that a production scan could not ship. It exists to
// measure the physics of the structure, exactly like walkspike.c does for the
// enumeration layer, and its tallies must match fdu's summary for the run to
// count (files/bytes/allocated match exactly; dirs differ by one because the
// root is not its own child).
//
// 2026-08-15, 450,463-entry generated tree, 4 vCPU Linux VM, warm, 4 workers:
// this spike ~199 ms and <= 23 MiB peak RSS beside dut ~179 ms, where the fdu
// CLI tree view (--cache off) measured ~849 ms and ~279 MiB. See the research
// note research-2026-08-15-consumer-structural-headroom.md.
//
// Build: rustc -O -o arena_spike arena_spike.rs
// Run:   ./arena_spike /path/to/tree [workers]

use std::env;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

#[derive(Clone, Copy, Default)]
struct Tally {
    files: u64,
    dirs: u64,
    bytes: u64,
    allocated: u64,
    newest_mtime: i64,
}

impl Tally {
    fn absorb(&mut self, o: &Tally) {
        self.files += o.files;
        self.dirs += o.dirs;
        self.bytes += o.bytes;
        self.allocated += o.allocated;
        if o.newest_mtime > self.newest_mtime {
            self.newest_mtime = o.newest_mtime;
        }
    }
}

// One row per completed directory: slot, parent slot, depth, direct tally.
struct DirRow {
    parent: u32,
    depth: u32,
    tally: Tally,
}

// Retained per file entry, worker-local: 24 bytes + name bytes in the arena.
#[allow(dead_code)]
struct FileRec {
    name_off: u32,
    name_len: u16,
    kind: u8,
    parent_slot: u32,
    size: u64,
    allocated: u64,
    mtime: i64,
}

struct Shared {
    // (abs path, slot, depth); slot indexes `rows`.
    queue: Mutex<Vec<(PathBuf, u32, u32)>>,
    rows: Mutex<Vec<Option<DirRow>>>,
}

fn main() {
    let mut args = env::args().skip(1);
    let root = PathBuf::from(args.next().expect("usage: arena_spike ROOT [WORKERS]"));
    let workers: usize = args.next().map(|w| w.parse().expect("workers")).unwrap_or(4);

    let started = Instant::now();
    let shared = Shared {
        queue: Mutex::new(Vec::new()),
        rows: Mutex::new(Vec::new()),
    };
    {
        let mut rows = shared.rows.lock().expect("rows");
        rows.push(None); // slot 0 = root
        shared.queue.lock().expect("queue").push((root.clone(), 0, 0));
    }

    let walk_started = Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| worker(&shared));
        }
    });
    let walk_ns = walk_started.elapsed().as_nanos();

    // Roll-up: one pass over directories, deepest first. Parents always have
    // smaller depth, so after sorting by depth descending each child folds into
    // a parent not yet folded.
    let rollup_started = Instant::now();
    let rows = shared.rows.into_inner().expect("rows");
    let mut rows: Vec<DirRow> = rows.into_iter().map(|r| r.expect("complete")).collect();
    let mut order: Vec<u32> = (1..rows.len() as u32).collect();
    order.sort_unstable_by_key(|&i| std::cmp::Reverse(rows[i as usize].depth));
    for i in order {
        let child = rows[i as usize].tally;
        let parent = rows[i as usize].parent;
        rows[parent as usize].tally.absorb(&child);
        rows[parent as usize].tally.dirs += 1;
    }
    let rollup_ns = rollup_started.elapsed().as_nanos();

    let total = rows[0].tally;
    let wall_ns = started.elapsed().as_nanos();
    println!(
        "{{\"files\":{},\"dirs\":{},\"bytes\":{},\"allocated\":{},\"newest_mtime\":{},\"wall_ms\":{:.1},\"walk_ms\":{:.1},\"rollup_ms\":{:.1},\"workers\":{}}}",
        total.files,
        total.dirs,
        total.bytes,
        total.allocated,
        total.newest_mtime,
        wall_ns as f64 / 1e6,
        walk_ns as f64 / 1e6,
        rollup_ns as f64 / 1e6,
        workers,
    );
}

fn worker(shared: &Shared) {
    // Worker-local retained state: the "index shard".
    let mut names: Vec<u8> = Vec::with_capacity(1 << 20);
    let mut files: Vec<FileRec> = Vec::with_capacity(1 << 16);
    let mut claimed: Vec<(PathBuf, u32, u32)> = Vec::new();
    let mut discovered: Vec<(PathBuf, u32, u32)> = Vec::new();
    let mut completed: Vec<(u32, DirRow)> = Vec::new();

    loop {
        claimed.clear();
        {
            let mut q = shared.queue.lock().expect("queue");
            let take = q.len().min(16);
            if take == 0 {
                drop(q);
                // Naive termination: spin briefly; if the queue stays empty and
                // all workers are idle we may exit early on a race. For a spike
                // over a static tree a short grace pass is enough: retry a few
                // times before giving up.
                let mut retries = 0;
                loop {
                    std::thread::yield_now();
                    let mut q = shared.queue.lock().expect("queue");
                    if !q.is_empty() {
                        let take = q.len().min(16);
                        let at = q.len() - take;
                        claimed.extend(q.drain(at..));
                        break;
                    }
                    drop(q);
                    retries += 1;
                    if retries > 2000 {
                        flush(shared, &mut completed);
                        return;
                    }
                }
            } else {
                let at = q.len() - take;
                claimed.extend(q.drain(at..));
            }
        }

        for (dir, slot, depth) in claimed.drain(..) {
            let mut tally = Tally::default();
            let listing = match fs::read_dir(&dir) {
                Ok(l) => l,
                Err(_) => {
                    completed.push((slot, DirRow { parent: 0, depth, tally }));
                    continue;
                }
            };
            for item in listing.flatten() {
                let name = item.file_name();
                let Ok(meta) = item.metadata() else { continue };
                let ft = meta.file_type();
                let name_bytes = name.as_bytes();
                let name_off = names.len() as u32;
                names.extend_from_slice(name_bytes);
                if ft.is_dir() {
                    // Allocate a global slot and enqueue.
                    let child_slot = {
                        let mut rows = shared.rows.lock().expect("rows");
                        rows.push(None);
                        (rows.len() - 1) as u32
                    };
                    discovered.push((dir.join(&name), child_slot, depth + 1));
                    // Parent and depth are fixed at discovery; completion later
                    // fills in the tally.
                    parent_link(shared, child_slot, slot, depth + 1);
                } else if ft.is_file() {
                    let size = meta.len();
                    let allocated = meta.blocks() * 512;
                    let mtime = meta.mtime_nsec() + meta.mtime() * 1_000_000_000;
                    tally.files += 1;
                    tally.bytes += size;
                    tally.allocated += allocated;
                    if mtime > tally.newest_mtime {
                        tally.newest_mtime = mtime;
                    }
                    files.push(FileRec {
                        name_off,
                        name_len: name_bytes.len() as u16,
                        kind: 1,
                        parent_slot: slot,
                        size,
                        allocated,
                        mtime,
                    });
                }
                // Symlinks/other: counted nowhere, matching fdu's tree totals for
                // files/dirs/bytes on this generated tree (143 symlinks ignored).
            }
            completed.push((slot, DirRow { parent: u32::MAX, depth, tally }));
        }

        if !discovered.is_empty() {
            let mut q = shared.queue.lock().expect("queue");
            q.append(&mut discovered);
        }
        if completed.len() >= 64 {
            flush(shared, &mut completed);
        }
    }
}

// Record the parent/depth for a directory slot as soon as it is allocated, so the
// completion only fills in the tally.
fn parent_link(shared: &Shared, slot: u32, parent: u32, depth: u32) {
    let mut rows = shared.rows.lock().expect("rows");
    rows[slot as usize] = Some(DirRow { parent, depth, tally: Tally::default() });
}

fn flush(shared: &Shared, completed: &mut Vec<(u32, DirRow)>) {
    if completed.is_empty() {
        return;
    }
    let mut rows = shared.rows.lock().expect("rows");
    for (slot, row) in completed.drain(..) {
        match &mut rows[slot as usize] {
            Some(existing) => existing.tally = row.tally,
            none => *none = Some(row),
        }
    }
}
