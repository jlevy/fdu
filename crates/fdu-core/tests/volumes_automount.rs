//! A report that names `volumes` never mounts what its walk did not list (review F1 on
//! #209), or what has been unmounted since the walk listed it (reviews G1 and H1).
//!
//! On Linux a capacity call on an unmounted autofs trigger asks the automounter to mount
//! it and waits for the answer, exactly as a listing would, and so does any lookup of a path
//! through one, or of a name an indirect map does not hold. The walk lists a trigger only
//! where its scope descends, so `volumes` may ask only there: a trigger kept at the scan
//! depth keeps its row, with capacity unknown. And an automounter unmounts what it mounted
//! once it sits idle, so a report from an index walked earlier touches a path below its root
//! only where the mount table lists a mount there that is no automount trigger. A walk that
//! mounts a trigger reports it as one filesystem, the one its listing found (reviews G4 and
//! H4).
//!
//! The tests mount real triggers and answer their requests themselves, counting them: a
//! direct trigger that refuses each request, so nothing is ever mounted; one that mounts a
//! filesystem of its own, and one that mounts a second below it, and then expires them; and
//! an indirect map whose automounter makes and mounts a key's directory on request, as
//! automount(8) does, and removes it when the mount expires. One more needs no automounter:
//! debugfs holds `tracing`, a kernel automount point, as an NFS nested export is. Mounting
//! needs root, so the tests run only as root on a kernel with autofs, and otherwise say why
//! they skipped: the correctness runbook's root pass runs them, and CI, which runs
//! unprivileged, cannot.
//!
//! The kernel never sends a request for an access by the automounter's own process
//! group, so the walk runs in a child process in a group of its own: this test binary
//! again, running only [`walk_in_a_child_process`].

#![cfg(target_os = "linux")]
// The trigger is mounted and answered through `mount` and `ioctl`, which the standard
// library does not wrap.
#![allow(unsafe_code)]

use std::ffi::{CString, OsStr, OsString};
use std::fs::File;
use std::io::{BufRead as _, Read as _, Write as _};
use std::os::fd::{AsRawFd as _, FromRawFd as _, OwnedFd};
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::process::CommandExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant, SystemTime};

use fdu_core::query::{Basis, Query, Request, Section, ViewSpec};

/// The root the child process walks; set only for the child.
const CHILD_ROOT: &str = "FDU_TEST_AUTOMOUNT_CHILD_ROOT";
/// The child's scan depth, or `unbounded`.
const CHILD_DEPTH: &str = "FDU_TEST_AUTOMOUNT_CHILD_DEPTH";
/// Set when the child reports a second time from the index it walked, once its parent
/// says so on its standard input.
const CHILD_AGAIN: &str = "FDU_TEST_AUTOMOUNT_CHILD_AGAIN";
// The three requests are typed `libc::Ioctl`, the type `ioctl` takes on this target:
// `c_ulong` with glibc and `c_int` with musl, so the file builds for both.
/// `AUTOFS_IOC_READY`: the request with this token is answered; the mount is in place.
const AUTOFS_IOC_READY: libc::Ioctl = 0x9360;
/// `AUTOFS_IOC_FAIL`: refuse the request with this token.
const AUTOFS_IOC_FAIL: libc::Ioctl = 0x9361;
/// `AUTOFS_IOC_CATATONIC`: stop sending requests, so every later access fails at once.
const AUTOFS_IOC_CATATONIC: libc::Ioctl = 0x9362;
/// How long one child walk may take before the test fails rather than waits.
const CHILD_DEADLINE: Duration = Duration::from_secs(30);

/// How the automounter answers a request.
#[derive(Clone)]
enum Answer {
    /// Refuse it, so the trigger is never mounted.
    Refuse,
    /// Bind-mount this directory at the trigger, then say the mount is ready.
    Mount(PathBuf),
    /// Bind-mount this directory at the trigger and a tmpfs holding one file at its
    /// `inner`, as an NFS server's nested export is mounted below its parent's.
    MountNested(PathBuf),
    /// For an indirect map: make the named key's directory, bind-mount its directory there,
    /// and say the mount is ready; refuse a name it does not hold.
    MountKey(Vec<(OsString, PathBuf)>),
}

/// The byte offset of an autofs v5 packet's name length, after its header, wait-queue
/// token, device, inode, and four ids (`struct autofs_v5_packet` in `linux/auto_fs.h`).
const NAME_LENGTH_AT: usize = 40;

/// A direct autofs trigger at `path` with this process as its automounter: each request
/// is counted and answered as [`Answer`] says.
struct Trigger {
    path: PathBuf,
    control: OwnedFd,
    requests: Arc<AtomicUsize>,
    answering: Option<std::thread::JoinHandle<()>>,
}

