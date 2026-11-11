//! A report that names `volumes` never mounts what its walk did not list (review F1 on
//! #209).
//!
//! On Linux a capacity call on an unmounted autofs trigger asks the automounter to mount
//! it and waits for the answer, exactly as a listing would. The walk lists a trigger only
//! where its scope descends, so `volumes` may ask only there: a trigger kept at the scan
//! depth keeps its row, with capacity unknown.
//!
//! The test mounts a real direct trigger and answers its requests itself, refusing each
//! one so nothing is ever mounted, and counts them. Mounting needs root, so the test runs
//! only as root on a kernel with autofs, and otherwise says why it skipped: the
//! correctness runbook's root pass runs it, and CI, which runs unprivileged, cannot.
//!
//! The kernel never sends a request for an access by the automounter's own process
//! group, so the walk runs in a child process in a group of its own: this test binary
//! again, running only [`walk_in_a_child_process`].

#![cfg(target_os = "linux")]
// The trigger is mounted and answered through `mount` and `ioctl`, which the standard
// library does not wrap.
#![allow(unsafe_code)]

use std::ffi::CString;
use std::fs::File;
use std::io::Read as _;
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
/// `AUTOFS_IOC_FAIL`: refuse the request with this token.
const AUTOFS_IOC_FAIL: libc::c_ulong = 0x9361;
/// `AUTOFS_IOC_CATATONIC`: stop sending requests, so every later access fails at once.
const AUTOFS_IOC_CATATONIC: libc::c_ulong = 0x9362;
/// How long one child walk may take before the test fails rather than waits.
const CHILD_DEADLINE: Duration = Duration::from_secs(30);

/// A direct autofs trigger at `path` with this process as its automounter: each request
/// is counted and refused, so the trigger is never mounted.
struct Trigger {
    path: PathBuf,
    control: OwnedFd,
    requests: Arc<AtomicUsize>,
    answering: Option<std::thread::JoinHandle<()>>,
}

impl Trigger {
    /// Mount the trigger, or say why this host cannot.
    fn mount(path: &Path) -> Result<Self, String> {
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
        let options = format!("fd={},pgrp={group},minproto=5,maxproto=5,direct", write.as_raw_fd());
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
            let mut packets = File::from(read);
            std::thread::spawn(move || {
                let mut packet = [0_u8; 512];
                // Each read is one request; end of file is the kernel closing the pipe
                // when the trigger goes catatonic.
                while let Ok(length) = packets.read(&mut packet) {
                    let token_at = 8;
                    let token_width = std::mem::size_of::<libc::c_ulong>();
                    if length < token_at + token_width {
                        return;
                    }
                    let mut token = [0_u8; 8];
                    token[..token_width].copy_from_slice(&packet[token_at..token_at + token_width]);
                    let token = libc::c_ulong::try_from(u64::from_ne_bytes(token))
                        .expect("a token fits its own width");
                    requests.fetch_add(1, Ordering::SeqCst);
                    // SAFETY: `control` is the trigger's open descriptor, which outlives
                    // this thread, and the request takes the token by value.
                    unsafe { libc::ioctl(control, AUTOFS_IOC_FAIL, token) };
                }
            })
        };
        Ok(Self { path: path.to_path_buf(), control, requests, answering: Some(answering) })
    }

    fn requests(&self) -> usize {
        self.requests.load(Ordering::SeqCst)
    }
}

impl Drop for Trigger {
    fn drop(&mut self) {
        // SAFETY: `control` is the trigger's open descriptor.
        unsafe { libc::ioctl(self.control.as_raw_fd(), AUTOFS_IOC_CATATONIC, 0) };
        if let Some(answering) = self.answering.take() {
            let _ = answering.join();
        }
        let target = CString::new(self.path.as_os_str().as_bytes()).expect("target");
        // SAFETY: `target` is NUL-terminated and outlives the call.
        unsafe { libc::umount2(target.as_ptr(), libc::MNT_DETACH) };
    }
}

/// What a child walk prints before each volume row, which the test harness's own output
/// may share a line with.
const ROW: &str = "volume-row\t";

/// Walk `root` in a child process of its own process group, and return its volume rows,
/// each as its path and why its capacity is unknown (`known` when it is not).
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
    let request = Request::new(
        Basis::held_by(&index),
        Query { views: vec![ViewSpec::Volumes], ..Query::default() },
        SystemTime::now(),
    );
    let observed = fdu_core::query::observe_volumes(&[&index], &request);
    let report = fdu_core::query::report_with_capacity(
        &index,
        &request,
        SystemTime::now(),
        observed.as_ref(),
    )
    .expect("report");
    let Section::Volumes(volumes) = &report.sections[0] else { panic!("volumes") };
    for row in &volumes.rows {
        println!(
            "{ROW}{}\t{}",
            row.path.display(),
            row.unknown_reason.as_deref().unwrap_or("known")
        );
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
        ["\tknown", "trig\tthe walk did not list this directory"],
        "the root is asked, and the trigger keeps its row, capacity unknown, with the walk's \
         reason"
    );

    walk_in_a_child(base.path(), None);
    assert!(trigger.requests() > 0, "the control: a walk that lists the trigger asks for it");
    drop(trigger);
}