/// `mount(2)` with NUL-terminated copies of its arguments.
fn mount(source: &str, target: &Path, kind: Option<&str>, flags: libc::c_ulong) -> i32 {
    let source = CString::new(source).expect("source");
    let target = CString::new(target.as_os_str().as_bytes()).expect("target");
    let kind = kind.map(|kind| CString::new(kind).expect("type"));
    // SAFETY: every argument is a NUL-terminated string that outlives the call, or null.
    unsafe {
        libc::mount(
            source.as_ptr(),
            target.as_ptr(),
            kind.as_ref().map_or(std::ptr::null(), |kind| kind.as_ptr()),
            flags,
            std::ptr::null(),
        )
    }
}

/// Unmount what is mounted at `path`, topmost first; `false` where nothing was.
fn unmount(path: &Path, flags: i32) -> bool {
    let target = CString::new(path.as_os_str().as_bytes()).expect("target");
    // SAFETY: `target` is NUL-terminated and outlives the call.
    unsafe { libc::umount2(target.as_ptr(), flags) == 0 }
}

impl Trigger {
    /// Mount the trigger, or say why this host cannot.
    fn mount(path: &Path) -> Result<Self, String> {
        Self::mount_answering(path, Answer::Refuse)
    }

    /// Mount the trigger with its requests answered as `answer` says.
    fn mount_answering(path: &Path, answer: Answer) -> Result<Self, String> {
        Self::mount_map(path, "direct", answer)
    }

    /// Mount an indirect map at `path`, each of whose `keys` the automounter mounts when it
    /// is asked for by name.
    fn mount_indirect(path: &Path, keys: Vec<(OsString, PathBuf)>) -> Result<Self, String> {
        Self::mount_map(path, "indirect", Answer::MountKey(keys))
    }

    /// Mount an autofs map of `kind`, `direct` or `indirect`, at `path`.
    fn mount_map(path: &Path, kind: &str, answer: Answer) -> Result<Self, String> {
        let mut pipe = [0; 2];
        // SAFETY: `pipe` is a valid two-element array the call fills on success.
        if unsafe { libc::pipe2(pipe.as_mut_ptr(), libc::O_CLOEXEC) } != 0 {
            return Err(format!("pipe2: {}", std::io::Error::last_os_error()));
        }
        // SAFETY: both descriptors were just created and are owned here alone.
        let (read, write) =
            unsafe { (OwnedFd::from_raw_fd(pipe[0]), OwnedFd::from_raw_fd(pipe[1])) };
        // SAFETY: `getpgrp` cannot fail.
        let group = unsafe { libc::getpgrp() };
        let options = format!("fd={},pgrp={group},minproto=5,maxproto=5,{kind}", write.as_raw_fd());
        let source = CString::new("fdu-test-trigger").expect("source");
        let target = CString::new(path.as_os_str().as_bytes()).expect("target");
        let kind = CString::new("autofs").expect("type");
        let options = CString::new(options).expect("options");
        // SAFETY: every argument is a NUL-terminated string that outlives the call.
        let mounted = unsafe {
            libc::mount(source.as_ptr(), target.as_ptr(), kind.as_ptr(), 0, options.as_ptr().cast())
        };
        if mounted != 0 {
            return Err(format!("mount autofs: {}", std::io::Error::last_os_error()));
        }
        // The kernel holds its own reference to the write end now.
        drop(write);
        // This process is the automounter's group, so opening the trigger asks nothing.
        let control = File::open(path).map_err(|error| format!("open trigger: {error}"))?;
        let control = OwnedFd::from(control);
        let requests = Arc::new(AtomicUsize::new(0));
        let answering = {
            let requests = Arc::clone(&requests);
            let control = control.as_raw_fd();
            let trigger = path.to_path_buf();
            let mut packets = File::from(read);
            std::thread::spawn(move || {
                let mut packet = [0_u8; 512];
                // Each read is one request; end of file is the kernel closing the pipe
                // when the trigger goes catatonic.
                while let Ok(length) = packets.read(&mut packet) {
                    // The packet's header is two `int`s, and its wait-queue token an
                    // `autofs_wqt_t`, which is 32 bits wide on every architecture but ia64
                    // and alpha (`linux/auto_fs.h`); the `__u32` device follows it. Read as
                    // a `c_ulong`, a 64-bit big-endian host would take the device for the
                    // token (review G8 on #209).
                    let token_at = 8;
                    let Some(token) = packet
                        .get(token_at..token_at + 4)
                        .filter(|_| length >= token_at + 4)
                        .and_then(|bytes| <[u8; 4]>::try_from(bytes).ok())
                    else {
                        return;
                    };
                    let token = libc::c_ulong::from(u32::from_ne_bytes(token));
                    requests.fetch_add(1, Ordering::SeqCst);
                    let answer = match &answer {
                        // This thread is in the automounter's group, so the bind mount
                        // resolves the trigger without asking itself again.
                        Answer::Mount(source) => ready(bind(source, &trigger)),
                        Answer::MountNested(source) => ready(
                            bind(source, &trigger)
                                && Tmpfs::mount(&trigger.join("inner"))
                                    .map(std::mem::forget)
                                    .is_ok()
                                && std::fs::write(
                                    trigger.join("inner").join("y.bin"),
                                    [b'y'; 3000],
                                )
                                .is_ok(),
                        ),
                        Answer::MountKey(keys) => {
                            let name = packet
                                .get(NAME_LENGTH_AT..NAME_LENGTH_AT + 4)
                                .and_then(|bytes| <[u8; 4]>::try_from(bytes).ok())
                                .and_then(|bytes| usize::try_from(u32::from_ne_bytes(bytes)).ok())
                                .and_then(|length| {
                                    packet.get(NAME_LENGTH_AT + 4..NAME_LENGTH_AT + 4 + length)
                                })
                                .map(OsStr::from_bytes);
                            let source = keys
                                .iter()
                                .find(|(key, _)| Some(key.as_os_str()) == name)
                                .map(|(_, source)| source);
                            ready(source.is_some_and(|source| {
                                let key = trigger.join(name.expect("a named key"));
                                let _ = std::fs::create_dir(&key);
                                bind(source, &key)
                            }))
                        }
                        Answer::Refuse => AUTOFS_IOC_FAIL,
                    };
                    // SAFETY: `control` is the trigger's open descriptor, which outlives
                    // this thread, and the request takes the token by value.
                    unsafe { libc::ioctl(control, answer, token) };
                }
            })
        };
        Ok(Self { path: path.to_path_buf(), control, requests, answering: Some(answering) })
    }

    fn requests(&self) -> usize {
        self.requests.load(Ordering::SeqCst)
    }

    /// Mount `source` at the key `name` of an indirect map, as its automounter would for a
    /// user already working there: this process is the automounter's group, so it makes the
    /// key's directory and mounts on it without a request.
    fn mount_key(&self, name: &str, source: &Path) {
        let key = self.path.join(name);
        std::fs::create_dir(&key).expect("make the key's directory");
        assert!(bind(source, &key), "mount the key");
    }

    /// Expire the key `name` of an indirect map as automount(8) does for a map that is not
    /// browsable: unmount it, and remove its directory.
    fn expire_key(&self, name: &str) {
        let key = self.path.join(name);
        assert!(unmount(&key, 0), "unmount the key");
        std::fs::remove_dir(&key).expect("remove the key's directory");
    }
}

/// Bind-mount `source` at `target`, privately, so that a mount below either does not
/// propagate to the other.
fn bind(source: &Path, target: &Path) -> bool {
    let source = source.to_str().expect("a UTF-8 scratch path");
    mount(source, target, None, libc::MS_BIND) == 0
        && mount("none", target, None, libc::MS_PRIVATE) == 0
}

/// The automounter's answer for a mount that was, or was not, made.
fn ready(mounted: bool) -> libc::Ioctl {
    if mounted { AUTOFS_IOC_READY } else { AUTOFS_IOC_FAIL }
}

/// Whether the mount table lists a mount at exactly `path`, read without touching it.
fn mounted_at(path: &Path) -> bool {
    let table = std::fs::read_to_string("/proc/self/mountinfo").expect("the mount table");
    let wanted = path.to_str().expect("a UTF-8 scratch path");
    table.lines().any(|line| line.split(' ').nth(4) == Some(wanted))
}

impl Drop for Trigger {
    fn drop(&mut self) {
        // SAFETY: `control` is the trigger's open descriptor.
        unsafe { libc::ioctl(self.control.as_raw_fd(), AUTOFS_IOC_CATATONIC, 0) };
        if let Some(answering) = self.answering.take() {
            let _ = answering.join();
        }
        // A filesystem the automounter mounted, if it is still there, then the trigger.
        while unmount(&self.path, libc::MNT_DETACH) {}
    }
}

/// A tmpfs mounted at a path for one test, unmounted when dropped.
struct Tmpfs(PathBuf);

impl Tmpfs {
    /// Mount a tmpfs at `path`, privately, so that a mount below a bind of it does not
    /// propagate back to it.
    fn mount(path: &Path) -> Result<Self, String> {
        if mount("fdu-test-tmpfs", path, Some("tmpfs"), 0) != 0 {
            return Err(format!("mount tmpfs: {}", std::io::Error::last_os_error()));
        }
        let mounted = Self(path.to_path_buf());
        if mount("none", path, None, libc::MS_PRIVATE) != 0 {
            return Err(format!("make the tmpfs private: {}", std::io::Error::last_os_error()));
        }
        Ok(mounted)
    }
}

impl Drop for Tmpfs {
    fn drop(&mut self) {
        unmount(&self.0, libc::MNT_DETACH);
    }
}

/// What a child walk prints before each volume row, which the test harness's own output
/// may share a line with.
const ROW: &str = "volume-row\t";

/// Walk `root` in a child process of its own process group, and return its volume rows,
/// each as its path, its counted files, and why its capacity is unknown (`known` when it is
/// not).
fn walk_in_a_child(root: &Path, depth: Option<usize>) -> Vec<String> {
    let mut child = Command::new(std::env::current_exe().expect("this test binary"))
        .args(["--exact", "walk_in_a_child_process", "--nocapture", "--test-threads", "1"])
        .env(CHILD_ROOT, root)
        .env(CHILD_DEPTH, depth.map_or_else(|| "unbounded".to_owned(), |depth| depth.to_string()))
        .process_group(0)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn the child walk");
    let started = Instant::now();
    loop {
        if child.try_wait().expect("child status").is_some() {
            break;
        }
        if started.elapsed() > CHILD_DEADLINE {
            let _ = child.kill();
            panic!("the child walk did not finish in {CHILD_DEADLINE:?}");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let output = child.wait_with_output().expect("child output");
    assert!(output.status.success(), "the child walk failed: {output:?}");
    String::from_utf8(output.stdout)
        .expect("utf-8")
        .lines()
        .filter_map(|line| line.find(ROW).map(|at| line[at + ROW.len()..].to_owned()))
        .collect()
}

/// What the second report of a child that reports twice prints before each volume row.
const AGAIN_ROW: &str = "volume-row-again\t";
/// What a child that reports twice prints once it has walked and reported the first time.
const WALKED: &str = "walked-and-reported";

/// Walk `root` in a child process of its own process group, report its volumes, then run
/// `between` and have the child report again from the index it walked: a route that reads
/// an index it did not just walk, as `--stale-ok`, a watch, and a retained `Index` do.
/// Returns each report's rows, as its path, its counted files, and why its capacity is
/// unknown (`known` when it is not).
fn walk_and_report_again(root: &Path, between: impl FnOnce()) -> (Vec<String>, Vec<String>) {
    let mut child = Command::new(std::env::current_exe().expect("this test binary"))
        .args(["--exact", "walk_in_a_child_process", "--nocapture", "--test-threads", "1"])
        .env(CHILD_ROOT, root)
        .env(CHILD_DEPTH, "unbounded")
        .env(CHILD_AGAIN, "1")
        .process_group(0)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn the child walk");
    let mut lines = std::io::BufReader::new(child.stdout.take().expect("stdout")).lines();
    let mut first = Vec::new();
    let started = Instant::now();
    loop {
        let line = lines.next().expect("the child walk ended before it reported").expect("line");
        if line.contains(WALKED) {
            break;
        }
        if let Some(at) = line.find(ROW) {
            first.push(line[at + ROW.len()..].to_owned());
        }
        assert!(started.elapsed() < CHILD_DEADLINE, "the child walk did not report in time");
    }
    between();
    let mut stdin = child.stdin.take().expect("stdin");
    writeln!(stdin, "again").expect("tell the child to report again");
    drop(stdin);
    let reader = std::thread::spawn(move || {
        lines
            .map_while(Result::ok)
            .filter_map(|line| {
                line.find(AGAIN_ROW).map(|at| line[at + AGAIN_ROW.len()..].to_owned())
            })
            .collect::<Vec<_>>()
    });
    let started = Instant::now();
    while child.try_wait().expect("child status").is_none() {
        if started.elapsed() > CHILD_DEADLINE {
            let _ = child.kill();
            panic!("the child's second report did not finish in {CHILD_DEADLINE:?}");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(child.wait().expect("child status").success(), "the child walk failed");
    (first, reader.join().expect("the child's output"))
}

/// The walk this test runs in a child process; outside one it does nothing.
#[test]
fn walk_in_a_child_process() {
    let Some(root) = std::env::var_os(CHILD_ROOT) else { return };
    let depth = std::env::var(CHILD_DEPTH).expect("depth");
    let config = fdu_core::ScanConfig {
        max_depth: (depth != "unbounded").then(|| depth.parse().expect("a depth")),
        ..fdu_core::ScanConfig::default()
    };
    // An unbounded walk lists the trigger, which fails when its mount is refused.
    let index = match fdu_core::scan::scan_into_index(Path::new(&root), &config) {
        Ok((index, _)) => index,
        Err(error) => panic!("walk: {error}"),
    };
    let report_volumes = |prefix: &str| {
        let request = Request::new(
            Basis::held_by(&index),
            Query { views: vec![ViewSpec::Volumes], ..Query::default() },
            SystemTime::now(),
        );
        let observed = fdu_core::query::observe_volumes(&[&index], &request);
        let report =
            fdu_core::query::report(&index, &request, SystemTime::now(), observed.as_ref())
                .expect("report");
        let Section::Volumes(volumes) = &report.sections[0] else { panic!("volumes") };
        for row in &volumes.rows {
            println!(
                "{prefix}{}\t{}\t{}",
                row.path.display(),
                row.counted.files,
                row.unknown_reason.as_deref().unwrap_or("known")
            );
        }
    };
    report_volumes(ROW);
    if std::env::var_os(CHILD_AGAIN).is_some() {
        println!("{WALKED}");
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).expect("the parent's word");
        report_volumes(AGAIN_ROW);
    }
}

/// A trigger kept at the scan depth is never asked for its capacity, so it is never
/// mounted; an unbounded walk lists it, and is the control that shows a request would
/// have been seen.
#[test]
fn a_trigger_below_the_scan_depth_is_never_mounted_by_volumes() {
    if std::env::var_os(CHILD_ROOT).is_some() {
        return;
    }
    // SAFETY: `geteuid` cannot fail.
    if unsafe { libc::geteuid() } != 0 {
        eprintln!("skipped: mounting an autofs trigger needs root");
        return;
    }
    let base = tempfile::tempdir().expect("base");
    let trigger_path = base.path().join("trig");
    std::fs::create_dir(&trigger_path).expect("trigger directory");
    std::fs::write(base.path().join("a.txt"), b"some bytes").expect("file");
    let trigger = match Trigger::mount(&trigger_path) {
        Ok(trigger) => trigger,
        Err(reason) => {
            eprintln!("skipped: this host cannot mount an autofs trigger ({reason})");
            return;
        }
    };

    let rows = walk_in_a_child(base.path(), Some(1));
    assert_eq!(trigger.requests(), 0, "a scan-depth walk with volumes mounted the trigger");
    assert_eq!(
        rows,
        ["\t1\tknown", "trig\t0\tthe walk did not list this directory"],
        "the root is asked, and the trigger keeps its row, capacity unknown, with the walk's \
         reason"
    );

    walk_in_a_child(base.path(), None);
    assert!(trigger.requests() > 0, "the control: a walk that lists the trigger asks for it");
    drop(trigger);
}

/// A walk that mounts a trigger reports the filesystem it mounted as one row, the one its
/// listing found (review G4 on #209). And once the automounter has unmounted it again, as
/// it does after an idle timeout, a report from the index walked earlier never mounts it
/// again: the trigger keeps its row, with the counts the walk made and capacity unknown,
/// and the automounter sees no request (review G1).
#[test]
fn an_automount_that_expired_since_the_walk_is_never_mounted_again_by_volumes() {
    if std::env::var_os(CHILD_ROOT).is_some() {
        return;
    }
    // SAFETY: `geteuid` cannot fail.
    if unsafe { libc::geteuid() } != 0 {
        eprintln!("skipped: mounting an autofs trigger needs root");
        return;
    }
    let base = tempfile::tempdir().expect("base");
    let root = base.path().join("root");
    let trigger_path = root.join("data").join("trig");
    std::fs::create_dir_all(&trigger_path).expect("trigger directory");
    std::fs::write(root.join("a.txt"), b"some bytes").expect("file");
    // The filesystem the automounter mounts, with files at its top and in two directories.
    let source = base.path().join("src");
    std::fs::create_dir(&source).expect("source directory");
    let filesystem = match Tmpfs::mount(&source) {
        Ok(filesystem) => filesystem,
        Err(reason) => {
            eprintln!("skipped: this host cannot mount a tmpfs ({reason})");
            return;
        }
    };
    for (name, size) in [("top.bin", 5000), ("sub1/one.bin", 7000), ("sub2/two.bin", 9000)] {
        let path = source.join(name);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("directory");
        std::fs::write(path, vec![b'x'; size]).expect("file");
    }
    let trigger = match Trigger::mount_answering(&trigger_path, Answer::Mount(source.clone())) {
        Ok(trigger) => trigger,
        Err(reason) => {
            eprintln!("skipped: this host cannot mount an autofs trigger ({reason})");
            return;
        }
    };

    let trig = ["data", "trig"].iter().collect::<PathBuf>().display().to_string();
    let mut after_walk = 0;
    let (walked, again) = walk_and_report_again(&root, || {
        after_walk = trigger.requests();
        // The idle timeout passes: the automounter unmounts what it mounted, and the
        // trigger is left in its place.
        assert!(unmount(&trigger_path, 0), "expire the automount");
    });
    assert_eq!(after_walk, 1, "the control: the walk's listing mounted the trigger");
    assert_eq!(
        walked,
        ["\t1\tknown".to_owned(), format!("{trig}\t3\tknown")],
        "one row for the filesystem the walk mounted, keyed by it and counting all its files"
    );
    assert_eq!(trigger.requests(), 1, "a report from the retained index mounted the trigger");
    assert_eq!(
        again,
        ["\t1\tknown".to_owned(), format!("{trig}\t3\t{AUTOFS_TOPMOST}")],
        "the trigger keeps its row and its counts, with capacity unknown and the reason"
    );
    drop(trigger);
    drop(filesystem);
}

/// Why a row's capacity is unknown where autofs is topmost at its path.
const AUTOFS_TOPMOST: &str = "an automount point (autofs) is topmost here, so it was not asked";
/// Why a row's capacity is unknown where the mount table lists no mount at its path.
const NOT_A_MOUNT_POINT: &str = "not a mount point now, so it was not asked";

/// Whether this process can mount, or else the reason the test skipped, printed.
fn can_mount() -> bool {
    // SAFETY: `geteuid` cannot fail.
    let root = unsafe { libc::geteuid() } == 0;
    if !root {
        eprintln!("skipped: mounting needs root");
    }
    root
}

/// A mount made for one test, detached when dropped.
struct Mounted(PathBuf);

impl Drop for Mounted {
    fn drop(&mut self) {
        while unmount(&self.0, libc::MNT_DETACH) {}
    }
}

/// A tmpfs at `path` holding `files`, each a relative path and a size, or the reason the host
/// cannot mount one.
fn filled_tmpfs(path: &Path, files: &[(&str, usize)]) -> Result<Tmpfs, String> {
    std::fs::create_dir_all(path).expect("source directory");
    let filesystem = Tmpfs::mount(path)?;
    for (name, size) in files {
        let file = path.join(name);
        std::fs::create_dir_all(file.parent().expect("parent")).expect("directory");
        std::fs::write(file, vec![b'x'; *size]).expect("file");
    }
    Ok(filesystem)
}

/// A key of an indirect map, the usual way `/home` and `/net` are automounted, whose mount
/// has expired is gone from its map: automount(8) removed its directory. A lookup of the
/// missing name asks the automounter, a stat that passes `AT_NO_AUTOMOUNT` included, so a
/// report from an index walked earlier touches the key only where the mount table lists a
/// mount at its path (review H1 on #209). It keeps its row and its counts.
#[test]
fn an_expired_key_of_an_indirect_map_is_never_mounted_again_by_volumes() {
    if std::env::var_os(CHILD_ROOT).is_some() || !can_mount() {
        return;
    }
    let base = tempfile::tempdir().expect("base");
    let root = base.path().join("root");
    let auto = root.join("auto");
    std::fs::create_dir_all(&auto).expect("map directory");
    let source = base.path().join("src");
    let files = [("top.bin", 5000), ("sub1/one.bin", 7000), ("sub2/two.bin", 9000)];
    let filesystem = match filled_tmpfs(&source, &files) {
        Ok(filesystem) => filesystem,
        Err(reason) => {
            eprintln!("skipped: this host cannot mount a tmpfs ({reason})");
            return;
        }
    };
    let map = match Trigger::mount_indirect(&auto, vec![("alice".into(), source.clone())]) {
        Ok(map) => map,
        Err(reason) => {
            eprintln!("skipped: this host cannot mount an autofs map ({reason})");
            return;
        }
    };
    // A user is working in the key, so it was mounted before the walk.
    map.mount_key("alice", &source);

    let alice = ["auto", "alice"].iter().collect::<PathBuf>().display().to_string();
    let mut after_walk = usize::MAX;
    let (walked, again) = walk_and_report_again(&root, || {
        after_walk = map.requests();
        // The idle timeout passes: the key is unmounted and its directory removed.
        map.expire_key("alice");
    });
    assert_eq!(after_walk, 0, "the key was mounted already, and the walk asked for nothing");
    assert_eq!(map.requests(), 0, "a report from the retained index asked for the expired key");
    assert_eq!(
        walked,
        [
            "\t0\tknown".to_owned(),
            format!("auto\t0\t{AUTOFS_TOPMOST}"),
            format!("{alice}\t3\tknown")
        ]
    );
    assert_eq!(
        again,
        [
            "\t0\tknown".to_owned(),
            format!("auto\t0\t{AUTOFS_TOPMOST}"),
            format!("{alice}\t3\t{NOT_A_MOUNT_POINT}"),
        ],
        "the key keeps its row and its counts, with capacity unknown and the reason"
    );
    drop(map);
    drop(filesystem);
}

/// A report rooted at an indirect map, as `fdu /home` is, asks the map itself, the root the
/// user named, and none of the keys that have expired since its walk: each was mounted
/// again before the mount table was read (review H1 on #209).
#[test]
fn a_report_rooted_at_an_indirect_map_mounts_none_of_its_expired_keys() {
    if std::env::var_os(CHILD_ROOT).is_some() || !can_mount() {
        return;
    }
    let base = tempfile::tempdir().expect("base");
    let home = base.path().join("home");
    std::fs::create_dir(&home).expect("map directory");
    let keys = ["alice", "bob", "carol"];
    let mut sources = Vec::new();
    for key in keys {
        let source = base.path().join(format!("src-{key}"));
        match filled_tmpfs(&source, &[("top.bin", 1000), ("work/w.bin", 10_000)]) {
            Ok(filesystem) => sources.push((key, source, filesystem)),
            Err(reason) => {
                eprintln!("skipped: this host cannot mount a tmpfs ({reason})");
                return;
            }
        }
    }
    let answers = sources.iter().map(|(key, source, _)| ((*key).into(), source.clone())).collect();
    let map = match Trigger::mount_indirect(&home, answers) {
        Ok(map) => map,
        Err(reason) => {
            eprintln!("skipped: this host cannot mount an autofs map ({reason})");
            return;
        }
    };
    // Every user is logged in, so every key was mounted before the walk.
    for (key, source, _) in &sources {
        map.mount_key(key, source);
    }

    let mut after_walk = usize::MAX;
    let (walked, again) = walk_and_report_again(&home, || {
        after_walk = map.requests();
        for key in keys {
            map.expire_key(key);
        }
    });
    assert_eq!(after_walk, 0, "every key was mounted already, and the walk asked for nothing");
    assert_eq!(map.requests(), 0, "a report from the retained index asked for an expired key");
    let rows = |reason: &str| {
        std::iter::once("\t0\tknown".to_owned())
            .chain(keys.iter().map(|key| format!("{key}\t2\t{reason}")))
            .collect::<Vec<_>>()
    };
    assert_eq!(walked, rows("known"));
    assert_eq!(again, rows(NOT_A_MOUNT_POINT), "the map is asked, and no key is touched");
    drop(map);
}

/// debugfs holds `tracing`, a kernel automount point: a directory on debugfs's device until
/// something resolves through it, as an NFS nested export or referral is on its parent's.
/// A walk lists it, which mounts tracefs there, and records it as its listing finds it, so
/// two cold walks agree whether it was mounted before (review H4 on #209). And once it is
/// unmounted again, a report from the index walked first never mounts it: the mount table
/// lists nothing at its path (review H1). No automounter is needed.
#[test]
fn a_kernel_automount_is_counted_as_its_listing_finds_it_and_never_mounted_again() {
    if std::env::var_os(CHILD_ROOT).is_some() || !can_mount() {
        return;
    }
    let base = tempfile::tempdir().expect("base");
    let root = base.path().join("root");
    let dbg = root.join("dbg");
    std::fs::create_dir_all(&dbg).expect("debugfs directory");
    if mount("none", &dbg, Some("debugfs"), 0) != 0 {
        eprintln!("skipped: this host cannot mount debugfs ({})", std::io::Error::last_os_error());
        return;
    }
    let _debugfs = Mounted(dbg.clone());
    let tracing = dbg.join("tracing");
    // Listed, never stated: a stat without `AT_NO_AUTOMOUNT` would mount it.
    let listed = std::fs::read_dir(&dbg)
        .expect("list debugfs")
        .any(|entry| entry.is_ok_and(|entry| entry.file_name() == "tracing"));
    if !listed || mounted_at(&tracing) {
        eprintln!("skipped: this host's debugfs holds no unmounted tracing automount point");
        return;
    }
    let _tracefs = Mounted(tracing.clone());

    let rows = |index: &fdu_core::Index| -> Vec<(String, usize, u64, String)> {
        let request = Request::new(
            Basis::held_by(index),
            Query { views: vec![ViewSpec::Volumes], ..Query::default() },
            SystemTime::now(),
        );
        let observed = fdu_core::query::observe_volumes(&[index], &request);
        let report = fdu_core::query::report(index, &request, SystemTime::now(), observed.as_ref())
            .expect("report");
        let Section::Volumes(volumes) = &report.sections[0] else { panic!("volumes") };
        volumes
            .rows
            .iter()
            .map(|row| {
                (
                    row.path.display().to_string(),
                    row.also_entered.len(),
                    row.counted.files,
                    row.unknown_reason.clone().unwrap_or_else(|| "known".to_owned()),
                )
            })
            .collect()
    };
    let config = fdu_core::ScanConfig::default();
    let (first, _) = fdu_core::scan::scan_into_index(&root, &config).expect("first walk");
    assert!(mounted_at(&tracing), "the control: the walk's listing mounted the automount point");
    let walked_first = rows(&first);
    let (second, _) = fdu_core::scan::scan_into_index(&root, &config).expect("second walk");
    let walked_second = rows(&second);
    let trace = ["dbg", "tracing"].iter().collect::<PathBuf>().display().to_string();
    assert_eq!(
        walked_first.iter().map(|row| (row.0.as_str(), row.1)).collect::<Vec<_>>(),
        [("", 0), ("dbg", 0), (trace.as_str(), 0)],
        "one row for tracefs, where the listing found it: {walked_first:?}"
    );
    assert_eq!(walked_first, walked_second, "two cold walks over the same tree agree");

    // Unmounted again, as an NFS submount expires.
    assert!(unmount(&tracing, 0), "unmount tracefs");
    let again = rows(&first);
    assert!(!mounted_at(&tracing), "a report from the retained index mounted it again");
    assert_eq!(
        again.last().map(|row| (row.0.as_str(), row.3.as_str())),
        Some((trace.as_str(), NOT_A_MOUNT_POINT)),
        "{again:?}"
    );
}

/// Every entry is checked, also one whose filesystem was already asked at an earlier path.
/// A filesystem mounted plainly at `data/m1` and automounted at `data/m2`, with a second
/// mounted below the trigger as an NFS server's nested export is: skipped, `data/m2` was
/// never found expired, and `data/m2/inner` was read through it, which mounted it again
/// (review H1 case 3 on #209). Checked, the trigger is topmost again, and the mount table
/// lists nothing below it, so nothing there is touched.
#[test]
fn an_entry_whose_filesystem_was_asked_elsewhere_is_still_checked() {
    if std::env::var_os(CHILD_ROOT).is_some() || !can_mount() {
        return;
    }
    let base = tempfile::tempdir().expect("base");
    let root = base.path().join("root");
    let (plain, trigger_path) = (root.join("data").join("m1"), root.join("data").join("m2"));
    std::fs::create_dir_all(&plain).expect("plain mount point");
    std::fs::create_dir_all(&trigger_path).expect("trigger directory");
    let source = base.path().join("src");
    let filesystem = match filled_tmpfs(&source, &[("top.bin", 5000)]) {
        Ok(filesystem) => filesystem,
        Err(reason) => {
            eprintln!("skipped: this host cannot mount a tmpfs ({reason})");
            return;
        }
    };
    std::fs::create_dir(source.join("inner")).expect("inner directory");
    assert!(bind(&source, &plain), "mount the filesystem plainly");
    let _plain = Mounted(plain.clone());
    let trigger = match Trigger::mount_answering(&trigger_path, Answer::MountNested(source.clone()))
    {
        Ok(trigger) => trigger,
        Err(reason) => {
            eprintln!("skipped: this host cannot mount an autofs trigger ({reason})");
            return;
        }
    };

    let at = |parts: &[&str]| parts.iter().collect::<PathBuf>().display().to_string();
    let mut after_walk = 0;
    let (walked, again) = walk_and_report_again(&root, || {
        after_walk = trigger.requests();
        // The idle timeout passes: both mounts expire, and the trigger is left in place.
        assert!(unmount(&trigger_path.join("inner"), 0), "expire the nested mount");
        assert!(unmount(&trigger_path, 0), "expire the automount");
    });
    assert_eq!(after_walk, 1, "the control: the walk's listing mounted the trigger");
    assert_eq!(trigger.requests(), 1, "a report read below the expired trigger and mounted it");
    let (m1, inner) = (at(&["data", "m1"]), at(&["data", "m2", "inner"]));
    assert_eq!(
        walked,
        ["\t0\tknown".to_owned(), format!("{m1}\t2\tknown"), format!("{inner}\t1\tknown")]
    );
    assert_eq!(
        again,
        [
            "\t0\tknown".to_owned(),
            format!("{m1}\t2\tknown"),
            format!("{inner}\t1\t{NOT_A_MOUNT_POINT}")
        ],
        "the filesystem is asked where it is plainly mounted, and nothing below the trigger is read"
    );
    drop(trigger);
    drop(filesystem);
}
